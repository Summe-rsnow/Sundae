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
        .rounded(px(8.0))
        .bg(move || theme().input.opacity(0.18))
        .border_1()
        .border_color(move || theme().border)
        .focus(|s| s.border_color(move || theme().ring))
        .child(search_icon(16.0))
        .child(
            input()
                .flex_1()
                .h_full()
                .value(move || query_signal.get())
                .on_change(move |text: &str| on_change(text.to_owned()))
                .placeholder(placeholder)
                .placeholder_color(|| rgb(0xa1a1aa))
                .caret_color(|| rgb(0xf4f4f5))
                .text_sm(),
        )
}

pub fn app_text_field(
    placeholder: &'static str,
    value_signal: Signal<String>,
    on_change: impl Fn(String) + 'static,
) -> impl IntoElement {
    input()
        .flex_1()
        .h_9()
        .px_3()
        .rounded(px(8.0))
        .bg(move || theme().input.opacity(0.18))
        .border_1()
        .border_color(move || theme().border)
        .focus(|s| s.border_color(move || theme().ring))
        .value(move || value_signal.get())
        .on_change(move |text: &str| on_change(text.to_owned()))
        .placeholder(placeholder)
        .placeholder_color(|| rgb(0xa1a1aa))
        .caret_color(|| rgb(0xf4f4f5))
        .text_sm()
}
