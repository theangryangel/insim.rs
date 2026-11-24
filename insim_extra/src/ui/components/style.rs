//! A shared dark look for button UIs: dark panels with light text, which stay
//! readable over a bright track. kitcar's own panels use it.
//!
//! Each is a starting point, so chain on whatever else a button needs:
//! `style::title().align_left()`, `style::text().clickable()`. For LFS's own
//! light look, see `BtnStyle::style_title` and friends in insim.

use insim::insim::BtnStyle;

/// A dark panel, or a button on one, with light grey text.
pub fn panel() -> BtnStyle {
    BtnStyle::default().dark()
}

/// A heading: yellow on dark.
pub fn title() -> BtnStyle {
    panel().yellow()
}

/// Ordinary text: light grey on dark. The same as [`panel`].
pub fn text() -> BtnStyle {
    panel().light_grey()
}

/// Text to stand out, such as the viewer's own row: white on dark.
pub fn active() -> BtnStyle {
    panel().white()
}

/// Text to recede, such as a hint or something unavailable: grey on dark.
pub fn muted() -> BtnStyle {
    panel().grey()
}

/// The button that does the thing, such as close or confirm: green on dark.
pub fn action() -> BtnStyle {
    panel().green()
}
