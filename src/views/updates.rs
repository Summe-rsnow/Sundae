//! Updates center view: virtualized list comparing installed versions against
//! scanned bucket manifests, with one-click per-app update and "Update All".

use nsh_base::theme;
use nsh_core::prelude::*;

use crate::scoop::process::ScoopOp;
use crate::store;
use crate::ui::{
    action_disabled, action_sm, icons, radius, status_badge, ButtonKind, ValueTone,
};

pub fn updates_view() -> impl IntoElement {
    let updates = computed(store::updates);

    view()
        .flex_1()
        .h_full()
        .p_5()
        .gap_4()
        .child(
            // Header bar
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            text("更新中心")
                                .role(AccessibilityRole::Header)
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(when(
                            move || updates.with(|u| !u.is_empty()),
                            move || {
                                status_badge(
                                    move || format!("{} 项待更新", updates.with(Vec::len)),
                                    ValueTone::Warn,
                                )
                            },
                        ))
                        .child(when(
                            move || updates.with(Vec::is_empty),
                            || status_badge("已全部为最新", ValueTone::Ok),
                        )),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(action_disabled(
                            ButtonKind::Secondary,
                            "同步仓库源",
                            store::is_task_running,
                            || store::enqueue_op(ScoopOp::BucketUpdate),
                        ))
                        .child(action_disabled(
                            ButtonKind::Primary,
                            "一键全部更新",
                            move || updates.with(Vec::is_empty) || store::is_task_running(),
                            || store::enqueue_op(ScoopOp::UpdateAll),
                        )),
                ),
        )
        .child(
            // Virtualized updates list (zero per-row local signals)
            virtual_list(move || updates.with(Vec::len), move |at| {
                let item = computed(move || updates.with(|list| list.get(at.index()).cloned()));

                view()
                    .w_full()
                    .px_4()
                    .py_3()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .rounded(radius)
                    .border_1()
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .hover(|s| s.bg(move || theme().muted))
                    .child(
                        view()
                            .flex_1()
                            .min_w(px(0.0))
                            .gap_1()
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        text(move || {
                                            item.with(|u| {
                                                u.as_ref()
                                                    .map(|x| x.name.clone())
                                                    .unwrap_or_default()
                                            })
                                        })
                                        .text_sm()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().foreground),
                                    )
                                    .child(status_badge(
                                        move || {
                                            item.with(|u| {
                                                u.as_ref()
                                                    .map(|x| x.bucket.clone())
                                                    .unwrap_or_default()
                                            })
                                        },
                                        ValueTone::Info,
                                    )),
                            )
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        text(move || {
                                            item.with(|u| {
                                                u.as_ref()
                                                    .map(|x| format!("当前 v{}", x.current_version))
                                                    .unwrap_or_default()
                                            })
                                        })
                                        .text_xs()
                                        .tabular_nums()
                                        .text_color(move || theme().muted_foreground),
                                    )
                                    .child(icons::svg_icon(icons::ARROW_RIGHT, 12.0, move || {
                                        theme().muted_foreground
                                    }))
                                    .child(
                                        text(move || {
                                            item.with(|u| {
                                                u.as_ref()
                                                    .map(|x| format!("最新 v{}", x.latest_version))
                                                    .unwrap_or_default()
                                            })
                                        })
                                        .text_xs()
                                        .tabular_nums()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || ValueTone::Ok.color()),
                                    ),
                            ),
                    )
                    .child(action_sm(ButtonKind::Primary, "立即更新", move || {
                        if let Some(u) = item.get_untracked() {
                            store::enqueue_op(ScoopOp::Update(u.name));
                        }
                    }))
            })
            .keys(move || {
                updates.with(|list| list.iter().map(|u| u.name.clone()).collect::<Vec<_>>())
            })
            .estimated_item_size(px(72.0))
            .row_gap(px(8.0))
            .empty(
                view()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .p_8()
                    .child(
                        view()
                            .size(px(56.0))
                            .items_center()
                            .justify_center()
                            .rounded_full()
                            .bg(move || ValueTone::Ok.color().opacity(0.14))
                            .child(icons::svg_icon(icons::SHIELD_CHECK, 28.0, move || {
                                ValueTone::Ok.color()
                            })),
                    )
                    .child(
                        text("所有已安装应用均为最新版本")
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(move || theme().foreground),
                    )
                    .child(
                        text("本地软件版本与当前 Bucket 索引一致，可点击「同步仓库源」拉取远程更新")
                            .text_xs()
                            .text_color(move || theme().muted_foreground),
                    ),
            )
            .flex_1(),
        )
}
