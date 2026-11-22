//! Sundae brand mark, system tray icon, and Scoop-specific Lucide vector icons.
//!
//! All UI icons use `currentColor` stroke markup rendered via `bento::ui::icons::svg_icon`,
//! inheriting Light/Dark semantic tokens with zero hardcoded colors.

use std::sync::LazyLock;

pub use bento::ui::icons::*;
use nsh_core::element::ImageSrc;
use nsh_core::prelude::*;

const SUNDAE_MARK_SVG: &str = include_str!("../../assets/icons/sundae.svg");
const TRAY_ICON_PNG: &[u8] = include_bytes!("../../assets/tray-icon.png");

/// Shield with checkmark — "all up to date" empty state.
pub const SHIELD_CHECK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><path d="m9 12 2 2 4-4"/></svg>"#;

/// Globe — network & proxy configuration section.
pub const GLOBE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>"#;

/// Zap / lightning bolt — Aria2 multi-connection download acceleration.
pub const ZAP: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>"#;

/// Git branch — Scoop source repository & branch settings.
pub const GIT_BRANCH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><line x1="6" x2="6" y1="3" y2="15"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/></svg>"#;

/// Sliders — advanced & compatibility settings.
pub const SLIDERS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><line x1="4" x2="4" y1="21" y2="14"/><line x1="4" x2="4" y1="10" y2="3"/><line x1="12" x2="12" y1="21" y2="12"/><line x1="12" x2="12" y1="8" y2="3"/><line x1="20" x2="20" y1="21" y2="16"/><line x1="20" x2="20" y1="12" y2="3"/><line x1="2" x2="6" y1="14" y2="14"/><line x1="10" x2="14" y1="8" y2="8"/><line x1="18" x2="22" y1="16" y2="16"/></svg>"#;

/// File text — inspect local `manifest.json`.
pub const FILE_TEXT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" x2="8" y1="13" y2="13"/><line x1="16" x2="8" y1="17" y2="17"/></svg>"#;

/// Arrow right — version upgrade delta (`v1.0 → v2.0`).
pub const ARROW_RIGHT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="m12 5 7 7-7 7"/></svg>"#;

fn encode_svg_data_uri(svg_content: &str) -> String {
    let mut encoded = String::from("data:image/svg+xml;utf8,");
    for b in svg_content.bytes() {
        match b {
            b' ' | b'"' | b'#' | b'%' | b'<' | b'>' | b'\r' | b'\n' => {
                encoded.push('%');
                encoded.push_str(&format!("{b:02X}"));
            }
            _ => encoded.push(b as char),
        }
    }
    encoded
}

/// Sundae vector brand mark for the title bar and About dialog.
pub fn sundae_logo(size: f32) -> AnyElement {
    static URI: LazyLock<String> = LazyLock::new(|| encode_svg_data_uri(SUNDAE_MARK_SVG));
    svg(URI.as_str()).size(px(size)).into_element()
}

/// Data URI for the system tray icon (`assets/tray-icon.png`).
pub fn tray_icon_src() -> ImageSrc {
    png_data_uri(TRAY_ICON_PNG)
}
