use std::ops::RangeInclusive;

use eframe::egui;

use crate::app::App;
use crate::ddc::{set_vcp, VCP_BRIGHTNESS, VCP_CONTRAST};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Monitor Control (Linux / ddcutil)");
            ui.add_space(10.0);

            if !self.ddcutil_available {
                ui.colored_label(egui::Color32::RED, "ddcutil not installed");
                return;
            }

            if self.displays.is_empty() {
                ui.colored_label(egui::Color32::YELLOW, "No displays detected");
                return;
            }

            display_selector(self, ui);
            ui.separator();

            if let Some(v) = slider(ui, "Brightness", &mut self.brightness, 1..=100) {
                match set_vcp(self.display_id(), VCP_BRIGHTNESS, v) {
                    Ok(_) => {
                        self.status = Some(format!("Set brightness to {}%", v));
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to set brightness: {}", e));
                    }
                }
            }

            if let Some(v) = slider(ui, "Contrast", &mut self.contrast, 1..=100) {
                match set_vcp(self.display_id(), VCP_CONTRAST, v) {
                    Ok(_) => {
                        self.status = Some(format!("Set contrast to {}%", v));
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to set contrast: {}", e));
                    }
                }
            }

            if self.gamma_available() {
                ui.separator();

                if let Some(v) = slider(ui, "Software Dimming", &mut self.gamma_dimming, 20..=100) {
                    if let (Some(backend), Some(output)) = (&self.gamma_backend, self.gamma_output()) {
                        match backend.apply_dimming(output, v) {
                            Ok(_) => {
                                self.gamma_dimming = v;
                                if v < 100 {
                                    self.status = Some(format!("Set software dimming to {}%", v));
                                } else {
                                    self.status = Some("Software dimming removed".into());
                                }
                                self.error = None;
                            }
                            Err(e) => {
                                self.error = Some(format!("Gamma error: {}", e));
                            }
                        }
                    }
                }

                if ui.button("Reset Gamma").clicked() {
                    if let (Some(backend), Some(output)) = (&self.gamma_backend, self.gamma_output()) {
                        if backend.reset(output).is_ok() {
                            self.gamma_dimming = 100;
                            self.status = Some("Reset gamma to normal".into());
                        }
                    }
                }
            }

            ui.separator();
            messages(self, ui);
        });
    }
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
