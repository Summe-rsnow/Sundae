//! The app's Rust UI: its root component, which `nsh run`, `nsh package`
//! and `cargo run` mount.

pub mod scoop;
pub mod shell;
pub mod store;
pub mod ui;
pub mod views;

use nsh_core::prelude::*;
use shell::shell_view;
use store::AppState;

/// The app's UI set (`nsh.json` `"rust"`): what a TSX entry may import from
/// `sundae/native` (`README.md`, "Adding a TSX entry").
#[native_module("sundae/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>();
}

/// The application's root component.
#[component]
pub fn App() -> impl IntoElement {
    let state = AppState::new();
    shell_view(state)
}
