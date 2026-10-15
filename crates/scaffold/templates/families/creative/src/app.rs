#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

//! `eframe::App` shell in the Craft style: app bar, tool rail, document canvas,
//! right dock (layers / properties / history), status bar.
//!
//! Wiring order in `logic`: close decision → settings `tick` → quit hook last.
//! No painting in `logic`; no close decision in `ui` (it doesn't run hidden).

use std::time::Instant;

use craft_settings::{LoadReport, SaveStatus, SettingsStore};
use craft_ui::buttons::{ButtonVisuals, IconButton, IconButtonStyle};
use craft_ui::docking::{DockArea, DockContent, DockStyle, Layout, Node, Permissions};
use egui::{Color32, CornerRadius, FontId, Stroke, Ui, Vec2, WidgetText};

use crate::icons;
use crate::settings::{Brightness, Prefs, Workspace};

/// App bar height (compact, as in VectorCraft) and tool-rail width.
const APP_BAR_HEIGHT: f32 = 32.0;
const RAIL_WIDTH: f32 = 48.0;
const STATUS_HEIGHT: f32 = 24.0;
const TOOL_PITCH: f32 = 30.0;

/// All colors the shell uses. Never hard-code a color in a panel: add it here.
/// Values follow VectorCraft's MediumDark scale; other brightnesses derive by
/// shifting fills while keeping accent, selection and text roles fixed.
#[derive(Clone, Copy, Debug)]
pub struct Tokens {
    pub dark: bool,
    pub app_bar: Color32,
    pub panel: Color32,
    pub panel_darker: Color32,
    pub input: Color32,
    pub input_border: Color32,
    pub border: Color32,
    pub divider: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub icon: Color32,
    pub hover: Color32,
    pub tool_active: Color32,
    pub accent: Color32,
    pub accent_strong: Color32,
    pub row_selected: Color32,
    pub pasteboard: Color32,
    pub canvas: Color32,
}

impl Tokens {
    pub fn for_brightness(b: Brightness) -> Self {
        let hex = |s: u32| Color32::from_rgb((s >> 16) as u8, (s >> 8) as u8, s as u8);
        match b {
            Brightness::Dark => Self {
                dark: true,
                app_bar: hex(0x1e1e1e),
                panel: hex(0x282828),
                panel_darker: hex(0x202020),
                input: hex(0x171717),
                input_border: hex(0x3d3d3d),
                border: hex(0x141414),
                divider: hex(0x333333),
                text: hex(0xd8d8d8),
                text_dim: hex(0xa8a8a8),
                icon: hex(0xcfcfcf),
                hover: hex(0x3a3a3a),
                tool_active: hex(0x141414),
                accent: hex(0x378ef0),
                accent_strong: hex(0x1473e6),
                row_selected: hex(0x484b50),
                pasteboard: hex(0x161616),
                canvas: hex(0xf2f2f2),
            },
            Brightness::MediumDark => Self {
                dark: true,
                app_bar: hex(0x262626),
                panel: hex(0x323232),
                panel_darker: hex(0x282828),
                input: hex(0x1f1f1f),
                input_border: hex(0x464646),
                border: hex(0x1b1b1b),
                divider: hex(0x3f3f3f),
                text: hex(0xd8d8d8),
                text_dim: hex(0xa8a8a8),
                icon: hex(0xcfcfcf),
                hover: hex(0x444444),
                tool_active: hex(0x1d1d1d),
                accent: hex(0x378ef0),
                accent_strong: hex(0x1473e6),
                row_selected: hex(0x484b50),
                pasteboard: hex(0x1e1e1e),
                canvas: hex(0xffffff),
            },
            Brightness::MediumLight => Self {
                dark: false,
                app_bar: hex(0xd6d6d6),
                panel: hex(0xe4e4e4),
                panel_darker: hex(0xdcdcdc),
                input: hex(0xffffff),
                input_border: hex(0xb5b5b5),
                border: hex(0xa8a8a8),
                divider: hex(0xbfbfbf),
                text: hex(0x2b2b2b),
                text_dim: hex(0x5c5c5c),
                icon: hex(0x3a3a3a),
                hover: hex(0xcfcfcf),
                tool_active: hex(0xc2c2c2),
                accent: hex(0x1473e6),
                accent_strong: hex(0x0d5cbd),
                row_selected: hex(0xb9d3f2),
                pasteboard: hex(0xbcbcbc),
                canvas: hex(0xffffff),
            },
            Brightness::Light => Self {
                dark: false,
                app_bar: hex(0xe8e8e8),
                panel: hex(0xf4f4f4),
                panel_darker: hex(0xececec),
                input: hex(0xffffff),
                input_border: hex(0xc6c6c6),
                border: hex(0xbdbdbd),
                divider: hex(0xd0d0d0),
                text: hex(0x1f1f1f),
                text_dim: hex(0x555555),
                icon: hex(0x333333),
                hover: hex(0xdedede),
                tool_active: hex(0xd2d2d2),
                accent: hex(0x1473e6),
                accent_strong: hex(0x0d5cbd),
                row_selected: hex(0xb9d3f2),
                pasteboard: hex(0xc8c8c8),
                canvas: hex(0xffffff),
            },
        }
    }
}

/// Stable, nonlocalized panel ids. Never rename: `Layout<P>` persistence and
/// `Location` restore keys on them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Panel {
    Canvas,
    Layers,
    Properties,
    History,
}

impl Panel {
    /// Document-adjacent panels the View menu toggles. The canvas is
    /// `PROTECTED`: listed here so the menu can reveal it, but never closed.
    const TOGGLABLE: [Self; 3] = [Self::Layers, Self::Properties, Self::History];

    fn title(self) -> &'static str {
        match self {
            Self::Canvas => "Canvas",
            Self::Layers => "Layers",
            Self::Properties => "Properties",
            Self::History => "History",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Canvas => "shapes",
            Self::Layers => "layers",
            Self::Properties => "sliders",
            Self::History => "history",
        }
    }
}

/// Tools in the left rail: (icon name, stable id). The rail is single-column
/// at 48 pt with a 30 pt pitch, as in VectorCraft's Basic toolbar.
const TOOLS: &[(&str, &str)] = &[
    ("mouse-pointer-2", "selection"),
    ("square-dashed", "marquee"),
    ("pen", "pen"),
    ("brush", "brush"),
    ("type", "type"),
    ("shapes", "shapes"),
    ("hand", "hand"),
    ("zoom-in", "zoom"),
];

/// Panel bodies with borrows split from the layout: the `DockArea` snapshot
/// renders against `&Layout` while bodies mutate store/workspace/log only.
/// Structural `Action`s apply after `show` returns, never inside callbacks.
struct DockBodies<'a> {
    store: &'a mut SettingsStore<Prefs>,
    workspace: &'a mut Workspace,
    dirty: &'a mut bool,
    tokens: Tokens,
}

impl DockContent<Panel> for DockBodies<'_> {
    /// Every body runs inside the family panel frame (12/10/10/8 insets, as in
    /// VectorCraft's `dock.rs`): bare widgets on the dock surface touch the
    /// tab strip, so the frame is structural, not decoration. The canvas keeps
    /// its own pasteboard paint and only borrows the horizontal insets.
    fn body(&mut self, ui: &mut Ui, panel: &Panel) {
        if matches!(panel, Panel::Canvas) {
            return self.canvas_body(ui);
        }
        egui::Frame::NONE.inner_margin(egui::Margin { left: 12, right: 10, top: 10, bottom: 8 }).show(ui, |ui| match panel {
            Panel::Canvas => self.canvas_body(ui),
            Panel::Layers => self.layers_body(ui),
            Panel::Properties => self.properties_body(ui),
            Panel::History => self.history_body(ui),
        });
    }

    fn label(&mut self, _ui: &Ui, panel: &Panel, logical: &str, _font: &FontId, _color: Color32) -> WidgetText {
        let _ = logical;
        // Tab strip shows the panel icon; accessibility keeps the logical text.
        let _ = panel.icon();
        WidgetText::from(Panel::title(*panel))
    }
}

impl DockBodies<'_> {
    /// The document canvas: pasteboard fill, centered page, zoom label.
    /// Real apps paint their document here; the template paints a placeholder
    /// page so the shell reads as a creative editor, not an empty window.
    fn canvas_body(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let pasteboard = ui.available_rect_before_wrap();
        ui.painter().rect_filled(pasteboard, 0.0, t.pasteboard);
        let page_w = 420.0 * self.workspace.zoom;
        let page_h = 594.0 * self.workspace.zoom;
        let page = egui::Rect::from_center_size(pasteboard.center(), Vec2::new(page_w, page_h));
        ui.painter().rect_filled(page, 2.0, t.canvas);
        ui.painter().rect_stroke(page, 2.0, Stroke::new(1.0, t.border), egui::StrokeKind::Outside);
        // Dirty marker: accent-strong corner fold while the document is unsaved.
        if *self.dirty {
            let fold = egui::Rect::from_min_size(page.min, Vec2::splat(14.0));
            ui.painter().rect_filled(fold, 2.0, t.accent_strong);
        }
        ui.painter().text(
            page.center(),
            egui::Align2::CENTER_CENTER,
            format!("{}%\nReplace with the document view", (self.workspace.zoom * 100.0).round()),
            FontId::proportional(13.0),
            t.text_dim,
        );
        // Keep egui state for the canvas rect without visible chrome.
        ui.allocate_rect(pasteboard, egui::Sense::hover());
    }

    fn layers_body(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        ui.label(egui::RichText::new("Layers").strong().color(t.text));
        let mut rename = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (i, name) in ["Background", "Artwork", "Type", "Guides"].iter().enumerate() {
                let selected = i == 1;
                let row = ui.selectable_label(selected, *name);
                if selected {
                    ui.painter().rect_filled(row.rect, 3.0, t.row_selected);
                }
            }
            ui.separator();
            // `input` / `input_border` style a real text field, not a bare label.
            ui.visuals_mut().extreme_bg_color = t.input;
            let frame = egui::Frame::NONE.fill(t.input).stroke(Stroke::new(1.0, t.input_border)).inner_margin(4.0);
            frame.show(ui, |ui| {
                let mut name = String::from("New layer");
                if ui.text_edit_singleline(&mut name).lost_focus() {
                    rename = Some(name);
                }
            });
        });
        if rename.is_some() {
            *self.dirty = true;
        }
    }

    fn properties_body(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        ui.label(egui::RichText::new("Properties").strong().color(t.text));
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Brightness:").color(t.text_dim));
            egui::ComboBox::from_id_salt("brightness-combo").selected_text(self.store.get().appearance.brightness.label()).show_ui(ui, |ui| {
                for b in Brightness::ALL {
                    if ui.selectable_label(self.store.get().appearance.brightness == b, b.label()).clicked() {
                        self.store.update(|p| p.appearance.brightness = b);
                    }
                }
            });
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("UI scale:").color(t.text_dim));
            let mut scale = self.store.get().appearance.ui_scale;
            if ui.add(egui::Slider::new(&mut scale, 0.75..=2.0).step_by(0.05)).changed() {
                self.store.update(|p| p.appearance.ui_scale = scale);
                ui.ctx().set_pixels_per_point(scale);
            }
        });
        ui.separator();
        ui.label(egui::RichText::new("Fill").color(t.text_dim));
        ui.horizontal(|ui| {
            for fill in [Color32::from_rgb(0x37, 0x8e, 0xf0), Color32::from_rgb(0xd7, 0x37, 0x2f), Color32::from_rgb(0x2f, 0xa3, 0x4d)] {
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(22.0), egui::Sense::click());
                ui.painter().rect_filled(rect, 4.0, fill);
                if response.hovered() {
                    ui.painter().rect_stroke(rect, 4.0, Stroke::new(1.0, t.text), egui::StrokeKind::Outside);
                }
            }
        });
    }

    fn history_body(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        ui.label(egui::RichText::new("History").strong().color(t.text));
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for entry in ["Open Document", "Select All", "Set Fill", "Zoom 100%"] {
                ui.label(egui::RichText::new(entry).color(t.text_dim));
            }
        });
    }
}

pub struct {{display-name}}app {
    store: SettingsStore<Prefs>,
    workspace: Workspace,
    tokens: Tokens,
    /// Problems from `open` shown once in the status bar, never fatal.
    startup_report: Option<LoadReport>,
    layout: Layout<Panel>,
    /// Unsaved-document state. Real apps replace the bool with their document.
    dirty: bool,
    show_save_prompt: bool,
    discard_confirmed: bool,
    /// Last save error from `tick`, shown until the next successful save.
    save_error: Option<String>,
}

impl {{display-name}}app {
    pub fn new(ctx: &egui::Context, store: SettingsStore<Prefs>, report: LoadReport) -> Self {
        let tokens = Tokens::for_brightness(store.get().appearance.brightness);
        apply_theme(ctx, tokens);
        ctx.set_pixels_per_point(store.get().appearance.ui_scale);
        Self {
            store,
            workspace: Workspace::new(),
            tokens,
            startup_report: Some(report),
            layout: Layout {
                root: Some(Node::Split {
                    axis: craft_ui::layout::SplitAxis::Horizontal,
                    size: craft_ui::layout::SplitSize::FixedSecond(300.0),
                    first: Box::new(Node::Tabs { panels: vec![Panel::Canvas], active: 0 }),
                    second: Box::new(Node::Tabs { panels: vec![Panel::Layers, Panel::Properties, Panel::History], active: 0 }),
                }),
                floating: Vec::new(),
            },
            dirty: false,
            show_save_prompt: false,
            discard_confirmed: false,
            save_error: None,
        }
    }

    fn save_document(&mut self) {
        // Real apps: atomic_write the document here (craft-atomic), then:
        self.dirty = false;
        self.discard_confirmed = false;
    }

    /// Re-derive tokens from settings and re-apply the shared widget loop.
    /// Called after any brightness change from the Properties panel.
    fn refresh_theme(&mut self, ctx: &egui::Context) {
        self.tokens = Tokens::for_brightness(self.store.get().appearance.brightness);
        apply_theme(ctx, self.tokens);
    }
}

/// Apply tokens to egui's global style, after VectorCraft's `theme::apply`:
/// one complete `Visuals` installed with `set_visuals` (not per-theme
/// storage that leaves the previous look active), the shared five-state loop
/// for widget fills, per-state border exceptions, selection/accent roles,
/// shadows, spacing and text sizes. Fonts stay app-owned.
fn apply_theme(ctx: &egui::Context, tokens: Tokens) {
    let t = tokens;
    let mut v = if t.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = t.panel;
    v.window_fill = t.panel;
    v.extreme_bg_color = t.input;
    v.faint_bg_color = t.panel_darker;
    v.window_stroke = Stroke::new(1.0, t.border);
    v.window_corner_radius = CornerRadius::same(6);
    v.menu_corner_radius = CornerRadius::same(6);
    v.selection.bg_fill = t.accent.gamma_multiply(0.55);
    v.selection.stroke = Stroke::new(1.0, t.accent);
    v.hyperlink_color = t.accent;
    v.override_text_color = Some(t.text);
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 4], blur: 14, spread: 0, color: Color32::from_black_alpha(if t.dark { 120 } else { 60 }) };
    v.window_shadow = egui::epaint::Shadow { offset: [0, 8], blur: 28, spread: 0, color: Color32::from_black_alpha(if t.dark { 140 } else { 70 }) };
    let w = &mut v.widgets;
    craft_ui::theme::WidgetPalette { noninteractive: t.panel_darker, inactive: t.panel, hovered: t.hover, active: t.tool_active, open: t.hover }
        .apply_to(w, CornerRadius::same(4), Stroke::new(1.0, t.text));
    w.noninteractive.bg_stroke = Stroke::new(1.0, t.divider);
    w.inactive.bg_stroke = Stroke::NONE;
    w.hovered.bg_stroke = Stroke::new(1.0, t.input_border);
    w.active.bg_stroke = Stroke::new(1.0, t.accent);
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 5.0);
        s.spacing.button_padding = egui::vec2(6.0, 2.0);
        s.spacing.interact_size = egui::vec2(24.0, 24.0);
        s.spacing.menu_margin = egui::Margin::same(6);
        s.spacing.slider_width = 120.0;
        s.text_styles = [
            (egui::TextStyle::Small, egui::FontId::proportional(11.0)),
            (egui::TextStyle::Body, egui::FontId::proportional(13.0)),
            (egui::TextStyle::Button, egui::FontId::proportional(13.0)),
            (egui::TextStyle::Heading, egui::FontId::proportional(15.0)),
            (egui::TextStyle::Monospace, egui::FontId::monospace(11.5)),
        ]
        .into();
        s.animation_time = 0.08;
    });
}

fn tool_button_style(tokens: Tokens) -> IconButtonStyle {
    let normal = ButtonVisuals { fill: Color32::TRANSPARENT, stroke: Stroke::NONE, icon: tokens.icon };
    IconButtonStyle {
        normal,
        hovered: ButtonVisuals { fill: tokens.hover, ..normal },
        pressed: None,
        selected: ButtonVisuals { fill: tokens.tool_active, icon: tokens.text, ..normal },
        corner_radius: CornerRadius::same(4),
        focus_stroke: Stroke::new(1.0, tokens.accent),
    }
}

impl eframe::App for {{display-name}}app {
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
        // Outside-in, like the real apps: app bar, tool rail, dock, status.
        // The canvas fills whatever the dock leaves through the shared layout.
        self.app_bar(ui);
        self.tool_rail(ui);
        self.dock(ui);
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

impl {{display-name}}app {
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

    /// 32 pt application bar: brand mark, File/View menus, brightness switcher,
    /// save state at the right. Panel ids are global egui ids — keep stable.
    fn app_bar(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let frame = egui::Frame::NONE.fill(t.app_bar).inner_margin(egui::Margin { left: 8, right: 14, top: 0, bottom: 0 });
        egui::Panel::top("app-bar").exact_size(APP_BAR_HEIGHT).frame(frame.stroke(Stroke::new(1.0, t.border))).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                ui.painter().circle_filled(ui.cursor().min + Vec2::new(9.0, 11.0), 6.0, t.accent);
                ui.add_space(22.0);
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
                        for panel in Panel::TOGGLABLE {
                            let visible = self.layout.contains(&panel);
                            if ui.selectable_label(visible, panel.title()).clicked() {
                                let action = if visible {
                                    craft_ui::docking::Action::Close { panel }
                                } else {
                                    craft_ui::docking::Action::Open { panel, anchor: None }
                                };
                                if action_applies(&mut self.layout, action).is_err() {
                                    // Logged to the console panel in real apps.
                                }
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
                    ui.add_space(8.0);
                    for b in Brightness::ALL {
                        let selected = self.store.get().appearance.brightness == b;
                        if ui.selectable_label(selected, egui::RichText::new(b.label()).size(11.0).color(t.text)).clicked() {
                            self.store.update(|p| p.appearance.brightness = b);
                            self.refresh_theme(ui.ctx());
                        }
                    }
                });
            });
        });
    }

    /// 48 pt single-column tool rail with a 30 pt pitch. Selection is
    /// workspace state (not settings): the active tool id lives in `Workspace`.
    /// Tooltips carry the tool name; shortcuts plug in at the call site.
    fn tool_rail(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let style = tool_button_style(t);
        let frame = egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(0, 4)).stroke(Stroke::new(1.0, t.border));
        egui::Panel::left("tool-rail").resizable(false).exact_size(RAIL_WIDTH).frame(frame).show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            for (icon_name, tool_id) in TOOLS {
                let selected = self.workspace.active_tool == *tool_id;
                let response = IconButton::new(tool_id, Vec2::new(RAIL_WIDTH, TOOL_PITCH), &style).selected(selected).show(ui, |ui, rect, tint| {
                    icons::icon(icon_name).tint(tint).paint_at(ui, rect.shrink(7.0));
                });
                if response.clicked() {
                    self.workspace.active_tool = tool_id;
                }
                response.on_hover_text(*tool_id);
            }
        });
    }

    /// Central dock: protected canvas tab + Layers/Properties/History tabs.
    /// Actions apply AFTER rendering the snapshot, never inside callbacks.
    fn dock(&mut self, ui: &mut Ui) {
        let Self { layout, store, workspace, dirty, tokens, .. } = self;
        let mut content = DockBodies { store, workspace, dirty, tokens: *tokens };
        egui::CentralPanel::default().show(ui, |ui| {
            let style = DockStyle::from_ui(ui);
            let output = DockArea::new(ui.id().with("dock")).show(
                ui,
                layout,
                &style,
                |panel| panel.title().to_owned(),
                |panel| {
                    if *panel == Panel::Canvas { Permissions::PROTECTED } else { Permissions::default() }
                },
                |ui, panel| content.body(ui, panel),
            );
            if output.error.is_some() {
                // Real apps log to their console; the template drops it to
                // keep `log` state (and this borrow) minimal.
            }
            for action in output.actions {
                let _ = layout.apply(action);
            }
        });
    }

    /// 24 pt status bar: zoom control, tool readout, settings rule / errors.
    fn status_bar(&mut self, ui: &mut Ui) {
        let t = self.tokens;
        let frame = egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(8, 0)).stroke(Stroke::new(1.0, t.border));
        egui::Panel::bottom("status-bar").exact_size(STATUS_HEIGHT).frame(frame).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                egui::ComboBox::from_id_salt("zoom-combo").selected_text(format!("{:.0}%", self.workspace.zoom * 100.0)).width(72.0).show_ui(
                    ui,
                    |ui| {
                        for percent in [25.0, 50.0, 100.0, 200.0, 400.0] {
                            if ui.selectable_label(self.workspace.zoom * 100.0 == percent, format!("{percent:.0}%")).clicked() {
                                self.workspace.zoom = percent / 100.0;
                            }
                        }
                    },
                );
                ui.separator();
                ui.label(egui::RichText::new(self.workspace.active_tool).size(11.0).color(t.text_dim));
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

/// Apply one docking action, validating first. Separated so menu handlers
/// stay one-liners; real apps route this through their command/undo path.
fn action_applies(layout: &mut Layout<Panel>, action: craft_ui::docking::Action<Panel>) -> Result<(), craft_ui::docking::DockError> {
    layout.apply(action)
}
