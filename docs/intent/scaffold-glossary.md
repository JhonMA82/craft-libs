# Glossary (scaffold scope)

Living glossary for the `craft-scaffold` work. Canonical terms only; challenged
inline against the audit and the 12 app sources.

| Term | Meaning | Not |
|---|---|---|
| Family | One of creative / office / minimal shell geometry. | A theme or palette. |
| Shell | The generated app's fixed chrome (bars, rail/ribbon, dock, status) + example bodies. | The document engine. |
| Plan | Validated CLI input (`AppId`, `Family`, features, out dir). Pure render input. | Raw argv strings. |
| AppId | Lowercase id (`[_a-z0-9-]`); derives config folder, `<APP>_CONFIG_DIR`, URI prefix. | Display name. |
| Tokens | App-owned full palette struct (per family, ~38 fields creative). | `WidgetPalette` (shared 5-state loop only). |
| Icon slot | Square centered sub-rect + `fit_to_exact_size` + tint; the only legal icon paint. | `rect.shrink()` on a non-square button rect (deforms). |
| Panel frame | `Frame::NONE.inner_margin(family insets)` wrapping every dock body. | Bare labels on the dock surface. |
| Style doc | Generated `docs/style.md` with Tokens/margins/slots/contracts + file:line anchors. | craft-libs docs (mechanisms, not look). |
| Golden harness | Offscreen `egui_kittest+wgpu` composition renders + local PNG workflow. | Committed reference PNGs (out of scope). |

Edge stress: `AppId` with uppercase/spaces → rejected at border
(`INVALID_APP_ID`), never sanitized silently. `--out` pointing at `/` or `$HOME`
→ rejected as non-empty (same rule, no special cases). Family typo →
`clap`-less manual parser lists the 3 valid values in the error.
