# Sundae Design Document

## 1. Product Purpose & Targets

**Sundae** is a native desktop GUI package manager for [Scoop](https://scoop.sh/), built on **Native Shell (NSH 0.6.1)** and the shared **`Bento`** UI/desktop toolkit (`bento`).

- **Primary target**: Windows desktop (`minSize: 880x600`, native hidden 48px title bar, system tray, keyboard-first desktop pointer controls).
- **Secondary compatibility**: macOS and Linux desktop builds compile cleanly (with Scoop-absent empty/guidance state when `~/scoop` is not present).
- **Visual family**: Shares the `Bento` / `Wonton` desktop family design language — calm, restrained, high-contrast Light & Dark semantic tokens (`theme.json`), 0.5rem base radius, crisp hairlines, and virtualized lists for zero-jank browsing of 5,000+ Scoop manifests.

---

## 2. Design System & Semantic Tokens

All colors in presentation code are consumed strictly via `nsh_base::theme()` (and `bento::ui` helpers). Zero raw `rgb(0x...)` or hex literals exist in `src/`.

| Role | Light (`theme.json`) | Dark (`theme.json`) | Usage |
|---|---|---|---|
| `background` / `foreground` | `#f8fafd` / `#1f1f1f` | `#0e1117` / `#e3e3e3` | Window root surface & primary copy |
| `sidebar` / `sidebar-foreground` | `#f0f4f9` / `#1f1f1f` | `#111620` / `#e3e3e3` | Left navigation rail & title bar chrome |
| `card` / `card-foreground` | `#ffffff` / `#1f1f1f` | `#151b26` / `#e3e3e3` | Package rows, bucket cards, config sections |
| `popover` / `popover-foreground` | `#ffffff` / `#1f1f1f` | `#1b2230` / `#e3e3e3` | Modal dialogs, toast, terminal surface |
| `primary` / `primary-foreground` | `#1a73e8` / `#ffffff` | `#8ab4f8` / `#062e6f` | Primary CTA (Install, Update All, Save), active filter chip |
| `secondary` / `secondary-foreground` | `#e8eef7` / `#1f1f1f` | `#232d3f` / `#e3e3e3` | Secondary actions, version badges |
| `muted` / `muted-foreground` | `#f0f4f9` / `#444746` | `#1b2230` / `#9aa8be` | Subtle containers, secondary metadata, paths |
| `accent` / `accent-foreground` | `#d3e3fd` / `#041e49` | `#25344d` / `#d3e3fd` | Active sidebar item, hover washes |
| `destructive` / `destructive-foreground` | `#b3261e` / `#ffffff` | `#f28b82` / `#0e1117` | Uninstall, Remove Bucket, Cancel Task, stderr logs |
| `success` | `#146c2e` | `#6dd58c` | Installed status, latest version delta, success toast |
| `warning` | `#b06000` | `#fdd663` | Held status, pending updates badge, running task |
| `border` / `input` / `ring` | `#dbe3ef` / `#c4cfe0` / `#1a73e8` | `#2c374b` / `#36445c` / `#8ab4f8` | Hairlines, input frames, keyboard focus-visible rings |
| `scrim` | `#041e4970` | `#000000cc` | Modal dialog backdrop |

### Typography & Spacing Scale
- **Hierarchy**: Page title (`text_lg` Semibold), section/card title (`text_sm` Semibold), body/controls (`text_sm` / `text_xs`), metadata & versions (`text_xs`, `tabular_nums`, monospace for CLI commands and terminal logs).
- **Spacing**: 4pt grid (`gap_1`=4pt, `gap_2`=8pt, `gap_3`=12pt, `gap_4`=16pt, `p_5`=20pt).
- **Radii**: `theme().radius` (`8px` / `0.5rem`) on controls, `radius + 4px` (`12px`) on cards & modals, `999px` only on intentional filter pills and status dots.

---

## 3. Window & Screen Architecture

```text
+-----------------------------------------------------------------------------+
| [Sundae SVG] Sundae [v1.0.0]   <Installed · Updates · Buckets>   [R][T][S/L/D] [_][O][X] |  48px Hidden Title Bar
+-------------------+---------------------------------------------------------+
| Sidebar (212px)   | Active Workspace View (flex_1)                          |
|  - Discover       |   - Discover: Search + Bucket chips + virtual_list      |
|  - Installed  [n] |   - Installed: Filter + virtual_list (Hold/Open/Uninst) |
|  - Updates    [n] |   - Updates: Check/Update All + virtual_list            |
|  - Buckets    [n] |   - Buckets: Added + Recommended + Custom Git form      |
|  - Config         |   - Config: Proxy, Aria2, Repo/Branch, Advanced, App    |
|                   +---------------------------------------------------------+
| [Task Status]     | Collapsible Terminal Drawer (240px, virtual_list logs)  |
+-------------------+---------------------------------------------------------+
  Overlays: Package Detail Dialog | Confirm Uninstall (--purge) | Confirm Remove Bucket
            Confirm Close Dialog  | About / Update Dialog       | Floating Notice Toast
```

### Key Architectural Invariants
1. **Store (`src/store.rs`)**: Module-level private signals only (zero `pub Signal`). All Scoop scanning (`BucketScanner::scan_buckets`, `InstalledManager::scan_installed`, `UpdatesManager::check_updates`) runs on a background thread via `rayon` and commits atomically back to the UI thread — cold start paints in <50ms.
2. **Virtualized Lists**: `discover.rs`, `installed.rs`, `updates.rs`, and `terminal_drawer.rs` use `virtual_list(count, |at| ...)` with zero per-row local signals and zero `dynamic` loops rebuilding list cards.
3. **Destructive Guardrails**: Uninstall opens `confirm_uninstall_dialog` (with optional `--purge` checkbox). Bucket removal opens `confirm_remove_bucket_dialog`. Window close respects `CloseBehavior` (`Ask` / `MinimizeToTray` / `Quit`).
4. **Cross-Platform Directory Opening**: Uses `std::process::Command` (`explorer` / `open` / `xdg-open`), never `nsh_core::open_url` (which rejects Windows drive-letter paths in NSH 0.6.1).

---

## 4. Motion Tokens & Motion Inventory

### Motion Tokens
- `instant`: `120ms` (`ease-out`) — hover background washes, border tints
- `fast`: `160ms` (`cubic-bezier(0.22, 1, 0.36, 1)` soft settle) — button/chip press scale (`0.97`, icon buttons `0.95`), toggle transitions
- `base`: `220ms` (`ease-in-out`) — sidebar selection state, status badge transitions, drawer reveal
- `slow`: `260ms` (`ease-out`) — modal dialog entrance (rise `8pt` + opacity), toast entrance
- **Reduced Motion**: Checked via `reduce_motion()`. When active, all scale/slide transitions and entrance animations collapse to instant state changes; no continuous pulse runs.

### Motion Inventory (All Screens)

| Surface / Screen | State Change | Tool & Tokens | Reduced Motion | Verified |
|---|---|---|---|---|
| **Global Controls (`bento::ui`)** | Button & filter chip hover / press | Native `.hover()` wash (`instant`, ease-out) + `.active(\|s\| s.scale(0.97))` (`fast`, soft settle) | Hover wash instant; press scale disabled (`1.0`) | [ ] |
| **Global Controls (`bento::ui`)** | Icon action & theme segmented button press | Native `.hover()` wash (`instant`) + `.active(\|s\| s.scale(0.95))` (`fast`) | Instant color change, no scale | [ ] |
| **Global Controls (`bento::ui`)** | Keyboard focus ring | Native `.focus_visible()` ring (`theme().ring`) outside border | Unchanged (always visible for accessibility) | [ ] |
| **Title Bar (`shell.rs`)** | Refresh / Terminal drawer toggle active state | `.style_when()` accent wash + transition (`instant`) | Instant swap | [ ] |
| **Title Bar (`shell.rs`)** | Theme mode segmented switch (System / Light / Dark) | `bento::shell::theme_mode_segmented` selected background transition (`base`, ease-in-out) | Instant swap | [ ] |
| **Sidebar (`shell.rs`)** | Active navigation route change | `bento::ui::sidebar_nav_item` selected fill + accent foreground transition (`base`, ease-in-out) | Instant swap | [ ] |
| **Sidebar (`shell.rs`)** | Dynamic badge counts (Installed / Updates / Buckets) | Fine-grained text/visibility binding (no layout jump; tabular nums) — intentionally no bounce on frequent data updates | Instant update | [ ] |
| **Discover / Installed / Updates** | Virtual list row hover & press | Native `.hover()` row tint (`instant`) + `.active()` deepened wash (rows never scale) | Instant wash | [ ] |
| **Discover / Installed / Updates** | Virtual list cell recycling & search filtering | Intentionally **no row enter animation** on search/scroll/recycle so 5,000+ manifest filtering stays 0-jank | N/A (always instant) | [ ] |
| **Terminal Drawer (`terminal_drawer.rs`)** | Drawer open / close | Conditional mount (`when`) with clean border separation; log stream virtualized with zero per-line animation | Instant show/hide | [ ] |
| **Dialogs (`dialogs.rs`, `bento`)** | Package Detail, Confirm Uninstall, Confirm Remove Bucket, Confirm Close, About | `bento::ui::modal_dialog` scrim fade + dialog surface entrance (`slow`, ease-out; exit `fast`) | Instant appear/dismiss | [ ] |
| **Notice Toast (`shell.rs`, `bento`)** | Toast appear / auto-dismiss (3s / 6s) | `bento::ui::notice_toast` bottom-edge floating reveal (`base`, ease-out) | Instant appear/dismiss | [ ] |
| **Loading / Indexing** | Background Rayon scan / running Scoop task | Status text + warning badge in title bar & sidebar footer; no blocking modal or spinning CPU loop | Static status label | [ ] |
| **Drag / Swipe / Live Shader** | None on Sundae screens | N/A (Sundae has no drag-reorder lists or continuous shaders) | N/A | [ ] |

---

## 5. Accessibility & Keyboard Contract

- Every interactive control uses semantic `bento::ui` wrappers built on `nsh_base` (`button()`, `input()`, `checkbox()`, `switch()`, `link()`).
- Every icon-only button (`Refresh`, `Terminal`, `Theme System/Light/Dark`, `Clear Search`, `Close Dialog`) carries an explicit `accessibility_label`.
- Global desktop shortcuts via `on_key`:
  - `mod+r`: Refresh local Scoop index & installed apps
  - `mod+j`: Toggle bottom terminal drawer
  - `escape`: Close topmost open dialog (Detail / Uninstall / Remove Bucket / About / Close Confirm)
- Status is never conveyed by color alone: every status badge pairs semantic tone with explicit Chinese text (`已安装`, `已锁定`, `安装异常`, `待更新`, `执行中`).
