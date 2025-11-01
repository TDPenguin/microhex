use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize, Clone)]
pub struct ColorConfig {
    pub status_bg: u8,
    pub status_fg: u8,
    pub help_bg: u8,
    pub help_fg: u8,
    pub header_fg: u8,
    pub cursor_active_bg: u8,
    pub cursor_active_fg: u8,
    pub cursor_inactive_bg: u8,
    pub cursor_inactive_fg: u8,
    pub changed_fg: u8,
    pub null_fg: u8,
    pub control_fg: u8,
    pub printable_fg: u8,
}

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub colors: ColorConfig,
    // Add other config fields here in the future
}

impl AppConfig {
    pub fn load(path: &str) -> Self {
        let default_toml = r#"# All color values below are ANSI 256-color codes (0-255).
# See: https://www.ditig.com/256-colors-cheat-sheet

[colors]
status_bg = 15
status_fg = 0
help_bg = 15
help_fg = 0
header_fg = 51
cursor_active_bg = 226
cursor_active_fg = 16
cursor_inactive_bg = 240
cursor_inactive_fg = 15
changed_fg = 208
null_fg = 242
control_fg = 33
printable_fg = 34
"#;
        if !Path::new(path).exists() {
            let _ = fs::write(path, default_toml);
        }
        fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_else(|| toml::from_str(default_toml).unwrap())
    }
}