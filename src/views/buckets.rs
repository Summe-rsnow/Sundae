//! Buckets management view: displays configured local Scoop buckets with package
//! counts, recommended official buckets, and a custom Git bucket form.
//! Uses `each` for the small bounded lists and opens `confirm_remove_bucket_dialog`
//! before any destructive `scoop bucket rm`.

use nsh_base::theme;
use nsh_core::prelude::*;

use crate::scoop::process::ScoopOp;
use crate::store;
use crate::ui::{
    action, action_disabled, action_sm, card, icons, section_header, status_badge, text_field,
    ButtonKind, ValueTone,
};

const POPULAR_BUCKETS: &[(&str, &str)] = &[
    ("main", "Scoop 默认官方基础命令行与开发工具源"),
    ("extras", "GUI 常用桌面软件及增强工具库"),
    ("versions", "软件包历史版本与替代版本库"),
    ("nerd-fonts", "开发者编程字体与图标字体库"),
    ("sysinternals", "微软官方 Sysinternals 系统诊断工具库"),
    ("java", "Java 开发环境与各类 JDK 发行版集合"),
    ("nonportable", "非绿色安装类桌面软件源"),
    ("games", "开源与独立游戏软件库"),
];

pub fn buckets_view() -> impl IntoElement {
    let configured_buckets = computed(store::buckets);
    let new_bucket_name = signal(String::new());
    let new_bucket_url = signal(String::new());

    view()
        .flex_1()
        .h_full()
        .p_5()
        .gap_4()
        .child(
            // Header
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
                            text("Bucket 仓库源")
                                .role(AccessibilityRole::Header)
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(status_badge(
                            move || format!("已配置 {} 个", store::bucket_count()),
                            ValueTone::Info,
                        )),
                )
                .child(action_disabled(
                    ButtonKind::Primary,
                    "同步全部源",
                    store::is_task_running,
                    || store::enqueue_op(ScoopOp::BucketUpdate),
                )),
        )
        .child(
            scroll().flex_1().child(
                view()
                    .gap_5()
                    .pr_2()
                    .child(
                        // Section 1: Added Buckets
                        view()
                            .gap_2()
                            .child(section_header("已添加的仓库源"))
                            .child(when(
                                move || configured_buckets.with(Vec::is_empty),
                                || {
                                    card().p_4().child(
                                        text("暂无已配置的 Bucket，可从下方官方推荐源一键添加。")
                                            .text_xs()
                                            .text_color(move || theme().muted_foreground),
                                    )
                                },
                            ))
                            .child(
                                view().gap_2().child(each(
                                    configured_buckets,
                                    |b: &String| b.clone(),
                                    |b_sig, _idx| {
                                        let bucket_name = b_sig.get_untracked();
                                        let bucket_count_name = bucket_name.clone();
                                        let bucket_rm = bucket_name.clone();
                                        let is_main = bucket_name.eq_ignore_ascii_case("main");

                                        card()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .px_4()
                                            .py_3()
                                            .child(
                                                view()
                                                    .flex_row()
                                                    .items_center()
                                                    .gap_3()
                                                    .child(icons::svg_icon(
                                                        icons::DATABASE,
                                                        16.0,
                                                        move || theme().primary,
                                                    ))
                                                    .child(
                                                        view()
                                                            .gap(px(2.0))
                                                            .child(
                                                                text(bucket_name)
                                                                    .text_sm()
                                                                    .font_weight(
                                                                        FontWeight::SEMIBOLD,
                                                                    )
                                                                    .text_color(move || {
                                                                        theme().foreground
                                                                    }),
                                                            )
                                                            .child(
                                                                text(move || {
                                                                    format!(
                                                                        "包含 {} 个软件包清单",
                                                                        store::bucket_package_count(
                                                                            &bucket_count_name
                                                                        )
                                                                    )
                                                                })
                                                                .text_xs()
                                                                .tabular_nums()
                                                                .text_color(move || {
                                                                    theme().muted_foreground
                                                                }),
                                                            ),
                                                    ),
                                            )
                                            .child(
                                                view()
                                                    .flex_row()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(status_badge("活跃", ValueTone::Ok))
                                                    .when(!is_main, |v| {
                                                        v.child(action_sm(
                                                            ButtonKind::Destructive,
                                                            "移除",
                                                            move || {
                                                                store::request_remove_bucket(
                                                                    bucket_rm.clone(),
                                                                );
                                                            },
                                                        ))
                                                    }),
                                            )
                                    },
                                )),
                            ),
                    )
                    .child(
                        // Section 2: Official & Popular Recommended Buckets
                        view()
                            .gap_2()
                            .child(section_header("官方与热门推荐源"))
                            .child(view().gap_2().children(POPULAR_BUCKETS.iter().map(
                                |&(name, desc)| {
                                    let is_added = move || {
                                        configured_buckets.with(|list| {
                                            list.iter().any(|b| b.eq_ignore_ascii_case(name))
                                        })
                                    };
                                    card()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .px_4()
                                        .py(px(10.0))
                                        .child(
                                            view()
                                                .flex_1()
                                                .min_w(px(0.0))
                                                .gap(px(2.0))
                                                .child(
                                                    text(name)
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(move || theme().foreground),
                                                )
                                                .child(
                                                    text(desc)
                                                        .text_xs()
                                                        .text_color(move || {
                                                            theme().muted_foreground
                                                        }),
                                                ),
                                        )
                                        .child(when(is_added, || {
                                            status_badge("已添加", ValueTone::Neutral)
                                        }))
                                        .child(when(
                                            move || !is_added(),
                                            move || {
                                                action_sm(ButtonKind::Secondary, "添加", move || {
                                                    store::enqueue_op(ScoopOp::BucketAdd {
                                                        name: name.to_owned(),
                                                        url: None,
                                                    });
                                                })
                                            },
                                        ))
                                },
                            ))),
                    )
                    .child(
                        // Section 3: Add Custom Third-Party Git Bucket
                        card()
                            .p_4()
                            .gap_3()
                            .child(section_header("添加自定义第三方仓库源"))
                            .child(
                                text("输入自定义 Bucket 名称与 Git 仓库地址（若为 Scoop 官方已知源，仓库地址可留空）。")
                                    .text_xs()
                                    .text_color(move || theme().muted_foreground),
                            )
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        view().w(px(180.0)).child(
                                            text_field(
                                                move || new_bucket_name.get(),
                                                "源名称 (如 dorado)",
                                                move |t| new_bucket_name.set(t.to_owned()),
                                            )
                                            .label("自定义仓库源名称"),
                                        ),
                                    )
                                    .child(
                                        view().flex_1().min_w(px(0.0)).child(
                                            text_field(
                                                move || new_bucket_url.get(),
                                                "Git 仓库地址 (可选，https://...)",
                                                move |t| new_bucket_url.set(t.to_owned()),
                                            )
                                            .label("自定义仓库 Git 地址"),
                                        ),
                                    )
                                    .child(action(ButtonKind::Primary, "添加源", move || {
                                        let name =
                                            new_bucket_name.get_untracked().trim().to_owned();
                                        let url =
                                            new_bucket_url.get_untracked().trim().to_owned();
                                        if name.is_empty() {
                                            store::notices().notify_warn("请先输入仓库源名称");
                                            return;
                                        }
                                        store::enqueue_op(ScoopOp::BucketAdd {
                                            name,
                                            url: if url.is_empty() { None } else { Some(url) },
                                        });
                                        new_bucket_name.set(String::new());
                                        new_bucket_url.set(String::new());
                                    })),
                            ),
                    ),
            ),
        )
}
