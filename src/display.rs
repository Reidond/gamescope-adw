use gtk::gdk;
use gtk::prelude::{Cast, DisplayExt, ListModelExt, MonitorExt};

use crate::settings::Resolution;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionPreset {
    pub label: &'static str,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayDefaults {
    pub native: Resolution,
    pub presets: Vec<ResolutionPreset>,
}

impl DisplayDefaults {
    pub fn detect() -> Option<Self> {
        let display = gdk::Display::default()?;
        let monitors = display.monitors();
        let monitor = monitors.item(0)?.downcast::<gdk::Monitor>().ok()?;
        let geometry = monitor.geometry();
        let scale = <gdk::Monitor as gdk::prelude::MonitorExt>::scale(&monitor).max(1.0);
        let native = Resolution {
            width: round_to_even((geometry.width() as f64 * scale).round() as u32),
            height: round_to_even((geometry.height() as f64 * scale).round() as u32),
        };
        Some(Self::new(native))
    }

    pub fn new(native: Resolution) -> Self {
        let mut presets = Vec::new();
        push_unique_preset(&mut presets, "Native", native);
        push_unique_preset(&mut presets, "1.5x scale", scaled_resolution(native, 1.5));
        push_unique_preset(&mut presets, "2x scale", scaled_resolution(native, 2.0));

        Self { native, presets }
    }

    pub fn preset_index_for(&self, resolution: Resolution) -> u32 {
        self.presets
            .iter()
            .position(|preset| preset.resolution == resolution)
            .unwrap_or_default() as u32
    }
}

impl Default for DisplayDefaults {
    fn default() -> Self {
        Self::new(Resolution {
            width: 1920,
            height: 1080,
        })
    }
}

fn push_unique_preset(
    presets: &mut Vec<ResolutionPreset>,
    label: &'static str,
    resolution: Resolution,
) {
    if resolution.width < 320 || resolution.height < 240 {
        return;
    }
    if presets.iter().any(|preset| preset.resolution == resolution) {
        return;
    }
    presets.push(ResolutionPreset { label, resolution });
}

fn scaled_resolution(native: Resolution, scale: f64) -> Resolution {
    Resolution {
        width: round_to_even((native.width as f64 / scale).round() as u32),
        height: round_to_even((native.height as f64 / scale).round() as u32),
    }
}

fn round_to_even(value: u32) -> u32 {
    value - (value % 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_unique_presets_from_native_resolution() {
        let defaults = DisplayDefaults::new(Resolution {
            width: 3840,
            height: 2160,
        });

        let rendered: Vec<_> = defaults
            .presets
            .iter()
            .map(|preset| {
                (
                    preset.label,
                    preset.resolution.width,
                    preset.resolution.height,
                )
            })
            .collect();

        assert_eq!(
            rendered,
            vec![
                ("Native", 3840, 2160),
                ("1.5x scale", 2560, 1440),
                ("2x scale", 1920, 1080),
            ]
        );
    }
}
