//! Discover view: instant search across thousands of Scoop bucket manifests,
//! horizontal bucket filter chips, and a zero-jank `virtual_list` with zero
//! per-row local signals.

use nsh_base::{button as base_button, theme};
use nsh_core::prelude::*;

use crate::scoop::process::ScoopOp;
use crate::store;
use crate::ui::{
    action_sm, empty_state, filter_chip, focus_ring, radius, search_input, status_badge,
    ButtonKind, ValueTone,
};

pub fn discover_view() -> impl IntoElement {
    let results = computed(store::filtered_discover_packages);
    let buckets_memo = computed(store::buckets);

    view()
        .flex_1()
        .h_full()
        .p_5()
        .gap_3()
        .child(
            // Header & Search Bar
            view()
                .gap_3()
                .child(
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
                                    text("探索软件")
                                        .role(AccessibilityRole::Header)
                                        .text_lg()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(move || theme().foreground),
                                )
                                .child(status_badge(
                                    move || format!("{} 项结果", results.with(Vec::len)),
                                    ValueTone::Neutral,
                                )),
                        )
                        .child(
                            view().w(px(320.0)).child(search_input(
                                store::search_query,
                                "搜索软件名称、命令或描述 (如 git, ripgrep)…",
                                "搜索软件",
                                |text| store::set_search_query(text),
                                || store::set_search_query(""),
                            )),
                        ),
                )
                .child(
                    // Bucket filter chip bar (horizontal scroll + `each`, zero `dynamic`)
                    scroll().horizontal().scrollbar(false).child(
                        view()
                            .test_id("bucket-filters")
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(filter_chip(
                                move || store::bucket_filter().is_none(),
                                "全部源",
                                || store::set_bucket_filter(None),
                            ))
                            .child(each(
                                buckets_memo,
                                |bucket: &String| bucket.clone(),
                                |bucket_sig, _idx| {
                                    let name_press = bucket_sig.get_untracked();
                                    let name_label = name_press.clone();
                                    filter_chip(
                                        move || {
                                            bucket_sig.with(|b| {
                                                store::bucket_filter().as_deref()
                                                    == Some(b.as_str())
                                            })
                                        },
                                        name_label,
                                        move || store::set_bucket_filter(Some(name_press.clone())),
                                    )
                                },
                            )),
                    ),
                ),
        )
        .child(
            // Loading state when indexing with no cached items yet
            when(
                move || store::is_indexing() && results.with(Vec::is_empty),
                || {
                    empty_state(
                        "正在后台扫描本地 Bucket 索引…",
                        "使用多线程并行解析 Scoop Manifest，界面保持响应",
                    )
                },
            ),
        )
        .child(
            // Virtualized package list (zero per-row local signals)
            when(
                move || !(store::is_indexing() && results.with(Vec::is_empty)),
                move || {
                    virtual_list(move || results.with(Vec::len), move |at| {
                        let pkg = computed(move || {
                            results.with(|items| items.get(at.index()).cloned())
                        });
                        let is_installed = computed(move || {
                            pkg.with(|p| {
                                p.as_ref()
                                    .is_some_and(|m| store::is_installed_package(&m.name))
                            })
                        });

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
                                // Clickable package summary opening Package Detail Dialog
                                base_button()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .items_start()
                                    .gap_1()
                                    .rounded(px(4.0))
                                    .focus_visible(focus_ring)
                                    .accessibility_label(move || {
                                        Some(SharedString::from(pkg.with(|p| {
                                            p.as_ref()
                                                .map(|m| format!("查看 {} 详情", m.name))
                                                .unwrap_or_default()
                                        })))
                                    })
                                    .on_press(move || {
                                        if let Some(m) = pkg.get_untracked() {
                                            store::open_package_detail(m);
                                        }
                                    })
                                    .child(
                                        view()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                text(move || {
                                                    pkg.with(|p| {
                                                        p.as_ref()
                                                            .map(|m| m.name.clone())
                                                            .unwrap_or_default()
                                                    })
                                                })
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(move || theme().foreground),
                                            )
                                            .child(status_badge(
                                                move || {
                                                    pkg.with(|p| {
                                                        p.as_ref()
                                                            .map(|m| m.bucket.clone())
                                                            .unwrap_or_default()
                                                    })
                                                },
                                                ValueTone::Info,
                                            ))
                                            .child(
                                                text(move || {
                                                    pkg.with(|p| {
                                                        p.as_ref()
                                                            .map(|m| format!("v{}", m.version))
                                                            .unwrap_or_default()
                                                    })
                                                })
                                                .text_xs()
                                                .tabular_nums()
                                                .text_color(move || theme().muted_foreground),
                                            ),
                                    )
                                    .child(
                                        text(move || {
                                            pkg.with(|p| {
                                                p.as_ref()
                                                    .map(|m| m.display_description().to_owned())
                                                    .unwrap_or_default()
                                            })
                                        })
                                        .w_full()
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
                                    .gap_2()
                                    .child(when(
                                        move || is_installed.get(),
                                        || status_badge("已安装", ValueTone::Ok),
                                    ))
                                    .child(action_sm(ButtonKind::Ghost, "详情", move || {
                                        if let Some(m) = pkg.get_untracked() {
                                            store::open_package_detail(m);
                                        }
                                    }))
                                    .child(when(
                                        move || is_installed.get(),
                                        move || {
                                            action_sm(ButtonKind::Secondary, "重装", move || {
                                                if let Some(m) = pkg.get_untracked() {
                                                    store::enqueue_op(ScoopOp::Install(m.name));
                                                }
                                            })
                                        },
                                    ))
                                    .child(when(
                                        move || !is_installed.get(),
                                        move || {
                                            action_sm(ButtonKind::Primary, "安装", move || {
                                                if let Some(m) = pkg.get_untracked() {
                                                    store::enqueue_op(ScoopOp::Install(m.name));
                                                }
                                            })
                                        },
                                    )),
                            )
                    })
                    .keys(move || {
                        results.with(|items| {
                            items
                                .iter()
                                .map(|p| format!("{}/{}", p.bucket, p.name))
                                .collect::<Vec<_>>()
                        })
                    })
                    .estimated_item_size(px(74.0))
                    .row_gap(px(8.0))
                    .empty(empty_state(
                        "没有找到匹配的软件包",
                        "尝试修改搜索关键词，或切换到「全部源」查看所有可用应用",
                    ))
                    .test_id("discover-list")
                    .flex_1()
                },
            ),
        )
}
