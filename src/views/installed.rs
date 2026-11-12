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
                        let manifest_path = app.manifest_path.clone();
                        let is_held = app.is_held;
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
                                                .child(if is_held {
                                                    app_badge("版本已锁定 (Hold)", BadgeVariant::Warning).into_element()
                                                } else {
                                                    view().into_element()
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
                                        .child(app_button(
                                            if is_held { "解锁" } else { "锁定" },
                                            ButtonVariant::Secondary,
                                            ButtonSize::Sm,
                                            move || {
                                                if is_held {
                                                    s_clone.enqueue_op(ScoopOp::Unhold(name_hold.clone()));
                                                } else {
                                                    s_clone.enqueue_op(ScoopOp::Hold(name_hold.clone()));
                                                }
                                            },
                                        ))
                                        .child(app_button(
                                            "打开目录",
                                            ButtonVariant::Secondary,
                                            ButtonSize::Sm,
                                            move || {
                                                let _ = Command::new("explorer").arg(&app_path).spawn();
                                            },
                                        ))
                                        .child(app_button(
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
                                        .child(app_button(
                                            "卸载",
                                            ButtonVariant::Danger,
                                            ButtonSize::Sm,
                                            {
                                                let s_un = state.clone();
                                                move || {
                                                    s_un.enqueue_op(ScoopOp::Uninstall(name_uninstall.clone()));
                                                }
                                            },
                                        )),
                                ),
                        );
                    }

                    scroll()
                        .flex_1()
                        .child(list_container)
                        .into_element()
                }
            }),
        )
}
