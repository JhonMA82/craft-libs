#![deny(clippy::unwrap_used, clippy::expect_used, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! End-to-end generation gate: render all three families to tempdirs and
//! `cargo check` each output. Requires network (eframe/egui from crates.io)
//! and a warm target dir; run explicitly, not in the default suite.
//!
//! ```sh
//! cargo test -p craft-scaffold --test generation
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

use craft_scaffold::plan::{AppId, Family, Features, Plan};
use craft_scaffold::render::render;
use craft_scaffold::write::write_plan;

fn generate(family: Family, dir: &Path) {
    let liga = match family {
        Family::Minimal => "minimal",
        Family::Creative => "creative",
        Family::Office => "office",
    };
    let plan = Plan {
        app_id: AppId::parse(&format!("gen-{liga}")).unwrap_or_else(|e| panic!("fixture: {e}")),
        display_name: format!("Gen {liga}"),
        family,
        features: Features::default(),
        out: dir.join(liga),
    };
    let files = render(&plan).unwrap_or_else(|e| panic!("render {liga}: {e}"));
    write_plan(&plan, &files).unwrap_or_else(|e| panic!("write {liga}: {e}"));
}

fn check(dir: &Path) {
    let status = Command::new("cargo")
        .args(["check", "--tests"])
        .arg("--offline")
        .current_dir(dir)
        .status()
        .unwrap_or_else(|e| panic!("spawn check in {}: {e}", dir.display()));
    assert!(status.success(), "cargo check failed in {}", dir.display());
}

#[test]
fn all_families_generate_and_check_clean() {
    // Outside `temp_dir()` on purpose: `/tmp` is often a small tmpfs and three
    // eframe target dirs exceed it (two prior `No space left` failures).
    // `CARGO_TARGET_TMPDIR` lets CI redirect; default is the crate's target.
    let root: PathBuf = std::env::var_os("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("generation-gate"));
    let _ = std::fs::remove_dir_all(&root);
    for family in [Family::Minimal, Family::Creative, Family::Office] {
        generate(family, &root);
    }
    for liga in ["minimal", "creative", "office"] {
        check(&root.join(liga));
    }
    let _ = std::fs::remove_dir_all(&root);
}
