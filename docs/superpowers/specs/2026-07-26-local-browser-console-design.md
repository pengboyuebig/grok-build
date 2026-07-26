# 本地浏览器控制台设计

## 目标

在保留 Tauri 桌面端和现有终端产品的前提下，提供可由浏览器访问的本地控制台。它复用当前 React 界面与 ACP 聊天、审批、终端跳转能力，不向局域网或公网暴露服务。

## 范围

- 新增一个 Rust Web 启动入口，监听 `127.0.0.1` 的随机可用端口。
- 提供 React 生产构建资源、命令目录和 WebSocket 聊天事件流。
- 浏览器可创建 ACP 会话、发送消息、响应审批、启动已受控的终端会话。
- 保留现有 Tauri IPC 和桌面端行为，不改变终端产品。
- 不提供文件读写桥接、远程访问、用户账户、多机协作或公网部署。

## 架构

`grok-web` 启动时生成高熵本地访问令牌，绑定回环地址，并在默认浏览器中打开带令牌的本地 URL。HTTP 请求和 WebSocket 握手都必须校验该令牌、`Origin` 以及回环 Host；服务响应不设置跨域许可。

Web 层调用与桌面端相同的会话服务。浏览器前端将现有 bridge 抽象改为可替换传输层：Tauri 环境沿用 `invoke`/event，Web 环境使用同源 HTTP 和 WebSocket。浏览器不能取得任意 shell 参数；终端启动继续使用现有 `LaunchRequest` 验证和显式参数向量。

## 接口

- `GET /health`：只返回服务健康状态，要求本地令牌。
- `GET /api/commands`：返回已白名单化的命令目录。
- `POST /api/sessions`：请求体包含已验证的工作目录，返回 ACP 会话标识。
- `POST /api/sessions/{id}/messages`：发送文本消息；拒绝空值和未知会话。
- `POST /api/approvals/{id}`：只接受显式允许或拒绝；允许仅映射到 ACP 的 `AllowOnce`。
- `POST /api/terminal-sessions`：使用现有受限启动请求启动终端。
- `GET /api/events`：WebSocket，推送 assistant delta、最终消息、审批与错误事件。

## 安全边界

- 仅绑定 `127.0.0.1`；不监听 `0.0.0.0`、`::` 或局域网地址。
- 令牌存于启动 URL 片段并在首次加载后由前端转入内存；不写入 localStorage、cookie 或日志。
- 每个 HTTP 请求使用 `X-Grok-Local-Token`；WebSocket 使用同名子协议令牌，服务器拒绝缺失或不匹配值。
- 仅允许 `Origin: http://127.0.0.1:<port>`；不提供 CORS 头。
- 客户端数据显示为 React 纯文本，不引入 HTML 注入、内联样式或动态脚本执行。

## 错误处理与体验

- 服务启动失败时命令行输出可操作错误并退出非零。
- 认证、会话创建或终端启动失败时返回结构化错误，并在活动面板展示文本错误。
- WebSocket 断开后前端显示连接状态；用户刷新页面后可用当前令牌重新连接。

## 验证

- Rust 单元/集成测试覆盖：回环绑定、令牌拒绝、Origin 拒绝、命令白名单、会话消息和审批映射。
- React 测试覆盖 Web bridge 的令牌携带、WebSocket 事件解析与错误展示。
- Playwright 启动本地测试服务器，验证聊天、审批和终端请求；另验证不带令牌的请求被拒绝。
- 最终运行 Cargo 测试/Clippy/格式/安全扫描、前端测试/构建/E2E。
