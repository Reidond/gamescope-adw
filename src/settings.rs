use std::ffi::OsString;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GamescopeSettings {
    pub output_resolution: Option<Resolution>,
    pub nested_resolution: Option<Resolution>,
    pub nested_refresh: Option<u32>,
    pub framerate_limit: Option<u32>,
    pub window_mode: WindowMode,
    pub scaler: Scaler,
    pub filter: Filter,
    pub hdr: bool,
    pub adaptive_sync: bool,
    pub mangoapp: bool,
    pub steam: bool,
    pub extra_args: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowMode {
    Windowed,
    Fullscreen,
    Borderless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scaler {
    Default,
    Auto,
    Integer,
    Fit,
    Fill,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Filter {
    Default,
    Linear,
    Nearest,
    Fsr,
    Nis,
    Pixel,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SettingsError {
    #[error("resolution must be greater than zero")]
    EmptyResolution,
    #[error("refresh rate must be greater than zero")]
    EmptyRefresh,
    #[error("FPS limit must be greater than zero")]
    EmptyFramerateLimit,
    #[error("extra Gamescope arguments are not valid shell words: {0}")]
    InvalidExtraArgs(String),
}

impl Default for GamescopeSettings {
    fn default() -> Self {
        Self {
            output_resolution: None,
            nested_resolution: None,
            nested_refresh: None,
            framerate_limit: None,
            window_mode: WindowMode::Windowed,
            scaler: Scaler::Default,
            filter: Filter::Default,
            hdr: false,
            adaptive_sync: false,
            mangoapp: false,
            steam: false,
            extra_args: String::new(),
        }
    }
}

impl GamescopeSettings {
    pub fn to_gamescope_args(&self) -> Result<Vec<OsString>, SettingsError> {
        let mut args = Vec::new();

        if let Some(resolution) = self.nested_resolution {
            validate_resolution(resolution)?;
            args.extend(["-w", &resolution.width.to_string()].map(OsString::from));
            args.extend(["-h", &resolution.height.to_string()].map(OsString::from));
        }

        if let Some(resolution) = self.output_resolution {
            validate_resolution(resolution)?;
            args.extend(["-W", &resolution.width.to_string()].map(OsString::from));
            args.extend(["-H", &resolution.height.to_string()].map(OsString::from));
        }

        if let Some(refresh) = self.nested_refresh {
            if refresh == 0 {
                return Err(SettingsError::EmptyRefresh);
            }
            args.extend(["-r", &refresh.to_string()].map(OsString::from));
        }

        if let Some(limit) = self.framerate_limit {
            if limit == 0 {
                return Err(SettingsError::EmptyFramerateLimit);
            }
            args.extend(["--framerate-limit", &limit.to_string()].map(OsString::from));
        }

        match self.window_mode {
            WindowMode::Windowed => {}
            WindowMode::Fullscreen => args.push("-f".into()),
            WindowMode::Borderless => args.push("-b".into()),
        }

        if let Some(value) = self.scaler.gamescope_value() {
            args.extend(["-S", value].map(OsString::from));
        }

        if let Some(value) = self.filter.gamescope_value() {
            args.extend(["-F", value].map(OsString::from));
        }

        if self.hdr {
            args.push("--hdr-enabled".into());
        }

        if self.adaptive_sync {
            args.push("--adaptive-sync".into());
        }

        if self.mangoapp {
            args.push("--mangoapp".into());
        }

        if self.steam {
            args.push("--steam".into());
        }

        if !self.extra_args.trim().is_empty() {
            let extra = shell_words::split(&self.extra_args)
                .map_err(|err| SettingsError::InvalidExtraArgs(err.to_string()))?;
            args.extend(extra.into_iter().map(OsString::from));
        }

        Ok(args)
    }
}

impl Scaler {
    pub const LABELS: [&'static str; 6] = [
        "System default",
        "Auto",
        "Integer",
        "Fit",
        "Fill",
        "Stretch",
    ];

    pub fn from_index(index: u32) -> Self {
        match index {
            1 => Self::Auto,
            2 => Self::Integer,
            3 => Self::Fit,
            4 => Self::Fill,
            5 => Self::Stretch,
            _ => Self::Default,
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Default => 0,
            Self::Auto => 1,
            Self::Integer => 2,
            Self::Fit => 3,
            Self::Fill => 4,
            Self::Stretch => 5,
        }
    }

    fn gamescope_value(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Auto => Some("auto"),
            Self::Integer => Some("integer"),
            Self::Fit => Some("fit"),
            Self::Fill => Some("fill"),
            Self::Stretch => Some("stretch"),
        }
    }
}

impl Filter {
    pub const LABELS: [&'static str; 6] =
        ["System default", "Linear", "Nearest", "FSR", "NIS", "Pixel"];

    pub fn from_index(index: u32) -> Self {
        match index {
            1 => Self::Linear,
            2 => Self::Nearest,
            3 => Self::Fsr,
            4 => Self::Nis,
            5 => Self::Pixel,
            _ => Self::Default,
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Default => 0,
            Self::Linear => 1,
            Self::Nearest => 2,
            Self::Fsr => 3,
            Self::Nis => 4,
            Self::Pixel => 5,
        }
    }

    fn gamescope_value(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Linear => Some("linear"),
            Self::Nearest => Some("nearest"),
            Self::Fsr => Some("fsr"),
            Self::Nis => Some("nis"),
            Self::Pixel => Some("pixel"),
        }
    }
}

impl WindowMode {
    pub const LABELS: [&'static str; 3] = ["Windowed", "Fullscreen", "Borderless"];

    pub fn from_index(index: u32) -> Self {
        match index {
            1 => Self::Fullscreen,
            2 => Self::Borderless,
            _ => Self::Windowed,
        }
    }

    pub fn index(self) -> u32 {
        match self {
            Self::Windowed => 0,
            Self::Fullscreen => 1,
            Self::Borderless => 2,
        }
    }
}

fn validate_resolution(resolution: Resolution) -> Result<(), SettingsError> {
    if resolution.width == 0 || resolution.height == 0 {
        return Err(SettingsError::EmptyResolution);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_gamescope_arguments_in_expected_order() {
        let settings = GamescopeSettings {
            nested_resolution: Some(Resolution {
                width: 1280,
                height: 720,
            }),
            output_resolution: Some(Resolution {
                width: 1920,
                height: 1080,
            }),
            nested_refresh: Some(60),
            framerate_limit: Some(60),
            window_mode: WindowMode::Fullscreen,
            scaler: Scaler::Fit,
            filter: Filter::Fsr,
            hdr: true,
            adaptive_sync: true,
            mangoapp: true,
            steam: true,
            extra_args: "--sharpness 4 --prefer-output 'DP-1'".to_string(),
        };

        let args = settings.to_gamescope_args().unwrap();
        let rendered: Vec<String> = args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(
            rendered,
            vec![
                "-w",
                "1280",
                "-h",
                "720",
                "-W",
                "1920",
                "-H",
                "1080",
                "-r",
                "60",
                "--framerate-limit",
                "60",
                "-f",
                "-S",
                "fit",
                "-F",
                "fsr",
                "--hdr-enabled",
                "--adaptive-sync",
                "--mangoapp",
                "--steam",
                "--sharpness",
                "4",
                "--prefer-output",
                "DP-1",
            ]
        );
    }

    #[test]
    fn rejects_invalid_extra_args() {
        let settings = GamescopeSettings {
            extra_args: "'unterminated".to_string(),
            ..Default::default()
        };

        assert!(matches!(
            settings.to_gamescope_args(),
            Err(SettingsError::InvalidExtraArgs(_))
        ));
    }
}
