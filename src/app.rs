use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use eframe::egui;

use crate::commands::command_exists;
use crate::ddc::{VCP_BRIGHTNESS, VCP_CONTRAST, read_vcp};
use crate::display::{Display, enumerate_displays};
use crate::gamma::{self, GammaBackend, map_drm_to_xrandr};

pub enum InitState {
    Pending, // Not started yet (no ctx available)
    Loading, // Thread spawned, waiting for result
    Ready,
    Failed(String),
}

#[derive(Default)]
pub struct InitResult {
    pub ddcutil_available: bool,
    pub displays: Vec<Display>,
    pub gamma_backend: Option<Box<dyn GammaBackend>>,
    pub gamma_output_map: HashMap<String, String>,
    pub brightness: u8,
    pub contrast: u8,
    pub error: Option<String>,
}

pub struct App {
    pub displays: Vec<Display>,
    pub selected_display: usize,

    pub brightness: u8,
    pub contrast: u8,
    pub gamma_dimming: u8,

    pub ddcutil_available: bool,
    pub gamma_backend: Option<Box<dyn GammaBackend>>,
    pub gamma_output_map: HashMap<String, String>,

    pub status: Option<String>,
    pub error: Option<String>,

    pub init_state: InitState,
    pub init_receiver: Option<Receiver<InitResult>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            displays: Vec::new(),
            selected_display: 0,
            brightness: 50,
            contrast: 50,
            gamma_dimming: 100,
            ddcutil_available: false,
            gamma_backend: None,
            gamma_output_map: HashMap::new(),
            status: None,
            error: None,
            init_state: InitState::Pending,
            init_receiver: None,
        }
    }

    pub fn start_init(&mut self, ctx: egui::Context) {
        let (tx, rx) = mpsc::channel();
        self.init_receiver = Some(rx);
        self.init_state = InitState::Loading;

        thread::spawn(move || {
            let result = Self::init_blocking();
            let _ = tx.send(result);
            ctx.request_repaint(); // Wake UI exactly once
        });
    }

    fn init_blocking() -> InitResult {
        let mut result = InitResult::default();

        if !command_exists("ddcutil", &["--version"]) {
            result.error = Some("ddcutil not found. Install with: sudo apt install ddcutil".into());
            return result;
        }

        result.ddcutil_available = true;
        result.gamma_backend = gamma::create_backend();
        result.displays = enumerate_displays();

        // Build gamma output map
        if let Some(ref backend) = result.gamma_backend {
            if let Ok(xrandr_outputs) = backend.enumerate_outputs() {
                for display in &result.displays {
                    if let Some(xrandr_name) =
                        map_drm_to_xrandr(&display.drm_connector, &xrandr_outputs)
                    {
                        result
                            .gamma_output_map
                            .insert(display.id.clone(), xrandr_name);
                    }
                }
            }
        }

        if result.displays.is_empty() {
            result.error = Some("No displays found. Try running: sudo ddcutil detect".into());
        } else {
            // Read initial values from first display
            let display_id = &result.displays[0].id;
            if let Some(v) = read_vcp(display_id, VCP_BRIGHTNESS) {
                result.brightness = v;
            }
            if let Some(v) = read_vcp(display_id, VCP_CONTRAST) {
                result.contrast = v;
            }
        }

        result
    }

    pub fn display_id(&self) -> &str {
        &self.displays[self.selected_display].id
    }

    pub fn gamma_output(&self) -> Option<&str> {
        self.gamma_output_map
            .get(self.display_id())
            .map(|s| s.as_str())
    }

    pub fn gamma_available(&self) -> bool {
        self.gamma_backend.is_some() && self.gamma_output().is_some()
    }

    pub fn refresh_values(&mut self) {
        if let Some(v) = read_vcp(self.display_id(), VCP_BRIGHTNESS) {
            self.brightness = v;
        }
        if let Some(v) = read_vcp(self.display_id(), VCP_CONTRAST) {
            self.contrast = v;
        }

        self.status = Some("Values refreshed from monitor".into());
    }
}
