//! Desktop application shell for Sundae:
//! - 48px hidden title bar (`bento::shell::title_bar_shell`) with brand mark, version pill,
//!   live Scoop status summary, spinning refresh action, terminal drawer toggle, and theme switcher
//! - 212px left sidebar navigation (`bento::ui::sidebar_nav_item`)
//! - Main workspace hosting the active view + collapsible bottom virtualized terminal drawer
//! - System tray, close-button interception (`CloseBehavior`), keyboard shortcuts, floating toasts

use bento::shell::{
    bind_close_handler, close_behavior_submenu, confirm_close_dialog, remember_window_placement,
    title_bar_brand_clickable, title_bar_right_with_theme, title_bar_shell,
};
use nsh_base::{button as base_button, theme};
use nsh_core::element::desktop::{current_window, quit, set_tray, tray, MenuItem};
use nsh_core::prelude::*;

use crate::dialogs;
use crate::store::{self, Route};
use crate::ui::{
    action, card, color, focus_ring, icons, radius, sidebar_nav_item, status_badge,
    terminal_drawer, ButtonKind, IconKind, ValueTone, MOTION_FAST_MS,
};
use crate::views;

/// Root desktop screen of Sundae.
pub fn screen() -> impl IntoElement {
    let win = current_window();
    remember_window_placement(win.clone(), 880.0, 600.0);
    let close_confirm_open = signal(false);

    // Background task poller & floating toast auto-dismiss effect
    store::start_task_poller();
    store::notices().install_auto_dismiss();

    // Desktop keyboard shortcuts
    on_key("mod+r", |_| store::refresh_all());
    on_key("mod+j", |_| store::toggle_terminal());
    on_key("escape", move |_| {
        if close_confirm_open.get_untracked() {
            close_confirm_open.set(false);
        } else {
            store::dismiss_top_dialog();
        }
    });

    // Window close-button interception honoring CloseBehavior
    bind_close_handler(win.clone(), store::close_behavior_untracked, move || {
        close_confirm_open.set(true);
    });

    // System tray reflecting live CloseBehavior & allowing quick window restore/quit
    let win_tray = win.clone();
    effect(move || {
        let behavior = store::close_behavior();
        let show_win = win_tray.clone();
        let show_win_click = win_tray.clone();
        let updates_win = win_tray.clone();
        let about_win = win_tray.clone();

        set_tray(
            tray()
                .icon(icons::tray_icon_src())
                .tooltip(format!("Sundae v{} — Scoop 包管理器", crate::APP_VERSION))
                .on_click(move |_| {
                    show_win_click.show();
                    show_win_click.focus();
                })
                .menu([
                    MenuItem::new("显示主窗口").on_select(move || {
                        show_win.show();
                        show_win.focus();
                    }),
                    MenuItem::new("检查软件更新").on_select(move || {
                        updates_win.show();
                        updates_win.focus();
                        store::set_route(Route::Updates);
                        store::refresh_all();
                    }),
                    MenuItem::new("关于与检查更新…").on_select(move || {
                        about_win.show();
                        about_win.focus();
                        store::set_about_open(true);
                    }),
                    MenuItem::separator(),
                    close_behavior_submenu(behavior, store::set_close_behavior),
                    MenuItem::separator(),
                    MenuItem::new("退出 Sundae").on_select(quit),
                ]),
        );
    });
    cleanup(|| set_tray(None));

    let any_dialog_open = move || {
        close_confirm_open.get()
            || store::about_open()
            || store::detail_target().is_some()
            || store::uninstall_target().is_some()
            || store::remove_bucket_target().is_some()
    };

    view()
        .flex_1()
        .relative()
        .bg(move || theme().background)
        .child(title_bar())
        .child(
            view()
                .flex_1()
                .min_h(px(0.0))
                .flex_row()
                .child(sidebar())
                .child(workspace()),
        )
        // Floating bottom notice toast (hidden while modal dialogs are open)
        .child(store::notices().overlay(move || !any_dialog_open()))
        // Modal overlays
        .child(dialogs::package_detail_overlay())
        .child(dialogs::confirm_uninstall_overlay())
        .child(dialogs::confirm_remove_bucket_overlay())
        .child(dialogs::about_dialog_overlay())
        .child(when(
            move || close_confirm_open.get(),
            move || {
                confirm_close_dialog(
                    close_confirm_open,
                    win.clone(),
                    crate::APP_META.name,
                    store::set_close_behavior,
                )
            },
        ))
}

fn title_bar() -> impl IntoElement {
    let left = title_bar_brand_clickable(crate::APP_META, || store::set_about_open(true));

    let center = view()
        .flex_1()
        .min_w(px(0.0))
        .flex_row()
        .items_center()
        .justify_center()
        .gap_2()
        .child(
            text(move || {
                if store::is_missing_scoop() {
                    "未检测到本地 Scoop 环境".to_owned()
                } else if store::is_indexing() {
                    "正在后台扫描本地 Bucket 索引…".to_owned()
                } else {
                    format!(
                        "已安装 {} · 可更新 {} · {} 个源",
                        store::installed_count(),
                        store::updates_count(),
                        store::bucket_count()
                    )
                }
            })
            .number_of_lines(1)
            .text_xs()
            .tabular_nums()
            .text_color(move || theme().muted_foreground),
        )
        .child(when(store::is_task_running, || {
            status_badge(
                move || {
                    store::running_task_label().unwrap_or_else(|| "任务执行中".to_owned())
                },
                ValueTone::Warn,
            )
        }));

    let right_actions = view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(crate::ui::icon_action_spinning_with_tip(
            IconKind::Secondary,
            icons::REFRESH,
            "刷新索引",
            "刷新 Scoop 索引 (Ctrl+R)",
            store::is_indexing,
            store::is_indexing,
            store::refresh_all,
        ))
        .child(crate::ui::icon_action_with_tip(
            IconKind::Secondary,
            icons::TERMINAL,
            "切换终端抽屉",
            move || {
                if store::terminal_open() {
                    "收起终端抽屉 (Ctrl+J)"
                } else {
                    "展开终端抽屉 (Ctrl+J)"
                }
            },
            || false,
            store::toggle_terminal,
        ));

    let right = title_bar_right_with_theme(
        right_actions,
        store::theme_mode,
        store::set_theme_mode,
    );

    title_bar_shell(48.0, left, center, right).bg(move || theme().sidebar)
}

fn sidebar() -> impl IntoElement {
    view()
        .w(px(212.0))
        .flex_shrink_0()
        .h_full()
        .bg(move || theme().sidebar)
        .border_r_1()
        .border_color(move || color("sidebar-border"))
        .p_3()
        .gap_2()
        .child(
            view()
                .flex_1()
                .gap_1()
                .child(sidebar_nav_item(
                    move || store::active_route() == Route::Discover,
                    icons::COMPASS,
                    "探索",
                    || None,
                    || store::set_route(Route::Discover),
                ))
                .child(sidebar_nav_item(
                    move || store::active_route() == Route::Installed,
                    icons::PACKAGE,
                    "已安装",
                    || Some(store::installed_count()),
                    || store::set_route(Route::Installed),
                ))
                .child(sidebar_nav_item(
                    move || store::active_route() == Route::Updates,
                    icons::ARROW_UP_CIRCLE,
                    "更新中心",
                    || Some(store::updates_count()),
                    || store::set_route(Route::Updates),
                ))
                .child(sidebar_nav_item(
                    move || store::active_route() == Route::Buckets,
                    icons::DATABASE,
                    "仓库源",
                    || Some(store::bucket_count()),
                    || store::set_route(Route::Buckets),
                ))
                .child(sidebar_nav_item(
                    move || store::active_route() == Route::Config,
                    icons::SETTINGS,
                    "选项配置",
                    || None,
                    || store::set_route(Route::Config),
                )),
        )
        .child(
            // Sidebar footer: Terminal status toggle
            base_button()
                .on_press(store::toggle_terminal)
                .accessibility_label("切换任务执行终端")
                .w_full()
                .h(px(34.0))
                .px(px(10.0))
                .flex_row()
                .items_center()
                .justify_between()
                .gap_2()
                .rounded(radius)
                .border_1()
                .border_color(move || theme().border)
                .bg(move || {
                    if store::terminal_open() {
                        theme().accent
                    } else {
                        theme().card
                    }
                })
                .hover(|s| s.bg(move || theme().accent))
                .active(|s| s.scale(0.98))
                .transition_transform()
                .duration(MOTION_FAST_MS)
                .focus_visible(focus_ring)
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(icons::svg_icon(icons::TERMINAL, 14.0, move || {
                            if store::terminal_open() {
                                theme().accent_foreground
                            } else {
                                theme().muted_foreground
                            }
                        }))
                        .child(
                            text(move || {
                                let pending = store::pending_task_count();
                                if store::is_task_running() {
                                    if pending > 1 {
                                        format!("执行中 ({pending})")
                                    } else {
                                        "任务执行中".to_owned()
                                    }
                                } else {
                                    "任务终端".to_owned()
                                }
                            })
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(move || {
                                if store::terminal_open() {
                                    theme().accent_foreground
                                } else {
                                    theme().foreground
                                }
                            }),
                        ),
                )
                .child(
                    view()
                        .size_2()
                        .rounded_full()
                        .bg(move || {
                            if store::is_task_running() {
                                ValueTone::Warn.color()
                            } else {
                                ValueTone::Ok.color()
                            }
                        }),
                ),
        )
}

fn workspace() -> impl IntoElement {
    let show_missing = move || store::is_missing_scoop() && store::active_route() != Route::Config;

    view()
        .flex_1()
        .min_w(px(0.0))
        .h_full()
        .child(
            view()
                .flex_1()
                .min_h(px(0.0))
                .child(when(show_missing, missing_scoop_view))
                .child(when(
                    move || !show_missing() && store::active_route() == Route::Discover,
                    views::discover_view,
                ))
                .child(when(
                    move || !show_missing() && store::active_route() == Route::Installed,
                    views::installed_view,
                ))
                .child(when(
                    move || !show_missing() && store::active_route() == Route::Updates,
                    views::updates_view,
                ))
                .child(when(
                    move || !show_missing() && store::active_route() == Route::Buckets,
                    views::buckets_view,
                ))
                .child(when(
                    move || store::active_route() == Route::Config,
                    views::config_view,
                )),
        )
        .child(when(store::terminal_open, terminal_drawer))
}

fn missing_scoop_view() -> impl IntoElement {
    view()
        .flex_1()
        .items_center()
        .justify_center()
        .p_8()
        .child(
            card()
                .max_w(px(520.0))
                .p_6()
                .gap_4()
                .items_center()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(icons::svg_icon(icons::ALERT_TRIANGLE, 20.0, move || {
                            ValueTone::Warn.color()
                        }))
                        .child(
                            text("未检测到本地 Scoop 环境")
                                .text_base()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        ),
                )
                .child(
                    text(move || {
                        format!(
                            "Sundae 需要依赖本地已安装的 Scoop 包管理器。未在 {} 找到 apps 或 buckets 目录。",
                            store::scoop_root_display()
                        )
                    })
                    .text_xs()
                    .text_color(move || theme().muted_foreground),
                )
                .child(
                    view()
                        .w_full()
                        .px_3()
                        .py_2()
                        .rounded(radius)
                        .border_1()
                        .border_color(move || theme().border)
                        .bg(move || theme().muted)
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            text("irm get.scoop.sh | iex")
                                .text_xs()
                                .font_family(crate::ui::mono_font)
                                .text_color(move || theme().foreground),
                        )
                        .child(crate::ui::action_sm(ButtonKind::Secondary, "复制安装命令", || {
                            set_clipboard("irm get.scoop.sh | iex");
                            store::notices().notify_ok("已复制 Scoop 安装命令");
                        })),
                )
                .child(action(
                    ButtonKind::Primary,
                    "重新检测环境",
                    store::refresh_all,
                )),
        )
}
