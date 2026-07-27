# Grok Desktop

Grok Desktop is the graphical AI coding client. The existing `xai-grok-pager.exe` terminal UI remains a separate product.

For a browser-hosted local control surface, see `docs/browser-console.md`.

## Windows build targets

Run from the repository root:

```powershell
./scripts/build-windows.ps1 terminal
./scripts/build-windows.ps1 desktop
./scripts/build-windows.ps1 both
```

- `terminal` builds the established terminal product with Cargo.
- `desktop` builds the Tauri desktop package and its NSIS installer.
- `both` produces both products and includes the terminal executable beside the desktop bundle for one-click terminal handoff.

## Output locations

After a successful `desktop` or `both` build, artifacts are written to:

```
target/x86_64-pc-windows-msvc/release/bundle/nsis/Grok Desktop_0.1.0_x64-setup.exe
```

The terminal companion binary is copied into the desktop bundle automatically via the `externalBin` configuration in `tauri.conf.json`.

## Prerequisites

The desktop build requires:

- Rust toolchain pinned by `rust-toolchain.toml`
- Node.js and npm
- Tauri CLI: `cargo install tauri-cli --version '^2.0'`
- WebView2 Runtime
- Microsoft C++ build tools

## Code signing

Sign release installers with the organisation's Windows code-signing certificate before distribution. Configure the certificate thumbprint or path in the Tauri bundle settings when running in CI.
