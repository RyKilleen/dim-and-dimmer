use std::collections::HashMap;

use crate::gamma::GammaBackend;

pub struct WaylandBackend;

impl WaylandBackend {
    pub fn new() -> Option<Self> {
        // Wayland gamma control requires compositor-specific protocols
        // (wlr-gamma-control-unstable-v1 or similar)
        // For now, return None as this is not yet implemented
        None
    }
}

impl GammaBackend for WaylandBackend {
    fn name(&self) -> &'static str {
        "Wayland"
    }

    fn enumerate_outputs(&self) -> Result<HashMap<String, String>, String> {
        Err("Wayland gamma control not yet implemented".to_string())
    }

    fn apply_dimming(&self, _output: &str, _value: u8) -> Result<(), String> {
        Err("Wayland gamma control not yet implemented".to_string())
    }

    fn reset(&self, _output: &str) -> Result<(), String> {
        Err("Wayland gamma control not yet implemented".to_string())
    }
}
