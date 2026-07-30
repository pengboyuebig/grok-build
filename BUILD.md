# Grok Build 构建与运行指南

本文档说明如何在 Windows 上从源码构建并运行 `grok`（即 `xai-grok-pager`）。

## 前置环境

### 1. Rust 工具链

项目使用 `rust-toolchain.toml` 中指定的 Rust 版本（当前为 `1.92.0`）。安装 [`rustup`](https://rustup.rs/) 后，首次运行 `cargo` 会自动下载对应版本。

```powershell
cargo --version
rustc --version
```

### 2. protoc（协议缓冲区编译器）

构建 `xai-grok-tools-api` 等 crate 时需要 `protoc`。本项目仓库中的 `bin/protoc` 是一个 DotSlash 脚本，**仅支持 macOS 和 Linux**，Windows 上需要单独安装 Windows 版 `protoc`。

#### Windows 安装方式

最简单的方式是通过 `cargo` 安装已打包的 protoc 二进制：

```powershell
cargo install protoc-bin-vendored --locked
```

安装后确认路径：

```powershell
protoc-bin-which
```

典型输出：

```text
C:\Users\<用户名>\.cargo\registry\src\index.crates.io-...\protoc-bin-vendored-win32-3.2.0\bin\protoc.exe
```

构建前需要设置 `PROTOC` 环境变量指向该路径。

> **提示**：为了不用每次手动设置，建议把 `PROTOC` 添加到系统环境变量中。

## 构建

### Debug 构建

适合开发调试，产物较大（约 800+ MB），启动时可能因 Windows 默认栈空间不足而崩溃。

```powershell
$env:PROTOC="C:\Users\<用户名>\.cargo\registry\src\index.crates.io-...\protoc-bin-vendored-win32-3.2.0\bin\protoc.exe"
cargo build -p xai-grok-pager-bin
```

产物：`target\debug\xai-grok-pager.exe`

### Release 构建（推荐）

Release 构建经过优化，产物更小（约 260 MB），运行更快，通常不会出现栈溢出问题。

```powershell
$env:PROTOC="C:\Users\<用户名>\.cargo\registry\src\index.crates.io-...\protoc-bin-vendored-win32-3.2.0\bin\protoc.exe"
cargo build -p xai-grok-pager-bin --release
```

产物：`target\release\xai-grok-pager.exe`

## 运行

### 直接运行

```powershell
target\release\xai-grok-pager.exe
```

### 如果遇到栈溢出

Windows 线程默认栈空间为 1 MB，Debug 构建容易触发栈溢出。若出现：

```text
thread 'main' has overflowed its stack
```

可临时增大栈空间：

```powershell
$env:RUST_MIN_STACK="8388608"
target\release\xai-grok-pager.exe
```

或更彻底地改用 Release 构建。

## 常见问题

### 1. 为什么 Debug 版有 800+ MB，而 Release 只有 260 MB？

- Debug 构建无优化，包含完整调试符号；
- Release 构建开启优化、死代码消除，体积小很多；
- 你之前使用的约 229 MB 的包就是 Release 构建产物。

### 2. `bin/protoc` 在 Windows 上无法执行

错误示例：

```text
bin/protoc found at `..\..\..\bin/protoc` but failed to execute: %1 不是有效的 Win32 应用程序。 (os error 193)
```

这是因为 `bin/protoc` 是 DotSlash 包装脚本，不支持 Windows。按照上文安装 `protoc-bin-vendored` 并设置 `PROTOC` 环境变量即可。

### 3. 如何设置永久环境变量

在 PowerShell 中以用户级别设置：

```powershell
[System.Environment]::SetEnvironmentVariable(
    "PROTOC",
    "C:\Users\<用户名>\.cargo\registry\src\index.crates.io-...\protoc-bin-vendored-win32-3.2.0\bin\protoc.exe",
    "User"
)
```

设置后重新打开 PowerShell 即可生效。

## 参考

- 项目 README：`README.md`
- Rust 工具链配置：`rust-toolchain.toml`
- 工作区配置：`Cargo.toml`
