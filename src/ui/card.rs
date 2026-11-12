use nsh_base::theme;
use nsh_core::prelude::*;

pub fn app_card() -> View {
    view()
        .bg(move || theme().card)
        .border_1()
        .border_color(move || theme().border.opacity(0.6))
        .rounded(px(10.0))
        .p_4()
        .hover(|s| s.border_color(move || theme().border.opacity(0.95)))
}
