#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! Minimal `eframe::App` shell: menu bar, document view, status bar.
//! Panel ids (`menu-bar`, `status-bar`) are global egui ids — keep stable.
//!
//! Wiring order in `logic`: close decision → settings `tick` → quit hook last.
//! No painting in `logic`; no close decision in `ui` (it doesn't run hidden).

use std::time::Instant;

use craft_settings::{LoadReport, SaveStatus, SettingsStore};
use craft_ui::theme::WidgetPalette;
use egui::{Color32, CornerRadius, Stroke, Ui};

use crate::settings::{Brightness, Prefs};

/// All colors the shell uses. Never hard-code a color in a panel: add it here.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub bar: Color32,
    pub panel: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
}

impl Tokens {
    pub fn for_brightness(b: Brightness) -> Self {
        let hex = |s: u32| Color32::from_rgb((s >> 16) as u8, (s >> 8) as u8, s as u8);
        match b {
            Brightness::Dark => Self {
                bar: hex(0x1e1e1e),
                panel: hex(0x282828),
                border: hex(0x141414),
                text: hex(0xd8d8d8),
                text_dim: hex(0xa8a8a8),
                accent: hex(0x378ef0),
            },
            Brightness::MediumDark => Self {
                bar: hex(0x262626),
                panel: hex(0x323232),
                border: hex(0x1b1b1b),
                text: hex(0xd8d8d8),
                text_dim: hex(0xa8a8a8),
                accent: hex(0x378ef0),
            },
            Brightness::MediumLight => Self {
                bar: hex(0xd6d6d6),
                panel: hex(0xe4e4e4),
                border: hex(0xa8a8a8),
                text: hex(0x2b2b2b),
                text_dim: hex(0x5c5c5c),
                accent: hex(0x1473e6),
            },
            Brightness::Light => Self {
                bar: hex(0xe8e8e8),
                panel: hex(0xf4f4f4),
                border: hex(0xbdbdbd),
                text: hex(0x1f1f1f),
                text_dim: hex(0x555555),
                accent: hex(0x1473e6),
            },
        }
    }

    /// The five widget-state fills shared via craft-ui. Borders, expansion,
    /// selection and every other visual stay egui's.
    fn widget_palette(self) -> WidgetPalette {
        WidgetPalette { noninteractive: self.panel, inactive: self.panel, hovered: self.bar, active: self.bar, open: self.bar }
    }
}

pub struct {{AppStruct}} {
    store: SettingsStore<Prefs>,
    tokens: Tokens,
    /// Problems from `open` shown once in the status bar, never fatal.
    startup_report: Option<LoadReport>,
    /// Unsaved-document state. Real apps replace the bool with their document.
    dirty: bool,
    show_save_prompt: bool,
    discard_confirmed: bool,
    /// Last save error from `tick`, shown until the next successful save.
    save_error: Option<String>,
}

impl {{AppStruct}} {
    pub fn new(ctx: &egui::Context, store: SettingsStore<Prefs>, report: LoadReport) -> Self {
        let tokens = Tokens::for_brightness(store.get().appearance.brightness);
        apply_theme(ctx, tokens);
        ctx.set_pixels_per_point(store.get().appearance.ui_scale);
        Self { store, tokens, startup_report: Some(report), dirty: false, show_save_prompt: false, discard_confirmed: false, save_error: None }
    }

    fn save_document(&mut self) {
        // Real apps: atomic_write the document here (craft-atomic), then:
        self.dirty = false;
        self.discard_confirmed = false;
    }

    /// Re-derive tokens from settings and re-apply the shared widget loop.
    fn refresh_theme(&mut self, ctx: &egui::Context) {
        self.tokens = Tokens::for_brightness(self.store.get().appearance.brightness);
        apply_theme(ctx, self.tokens);
    }
}

/// Apply the app's tokens through the shared five-state loop. Fonts, complete
/// palettes, selection colors and spacing stay app-owned.
fn apply_theme(ctx: &egui::Context, tokens: Tokens) {
    let palette = tokens.widget_palette();
    let foreground = Stroke::new(1.0, tokens.text);
    ctx.all_styles_mut(|style| palette.apply_to(&mut style.visuals.widgets, CornerRadius::same(4), foreground));
    ctx.set_visuals_of(egui::Theme::Dark, egui::Visuals { panel_fill: tokens.panel, window_fill: tokens.panel, ..egui::Visuals::dark() });
    ctx.set_visuals_of(egui::Theme::Light, egui::Visuals { panel_fill: tokens.panel, window_fill: tokens.panel, ..egui::Visuals::light() });
}

impl eframe::App for {{AppStruct}} {
    /// Per-frame non-painting work. Order matters:
    /// 1. app close decision (unsaved check → `CancelClose` + prompt state),
    /// 2. settings `tick` (+ `request_repaint_after` so idle apps still save),
    /// 3. `craft_quit::on_close_request` LAST (sees the app's `CancelClose`).
    ///
    /// Runs even while minimized/hidden, when `ui` does not.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && !self.discard_confirmed && !self.show_save_prompt {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_save_prompt = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        }

        let tick = self.store.tick(Instant::now());
        match &tick.status {
            SaveStatus::Failed(e) => self.save_error = Some(e.to_string()),
            SaveStatus::Saved => {
                self.save_error = None;
                self.refresh_theme(ctx);
            }
            SaveStatus::Idle | SaveStatus::Pending => {}
        }
        if let Some(deadline) = tick.next_deadline {
            ctx.request_repaint_after(deadline.saturating_duration_since(Instant::now()));
        }

        craft_quit::on_close_request(ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        if self.show_save_prompt {
            self.save_prompt(ui);
        }
        self.menu_bar(ui);
        self.document(ui);
        self.status_bar(ui);
    }

    /// Persist settings on exit. winit's `applicationWillTerminate:` runs this
    /// before the macOS process ends; code after `run_native` in `main` does
    /// NOT run on that path, so everything durable belongs here.
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(e) = self.store.flush() {
            eprintln!("settings flush: {e}");
        }
    }
}

impl {{AppStruct}} {
    fn save_prompt(&mut self, ui: &mut Ui) {
        egui::Window::new("Unsaved changes").collapsible(false).resizable(false).show(ui.ctx(), |ui| {
            ui.label("The document has unsaved changes.");
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    self.save_document();
                    self.show_save_prompt = false;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button("Don't save").clicked() {
                    self.discard_confirmed = true;
                    self.show_save_prompt = false;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button("Cancel").clicked() {
                    self.show_save_prompt = false;
                    self.discard_confirmed = false;
                }
            });
        });
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let frame = egui::Frame::NONE.fill(t.bar).inner_margin(egui::Margin { left: 8, right: 14, top: 0, bottom: 0 });
        egui::Panel::top("menu-bar").exact_size(32.0).frame(frame.stroke(Stroke::new(1.0, t.border))).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button(egui::RichText::new("File").color(t.text), |ui| {
                        if ui.button("Save").clicked() {
                            self.save_document();
                            ui.close();
                        }
                        if ui.button("Quit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                            ui.close();
                        }
                    });
                    ui.menu_button(egui::RichText::new("View").color(t.text), |ui| {
                        for b in Brightness::ALL {
                            if ui.selectable_label(self.store.get().appearance.brightness == b, b.label()).clicked() {
                                self.store.update(|p| p.appearance.brightness = b);
                                self.refresh_theme(ui.ctx());
                                ui.close();
                            }
                        }
                    });
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(if self.dirty { "●" } else { "Saved" }).size(11.0).color(if self.dirty {
                        t.accent
                    } else {
                        t.text_dim
                    }));
                });
            });
        });
    }

    /// Document body wrapped in the family panel frame: 12/10/10/8 insets, so
    /// text never touches the window edge (see `docs/style.md`).
    fn document(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        egui::CentralPanel::default().show(ui, |ui| {
            egui::Frame::NONE.inner_margin(egui::Margin { left: 12, right: 10, top: 10, bottom: 8 }).show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Icon slot: square centered rect + exact fit + theme tint.
                    // Never `shrink()` a non-square rect (deforms the SVG).
                    let slot = egui::Rect::from_center_size(ui.cursor().min + egui::Vec2::splat(11.0), egui::Vec2::splat(22.0));
                    crate::icons::icon("square-dashed").tint(t.text_dim).paint_at(ui, slot);
                    ui.add_space(26.0);
                    ui.label(egui::RichText::new("{{display-name}}").heading().color(t.text));
                });
                ui.label(egui::RichText::new("Replace with the document view.").color(t.text_dim));
                if ui.button("Make dirty").clicked() {
                    self.dirty = true;
                }
            });
        });
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let frame = egui::Frame::NONE.fill(t.bar).inner_margin(egui::Margin::symmetric(8, 0)).stroke(Stroke::new(1.0, t.border));
        egui::Panel::bottom("status-bar").exact_size(24.0).frame(frame).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                if let Some(report) = self.startup_report.take() {
                    let dir = report.dir.as_ref().map_or("memory".to_owned(), |d| d.display().to_string());
                    let problems = report.has_problems();
                    ui.label(egui::RichText::new(format!("{dir} ({})", report.rule)).size(11.0).color(if problems {
                        Color32::RED
                    } else {
                        t.text_dim
                    }));
                    ui.separator();
                }
                if let Some(e) = &self.save_error {
                    ui.label(egui::RichText::new(format!("save failed (retrying): {e}")).size(11.0).color(Color32::RED));
                } else if self.store.is_read_only() {
                    ui.label(egui::RichText::new("read-only settings").size(11.0).color(t.text_dim));
                }
            });
        });
    }
}
