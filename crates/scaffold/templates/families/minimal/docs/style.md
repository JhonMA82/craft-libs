# {{display-name}} — style guide (family: minimal)

Authoritative look contract for this app. An agent adding a panel, button or
dialog follows this file; what contradicts it loses. Shared mechanism docs
live in craft-libs (`crates/ui/README.md`, `docs/ui-docking.md`,
`docs/theme-infrastructure.md`); this file owns the app's look.

## Tokens (`src/app.rs`, `Tokens::for_brightness`)

| Role | MediumDark | Light | Notes |
|---|---|---|---|
| `bar` | `#262626` | `#e8e8e8` | menu + status bars |
| `panel` | `#323232` | `#f4f4f4` | document background, windows |
| `border` | `#1b1b1b` | `#bdbdbd` | 1.0pt frame strokes |
| `text` / `text_dim` | `#d8d8d8` / `#a8a8a8` | `#1f1f1f` / `#555555` | never hard-code grays elsewhere |
| `accent` | `#378ef0` | `#1473e6` | dirty dot, focus; nothing else |

Add a color here first; never a literal `Color32` in a panel. The shared
`WidgetPalette` loop applies the five widget fills from these tokens; borders,
expansion and selection stay egui's.

## Shell map (egui ids — stable, they persist widget state)

- `menu-bar` (`Panel::top`, 32pt, inner `left 8 / right 14`): File (Save/Quit),
  View (brightness list), dirty `●`/Saved at the right.
- `status-bar` (`Panel::bottom`, 24pt, inner `symmetric(8, 0)`): settings dir +
  rule, save errors, read-only flag.
- `Unsaved changes` window: Save / Don't save / Cancel.

## Rules

1. **Panel frame**: every body wraps in
   `Frame::NONE.inner_margin(Margin { left: 12, right: 10, top: 10, bottom: 8 })`.
   Bare labels on the dock surface are a bug (text touches the edge).
2. **Icon slot**: square centered sub-rect + `fit_to_exact_size(rect.size())` +
   theme tint. Never `shrink()` a non-square button rect (deforms the SVG).
3. **Buttons**: `craft_ui::buttons::IconButton` with an explicit style struct;
   ids from scope/position (`push_id` when items reorder), never from
   translated labels; tooltips via `on_hover_text`; dispatch from `Response`.
4. **Settings**: `#[serde(default)]` everywhere, never `deny_unknown_fields`;
   schema change = bump `SCHEMA_VERSION` + idempotent `migrate` step.
5. **Close flow**: dirty → `CancelClose` + modal in `logic` (runs hidden);
   `craft_quit::on_close_request` last; durable work in `on_exit` only.
6. **Panel ids**: stable nonlocalized strings; never rename once shipped.
