//! Sundae's styled UI layer: re-exports shared `bento::ui` controls alongside
//! Sundae-specific vector icons and the virtualized terminal drawer.

pub mod icons;
pub mod terminal_drawer;

pub use bento::ui::*;
pub use terminal_drawer::terminal_drawer;
