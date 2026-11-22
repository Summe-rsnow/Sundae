<div align="center">

<img src="assets/logo.svg" alt="Sundae Logo" width="112" />

# Sundae

**专为 Windows Scoop 打造的轻快原生桌面软件包管理器**

在一个清爽克制的原生桌面窗口内完成软件包检索、已安装应用管理、一键更新、Bucket 仓库源配置与实时命令终端监控，无需在命令行窗口间来回切换。  
基于 **Rust + [NSH](https://github.com/longbridge/nsh) + [Bento](https://github.com/Summe-rsnow/Bento) `v1.0.0`** 构建，纯原生渲染，无 JavaScript 运行时开销。

<p>
  <a href="https://github.com/Summe-rsnow/Sundae/releases"><img src="https://img.shields.io/badge/Version-v1.0.0-f43f5e?style=flat-square" alt="Version v1.0.0" /></a>
  <img src="https://img.shields.io/badge/Rust-2024-orange?style=flat-square&logo=rust" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/Platform-Windows-0ea5e9?style=flat-square" alt="Platform Windows" />
  <img src="https://img.shields.io/badge/UI-NSH%20Native-6366f1?style=flat-square" alt="UI NSH Native" />
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-146c2e?style=flat-square" alt="License Apache-2.0" /></a>
</p>

<p>
  <a href="#-核心特性">核心特性</a> ·
  <a href="#-快速开始">快速开始</a> ·
  <a href="#-项目结构">项目结构</a> ·
  <a href="#-家族工具集">家族工具集</a> ·
  <a href="https://github.com/Summe-rsnow/Sundae/releases">下载 Release</a>
</p>

</div>

---

## ✨ 核心特性

- **🔍 探索软件 (Discover)**
  - 后台基于 **Rayon** 多线程并行扫描本地 Bucket 清单索引，结合 `virtual_list` 虚拟化列表零卡顿渲染 **5,000+** 软件包。
  - 支持按名称、命令与描述实时检索，以及横向 **Bucket 筛选芯片** 快速过滤来源。
  - 点击任意软件包即可打开 **Manifest 详情弹窗**，查看开源协议、项目主页、可执行命令（Shims）、前置依赖（Depends）与安装说明（Notes），并支持一键复制 `scoop install` 命令或直接安装 / 重装。
- **📦 已安装管理 (Installed)**
  - 支持关键词即时过滤已安装软件，清晰区分正常已安装、**已锁定版本（Hold / Unhold）** 与安装异常状态。
  - 每行提供版本锁定切换、一键在系统文件管理器打开安装目录、打开本地 `manifest.json`，以及带 `--purge` 持久化数据清理选项的**二次确认卸载弹窗**。
- **⬆️ 更新中心 (Updates)**
  - 自动比对本地已安装版本与 Bucket 最新清单版本，高亮展示版本差值（`当前版本 → 最新版本`）。
  - 支持顶栏一键**同步仓库源**、单个应用即时更新或「**一键全部更新**」。
- **🗄️ Bucket 仓库源 (Buckets)**
  - 直观展示已添加的仓库源及各源收录的软件包数量，支持一键添加常见官方与热门源（`main`、`extras`、`versions`、`nerd-fonts`、`sysinternals`、`java`、`nonportable`、`games`）。
  - 支持填写自定义名称与 Git 仓库地址添加第三方源，并在移除仓库源前弹出**安全确认弹窗**防止误删。
- **⚙️ 选项配置 (Config)**
  - 可视化编辑 `~/.config/scoop/config.json`：支持 HTTP / SOCKS 代理快捷预设（`none` / `default` / `127.0.0.1:7890`）与 GitHub Token、**Aria2** 多线程下载加速（开关及 `aria2-split` 并发数）、Scoop 核心仓库镜像（GitHub / Gitee）与更新分支（`master` / `develop`），以及 SQLite 缓存与忽略运行中进程等高级开关。
  - 支持即时生效的 **Sundae 窗口与托盘关闭行为**（`每次询问` / `最小化到托盘` / `直接退出应用`）。
- **🖥️ 实时终端抽屉与原生桌面体验**
  - 底部集成可折叠虚拟化**任务执行终端抽屉**，实时流式输出 Scoop 命令日志（区分标准输出与错误着色），支持任务队列、停止当前任务与清空日志。
  - 内置桌面快捷键（`Ctrl+R` 刷新本地索引、`Ctrl+J` 切换终端抽屉、`Esc` 关闭当前顶层弹窗）、跟随系统 / 浅色 / 深色三模式外观切换、Windows 系统托盘常驻以及 **GitHub Releases** 新版本检测。

---

## 🚀 快速开始

### 方式一：下载预编译安装包

前往 [GitHub Releases](https://github.com/Summe-rsnow/Sundae/releases) 下载最新版本的 Windows 安装包（`.exe`）或便携压缩包（`.zip`），解压 / 安装即可使用。

### 方式二：从源码运行与构建

#### 环境要求

- [Rust](https://www.rust-lang.org/tools/install)（Edition 2024）
- [NSH CLI](https://nsh.dev/docs/installation)（`0.6.1+`）

#### 运行开发版

```bash
# 克隆仓库
git clone https://github.com/Summe-rsnow/Sundae.git
cd Sundae

# 以本机后端运行（推荐，会完整加载 nsh.json 与 theme.json 主题配置）
nsh run
```

> **提示**：直接使用 `cargo run` 也可以启动，但不会读取 `nsh.json` 和 `theme.json`，建议始终使用 `nsh run` 以获得完整的原生窗口与主题效果。

#### 检查、测试与打包

```bash
# 代码、类型与构建检查
nsh check

# 运行单元测试、Headless UI 交互测试与 TypeScript 端到端测试
nsh test

# 打包 Windows 发行版产物（输出至 .nsh/out/）
nsh package --target windows
```

---

## 📁 项目结构

```text
Sundae/
├── .github/workflows/
│   └── release.yml         # Tag 推送自动构建并发布 GitHub Release 工作流
├── assets/                 # 应用图标、Logo、品牌标与 64×64 托盘图标
├── nsh.json                # 应用元信息、版本号（1.0.0）、窗口配置与主题声明
├── theme.json              # 浅色 / 深色语义色 Token（与 Bento 家族规范对齐）
├── DESIGN.md               # 设计规范、布局架构、动效清单与无障碍约定
├── src/
│   ├── lib.rs              # 根组件 App（#[native_module]）与 APP_META 常量
│   ├── main.rs             # 桌面二进制入口
│   ├── shell.rs            # 桌面壳：48px 隐藏标题栏、212px 侧边栏、托盘与快捷键
│   ├── dialogs.rs          # 关于/更新弹窗、Package Detail 详情、卸载与移除源确认弹窗
│   ├── store/
│   │   ├── mod.rs          # 全局响应式状态（零 pub Signal）、过滤查询与弹窗状态
│   │   └── actions.rs      # Rayon 后台扫描、任务队列轮询、配置保存与更新检测动作
│   ├── scoop/
│   │   ├── mod.rs          # Scoop 核心领域模块导出
│   │   ├── env.rs          # Scoop 根目录环境检测
│   │   ├── config.rs       # Scoop config.json 与 Sundae 桌面偏好持久化读写
│   │   ├── manifest.rs     # Scoop Manifest JSON 解析与命令/依赖提取
│   │   ├── scanner.rs      # Rayon 多线程 Bucket 清单并行扫描与搜索索引
│   │   ├── installed.rs    # 已安装应用扫描、Hold/Unhold 切换与跨平台目录/文件打开
│   │   ├── updates.rs      # 本地版本与 Bucket 最新清单版本差值比对
│   │   └── process.rs      # Scoop 子进程任务队列与流式 stdout/stderr 日志收集
│   ├── views/
│   │   ├── mod.rs          # 五大主视图导出
│   │   ├── discover.rs     # 探索软件视图（搜索、Bucket 筛选与 virtual_list）
│   │   ├── installed.rs    # 已安装应用管理视图（过滤、锁定、打开目录与卸载）
│   │   ├── updates.rs      # 更新中心视图（差值对比、单个/全部更新）
│   │   ├── buckets.rs      # Bucket 仓库源管理视图（已添加、热门推荐与自定义源）
│   │   └── config.rs       # 选项配置视图（代理、Aria2、镜像源、高级开关与关闭行为）
│   └── ui/
│       ├── mod.rs          # 统一导出 bento::ui 组件与终端抽屉
│       ├── icons.rs        # 复用 bento::ui::icons，托管 Sundae 品牌标与托盘图标
│       └── terminal_drawer.rs # 底部可折叠虚拟化实时命令终端抽屉
├── tests/
│   ├── screen.rs           # Rust Headless 端到端 UI 交互测试
│   ├── scoop_config_test.rs    # ScoopConfig / SundaePrefs 读写集成测试
│   ├── scoop_installed_test.rs # 已安装应用扫描与版本锁定集成测试
│   ├── scoop_manifest_test.rs  # Manifest 解析与容错集成测试
│   ├── scoop_process_test.rs   # ScoopOp 参数构造与任务队列测试
│   └── scoop_scanner_test.rs   # Rayon 并发扫描与索引检索测试
└── App.test.tsx            # TypeScript 端到端测试入口
```

---

## 🍱 家族工具集

| 工具 | 当前版本 | 仓库地址 | 定位与职责 |
|---|---|---|---|
| **Bento** | `v1.0.0` | [Summe-rsnow/Bento](https://github.com/Summe-rsnow/Bento) | 共享原生 UI 组件库、统一 24×24 图标库与桌面壳基础设施 |
| **Wonton** | `v1.2.1` | [Summe-rsnow/Wonton](https://github.com/Summe-rsnow/Wonton) | 专为 Antigravity 打造的进程级独立代理与多账号额度管理工具 |
| **Sundae** | `v1.0.0` | [Summe-rsnow/Sundae](https://github.com/Summe-rsnow/Sundae) | 专为 Windows Scoop 打造的轻快原生桌面软件包管理器 |

---

## ⚠️ 注意事项

1. **Scoop 环境依赖**：Sundae 依赖本地已安装的 Scoop（默认读取 `SCOOP` 环境变量或 `~/scoop`），软件安装、更新、卸载与仓库源操作均通过调用本机 `scoop` 命令执行。
2. **配置文件兼容**：保存选项配置时直接读写 `~/.config/scoop/config.json`，并保留原本由命令行管理的未知字段（如 `last_update`、`root_path`），不会覆盖已有配置。
3. **跨平台说明**：在 macOS / Linux 上可正常编译并启动界面（未检测到本地 Scoop 环境时自动进入引导视图），实际 Scoop 包管理操作面向 Windows 桌面环境。

---

## 🤝 参与贡献

欢迎提交 Issue 或 Pull Request 来改进 Sundae！  
提交前请确保 `nsh check` 与 `nsh test`（或 `cargo test`）均通过且无编译警告。

## 📄 开源协议

本项目基于 [Apache License 2.0](LICENSE) 开源。
