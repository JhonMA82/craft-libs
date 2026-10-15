//! App-owned settings type: sections, defaults, migrations.
//!
//! Each top-level field becomes one `[section]` in `config.toml`. A section
//! whose values no longer fit falls back to its defaults; the rest is kept.
//!
//! Rules (from craft-settings): `#[serde(default)]` on every struct, never
//! `deny_unknown_fields`, bump `SCHEMA_VERSION` with a `migrate` step.

use craft_settings::{DocumentMut, MigrateError, Settings};
use serde::{Deserialize, Serialize};

/// Rename `APP_ID` to the new app's lowercase id. It names the config folder
/// (`~/.config/<id>/`, `~/Library/Application Support/<id>/`, `%APPDATA%\<id>\`)
/// and the `<APP>_CONFIG_DIR` override variable.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub general: General,
    pub appearance: Appearance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct General {
    /// BCP-47 language tag; empty = system default.
    pub language: String,
    pub autosave_minutes: u32,
}

impl Default for General {
    fn default() -> Self {
        Self { language: String::new(), autosave_minutes: 10 }
    }
}

/// UI brightness levels, after VectorCraft's Illustrator-style scale.
/// Serialized as stable camelCase ids in `config.toml` (`"mediumDark"`, …):
/// never rename a variant once config files exist in the wild. Unknown values
/// fall back per-section (the whole `[appearance]` resets to defaults, the
/// rest of the file is kept) — the craft-settings contract, so no lenient
/// string parsing is needed at runtime.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Brightness {
    Dark,
    #[default]
    MediumDark,
    MediumLight,
    Light,
}

impl Brightness {
    pub const ALL: [Self; 4] = [Self::Dark, Self::MediumDark, Self::MediumLight, Self::Light];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::MediumDark => "Medium Dark",
            Self::MediumLight => "Medium Light",
            Self::Light => "Light",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Appearance {
    pub brightness: Brightness,
    pub ui_scale: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self { brightness: Brightness::default(), ui_scale: 1.0 }
    }
}

/// Workspace state that intentionally lives OUTSIDE `config.toml`:
/// panel layout and zoom change every session and are not worth comments or
/// hand edits. Real apps persist this to `ui.ron` beside `config.toml`
/// (see docs/settings-storage.md); the template keeps it in memory.
#[derive(Debug, Clone, Default)]
pub struct Workspace {
    pub zoom: f32,
    pub active_tool: &'static str,
}

impl Workspace {
    pub fn new() -> Self {
        Self { zoom: 1.0, active_tool: "selection" }
    }
}

impl Settings for Prefs {
    const APP_ID: &'static str = "template-app";
    const SCHEMA_VERSION: u32 = 1;

    /// Upgrade `doc` from `from` to `from + 1`. Called once per step before
    /// typed parsing. Keep steps idempotent; a failed step leaves the file
    /// untouched and the store read-only.
    fn migrate(from: u32, doc: &mut DocumentMut) -> Result<(), MigrateError> {
        // Example v1 -> v2: rename `[appearance] dark_mode` to `theme`.
        // match from {
        //     1 => { /* edit doc, return Ok(()) */ }
        //     _ => return Err(MigrateError::new(from, format!("unknown schema {from}"))),
        // }
        let _ = (from, doc);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons;

    #[test]
    fn defaults_are_a_valid_starting_point() {
        let prefs = Prefs::default();
        assert_eq!(prefs.appearance.brightness, Brightness::MediumDark);
        assert_eq!(prefs.general.autosave_minutes, 10);
        assert_eq!(Prefs::SCHEMA_VERSION, 1);
    }

    #[test]
    fn brightness_round_trips_through_toml_as_stable_ids() {
        // `config.toml` spelling is the contract: never rename a variant.
        let doc: toml_edit::DocumentMut = toml_edit::ser::to_document(&Prefs::default()).unwrap_or_default();
        assert_eq!(doc["appearance"]["brightness"].as_str(), Some("mediumDark"));
        let back: Prefs = toml_edit::de::from_document(doc).unwrap_or_default();
        assert_eq!(back.appearance.brightness, Brightness::MediumDark);
    }

    #[test]
    fn every_catalog_icon_is_registered() {
        for name in icons::NAMES {
            assert!(icons::icon_set().contains(name), "missing icon: {name}");
        }
    }
}
