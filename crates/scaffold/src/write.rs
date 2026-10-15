#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! Disk writer for rendered plans: refuses non-empty `--out`, writes files,
//! rewrites intra-checkout path-deps to git pins when `--out` leaves the repo.

use std::path::{Path, PathBuf};

use crate::plan::{Plan, ScaffoldError};
use crate::render::OutputFile;

/// Write `files` into `plan.out`. Refuses when the dir exists and is not
/// empty — never overwrites. Creates parents as needed. When the generated
/// manifest's `../../crates/<name>` path-deps do not resolve from `out`, they
/// are rewritten to git pins so the app compiles outside this checkout.
pub fn write_plan(plan: &Plan, files: &[OutputFile]) -> Result<PathBuf, ScaffoldError> {
    if dir_is_non_empty(&plan.out) {
        return Err(ScaffoldError::OutputNotEmpty { path: plan.out.clone() });
    }
    let outside = !path_deps_resolve(&plan.out);
    for file in files {
        let dest = plan.out.join(&file.path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ScaffoldError::Io { path: parent.into(), why: e.to_string() })?;
        }
        let bytes = if outside && file.path.ends_with("Cargo.toml") { rewrite_to_git_pins(&file.bytes) } else { file.bytes.clone() };
        std::fs::write(&dest, &bytes).map_err(|e| ScaffoldError::Io { path: dest, why: e.to_string() })?;
    }
    Ok(plan.out.clone())
}

/// Pinned craft-libs revision for apps generated outside the checkout.
/// Bump with care: generated apps pin it in their `Cargo.toml`.
pub const CRAFT_LIBS_GIT: &str = "https://github.com/storytold/craft-libs";

/// True when `out/../../crates/<name>/Cargo.toml` exist for every craft dep —
/// i.e. the generated manifest's relative path-deps resolve from `out`.
/// What matters is where the deps resolve, not what the directory is called.
fn path_deps_resolve(out: &Path) -> bool {
    let anchor = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let out_abs: PathBuf = if out.is_absolute() { out.into() } else { anchor.join(out) };
    ["settings", "quit", "ui"].iter().all(|dep| out_abs.join("../../crates").join(dep).join("Cargo.toml").exists())
}

/// Replace `path = "../../crates/<name>"` deps with git pins.
fn rewrite_to_git_pins(manifest: &[u8]) -> Vec<u8> {
    let mut text = String::from_utf8_lossy(manifest).into_owned();
    for dep in ["settings", "quit", "ui"] {
        let path_form = format!("craft-{dep} = {{ path = \"../../crates/{dep}\"");
        let git_form = format!("craft-{dep} = {{ git = \"{CRAFT_LIBS_GIT}\"");
        text = text.replace(&path_form, &git_form);
    }
    text.into_bytes()
}

/// True when `dir` exists and holds at least one entry. Missing dir = empty
/// (it will be created); unreadable dir = error at write time, not here.
fn dir_is_non_empty(dir: &Path) -> bool {
    std::fs::read_dir(dir).map(|mut entries| entries.next().is_some()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use crate::plan::{AppId, Family, Features, Plan};

    use super::*;

    fn plan_at(out: &Path) -> Plan {
        Plan {
            app_id: AppId::parse("myapp").unwrap_or_else(|e| panic!("fixture: {e}")),
            display_name: "My App".into(),
            family: Family::Minimal,
            features: Features::default(),
            out: out.into(),
        }
    }

    fn file(path: &str) -> OutputFile {
        OutputFile { path: path.into(), bytes: b"content".to_vec() }
    }

    #[test]
    fn writes_files_into_missing_dir() {
        let dir = std::env::temp_dir().join(format!("scaffold-write-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let plan = plan_at(&dir);
        let out = write_plan(&plan, &[file("src/main.rs"), file("Cargo.toml")]).unwrap_or_else(|e| panic!("write: {e}"));
        assert_eq!(out, dir);
        assert!(dir.join("src/main.rs").exists());
        assert!(dir.join("Cargo.toml").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn refuses_non_empty_out() {
        let dir = std::env::temp_dir().join(format!("scaffold-nonempty-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("setup: {e}"));
        std::fs::write(dir.join("existing.txt"), b"x").unwrap_or_else(|e| panic!("setup: {e}"));
        let plan = plan_at(&dir);
        let Err(ScaffoldError::OutputNotEmpty { path }) = write_plan(&plan, &[file("a.txt")]) else {
            panic!("overwrote non-empty dir");
        };
        assert_eq!(path, dir);
        assert!(!dir.join("a.txt").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rewrites_path_deps_to_git_pins_outside_the_checkout() {
        let dir = std::env::temp_dir().join(format!("scaffold-pins-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let manifest = b"craft-settings = { path = \"../../crates/settings\" }".to_vec();
        let plan = plan_at(&dir);
        write_plan(&plan, &[OutputFile { path: "Cargo.toml".into(), bytes: manifest }]).unwrap_or_else(|e| panic!("write: {e}"));
        let written = std::fs::read(dir.join("Cargo.toml")).unwrap_or_else(|e| panic!("read: {e}"));
        let text = String::from_utf8_lossy(&written);
        assert!(text.contains("git = \"https://github.com/storytold/craft-libs\""), "pins missing: {text}");
        assert!(!text.contains("../../crates"), "path-dep leaked: {text}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
