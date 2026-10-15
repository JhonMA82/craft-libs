# {{display-name}} — style guide (family: office)

Authoritative look contract for this app. An agent adding a ribbon group,
dialog or pane follows this file; what contradicts it loses. Shared mechanism
docs live in craft-libs (`crates/ui/README.md`); this file owns the app's look.
Ribbon geometry follows the Word/Grid/Deck family (`RIBBON_H = 66`).

## Tokens (`src/app.rs`, `Tokens::for_brightness`)

| Role | MediumDark | Purpose |
|---|---|---|
| `chrome` / `chrome_dark` | `#262626` / `#1e1e1e` | title bar, tabs, ribbon |
| `panel` | `#323232` | document background, windows |
| `border` | `#1b1b1b` | 1.0pt frame strokes, group separators |
| `text` / `text_dim` | `#d8d8d8` / `#a8a8a8` | never hard-code grays elsewhere |
| `accent` / `accent_text` | `#378ef0` / `#ffffff` | dirty dot, selection |
| `hover` | `#444444` | button wells |

Add a color here first; never a literal `Color32` in a panel.

## Shell map (egui ids — stable, they persist widget state)

- `title-bar` (`Panel::top`, 38pt, inner `symmetric(8, 0)`): display name +
  dirty `●`/Saved at the right.
- `tabs` (`Panel::top`, 28pt): contextual tabs (`Home Insert Layout Review
  View`); click selects, **double-click collapses the ribbon**.
- `ribbon` (`Panel::top`, 66+20pt or 32pt collapsed, inner `symmetric(8, 4)`):
  groups left-to-right with 13pt separators; overflow behind `…`.
- `status-bar` (`Panel::bottom`, 26pt, inner `symmetric(8, 0)`): active tab,
  settings dir + rule, save errors.
- `Unsaved changes` window: Save / Don't save / Cancel.

## Rules

1. **Ribbon fit**: `craft_ui::layout::fit_toolbar(natural, available, 8.0,
   28.0)`; visible groups keep order + natural widths; every overflow index
   reachable through the enabled, focusable `…` control's menu. A `None` fit
   (window narrower than the `…` control) shows the narrow-window message —
   never silently dropped commands.
2. **Icon slot**: square sub-rect + `fit_to_exact_size` + theme tint. Never
   `shrink()` a non-square rect.
3. **Panel frame**: document body wraps in
   `Frame::NONE.inner_margin(Margin { left: 12, right: 10, top: 10, bottom: 8 })`.
4. **Collapse**: `ribbon_collapsed` is workspace state (not settings);
   double-click toggles, tabs stay visible.
5. **Settings**: `#[serde(default)]` everywhere, never `deny_unknown_fields`;
   schema change = bump `SCHEMA_VERSION` + idempotent `migrate`.
6. **Close flow**: dirty → `CancelClose` + modal in `logic`;
   `craft_quit::on_close_request` last; durable work in `on_exit` only.
