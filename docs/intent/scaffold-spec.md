# Spec: interactive app scaffolding (`craft-scaffold`)

## Problema

Starting a new Rust + eframe/egui desktop app on craft-libs means
copy-pasting `templates/desktop-egui` and hand-renaming 5 identifiers, with no
guidance for the office-ribbon or minimal shells, no icon/padding contracts
enforced, and no style doc for agents to scale the app. Evidence: template
review 2026-10-14 (deformed icons from non-square paint rect, panel bodies
without the real apps' inner margins); audit `docs/ui-components-audit.md`
(families diverge: rail vs ribbon vs toolbar; shared are mechanisms only).

## Solución

A `crates/scaffold` CLI (`craft-scaffold`) that asks for app id, display name,
UI family and optional features (interactive or `--no-interactive` flags) and
renders a compilable app folder: family shell + shared wiring + `docs/style.md`
so agents extend it in-style. Zero new external dependencies.

## User stories

- [ ] As a developer I run `cargo run -p craft-scaffold`, answer 4 questions,
      and get a folder where `cargo test` passes, so I start from a
      professional shell instead of an empty window.
- [ ] As an agent I read the generated `docs/style.md` (Tokens, margins, icon
      slots, panel-id stability, file:line anchors) and add a panel without
      breaking the look, so no human re-review of spacing is needed.
- [ ] As a developer I re-run with `--no-interactive` flags in CI, so sample
      apps regenerate reproducibly.

## Contratos

```rust
/// Validated once at the CLI border; render is a pure function of Plan.
pub struct Plan {
    pub app_id: AppId,          // lowercase, [_a-z0-9-]; derives folder, env var, URI prefix
    pub display_name: String,   // non-empty; macOS menu + window title
    pub family: Family,         // Creative | Office | Minimal
    pub features: Features,     // { print: bool, cms: bool } — both default false
    pub out: PathBuf,           // must not exist OR be empty; created by render
}

pub enum ScaffoldError {
    InvalidAppId { got: String },   // code: INVALID_APP_ID
    OutputNotEmpty { path: PathBuf },// code: OUTPUT_NOT_EMPTY
    Render { file: String, why: String }, // code: RENDER
    Io { path: PathBuf, why: String },    // code: IO
}

/// Pure: Plan -> files. No disk, no network. Unit-testable.
pub fn render(plan: &Plan) -> Result<Vec<OutputFile>, ScaffoldError>;
pub struct OutputFile { pub path: String, pub bytes: Vec<u8> }
```

CLI surface: `craft-scaffold [--no-interactive] [--app-id ID]
[--display-name NAME] [--family creative|office|minimal] [--with-print]
[--with-cms] --out DIR`. Interactive mode asks exactly: app-id, display-name,
family, print?, cms? — in that order, defaults `myapp`/`My App`/`creative`/no/no.
`--out` defaults to `./<app-id>`; refuses non-empty dirs (no overwrite, ever).

Generated app contract (all families): `cargo fmt --check`, `cargo clippy
--all-targets -- -D warnings`, `cargo test` green on a warm machine;
`docs/style.md` present with Tokens table, margin constants, icon-slot rule
(square centered + `fit_to_exact_size`), tab contract (`tabs::fit` params),
toolbar contract (`fit_toolbar` + overflow menu), panel ids + stability note.

## Decisiones

| Fecha | Decisión | Razón | Alternativa descartada |
|---|---|---|---|
| 2026-10-14 | Render-texto con `{{placeholders}}`, no AST-splicing | output legible, 0 deps, test cierra el riesgo (generar+compilar las 3 familias) | `syn`+`prettyplease`: renombres garantizados pero 5× complejidad y formato reimpreso |
| 2026-10-14 | Las 3 familias en v1 | geometrías ya medidas en las 12 apps; costo marginal bajo sobre el mismo `Plan` | solo creative: deja fuera a Word/Grid/Deck-likes, el motivo del pedido |
| 2026-10-14 | Inglés generado | coincide con el corpus de la suite (12 apps, docs, comentarios) | español: mezcla inédita, peor para agentes |
| 2026-10-14 | `--out` dentro o fuera del repo, ambos | path-deps se reescriben a git pins cuando `--out` sale del repo | solo dentro: limita el uso real |
| 2026-10-14 | `templates/desktop-egui` convive como ejemplo curado | el generador lo puede regenerar pero el curado a mano sirve de referencia visual | reemplazarlo: pierde el ejemplo estable |
| 2026-10-14 | `style.md` uno por familia, a mano | precisión > mantenimiento; 3 archivos cortos con valores exactos | uno renderizado con deltas: genérico, menos útil al agente |

## No-objetivos

- Nuevo framework UI compartido o unión de `Tokens` en craft-libs.
- Motor de documento, undo, formato de archivo, fuentes, i18n, packaging,
  runner WASM en apps generadas.
- PNGs golden commiteados (solo el harness + instrucciones de generación local).
- Sobrescribir directorios no vacíos; migrar apps existentes al scaffold.

## ASSUMPTIONS

- A1: eframe/egui 0.36 siguen siendo las versiones pinned cuando se construya
  (revalidar contra `Cargo.lock` en F4; si subieron, adaptar features).
- A2: `cargo check` del output generado es el seam de test suficiente para
  "compila" (no se ejecuta la GUI en CI headless).
- A3: El usuario acepta que `templates/desktop-egui` conviva (decisión
  provisoria del brief; se ratifica en revisión F6).

## Seams de test (acordados por escrito, ideal 1)

- Seam primario: `render(&Plan) -> Vec<OutputFile>` (puro, sin disco).
  Comportamientos: las 3 familias renderizan sin error; todo `{{placeholder}}`
  queda sustituido (grep `{{` sobre el output = vacío); `--out` no vacío se
  rechaza antes de renderizar.
- Seam secundario (integración, 1 test): generar las 3 familias a dirs
  temporales y correr `cargo check` en cada una (requiere red en CI; se marca
  `#[ignore]` si no hay red... no: se exige red, es el mismo requisito que
  compilar cualquier app eframe).

## Mapa de módulos

- `scaffold-plan` (en `crates/scaffold/src/plan.rs`): `Plan`, `AppId`
  validación, `Family`, `Features`, `ScaffoldError`. Sin deps craft. Orden 1.
- `scaffold-render` (en `crates/scaffold/src/render.rs`): plantillas por
  familia + `render()`. Depende de `scaffold-plan`. Orden 2.
- `scaffold-cli` (en `crates/scaffold/src/main.rs`): preguntas interactivas
  (stdin/stdout, sin deps) + flags manuales + escritura a disco + reescritura
  path-deps→git-pins fuera del repo. Depende de ambos. Orden 3.
- `shell-common` (plantilla, no crate): `icon_slot`, `panel_frame`,
  doc-tab-strip con `tabs::fit`, toolbar con `fit_toolbar`. Orden 2 (con render).
- `shell-creative` / `shell-office` / `shell-minimal` (plantillas): barras,
  rail/ribbon, bodies ejemplo, `Tokens`, `docs/style.md` por familia. Orden 3.
