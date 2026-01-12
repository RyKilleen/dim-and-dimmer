use crate::commands::run_capture;

#[derive(Debug)]
pub struct Display {
    pub id: String,
    pub name: String,
    pub drm_connector: String,
}

pub fn enumerate_displays() -> Vec<Display> {
    let Ok(output) = run_capture("ddcutil", &["detect"]) else {
        return Vec::new();
    };

    parse_ddcutil_detect(&output)
}

fn save_display(
    displays: &mut Vec<Display>,
    id: Option<String>,
    name: Option<String>,
    connector: Option<String>,
) {
    if let (Some(id), Some(connector)) = (id, connector) {
        let name = name.unwrap_or_else(|| format!("Display {}", id));
        displays.push(Display { id, name, drm_connector: connector });
    }
}

fn parse_ddcutil_detect(output: &str) -> Vec<Display> {
    let mut displays = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_name: Option<String> = None;
    let mut current_connector: Option<String> = None;

    for line in output.lines() {
        let trimmed = line.trim();

        if let Some(new_id) = trimmed.strip_prefix("Display ").filter(|s| !s.contains("not found")) {
            save_display(&mut displays, current_id.take(), current_name.take(), current_connector.take());
            current_id = Some(new_id.to_string());
            current_name = None;
            current_connector = None;
        }

        // "DRM connector: card1-DP-1"
        if trimmed.starts_with("DRM connector:") {
            current_connector = trimmed
                .strip_prefix("DRM connector:")
                .map(|s| s.trim().to_string());
        }

        // "Model: DELL U2715H"
        if trimmed.starts_with("Model:") {
            current_name = trimmed
                .strip_prefix("Model:")
                .map(|s| s.trim().to_string());
        }

        // Fallback to monitor name if Model not found
        // "Monitor: name"
        if current_name.is_none() && trimmed.starts_with("Monitor:") {
            current_name = trimmed
                .strip_prefix("Monitor:")
                .map(|s| s.trim().to_string());
        }
    }

    save_display(&mut displays, current_id, current_name, current_connector);

    displays
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ddcutil_detect() {
        let output = r#"Display 1
   I2C bus:  /dev/i2c-7
   DRM connector:           card1-DP-1
   EDID synopsis:
      Mfg id:               DEL - Dell Inc.
      Model:                DELL U2715H
      Product code:         16614  (0x40E6)
      Serial number:        ABC123
      Binary serial number: 123456 (0x0001E240)
      Manufacture year:     2016,  Week: 52
   VCP version:         2.1

Display 2
   I2C bus:  /dev/i2c-8
   DRM connector:           card1-HDMI-A-1
   EDID synopsis:
      Mfg id:               SAM - Samsung
      Model:                Samsung 27"
      Product code:         1234
      Serial number:        XYZ789
      Binary serial number: 789012
      Manufacture year:     2020,  Week: 10
   VCP version:         2.2
"#;

        let displays = parse_ddcutil_detect(output);
        assert_eq!(displays.len(), 2);

        assert_eq!(displays[0].id, "1");
        assert_eq!(displays[0].name, "DELL U2715H");
        assert_eq!(displays[0].drm_connector, "card1-DP-1");

        assert_eq!(displays[1].id, "2");
        assert_eq!(displays[1].name, "Samsung 27\"");
        assert_eq!(displays[1].drm_connector, "card1-HDMI-A-1");
    }
}
