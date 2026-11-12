use crate::store::{AppState, Route};
use crate::ui::*;
use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn shell_view(state: AppState) -> impl IntoElement {
    let active_route = state.active_route;
    let missing_sig = state.is_missing_scoop;

    view()
        .flex_1()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .child(
            // If Scoop is missing: show onboarding warning banner
            view()
                .flex_1()
                .flex_row()
                .child(
                    // Left persistent sidebar (220px)
                    view()
                        .w(px(220.0))
                        .h_full()
                        .bg(move || theme().card)
                        .border_r_1()
                        .border_color(move || theme().border)
                        .p_4()
                        .gap_2()
                        .child(
                            // Brand Header
                            view()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .pb_4()
                                .border_b_1()
                                .border_color(move || theme().border.opacity(0.5))
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(text("🍦").text_xl())
                                        .child(
                                            view()
                                                .child(text("Sundae").text_base().font_weight(FontWeight::BOLD))
                                                .child(text("Scoop GUI").text_xs().text_color(move || theme().muted_foreground)),
                                        ),
                                )
                                .child(
                                    // Refresh button
                                    button()
                                        .p_1p5()
                                        .rounded(move || theme().radius)
                                        .hover(|s| s.bg(move || theme().muted))
                                        .on_press({
                                            let s = state.clone();
                                            move || s.refresh_all()
                                        })
                                        .child(refresh_icon(14.0)),
                                ),
                        )
                        .child(
                            // Navigation list
                            view()
                                .flex_1()
                                .pt_2()
                                .gap_1()
                                .child(nav_item(
                                    "探索",
                                    search_icon(16.0),
                                    Route::Discover,
                                    active_route,
                                    || None,
                                ))
                                .child(nav_item(
                                    "已安装",
                                    package_icon(16.0),
                                    Route::Installed,
                                    active_route,
                                    {
                                        let s = state.installed_apps;
                                        move || Some(format!("{}", s.with(|a| a.len())))
                                    },
                                ))
                                .child(nav_item(
                                    "更新中心",
                                    refresh_icon(16.0),
                                    Route::Updates,
                                    active_route,
                                    {
                                        let s = state.updates_list;
                                        move || Some(format!("{}", s.with(|u| u.len())))
                                    },
                                ))
                                .child(nav_item(
                                    "仓库源",
                                    bucket_icon(16.0),
                                    Route::Buckets,
                                    active_route,
                                    {
                                        let s = state.search_index;
                                        move || Some(format!("{}", s.with(|i| i.buckets().len())))
                                    },
                                )),
                        )
                        .child(
                            // Bottom drawer toggle
                            button()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .p_2p5()
                                .rounded(move || theme().radius)
                                .bg(move || theme().muted.opacity(0.4))
                                .hover(|s| s.bg(move || theme().muted.opacity(0.8)))
                                .on_press({
                                    let drawer_sig = state.terminal_drawer_open;
                                    move || drawer_sig.update(|open| *open = !*open)
                                })
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(terminal_icon(15.0))
                                        .child(text("终端任务").text_xs().font_weight(FontWeight::MEDIUM)),
                                )
                                .child(
                                    view()
                                        .w(px(8.0))
                                        .h(px(8.0))
                                        .rounded(px(999.0))
                                        .bg({
                                            let q = state.task_queue.clone();
                                            move || {
                                                if q.is_running() {
                                                    status_color::warning()
                                                } else {
                                                    status_color::success()
                                                }
                                            }
                                        }),
                                ),
                        ),
                )
                .child(
                    // Right Content Area
                    view()
                        .flex_1()
                        .h_full()
                        .child(dynamic({
                            let s_route = state.clone();
                            move || {
                                if missing_sig.get() {
                                    missing_scoop_view(s_route.clone()).into_element()
                                } else {
                                    match active_route.get() {
                                        Route::Discover => crate::views::discover_view(s_route.clone()).into_element(),
                                        Route::Installed => crate::views::installed_view(s_route.clone()).into_element(),
                                        Route::Updates => crate::views::updates_view(s_route.clone()).into_element(),
                                        Route::Buckets => crate::views::buckets_view(s_route.clone()).into_element(),
                                    }
                                }
                            }
                        })),
                ),
            )
        .child(dynamic({
            let s = state.clone();
            let drawer_open = state.terminal_drawer_open;
            move || {
                if drawer_open.get() {
                    crate::views::terminal_drawer(s.clone()).into_element()
                } else {
                    view().into_element()
                }
            }
        }))
}

fn nav_item(
    label: &'static str,
    icon: impl IntoElement,
    route: Route,
    active_route: Signal<Route>,
    count_badge: impl Fn() -> Option<String> + 'static,
) -> impl IntoElement {
    button()
        .flex_row()
        .items_center()
        .justify_between()
        .px_3()
        .py_2()
        .rounded(move || theme().radius)
        .bg(move || {
            if active_route.get() == route {
                theme().primary.opacity(0.12)
            } else {
                theme().primary.opacity(0.0)
            }
        })
        .hover(move |s| {
            if active_route.get() != route {
                s.bg(theme().muted.opacity(0.4))
            } else {
                s
            }
        })
        .on_press(move || active_route.set(route))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap_2p5()
                .child(icon)
                .child(
                    text(label)
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(move || {
                            if active_route.get() == route {
                                theme().primary
                            } else {
                                theme().foreground
                            }
                        }),
                ),
        )
        .child(dynamic(move || {
            if let Some(count_str) = count_badge() {
                if count_str != "0" {
                    app_badge(count_str, BadgeVariant::Secondary).into_element()
                } else {
                    view().into_element()
                }
            } else {
                view().into_element()
            }
        }))
}

fn missing_scoop_view(state: AppState) -> impl IntoElement {
    view()
        .flex_1()
        .items_center()
        .justify_center()
        .p_8()
        .child(
            app_card()
                .max_w(px(520.0))
                .items_center()
                .gap_4()
                .child(text("⚠️ 未检测到 Scoop 环境").text_lg().font_weight(FontWeight::BOLD))
                .child(
                    text("Sundae 需要依赖本地已安装的 Scoop 包管理器。未在 %USERPROFILE%\\scoop 找到相关目录。")
                        .text_sm()
                        .text_color(move || theme().muted_foreground),
                )
                .child(
                    view()
                        .w_full()
                        .p_3()
                        .bg(move || theme().muted)
                        .rounded(move || theme().radius)
                        .child(
                            text("irm get.scoop.neil.im | iex")
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM),
                        ),
                )
                .child(
                    app_button("重新检测环境", ButtonVariant::Primary, ButtonSize::Md, move || {
                        state.refresh_all();
                    }),
                ),
        )
}
