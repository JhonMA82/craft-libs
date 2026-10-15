//! Shell composition goldens: render representative shell surfaces offscreen
//! and compare against locally stored references. Paired same-renderer
//! comparisons, not portable GPU files: run on one machine, review PNGs,
//! commit nothing.
//!
//! The harness exercises the shell's shared surfaces (panel frame, icon slot,
//! bar fills) through `egui::Context::run_ui` — no `eframe::Frame` needed
//! (its fields are crate-private) and no OS window opens.
//!
//! Workflow:
//! ```sh
//! SHELL_GOLDEN_DIR=/tmp/{{app-id}}-goldens cargo test --test shell_golden
//! ```
//! First run writes `-actual.png` files (no `-expected.png` yet) and passes.
//! Copy actual→expected after visual review; later runs fail on any changed
//! pixel. Needs a GPU-capable renderer (wgpu). Without one the test fails
//! cleanly with "renderer unavailable" — install mesa-vulkan-drivers on Linux
//! CI (`WGPU_BACKEND=vulkan`) like craft-libs `.github/workflows/ui.yml`.

use std::path::PathBuf;

use egui::{Context, FullOutput, Pos2, RawInput, Rect, Stroke, Ui};
use egui_kittest::{TestRenderer, wgpu::WgpuTestRenderer};
use image::RgbaImage;

use {{package_underscore}}::app::Tokens;
use {{package_underscore}}::icons::icon_set;
use {{package_underscore}}::settings::Brightness;

const WIDTH: f32 = 1100.0;
const HEIGHT: f32 = 750.0;
const SCALES: &[f32] = &[1.0, 2.0];

fn context(scale: f32) -> Context {
    let ctx = Context::default();
    ctx.set_pixels_per_point(scale);
    // Empty fonts (no glyphs) keep the comparison font-independent: text
    // layout differs per machine, painted shapes do not. Labels below are
    // layout spacers, never compared content — typography is app-tested.
    ctx.set_fonts(egui::FontDefinitions::empty());
    ctx.all_styles_mut(|style| {
        style.animation_time = 0.0;
        style.visuals.text_cursor.blink = false;
    });
    // SVG rasterization needs the app's own loader + catalog bytes, exactly
    // like production (`main.rs` installs them; here the harness does).
    // `source()` already returns `egui::load::Bytes`: pass through.
    egui_extras::install_image_loaders(&ctx);
    let egui::ImageSource::Bytes { uri, bytes } = icon_set().source("square-dashed") else { return ctx };
    ctx.include_bytes(uri, bytes);
    ctx
}

fn raw_input(frame: usize) -> RawInput {
    RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(WIDTH, HEIGHT))), time: Some(frame as f64 / 60.0), ..Default::default() }
}

/// Representative shell surfaces with the family contracts applied:
/// bar fills, the 12/10/10/8 panel frame, a square icon slot with a real
/// rasterized SVG. Shapes and fills are compared; text is a layout spacer.
fn paint_shell(ui: &mut Ui, tokens: Tokens) {
    ui.painter().rect_filled(ui.available_rect_before_wrap(), 0.0, tokens.panel);
    // Deterministic 32pt bar: `horizontal_centered` needs real font metrics
    // (empty harness fonts expand it past the screen and push everything
    // below out of view). Fixed allocation keeps geometry font-independent.
    egui::Frame::NONE.fill(tokens.bar).inner_margin(egui::Margin { left: 8, right: 14, top: 0, bottom: 0 }).show(ui, |ui| {
        let bar = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 32.0));
        ui.allocate_rect(bar, egui::Sense::hover());
        ui.painter().rect_filled(bar, 0.0, tokens.bar);
    });
    egui::Frame::NONE.inner_margin(egui::Margin { left: 12, right: 10, top: 10, bottom: 8 }).show(ui, |ui| {
        ui.label(egui::RichText::new("Panel body").color(tokens.text));
        let slot = egui::Rect::from_min_size(ui.cursor().min, egui::Vec2::splat(22.0));
        ui.painter().rect_stroke(slot, 3.0, Stroke::new(1.0, tokens.border), egui::StrokeKind::Inside);
        {{package_underscore}}::icons::icon("square-dashed").tint(tokens.text).paint_at(ui, slot);
    });
}

fn render_case(scale: f32, brightness: Brightness) -> RgbaImage {
    let mut renderer = WgpuTestRenderer::new();
    let ctx = context(scale);
    let tokens = Tokens::for_brightness(brightness);
    let mut output = FullOutput::default();
    for frame in 0..3 {
        output = ctx.run_ui(raw_input(frame), |ui| paint_shell(ui, tokens));
        renderer.handle_delta(&mut output.textures_delta);
    }
    renderer.render(&ctx, &output).unwrap_or_else(|_| panic!("offscreen renderer unavailable (see module docs)"))
}

fn compare(name: &str, dir: &Option<PathBuf>, actual: &RgbaImage) {
    let Some(dir) = dir else { return };
    std::fs::create_dir_all(dir).unwrap_or_else(|e| panic!("golden dir: {e}"));
    let actual_path = dir.join(format!("{name}-actual.png"));
    actual.save(&actual_path).unwrap_or_else(|e| panic!("save actual: {e}"));
    let expected_path = dir.join(format!("{name}-expected.png"));
    if !expected_path.exists() {
        return;
    }
    let expected = image::open(&expected_path).unwrap_or_else(|e| panic!("open expected: {e}")).to_rgba8();
    assert_eq!(expected.dimensions(), actual.dimensions(), "{name}: size changed");
    let mut changed = 0;
    let mut difference = RgbaImage::new(actual.width(), actual.height());
    for ((e, a), d) in expected.pixels().zip(actual.pixels()).zip(difference.pixels_mut()) {
        if e != a {
            changed += 1;
            *d = image::Rgba([255, 0, 128, 255]);
        }
    }
    difference.save(dir.join(format!("{name}-difference.png"))).unwrap_or_else(|e| panic!("save diff: {e}"));
    assert_eq!(changed, 0, "{name}: {changed} pixels changed (see -difference.png)");
}

#[test]
fn shell_composition_matches_local_references() {
    let dir = std::env::var_os("SHELL_GOLDEN_DIR").map(PathBuf::from);
    for scale in SCALES {
        for brightness in Brightness::ALL {
            let name = format!("shell-s{scale}-{:?}", brightness);
            let actual = render_case(*scale, brightness);
            compare(&name, &dir, &actual);
        }
    }
}
