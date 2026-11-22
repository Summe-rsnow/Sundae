//! Sundae brand mark, system tray icon, and re-exports of shared Bento stroke icons.

use std::sync::OnceLock;

pub use bento::ui::icons::*;
use nsh_core::element::ImageSrc;
use nsh_core::prelude::*;

pub const BRAND_MARK: &str = include_str!("../../assets/logo-mark.svg");
const TRAY_ICON_PNG: &[u8] = include_bytes!("../../assets/tray-icon.png");

/// Renders the full-color Sundae brand mark.
pub fn brand_mark(size: f32) -> impl IntoElement {
    bento::ui::icons::brand_mark_svg(BRAND_MARK, size)
}

/// Returns the cached full-color 64x64 data URI for the system tray icon.
pub fn tray_icon_src() -> ImageSrc {
    static DATA_URI: OnceLock<ImageSrc> = OnceLock::new();
    DATA_URI
        .get_or_init(|| bento::ui::png_data_uri(TRAY_ICON_PNG))
        .clone()
}
