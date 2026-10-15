# ADR-001: text-template rendering for `craft-scaffold`

- Status: Proposed
- Date: 2026-10-14
- Context: the generator must turn a `Plan` into a compilable app folder.
  Two structurally different designs: (A) text templates with
  `{{placeholders}}` rendered by pure string substitution; (B) parsing a base
  app with `syn`, applying typed recipes, reprinting with `prettyplease`.
- Decision: (A) text templates.
- Rationale: output stays human-legible with original formatting; zero new
  dependencies (workspace rule: boring design, no needless abstraction);
  the failure mode of A (missing placeholder) is closed by one test
  (generate all 3 families, grep `{{`, `cargo check` each output).
- Trade-off: renames are grep-guaranteed, not compiler-guaranteed; adding a
  new identifier requires touching every family template + the placeholder
  test. Accepted: families change rarely, the test fails loudly.
- Reversibility: medium. Switching to B later means rewriting `render.rs` +
  adding `syn`/`prettyplease`; `Plan` and the CLI surface are unaffected.
