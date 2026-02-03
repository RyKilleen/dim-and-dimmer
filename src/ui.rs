use std::ops::RangeInclusive;

use eframe::egui;

use crate::app::{App, InitState};
use crate::ddc::{set_vcp, VCP_BRIGHTNESS, VCP_CONTRAST};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Start init on first frame
        if matches!(self.init_state, InitState::Pending) {
            self.start_init(ctx.clone());
        }

        // Receive result (only succeeds once, after thread's request_repaint)
        if let Some(rx) = self.init_receiver.take() {
            match rx.try_recv() {
                Ok(result) => {
                    self.ddcutil_available = result.ddcutil_available;
                    self.displays = result.displays;
                    self.gamma_backend = result.gamma_backend;
                    self.gamma_output_map = result.gamma_output_map;
                    self.brightness = result.brightness;
                    self.contrast = result.contrast;

                    if let Some(err) = result.error {
                        self.init_state = InitState::Failed(err);
                    } else {
                        self.init_state = InitState::Ready;
                        self.status = Some("Ready".into());
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    self.init_receiver = Some(rx); // Put it back
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.init_state = InitState::Failed("Initialization thread crashed".into());
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Monitor Control (Linux / ddcutil)");
            ui.add_space(10.0);

            match &self.init_state {
                InitState::Pending | InitState::Loading => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Detecting displays...");
                    });
                    // No request_repaint() here - thread will wake us
                }
                InitState::Failed(err) => {
                    ui.colored_label(egui::Color32::RED, format!("Initialization failed: {}", err));
                }
                InitState::Ready => {
                    show_main_ui(self, ui);
                }
            }
        });
    }
}

fn show_main_ui(app: &mut App, ui: &mut egui::Ui) {
    if !app.ddcutil_available {
        ui.colored_label(egui::Color32::RED, "ddcutil not installed");
        return;
    }

    if app.displays.is_empty() {
        ui.colored_label(egui::Color32::YELLOW, "No displays detected");
        return;
    }

    display_selector(app, ui);
    ui.separator();

    if let Some(v) = slider(ui, "Brightness", &mut app.brightness, 1..=100) {
        match set_vcp(app.display_id(), VCP_BRIGHTNESS, v) {
            Ok(_) => {
                app.status = Some(format!("Set brightness to {}%", v));
                app.error = None;
            }
            Err(e) => {
                app.error = Some(format!("Failed to set brightness: {}", e));
            }
        }
    }

    if let Some(v) = slider(ui, "Contrast", &mut app.contrast, 1..=100) {
        match set_vcp(app.display_id(), VCP_CONTRAST, v) {
            Ok(_) => {
                app.status = Some(format!("Set contrast to {}%", v));
                app.error = None;
            }
            Err(e) => {
                app.error = Some(format!("Failed to set contrast: {}", e));
            }
        }
    }

    if app.gamma_available() {
        ui.separator();

        if let Some(v) = slider(ui, "Software Dimming", &mut app.gamma_dimming, 20..=100) {
            if let (Some(backend), Some(output)) = (&app.gamma_backend, app.gamma_output()) {
                match backend.apply_dimming(output, v) {
                    Ok(_) => {
                        app.gamma_dimming = v;
                        if v < 100 {
                            app.status = Some(format!("Set software dimming to {}%", v));
                        } else {
                            app.status = Some("Software dimming removed".into());
                        }
                        app.error = None;
                    }
                    Err(e) => {
                        app.error = Some(format!("Gamma error: {}", e));
                    }
                }
            }
        }

        if ui.button("Reset Gamma").clicked() {
            if let (Some(backend), Some(output)) = (&app.gamma_backend, app.gamma_output()) {
                if backend.reset(output).is_ok() {
                    app.gamma_dimming = 100;
                    app.status = Some("Reset gamma to normal".into());
                }
            }
        }
    }

    ui.separator();
    messages(app, ui);
}

fn slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut u8,
    range: RangeInclusive<u8>,
) -> Option<u8> {
    ui.label(label);

    let mut temp = *value;
    let response = ui.add(egui::Slider::new(&mut temp, range).suffix("%"));

    if response.changed() {
        *value = temp;
    }

    ui.add_space(10.0);

    if response.drag_stopped() {
        Some(temp)
    } else {
        None
    }
}

fn display_selector(app: &mut App, ui: &mut egui::Ui) {
    let mut refresh = false;

    ui.horizontal(|ui| {
        egui::ComboBox::from_label("Display")
            .selected_text(&app.displays[app.selected_display].name)
            .show_ui(ui, |ui| {
                for (i, display) in app.displays.iter().enumerate() {
                    if ui
                        .selectable_value(&mut app.selected_display, i, &display.name)
                        .clicked()
                    {
                        refresh = true;
                    }
                }
            });

        refresh |= ui.button("Refresh").clicked();
    });

    if refresh {
        app.refresh_values();
    }
}

fn messages(app: &App, ui: &mut egui::Ui) {
    if let Some(status) = &app.status {
        ui.colored_label(egui::Color32::GREEN, format!("[OK] {}", status));
    }
    if let Some(error) = &app.error {
        ui.colored_label(egui::Color32::RED, format!("[Error] {}", error));
    }
}
