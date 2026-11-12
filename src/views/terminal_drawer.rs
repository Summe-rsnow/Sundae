use crate::store::AppState;
use crate::ui::*;
use nsh_base::theme;
use nsh_core::prelude::*;

pub fn terminal_drawer(state: AppState) -> impl IntoElement {
    let logs_sig = state.task_logs;
    let drawer_open_sig = state.terminal_drawer_open;
    let queue = state.task_queue.clone();

    view()
        .w_full()
        .h(px(260.0))
        .bg(rgb(0x18181b)) // Dark terminal surface
        .border_t_1()
        .border_color(move || theme().border)
        .child(
            // Drawer Top Bar
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .px_4()
                .py_2()
                .bg(rgb(0x27272a))
                .border_b_1()
                .border_color(rgb(0x3f3f46))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(terminal_icon(15.0))
                        .child(text("任务执行终端").text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0xf4f4f5)))
                        .child(dynamic({
                            let q = queue.clone();
                            move || {
                                if q.is_running() {
                                    app_badge("正在执行", BadgeVariant::Warning).into_element()
                                } else {
                                    app_badge("就绪", BadgeVariant::Secondary).into_element()
                                }
                            }
                        })),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(app_button("取消当前任务", ButtonVariant::Danger, ButtonSize::Sm, {
                            let q = queue.clone();
                            move || q.cancel_current()
                        }))
                        .child(app_button("清空", ButtonVariant::Ghost, ButtonSize::Sm, {
                            let l = logs_sig;
                            move || l.set(Vec::new())
                        }))
                        .child(app_button("收起", ButtonVariant::Secondary, ButtonSize::Sm, move || {
                            drawer_open_sig.set(false);
                        })),
                ),
        )
        .child(
            // Log Stream Container
            scroll()
                .flex_1()
                .p_3()
                .child(dynamic({
                    move || {
                        let lines = logs_sig.get();

                        if lines.is_empty() {
                            return view()
                                .p_2()
                                .child(
                                    text("等待任务执行... (安装、更新或卸载时将在此输出实时控制台日志)")
                                        .text_xs()
                                        .text_color(rgb(0x71717a)),
                                )
                                .into_element();
                        }

                        let mut log_view = view().gap_1();

                        for line in lines {
                            let is_err = line.is_err;
                            log_view = log_view.child(
                                text(line.text)
                                    .text_xs()
                                    .font_family("Consolas")
                                    .text_color(if is_err {
                                        rgb(0xf87171) // Red 400
                                    } else {
                                        rgb(0xe4e4e7) // Zinc 200
                                    }),
                            );
                        }

                        log_view.into_element()
                    }
                })),
        )
}
