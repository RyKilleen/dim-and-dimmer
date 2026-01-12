mod wayland;
mod x11;

use std::collections::HashMap;
use std::env;

pub use x11::X11Backend;
pub use wayland::WaylandBackend;

pub trait GammaBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn enumerate_outputs(&self) -> Result<HashMap<String, String>, String>;
    fn apply_dimming(&self, output: &str, value: u8) -> Result<(), String>;
    fn reset(&self, output: &str) -> Result<(), String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    X11,
    Wayland,
    Unknown,
}

pub fn detect_session() -> SessionType {
    if let Ok(session_type) = env::var("XDG_SESSION_TYPE") {
        match session_type.to_lowercase().as_str() {
            "x11" => return SessionType::X11,
            "wayland" => return SessionType::Wayland,
            _ => {}
        }
    }

    if env::var("WAYLAND_DISPLAY").is_ok() {
        return SessionType::Wayland;
    }

    if env::var("DISPLAY").is_ok() {
        return SessionType::X11;
    }

    SessionType::Unknown
}

pub fn create_backend() -> Option<Box<dyn GammaBackend>> {
    match detect_session() {
        SessionType::X11 => X11Backend::new().map(|b| Box::new(b) as Box<dyn GammaBackend>),
        SessionType::Wayland => WaylandBackend::new().map(|b| Box::new(b) as Box<dyn GammaBackend>),
        SessionType::Unknown => None,
    }
}

/// Parses connector parts like ["HDMI", "A", "1"] or ["DP", "1"] into (type, index).
fn parse_connector_type_and_index(parts: &[&str]) -> Option<(String, i32)> {
    if parts.len() >= 3 && parts[1] == "A" {
        let idx: i32 = parts[2].parse().ok()?;
        Some((format!("{}-{}", parts[0], parts[1]), idx))
    } else if parts.len() >= 2 {
        let idx: i32 = parts.last()?.parse().ok()?;
        Some((parts[..parts.len() - 1].join("-"), idx))
    } else {
        None
    }
}

/// Map a DRM connector name (e.g., "card1-DP-1") to an xrandr output name (e.g., "DP-0")
pub fn map_drm_to_xrandr(drm_connector: &str, xrandr_outputs: &HashMap<String, String>) -> Option<String> {
    // Extract connector type and index from DRM name
    // Format: "card{N}-{TYPE}-{INDEX}" e.g., "card1-DP-1", "card0-HDMI-A-1"
    let parts: Vec<&str> = drm_connector.split('-').collect();
    if parts.len() < 3 {
        return None;
    }

    let connector_type_and_index = &parts[1..];

    let (conn_type, drm_index) = parse_connector_type_and_index(connector_type_and_index)?;

    // NVIDIA uses 0-based indexing in xrandr, DRM uses 1-based
    // Try both the original index and index-1 to handle driver differences
    let xrandr_name_0based = format!("{}-{}", conn_type, drm_index - 1);
    let xrandr_name_1based = format!("{}-{}", conn_type, drm_index);

    if xrandr_outputs.contains_key(&xrandr_name_0based) {
        Some(xrandr_name_0based)
    } else if xrandr_outputs.contains_key(&xrandr_name_1based) {
        Some(xrandr_name_1based)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_drm_to_xrandr_dp() {
        let mut outputs = HashMap::new();
        outputs.insert("DP-0".to_string(), "connected".to_string());
        outputs.insert("DP-1".to_string(), "connected".to_string());

        // DRM DP-1 should map to xrandr DP-0 (NVIDIA) or DP-1 (others)
        let result = map_drm_to_xrandr("card1-DP-1", &outputs);
        assert!(result == Some("DP-0".to_string()) || result == Some("DP-1".to_string()));
    }

    #[test]
    fn test_map_drm_to_xrandr_hdmi() {
        let mut outputs = HashMap::new();
        outputs.insert("HDMI-A-0".to_string(), "connected".to_string());

        let result = map_drm_to_xrandr("card0-HDMI-A-1", &outputs);
        assert_eq!(result, Some("HDMI-A-0".to_string()));
    }
}
