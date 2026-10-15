#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! Pure renderer: [`Plan`] → files. No filesystem, no network, no prompts.
//! Family templates live in `templates/families/<family>/`; `render()`
//! substitutes every `{{placeholder}}` or fails the file.

use crate::plan::{Family, Plan, ScaffoldError};

/// One generated file: repo-relative path + bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

/// Render `plan` to files. All three families ship.
pub fn render(plan: &Plan) -> Result<Vec<OutputFile>, ScaffoldError> {
    match plan.family {
        Family::Minimal => render_family(plan, "minimal", MINIMAL),
        Family::Creative => render_family(plan, "creative", CREATIVE),
        Family::Office => render_family(plan, "office", OFFICE),
    }
}
/// `{{placeholder}}` vocabulary: app-id, display-name, package name, struct
/// name, env stem, per-crate source paths. A file keeping any `{{` after
/// substitution fails `Render` — never ships half-rendered.
fn vars_for(plan: &Plan, package: &str, struct_name: &str) -> Vec<(String, String)> {
    let package_underscore = package.replace('-', "_");
    vec![
        ("{{app-id}}".into(), plan.app_id.as_str().into()),
        ("{{display-name}}".into(), plan.display_name.clone()),
        ("{{package-name}}".into(), package.into()),
        ("{{package_underscore}}".into(), package_underscore),
        ("{{AppStruct}}".into(), struct_name.into()),
        ("{{ENV_STEM}}".into(), plan.app_id.env_stem()),
        ("{{settings-path}}".into(), "../../crates/settings".into()),
        ("{{quit-path}}".into(), "../../crates/quit".into()),
        ("{{ui-path}}".into(), "../../crates/ui".into()),
    ]
}

fn render_family(plan: &Plan, _family: &str, templates: &[(&str, &str)]) -> Result<Vec<OutputFile>, ScaffoldError> {
    let package = plan.app_id.as_str().replace('_', "-");
    let struct_name = struct_name_for(plan.app_id.as_str());
    let vars = vars_for(plan, &package, &struct_name);
    templates
        .iter()
        .map(|(path, template)| {
            let mut text = (*template).to_owned();
            for (key, value) in &vars {
                text = text.replace(key.as_str(), value.as_str());
            }
            if text.contains("{{") || text.contains("}}") {
                return Err(ScaffoldError::Render { file: (*path).into(), why: "unsubstituted placeholder left".into() });
            }
            Ok(OutputFile { path: (*path).into(), bytes: text.into_bytes() })
        })
        .collect()
}

const MINIMAL: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../templates/families/minimal/Cargo.toml")),
    ("rustfmt.toml", include_str!("../templates/families/minimal/rustfmt.toml")),
    ("src/main.rs", include_str!("../templates/families/minimal/src/main.rs")),
    ("src/lib.rs", include_str!("../templates/families/minimal/src/lib.rs")),
    ("src/app.rs", include_str!("../templates/families/minimal/src/app.rs")),
    ("src/settings.rs", include_str!("../templates/families/minimal/src/settings.rs")),
    ("src/icons.rs", include_str!("../templates/families/minimal/src/icons.rs")),
    ("tests/shell_golden.rs", include_str!("../templates/families/minimal/tests/shell_golden.rs")),
    ("docs/style.md", include_str!("../templates/families/minimal/docs/style.md")),
    ("AGENTS.md", include_str!("../templates/families/minimal/AGENTS.md")),
];

const CREATIVE: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../templates/families/creative/Cargo.toml")),
    ("rustfmt.toml", include_str!("../templates/families/creative/rustfmt.toml")),
    ("src/main.rs", include_str!("../templates/families/creative/src/main.rs")),
    ("src/lib.rs", include_str!("../templates/families/creative/src/lib.rs")),
    ("src/app.rs", include_str!("../templates/families/creative/src/app.rs")),
    ("src/settings.rs", include_str!("../templates/families/creative/src/settings.rs")),
    ("src/icons.rs", include_str!("../templates/families/creative/src/icons.rs")),
    ("tests/shell_golden.rs", include_str!("../templates/families/creative/tests/shell_golden.rs")),
    ("docs/style.md", include_str!("../templates/families/creative/docs/style.md")),
    ("AGENTS.md", include_str!("../templates/families/creative/AGENTS.md")),
];

const OFFICE: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../templates/families/office/Cargo.toml")),
    ("rustfmt.toml", include_str!("../templates/families/office/rustfmt.toml")),
    ("src/main.rs", include_str!("../templates/families/office/src/main.rs")),
    ("src/lib.rs", include_str!("../templates/families/office/src/lib.rs")),
    ("src/app.rs", include_str!("../templates/families/office/src/app.rs")),
    ("src/settings.rs", include_str!("../templates/families/office/src/settings.rs")),
    ("src/icons.rs", include_str!("../templates/families/office/src/icons.rs")),
    ("tests/shell_golden.rs", include_str!("../templates/families/office/tests/shell_golden.rs")),
    ("docs/style.md", include_str!("../templates/families/office/docs/style.md")),
    ("AGENTS.md", include_str!("../templates/families/office/AGENTS.md")),
];

/// `my-app` → `Myapp`: ASCII-only CamelCase for the generated struct name.
/// Non-ASCII never reaches here (`AppId::parse` rejects it at the border).
fn struct_name_for(app_id: &str) -> String {
    let mut out = String::with_capacity(app_id.len());
    let mut upper = true;
    for c in app_id.chars() {
        if c == '-' || c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::plan::{AppId, Features};

    use super::*;

    fn plan() -> Plan {
        let app_id = AppId::parse("myapp").unwrap_or_else(|e| panic!("fixture id: {e}"));
        Plan { app_id, display_name: "My App".into(), family: Family::Minimal, features: Features::default(), out: PathBuf::from("/tmp/myapp") }
    }

    #[test]
    fn minimal_renders_ten_files_without_placeholders() {
        let files = render(&plan()).unwrap_or_else(|e| panic!("render: {e}"));
        assert_eq!(files.len(), 10);
        for f in &files {
            let text = String::from_utf8_lossy(&f.bytes);
            assert!(!text.contains("{{"), "leftover placeholder in {}", f.path);
            assert!(!text.contains("}}"), "leftover placeholder in {}", f.path);
        }
    }

    #[test]
    fn minimal_substitutes_app_identity_everywhere() {
        let files = render(&plan()).unwrap_or_else(|e| panic!("render: {e}"));
        let main = files.iter().find(|f| f.path == "src/main.rs").unwrap_or_else(|| panic!("no main.rs"));
        let text = String::from_utf8_lossy(&main.bytes);
        assert!(text.contains("\"myapp\""), "APP_ID missing");
        assert!(text.contains("\"My App\""), "display name missing");
        assert!(text.contains("Myapp"), "struct name missing");
        let harness = files.iter().find(|f| f.path == "tests/shell_golden.rs").unwrap_or_else(|| panic!("no harness"));
        let harness = String::from_utf8_lossy(&harness.bytes);
        assert!(harness.contains("SHELL_GOLDEN_DIR"), "golden workflow missing");
    }

    #[test]
    fn creative_renders_ten_files_without_placeholders() {
        let mut p = plan();
        p.family = Family::Creative;
        let files = render(&p).unwrap_or_else(|e| panic!("render: {e}"));
        assert_eq!(files.len(), 10);
        for f in &files {
            let text = String::from_utf8_lossy(&f.bytes);
            assert!(!text.contains("{{"), "leftover placeholder in {}", f.path);
            assert!(!text.contains("}}"), "leftover placeholder in {}", f.path);
        }
        let app = files.iter().find(|f| f.path == "src/app.rs").unwrap_or_else(|| panic!("no app.rs"));
        let text = String::from_utf8_lossy(&app.bytes);
        assert!(text.contains("tool-rail"), "rail missing");
        assert!(text.contains("Permissions::PROTECTED"), "canvas protection missing");
        assert!(text.contains("Margin { left: 12, right: 10, top: 10, bottom: 8 }"), "panel frame missing");
    }

    #[test]
    fn office_renders_ten_files_with_ribbon_and_overflow() {
        let mut p = plan();
        p.family = Family::Office;
        let files = render(&p).unwrap_or_else(|e| panic!("render: {e}"));
        assert_eq!(files.len(), 10);
        for f in &files {
            let text = String::from_utf8_lossy(&f.bytes);
            assert!(!text.contains("{{"), "leftover placeholder in {}", f.path);
            assert!(!text.contains("}}"), "leftover placeholder in {}", f.path);
        }
        let app = files.iter().find(|f| f.path == "src/app.rs").unwrap_or_else(|| panic!("no app.rs"));
        let text = String::from_utf8_lossy(&app.bytes);
        assert!(text.contains("fit_toolbar"), "ribbon fit missing");
        assert!(text.contains("ribbon_collapsed"), "collapse state missing");
        assert!(text.contains("RIBBON_H"), "ribbon metrics missing");
    }
}
