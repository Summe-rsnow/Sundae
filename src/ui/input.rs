use crate::ui::icons::search_icon;
use nsh_base::{input, theme};
use nsh_core::prelude::*;

pub fn search_bar(
    placeholder: &'static str,
    query_signal: Signal<String>,
    on_change: impl Fn(String) + 'static,
) -> impl IntoElement {
    view()
        .flex_1()
        .flex_row()
        .items_center()
        .h_9()
        .px_3()
        .gap_2()
        .rounded(move || theme().radius)
        .bg(move || theme().input.opacity(0.2))
        .border_1()
        .border_color(move || theme().border)
        .focus(|s| s.border_color(move || theme().ring))
        .child(search_icon(16.0))
        .child(
            input()
                .flex_1()
                .value(move || query_signal.get())
                .on_change(move |text: &str| on_change(text.to_owned()))
                .placeholder(placeholder)
                .placeholder_color(|| color(theme().muted_foreground))
                .caret_color(|| color(theme().primary))
                .text_sm()
                .text_color(move || theme().foreground),
        )
}
