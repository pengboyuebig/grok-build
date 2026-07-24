# Grok Desktop Console Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Windows `grok-desktop.exe` AI coding client with conversational UI, graphical slash-command controls, and one-click terminal TUI handoff while retaining `xai-grok-pager.exe` unchanged.

**Architecture:** Keep `xai-grok-pager-bin` as the terminal composition root. Add `xai-grok-desktop`, a Tauri 2 Rust binary with a React/TypeScript webview. Its Rust bridge validates all IPC, maps slash/ACP command metadata into typed UI menus, connects to the existing agent/leader protocol, and starts the sibling terminal EXE without invoking a shell.

**Tech Stack:** Rust 2024, Tauri 2, Tokio, serde, existing `xai-grok-shell` and ACP types, React 18, TypeScript, Vite, Tailwind, Vitest, Playwright, Cargo and Tauri Windows bundling.

---

## Scope and boundaries

- Preserve `cargo build --release -p xai-grok-pager-bin` and existing TUI behavior.
- Add `cargo build --release -p xai-grok-desktop` for `grok-desktop.exe`.
- Desktop supports chat, streaming text/tool activity, explicit approval, files/diffs, terminal output, sessions, model/effort/permission controls, command menus, workspace selection, and TUI launch.
- Known commands have typed controls. Unknown ACP and skill commands are prompt-dispatched, never executed as OS commands.
- Do not implement terminal emulation, visual Git history, or every experimental command in v1.

## File map

| Path | Responsibility |
| --- | --- |
| `Cargo.toml` | Add desktop workspace member. |
| `crates/codegen/xai-grok-desktop/Cargo.toml` | Desktop package and dependencies. |
| `crates/codegen/xai-grok-desktop/src/lib.rs` | App state and Tauri registration. |
| `crates/codegen/xai-grok-desktop/src/domain/*.rs` | Pure command, chat, approval and launch models. |
| `crates/codegen/xai-grok-desktop/src/services/*.rs` | Agent adapter, typed catalog and terminal launcher. |
| `crates/codegen/xai-grok-desktop/src/commands/*.rs` | Validated Tauri IPC handlers. |
| `crates/codegen/xai-grok-desktop/ui/` | React application and tests. |
| `scripts/build-windows.ps1` | terminal/desktop/both build entry points. |
| `docs/desktop-console.md` | Build and operating guide. |

## Task 1: Create the separate desktop package

**Files:** modify `Cargo.toml`; create `crates/codegen/xai-grok-desktop/{Cargo.toml,src/lib.rs,src/main.rs,tests/package_contract.rs}`.

- [ ] Write this failing test first:

```rust
#[test]
fn has_the_expected_desktop_identity() {
    assert_eq!(xai_grok_desktop::PRODUCT_NAME, "Grok Desktop");
    assert_eq!(xai_grok_desktop::BINARY_NAME, "grok-desktop");
}
```

- [ ] Run `cargo test -p xai-grok-desktop --test package_contract`; expect Cargo to report the package is missing.
- [ ] Add the workspace member and package. `main.rs` calls `xai_grok_desktop::run()`; `lib.rs` publishes the two constants. Do not change `xai-grok-pager-bin`.
- [ ] Run `cargo test -p xai-grok-desktop --test package_contract` and `cargo check -p xai-grok-desktop`; expect pass.
- [ ] Commit: `git commit -m "[桌面端] 新增图形客户端包骨架"`.

## Task 2: Map slash commands to safe menu definitions

**Files:** create `src/domain/command_catalog.rs`; create `tests/command_catalog.rs`.

- [ ] Write failing tests:

```rust
#[test]
fn rename_becomes_a_form() {
    let item = DesktopCommand::from_slash("/rename", true, true);
    assert_eq!(item.kind, CommandKind::Form);
}

#[test]
fn quit_requires_confirmation() {
    assert!(DesktopCommand::from_slash("/quit", false, false).requires_confirmation);
}

#[test]
fn dynamic_commands_never_spawn_processes() {
    let item = DesktopCommand::from_slash("/plugin-action", true, false);
    assert_eq!(item.kind, CommandKind::PromptDispatch);
    assert!(!item.can_spawn_process);
}
```

- [ ] Run `cargo test -p xai-grok-desktop --test command_catalog`; expect compilation failure.
- [ ] Define serializable `DesktopCommand`, `CommandKind`, `CommandArgument`, and `CommandCategory`. Explicitly confirm `/quit`, `/new`, `/clear`, `/rewind`, `/logout`, memory clear, and worktree removal. Dynamic commands always use `PromptDispatch`.
- [ ] Rerun the same command; expect pass.
- [ ] Commit: `git commit -m "[桌面端] 增加安全命令菜单模型"`.

## Task 3: Implement a shell-free terminal handoff

**Files:** create `src/domain/terminal_launch.rs`, `src/services/terminal_launcher.rs`, `tests/terminal_launcher.rs`.

- [ ] Write failing tests:

```rust
#[test]
fn creates_argument_vector_not_shell_text() {
    let spec = build_launch_spec(LaunchRequest::new("C:/work/demo", Some("grok-build"), Some("high"), PermissionMode::Ask)).unwrap();
    assert_eq!(spec.args, vec!["--cwd", "C:/work/demo", "--model", "grok-build", "--reasoning-effort", "high"]);
}

#[test]
fn invalid_directory_is_rejected() {
    assert!(build_launch_spec(LaunchRequest::new("Z:/missing", None, None, PermissionMode::Ask)).is_err());
}
```

- [ ] Run `cargo test -p xai-grok-desktop --test terminal_launcher`; expect failure.
- [ ] Canonicalize workspace paths; use `std::process::Command::new` and `.args`, never `cmd.exe`, PowerShell, or a concatenated string. Accept only `Ask`, `Auto`, and `AlwaysApprove`; resolve sibling `xai-grok-pager.exe` first.
- [ ] Rerun test; expect pass.
- [ ] Commit: `git commit -m "[桌面端] 支持安全启动终端会话"`.

## Task 4: Bridge the existing agent protocol

**Files:** create `src/domain/chat.rs`, `src/services/agent_client.rs`, `src/commands/chat.rs`, `tests/agent_event_mapping.rs`.

- [ ] Write failing tests:

```rust
#[test]
fn maps_text_delta_to_frontend_event() {
    let event = map_agent_event(TestAgentEvent::TextDelta("hello".into()));
    assert_eq!(event.kind, FrontendEventKind::AssistantDelta);
}

#[test]
fn approval_starts_pending() {
    let event = map_agent_event(TestAgentEvent::ApprovalRequested { id: "a1".into() });
    assert_eq!(event.kind, FrontendEventKind::ApprovalRequested);
    assert!(!event.approved);
}
```

- [ ] Run `cargo test -p xai-grok-desktop --test agent_event_mapping`; expect failure.
- [ ] Define one adapter trait with a production implementation over existing shell/leader/ACP interfaces and an in-memory test implementation. Emit serializable events only: assistant delta/final, tool start/end, file change, terminal output, approval, error, session change. Approval stays pending until an IPC response.
- [ ] Run test and `cargo check -p xai-grok-desktop`; expect pass.
- [ ] Commit: `git commit -m "[桌面端] 接入代理对话和工具事件桥接"`.

## Task 5: Add a narrow IPC boundary

**Files:** create `src/commands/{mod.rs,catalog.rs,terminal.rs}`, `tests/ipc_contract.rs`.

- [ ] Write failing tests:

```rust
#[test]
fn ipc_schema_excludes_raw_process_inputs() {
    let schema = desktop_invoke_schema();
    assert!(!schema.contains("rawArgs"));
    assert!(!schema.contains("program"));
}
```

- [ ] Run `cargo test -p xai-grok-desktop --test ipc_contract`; expect failure.
- [ ] Expose only `get_command_catalog`, `start_session`, `send_message`, `respond_to_approval`, `launch_terminal_session`, `list_sessions`, `open_workspace_dialog`, and `set_preference`. Validate every request before service use.
- [ ] Rerun test; expect pass.
- [ ] Commit: `git commit -m "[桌面端] 定义受限的图形界面调用接口"`.

## Task 6: Build the secure React foundation

**Files:** create `ui/{package.json,vite.config.ts,src/main.tsx,src/lib/bridge.ts,src/lib/bridge.test.ts}`.

- [ ] Write the failing test:

```ts
it('rejects a command without a slash name', () => {
  expect(() => validateCatalog({ commands: [{ name: 'model' }] })).toThrow();
});
```

- [ ] Run `npm test -- --run src/lib/bridge.test.ts` from `ui`; expect module-not-found failure.
- [ ] Configure React, TypeScript, Tailwind, Vitest, Testing Library, Tauri API and Zod. Validate bridge data; render all external data as React text. Do not use `dangerouslySetInnerHTML`, inline style, dynamic URL, `eval`, or `new Function`.
- [ ] Run `npm test -- --run src/lib/bridge.test.ts` then `npm run build`; expect pass.
- [ ] Commit: `git commit -m "[桌面端] 建立安全的图形界面工程"`.

## Task 7: Implement the code-chat workspace

**Files:** create `ui/src/features/chat/{ChatWorkspace.tsx,ChatWorkspace.test.tsx,useChatSession.ts,types.ts}`.

- [ ] Write failing test:

```tsx
it('renders streamed assistant text after sending', async () => {
  render(<ChatWorkspace sessionId="s1" />);
  await userEvent.type(screen.getByLabelText('消息输入'), '修复测试');
  await userEvent.click(screen.getByRole('button', { name: '发送' }));
  emitDesktopEvent('chat:event', { kind: 'assistant_delta', text: '我来处理。' });
  expect(await screen.findByText('我来处理。')).toBeInTheDocument();
});
```

- [ ] Run `npm test -- --run src/features/chat/ChatWorkspace.test.tsx`; expect module-not-found failure.
- [ ] Implement accessible Chinese-labelled message composer, text-only messages, event streaming, stop control and recoverable error state using Tailwind class names only.
- [ ] Rerun test; expect pass without React warnings.
- [ ] Commit: `git commit -m "[桌面端] 实现代码对话工作区"`.

## Task 8: Add menus, forms, approvals and activity

**Files:** create `ui/src/features/commands/{CommandMenu.tsx,CommandMenu.test.tsx}`, `ui/src/features/approvals/{ApprovalDialog.tsx,ApprovalDialog.test.tsx}`, `ui/src/features/activity/ActivityPanel.tsx`.

- [ ] Write failing tests:

```tsx
it('opens a parameter form for rename', async () => {
  render(<CommandMenu commands={[renameCommand]} />);
  await userEvent.click(screen.getByRole('button', { name: '/rename' }));
  expect(screen.getByLabelText('会话名称')).toBeInTheDocument();
});

it('requires a click before approving', async () => {
  render(<ApprovalDialog request={approvalRequest} />);
  expect(invoke).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole('button', { name: '允许' }));
  expect(invoke).toHaveBeenCalledWith('respond_to_approval', { id: 'a1', approved: true });
});
```

- [ ] Run tests; expect module-not-found failure.
- [ ] Implement categorized menu, typed forms, destructive-action confirmation, and activity rows for tools/diffs/terminal/errors. Do not provide arbitrary executable or shell-argument fields.
- [ ] Rerun tests; expect pass.
- [ ] Commit: `git commit -m "[桌面端] 增加命令菜单和权限控制"`.

## Task 9: Assemble sessions, workspace, controls and handoff

**Files:** create `ui/src/{App.tsx,App.test.tsx}`, `ui/src/features/workspace/WorkspacePicker.tsx`, `ui/src/features/sessions/SessionList.tsx`.

- [ ] Write failing test:

```tsx
it('hands selected context to the terminal launcher', async () => {
  render(<App />);
  await userEvent.click(screen.getByRole('button', { name: '打开终端会话' }));
  expect(invoke).toHaveBeenCalledWith('launch_terminal_session', {
    request: expect.objectContaining({ cwd: 'C:/work/demo', model: 'grok-build' }),
  });
});
```

- [ ] Run `npm test -- --run src/App.test.tsx`; expect module-not-found failure.
- [ ] Compose the three-pane application: sessions/menu, chat, activity. Add folder picker, model/effort/permission selection, non-secret preferences, and one-click terminal handoff.
- [ ] Run test and `npm run build`; expect pass.
- [ ] Commit: `git commit -m "[桌面端] 组装会话控制台和终端跳转"`.

## Task 10: Package both Windows products

**Files:** create `crates/codegen/xai-grok-desktop/tauri.conf.json`, `scripts/build-windows.ps1`, `docs/desktop-console.md`, `tests/build_target_contract.rs`.

- [ ] Write failing test:

```rust
#[test]
fn script_supports_all_product_targets() {
    let script = include_str!("../../../scripts/build-windows.ps1");
    assert!(script.contains("ValidateSet('terminal', 'desktop', 'both')"));
}
```

- [ ] Run `cargo test -p xai-grok-desktop --test build_target_contract`; expect failure.
- [ ] Make the script accept exactly `terminal`, `desktop`, and `both`, mapping to the pager Cargo build, desktop Tauri build, or both. Bundle the terminal EXE beside the desktop EXE. Document output location and Windows code-signing requirements.
- [ ] Rerun test; expect pass.
- [ ] Commit: `git commit -m "[桌面端] 配置Windows双入口打包"`.

## Task 11: End-to-end and security verification

**Files:** create `ui/e2e/desktop.spec.ts`, `docs/desktop-console-verification.md`.

- [ ] Write a failing Playwright test that sends a chat prompt, receives a tool approval, explicitly approves it, and launches the terminal session.
- [ ] Run `npm run test:e2e -- --project desktop`; expect harness failure before it is implemented.
- [ ] Add a test-only in-memory agent adapter and terminal launcher. Add CI scan that fails on `dangerouslySetInnerHTML`, `innerHTML`, `outerHTML`, `document.write`, `eval(`, `new Function`, `style={{`, and shell-spawn patterns.
- [ ] Run:

```powershell
cargo fmt --all --check
cargo test -p xai-grok-desktop
cargo clippy -p xai-grok-desktop -- -D warnings
Push-Location crates/codegen/xai-grok-desktop/ui
npm test -- --run
npm run build
npm run test:e2e -- --project desktop
Pop-Location
./scripts/build-windows.ps1 both
```

- [ ] Record tool versions, commands, artifacts and results in `docs/desktop-console-verification.md`; commit with `[桌面端] 完成图形客户端端到端验证`.

## Acceptance criteria

- [ ] Terminal TUI stays independently buildable.
- [ ] Desktop product builds to Windows EXE/installer.
- [ ] Desktop supports coding chat, tool events, explicit permissions, sessions, model controls, menu actions and validated TUI launch.
- [ ] No user input becomes a shell command; no untrusted string becomes HTML, style or URL.
- [ ] Rust, UI, E2E, security scan, lint, formatting and package verification results are documented.
