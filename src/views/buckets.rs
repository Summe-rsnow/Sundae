use crate::scoop::process::ScoopOp;
use crate::store::AppState;
use crate::ui::*;
use nsh_base::theme;
use nsh_core::prelude::*;

pub fn buckets_view(state: AppState) -> impl IntoElement {
    let index_sig = state.search_index;
    let new_bucket_name = signal(String::new());
    let new_bucket_url = signal(String::new());

    let popular_buckets = vec![
        ("main", "Scoop 默认官方基础软件源"),
        ("extras", "GUI 常用桌面软件及增强工具库"),
        ("versions", "软件包历史版本与替代版本库"),
        ("nerd-fonts", "开发者编程字体及图标字体库"),
        ("sysinternals", "微软官方 Sysinternals 系统诊断工具库"),
        ("java", "Java 开发环境与各类 JDK 发行版集合"),
    ];

    view()
        .flex_1()
        .h_full()
        .p_6()
        .gap_6()
        .child(
            // Header
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(text("Bucket 仓库源管理").text_xl().font_weight(FontWeight::BOLD))
                        .child(dynamic({
                            move || {
                                let count = index_sig.with(|i| i.buckets().len());
                                app_badge(format!("已配置 {} 个", count), BadgeVariant::Primary).into_element()
                            }
                        })),
                )
                .child(
                    app_button("一键同步全部源", ButtonVariant::Primary, ButtonSize::Md, {
                        let s = state.clone();
                        move || s.enqueue_op(ScoopOp::BucketUpdate)
                    }),
                ),
        )
        .child(
            // Body container with scroll
            scroll()
                .flex_1()
                .child(
                    view()
                        .gap_6()
                        .child(
                            // Added Buckets List
                            view()
                                .gap_3()
                                .child(text("当前已添加的源").text_base().font_weight(FontWeight::BOLD))
                                .child(dynamic({
                                    let state = state.clone();
                                    move || {
                                        let buckets = index_sig.with(|i| i.buckets().to_vec());

                                        if buckets.is_empty() {
                                            return view()
                                                .p_4()
                                                .child(text("暂无已配置的 Bucket，请在下方添加推荐源").text_xs().text_color(move || theme().muted_foreground))
                                                .into_element();
                                        }

                                        let mut b_list = view().gap_2();

                                        for b in buckets {
                                            let s_clone = state.clone();
                                            let b_name = b.clone();
                                            let b_remove = b.clone();
                                            let is_default_main = b_name == "main";

                                            b_list = b_list.child(
                                                app_card()
                                                    .flex_row()
                                                    .items_center()
                                                    .justify_between()
                                                    .p_4()
                                                    .child(
                                                        view()
                                                            .flex_row()
                                                            .items_center()
                                                            .gap_3()
                                                            .child(bucket_icon(20.0))
                                                            .child(
                                                                view()
                                                                    .gap_1()
                                                                    .child(text(b_name).text_base().font_weight(FontWeight::BOLD))
                                                                    .child(text("本地 git 镜像同步就绪").text_xs().text_color(move || theme().muted_foreground)),
                                                            ),
                                                    )
                                                    .child(
                                                        view()
                                                            .flex_row()
                                                            .items_center()
                                                            .gap_2()
                                                            .child(app_badge("活跃", BadgeVariant::Success))
                                                            .child(if !is_default_main {
                                                                app_button("移除", ButtonVariant::Danger, ButtonSize::Sm, move || {
                                                                    s_clone.enqueue_op(ScoopOp::BucketRm(b_remove.clone()));
                                                                })
                                                                .into_element()
                                                            } else {
                                                                view().into_element()
                                                            }),
                                                    ),
                                            );
                                        }

                                        b_list.into_element()
                                    }
                                })),
                        )
                        .child(
                            // Recommended Buckets
                            view()
                                .gap_3()
                                .child(text("官方与热门推荐源").text_base().font_weight(FontWeight::BOLD))
                                .child(dynamic({
                                    let state = state.clone();
                                    let popular = popular_buckets.clone();
                                    move || {
                                        let current_buckets = index_sig.with(|i| i.buckets().to_vec());
                                        let mut pop_list = view().gap_2();

                                        for (name, desc) in &popular {
                                            let is_added = current_buckets.iter().any(|b| b == name);
                                            let s_clone = state.clone();
                                            let n_str = (*name).to_string();

                                            pop_list = pop_list.child(
                                                app_card()
                                                    .flex_row()
                                                    .items_center()
                                                    .justify_between()
                                                    .p_3()
                                                    .child(
                                                        view()
                                                            .gap_1()
                                                            .child(text(*name).text_sm().font_weight(FontWeight::BOLD))
                                                            .child(text(*desc).text_xs().text_color(move || theme().muted_foreground)),
                                                    )
                                                    .child(if is_added {
                                                        app_badge("已添加", BadgeVariant::Secondary).into_element()
                                                    } else {
                                                        app_button("添加", ButtonVariant::Secondary, ButtonSize::Sm, move || {
                                                            s_clone.enqueue_op(ScoopOp::BucketAdd {
                                                                name: n_str.clone(),
                                                                url: None,
                                                            });
                                                        })
                                                        .into_element()
                                                    }),
                                            );
                                        }

                                        pop_list.into_element()
                                    }
                                })),
                        )
                        .child(
                            // Add Custom Bucket Section
                            app_card()
                                .gap_3()
                                .child(text("添加第三方自定义源").text_base().font_weight(FontWeight::BOLD))
                                .child(
                                    view()
                                        .flex_row()
                                        .gap_3()
                                        .child(
                                            view()
                                                .w(px(200.0))
                                                .child(app_text_field("源名称 (例如: dorado)", new_bucket_name, move |t| new_bucket_name.set(t))),
                                        )
                                        .child(
                                            app_text_field("Git 仓库地址 (可选，支持 https://...)", new_bucket_url, move |t| new_bucket_url.set(t)),
                                        )
                                        .child(app_button("确认添加", ButtonVariant::Primary, ButtonSize::Md, {
                                            let s = state.clone();
                                            move || {
                                                let name = new_bucket_name.get().trim().to_string();
                                                let url = new_bucket_url.get().trim().to_string();
                                                if !name.is_empty() {
                                                    s.enqueue_op(ScoopOp::BucketAdd {
                                                        name,
                                                        url: if url.is_empty() { None } else { Some(url) },
                                                    });
                                                    new_bucket_name.set(String::new());
                                                    new_bucket_url.set(String::new());
                                                }
                                            }
                                        })),
                                ),
                        ),
                ),
        )
}
