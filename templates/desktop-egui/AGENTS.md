# AGENTS.md — `templates/desktop-egui`

Starter template for a new eframe/egui desktop app. Human guide: `README.md`.
Copy the folder out of craft-libs; never add it as a dependency, never make it
a workspace member (parent `cargo test --workspace` must keep ignoring it).

## Rename contract (all 5, grep: `template-app|Template App|TemplateApp`)

1. `template-app` → lowercase `APP_ID` (`src/main.rs` `APP_ID`, `Prefs::APP_ID`,
   `icons.rs` `URI_PREFIX`, docs). Drives config folder + `<APP>_CONFIG_DIR`.
2. `"Template App"` → display name (macOS menu in `main.rs`).
3. `TemplateApp` → app struct (`src/app.rs`).
4. `template-desktop-egui` → package + binary (`Cargo.toml`).
5. `ICONS` in `src/icons.rs` → real assets + matching `SvgStyle`; record
   licenses in the new app's ATTRIBUTION.md.

## Invariants (do not break when extending)

- Shell order in `ui`: `app-bar` → `tool-rail` → `dock` → `status-bar`.
  Panel ids (`app-bar`, `tool-rail`, `dock`, `status-bar`, `zoom-combo`,
  `brightness-combo`) are global egui ids — keep stable, they persist state.
- `logic` order: close decision → `store.tick` (+ `request_repaint_after` on
  `next_deadline`) → `craft_quit::on_close_request` LAST. No painting in `logic`;
  no close decision in `ui` (it doesn't run minimized/hidden).
- `on_exit` owns `store.flush()`; nothing durable after `run_native` in `main`.
- `Panel::{Canvas, Layers, Properties, History}` are stable/persistent — never
  rename once layouts ship. Canvas is `PROTECTED`; rest is `default()`.
- Dock `Action`s apply AFTER rendering the snapshot, never inside callbacks.
  `dock()` reborrows disjoint fields INSIDE the `show` closure — never hoist
  `&self.layout` / `&mut self.*` across the closure boundary (E0500).
- `IconButton` ids come from scope/position (`push_id` when items reorder),
  never from translated labels. Components return `Response`; the app
  dispatches. Tooltips via `on_hover_text`, never `tooltip_text`.
- `Tokens` is the ONLY place for shell colors; `Brightness` serializes as
  stable camelCase (`rename_all`) — never rename a variant. `Workspace`
  (zoom, tool) stays OUTSIDE `config.toml`.
- `#[serde(default)]` on every settings struct; never `deny_unknown_fields`;
  schema change = bump `SCHEMA_VERSION` + `migrate` step (idempotent, per-step).
- Never `unwrap`/`expect`/`panic`/`todo` (crate denies them); workspace forbids
  `unsafe`. Report via `LoadReport`/`TickResult`/status bar, keep running.
- One SVG URI prefix per catalog/style per egui context. `TOOLS` pairs
  `(icon-name, tool-id)`; selection lives in `Workspace`, not settings.

## Boundaries (template vs craft-libs)

- App owns: `Prefs` shape/defaults, theme palettes, fonts, i18n, commands,
  documents, icons bytes, layout persistence policy, native windows.
- craft-libs owns: folder resolution, atomic writes, quit path, widget
  mechanics, docking transactions, SVG byte prep. Do not reimplement these in
  the app; do not push app policy into craft-libs.
- `craft-cms`/`craft-print` stay opt-in, never default template deps.

## Before merging changes to this template

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

(Run inside the template folder — standalone `[workspace]`, not the parent one.)
First build needs network (eframe/egui from crates.io).
