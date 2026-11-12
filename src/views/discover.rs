use crate::scoop::process::ScoopOp;
use crate::store::AppState;
use crate::ui::*;
use nsh_base::{button, theme};
use nsh_core::prelude::*;

pub fn discover_view(state: AppState) -> impl IntoElement {
    let query_sig = state.search_query;
    let filter_sig = state.selected_bucket_filter;
    let index_sig = state.search_index;
    let is_indexing_sig = state.is_indexing;
    let installed_sig = state.installed_apps;

    view()
        .flex_1()
        .h_full()
        .p_6()
        .gap_4()
        .child(
            // Top search and filters bar
            view()
                .gap_3()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(search_bar(
                            "快速搜索应用或描述 (例如: git, ripgrep, neovim)...",
                            query_sig,
                            move |text| query_sig.set(text),
                        )),
                )
                .child(
                    // Bucket Filter Chips
                    dynamic({
                        move || {
                            let buckets = index_sig.with(|i| i.buckets().to_vec());
                            let current_filter = filter_sig.get();

                            let mut chips = view().flex_row().items_center().gap_2();

                            // "All" chip
                            let is_all = current_filter.is_none();
                            chips = chips.child(
                                button()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(999.0))
                                    .bg(move || {
                                        if is_all {
                                            theme().primary
                                        } else {
                                            theme().muted
                                        }
                                    })
                                    .on_press(move || filter_sig.set(None))
                                    .child(
                                        text("全部源")
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(move || {
                                                if is_all {
                                                    theme().primary_foreground
                                                } else {
                                                    theme().foreground
                                                }
                                            }),
                                    ),
                            );

                            for b in buckets {
                                let b_clone = b.clone();
                                let is_active = current_filter.as_deref() == Some(&b);
                                chips = chips.child(
                                    button()
                                        .px_3()
                                        .py_1()
                                        .rounded(px(999.0))
                                        .bg(move || {
                                            if is_active {
                                                theme().primary
                                            } else {
                                                theme().muted
                                            }
                                        })
                                        .on_press(move || filter_sig.set(Some(b_clone.clone())))
                                        .child(
                                            text(b)
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(move || {
                                                    if is_active {
                                                        theme().primary_foreground
                                                    } else {
                                                        theme().foreground
                                                    }
                                                }),
                                        ),
                                );
                            }

                            chips.into_element()
                        }
                    }),
                ),
        )
        .child(
            // Results list
            dynamic({
                let state = state.clone();
                move || {
                    if is_indexing_sig.get() {
                        return view()
                            .flex_1()
                            .items_center()
                            .justify_center()
                            .child(text("正在建立本地 Bucket 索引...").text_sm().text_color(move || theme().muted_foreground))
                            .into_element();
                    }

                    let query = query_sig.get();
                    let bucket_filter = filter_sig.get();

                    let results: Vec<_> = index_sig.with(|idx| {
                        idx.search(&query, bucket_filter.as_deref())
                            .into_iter()
                            .take(150) // Render up to 150 items for crisp performance
                            .cloned()
                            .collect()
                    });

                    if results.is_empty() {
                        return view()
                            .flex_1()
                            .items_center()
                            .justify_center()
                            .gap_3()
                            .child(search_icon(36.0))
                            .child(text("没有找到符合条件的应用").text_base().font_weight(FontWeight::MEDIUM))
                            .child(text("请检查搜索关键词或尝试切换 Bucket 源").text_xs().text_color(move || theme().muted_foreground))
                            .into_element();
                    }

                    let mut list = view().gap_2();

                    for pkg in results {
                        let pkg_name = pkg.name.clone();
                        let is_installed = installed_sig.with(|apps| apps.iter().any(|a| a.name == pkg_name));
                        let s_clone = state.clone();
                        let pkg_name_clone = pkg.name.clone();

                        list = list.child(
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
                                                .child(text(pkg.name).text_base().font_weight(FontWeight::BOLD))
                                                .child(app_badge(pkg.bucket, BadgeVariant::Secondary))
                                                .child(text(format!("v{}", pkg.version)).text_xs().text_color(move || theme().muted_foreground)),
                                        )
                                        .child({
                                            let desc = if pkg.description.is_empty() {
                                                "暂无描述".to_string()
                                            } else {
                                                pkg.description
                                            };
                                            text(desc).text_sm().text_color(move || theme().muted_foreground)
                                        }),
                                )
                                .child(
                                    view()
                                        .items_center()
                                        .gap_2()
                                        .child(if is_installed {
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .child(app_badge("已安装", BadgeVariant::Success))
                                                .child(app_button(
                                                    "重装",
                                                    ButtonVariant::Secondary,
                                                    ButtonSize::Sm,
                                                    move || {
                                                        s_clone.enqueue_op(ScoopOp::Install(pkg_name_clone.clone()));
                                                    },
                                                ))
                                                .into_element()
                                        } else {
                                            app_button(
                                                "安装",
                                                ButtonVariant::Primary,
                                                ButtonSize::Sm,
                                                move || {
                                                    s_clone.enqueue_op(ScoopOp::Install(pkg_name_clone.clone()));
                                                },
                                            )
                                            .into_element()
                                        }),
                                ),
                        );
                    }

                    scroll()
                        .flex_1()
                        .pr_4()
                        .child(list)
                        .into_element()
                }
            }),
        )
}
