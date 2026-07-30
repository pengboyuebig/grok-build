# Grok Build 发布流程文档

## 项目结构概览

```
grok-build-main/
├── crates/codegen/xai-grok-pager-bin/   # CLI/TUI 入口 (发布为 grok)
├── grok-desktop/                        # Tauri 桌面应用
│   ├── src/                             # React 前端
│   └── src-tauri/                       # Tauri 后端
└── scripts/                             # 构建/发布脚本
```

---

## 一、本地开发启动

### 1.1 CLI/TUI 开发模式

```bash
# 进入工作区根目录
cd D:\Program Files (x86)\grok-build-main\grok-build-main

# 开发模式运行（热重载需配合 cargo-watch）
cargo run -p xai-grok-pager-bin

# 仅构建检查（快速验证）
cargo check -p xai-grok-pager-bin

# 运行测试
cargo test -p xai-grok-pager-bin
```

**前置依赖**：
- Rust toolchain（由 `rust-toolchain.toml` 固定版本，`rustup` 自动安装）
- [DotSlash](https://dotslash-cli.com) - `cargo install dotslash`
- `protoc` - 通过 DotSlash 解析 `bin/protoc`

### 1.2 桌面端开发模式

```bash
cd grok-desktop

# 1. 构建 CLI 二进制（桌面端需要嵌入 grok.exe）
cd ..
cargo build -p xai-grok-pager-bin --release

# 2. 复制二进制到 Tauri resources 目录
./scripts/copy-grok-binary.ps1

# 3. 安装前端依赖
npm install

# 4. 启动 Tauri 开发服务器
npx tauri dev
```

**前置依赖**：
- Node.js 18+ / npm
- Rust toolchain
- Windows: Visual Studio Build Tools / WebView2

---

## 二、CLI/TUI 打包发布

### 2.1 本地构建 Release 二进制

```bash
# 标准 release 构建
cargo build -p xai-grok-pager-bin --release

# 产物位置
# target/release/xai-grok-pager.exe (Windows)
# target/release/xai-grok-pager (Linux/macOS)

# 硬化发布构建（用于分发，启用 LTO、strip 等）
cargo build -p xai-grok-pager-bin --profile release-dist

# 产物位置
# target/release-dist/xai-grok-pager.exe
```

**构建产物**：
| 平台 | 产物名 | 发布名 |
|------|--------|--------|
| Windows x64 | `xai-grok-pager.exe` | `grok.exe` |
| Linux x64 | `xai-grok-pager` | `grok` |
| macOS x64/arm64 | `xai-grok-pager` | `grok` |

### 2.2 交叉编译（可选，需配置）

```bash
# Linux x64 (需 musl target)
rustup target add x86_64-unknown-linux-musl
cargo build -p xai-grok-pager-bin --release --target x86_64-unknown-linux-musl

# macOS (需配置 cross-compilation toolchain)
cargo build -p xai-grok-pager-bin --release --target x86_64-apple-darwin
cargo build -p xai-grok-pager-bin --release --target aarch64-apple-darwin
```

### 2.3 生成安装包/分发制品

**Windows (NSIS/Zip)**：
```bash
# 使用 cargo-dist 或手动打包
# 产物命名约定: grok-<version>-<platform>.zip
# 例如: grok-0.2.112-windows-x86_64.zip
```

**Linux (tar.gz + .deb/.rpm)**：
```bash
# tar.gz
tar czf grok-<version>-linux-x86_64.tar.gz -C target/release xai-grok-pager

# .deb/.rpm 可使用 cargo-deb / cargo-rpm
cargo install cargo-deb
cargo deb -p xai-grok-pager-bin
```

**macOS (tar.gz + .dmg)**：
```bash
# 签名 + 公证 (Apple Developer ID 必需)
codesign --sign "Developer ID Application: ..." target/release/xai-grok-pager
# 创建 .dmg 使用 create-dmg 或 hdiutil
```

### 2.4 发布脚本参考（内部使用）

项目内部使用 `install.sh` 从 CDN 下载预构建二进制：
- 源: `https://x.ai/cli/grok-<version>-<platform>`
- 安装目录: `~/.grok/bin/grok`
- 支持通道: `stable` / `alpha` / `enterprise`

---

## 三、桌面端打包发布

### 3.1 构建流程

```bash
cd grok-desktop

# 1. 确保 CLI 二进制已构建 (release-dist profile 推荐)
cd ..
cargo build -p xai-grok-pager-bin --profile release-dist

# 2. 复制二进制到 Tauri resources
./scripts/copy-grok-binary.ps1 -Source "D:\Program Files (x86)\grok-build-main\grok-build-main\target\release-dist\xai-grok-pager.exe"

# 3. 构建前端
npm run build
# 等同于: tsc --noEmit && vite build

# 4. Tauri 打包
npx tauri build
```

### 3.2 Tauri 构建产物

```
grok-desktop/src-tauri/target/release/bundle/
├── nsis/
│   └── Grok Desktop_0.1.0_x64-setup.exe    # NSIS 安装包
├── msi/
│   └── Grok Desktop_0.1.0_x64.msi          # MSI 安装包 (可选)
└── app/                                    # 解压版
    └── Grok Desktop.exe
```

### 3.3 关键配置 (`tauri.conf.json`)

```json
{
  "bundle": {
    "active": true,
    "targets": "nsis",
    "resources": ["resources/grok.exe"],
    "windows": {
      "webviewInstallMode": { "type": "embedBootstrapper" }
    }
  }
}
```

**关键点**：
- `resources/grok.exe` - 嵌入的 CLI sidecar，运行时以 `agent serve` 启动
- `nsis` - Windows 安装包格式
- `embedBootstrapper` - 内嵌 WebView2 引导程序，无需用户预装

### 3.4 代码签名（生产环境必需）

```bash
# Windows 代码签名
signtool sign /fd sha256 /tr http://timestamp.digicert.com \
  /td sha256 /a "src-tauri/target/release/bundle/nsis/Grok Desktop_0.1.0_x64-setup.exe"

# 或配置 tauri.conf.json 的 windows.certificateThumbprint
```

---

## 四、完整发布流程清单

### 4.1 版本发布前检查

- [ ] 更新版本号：
  - `crates/codegen/xai-grok-pager-bin/Cargo.toml` (version)
  - `grok-desktop/package.json` (version)
  - `grok-desktop/src-tauri/tauri.conf.json` (version)
- [ ] 更新 `CHANGELOG.md`
- [ ] 运行完整测试套件：`cargo test --workspace`
- [ ] 代码格式化：`cargo fmt --all`
- [ ] Clippy 检查：`cargo clippy --workspace -- -D warnings`

### 4.2 CLI 发布步骤

```bash
# 1. 构建所有平台产物 (建议在 CI 中跨平台构建)
cargo build -p xai-grok-pager-bin --profile release-dist

# 2. 重命名并打包
# Windows: xai-grok-pager.exe -> grok.exe -> zip
# Linux: xai-grok-pager -> grok -> tar.gz
# macOS: xai-grok-pager -> grok -> tar.gz + codesign + notarize

# 3. 上传到 CDN (x.ai/cli 或 GCS)
# 路径格式: /grok-<version>-<platform>(.exe)

# 4. 更新版本指针文件
# stable -> <version>
# alpha -> <version>
```

### 4.3 桌面端发布步骤

```bash
# 1. 确保 CLI 已发布并可下载
# 2. 构建桌面端
cd grok-desktop
./scripts/copy-grok-binary.ps1  # 指向 release-dist 产物
npm run build
npx tauri build

# 3. 签名安装包
signtool sign ... "src-tauri/target/release/bundle/nsis/Grok Desktop_<version>_x64-setup.exe"

# 4. 上传发布制品
# - NSIS .exe
# - (可选) MSI
# - (可选) 便携版 zip
```

### 4.4 发布后验证

- [ ] CLI: `grok --version` 显示正确版本
- [ ] CLI: `grok login` / `grok chat` 基本功能正常
- [ ] 桌面端: 安装包安装无报错，启动进入工作区选择
- [ ] 桌面端: Agent 启动连接正常，可发送提示词
- [ ] 自动更新通道配置正确（如配置 Tauri updater）

---

## 五、CI/CD 集成建议

### GitHub Actions 矩阵构建示例

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  cli:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            artifact: grok-linux-x86_64.tar.gz
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: grok-macos-x86_64.tar.gz
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: grok-macos-aarch64.tar.gz
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: grok-windows-x86_64.zip
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          target: ${{ matrix.target }}
      - name: Build CLI
        run: cargo build -p xai-grok-pager-bin --profile release-dist --target ${{ matrix.target }}
      - name: Package
        run: |
          # 打包逻辑...
      - name: Upload
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: ${{ matrix.artifact }}

  desktop:
    needs: cli
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - name: Download CLI artifact
        uses: actions/download-artifact@v4
        with:
          name: grok-windows-x86_64.zip
      - name: Build Desktop
        run: |
          # 解压 CLI 到 resources
          # npm ci && npm run build
          # npx tauri build
      - name: Sign & Upload
        run: |
          signtool sign ...
          # 上传到 Release
```

---

## 六、常见问题排查

| 问题 | 解决方案 |
|------|----------|
| `cargo build` 报错 `linker not found` | Windows 安装 Visual Studio Build Tools + Windows 10 SDK |
| Tauri `WebView2` 报错 | 确保 `webviewInstallMode: embedBootstrapper` 或用户已装 WebView2 |
| Sidecar `grok.exe` 启动失败 | 检查 `resources/grok.exe` 存在且有执行权限；查看 Tauri 日志 `%APPDATA%\Grok Desktop\log` |
| 代码签名失败 | 确保证书在当前用户/机器存储区，时间戳服务器可达 |
| 交叉编译链接错误 | 使用 `cross` 或配置正确的 linker (`lld`, `clang`) |

---

## 七、相关文件速查

| 文件 | 用途 |
|------|------|
| `Cargo.toml` (workspace root) | 依赖版本、profile 配置 |
| `crates/codegen/xai-grok-pager-bin/Cargo.toml` | CLI 入口配置 |
| `grok-desktop/tauri.conf.json` | Tauri 打包配置 |
| `grok-desktop/package.json` | 前端构建脚本 |
| `grok-desktop/scripts/copy-grok-binary.ps1` | 复制 CLI 到 resources |
| `crates/codegen/xai-grok-pager/scripts/install.sh` | CLI 安装脚本参考 |
| `rust-toolchain.toml` | Rust 版本锁定 |

---

*文档版本: 1.0 | 更新日期: $(date +%Y-%m-%d) | 维护者: Grok Build Team*