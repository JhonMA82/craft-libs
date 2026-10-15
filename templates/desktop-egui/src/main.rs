#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! Binary entry: parse `--config-dir`, open [`crate::settings::Prefs`] store,
//! install the macOS app menu, run eframe, flush on exit via `on_exit`.
//!
//! Copy this folder out of craft-libs to start a new app. It is a template,
//! not a library: never add it as a dependency.

mod app;
mod icons;
mod settings;

use std::path::PathBuf;

use craft_settings::{Location, SettingsStore};

use crate::app::TemplateApp;
use crate::settings::Prefs;

/// `APP_ID` doubles as the config folder name and the `<APP>_CONFIG_DIR` stem.
/// Rename to the new app's lowercase id (e.g. `"myapp"` → `MYAPP_CONFIG_DIR`).
const APP_ID: &str = "template-app";

fn main() {
    // `--config-dir <dir>`: explicit folder override for tests/demos.
    let mut args = std::env::args().skip(1);
    let location = match args.next().as_deref() {
        Some("--config-dir") => args.next().map(PathBuf::from).map_or(Location::Auto, Location::Dir),
        _ => Location::Auto,
    };
    let (store, report) = SettingsStore::<Prefs>::open(location);
    if report.has_problems() {
        eprintln!("settings: {report:?} (rule: {})", report.rule);
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 750.0]).with_app_id(APP_ID),
        ..Default::default()
    };

    // Moved into the app creator: `run_native` may call it, but the store must
    // exist exactly once, and code after `run_native` does not run on macOS quit.
    let store_cell = std::cell::RefCell::new(Some((store, report)));
    let result = eframe::run_native(
        APP_ID,
        native_options,
        Box::new(move |ctx| {
            // Install the native Quit menu (macOS) BEFORE winit replaces it.
            // On error the app still runs; Cmd+Q just quits without asking.
            if let Err(e) = craft_quit::install_app_menu(&ctx.egui_ctx, "Template App") {
                eprintln!("app menu: {e}");
            }
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            let (store, report) = store_cell.take().unwrap_or_else(|| SettingsStore::<Prefs>::open(Location::Memory));
            Ok(Box::new(TemplateApp::new(&ctx.egui_ctx, store, report)) as Box<dyn eframe::App>)
        }),
    );
    if let Err(e) = result {
        eprintln!("run_native: {e}");
    }
}
