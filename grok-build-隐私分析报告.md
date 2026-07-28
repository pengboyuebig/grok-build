# Grok-Build 项目隐私与数据收集分析报告

> 分析日期：2026-07-28
> 项目：grok-build (xAI Grok CLI 工具)
> 代码语言：Rust

---

## 一、项目概述

grok-build 是 xAI 开发的终端 AI 编码代理（CLI/TUI 工具），使用 Rust 编写，包含 76+ 个 crate。支持 macOS、Linux、Windows 三平台。

---

## 二、会收集并上传到远程服务器的数据

### 2.1 workspace_environment.json（自动上传）

**在会话绑定时自动收集并上传到远程 GCS**，路径为 `{session_id}/workspace_environment.json`。

| 收集字段 | 说明 | 代码位置 |
|----------|------|----------|
| `hostname` | 主机的机器名 | `workspace_environment.rs:162`，读取 `$HOSTNAME` 环境变量 |
| `cwd` | 工作目录的完整绝对路径 | `workspace_environment.rs:164`，`cwd.to_string_lossy()` |
| `host_os` | 操作系统类型 | `workspace_environment.rs:198`，`std::env::consts::OS` |
| `host_arch` | CPU 架构 | `workspace_environment.rs:199`，`std::env::consts::ARCH` |
| `repo_root` | Git 仓库根目录完整路径 | `workspace_environment.rs:216`，通过 libgit2 发现 |
| `remote_url` | Git origin 远程仓库 URL | `workspace_environment.rs:224`，**凭证已脱敏剥离** |
| `session_id` | 会话唯一标识 | 传入参数 |
| `user_id` | 用户 ID | 从认证身份获取 |
| `principal_type` | 主体类型 (User/Team) | 从认证身份获取 |
| `principal_id` | 团队 ID | 从认证身份获取 |
| `sandbox_id` | 沙盒 ID | 服务器元数据 |
| `sandbox_profile` | 沙盒配置名称 | `$GROK_SANDBOX_PROFILE` 环境变量 |
| `schema_version` | 架构版本 "v1" | 硬编码 |
| `workspace_version` | crate 版本号 | 硬编码 |
| `recorded_at` | 记录时间 RFC3339 | `Utc::now()` |

**上传流程**：`crates/codegen/xai-grok-workspace/src/upload/handle.rs:2460-2588`

- 触发条件：会话绑定时自动触发
- 禁用开关：设置 `data_collection_disabled` 标志可禁止此上传（`handle.rs:2467`）

**安全措施**：
- Git remote URL 在上传前通过 `strip_url_credentials()` 剥离凭证（如 `https://token:secret@github.com/...` → `https://github.com/...`）

---

### 2.2 LLM API 对话数据

**所有对话内容、代码差异、工具调用结果均通过 HTTPS 发送到 `cli-chat-proxy.grok.com` API。**

| 请求头 | 说明 |
|--------|------|
| `x-grok-conv-id` | 对话 ID |
| `x-grok-req-id` | 请求 ID |
| `x-grok-session-id` | 会话 ID |
| `x-grok-agent-id` | Agent ID |
| `x-grok-user-id` | 用户 ID（可选） |
| `x-grok-deployment-id` | 部署 ID（可选） |
| `x-grok-client-version` | 客户端版本号 |
| `x-grok-client-identifier` | 客户端标识 ("grok-shell") |
| `Authorization` | Bearer 令牌 |
| `User-Agent` | 产品名 + 版本号 |

代码位置：`crates/codegen/xai-grok-sampler/src/client.rs:886-973`

> 注：这些请求属于使用 LLM 服务的必然行为，但你的所有代码和对话内容会经过 xAI 服务器处理。

---

### 2.3 Mixpanel 分析（默认关闭）

**向 `https://api.mixpanel.com` 发送产品遥测数据。**

- **`/track`**：发送 90+ 种事件（如 `prompt_submitted`、`tool_call_completed`、`model_switched` 等）
- **`/engage`**：同步用户画像

| 配置项 | 默认值（公开构建） |
|-------|------------------|
| `events_url` | 空（不发送） |
| `events_api_key` | 空（不发送） |
| `mixpanel_token` | 空（不发送） |
| `mixpanel_enabled` | `false` |

> 根据 README 文档：*"Builds from the public source tree carry no telemetry defaults... nothing is sent unless you supply values here or via env."*
> 即：从公开源码构建的版本，默认**不发送任何遥测数据**到 Mixpanel。

代码位置：`crates/codegen/xai-grok-telemetry/src/config.rs:119-155`

**安全措施**：
- 所有发送到 Mixpanel 的字符串值会先通过 `xai_grok_secrets::redact_json_string_values()` 脱敏处理

---

### 2.4 文件上传

**通过 `cli-chat-proxy` 代理上传文件到远程 GCS。** 上传的 `X-Storage-Path` 是远程 GCS 路径（如 `{session_id}/workspace_environment.json`），不是本地文件路径。

代码位置：`crates/codegen/xai-file-utils/src/storage_client.rs`

---

### 2.5 遥测模式定义

```rust
pub enum TelemetryMode {
    Disabled,       // 什么都不发（企业默认）
    SessionMetrics, // 仅元数据生命周期事件，无内容
    Enabled,        // 完整产品遥测（事件 + Mixpanel）
}
```

代码位置：`crates/codegen/xai-grok-telemetry/src/config.rs:10-55`

---

## 三、不发送数据到外部的模块

### 3.1 自动更新 (`xai-grok-update`)

仅发起**未认证的 HTTP GET 请求**，检查版本号和下载二进制文件，**不附带任何用户数据或本地信息**。

- `GET https://x.ai/cli/{channel}` — 获取版本信息
- `GET https://storage.googleapis.com/grok-build-public-artifacts/cli/{channel}` — GCS 回退
- 下载二进制文件

代码位置：`crates/codegen/xai-grok-update/src/version.rs:274-339`

### 3.2 认证模块 (`xai-grok-auth`)

纯 trait 抽象定义，仅在请求头中注入 `Authorization: Bearer {token}`，**不自行发起额外的网络请求或上传数据**。

### 3.3 本地路径读取

所有 `dirs::home_dir()`、`std::env::current_dir()`、`whoami`、`hostname` 等调用，除了 `workspace_environment.json` 上传以外，均**仅限于本地功能使用**（配置路径解析、Git 发现、UI 显示等），不通过网络发送。

---

## 四、所有远程端点汇总

| 域名 | 用途 | 数据传输方向 |
|------|------|:---:|
| `cli-chat-proxy.grok.com/v1` | LLM API 代理 | 对话内容上传 |
| `api.mixpanel.com/track` | 产品分析 | 事件数据上传（默认关闭） |
| `api.mixpanel.com/engage` | 用户画像 | 用户属性上传（默认关闭） |
| `assets.grok.com` | 资源服务器 | 下载 |
| `code.grok.com/ws/code-agent` | WebSocket relay | 双向 |
| `grok.com/ws/gw/` | WebSocket 网关 | 双向 |
| `api.x.ai/v1` | LLM API（直连） | 对话内容上传 |
| `console.x.ai` | 管理控制台 | 浏览器页面 |
| `accounts.x.ai` | OAuth 账户 | 认证流程 |
| `auth.x.ai` | OIDC 认证 | 认证流程 |
| `x.ai/cli/{channel}` | 版本检查 | 下载（无用户数据） |
| `storage.googleapis.com/...` | 更新二进制下载 | 下载（无用户数据） |
| `computer-hub.grok.com/v1/tools` | 计算机中心工具 | 双向 |

> 无 Google Analytics、Segment 或其他第三方分析工具的痕迹。

---

## 五、隐私风险等级评估

| 风险等级 | 项目 | 说明 |
|:---:|------|------|
| **高** | LLM API 对话数据 | 你的所有代码、对话、文件内容都经过 xAI 服务器 |
| **中** | workspace_environment.json | 主机名、工作目录路径、Git 仓库信息自动上传 |
| **低** | Mixpanel 遥测 | 默认关闭，公开构建不发送 |
| **无** | 自动更新 | 不传输任何用户数据 |

---

## 六、建议的防护措施

1. **禁用遥测**：在 `.grok/config.toml` 中确保以下配置：
   ```toml
   [telemetry]
   events_url = ""
   events_api_key = ""
   mixpanel_token = ""
   mixpanel_enabled = false
   ```

2. **禁用数据收集**：设置 `data_collection_disabled` 标志阻止 `workspace_environment.json` 自动上传。

3. **注意对话内容**：使用 LLM 功能时，所有发给模型的代码和数据都会经过 xAI 服务器处理。敏感业务逻辑建议在本地编写后再提交，或在使用时注意不要将机密数据粘贴到对话中。

4. **监控环境变量**：检查以下环境变量是否被设置（若未设置则遥测功能不会启用）：
   - `GROK_PRODUCTION_CLI_CHAT_PROXY_BASE_URL`
   - `GROK_PRODUCTION_WS_URL`
   - `GROK_PRODUCTION_WS_ORIGIN`
   - `GROK_PRODUCTION_ASSET_SERVER_URL`
