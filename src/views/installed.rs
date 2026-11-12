use crate::scoop::installed::AppInstallStatus;
use crate::scoop::process::ScoopOp;
use crate::store::AppState;
use crate::ui::*;
use nsh_base::theme;
use nsh_core::prelude::*;
use std::process::Command;

pub fn installed_view(state: AppState) -> impl IntoElement {
    let installed_sig = state.installed_apps;
    let local_search = signal(String::new());

    view()
        .flex_1()
        .h_full()
        .p_6()
        .gap_4()
        .child(
            // Header bar
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(text("已安装应用").text_xl().font_weight(FontWeight::BOLD))
                        .child(dynamic({
                            move || {
                                let count = installed_sig.with(|apps| apps.len());
                                app_badge(format!("共 {} 个", count), BadgeVariant::Primary).into_element()
                            }
                        })),
                )
                .child(
                    view()
                        .w(px(260.0))
                        .child(search_bar(
                            "过滤已安装应用...",
                            local_search,
                            move |t| local_search.set(t),
                        )),
                ),
        )
        .child(
            // List of installed applications
            dynamic({
                let state = state.clone();
                move || {
                    let filter_text = local_search.get().trim().to_ascii_lowercase();
                    let apps: Vec<_> = installed_sig.with(|list| {
                        list.iter()
                            .filter(|app| {
                                filter_text.is_empty()
                                    || app.name.to_ascii_lowercase().contains(&filter_text)
                            })
                            .cloned()
                            .collect()
                    });

                    if apps.is_empty() {
                        return view()
                            .flex_1()
                            .items_center()
                            .justify_center()
                            .gap_3()
                            .child(package_icon(36.0))
                            .child(text("暂无匹配的已安装应用").text_base().font_weight(FontWeight::MEDIUM))
                            .child(text("你可以前往「探索」视图快速检索并安装应用").text_xs().text_color(move || theme().muted_foreground))
                            .into_element();
                    }

                    let mut list_container = view().gap_2();

                    for app in apps {
                        let app_name = app.name.clone();
                        let app_path = app.path.clone();
                        let app_path_clone = app.path.clone();
                        let manifest_path = app.manifest_path.clone();
                        let status = app.status;
                        let is_held = app.is_held();
                        let is_failed = app.is_failed();
                        let s_clone = state.clone();
                        let name_hold = app_name.clone();
                        let name_uninstall = app_name.clone();

                        list_container = list_container.child(
                            app_card()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .p_4()
                                .child(
                                    view()
                                        .flex_1()
                                        .gap_1()
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .child(text(app.name).text_base().font_weight(FontWeight::BOLD))
                                                .child(app_badge(format!("v{}", app.version), BadgeVariant::Secondary))
                                                .child(match status {
                                                    AppInstallStatus::Held => app_badge("已锁定版本 (Hold)", BadgeVariant::Warning).into_element(),
                                                    AppInstallStatus::Failed => app_badge("安装损坏/未完成", BadgeVariant::Danger).into_element(),
                                                    AppInstallStatus::Installed => view().into_element(),
                                                }),
                                        )
                                        .child(
                                            text(format!("路径: {}", app.path.display()))
                                                .text_xs()
                                                .text_color(move || theme().muted_foreground),
                                        ),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(if is_failed {
                                            // Actions for broken/failed installs
                                            view()
                                                .flex_row()
                                                .gap_2()
                                                .child(app_button_with_icon(
                                                    refresh_icon(13.0),
                                                    "重新安装",
                                                    ButtonVariant::Primary,
                                                    ButtonSize::Sm,
                                                    {
                                                        let s = s_clone.clone();
                                                        let n = name_hold.clone();
                                                        move || s.enqueue_op(ScoopOp::Install(n.clone()))
                                                    },
                                                ))
                                                .child(app_button_with_icon(
                                                    trash_icon(13.0),
                                                    "清理",
                                                    ButtonVariant::Danger,
                                                    ButtonSize::Sm,
                                                    {
                                                        let s = s_clone.clone();
                                                        let n = name_uninstall.clone();
                                                        move || s.enqueue_op(ScoopOp::Uninstall(n.clone()))
                                                    },
                                                ))
                                                .into_element()
                                        } else {
                                            // Actions for normal/held installs
                                            view()
                                                .flex_row()
                                                .gap_2()
                                                .child(app_button_with_icon(
                                                    if is_held { unlock_icon(13.0) } else { lock_icon(13.0) },
                                                    if is_held { "解锁" } else { "锁定" },
                                                    ButtonVariant::Secondary,
                                                    ButtonSize::Sm,
                                                    {
                                                        let s = s_clone.clone();
                                                        let n = name_hold.clone();
                                                        let p_dir = app_path_clone.clone();
                                                        move || {
                                                            if is_held {
                                                                s.enqueue_op(ScoopOp::Unhold(n.clone()));
                                                                // Also update local scoop-install.json immediately
                                                                for fname in &["scoop-install.json", "install.json"] {
                                                                    let p = p_dir.join(fname);
                                                                    if p.exists() {
                                                                        if let Ok(c) = std::fs::read_to_string(&p) {
                                                                            if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&c) {
                                                                                if let Some(obj) = val.as_object_mut() {
                                                                                    obj.remove("hold");
                                                                                    let _ = std::fs::write(&p, serde_json::to_string_pretty(&obj).unwrap_or_default());
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                s.enqueue_op(ScoopOp::Hold(n.clone()));
                                                                for fname in &["scoop-install.json", "install.json"] {
                                                                    let p = p_dir.join(fname);
                                                                    if p.exists() {
                                                                        if let Ok(c) = std::fs::read_to_string(&p) {
                                                                            if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&c) {
                                                                                if let Some(obj) = val.as_object_mut() {
                                                                                    obj.insert("hold".to_string(), serde_json::Value::Bool(true));
                                                                                    let _ = std::fs::write(&p, serde_json::to_string_pretty(&obj).unwrap_or_default());
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            s.refresh_all();
                                                        }
                                                    },
                                                ))
                                                .child(app_button_with_icon(
                                                    folder_icon(13.0),
                                                    "打开目录",
                                                    ButtonVariant::Secondary,
                                                    ButtonSize::Sm,
                                                    move || {
                                                        let _ = Command::new("explorer").arg(&app_path).spawn();
                                                    },
                                                ))
                                                .child(app_button_with_icon(
                                                    file_text_icon(13.0),
                                                    "配置",
                                                    ButtonVariant::Secondary,
                                                    ButtonSize::Sm,
                                                    move || {
                                                        if manifest_path.exists() {
                                                            let _ = Command::new("cmd")
                                                                .args(["/c", "start", "", &manifest_path.to_string_lossy()])
                                                                .spawn();
                                                        }
                                                    },
                                                ))
                                                .child(app_button_with_icon(
                                                    trash_icon(13.0),
                                                    "卸载",
                                                    ButtonVariant::Danger,
                                                    ButtonSize::Sm,
                                                    {
                                                        let s_un = state.clone();
                                                        let n = name_uninstall.clone();
                                                        move || {
                                                            s_un.enqueue_op(ScoopOp::Uninstall(n.clone()));
                                                        }
                                                    },
                                                ))
                                                .into_element()
                                        }),
                                ),
                        );
                    }

                    scroll()
                        .flex_1()
                        .pr_4()
                        .child(list_container)
                        .into_element()
                }
            }),
        )
}
