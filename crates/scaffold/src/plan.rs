#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! Validated generator input: [`AppId`], [`Family`], [`Features`], [`Plan`]
//! and the typed [`ScaffoldError`]. Built once at the CLI border.

use std::fmt;
use std::path::PathBuf;

/// Lowercase application id: config folder, `<APP>_CONFIG_DIR` stem,
/// egui app id and SVG URI prefix derive from it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppId(String);

impl AppId {
    /// Validate `raw` at the CLI border. Rejects (never sanitizes): empty,
    /// uppercase, spaces, underscores, dots, slashes.
    pub fn parse(raw: &str) -> Result<Self, ScaffoldError> {
        let valid = !raw.is_empty() && raw.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if valid { Ok(Self(raw.to_owned())) } else { Err(ScaffoldError::InvalidAppId { got: raw.to_owned() }) }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `MYAPP_CONFIG_DIR`-style env stem for an app id (`photo-craft` → `PHOTO_CRAFT_CONFIG_DIR`).
    pub fn env_stem(&self) -> String {
        self.0.replace('-', "_").to_uppercase()
    }
}

/// UI shell geometry family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Creative,
    Office,
    Minimal,
}

impl Family {
    /// Exactly the three documented names; anything else is `None`
    /// (the CLI prints the valid list, never guesses).
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "creative" => Some(Self::Creative),
            "office" => Some(Self::Office),
            "minimal" => Some(Self::Minimal),
            _ => None,
        }
    }
}

/// Optional generated-app features. Both default off.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Features {
    pub print: bool,
    pub cms: bool,
}

/// Validated generator input. See `docs/intent/scaffold-spec.md`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub app_id: AppId,
    pub display_name: String,
    pub family: Family,
    pub features: Features,
    pub out: PathBuf,
}

/// Typed generator failure with a stable code string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScaffoldError {
    InvalidAppId { got: String },
    OutputNotEmpty { path: PathBuf },
    Render { file: String, why: String },
    Io { path: PathBuf, why: String },
}

impl ScaffoldError {
    /// Stable machine-readable code for each variant.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidAppId { .. } => "INVALID_APP_ID",
            Self::OutputNotEmpty { .. } => "OUTPUT_NOT_EMPTY",
            Self::Render { .. } => "RENDER",
            Self::Io { .. } => "IO",
        }
    }
}

impl fmt::Display for ScaffoldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAppId { got } => write!(f, "invalid app id {got:?}: use lowercase letters, digits and dashes (e.g. \"myapp\")"),
            Self::OutputNotEmpty { path } => write!(f, "refusing to overwrite non-empty {}", path.display()),
            Self::Render { file, why } => write!(f, "cannot render {file}: {why}"),
            Self::Io { path, why } => write!(f, "cannot write {}: {why}", path.display()),
        }
    }
}

impl std::error::Error for ScaffoldError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_app_ids_are_accepted_verbatim() {
        for id in ["myapp", "my-app", "app2", "a", "photo-craft-2"] {
            assert_eq!(AppId::parse(id).map(|a| a.as_str().to_owned()), Ok(id.to_owned()), "id: {id}");
        }
    }

    #[test]
    fn invalid_app_ids_are_rejected_with_their_spelling() {
        for id in ["", "MyApp", "my app", "my_app", "my.app", "UPPER", "a/b"] {
            let Err(e) = AppId::parse(id) else { panic!("accepted invalid id: {id}") };
            assert_eq!(e.code(), "INVALID_APP_ID");
            assert!(matches!(e, ScaffoldError::InvalidAppId { got } if got == id), "id: {id}");
        }
    }

    #[test]
    fn family_parses_exactly_the_three_documented_names() {
        assert_eq!(Family::parse("creative"), Some(Family::Creative));
        assert_eq!(Family::parse("office"), Some(Family::Office));
        assert_eq!(Family::parse("minimal"), Some(Family::Minimal));
        assert_eq!(Family::parse("Creative"), None);
        assert_eq!(Family::parse("studio"), None);
        assert_eq!(Family::parse(""), None);
    }

    #[test]
    fn error_codes_are_stable_strings() {
        assert_eq!(ScaffoldError::InvalidAppId { got: "x".into() }.code(), "INVALID_APP_ID");
        assert_eq!(ScaffoldError::OutputNotEmpty { path: PathBuf::from("d") }.code(), "OUTPUT_NOT_EMPTY");
        assert_eq!(ScaffoldError::Render { file: "f".into(), why: "w".into() }.code(), "RENDER");
        assert_eq!(ScaffoldError::Io { path: PathBuf::from("p"), why: "w".into() }.code(), "IO");
    }
}
