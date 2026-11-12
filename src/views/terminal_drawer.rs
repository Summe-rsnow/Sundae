use crate::store::AppState;
use crate::ui::tokens::terminal_color;
use crate::ui::*;
use nsh_base::theme;
use nsh_core::prelude::*;

pub fn terminal_drawer(state: AppState) -> impl IntoElement {
    let logs_sig = state.task_logs;
    let drawer_open_sig = state.terminal_drawer_open;
    let queue = state.task_queue.clone();
    let is_running_sig = state.is_task_running;

    view()
        .w_full()
        .h(px(260.0))
        .bg(move || terminal_color::bg())
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
                .bg(move || terminal_color::header_bg())
                .border_b_1()
                .border_color(move || terminal_color::border())
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(terminal_icon(15.0))
                        .child(text("任务执行终端").text_xs().font_weight(FontWeight::BOLD).text_color(move || terminal_color::title_text()))
                        .child(dynamic({
                            move || {
                                if is_running_sig.get() {
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
                                        .text_color(move || terminal_color::muted_text()),
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
                                    .text_color(move || {
                                        if is_err {
                                            terminal_color::error_text()
                                        } else {
                                            terminal_color::output_text()
                                        }
                                    }),
                            );
                        }

                        log_view.into_element()
                    }
                })),
        )
}
