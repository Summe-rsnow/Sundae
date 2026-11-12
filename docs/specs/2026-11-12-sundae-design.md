# Sundae (Scoop GUI) Design Document

**Date:** 2026-11-12  
**Status:** Approved  
**Target Architecture:** NSH Pure Rust Mode (`--template rust`)  
**Project Path:** `C:/my/AI/nsh/Sundae`

---

## 1. Intent & The Why

Scoop is an exceptionally clean, non-polluting package manager for Windows (installing CLI tools and GUI applications into user space without UAC popups). However, managing Scoop via CLI lacks:
- A fast visual overview of installed applications and their disk/directory locations.
- Immediate discovery and fuzzy search across multiple buckets (main, extras, nerd-fonts, etc.) without slow CLI round-trips.
- One-click bulk updates and visual diffs of new versions.
- Safe visual operations (hold/unhold versions, view manifest, inspect dependencies).

Existing GUI alternatives are either Electron-based (heavy, slow startup, bloated memory), unmaintained, or incomplete. 
**Sundae** leverages **Native Shell (NSH)** in **pure Rust** to deliver a blazingly fast, native-feeling, sub-second cold-start desktop GUI for Scoop on Windows.

---

## 2. Goals, Non-Goals, and Anti-Goals

### Goals
- **Instant Cold Start & Native Performance**: Launches in <100ms with negligible RAM usage (~20-40MB).
- **Sub-50ms Global Search**: Direct parallel parsing of local bucket manifests via Rust multi-threading (`rayon` + `serde_json`), avoiding sluggish PowerShell/CLI calls for search and browsing.
- **Complete Visual Package Management**:
  - Browse & search across all configured buckets.
  - Inspect application details (version, description, license, homepage, bins).
  - View installed apps, versions, hold status, and local install directories.
  - Check for outdated packages with version diffs and trigger single/bulk upgrades.
  - Manage Buckets (add, remove, update/sync).
  - Real-time task runner drawer with live stdout/stderr streams from `scoop` commands.
- **Design Excellence**: Strict adherence to `nsh-design-guides` and NSH Rust component conventions:
  - Semantic Theme Tokens (light/dark mode, no raw colors in views).
  - Centralized `ui/` layer before composing pages.
## 3. Constraints & Platform Fit

- **Platform**: Windows 10 (1809+) and Windows 11 (x64 / arm64).
- **Scope**: Focused exclusively on user-level Scoop (`%USERPROFILE%\scoop`, `$env:SCOOP`). Machine-wide global installations (`%ProgramData%\scoop`) require administrative elevation (UAC) and are excluded from v1 to preserve zero-elevation safety.
- **Environment**:
  - Scoop root: resolved via `$env:SCOOP`, falling back to `%USERPROFILE%\scoop`.
  - Subdirectories: `buckets/`, `apps/`, `shims/`.
  - PowerShell 5.1+ / pwsh available in environment.
- **Framework Constraints (NSH Rust)**:
  - Component bodies run once; only reactive closures/signals run again.
  - UI composed strictly through app-owned `src/ui/` controls wrapping `nsh_base`.
  - Background work managed via async tasks and NSH signals.

---

## 3. Constraints & Platform Fit

- **Platform**: Windows 10 (1809+) and Windows 11 (x64 / arm64).
- **Environment**:
  - Scoop root typically located at `%USERPROFILE%\scoop` or custom `$env:SCOOP`.
  - Global apps at `%ProgramData%\scoop` or `$env:SCOOP_GLOBAL`.
  - PowerShell available in environment.
- **Framework Constraints (NSH Rust)**:
  - Component bodies run once; only reactive closures/signals run again.
  - UI composed strictly through app-owned `src/ui/` controls wrapping `nsh_base`.
  - Background work managed via async tasks and NSH signals.

---

## 4. Approaches & Architecture Selection

### Evaluated Approaches

1. **Approach A: Pure Rust (`--template rust`) — SELECTED**
   - *Architecture*: Rust binary compiled against `nsh-core`, `nsh-base`, `nsh-platform-windows`.
   - *Pros*: Maximum execution speed, zero JS runtime overhead, single self-contained binary, direct multi-threaded filesystem access for manifest parsing.
   - *Tradeoffs*: UI views written using NSH Rust DSL rather than JSX; changes require recompilation.

2. **Approach B: Mixed (`--template mixed`)**
   - *Architecture*: Rust core crate + TSX frontend running on embedded QuickJS.
   - *Pros*: Fast ~120ms UI hot-reload during development.
   - *Tradeoffs*: QuickJS runtime footprint, FFI serialization overhead between Rust and TSX.

3. **Approach C: Pure TSX (`--template ts`)**
   - *Architecture*: Pure TSX on NSH prebuilt runtime.
   - *Tradeoffs*: Inadequate throughput for parallel scanning tens of thousands of JSON manifest files.

*Decision*: User approved **Approach A (Pure Rust)**.

---

## 5. Architectural Specifications

### 5.1 System Structure

```text
Sundae/
  Cargo.toml               # package nsh-app-sundae, dependencies
  nsh.json                 # name="Sundae", id="dev.nsh.sundae", rust=".", root="App"
  src/
    main.rs                # Desktop backend runner (nsh_platform_windows::run_app)
    lib.rs                 # #[native_module], root App component
    shell.rs               # Window shell: Sidebar navigation + Main content area + Drawer
    store/                 # Reactive application state
      mod.rs
      state.rs             # Core signals: active_route, search_query, active_task
    scoop/                 # Pure domain logic (Decoupled from NSH UI, unit-testable)
      mod.rs
      env.rs               # Scoop environment discovery (root, buckets, apps, shims)
      manifest.rs          # Manifest struct & parser (name, version, description, etc.)
      scanner.rs           # Multi-threaded bucket scanner (rayon + serde_json)
      installed.rs         # Installed packages inspection & hold status
      updates.rs           # Version comparison and updates detection
      process.rs           # Child process runner for scoop commands with line streaming
    ui/                    # Design System (nsh-design-guides compliant)
      mod.rs
      tokens.rs            # Color tokens, spacing, typography scales
      motion.rs            # Transition curves, spring definitions
      icons.rs             # SVG icons (search, package, refresh, database, terminal, etc.)
      button.rs            # Button variants (primary, secondary, ghost, danger) & sizes
      input.rs             # Search and text input with focus ring & clear button
      badge.rs             # Semantic status badge (installed, update available, hold)
      card.rs              # Surface & card containers with subtle borders
      list_row.rs          # Standard data list item with hover wash
      drawer.rs            # Animated bottom drawer for terminal/task output
      dialog.rs            # Confirm alert dialogs (e.g. confirm uninstall)
    views/                 # Feature Views
      mod.rs
      discover.rs          # Search & bucket browse view
      installed.rs         # Installed apps list & management view
      updates.rs           # Outdated apps list & bulk upgrade view
      buckets.rs           # Bucket list, add/remove/sync view
      terminal_drawer.rs   # Real-time console log viewer
```

### 5.2 Core Domain Engine (`src/scoop/`)

- **Environment Detection (`env.rs`)**:
  - Resolves `scoop_path`: Checks `SCOOP` environment variable, falls back to `dirs::home_dir().join("scoop")`.
  - Subdirectories: `buckets/`, `apps/`, `shims/`.
- **Manifest Scanner (`scanner.rs`)**:
  - Traverses all `.json` files in `buckets/<bucket_name>/bucket/*.json`.
  - Leverages `rayon` to parse thousands of JSON manifests concurrently into `PackageManifest` structs.
  - Builds an in-memory index for sub-50ms instant fuzzy/substring search over names and descriptions.
- **Process Executor & Concurrency (`process.rs`)**:
  - **Queue Model (FIFO)**: Only one mutating Scoop command runs at a time (Scoop operations conflict if run concurrently). Subsequent install/update commands are queued.
  - **Execution**: Spawns `scoop <args>` via `std::process::Command` with piped stdout/stderr.
  - **Cancellation**: Supports cancelling the running command by terminating the child process tree.
  - **Streaming**: Emits line events over an async channel to update the UI task log signal.
- **Data Freshness & Indexing (`scanner.rs`)**:
  - **Cold Start**: Window renders interactively in <100ms. Manifest scanning runs on a background thread pool (`rayon`).
  - **Sync Policy**: Offline-first. No network calls on launch. Manifests are read from local git clones. Bucket syncing (`scoop update`) is triggered manually via the UI.

### 5.3 User Interface & Interaction Design

1. **Navigation Shell (`shell.rs`)**:
   - Left Sidebar (220px fixed):
     - App Brand: "Sundae" with version and status dot.
     - Nav Items:
       - 🔍 **Discover (探索)**
       - 📦 **Installed (已安装)** [Badge: total installed count]
       - 🚀 **Updates (更新中心)** [Badge: pending updates count]
       - 🪣 **Buckets (仓库源)**
     - Bottom: Quick link to task drawer toggle with status indicator.
   - Right Main View: Active view component driven by `active_route` signal.
   - Bottom Overlay Drawer: Terminal Drawer (collapsible, min 40px summary bar, max 280px log stream).

2. **Discover View (`views/discover.rs`)**:
   - Top Bar: Search input with instant keyboard focus (`Ctrl+F`), bucket filter dropdown/segmented switch.
   - Body: Virtualized list or grid of packages displaying name, latest version, bucket source, short description, and an "Install" button.
   - Selection: Clicking a package shows manifest details (binaries, homepage, license).

3. **Installed View (`views/installed.rs`)**:
   - Displays all currently installed apps from `$SCOOP/apps`.
   - Actions per row: "Hold/Unhold", "Open Folder" (`explorer.exe`), "Open Manifest", "Uninstall".
   - Search filter within installed apps.

4. **Updates View (`views/updates.rs`)**:
   - Header with "Check for Updates" and "Update All" action buttons.
   - List of outdated packages showing `current_version -> latest_version`.
   - Single-item "Update" button per row.

5. **Buckets View (`views/buckets.rs`)**:
   - Cards/list of added buckets (e.g. `main`, `extras`, `nerd-fonts`).
   - "Add Bucket" dialog with recommended popular buckets + custom git URL input.
   - "Sync / Update All Buckets" button.

6. **Terminal Drawer (`views/terminal_drawer.rs`)**:
   - Displays streaming console output from current running Scoop commands.
   - Monospace font, auto-scroll to bottom, copy logs button, clear button.

---

## 6. Motion Inventory

Per `nsh-design-guides`:
- **Nav Item Hover**: Background wash `theme().surface_hover`, duration 120ms ease-out.
- **Tab/Route Switch**: Subtle fade & vertical slide (offset 4px, 150ms cubic-bezier(0.16, 1, 0.3, 1)).
- **List Item Hover**: Subtle border/background highlight, 100ms ease.
- **Terminal Drawer**: Bottom slide-up transition with spring animation (damping 0.85, duration 220ms).
- **Badge Pulse**: Pulsing dot animation when a background scoop task is actively running.
- **Reduced Motion**: All animations degrade to instant opacity swaps when reduced motion is preferred.

---

## 7. What is Settled as Builder Calls

- **Scope**: User-level Scoop only (`$SCOOP`). No UAC elevation in v1.
- **Manifest Details & Inspect**: Displays `version`, `description`, `homepage`, `license`, `bin`, and dependencies list tags (`depends`, `suggest`).
- **Open Manifest**: Launches the JSON manifest file in the system default text editor.
- **Sync Policy**: Manual on-demand sync (`scoop update`) to guarantee instant offline-friendly launch.
- **Command Concurrency**: Single-flight queue with graceful process cancellation.
- **ANSI Parsing**: Strips or translates ANSI color codes for the terminal log drawer.
- **Window Bounds**: Default size 1024x680, minimum size 840x560.
- **Missing Scoop Handling**: When Scoop is not detected, display an informative empty state alerting the user that Scoop is not installed, showing the official install command and a retry check button.
