use std::collections::HashMap;

use crate::commands::{command_exists, run, run_capture};
use crate::gamma::GammaBackend;

pub struct X11Backend;

impl X11Backend {
    pub fn new() -> Option<Self> {
        if command_exists("xrandr", &["--version"]) {
            Some(Self)
        } else {
            None
        }
    }
}

impl GammaBackend for X11Backend {
    fn name(&self) -> &'static str {
        "X11 (xrandr)"
    }

    fn enumerate_outputs(&self) -> Result<HashMap<String, String>, String> {
        let output = run_capture("xrandr", &["--query"])?;
        let mut outputs = HashMap::new();

        for line in output.lines() {
            // Lines like: "DP-0 connected primary 2560x1440+0+0 ..."
            // or: "HDMI-0 disconnected ..."
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && (parts[1] == "connected" || parts[1] == "disconnected") {
                outputs.insert(parts[0].to_string(), parts[1].to_string());
            }
        }

        Ok(outputs)
    }

    fn apply_dimming(&self, output: &str, value: u8) -> Result<(), String> {
        let brightness = value as f32 / 100.0;
        run("xrandr", &["--output", output, "--brightness", &brightness.to_string()])
    }

    fn reset(&self, output: &str) -> Result<(), String> {
        run("xrandr", &["--output", output, "--brightness", "1.0"])
    }
}
