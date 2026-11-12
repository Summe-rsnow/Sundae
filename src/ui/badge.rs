use crate::ui::tokens::status_color;
use nsh_base::theme;
use nsh_core::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

pub fn app_badge(text_content: impl IntoReactive<SharedString>, variant: BadgeVariant) -> impl IntoElement {
    let base = view()
        .px_2()
        .py(px(2.0))
        .rounded(px(999.0))
        .items_center()
        .justify_center();

    match variant {
        BadgeVariant::Primary => base
            .bg(move || theme().primary.opacity(0.15))
            .child(text(text_content).text_xs().font_weight(FontWeight::MEDIUM).text_color(move || theme().primary)),
        BadgeVariant::Secondary => base
            .bg(move || theme().muted)
            .child(text(text_content).text_xs().font_weight(FontWeight::MEDIUM).text_color(move || theme().muted_foreground)),
        BadgeVariant::Success => base
            .bg(move || status_color::success().opacity(0.15))
            .child(text(text_content).text_xs().font_weight(FontWeight::MEDIUM).text_color(move || status_color::success())),
        BadgeVariant::Warning => base
            .bg(move || status_color::warning().opacity(0.15))
            .child(text(text_content).text_xs().font_weight(FontWeight::MEDIUM).text_color(move || status_color::warning())),
        BadgeVariant::Danger => base
            .bg(move || status_color::danger().opacity(0.15))
            .child(text(text_content).text_xs().font_weight(FontWeight::MEDIUM).text_color(move || status_color::danger())),
    }
}
