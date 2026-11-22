//! Sundae: Native desktop Scoop package manager built on NSH and Bento.

pub mod dialogs;
pub mod scoop;
pub mod shell;
pub mod store;
pub mod ui;
pub mod views;

use bento::shell::init_desktop_window;
use nsh_base::theme;
use nsh_core::prelude::*;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The app's UI set (`nsh.json` `"rust"`).
#[native_module("sundae/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}

/// The application's root component (`nsh.json` `"root"`).
#[component]
pub fn App() -> impl IntoElement {
    init_desktop_window(880.0, 600.0, 48.0);
    store::load();

    view()
        .flex_1()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .p_safe_area()
        .child(shell::screen())
}
