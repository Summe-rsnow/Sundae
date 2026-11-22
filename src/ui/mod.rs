//! Sundae's styled UI layer: re-exports shared `bento::ui` controls alongside
//! Sundae-specific vector icons and the virtualized terminal drawer.

pub mod icons;
pub mod terminal_drawer;

pub use bento::ui::*;
pub use terminal_drawer::terminal_drawer;

/// Monospace font family from the active theme, with a safe platform fallback.
pub fn mono_font() -> nsh_core::prelude::SharedString {
    nsh_base::theme()
        .font_mono
        .clone()
        .unwrap_or_else(|| "monospace".into())
}
