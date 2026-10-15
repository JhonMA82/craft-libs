# CONSTRAINTS.md — `craft-scaffold` quality floor

Fixed once in F3. Enforced in F4–F7. Lowering it needs the user's explicit call.

- Tests: every slice ships its tests; `cargo test -p craft-scaffold` green.
  The render seam has a no-`{{` leftover test; the integration seam generates
  all 3 families and `cargo check`s each output (network required, same as any
  eframe build).
- Types: workspace lints hold (`unsafe` forbidden; `unwrap`/`expect`/`panic`/
  `todo` denied). CLI parse errors are typed `ScaffoldError` with codes, never
  bare strings; generated apps inherit the same denies.
- A11y: generated shells keep the shared contracts (IconButton focus stroke +
  AccessKit label, `Tab` roles, stable ids independent of labels).
- Security: `--out` never overwrites non-empty dirs; no network calls in the
  CLI itself (only the generated app's later `cargo build` fetches crates);
  `AppId` validated at the border, never sanitized silently.
- Docs: every slice updating generated output also updates that family's
  `style.md` template + the placeholder-coverage test.

| Dimension | Rule | Command | Stage |
|---|---|---|---|
| fmt | `cargo fmt --check`, both CLI and generated output | `cargo fmt --check` / `cargo fmt --check` in `--out` | every slice |
| clippy | `-D warnings --all-targets` on CLI; generated apps same | `cargo clippy -p craft-scaffold --all-targets -- -D warnings` | every slice |
| render purity | `render()` takes `&Plan`, returns files; no FS/net inside | `cargo test -p craft-scaffold render_` | slice 2+ |
| output compiles | all 3 families `cargo check` clean (warm+network) | `cargo test -p craft-scaffold -- --ignored generation_compiles` | slice 5 |
| no leftovers | no `{{` in any generated file | `cargo test -p craft-scaffold no_placeholders_left` | slice 2+ |

Measured (warm machine, 2026-10-14): template `cargo test --bin` 12s;
`cargo check` of one generated app ~4 min cold / ~2s warm. Full generation
suite budget ~90s warm (excludes cold cargo fetch).
