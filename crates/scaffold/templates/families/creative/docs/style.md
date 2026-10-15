# {{display-name}} — style guide (family: creative)

Authoritative look contract for this app. An agent adding a panel, button or
dialog follows this file; what contradicts it loses. Shared mechanism docs
live in craft-libs (`crates/ui/README.md`, `docs/ui-docking.md`,
`docs/theme-infrastructure.md`); this file owns the app's look. Composition
follows VectorCraft (`chrome.rs` app bar + `toolbar.rs` rail + `dock.rs` right
group + status bar).

## Tokens (`src/app.rs`, `Tokens::for_brightness`)

16 roles; values below are MediumDark (the default). Dark shifts fills darker,
MediumLight/Light mirror them; accent, selection and text roles stay fixed.

| Role | MediumDark | Purpose |
|---|---|---|
| `app_bar` | `#262626` | 32pt app bar |
| `panel` / `panel_darker` | `#323232` / `#282828` | rail, dock, inputs |
| `input` / `input_border` | `#1f1f1f` / `#464646` | text fields |
| `border` | `#1b1b1b` | 1.0–1.5pt frame strokes |
| `text` / `text_dim` / `icon` | `#d8d8d8` / `#a8a8a8` / `#cfcfcf` | never hard-code grays elsewhere |
| `hover` / `tool_active` | `#444444` / `#1d1d1d` | wells, well radius 1–4 |
| `accent` / `accent_strong` | `#378ef0` / `#1473e6` | brand dot, dirty dot, focus |
| `row_selected` | `#484b50` | layer rows |
| `pasteboard` / `canvas` | `#1e1e1e` / `#ffffff` | canvas surround + page |

Add a color here first; never a literal `Color32` in a panel. The shared
`WidgetPalette` loop applies the five widget fills; borders, expansion and
selection stay egui's (`docs/theme-infrastructure.md`).

## Shell map (egui ids — stable, they persist widget state)

- `app-bar` (`Panel::top`, 32pt, inner `left 8 / right 14`): brand dot
  (accent, r=6), File (Save/Quit), View (toggle Layers/Properties/History),
  4 brightness labels, dirty `●`/Saved.
- `tool-rail` (`Panel::left`, 48pt, inner `symmetric(0, 4)`): 8 tools at
  30pt pitch (`selection marquee pen brush type shapes hand zoom`); selection
  in `Workspace`, tooltips = tool name, shortcuts plug in at the call site.
- `dock` (`DockArea` in `CentralPanel`): `Canvas` (`PROTECTED` — no
  move/float/close), `Layers` (4 rows + new-layer field), `Properties`
  (brightness combo + scale slider + fills), `History`.
- `status-bar` (`Panel::bottom`, 24pt, inner `symmetric(8, 0)`): `zoom-combo`
  (25–400%), active tool, settings dir + rule, save errors.
- `Unsaved changes` window: Save / Don't save / Cancel.

## Rules

1. **Icon slot** (`toolbar.rs:227` contract): square centered sub-rect
   (18×18 in a 36×29 slot), `fit_to_exact_size`, theme tint. Never `shrink()`
   a non-square button rect — it deforms the SVG.
2. **Panel frame** (`dock.rs:102` contract): every body wraps in
   `Frame::NONE.inner_margin(Margin { left: 12, right: 10, top: 10, bottom: 8 })`.
   Bare labels on the dock surface are a bug.
3. **Buttons**: `craft_ui::buttons::IconButton` + explicit style; ids from
   scope/position (`push_id` when items reorder), never translated labels;
   `on_hover_text` tooltips; dispatch from `Response`.
4. **Tabs**: doc strips use `craft_ui::tabs::fit` (natural = galley + 24,
   minimum 40, overflow 18) + `Tab::show_at`; overflow through an enabled
   `…` control with a menu. Never silently drop a tab.
5. **Toolbar overflow**: `craft_ui::layout::fit_toolbar`; every overflow index
   reachable through the overflow control's menu.
6. **Settings**: `#[serde(default)]` everywhere, never `deny_unknown_fields`;
   schema change = bump `SCHEMA_VERSION` + idempotent `migrate`.
   `Workspace` (zoom, tool) stays OUTSIDE `config.toml`.
7. **Close flow**: dirty → `CancelClose` + modal in `logic` (runs hidden);
   `craft_quit::on_close_request` last; durable work in `on_exit` only.
8. **Panel ids**: `Panel::{Canvas, Layers, Properties, History}` are stable
   nonlocalized ids; never rename once shipped. Canvas stays `PROTECTED`.
