#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! Office `eframe::App` shell: title bar, contextual tabs, collapsible ribbon,
//! document view, status bar. Panel ids are global egui ids — keep stable.
//!
//! Ribbon geometry follows the Word/Grid/Deck family: 66pt content rows
//! (`RIBBON_H`), 13pt group separators, big buttons with label beneath and
//! small buttons in rows. Groups that don't fit move to the `…` overflow menu
//! through `craft_ui::layout::fit_toolbar` — commands are never dropped.
//!
//! Wiring order in `logic`: close decision → settings `tick` → quit hook last.
//! No painting in `logic`; no close decision in `ui` (it doesn't run hidden).

use std::time::Instant;

use craft_settings::{LoadReport, SaveStatus, SettingsStore};
use craft_ui::layout::fit_toolbar;
use egui::{Color32, CornerRadius, Stroke, Ui, Vec2};

use crate::icons;
use crate::settings::{Brightness, Prefs};

/// Ribbon content height (rows of small buttons + big buttons with labels).
pub const RIBBON_H: f32 = 66.0;
/// Collapsed ribbon strip: tabs stay reachable, groups hide.
const RIBBON_COLLAPSED_H: f32 = 32.0;
/// Width of a group separator (vertical rule + padding).
const GROUP_SEP: f32 = 13.0;
/// Width of the `…` overflow control.
const OVERFLOW_W: f32 = 28.0;

/// All colors the shell uses. Never hard-code a color in a panel: add it here.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub chrome: Color32,
    pub chrome_dark: Color32,
    pub panel: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub accent_text: Color32,
    pub hover: Color32,
}

impl Tokens {
    pub fn for_brightness(b: Brightness) -> Self {
        let hex = |s: u32| Color32::from_rgb((s >> 16) as u8, (s >> 8) as u8, s as u8);
        match b {
            Brightness::Dark => Self {
                chrome: hex(0x1e1e1e),
                chrome_dark: hex(0x161616),
                panel: hex(0x282828),
                border: hex(0x141414),
                text: hex(0xd8d8d8),
                text_dim: hex(0xa8a8a8),
                accent: hex(0x378ef0),
                accent_text: hex(0xffffff),
                hover: hex(0x3a3a3a),
            },
            Brightness::MediumDark => Self {
                chrome: hex(0x262626),
                chrome_dark: hex(0x1e1e1e),
                panel: hex(0x323232),
                border: hex(0x1b1b1b),
                text: hex(0xd8d8d8),
                text_dim: hex(0xa8a8a8),
                accent: hex(0x378ef0),
                accent_text: hex(0xffffff),
                hover: hex(0x444444),
            },
            Brightness::MediumLight => Self {
                chrome: hex(0xd6d6d6),
                chrome_dark: hex(0xc4c4c4),
                panel: hex(0xe4e4e4),
                border: hex(0xa8a8a8),
                text: hex(0x2b2b2b),
                text_dim: hex(0x5c5c5c),
                accent: hex(0x1473e6),
                accent_text: hex(0xffffff),
                hover: hex(0xcfcfcf),
            },
            Brightness::Light => Self {
                chrome: hex(0xe8e8e8),
                chrome_dark: hex(0xd8d8d8),
                panel: hex(0xf4f4f4),
                border: hex(0xbdbdbd),
                text: hex(0x1f1f1f),
                text_dim: hex(0x555555),
                accent: hex(0x1473e6),
                accent_text: hex(0xffffff),
                hover: hex(0xdedede),
            },
        }
    }
}

/// One ribbon group: label + natural width + buttons. Widths are measured at
/// runtime from galleys; the numbers here seed `fit_toolbar` before layout.
pub struct RibbonGroup {
    pub id: &'static str,
    pub label: &'static str,
    pub width: f32,
}

/// Example groups. Real apps build these from their command registry; widths
/// come from measured content, never from these seeds alone.
const GROUPS: &[RibbonGroup] = &[
    RibbonGroup { id: "clipboard", label: "Clipboard", width: 150.0 },
    RibbonGroup { id: "font", label: "Font", width: 260.0 },
    RibbonGroup { id: "paragraph", label: "Paragraph", width: 220.0 },
    RibbonGroup { id: "styles", label: "Styles", width: 200.0 },
];

/// Contextual tabs above the ribbon. Real apps derive these from selection.
const TABS: &[&str] = &["Home", "Insert", "Layout", "Review", "View"];

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
    active_tab: &'static str,
    ribbon_collapsed: bool,
}

impl {{AppStruct}} {
    pub fn new(ctx: &egui::Context, store: SettingsStore<Prefs>, report: LoadReport) -> Self {
        let tokens = Tokens::for_brightness(store.get().appearance.brightness);
        apply_theme(ctx, tokens);
        ctx.set_pixels_per_point(store.get().appearance.ui_scale);
        Self {
            store,
            tokens,
            startup_report: Some(report),
            dirty: false,
            show_save_prompt: false,
            discard_confirmed: false,
            save_error: None,
            active_tab: "Home",
            ribbon_collapsed: false,
        }
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

/// Apply the app's tokens through the shared five-state loop.
fn apply_theme(ctx: &egui::Context, tokens: Tokens) {
    let palette = craft_ui::theme::WidgetPalette {
        noninteractive: tokens.panel,
        inactive: tokens.panel,
        hovered: tokens.hover,
        active: tokens.hover,
        open: tokens.hover,
    };
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
        self.title_bar(ui);
        self.tabs(ui);
        self.ribbon(ui);
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

    fn title_bar(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let frame = egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(8, 0));
        egui::Panel::top("title-bar").exact_size(38.0).frame(frame.stroke(Stroke::new(1.0, t.border))).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new("{{display-name}}").strong().color(t.text));
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

    /// Contextual tab strip. Double-click collapses the ribbon below.
    fn tabs(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let frame = egui::Frame::NONE.fill(t.chrome);
        egui::Panel::top("tabs").exact_size(28.0).frame(frame).show(ui, |ui| {
            ui.horizontal(|ui| {
                for tab in TABS {
                    let selected = self.active_tab == *tab;
                    let resp = ui.selectable_label(selected, egui::RichText::new(*tab).color(if selected { t.text } else { t.text_dim }));
                    if resp.clicked() {
                        self.active_tab = tab;
                    }
                    if resp.double_clicked() {
                        self.ribbon_collapsed = !self.ribbon_collapsed;
                    }
                }
            });
        });
    }

    /// Ribbon with `fit_toolbar` overflow: visible groups keep natural widths
    /// in order; the rest move behind the `…` control's menu. The `…` control
    /// is always focusable and every overflow group stays reachable.
    fn ribbon(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let h = if self.ribbon_collapsed { RIBBON_COLLAPSED_H } else { RIBBON_H + 20.0 };
        let frame = egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(8, 4));
        egui::Panel::top("ribbon").exact_size(h).frame(frame.stroke(Stroke::new(1.0, t.border))).show(ui, |ui| {
            if self.ribbon_collapsed {
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new(format!("{} — double-click a tab to expand", self.active_tab)).size(11.0).color(t.text_dim));
                });
                return;
            }
            let available = ui.available_width();
            let natural: Vec<f32> = GROUPS.iter().map(|g| g.width + GROUP_SEP).collect();
            let fit = fit_toolbar(&natural, available, 8.0, OVERFLOW_W);
            let Some(fit) = fit else {
                ui.label(egui::RichText::new("Ribbon too narrow — widen the window.").color(t.text_dim));
                return;
            };
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                for (i, _) in &fit.shown {
                    let Some(group) = GROUPS.get(*i) else { continue };
                    self.ribbon_group(ui, group);
                    self.group_separator(ui);
                }
                if !fit.overflow.is_empty() {
                    self.overflow_control(ui, &fit.overflow);
                }
            });
        });
    }

    /// One group: icon row + label beneath. Real apps fill this from their
    /// command registry; the template paints placeholder buttons.
    fn ribbon_group(&mut self, ui: &mut Ui, group: &RibbonGroup) {
        let t = self.tokens;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                for (n, name) in ["save", "swatch", "sliders"].iter().enumerate() {
                    let slot = egui::Rect::from_min_size(ui.cursor().min, Vec2::splat(28.0));
                    let (_, resp) = ui.allocate_exact_size(Vec2::splat(28.0), egui::Sense::click());
                    if resp.hovered() {
                        ui.painter().rect_filled(slot, 3.0, t.hover);
                    }
                    icons::icon(name).tint(t.text).paint_at(ui, slot.shrink(5.0));
                    if resp.clicked() {
                        self.dirty = n == 0;
                    }
                    resp.on_hover_text(*name);
                }
            });
            ui.label(egui::RichText::new(group.label).size(10.0).color(t.text_dim));
        });
    }

    fn group_separator(&self, ui: &mut Ui) {
        let t = self.tokens;
        let rect = egui::Rect::from_min_size(ui.cursor().min, Vec2::new(1.0, RIBBON_H - 6.0));
        ui.painter().vline(rect.center().x, rect.y_range(), Stroke::new(1.0, t.border));
        ui.add_space(GROUP_SEP);
    }

    /// The `…` overflow control: enabled, focusable, every hidden group in its
    /// menu. Hiding commands silently is a bug — this menu is the contract.
    fn overflow_control(&mut self, ui: &mut Ui, overflow: &[usize]) {
        let t = self.tokens;
        let (_, resp) = ui.allocate_exact_size(Vec2::new(OVERFLOW_W, RIBBON_H - 6.0), egui::Sense::click());
        let label = egui::RichText::new("…").size(16.0).color(t.text);
        ui.painter().text(resp.rect.center(), egui::Align2::CENTER_CENTER, label.text().to_owned(), egui::FontId::proportional(16.0), t.text);
        let resp = resp.on_hover_text("More commands");
        egui::Popup::menu(&resp).show(|ui| {
            for i in overflow {
                if let Some(group) = GROUPS.get(*i)
                    && ui.button(group.label).clicked()
                {
                    ui.close();
                }
            }
        });
    }

    /// Document body wrapped in the family panel frame: 12/10/10/8 insets, so
    /// text never touches the window edge (see `docs/style.md`).
    fn document(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        egui::CentralPanel::default().show(ui, |ui| {
            egui::Frame::NONE.inner_margin(egui::Margin { left: 12, right: 10, top: 10, bottom: 8 }).show(ui, |ui| {
                ui.label(egui::RichText::new(format!("{} — {}", self.active_tab, self.dirty)).color(t.text_dim));
                ui.separator();
                ui.label(egui::RichText::new("Replace with the document view.").color(t.text));
                if ui.button("Make dirty").clicked() {
                    self.dirty = true;
                }
            });
        });
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let frame = egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(8, 0)).stroke(Stroke::new(1.0, t.border));
        egui::Panel::bottom("status-bar").exact_size(26.0).frame(frame).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new(self.active_tab).size(11.0).color(t.text_dim));
                ui.separator();
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
