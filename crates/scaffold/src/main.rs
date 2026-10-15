#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! `craft-scaffold`: ask what the new app needs, render it, write it.
//!
//! Interactive mode asks exactly four questions (app id, display name,
//! family, optional features); `--no-interactive` takes the same answers as
//! flags for CI. See `docs/intent/scaffold-spec.md`.

use std::path::PathBuf;

use craft_scaffold::plan::{AppId, Family, Features, Plan, ScaffoldError};

fn main() {
    if let Err(e) = run(std::env::args().skip(1).collect()) {
        eprintln!("craft-scaffold: {e} (code: {})", e.code());
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<PathBuf, ScaffoldError> {
    let plan = if args.iter().any(|a| a == "--no-interactive") { plan_from_flags(&args)? } else { plan_interactive(&args)? };
    // `write_plan` also rewrites path-deps to git pins when `out` leaves the
    // checkout, so the CLI never branches on delivery here.
    let files = craft_scaffold::render::render(&plan)?;
    craft_scaffold::write::write_plan(&plan, &files)
}

/// Manual flag parser (zero deps): `--app-id ID`, `--display-name NAME`,
/// `--family creative|office|minimal`, `--with-print`, `--with-cms`,
/// `--out DIR` (default `./<app-id>`).
fn plan_from_flags(args: &[String]) -> Result<Plan, ScaffoldError> {
    let mut app_id: Option<String> = None;
    let mut display_name: Option<String> = None;
    let mut family: Option<String> = None;
    let mut features = Features::default();
    let mut out: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--app-id" => {
                i += 1;
                app_id = args.get(i).cloned();
            }
            "--display-name" => {
                i += 1;
                display_name = args.get(i).cloned();
            }
            "--family" => {
                i += 1;
                family = args.get(i).cloned();
            }
            "--with-print" => features.print = true,
            "--with-cms" => features.cms = true,
            "--out" => {
                i += 1;
                out = args.get(i).map(PathBuf::from);
            }
            "--no-interactive" => {}
            other => {
                return Err(ScaffoldError::Render { file: "args".into(), why: format!("unknown flag {other:?}; see --help") });
            }
        }
        i += 1;
    }
    let app_id = AppId::parse(&app_id.unwrap_or_else(|| "myapp".into()))?;
    let family = Family::parse(&family.unwrap_or_else(|| "creative".into()))
        .ok_or_else(|| ScaffoldError::Render { file: "family".into(), why: "expected one of: creative, office, minimal".into() })?;
    let display_name = display_name.unwrap_or_else(|| "My App".into());
    if display_name.is_empty() {
        return Err(ScaffoldError::Render { file: "display-name".into(), why: "must not be empty".into() });
    }
    let out = out.unwrap_or_else(|| PathBuf::from(format!("./{}", app_id.as_str())));
    Ok(Plan { app_id, display_name, family, features, out })
}

/// Interactive mode: four questions on stdin, defaults on empty input.
/// A `--family`/`--out`/etc. flag still overrides its own question: flags are
/// parsed first as resolved defaults, then each question offers them.
fn plan_interactive(args: &[String]) -> Result<Plan, ScaffoldError> {
    let preset = plan_from_flags(&[vec!["--no-interactive".to_owned()], args.to_vec()].concat())?;
    let app_id = AppId::parse(&ask("App id (lowercase, e.g. myapp)", preset.app_id.as_str()))?;
    let display_name = ask("Display name", &preset.display_name);
    if display_name.is_empty() {
        return Err(ScaffoldError::Render { file: "display-name".into(), why: "must not be empty".into() });
    }
    let family = ask(
        "Family (creative|office|minimal)",
        match preset.family {
            Family::Creative => "creative",
            Family::Office => "office",
            Family::Minimal => "minimal",
        },
    );
    let family = Family::parse(&family)
        .ok_or_else(|| ScaffoldError::Render { file: "family".into(), why: "expected one of: creative, office, minimal".into() })?;
    let out = ask("Output dir", &preset.out.display().to_string());
    Ok(Plan { app_id, display_name, family, features: preset.features, out: PathBuf::from(out) })
}

fn ask(prompt: &str, default: &str) -> String {
    use std::io::{BufRead, Write};
    print!("{prompt} [{default}]: ");
    if std::io::stdout().flush().is_err() {
        return default.to_owned();
    }
    let stdin = std::io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_err() {
        return default.to_owned();
    }
    let line = line.trim();
    if line.is_empty() { default.to_owned() } else { line.to_owned() }
}
