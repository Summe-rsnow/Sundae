use nsh_base::theme;
use nsh_core::prelude::*;

pub fn list_row() -> View {
    view()
        .px_4()
        .py_3()
        .flex_row()
        .items_center()
        .border_b_1()
        .border_color(move || theme().border.opacity(0.6))
        .hover(|s| s.bg(move || theme().muted.opacity(0.35)))
}
