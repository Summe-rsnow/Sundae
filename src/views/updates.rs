use crate::scoop::process::ScoopOp;
use crate::store::AppState;
use crate::ui::*;
use nsh_base::theme;
use nsh_core::prelude::*;

pub fn updates_view(state: AppState) -> impl IntoElement {
    let updates_sig = state.updates_list;

    view()
        .flex_1()
        .h_full()
        .p_6()
        .gap_4()
        .child(
            // Header Bar
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(text("更新中心").text_xl().font_weight(FontWeight::BOLD))
                        .child(dynamic({
                            move || {
                                let count = updates_sig.with(|u| u.len());
                                if count > 0 {
                                    app_badge(format!("{} 项待更新", count), BadgeVariant::Warning).into_element()
                                } else {
                                    app_badge("已全部为最新", BadgeVariant::Success).into_element()
                                }
                            }
                        })),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(app_button("检查更新", ButtonVariant::Secondary, ButtonSize::Md, {
                            let s = state.clone();
                            move || s.refresh_all()
                        }))
                        .child(app_button("一键全部更新", ButtonVariant::Primary, ButtonSize::Md, {
                            let s = state.clone();
                            move || s.enqueue_op(ScoopOp::UpdateAll)
                        })),
                ),
        )
        .child(
            // Updates list
            dynamic({
                let state = state.clone();
                move || {
                    let updates = updates_sig.get();

                    if updates.is_empty() {
                        return view()
                            .flex_1()
                            .items_center()
                            .justify_center()
                            .gap_3()
                            .child(
                                view()
                                    .p_4()
                                    .rounded(px(999.0))
                                    .bg(move || status_color::success().opacity(0.12))
                                    .child(shield_check_icon(36.0)),
                            )
                            .child(text("所有已安装应用均为最新版本").text_base().font_weight(FontWeight::BOLD))
                            .child(text("Scoop 本地仓库已与当前 Bucket 保持一致").text_xs().text_color(move || theme().muted_foreground))
                            .into_element();
                    }

                    let mut list = view().gap_2();

                    for item in updates {
                        let name = item.name.clone();
                        let s_clone = state.clone();

                        list = list.child(
                            app_card()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .p_4()
                                .child(
                                    view()
                                        .gap_1()
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .child(text(item.name).text_base().font_weight(FontWeight::BOLD))
                                                .child(app_badge(item.bucket, BadgeVariant::Secondary)),
                                        )
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .child(text(format!("当前: v{}", item.current_version)).text_xs().text_color(move || theme().muted_foreground))
                                                .child(arrow_right_icon(14.0))
                                                .child(text(format!("最新: v{}", item.latest_version)).text_xs().font_weight(FontWeight::BOLD).text_color(move || status_color::success())),
                                        ),
                                )
                                .child(
                                    app_button("立即更新", ButtonVariant::Primary, ButtonSize::Sm, move || {
                                        s_clone.enqueue_op(ScoopOp::Update(name.clone()));
                                    }),
                                ),
                        );
                    }

                    scroll()
                        .flex_1()
                        .pr_4()
                        .child(list)
                        .into_element()
                }
            }),
        )
}
