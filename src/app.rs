use std::collections::HashMap;

use crate::commands::command_exists;
use crate::ddc::{read_vcp, VCP_BRIGHTNESS, VCP_CONTRAST};
use crate::display::{enumerate_displays, Display};
use crate::gamma::{self, GammaBackend, map_drm_to_xrandr};

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
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
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
        };

        if !command_exists("ddcutil", &["--version"]) {
            app.error = Some("ddcutil not found. Install with: sudo apt install ddcutil".into());
            return app;
        }

        app.ddcutil_available = true;
        app.gamma_backend = gamma::create_backend();

        app.displays = enumerate_displays();

        app.build_gamma_output_map();

        if app.displays.is_empty() {
            app.error = Some("No displays found. Try running: sudo ddcutil detect".into());
        } else {
            app.refresh_values();
        }

        app
    }

    pub fn display_id(&self) -> &str {
        &self.displays[self.selected_display].id
    }

    pub fn gamma_output(&self) -> Option<&str> {
        self.gamma_output_map.get(self.display_id()).map(|s| s.as_str())
    }

    pub fn gamma_available(&self) -> bool {
        self.gamma_backend.is_some() && self.gamma_output().is_some()
    }

    fn build_gamma_output_map(&mut self) {
        let Some(ref backend) = self.gamma_backend else { return };
        let Ok(xrandr_outputs) = backend.enumerate_outputs() else { return };

        for display in &self.displays {
            if let Some(xrandr_name) = map_drm_to_xrandr(&display.drm_connector, &xrandr_outputs) {
                self.gamma_output_map.insert(display.id.clone(), xrandr_name);
            }
        }
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
