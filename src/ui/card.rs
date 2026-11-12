use nsh_base::theme;
use nsh_core::prelude::*;

pub fn app_card() -> View {
    view()
        .bg(move || theme().card)
        .border_1()
        .border_color(move || theme().border)
        .rounded(move || theme().radius)
        .p_4()
}
