use crate::store::AppState;
use crate::ui::*;
use nsh_base::theme;
use nsh_core::prelude::*;

pub fn config_view(state: AppState) -> impl IntoElement {
    let cfg_sig = state.scoop_config;

    // Local draft states
    let initial_cfg = cfg_sig.get();
    let proxy_input = signal(initial_cfg.proxy.unwrap_or_default());
    let token_input = signal(initial_cfg.gh_token.unwrap_or_default());
    let repo_input = signal(initial_cfg.scoop_repo);
    let branch_input = signal(initial_cfg.scoop_branch);
    let aria2_enabled = signal(initial_cfg.aria2_enabled);
    let aria2_split = signal(initial_cfg.aria2_split.to_string());
    let sqlite_cache = signal(initial_cfg.use_sqlite_cache);
    let ignore_running = signal(initial_cfg.ignore_running_processes);

    let save_status = signal(Option::<String>::None);

    view()
        .flex_1()
        .h_full()
        .p_6()
        .gap_6()
        .child(
            // Header Bar
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(text("Scoop 选项配置").text_xl().font_weight(FontWeight::BOLD))
                        .child(app_badge("~/.config/scoop/config.json", BadgeVariant::Secondary)),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(dynamic({
                            move || {
                                if let Some(msg) = save_status.get() {
                                    app_badge(msg, BadgeVariant::Success).into_element()
                                } else {
                                    view().into_element()
                                }
                            }
                        }))
                        .child(app_button("保存配置", ButtonVariant::Primary, ButtonSize::Md, {
                            let cfg_sig = cfg_sig;
                            let proxy_input = proxy_input;
                            let token_input = token_input;
                            let repo_input = repo_input;
                            let branch_input = branch_input;
                            let aria2_enabled = aria2_enabled;
                            let aria2_split = aria2_split;
                            let sqlite_cache = sqlite_cache;
                            let ignore_running = ignore_running;
                            let save_status = save_status;

                            move || {
                                let mut current = cfg_sig.get();

                                let proxy_val = proxy_input.get().trim().to_string();
                                current.proxy = if proxy_val.is_empty() { None } else { Some(proxy_val) };

                                let token_val = token_input.get().trim().to_string();
                                current.gh_token = if token_val.is_empty() { None } else { Some(token_val) };

                                current.scoop_repo = repo_input.get().trim().to_string();
                                current.scoop_branch = branch_input.get().trim().to_string();
                                current.aria2_enabled = aria2_enabled.get();
                                current.aria2_split = aria2_split.get().trim().parse().unwrap_or(5);
                                current.use_sqlite_cache = sqlite_cache.get();
                                current.ignore_running_processes = ignore_running.get();

                                match current.save() {
                                    Ok(_) => {
                                        cfg_sig.set(current);
                                        save_status.set(Some("✓ 配置已保存生效".to_string()));
                                    }
                                    Err(e) => {
                                        save_status.set(Some(format!("保存失败: {}", e)));
                                    }
                                }
                            }
                        })),
                ),
        )
        .child(
            // Scrollable Settings Sections
            scroll()
                .flex_1()
                .child(
                    view()
                        .gap_6()
                        .child(
                            // Section 1: Network & Proxy
                            app_card()
                                .gap_4()
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(globe_icon(18.0))
                                        .child(text("网络代理与 GitHub 访问").text_base().font_weight(FontWeight::BOLD)),
                                )
                                .child(
                                    view()
                                        .gap_2()
                                        .child(text("HTTP / SOCKS 代理地址 (例如 127.0.0.1:7890，留空或设为 default 为系统代理，设为 none 直连):").text_xs().text_color(move || theme().muted_foreground))
                                        .child(search_bar("代理地址...", proxy_input, move |t| proxy_input.set(t)))
                                        .child(
                                            view()
                                                .flex_row()
                                                .gap_2()
                                                .child(app_button("直连 (none)", ButtonVariant::Secondary, ButtonSize::Sm, move || proxy_input.set("none".into())))
                                                .child(app_button("系统代理 (default)", ButtonVariant::Secondary, ButtonSize::Sm, move || proxy_input.set("default".into())))
                                                .child(app_button("本地 7890 端口", ButtonVariant::Secondary, ButtonSize::Sm, move || proxy_input.set("127.0.0.1:7890".into()))),
                                        ),
                                )
                                .child(
                                    view()
                                        .gap_2()
                                        .child(text("GitHub Token (可选，配置后可避免 GitHub API 速率限制或用于私有仓库):").text_xs().text_color(move || theme().muted_foreground))
                                        .child(search_bar("ghp_xxxxxxxxxxxx", token_input, move |t| token_input.set(t))),
                                ),
                        )
                        .child(
                            // Section 2: Aria2 Acceleration
                            app_card()
                                .gap_4()
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(zap_icon(18.0))
                                        .child(text("Aria2 多线程下载加速").text_base().font_weight(FontWeight::BOLD)),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            view()
                                                .gap_1()
                                                .child(text("启用 Aria2 多连接下载 (aria2-enabled)").text_sm().font_weight(FontWeight::MEDIUM))
                                                .child(text("大幅加快从 GitHub Releases 或外部源下载大体积应用的速度").text_xs().text_color(move || theme().muted_foreground)),
                                        )
                                        .child(dynamic({
                                            move || {
                                                let is_on = aria2_enabled.get();
                                                app_button(
                                                    if is_on { "已启用 (点击禁用)" } else { "已禁用 (点击启用)" },
                                                    if is_on { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                                    ButtonSize::Sm,
                                                    move || aria2_enabled.update(|v| *v = !*v),
                                                )
                                                .into_element()
                                            }
                                        })),
                                )
                                .child(
                                    view()
                                        .gap_2()
                                        .child(text("多线程连接并发数 (aria2-split，推荐 5 ~ 16):").text_xs().text_color(move || theme().muted_foreground))
                                        .child(
                                            view()
                                                .w(px(200.0))
                                                .child(search_bar("分块并发数 (默认 5)", aria2_split, move |t| aria2_split.set(t))),
                                        ),
                                ),
                        )
                        .child(
                            // Section 3: Scoop Source Repository & Branch
                            app_card()
                                .gap_4()
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(git_branch_icon(18.0))
                                        .child(text("Scoop 源仓库镜像与分支").text_base().font_weight(FontWeight::BOLD)),
                                )
                                .child(
                                    view()
                                        .gap_2()
                                        .child(text("Scoop 源码仓库地址 (scoop_repo):").text_xs().text_color(move || theme().muted_foreground))
                                        .child(search_bar("Scoop Git 仓库 URL...", repo_input, move |t| repo_input.set(t)))
                                        .child(
                                            view()
                                                .flex_row()
                                                .gap_2()
                                                .child(app_button("国内 Gitee 镜像加速", ButtonVariant::Secondary, ButtonSize::Sm, move || repo_input.set("https://gitee.com/scoop-installer/scoop".into())))
                                                .child(app_button("官方 GitHub 原版", ButtonVariant::Secondary, ButtonSize::Sm, move || repo_input.set("https://github.com/ScoopInstaller/Scoop".into()))),
                                        ),
                                )
                                .child(
                                    view()
                                        .gap_2()
                                        .child(text("分支 (scoop_branch，可选 master 或 develop):").text_xs().text_color(move || theme().muted_foreground))
                                        .child(
                                            view()
                                                .flex_row()
                                                .gap_3()
                                                .child(
                                                    view()
                                                        .w(px(200.0))
                                                        .child(search_bar("分支名称", branch_input, move |t| branch_input.set(t))),
                                                )
                                                .child(app_button("master (稳定)", ButtonVariant::Secondary, ButtonSize::Sm, move || branch_input.set("master".into())))
                                                .child(app_button("develop (预览)", ButtonVariant::Secondary, ButtonSize::Sm, move || branch_input.set("develop".into()))),
                                        ),
                                ),
                        )
                        .child(
                            // Section 4: Performance & Advanced Behaviors
                            app_card()
                                .gap_4()
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap_2()
                                        .child(sliders_icon(18.0))
                                        .child(text("高级与兼容性设置").text_base().font_weight(FontWeight::BOLD)),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            view()
                                                .gap_1()
                                                .child(text("使用 SQLite 数据库缓存 (use_sqlite_cache)").text_sm().font_weight(FontWeight::MEDIUM))
                                                .child(text("加速 scoop search 与 scoop shim 命令行查询").text_xs().text_color(move || theme().muted_foreground)),
                                        )
                                        .child(dynamic({
                                            move || {
                                                let is_on = sqlite_cache.get();
                                                app_button(
                                                    if is_on { "已启用" } else { "未启用" },
                                                    if is_on { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                                    ButtonSize::Sm,
                                                    move || sqlite_cache.update(|v| *v = !*v),
                                                )
                                                .into_element()
                                            }
                                        })),
                                )
                                .child(
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            view()
                                                .gap_1()
                                                .child(text("忽略运行中进程 (ignore_running_processes)").text_sm().font_weight(FontWeight::MEDIUM))
                                                .child(text("若目标应用正在运行，不强制中断更新或卸载流程").text_xs().text_color(move || theme().muted_foreground)),
                                        )
                                        .child(dynamic({
                                            move || {
                                                let is_on = ignore_running.get();
                                                app_button(
                                                    if is_on { "已启用" } else { "未启用" },
                                                    if is_on { ButtonVariant::Primary } else { ButtonVariant::Secondary },
                                                    ButtonSize::Sm,
                                                    move || ignore_running.update(|v| *v = !*v),
                                                )
                                                .into_element()
                                            }
                                        })),
                                ),
                        ),
                ),
        )
}
