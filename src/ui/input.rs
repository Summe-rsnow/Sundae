use crate::ui::icons::search_icon;
use nsh_base::input;
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
        .bg(rgb(0x27272a))
        .border_1()
        .border_color(rgb(0x3f3f46))
        .focus(|s| s.border_color(rgb(0x38bdf8)))
        .child(search_icon(16.0))
        .child(
            input()
                .flex_1()
                .h(px(28.0))
                .value(move || query_signal.get())
                .on_change(move |text: &str| on_change(text.to_owned()))
                .placeholder(placeholder)
                .placeholder_color(|| rgb(0xa1a1aa))
                .caret_color(|| rgb(0x38bdf8))
                .text_sm()
                .text_color(rgb(0xf4f4f5)),
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
        .bg(rgb(0x27272a))
        .border_1()
        .border_color(rgb(0x3f3f46))
        .focus(|s| s.border_color(rgb(0x38bdf8)))
        .value(move || value_signal.get())
        .on_change(move |text: &str| on_change(text.to_owned()))
        .placeholder(placeholder)
        .placeholder_color(|| rgb(0xa1a1aa))
        .caret_color(|| rgb(0x38bdf8))
        .text_sm()
        .text_color(rgb(0xf4f4f5))
}
