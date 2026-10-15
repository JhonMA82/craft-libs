//! App-owned SVG catalog: one fallback icon. Replace with the app's own
//! attributed assets (see `docs/style.md`, icon-slot rule) and record each
//! asset's license in the app's ATTRIBUTION.md.
//!
//! craft-ui prepares trusted compiled-in bytes (literal `currentColor` +
//! `stroke-width` substitution). Parsing and rasterization stay with
//! `egui_extras`; the app installs its image loaders before drawing.
//!
//! One URI prefix per catalog/style combination in the same egui context:
//! egui caches by URI, so different bytes under one URI reuse stale content.

use std::sync::LazyLock;

use craft_ui::icons::{SvgIconSet, SvgStyle};

/// Trusted compiled-in catalog. The template ships one fallback glyph so the
/// generated app compiles and paints before real assets exist.
static ICONS: &[(&str, &[u8])] = &[(
    "square-dashed",
    b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\"><path d=\"M5 3H3v2M9 3h2M15 3h2M21 3h-2M5 21H3v-2M9 21h2M15 21h2M21 21h-2M3 5v2M3 11v2M3 17v2M21 5v2M21 11v2M21 17v2\"/></svg>",
)];

/// `currentColor` must be white-ish: icons tint at draw time from the active
/// `Tokens`. `stroke_width` replaces exactly `stroke-width="2"`.
const STYLE: SvgStyle = SvgStyle { current_color: "#ffffff", stroke_width: Some("1.75") };
const URI_PREFIX: &str = "bytes://{{app-id}}/icons/";
const FALLBACK: &str = "square-dashed";

/// Shared prepared bytes; lookups are cheap `Arc` clones.
static ICON_SET: LazyLock<SvgIconSet> = LazyLock::new(|| SvgIconSet::new(ICONS, STYLE, URI_PREFIX, FALLBACK));

/// Borrow the shared catalog.
pub fn icon_set() -> &'static SvgIconSet {
    &ICON_SET
}

/// Image source for `name`, falling back to [`FALLBACK`] (or empty bytes the
/// loader reports as an error when even the fallback is missing — no panic).
/// Paint through `icon_slot` (`docs/style.md`): square centered rect +
/// `fit_to_exact_size`, never `shrink()` on a non-square button rect.
pub fn icon(name: &str) -> egui::Image<'static> {
    egui::Image::new(icon_set().source(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_exists_and_unknown_names_do_not_panic() {
        assert!(icon_set().contains(FALLBACK));
        assert!(!icon_set().contains("no-such-icon"));
        let egui::ImageSource::Bytes { uri, bytes: _ } = icon_set().source("no-such-icon") else { return };
        assert_eq!(uri.as_ref(), format!("{URI_PREFIX}no-such-icon.svg"));
    }
}
