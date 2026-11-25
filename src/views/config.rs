//! Configuration view: visual editor for `~/.config/scoop/config.json` (proxy presets,
//! GitHub token, Aria2 acceleration, Scoop mirror/branch, SQLite cache) plus Sundae desktop
//! preferences (Auto-start & `CloseBehavior`). Uses `bento::ui` switches, fields, and floating Notice toasts.

use bento::shell::CloseBehavior;
use nsh_base::theme;
use nsh_core::prelude::*;

use crate::store;
use crate::ui::{
    action, action_sm, card, icons, radio_option, section_header, status_badge, text_field,
    toggle_switch, ButtonKind, ValueTone,
};

pub fn config_view() -> impl IntoElement {
    let initial = store::scoop_config();
    let proxy_input = signal(initial.proxy.unwrap_or_default());
    let token_input = signal(initial.gh_token.unwrap_or_default());
    let repo_input = signal(initial.scoop_repo);
    let branch_input = signal(initial.scoop_branch);
    let aria2_enabled = signal(initial.aria2_enabled);
    let aria2_split = signal(initial.aria2_split.to_string());
    let sqlite_cache = signal(initial.use_sqlite_cache);
    let ignore_running = signal(initial.ignore_running_processes);

    let save = move || {
        let mut cfg = store::scoop_config();

        let proxy_val = proxy_input.get_untracked().trim().to_owned();
        cfg.proxy = if proxy_val.is_empty() {
            None
        } else {
            Some(proxy_val)
        };

        let token_val = token_input.get_untracked().trim().to_owned();
        cfg.gh_token = if token_val.is_empty() {
            None
        } else {
            Some(token_val)
        };

        let repo_val = repo_input.get_untracked().trim().to_owned();
        if !repo_val.is_empty() {
            cfg.scoop_repo = repo_val;
        }
        let branch_val = branch_input.get_untracked().trim().to_owned();
        if !branch_val.is_empty() {
            cfg.scoop_branch = branch_val;
        }

        cfg.aria2_enabled = aria2_enabled.get_untracked();
        cfg.aria2_split = aria2_split
            .get_untracked()
            .trim()
            .parse::<u32>()
            .unwrap_or(5)
            .clamp(1, 32);
        cfg.use_sqlite_cache = sqlite_cache.get_untracked();
        cfg.ignore_running_processes = ignore_running.get_untracked();

        store::save_scoop_config(cfg);
    };

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
                            text("Scoop 选项配置")
                                .role(AccessibilityRole::Header)
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || theme().foreground),
                        )
                        .child(status_badge(
                            "~/.config/scoop/config.json",
                            ValueTone::Neutral,
                        )),
                )
                .child(action(ButtonKind::Primary, "保存配置", save)),
        )
        .child(
            scroll().flex_1().child(
                view()
                    .gap_4()
                    .pr_2()
                    .child(
                        // Section 1: Network & Proxy
                        card()
                            .p_4()
                            .gap_3()
                            .child(section_title_with_icon(
                                icons::GLOBE,
                                "网络代理与 GitHub 访问",
                            ))
                            .child(
                                view()
                                    .gap_2()
                                    .child(
                                        text("HTTP / SOCKS 代理地址（如 127.0.0.1:7890；留空或 default 使用系统代理；none 为直连）")
                                            .text_xs()
                                            .text_color(move || theme().muted_foreground),
                                    )
                                    .child(
                                        text_field(
                                            move || proxy_input.get(),
                                            "代理地址 (例如 127.0.0.1:7890)",
                                            move |t| proxy_input.set(t.to_owned()),
                                        )
                                        .label("Scoop 代理地址"),
                                    )
                                    .child(
                                        view()
                                            .flex_row()
                                            .flex_wrap()
                                            .gap_2()
                                            .child(action_sm(
                                                ButtonKind::Secondary,
                                                "直连 (none)",
                                                move || proxy_input.set("none".to_owned()),
                                            ))
                                            .child(action_sm(
                                                ButtonKind::Secondary,
                                                "系统代理 (default)",
                                                move || proxy_input.set("default".to_owned()),
                                            ))
                                            .child(action_sm(
                                                ButtonKind::Secondary,
                                                "本地 7890 端口",
                                                move || {
                                                    proxy_input.set("127.0.0.1:7890".to_owned())
                                                },
                                            )),
                                    ),
                            )
                            .child(
                                view()
                                    .gap_1p5()
                                    .child(
                                        text("GitHub Token（可选，用于避免 GitHub API 速率限制或访问私有源）")
                                            .text_xs()
                                            .text_color(move || theme().muted_foreground),
                                    )
                                    .child(
                                        text_field(
                                            move || token_input.get(),
                                            "ghp_xxxxxxxxxxxx",
                                            move |t| token_input.set(t.to_owned()),
                                        )
                                        .masked(true)
                                        .label("GitHub Token"),
                                    ),
                            ),
                    )
                    .child(
                        // Section 2: Aria2 Acceleration
                        card()
                            .p_4()
                            .gap_3()
                            .child(section_title_with_icon(
                                icons::ZAP,
                                "Aria2 多线程下载加速",
                            ))
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .gap_3()
                                    .child(
                                        view()
                                            .flex_1()
                                            .gap(px(2.0))
                                            .child(
                                                text("启用 Aria2 多连接下载 (aria2-enabled)")
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(move || theme().foreground),
                                            )
                                            .child(
                                                text("显著加快从 GitHub Releases 及外部镜像下载大型安装包的速度")
                                                    .text_xs()
                                                    .text_color(move || theme().muted_foreground),
                                            ),
                                    )
                                    .child(toggle_switch(
                                        move || aria2_enabled.get(),
                                        "启用 Aria2 多线程下载",
                                        None,
                                        move |next| aria2_enabled.set(next),
                                    )),
                            )
                            .child(
                                view()
                                    .gap_1p5()
                                    .child(
                                        text("多线程分块并发数 (aria2-split，推荐 5 ~ 16)")
                                            .text_xs()
                                            .text_color(move || theme().muted_foreground),
                                    )
                                    .child(
                                        view().w(px(180.0)).child(
                                            text_field(
                                                move || aria2_split.get(),
                                                "分块并发数 (默认 5)",
                                                move |t| aria2_split.set(t.to_owned()),
                                            )
                                            .label("Aria2 分块并发数"),
                                        ),
                                    ),
                            ),
                    )
                    .child(
                        // Section 3: Scoop Source Repository & Branch
                        card()
                            .p_4()
                            .gap_3()
                            .child(section_title_with_icon(
                                icons::GIT_BRANCH,
                                "Scoop 源仓库镜像与分支",
                            ))
                            .child(
                                view()
                                    .gap_2()
                                    .child(
                                        text("Scoop 核心仓库地址 (scoop_repo)")
                                            .text_xs()
                                            .text_color(move || theme().muted_foreground),
                                    )
                                    .child(
                                        text_field(
                                            move || repo_input.get(),
                                            "https://github.com/ScoopInstaller/Scoop",
                                            move |t| repo_input.set(t.to_owned()),
                                        )
                                        .label("Scoop 仓库地址"),
                                    )
                                    .child(
                                        view()
                                            .flex_row()
                                            .gap_2()
                                            .child(action_sm(
                                                ButtonKind::Secondary,
                                                "官方 GitHub 原版",
                                                move || {
                                                    repo_input.set(
                                                        "https://github.com/ScoopInstaller/Scoop"
                                                            .to_owned(),
                                                    )
                                                },
                                            ))
                                            .child(action_sm(
                                                ButtonKind::Secondary,
                                                "Gitee 镜像加速",
                                                move || {
                                                    repo_input.set(
                                                        "https://gitee.com/scoop-installer/scoop"
                                                            .to_owned(),
                                                    )
                                                },
                                            )),
                                    ),
                            )
                            .child(
                                view()
                                    .gap_1p5()
                                    .child(
                                        text("更新分支 (scoop_branch)")
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
                                                        move || branch_input.get(),
                                                        "master",
                                                        move |t| branch_input.set(t.to_owned()),
                                                    )
                                                    .label("Scoop 分支"),
                                                ),
                                            )
                                            .child(action_sm(
                                                ButtonKind::Secondary,
                                                "master (稳定)",
                                                move || branch_input.set("master".to_owned()),
                                            ))
                                            .child(action_sm(
                                                ButtonKind::Secondary,
                                                "develop (预览)",
                                                move || branch_input.set("develop".to_owned()),
                                            )),
                                    ),
                            ),
                    )
                    .child(
                        // Section 4: Advanced & Compatibility
                        card()
                            .p_4()
                            .gap_3()
                            .child(section_title_with_icon(
                                icons::SLIDERS,
                                "高级与兼容性选项",
                            ))
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .gap_3()
                                    .child(
                                        view()
                                            .flex_1()
                                            .gap(px(2.0))
                                            .child(
                                                text("启用 SQLite 数据库缓存 (use_sqlite_cache)")
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(move || theme().foreground),
                                            )
                                            .child(
                                                text("加速命令行 scoop search 与 shim 索引查询")
                                                    .text_xs()
                                                    .text_color(move || theme().muted_foreground),
                                            ),
                                    )
                                    .child(toggle_switch(
                                        move || sqlite_cache.get(),
                                        "启用 SQLite 缓存",
                                        None,
                                        move |next| sqlite_cache.set(next),
                                    )),
                            )
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .gap_3()
                                    .child(
                                        view()
                                            .flex_1()
                                            .gap(px(2.0))
                                            .child(
                                                text("忽略运行中进程 (ignore_running_processes)")
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(move || theme().foreground),
                                            )
                                            .child(
                                                text("目标应用正在运行时不阻止更新或卸载流程")
                                                    .text_xs()
                                                    .text_color(move || theme().muted_foreground),
                                            ),
                                    )
                                    .child(toggle_switch(
                                        move || ignore_running.get(),
                                        "忽略运行中进程",
                                        None,
                                        move |next| ignore_running.set(next),
                                    )),
                            ),
                    )
                    .child(
                        // Section 5: Sundae Desktop Preferences (Auto-start & CloseBehavior)
                        card()
                            .p_4()
                            .gap_3()
                            .child(section_title_with_icon(
                                icons::SETTINGS,
                                "Sundae 窗口与托盘行为",
                            ))
                            .child(
                                view()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .gap_3()
                                    .child(
                                        view()
                                            .flex_1()
                                            .gap(px(2.0))
                                            .child(
                                                text("开机自启")
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(move || theme().foreground),
                                            )
                                            .child(
                                                text("登录系统时自动启动 Sundae（即时生效，与托盘菜单同步）")
                                                    .text_xs()
                                                    .text_color(move || theme().muted_foreground),
                                            ),
                                    )
                                    .child(toggle_switch(
                                        store::autostart_enabled,
                                        "开机自启",
                                        None,
                                        store::set_autostart_enabled,
                                    )),
                            )
                            .child(
                                view()
                                    .h(px(1.0))
                                    .w_full()
                                    .bg(move || theme().border),
                            )
                            .child(
                                text("点击主窗口右上角关闭按钮时的默认行为（即时保存生效）：")
                                    .text_xs()
                                    .text_color(move || theme().muted_foreground),
                            )
                            .child(
                                view()
                                    .gap_2()
                                    .child(radio_option(
                                        move || store::close_behavior() == CloseBehavior::Ask,
                                        CloseBehavior::Ask.label(),
                                        "关闭窗口时弹出确认框，自由选择最小化到托盘或退出",
                                        "每次询问",
                                        || store::set_close_behavior(CloseBehavior::Ask),
                                    ))
                                    .child(radio_option(
                                        move || store::close_behavior() == CloseBehavior::Tray,
                                        CloseBehavior::Tray.label(),
                                        "隐藏主窗口并驻留系统托盘，适合后台继续执行 Scoop 任务",
                                        "最小化到托盘",
                                        || store::set_close_behavior(CloseBehavior::Tray),
                                    ))
                                    .child(radio_option(
                                        move || store::close_behavior() == CloseBehavior::Exit,
                                        CloseBehavior::Exit.label(),
                                        "直接退出 Sundae 主程序与托盘图标",
                                        "直接退出应用",
                                        || store::set_close_behavior(CloseBehavior::Exit),
                                    )),
                            ),
                    ),
            ),
        )
}

fn section_title_with_icon(icon_markup: &'static str, title: &'static str) -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .gap_2()
        .child(icons::svg_icon(icon_markup, 16.0, move || theme().primary))
        .child(section_header(title))
}
