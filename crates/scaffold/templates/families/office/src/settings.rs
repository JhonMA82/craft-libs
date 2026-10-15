//! App-owned settings type: sections, defaults, migrations.
//!
//! Each top-level field becomes one `[section]` in `config.toml`. A section
//! whose values no longer fit falls back to its defaults; the rest is kept.
//!
//! Rules (from craft-settings): `#[serde(default)]` on every struct, never
//! `deny_unknown_fields`, bump `SCHEMA_VERSION` with a `migrate` step.

use craft_settings::{DocumentMut, MigrateError, Settings};
use serde::{Deserialize, Serialize};

/// The config folder (`~/.config/{{app-id}}/`, `~/Library/Application
/// Support/{{app-id}}/`, `%APPDATA%\{{app-id}}\`) and the
/// `{{ENV_STEM}}_CONFIG_DIR` override derive from `APP_ID`.
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

/// UI brightness levels. Serialized as stable camelCase ids in `config.toml`
/// (`"mediumDark"`, …): never rename a variant once config files exist in the
/// wild. Unknown values fall back per-section — the craft-settings contract.
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

impl Settings for Prefs {
    const APP_ID: &'static str = "{{app-id}}";
    const SCHEMA_VERSION: u32 = 1;

    /// Upgrade `doc` from `from` to `from + 1`. Called once per step before
    /// typed parsing. Keep steps idempotent; a failed step leaves the file
    /// untouched and the store read-only.
    fn migrate(from: u32, doc: &mut DocumentMut) -> Result<(), MigrateError> {
        let _ = (from, doc);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_a_valid_starting_point() {
        let prefs = Prefs::default();
        assert_eq!(prefs.appearance.brightness, Brightness::MediumDark);
        assert_eq!(prefs.general.autosave_minutes, 10);
        assert_eq!(Prefs::SCHEMA_VERSION, 1);
    }

    #[test]
    fn brightness_round_trips_through_toml_as_stable_ids() {
        let doc: toml_edit::DocumentMut = toml_edit::ser::to_document(&Prefs::default()).unwrap_or_default();
        assert_eq!(doc["appearance"]["brightness"].as_str(), Some("mediumDark"));
        let back: Prefs = toml_edit::de::from_document(doc).unwrap_or_default();
        assert_eq!(back.appearance.brightness, Brightness::MediumDark);
    }
}
