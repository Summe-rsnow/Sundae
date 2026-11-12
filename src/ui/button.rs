use crate::ui::tokens::status_color;
use nsh_base::{button, theme};
use nsh_core::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    Md,
}

pub fn app_button(
    label: impl IntoReactive<SharedString>,
    variant: ButtonVariant,
    size: ButtonSize,
    on_press: impl Fn() + 'static,
) -> impl IntoElement {
    let base = button()
        .on_press(on_press)
        .items_center()
        .justify_center()
        .rounded(move || theme().radius)
        .active(|s| s.opacity(0.75))
        .focus_visible(|s| s.outline(px(2.0)).outline_color(move || theme().ring))
        .disabled_style(|s| s.opacity(0.4));

    let sized = match size {
        ButtonSize::Sm => base.h_7().px(px(10.0)),
        ButtonSize::Md => base.h_9().px(px(14.0)),
    };

    match variant {
        ButtonVariant::Primary => sized
            .bg(move || theme().primary)
            .child(text(label).text_sm().font_weight(FontWeight::MEDIUM).text_color(move || theme().primary_foreground)),
        ButtonVariant::Secondary => sized
            .bg(move || theme().secondary)
            .border_1()
            .border_color(move || theme().border)
            .child(text(label).text_sm().font_weight(FontWeight::MEDIUM).text_color(move || theme().secondary_foreground)),
        ButtonVariant::Ghost => sized
            .hover(|s| s.bg(move || theme().muted.opacity(0.5)))
            .child(text(label).text_sm().font_weight(FontWeight::MEDIUM).text_color(move || theme().foreground)),
        ButtonVariant::Danger => sized
            .bg(move || status_color::danger())
            .child(text(label).text_sm().font_weight(FontWeight::MEDIUM).text_color(move || rgb(0xffffff))),
    }
}
