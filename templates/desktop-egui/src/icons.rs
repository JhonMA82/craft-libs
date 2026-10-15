//! App-owned SVG catalog: 14 MIT-licensed outline tool icons (Lucide-style,
//! authored for this template, 24×24 grid, `currentColor` stroke 2).
//!
//! craft-ui prepares trusted compiled-in bytes (literal `currentColor` +
//! `stroke-width` substitution) and returns egui image sources. Parsing and
//! rasterization stay with `egui_extras`; install its image loaders (the
//! `svg` feature) before drawing.
//!
//! One URI prefix per catalog/style combination in the same egui context:
//! egui caches by URI, so different bytes under one URI reuse stale content.
//! Replacing this catalog with the app's own assets: keep `STYLE` in sync with
//! how the new SVGs spell their stroke, and record each asset's license in the
//! app's ATTRIBUTION.md.

use std::sync::LazyLock;

use craft_ui::icons::{SvgIconSet, SvgStyle};

macro_rules! icon {
    ($name:literal, $body:literal) => {
        (
            $name,
            concat!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\"",
                " fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
                $body,
                "</svg>"
            )
            .as_bytes(),
        )
    };
}

/// Tool + panel icons. Names follow the Lucide originals they are drawn from.
static ICONS: &[(&str, &[u8])] = &[
    icon!("mouse-pointer-2", "<path d=\"m4 4 7.07 17 2.51-7.39L21 11.07z\"/>"),
    icon!(
        "square-dashed",
        "<path d=\"M5 3H3v2M9 3h2M15 3h2M21 3h-2M5 21H3v-2M9 21h2M15 21h2M21 21h-2M3 5v2M3 11v2M3 17v2M21 5v2M21 11v2M21 17v2\"/>"
    ),
    icon!(
        "pen",
        "<path d=\"M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z\"/><path d=\"m15 5 4 4\"/>"
    ),
    icon!(
        "brush",
        "<path d=\"m14.622 17.897-10.68-2.913\"/><path d=\"M18.376 2.622a1 1 0 1 1 3.002 3.002L7.368 19.635a2 2 0 0 1-.855.506l-2.872.838a.5.5 0 0 1-.62-.62l.838-2.872a2 2 0 0 1 .506-.854z\"/>"
    ),
    icon!("type", "<path d=\"M4 7V4h16v3\"/><path d=\"M9 20h6\"/><path d=\"M12 4v16\"/>"),
    icon!(
        "shapes",
        "<path d=\"M8.3 10a.7.7 0 0 1-.626-1.079L11.4 3a.7.7 0 0 1 1.198-.043L16.3 8.9a.7.7 0 0 1-.572 1.1Z\"/><rect x=\"3\" y=\"14\" width=\"7\" height=\"7\" rx=\"1\"/><circle cx=\"17.5\" cy=\"17.5\" r=\"3.5\"/>"
    ),
    icon!(
        "hand",
        "<path d=\"M18 11V6a2 2 0 0 0-2-2a2 2 0 0 0-2 2\"/><path d=\"M14 10V4a2 2 0 0 0-2-2a2 2 0 0 0-2 2v2\"/><path d=\"M10 10.5V6a2 2 0 0 0-2-2a2 2 0 0 0-2 2v8\"/><path d=\"M18 8a2 2 0 1 1 4 0v6a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.86-5.99-2.34l-3.6-3.6a2 2 0 0 1 2.83-2.82L7 15\"/>"
    ),
    icon!(
        "zoom-in",
        "<circle cx=\"11\" cy=\"11\" r=\"8\"/><line x1=\"21\" x2=\"16.65\" y1=\"21\" y2=\"16.65\"/><line x1=\"11\" x2=\"11\" y1=\"8\" y2=\"14\"/><line x1=\"8\" x2=\"14\" y1=\"11\" y2=\"11\"/>"
    ),
    icon!(
        "layers",
        "<path d=\"m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z\"/><path d=\"m22 17.65-9.17 4.16a2 2 0 0 1-1.66 0L2 17.65\"/><path d=\"m22 12.65-9.17 4.16a2 2 0 0 1-1.66 0L2 12.65\"/>"
    ),
    icon!("swatch", "<rect x=\"3\" y=\"3\" width=\"18\" height=\"18\" rx=\"2\"/><path d=\"M3 9h18M9 21V9\"/>"),
    icon!(
        "sliders",
        "<line x1=\"21\" x2=\"14\" y1=\"4\" y2=\"4\"/><line x1=\"10\" x2=\"3\" y1=\"4\" y2=\"4\"/><line x1=\"21\" x2=\"12\" y1=\"12\" y2=\"12\"/><line x1=\"8\" x2=\"3\" y1=\"12\" y2=\"12\"/><line x1=\"21\" x2=\"16\" y1=\"20\" y2=\"20\"/><line x1=\"12\" x2=\"3\" y1=\"20\" y2=\"20\"/><line x1=\"14\" x2=\"14\" y1=\"2\" y2=\"6\"/><line x1=\"8\" x2=\"8\" y1=\"10\" y2=\"14\"/><line x1=\"16\" x2=\"16\" y1=\"18\" y2=\"22\"/>"
    ),
    icon!("history", "<path d=\"M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8\"/><path d=\"M3 3v5h5\"/><path d=\"M12 7v5l4 2\"/>"),
    icon!("terminal", "<path d=\"m4 17 6-6-6-6\"/><path d=\"M12 19h8\"/>"),
    icon!(
        "save",
        "<path d=\"M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z\"/><path d=\"M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7\"/><path d=\"M7 3v4a1 1 0 0 0 1 1h7\"/>"
    ),
];

/// `currentColor` must be white-ish: icons tint at draw time from the active
/// `Tokens`, and the shared substitution is literal text replacement.
/// `stroke_width` replaces exactly `stroke-width="2"` — the width above.
const STYLE: SvgStyle = SvgStyle { current_color: "#ffffff", stroke_width: Some("1.75") };
const URI_PREFIX: &str = "bytes://template-app/icons/";
const FALLBACK: &str = "square-dashed";

/// All icon names, for tests to assert catalog coverage.
#[cfg(test)]
pub const NAMES: &[&str] = &[
    "mouse-pointer-2",
    "square-dashed",
    "pen",
    "brush",
    "type",
    "shapes",
    "hand",
    "zoom-in",
    "layers",
    "swatch",
    "sliders",
    "history",
    "terminal",
    "save",
];

/// Shared prepared bytes; lookups are cheap `Arc` clones.
static ICON_SET: LazyLock<SvgIconSet> = LazyLock::new(|| SvgIconSet::new(ICONS, STYLE, URI_PREFIX, FALLBACK));

/// Borrow the shared catalog.
pub fn icon_set() -> &'static SvgIconSet {
    &ICON_SET
}

/// Image source for `name`, falling back to [`FALLBACK`] (or empty bytes the
/// loader reports as an error when even the fallback is missing — no panic).
/// Set tint + size on the returned image at the call site.
pub fn icon(name: &str) -> egui::Image<'static> {
    egui::Image::new(icon_set().source(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_covers_every_named_icon_plus_fallback() {
        for name in NAMES {
            assert!(icon_set().contains(name), "missing icon: {name}");
        }
        assert!(icon_set().contains(FALLBACK));
        assert!(!icon_set().contains("no-such-icon"));
    }

    #[test]
    fn fallback_uses_the_requested_name_in_its_uri() {
        // Matches the PhotoCraft/VectorCraft cache-key policy craft-ui preserves.
        let egui::ImageSource::Bytes { uri, bytes: _ } = icon_set().source("no-such-icon") else { return };
        assert_eq!(uri.as_ref(), format!("{URI_PREFIX}no-such-icon.svg"));
    }
}
