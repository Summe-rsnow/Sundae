//! Modal dialogs for Sundae:
//! - `about_dialog_overlay`: Shared `bento::updater::about_dialog`
//! - `confirm_uninstall_overlay`: Destructive confirmation with optional `--purge`
//! - `confirm_remove_bucket_overlay`: Destructive confirmation before `scoop bucket rm`
//! - `package_detail_overlay`: Full Manifest viewer with homepage link, shims, notes & copy CTA

use nsh_base::theme;
use nsh_core::prelude::*;

use crate::scoop::manifest::PackageManifest;
use crate::scoop::process::ScoopOp;
use crate::store;
use crate::ui::{
    action, action_sm, checkbox, divider, external_link, icons, modal_dialog, radius, status_badge,
    ButtonKind, ValueTone,
};

/// Shared Bento About & GitHub Releases update checker dialog.
pub fn about_dialog_overlay() -> impl IntoElement {
    when(store::about_open, || {
        bento::updater::about_dialog(
            store::APP_META,
            store::update_status,
            store::check_for_updates,
            || store::set_about_open(false),
        )
    })
}

/// Confirmation dialog before uninstalling a Scoop app, with an optional `--purge` flag.
pub fn confirm_uninstall_overlay() -> impl IntoElement {
    when(
        move || store::uninstall_target().is_some(),
        || {
            let target = store::uninstall_target().unwrap_or_default();
            let purge = signal(false);
            let target_for_confirm = target.clone();
            let cancel = || store::close_uninstall_confirm();

            modal_dialog(
                420.0,
                cancel,
                view()
                    .gap_4()
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap_2p5()
                            .child(icons::svg_icon(icons::ALERT_TRIANGLE, 18.0, move || {
                                theme().destructive
                            }))
                            .child(
                                text(format!("确认卸载 {}", target))
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(move || theme().foreground),
                            ),
                    )
                    .child(
                        text("卸载将移除应用本体与相关 Shim 快捷命令。若勾选下方选项，也会一并删除该应用的 persist 持久化配置数据。")
                            .text_xs()
                            .text_color(move || theme().muted_foreground),
                    )
                    .child(
                        view()
                            .p_3()
                            .rounded(radius)
                            .border_1()
                            .border_color(move || theme().border)
                            .bg(move || theme().muted)
                            .child(checkbox(
                                move || purge.get(),
                                || false,
                                "同时清除持久化数据 (--purge)",
                                Some("同时清除持久化数据（追加 --purge 参数，不可恢复）"),
                                move |next| purge.set(next),
                            )),
                    )
                    .child(
                        view()
                            .flex_row()
                            .justify_end()
                            .gap_2()
                            .child(action(ButtonKind::Secondary, "取消", cancel))
                            .child(action(ButtonKind::Destructive, "确认卸载", move || {
                                let with_purge = purge.get_untracked();
                                store::close_uninstall_confirm();
                                store::enqueue_op(ScoopOp::uninstall(
                                    target_for_confirm.clone(),
                                    with_purge,
                                ));
                            })),
                    ),
            )
        },
    )
}

/// Confirmation dialog before removing a Scoop bucket repository.
pub fn confirm_remove_bucket_overlay() -> impl IntoElement {
    when(
        move || store::remove_bucket_target().is_some(),
        || {
            let bucket = store::remove_bucket_target().unwrap_or_default();
            let bucket_for_confirm = bucket.clone();
            let cancel = || store::close_remove_bucket_confirm();

            modal_dialog(
                400.0,
                cancel,
                view()
                    .gap_4()
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap_2p5()
                            .child(icons::svg_icon(icons::ALERT_TRIANGLE, 18.0, move || {
                                theme().destructive
                            }))
                            .child(
                                text(format!("确认移除仓库源 {}", bucket))
                                    .text_base()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(move || theme().foreground),
                            ),
                    )
                    .child(
                        text("移除后将删除本地对应的 Bucket 仓库目录（已安装的应用本体不会被卸载，但后续将无法从该源检查更新）。")
                            .text_xs()
                            .text_color(move || theme().muted_foreground),
                    )
                    .child(
                        view()
                            .flex_row()
                            .justify_end()
                            .gap_2()
                            .child(action(ButtonKind::Secondary, "取消", cancel))
                            .child(action(ButtonKind::Destructive, "确认移除", move || {
                                store::close_remove_bucket_confirm();
                                store::enqueue_op(ScoopOp::BucketRm(bucket_for_confirm.clone()));
                            })),
                    ),
            )
        },
    )
}

/// Full package Manifest detail dialog with homepage link, bins, depends, notes, and copy CTA.
pub fn package_detail_overlay() -> impl IntoElement {
    when(
        move || store::detail_target().is_some(),
        || {
            let Some(pkg) = store::detail_target() else {
                return view().into_element();
            };
            package_detail_content(pkg).into_element()
        },
    )
}

fn package_detail_content(pkg: PackageManifest) -> impl IntoElement {
    let install_cmd = pkg.install_command();
    let install_cmd_copy = install_cmd.clone();
    let pkg_name_for_install = pkg.name.clone();
    let pkg_name_for_check = pkg.name.clone();
    let pkg_name_for_label = pkg.name.clone();
    let cancel = || store::close_package_detail();

    let homepage = if pkg.homepage.trim().is_empty() {
        None
    } else {
        Some(SharedString::from(pkg.homepage.clone()))
    };

    modal_dialog(
        520.0,
        cancel,
        view()
            .gap_4()
            .child(
                // Title & badges
                view()
                    .gap_1p5()
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                text(pkg.name.clone())
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(move || theme().foreground),
                            )
                            .child(status_badge(pkg.bucket.clone(), ValueTone::Info))
                            .child(status_badge(format!("v{}", pkg.version), ValueTone::Neutral))
                            .child(when(
                                move || store::is_installed_package(&pkg_name_for_check),
                                || status_badge("已安装", ValueTone::Ok),
                            )),
                    )
                    .child(
                        text(pkg.display_description().to_owned())
                            .text_xs()
                            .text_color(move || theme().muted_foreground),
                    ),
            )
            .child(
                // Metadata & Copy Install Command Card
                view()
                    .p_3()
                    .gap_2()
                    .rounded(radius)
                    .border_1()
                    .border_color(move || theme().border)
                    .bg(move || theme().muted)
                    .child(meta_row(
                        "开源协议",
                        match &pkg.license_url {
                            Some(url) => external_link(
                                pkg.license.clone(),
                                Some(SharedString::from(url.clone())),
                            )
                            .into_element(),
                            None => text(pkg.license.clone())
                                .text_xs()
                                .text_color(move || theme().foreground)
                                .into_element(),
                        },
                    ))
                    .child(meta_row(
                        "项目主页",
                        match homepage {
                            Some(url) => {
                                external_link(url.clone(), Some(url)).into_element()
                            }
                            None => text("未提供")
                                .text_xs()
                                .text_color(move || theme().muted_foreground)
                                .into_element(),
                        },
                    ))
                    .child(divider())
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(
                                text(install_cmd)
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .number_of_lines(1)
                                    .text_xs()
                                    .font_family(crate::ui::mono_font)
                                    .text_color(move || theme().foreground),
                            )
                            .child(action_sm(ButtonKind::Secondary, "复制命令", move || {
                                set_clipboard(install_cmd_copy.clone());
                                store::notices()
                                    .notify_ok(format!("已复制命令：{}", install_cmd_copy));
                            })),
                    ),
            )
            .child(
                // Optional details (bins, depends, notes) inside bounded scroll
                scroll()
                    .max_h(px(200.0))
                    .child(
                        view()
                            .gap_3()
                            .when(!pkg.bins.is_empty(), |v| {
                                v.child(
                                    view()
                                        .gap_1p5()
                                        .child(
                                            text("可执行命令 (Shims)")
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(move || theme().muted_foreground),
                                        )
                                        .child(
                                            view()
                                                .flex_row()
                                                .flex_wrap()
                                                .gap_1p5()
                                                .children(pkg.bins.iter().map(|bin| {
                                                    view()
                                                        .px_2()
                                                        .py(px(2.0))
                                                        .rounded(px(6.0))
                                                        .border_1()
                                                        .border_color(move || theme().border)
                                                        .bg(move || theme().background)
                                                        .child(
                                                            text(bin.clone())
                                                                .text_xs()
                                                                .font_family(crate::ui::mono_font)
                                                                .text_color(move || {
                                                                    theme().foreground
                                                                }),
                                                        )
                                                })),
                                        ),
                                )
                            })
                            .when(!pkg.depends.is_empty(), |v| {
                                v.child(
                                    view()
                                        .gap_1p5()
                                        .child(
                                            text("前置依赖 (Depends)")
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(move || theme().muted_foreground),
                                        )
                                        .child(
                                            text(pkg.depends.join(", "))
                                                .text_xs()
                                                .text_color(move || theme().foreground),
                                        ),
                                )
                            })
                            .when(!pkg.notes.is_empty(), |v| {
                                v.child(
                                    view()
                                        .p_3()
                                        .gap_1()
                                        .rounded(radius)
                                        .border_1()
                                        .border_color(move || theme().border)
                                        .bg(move || theme().background)
                                        .child(
                                            text("安装说明 (Notes)")
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(move || ValueTone::Warn.color()),
                                        )
                                        .child(
                                            text(pkg.notes.join("\n"))
                                                .text_xs()
                                                .text_color(move || theme().muted_foreground),
                                        ),
                                )
                            }),
                    ),
            )
            .child(
                view()
                    .flex_row()
                    .justify_end()
                    .gap_2()
                    .child(action(ButtonKind::Ghost, "关闭", cancel))
                    .child(action(
                        ButtonKind::Primary,
                        move || {
                            if store::is_installed_package(&pkg_name_for_label) {
                                "重新安装"
                            } else {
                                "立即安装"
                            }
                        },
                        move || {
                            store::close_package_detail();
                            store::enqueue_op(ScoopOp::Install(pkg_name_for_install.clone()));
                        },
                    )),
            ),
    )
}

fn meta_row(label: &'static str, value: impl IntoElement) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            text(label)
                .flex_shrink_0()
                .text_xs()
                .text_color(move || theme().muted_foreground),
        )
        .child(value)
}
