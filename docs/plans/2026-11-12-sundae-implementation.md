# Sundae (Scoop GUI) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build Sundae, a high-performance, native-feeling Windows desktop GUI for Scoop written in pure Rust with Native Shell (NSH).

**Architecture:** A pure Rust NSH desktop application (`--template rust`) utilizing a clean two-tier architecture: a headless, unit-tested domain engine (`src/scoop/`) for instant parallel JSON manifest scanning and background process management, driving a reactive NSH presentation layer built strictly from a centralized design system (`src/ui/`) adhering to `nsh-design-guides`.

**Tech Stack:** Rust 2021, NSH (`nsh-core`, `nsh-base`, `nsh-platform-windows`), `serde`, `serde_json`, `rayon`, `dirs`, `parking_lot`.

**Spec:** `C:/my/AI/nsh/Sundae/docs/specs/2026-11-12-sundae-design.md`

## Global Constraints

- **Language & Mode**: Pure Rust only; no JavaScript VM, no TSX at runtime.
- **Root Component**: `App` declared via `#[component]` and registered in `#[native_module("sundae/native")]`.
- **Component Lifecycle**: Component bodies run once; reactive UI uses closure bindings `move || ...`.
- **UI Foundations**: All visual controls belong to `src/ui/` wrapping `nsh_base`; views never introduce raw hex colors or ad-hoc borders.
- **Theme Tokens**: All colors derived from semantic `theme()` tokens supporting Light and Dark modes.
- **Scoop Scope**: User-space Scoop only (`%USERPROFILE%\scoop`, `$env:SCOOP`). No administrative UAC prompts.
- **Process Safety**: Mutating operations (`install`, `update`, `uninstall`, `bucket add/rm`) run sequentially through a FIFO task queue.
- **Offline-First**: Zero network access on cold launch; manifest indexing scans local bucket git clones in background.

## Review Focus

1. **Missing or Corrupted Scoop Root**: If Scoop is not installed or `$SCOOP` directory is missing, app must display an informative banner with the official install command and a retry button without crashing.
2. **Malformed Manifest JSON**: If a third-party bucket contains malformed JSON files, the scanner must skip the file, log a warning, and continue parsing remaining packages.
3. **Command Cancellation**: When a user cancels a running Scoop task from the drawer, the child process tree must be cleanly terminated without leaving orphaned processes.
4. **App with No Binaries/No Description**: Manifests with missing optional fields (`description`, `homepage`, `bin`, `license`) must render safe default fallbacks in the UI.
5. **Rapid Keystroke Search**: Typing rapidly in the search input must filter in-memory indices smoothly without memory leaks or race conditions.

---

### Task 1: Scaffolding Project & NSH Rust Baseline

**Files:**
- Create: `C:/my/AI/nsh/Sundae/Cargo.toml`
- Create: `C:/my/AI/nsh/Sundae/nsh.json`
- Create: `C:/my/AI/nsh/Sundae/src/main.rs`
- Create: `C:/my/AI/nsh/Sundae/src/lib.rs`
- Test: `tests/smoke.rs`

**Interfaces:**
- Produces: `nsh_app_sundae::App` root component and entry points.

- [ ] **Step 1: Write initial `Cargo.toml` and `nsh.json`**
Configure package `nsh-app-sundae`, dependencies (`nsh-core = "0.5.0"`, `nsh-base = "0.5.0"`, `nsh-platform-windows = "0.5.0"`, `serde`, `serde_json`, `rayon`, `dirs`, `log`, `env_logger`). Configure `nsh.json` with `"name": "Sundae"`, `"rust": "."`, `"root": "App"`.

- [ ] **Step 2: Create `src/lib.rs` with `#[native_module]` and `App` stub**
Implement root `App` returning a basic `view().flex_1().bg(move || theme().background)` displaying "Sundae".

- [ ] **Step 3: Create `src/main.rs` for Windows desktop backend**
Set up `nsh_platform_windows::run_app` with window title "Sundae" and size (1024, 680).

- [ ] **Step 4: Verify build with `nsh check`**
Run `nsh check` in `C:/my/AI/nsh/Sundae` and verify zero errors.

- [ ] **Step 5: Commit**
`git init && git add . && git commit -m "chore: scaffold Sundae NSH Rust project"`

---

### Task 2: Domain Engine - Environment & Manifest Model

**Files:**
- Create: `src/scoop/mod.rs`
- Create: `src/scoop/env.rs`
- Create: `src/scoop/manifest.rs`
- Test: `tests/scoop_manifest_test.rs`

**Interfaces:**
- Produces: `ScoopEnv::detect() -> ScoopEnv`, `PackageManifest::from_json(name: &str, bucket: &str, content: &str) -> Result<PackageManifest>`

- [ ] **Step 1: Write failing unit test for `PackageManifest::from_json`**
Test parsing JSON with full fields and missing optional fields (`description`, `homepage`, `license`, `bin`, `suggest`, `depends`).

- [ ] **Step 2: Run test to verify it fails**
Run: `cargo test --test scoop_manifest_test` -> Expected FAIL (module not defined).

- [ ] **Step 3: Implement `ScoopEnv` in `src/scoop/env.rs`**
Implement detection logic resolving root directory (`$env:SCOOP` or `%USERPROFILE%\scoop`), `buckets/`, `apps/`, `shims/`, and `is_installed() -> bool`.

- [ ] **Step 4: Implement `PackageManifest` in `src/scoop/manifest.rs`**
Define serde struct handling string/array variations for `bin`, `license`, and `suggest`/`depends`.

- [ ] **Step 5: Run tests and verify PASS**
Run: `cargo test --test scoop_manifest_test` -> Expected: PASS.

- [ ] **Step 6: Commit**
`git add src/scoop/ tests/ && git commit -m "feat(scoop): add environment discovery and manifest parser"`

---

### Task 3: Domain Engine - Fast Bucket Scanner & Search Index

**Files:**
- Create: `src/scoop/scanner.rs`
- Modify: `src/scoop/mod.rs`
- Test: `tests/scoop_scanner_test.rs`

**Interfaces:**
- Consumes: `PackageManifest`, `ScoopEnv`
- Produces: `BucketScanner::scan_all(env: &ScoopEnv) -> SearchIndex`, `SearchIndex::search(&self, query: &str, bucket_filter: Option<&str>) -> Vec<&PackageManifest>`

- [ ] **Step 1: Write failing unit test for `SearchIndex`**
Test querying across multiple simulated packages by substring/fuzzy name and description matching.

- [ ] **Step 2: Run test to verify failure**
Run: `cargo test --test scoop_scanner_test` -> Expected FAIL.

- [ ] **Step 3: Implement `BucketScanner` and `SearchIndex`**
Use `rayon::prelude::*` to read all `buckets/<bucket>/bucket/*.json` in parallel, skipping unreadable or malformed files gracefully. Store manifests in an in-memory `SearchIndex`.

- [ ] **Step 4: Run tests and verify PASS**
Run: `cargo test --test scoop_scanner_test` -> Expected: PASS.

- [ ] **Step 5: Commit**
`git add src/scoop/scanner.rs tests/scoop_scanner_test.rs && git commit -m "feat(scoop): implement parallel bucket scanner and search index"`

---

### Task 4: Domain Engine - Installed Apps & Updates Detection

**Files:**
- Create: `src/scoop/installed.rs`
- Create: `src/scoop/updates.rs`
- Modify: `src/scoop/mod.rs`
- Test: `tests/scoop_installed_test.rs`

**Interfaces:**
- Consumes: `ScoopEnv`, `PackageManifest`, `SearchIndex`
- Produces: `InstalledManager::scan_installed(env: &ScoopEnv) -> Vec<InstalledApp>`, `UpdatesManager::check_updates(installed: &[InstalledApp], index: &SearchIndex) -> Vec<AppUpdate>`

- [ ] **Step 1: Write failing test for installed app detection and update comparison**
Test inspecting simulated `$SCOOP/apps/<app>/current/manifest.json`, detecting hold status, and finding version diffs against `SearchIndex`.

- [ ] **Step 2: Run test to verify failure**
Run: `cargo test --test scoop_installed_test` -> Expected FAIL.

- [ ] **Step 3: Implement `InstalledManager` and `UpdatesManager`**
Read installed apps from `$SCOOP/apps/`, extract current version, inspect hold marker. Compare version strings to detect outdated packages.

- [ ] **Step 4: Run tests and verify PASS**
Run: `cargo test --test scoop_installed_test` -> Expected: PASS.

- [ ] **Step 5: Commit**
`git add src/scoop/installed.rs src/scoop/updates.rs tests/ && git commit -m "feat(scoop): add installed apps scanning and update detection"`

---

### Task 5: Domain Engine - Process Execution & Task Queue

**Files:**
- Create: `src/scoop/process.rs`
- Modify: `src/scoop/mod.rs`
- Test: `tests/scoop_process_test.rs`

**Interfaces:**
- Produces: `TaskQueue::new()`, `TaskQueue::enqueue(&self, op: ScoopOp)`, `ScoopOp` (Install, Uninstall, Update, BucketAdd, BucketRm), line event receiver `Receiver<LogLine>`.

- [ ] **Step 1: Write failing test for `TaskQueue`**
Test queuing multiple commands, verifying sequential FIFO execution, capturing line outputs, and cancelling a running process.

- [ ] **Step 2: Run test to verify failure**
Run: `cargo test --test scoop_process_test` -> Expected FAIL.

- [ ] **Step 3: Implement `process.rs`**
Create a background worker thread with a FIFO channel. Execute `powershell -NoProfile -Command scoop <args>`. Stream stdout/stderr line-by-line over a channel. Support graceful child kill on cancel signal.

- [ ] **Step 4: Run tests and verify PASS**
Run: `cargo test --test scoop_process_test` -> Expected: PASS.

- [ ] **Step 5: Commit**
`git add src/scoop/process.rs tests/ && git commit -m "feat(scoop): implement FIFO task queue and process executor"`

---

### Task 6: NSH Design System & Core UI Controls (`src/ui/`)

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/tokens.rs`
- Create: `src/ui/motion.rs`
- Create: `src/ui/icons.rs`
- Create: `src/ui/button.rs`
- Create: `src/ui/input.rs`
- Create: `src/ui/badge.rs`
- Create: `src/ui/card.rs`
- Create: `src/ui/list_row.rs`

**Interfaces:**
- Produces: Reusable UI widgets (`ui::button`, `ui::search_input`, `ui::badge`, `ui::card`, `ui::list_row`, `ui::icon`).

- [ ] **Step 1: Implement `tokens.rs` and `motion.rs`**
Define semantic token helpers and motion tokens (durations, easings, spring configs) according to `nsh-design-guides`.

- [ ] **Step 2: Implement SVG `icons.rs`**
Add inline SVG paths for `search`, `package`, `refresh`, `bucket`, `terminal`, `chevron`, `check`, `x`, `folder`, `lock`.

- [ ] **Step 3: Implement `button.rs` and `input.rs`**
Wrap `nsh_base::button` with variants (`Primary`, `Secondary`, `Ghost`, `Danger`) and sizes. Wrap text input with search icon and focus ring.

- [ ] **Step 4: Implement `badge.rs`, `card.rs`, and `list_row.rs`**
Construct semantic status badges (success, warning, neutral), card containers, and list row widgets with hover feedback.

- [ ] **Step 5: Verify compilation with `nsh check`**
Run: `nsh check` -> Expected: PASS.

- [ ] **Step 6: Commit**
`git add src/ui/ && git commit -m "feat(ui): create centralized design system and core controls"`

---

### Task 7: Reactive State & Navigation Shell

**Files:**
- Create: `src/store/mod.rs`
- Create: `src/store/state.rs`
- Create: `src/shell.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: `AppState` global signals (`active_route`, `search_query`, `installed_list`, `updates_list`, `buckets_list`, `running_task`, `task_logs`), `shell::Shell` root layout.

- [ ] **Step 1: Implement `store/state.rs`**
Define `Route` enum (`Discover`, `Installed`, `Updates`, `Buckets`). Set up NSH signals for active route, search text, lists, and task queue status.

- [ ] **Step 2: Implement `src/shell.rs`**
Create the 2-column desktop window layout: 220px Left Sidebar with brand logo, nav buttons, and dynamic badge counts; Main content viewport; Bottom toggleable task drawer bar.

- [ ] **Step 3: Implement missing Scoop empty state banner**
If `ScoopEnv::detect().is_installed()` is false, display an informative card with install command and "Retry Detection" button.

- [ ] **Step 4: Connect Shell in `src/lib.rs`**
Mount `Shell` inside `App`. Run `nsh check` to verify types.

- [ ] **Step 5: Commit**
`git add src/store/ src/shell.rs src/lib.rs && git commit -m "feat(shell): implement desktop navigation shell and global state"`

---

### Task 8: Discover & Search View

**Files:**
- Create: `src/views/mod.rs`
- Create: `src/views/discover.rs`
- Modify: `src/shell.rs`

**Interfaces:**
- Consumes: `AppState`, `SearchIndex`, `ui::*`
- Produces: `views::discover_view() -> impl IntoElement`

- [ ] **Step 1: Implement `views/discover.rs` layout**
Build top search header (search input, bucket filter chips) and package grid/list.

- [ ] **Step 2: Implement package card with action buttons**
Display package name, version, bucket badge, short description, and "Install" button.

- [ ] **Step 3: Connect install action to `TaskQueue`**
Clicking "Install" enqueues `ScoopOp::Install(name)` and opens the terminal drawer.

- [ ] **Step 4: Verify with `nsh check`**
Run: `nsh check` -> Expected: PASS.

- [ ] **Step 5: Commit**
`git add src/views/ && git commit -m "feat(views): implement package discover and search view"`

---

### Task 9: Installed & Updates Views

**Files:**
- Create: `src/views/installed.rs`
- Create: `src/views/updates.rs`
- Modify: `src/views/mod.rs`

**Interfaces:**
- Consumes: `AppState`, `InstalledManager`, `UpdatesManager`, `ui::*`
- Produces: `views::installed_view()`, `views::updates_view()`

- [ ] **Step 1: Implement `views/installed.rs`**
Render installed packages list. Actions per row: "Hold/Unhold", "Open Folder" (`explorer.exe`), "Open Manifest" (default text editor), "Uninstall" (with confirm dialog).

- [ ] **Step 2: Implement `views/updates.rs`**
Render outdated packages showing `current -> latest` version diff badges. Include "Check for Updates" and "Update All" header buttons.

- [ ] **Step 3: Connect update actions to `TaskQueue`**
Wire single update and bulk update triggers to queue operations and stream logs.

- [ ] **Step 4: Verify with `nsh check`**
Run: `nsh check` -> Expected: PASS.

- [ ] **Step 5: Commit**
`git add src/views/installed.rs src/views/updates.rs && git commit -m "feat(views): implement installed apps manager and updates center"`

---

### Task 10: Buckets View & Live Terminal Drawer

**Files:**
- Create: `src/views/buckets.rs`
- Create: `src/views/terminal_drawer.rs`
- Modify: `src/views/mod.rs`
- Modify: `src/shell.rs`

**Interfaces:**
- Consumes: `AppState`, `TaskQueue`, `ui::*`
- Produces: `views::buckets_view()`, `views::terminal_drawer()`

- [ ] **Step 1: Implement `views/buckets.rs`**
List configured buckets (`main`, `extras`, etc.) and popular community buckets. Provide "Add Bucket" dialog and "Sync All Buckets" button.

- [ ] **Step 2: Implement `views/terminal_drawer.rs`**
Create expandable bottom drawer showing real-time log lines, auto-scroll to bottom, copy logs button, clear logs, and a "Cancel Running Task" button.

- [ ] **Step 3: Connect drawer state to shell**
Add smooth slide-up transition using `ui::motion` spring tokens.

- [ ] **Step 4: Verify with `nsh check`**
Run: `nsh check` -> Expected: PASS.

- [ ] **Step 5: Commit**
`git add src/views/buckets.rs src/views/terminal_drawer.rs && git commit -m "feat(views): implement buckets view and real-time terminal drawer"`

---

### Task 11: End-to-End Verification & Desktop Smoke Test

**Files:**
- Test: `tests/app_smoke.rs`

- [ ] **Step 1: Write headless test covering app startup and navigation**
Verify `App` mounts cleanly, initial state loads, and navigation between routes works without panics.

- [ ] **Step 2: Run all unit and integration tests**
Run: `cargo test` -> Expected: all tests PASS.

- [ ] **Step 3: Run `nsh check` across entire crate**
Run: `nsh check` -> Expected: 0 errors, 0 warnings.

- [ ] **Step 4: Launch and smoke test native Windows app**
Run `cargo run` (or `nsh run`), verify window opens, UI renders properly, and sidebar navigation functions smoothly.

- [ ] **Step 5: Final commit**
`git add . && git commit -m "feat: complete Sundae Scoop GUI v1"`
