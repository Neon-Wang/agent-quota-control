# Status Bar Cleanup and ChatGPT 7897 Launcher Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the generic tool launcher, add a dedicated ChatGPT desktop launcher pinned to proxy port 7897, and make status-bar icon, percentage, and state text independently configurable.

**Architecture:** `AppConfig` owns one global `StatusBarDisplayConfig`; `tray.rs` renders icons/titles from it and delegates process creation to a focused `chatgpt_launcher.rs`. The generic harness/launcher surface is removed from Rust state, commands, frontend navigation, and persisted configuration.

**Tech Stack:** Rust, Tauri 2, React, TypeScript, Vitest, Serde.

---

### Task 1: Configuration schema and migration

**Files:**
- Modify: `src-tauri/src/types.rs`
- Modify: `src-tauri/src/config.rs`
- Modify: `frontend/src/types.ts`

- [ ] Add failing Rust tests proving version-4 JSON migrates to version 5, preserves monitoring/proxy state, defaults all three display switches to true, and drops `selectedTools` after serialization.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml config::tests -- --nocapture` and verify the new tests fail because `status_bar_display` does not exist and version remains 4.
- [ ] Add `StatusBarDisplayConfig { show_icon, show_percentage, show_state_text }`, default all fields to true, remove `selected_tools` from `AppConfig`, and migrate versions below 5.
- [ ] Mirror the new camelCase type in `frontend/src/types.ts` and remove `ToolInfo`/`tools`/`selectedTools` types.
- [ ] Re-run the targeted Rust tests and verify they pass.

### Task 2: Dedicated ChatGPT 7897 launcher

**Files:**
- Create: `src-tauri/src/chatgpt_launcher.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/chatgpt_launcher.rs`

- [ ] Add failing unit tests for the fixed executable path, proxy environment, Chromium arguments, missing-app error, and already-running error using injected path/process checks.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml chatgpt_launcher::tests -- --nocapture` and verify failure before implementation.
- [ ] Implement a focused launch specification and `launch_chatgpt_with_7897()` using `std::process::Command` without invoking a shell.
- [ ] Expose `is_chatgpt_installed()` and `is_chatgpt_running()` for menu-state rendering.
- [ ] Re-run targeted tests and verify happy/error paths pass.

### Task 3: Tray menu and display combinations

**Files:**
- Modify: `src-tauri/src/tray.rs`
- Test: `src-tauri/src/tray.rs`

- [ ] Add failing tests for all title combinations: percentage plus state, percentage only, state only, and neither; also test icon visibility selection.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml tray::tests -- --nocapture` and verify the new expectations fail.
- [ ] Change tray construction/update to set icon and title independently from `status_bar_display`.
- [ ] Add a dedicated menu item whose label/enabled state reflects ChatGPT install/running state and dispatches only `launch_chatgpt_with_7897()`.
- [ ] Remove the selected-tools submenu and all `tool:` / `cli_tool:` menu event branches.
- [ ] Re-run tray tests and verify all combinations pass.

### Task 4: Retire the generic tool subsystem

**Files:**
- Delete: `src-tauri/src/harness.rs`
- Delete: `src-tauri/src/launcher.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/types.rs`
- Delete: `frontend/src/components/LaunchPanel.tsx`
- Delete: `frontend/src/components/ToolList.tsx`
- Modify: `frontend/src/App.tsx`
- Modify: `frontend/src/api.ts`
- Modify: `frontend/src/i18n.ts`
- Modify: `frontend/src/styles.css`

- [ ] Remove `set_selected_tools`, `launch_tool`, tool scanning, `DashboardState.tools`, tool-related Tauri command registration, the Tools navigation route, tool components, and now-unused styles.
- [ ] Run `rg -n "selected_tools|selectedTools|launch_tool|ToolInfo|已选择工具|工具选择|cli_tool:|tool:" frontend/src src-tauri/src` and require zero product-code matches.
- [ ] Run Rust check and frontend typecheck to catch dangling imports and types.

### Task 5: Frontend status-bar controls

**Files:**
- Modify: `frontend/src/components/MonitoringSettings.tsx`
- Modify: `frontend/src/api.ts`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `frontend/src/__tests__/App.test.tsx`

- [ ] Add failing frontend tests for the absence of the Tools navigation and for independently toggling icon, percentage, and state text through `set_status_bar_display`.
- [ ] Add a failing Rust command/config test proving display updates persist and rebuild dashboard state.
- [ ] Implement `set_status_bar_display`, frontend API binding, and an accessible “状态栏样式” panel with three labeled checkboxes.
- [ ] Run targeted frontend and Rust tests until green.

### Task 6: Documentation and full verification

**Files:**
- Modify: `README.md`

- [ ] Update the canonical usage section to describe the dedicated ChatGPT 7897 menu action and status-bar appearance switches; remove generic tool-launcher claims.
- [ ] Run `cargo fmt --manifest-path src-tauri/Cargo.toml --check`.
- [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- [ ] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`.
- [ ] Run `pnpm --dir frontend typecheck`, `pnpm --dir frontend test`, and `pnpm --dir frontend build`.
- [ ] Run a production Tauri app build without installing it, then inspect the built menu and settings UI locally.
- [ ] Run `git diff --check`, a no-emoji scan, and a final requirement-by-requirement audit.

Git commit and deployment steps are intentionally omitted until separately authorized.
