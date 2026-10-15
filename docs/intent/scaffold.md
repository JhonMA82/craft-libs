# Intent: interactive app scaffolding (`craft-scaffold`)

## Outcome

A CLI in this repo asks what's needed (app id, display name, UI family,
optional features) and creates a compilable app folder with detailed docs, so
an agent can scale the app following the same interface style. English
generated code, docs and comments (matches the 12-app suite corpus).

## User

A developer (or agent) starting a new Rust + eframe/egui desktop app on top of
craft-libs, wanting a professional-looking base with layouts and widgets ready.

## Why now

The `templates/desktop-egui` copy-paste starter exists but: it covers only the
VectorCraft-like creative shell, its icons deform (non-square paint rect), its
panel bodies lack the real apps' inner margins, and it teaches neither
`tabs::fit`/`fit_toolbar` nor the office-ribbon geometry. craft-libs itself
deliberately shares mechanisms, never look (audit §Themes) — so the look must
be measured per family (done: 12 app sources in /tmp/*-ref) and baked into the
generated code + style docs.

## Success

- `cargo run -p craft-scaffold -- --no-interactive --app-id myapp --family creative --out /tmp/myapp`
  produces a folder where `cargo clippy --all-targets -- -D warnings`,
  `cargo fmt --check` and `cargo test` pass on a warm machine.
- The generated app opens with the family's professional composition
  (creative: 32pt bar + 48pt rail + canvas + dock + 24pt status;
  office: title-bar + collapsible ribbon + document + status;
  minimal: menu + status) with correct icon aspect, panel margins, tabs fit
  and toolbar overflow from the first frame.
- `docs/style.md` in the generated app documents Tokens, margins, icon slots,
  tab/toolbar contracts and panel-id stability with file:line anchors, so an
  agent adding a panel/widget keeps the look without asking.

## Non-goals / anti-goals

- No new shared UI framework or theme union in craft-libs (audit forbids it).
- No document engine, undo, file format, fonts, i18n, packaging, WASM runner
  in generated apps (placeholders with real geometry only).
- No committed golden PNGs (harness ships, references generate locally).
- No `cargo-generate` dependency; zero new external deps for the CLI.

## Constraint

- Workspace rules hold: never-crash lints, `unsafe` forbidden, fmt + strict
  clippy + tests green. The CLI ships as `crates/scaffold` (`craft-scaffold`),
  the family shells as data/templates it renders — never as a runtime
  dependency of generated apps (generated code path-deps only craft-* libs).

## Open questions for F2

1. `--out` inside vs outside the repo (path-deps `../../crates/*` only work
   inside; outside needs git pins or a `--publish` rewrite)?
2. Keep evolving `templates/desktop-egui` in parallel, or replace it with the
   generator's `--family creative` output?
3. Generated `docs/style.md`: hand-written per family (3 files) vs rendered
   from one source with family deltas?
