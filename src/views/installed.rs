//! Installed applications view: virtualized list of installed Scoop packages with
//! hold/unhold version lock, directory & manifest launchers (via OS commands, never `open_url`),
//! package detail preview, and destructive uninstall confirmation dialog.

use nsh_base::theme;
use nsh_core::prelude::*;

use crate::scoop::installed::{open_directory, open_local_file, AppInstallStatus};
use crate::scoop::process::ScoopOp;
use crate::store;
use crate::ui::{
    action_sm, empty_state, icon_action, icons, radius, search_input, status_badge, ButtonKind,
    IconKind, ValueTone,
};

pub fn installed_view() -> impl IntoElement {
    let apps = computed(store::filtered_installed_apps);

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
                            text("已安装应用")
                                .role(AccessibilityRole::Header)
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(status_badge(
                            move || format!("共 {} 个", store::installed_count()),
                            ValueTone::Info,
                        )),
                )
                .child(
                    view().w(px(280.0)).child(search_input(
                        store::installed_filter,
                        "过滤已安装应用…",
                        "过滤已安装应用",
                        |text| store::set_installed_filter(text),
                        || store::set_installed_filter(""),
                    )),
                ),
        )
        .child(
            // Virtualized installed apps list (zero per-row local signals)
            virtual_list(move || apps.with(Vec::len), move |at| {
                let app = computed(move || apps.with(|list| list.get(at.index()).cloned()));
                let status = computed(move || {
                    app.with(|a| {
                        a.as_ref()
                            .map(|x| x.status)
                            .unwrap_or(AppInstallStatus::Installed)
                    })
                });
                let is_failed = move || status.get() == AppInstallStatus::Failed;
                let is_held = move || status.get() == AppInstallStatus::Held;

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
                                            app.with(|a| {
                                                a.as_ref()
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
                                            app.with(|a| {
                                                a.as_ref()
                                                    .map(|x| format!("v{}", x.version))
                                                    .unwrap_or_default()
                                            })
                                        },
                                        ValueTone::Neutral,
                                    ))
                                    .child(when(
                                        move || {
                                            app.with(|a| {
                                                a.as_ref().and_then(|x| x.bucket.as_ref()).is_some()
                                            })
                                        },
                                        move || {
                                            status_badge(
                                                move || {
                                                    app.with(|a| {
                                                        a.as_ref()
                                                            .and_then(|x| x.bucket.clone())
                                                            .unwrap_or_default()
                                                    })
                                                },
                                                ValueTone::Info,
                                            )
                                        },
                                    ))
                                    .child(when(is_held, || {
                                        status_badge("已锁定版本", ValueTone::Warn)
                                    }))
                                    .child(when(is_failed, || {
                                        status_badge("安装异常", ValueTone::Bad)
                                    })),
                            )
                            .child(
                                text(move || {
                                    app.with(|a| {
                                        a.as_ref()
                                            .map(|x| x.path.display().to_string())
                                            .unwrap_or_default()
                                    })
                                })
                                .number_of_lines(1)
                                .text_xs()
                                .text_color(move || theme().muted_foreground),
                            ),
                    )
                    .child(
                        view()
                            .flex_shrink_0()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .child(when(is_failed, move || {
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(action_sm(ButtonKind::Primary, "修复重装", move || {
                                        if let Some(a) = app.get_untracked() {
                                            store::enqueue_op(ScoopOp::Install(a.name));
                                        }
                                    }))
                                    .child(action_sm(ButtonKind::Destructive, "清理", move || {
                                        if let Some(a) = app.get_untracked() {
                                            store::request_uninstall(a.name);
                                        }
                                    }))
                            }))
                            .child(when(
                                move || !is_failed(),
                                move || {
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap_1p5()
                                        .child(action_sm(ButtonKind::Ghost, "详情", move || {
                                            if let Some(a) = app.get_untracked() {
                                                if let Some(manifest) = store::find_manifest(
                                                    &a.name,
                                                    a.bucket.as_deref(),
                                                ) {
                                                    store::open_package_detail(manifest);
                                                } else {
                                                    store::notices().notify_info(format!(
                                                        "未在当前 Bucket 索引中找到 {} 的清单详情",
                                                        a.name
                                                    ));
                                                }
                                            }
                                        }))
                                        .child(action_sm(
                                            ButtonKind::Secondary,
                                            move || if is_held() { "解锁" } else { "锁定" },
                                            move || {
                                                if let Some(a) = app.get_untracked() {
                                                    store::toggle_hold(&a);
                                                }
                                            },
                                        ))
                                        .child(icon_action(
                                            IconKind::Subtle,
                                            icons::FOLDER,
                                            "打开安装目录",
                                            move || {
                                                if let Some(a) = app.get_untracked() {
                                                    if let Err(err) = open_directory(&a.path) {
                                                        store::notices().notify_bad(format!(
                                                            "无法打开目录：{err}"
                                                        ));
                                                    }
                                                }
                                            },
                                        ))
                                        .child(icon_action(
                                            IconKind::Subtle,
                                            icons::FILE_TEXT,
                                            "打开 manifest.json",
                                            move || {
                                                if let Some(a) = app.get_untracked() {
                                                    if a.manifest_path.exists() {
                                                        if let Err(err) =
                                                            open_local_file(&a.manifest_path)
                                                        {
                                                            store::notices().notify_bad(format!(
                                                                "无法打开清单文件：{err}"
                                                            ));
                                                        }
                                                    } else {
                                                        store::notices()
                                                            .notify_warn("未找到本地 manifest.json");
                                                    }
                                                }
                                            },
                                        ))
                                        .child(icon_action(
                                            IconKind::Danger,
                                            icons::TRASH,
                                            "卸载应用",
                                            move || {
                                                if let Some(a) = app.get_untracked() {
                                                    store::request_uninstall(a.name);
                                                }
                                            },
                                        ))
                                },
                            )),
                    )
            })
            .keys(move || {
                apps.with(|list| list.iter().map(|a| a.name.clone()).collect::<Vec<_>>())
            })
            .estimated_item_size(px(74.0))
            .row_gap(px(8.0))
            .empty(empty_state(
                "暂无匹配的已安装应用",
                "你可以前往「探索」视图快速搜索并安装 Scoop 应用",
            ))
            .flex_1(),
        )
}
