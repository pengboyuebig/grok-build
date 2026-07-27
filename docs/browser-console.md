# Grok 本地浏览器控制台

浏览器控制台与 Tauri 桌面端共用 React 页面和 ACP 会话服务，但只允许本机访问。

## 启动

先构建前端资源：

```powershell
npm --prefix crates\codegen\xai-grok-desktop\ui run build
```

然后启动本地服务：

```powershell
cargo run -p xai-grok-desktop --features web-runtime --bin grok-web
```

命令行会输出 `http://127.0.0.1:<port>#token=<token>`。在同一台机器的浏览器中打开完整地址；URL fragment 中的令牌只在首次加载时保留于页面内存，随后会从地址栏移除。

## 安全限制

- 服务仅监听 `127.0.0.1` 随机端口，不提供局域网或公网访问。
- 所有 HTTP 请求必须携带本地令牌和精确的回环 Origin。
- WebSocket 连接必须在子协议中携带本地令牌。
- 页面不能直接执行 shell 命令；终端跳转由 Rust 后端通过受验证的 `LaunchRequest` 启动。
- ACP 工具的文件与终端请求仍默认拒绝；审批“允许”只会映射为单次允许。

## 故障排除

- 显示“缺少本地访问令牌”：从 `grok-web` 控制台输出重新复制完整 URL。
- 显示“web assets not found”：先执行前端构建命令，或设置 `GROK_WEB_ASSETS` 指向含 `index.html` 的目录。
- 会话创建失败：确认本地 Grok 认证与工作目录有效。
