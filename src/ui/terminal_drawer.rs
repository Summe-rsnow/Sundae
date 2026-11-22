//! Collapsible bottom real-time Scoop command terminal drawer.
//!
//! Uses `virtual_list` with stable `LogLine.id` keys and zero per-row local signals,
//! styled exclusively with Bento semantic tokens (`terminal`, `terminal-foreground`,
//! `terminal-border`, `destructive`, `muted_foreground`).

use nsh_base::theme;
use nsh_core::prelude::*;

use crate::store;
use crate::ui::{
    action_sm, action_sm_disabled, color, icons, status_badge, ButtonKind, ValueTone,
};

/// Collapsible terminal output panel mounted at the bottom of the workspace.
pub fn terminal_drawer() -> impl IntoElement {
    let list_ref = element_ref();

    // Keep scrolled near the newest log entry as output streams in.
    effect(move || {
        let count = store::log_count();
        if count > 0 {
            list_ref.scroll_to_end(false);
        }
    });

    view()
        .w_full()
        .h(px(236.0))
        .bg(move || color("terminal"))
        .border_t_1()
        .border_color(move || color("terminal-border"))
        .child(
            // Drawer Header Bar
            view()
                .h(px(38.0))
                .px_4()
                .flex_row()
                .items_center()
                .justify_between()
                .gap_3()
                .bg(move || theme().card)
                .border_b_1()
                .border_color(move || color("terminal-border"))
                .child(
                    view()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(icons::svg_icon(icons::TERMINAL, 14.0, move || {
                            theme().muted_foreground
                        }))
                        .child(
                            text("任务执行终端")
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(when(store::is_task_running, || {
                            status_badge(
                                move || {
                                    store::running_task_label()
                                        .unwrap_or_else(|| "执行中".to_owned())
                                },
                                ValueTone::Warn,
                            )
                        }))
                        .child(when(
                            move || !store::is_task_running(),
                            || status_badge("就绪", ValueTone::Neutral),
                        )),
                )
                .child(
                    view()
                        .flex_shrink_0()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(action_sm_disabled(
                            ButtonKind::Destructive,
                            "停止任务",
                            move || !store::is_task_running(),
                            store::cancel_current_task,
                        ))
                        .child(action_sm_disabled(
                            ButtonKind::Ghost,
                            "清空",
                            move || store::log_count() == 0,
                            store::clear_logs,
                        ))
                        .child(action_sm(ButtonKind::Secondary, "收起", || {
                            store::set_terminal_open(false)
                        })),
                ),
        )
        .child(
            // Virtualized real-time log stream
            virtual_list(store::log_count, move |at| {
                let entry = computed(move || store::log_at(at.index()));
                view()
                    .w_full()
                    .px_4()
                    .py(px(2.0))
                    .child(
                        text(move || {
                            entry.with(|line| {
                                line.as_ref().map(|l| l.text.clone()).unwrap_or_default()
                            })
                        })
                        .text_xs()
                        .font_family(crate::ui::mono_font)
                        .text_color(move || {
                            if entry.with(|line| line.as_ref().is_some_and(|l| l.is_err)) {
                                theme().destructive
                            } else {
                                color("terminal-foreground")
                            }
                        }),
                    )
            })
            .keys(store::log_ids)
            .estimated_item_size(px(20.0))
            .content_padding(px(8.0))
            .empty(
                view()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .p_6()
                    .child(
                        text("等待任务执行… 安装、更新、卸载或同步仓库源时，实时日志将在此显示。")
                            .text_xs()
                            .text_color(move || theme().muted_foreground),
                    ),
            )
            .flex_1()
            .element_ref(&list_ref),
        )
}
