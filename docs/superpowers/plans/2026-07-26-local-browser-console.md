# 本地浏览器控制台 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增只绑定本机回环地址的浏览器控制台，同时保留 Tauri 桌面端和终端产品。

**Architecture:** `grok-web` 二进制使用 Axum 提供本地 HTTP/WebSocket 服务；共享的会话服务管理 ACP 会话、审批与活动事件。React bridge 根据运行环境选择 Tauri IPC 或同源 HTTP/WebSocket，不改变页面组件接口。

**Tech Stack:** Rust、Tokio、Axum、Serde、React、Vite、Vitest、Playwright。

---

### Task 1: 本地访问鉴权与 HTTP 路由骨架

**Files:**
- Modify: `crates/codegen/xai-grok-desktop/Cargo.toml`
- Create: `crates/codegen/xai-grok-desktop/src/web/auth.rs`
- Create: `crates/codegen/xai-grok-desktop/src/web/mod.rs`
- Create: `crates/codegen/xai-grok-desktop/tests/web_auth.rs`

- [ ] **Step 1: 写入失败测试，要求仅回环地址、令牌和同源 Origin 才能访问。**

```rust
#[test]
fn rejects_missing_token_and_non_loopback_origin() {
    let auth = LocalAuth::new_for_test("test-token", 43123);
    assert!(!auth.authorizes(None, Some("http://127.0.0.1:43123")));
    assert!(!auth.authorizes(Some("test-token"), Some("http://localhost:43123")));
}
```

- [ ] **Step 2: 运行失败测试。**

Run: `cargo test -p xai-grok-desktop --test web_auth`

Expected: FAIL，因为 `LocalAuth` 尚不存在。

- [ ] **Step 3: 实现 `LocalAuth`、随机 32 字节令牌和仅允许 `127.0.0.1:<port>` 的 Origin 校验。**

```rust
pub struct LocalAuth { token: String, origin: String }

impl LocalAuth {
    pub fn authorizes(&self, token: Option<&str>, origin: Option<&str>) -> bool {
        token == Some(self.token.as_str()) && origin == Some(self.origin.as_str())
    }
}
```

- [ ] **Step 4: 运行测试确认通过。**

Run: `cargo test -p xai-grok-desktop --test web_auth`

Expected: PASS。

- [ ] **Step 5: 提交。**

```powershell
git add crates/codegen/xai-grok-desktop/Cargo.toml crates/codegen/xai-grok-desktop/src/web crates/codegen/xai-grok-desktop/tests/web_auth.rs
git commit -m "[浏览器端] 增加本地访问鉴权"
```

### Task 2: 共享会话服务与事件广播

**Files:**
- Create: `crates/codegen/xai-grok-desktop/src/services/session_service.rs`
- Modify: `crates/codegen/xai-grok-desktop/src/services/mod.rs`
- Modify: `crates/codegen/xai-grok-desktop/src/commands/chat.rs`
- Test: `crates/codegen/xai-grok-desktop/tests/approval_policy.rs`

- [ ] **Step 1: 写入失败测试，确认审批允许仅选择 `AllowOnce` 且取消保持 fail-closed。**

```rust
#[test]
fn approval_without_allow_once_is_cancelled() {
    assert_eq!(approval_decision(true, false, true), ApprovalDecision::Cancel);
}
```

- [ ] **Step 2: 运行测试确认现有逻辑的测试覆盖有效。**

Run: `cargo test -p xai-grok-desktop --test approval_policy`

Expected: PASS。

- [ ] **Step 3: 提取 `SessionService`，暴露 `start_session`、`send_message`、`respond_to_approval`、`subscribe_events`，并让 Tauri command 只负责参数转换。**

```rust
pub struct SessionService { agent: Mutex<Option<Arc<LiveAgent>>>, sessions: Mutex<Vec<String>>, events: broadcast::Sender<FrontendEvent> }
```

- [ ] **Step 4: 运行桌面端回归测试。**

Run: `cargo test -p xai-grok-desktop`

Expected: PASS。

- [ ] **Step 5: 提交。**

```powershell
git add crates/codegen/xai-grok-desktop/src/services crates/codegen/xai-grok-desktop/src/commands/chat.rs crates/codegen/xai-grok-desktop/tests
git commit -m "[浏览器端] 提取共享会话服务"
```

### Task 3: Web API 与 WebSocket 事件流

**Files:**
- Modify: `crates/codegen/xai-grok-desktop/src/web/mod.rs`
- Create: `crates/codegen/xai-grok-desktop/src/web/routes.rs`
- Create: `crates/codegen/xai-grok-desktop/tests/web_routes.rs`

- [ ] **Step 1: 写入失败路由测试，验证无令牌请求返回 401，非法 Origin 返回 403，命令目录返回受限命令。**

```rust
let response = app.oneshot(Request::get("/api/commands").body(Body::empty())?).await?;
assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
```

- [ ] **Step 2: 运行路由测试确认失败。**

Run: `cargo test -p xai-grok-desktop --test web_routes`

Expected: FAIL，因为路由尚未注册。

- [ ] **Step 3: 实现 `/health`、命令目录、创建会话、发消息、审批、终端启动和 `/api/events` WebSocket。所有变更请求验证令牌、Origin 和 JSON 请求体。**

```rust
Router::new()
  .route("/health", get(health))
  .route("/api/commands", get(command_catalog))
  .route("/api/sessions", post(start_session))
  .route("/api/events", get(events))
```

- [ ] **Step 4: 运行路由测试。**

Run: `cargo test -p xai-grok-desktop --test web_routes`

Expected: PASS。

- [ ] **Step 5: 提交。**

```powershell
git add crates/codegen/xai-grok-desktop/src/web crates/codegen/xai-grok-desktop/tests/web_routes.rs
git commit -m "[浏览器端] 增加本地Web接口"
```

### Task 4: Web 启动入口与静态 UI 托管

**Files:**
- Modify: `crates/codegen/xai-grok-desktop/Cargo.toml`
- Create: `crates/codegen/xai-grok-desktop/src/bin/grok-web.rs`
- Modify: `crates/codegen/xai-grok-desktop/src/web/mod.rs`
- Create: `crates/codegen/xai-grok-desktop/tests/web_server.rs`

- [ ] **Step 1: 写入失败测试，要求监听地址始终为 `127.0.0.1:0`，并返回含 token 片段的启动 URL。**

```rust
assert_eq!(server.address().ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
assert!(server.url().fragment().unwrap().contains("token="));
```

- [ ] **Step 2: 运行测试确认失败。**

Run: `cargo test -p xai-grok-desktop --test web_server`

Expected: FAIL，因为 `LocalWebServer` 尚不存在。

- [ ] **Step 3: 实现服务监听、静态文件回退和 `grok-web` CLI；只将 URL 输出到控制台，默认不自动打开浏览器。**

```rust
let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
println!("Open {}", server.url());
axum::serve(listener, router).await?;
```

- [ ] **Step 4: 运行服务测试。**

Run: `cargo test -p xai-grok-desktop --test web_server`

Expected: PASS。

- [ ] **Step 5: 提交。**

```powershell
git add crates/codegen/xai-grok-desktop/Cargo.toml crates/codegen/xai-grok-desktop/src/bin/grok-web.rs crates/codegen/xai-grok-desktop/src/web crates/codegen/xai-grok-desktop/tests/web_server.rs
git commit -m "[浏览器端] 增加本地Web启动入口"
```

### Task 5: React Web bridge 与环境切换

**Files:**
- Modify: `crates/codegen/xai-grok-desktop/ui/src/lib/bridge.ts`
- Create: `crates/codegen/xai-grok-desktop/ui/src/lib/webBridge.ts`
- Create: `crates/codegen/xai-grok-desktop/ui/src/lib/webBridge.test.ts`
- Modify: `crates/codegen/xai-grok-desktop/ui/src/main.tsx`

- [ ] **Step 1: 写入失败测试，验证 Web bridge 从 URL hash 读取一次令牌并在 HTTP 请求添加 `X-Grok-Local-Token`。**

```ts
expect(fetch).toHaveBeenCalledWith('/api/commands', expect.objectContaining({ headers: { 'X-Grok-Local-Token': 'test-token' } }));
```

- [ ] **Step 2: 运行测试确认失败。**

Run: `npm test -- --run src/lib/webBridge.test.ts`

Expected: FAIL，因为 Web bridge 尚不存在。

- [ ] **Step 3: 实现环境检测、HTTP 操作与 WebSocket 事件解析；用 `history.replaceState` 清除 hash，不使用 localStorage/cookie。**

```ts
const token = new URLSearchParams(window.location.hash.slice(1)).get('token');
window.history.replaceState(null, '', window.location.pathname);
```

- [ ] **Step 4: 运行 Web bridge 测试。**

Run: `npm test -- --run src/lib/webBridge.test.ts`

Expected: PASS。

- [ ] **Step 5: 提交。**

```powershell
git add crates/codegen/xai-grok-desktop/ui/src/lib crates/codegen/xai-grok-desktop/ui/src/main.tsx
git commit -m "[浏览器端] 接入Web传输层"
```

### Task 6: 端到端验证与文档

**Files:**
- Modify: `crates/codegen/xai-grok-desktop/ui/e2e/desktop.spec.ts`
- Create: `crates/codegen/xai-grok-desktop/ui/e2e/web.spec.ts`
- Modify: `docs/desktop-console.md`
- Create: `docs/browser-console.md`

- [ ] **Step 1: 写入浏览器 E2E，用测试服务器验证令牌缺失返回 401、WebSocket 审批事件及终端请求。**

```ts
await expect(page.getByText('需要你的确认')).toBeVisible();
await expect.poll(() => requests).toContain('/api/terminal-sessions');
```

- [ ] **Step 2: 运行 E2E 确认失败。**

Run: `npm run test:e2e -- --project web`

Expected: FAIL，因为 web 项目尚未配置。

- [ ] **Step 3: 增加 Playwright web 项目，补充启动、访问、鉴权和故障排除文档。**

- [ ] **Step 4: 全量验证。**

Run:

```powershell
cargo test -p xai-grok-desktop
cargo check -p xai-grok-desktop --features tauri-runtime
cargo clippy -p xai-grok-desktop -- -D warnings
cargo fmt -p xai-grok-desktop --check
.\scripts\security-scan.ps1
npm --prefix crates\codegen\xai-grok-desktop\ui test -- --run
npm --prefix crates\codegen\xai-grok-desktop\ui run build
npm --prefix crates\codegen\xai-grok-desktop\ui run test:e2e
```

Expected: all commands exit 0.

- [ ] **Step 5: 提交。**

```powershell
git add crates/codegen/xai-grok-desktop/ui/e2e docs
git commit -m "[浏览器端] 完善Web控制台验证"
```
