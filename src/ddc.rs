use crate::commands::{run, run_capture};

pub const VCP_BRIGHTNESS: &str = "10";
pub const VCP_CONTRAST: &str = "12";

pub fn read_vcp(display_id: &str, code: &str) -> Option<u8> {
    let output = run_capture(
        "ddcutil",
        &["getvcp", code, "--display", display_id, "--terse"],
    )
    .ok()?;

    output
        .split_whitespace()
        .nth(3)
        .and_then(|v| v.parse().ok())
}

pub fn set_vcp(display_id: &str, code: &str, value: u8) -> Result<(), String> {
    run(
        "ddcutil",
        &[
            "setvcp",
            code,
            &value.to_string(),
            "--display",
            display_id,
        ],
    )
}
