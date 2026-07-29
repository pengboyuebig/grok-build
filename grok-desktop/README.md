# Grok Desktop

Grok Desktop is a Windows Tauri shell for the existing Grok Build agent. It does not replace the agent runtime: the app launches the bundled `grok.exe` with `agent serve` and connects to its authenticated loopback ACP WebSocket endpoint.

## Prerequisites

- Windows x64
- Rust toolchain for building the Tauri backend
- Node.js for building the React frontend
- A built `xai-grok-pager.exe` from the Grok Build repository
- Existing local Grok/API credentials in `%USERPROFILE%\\.grok`, `GROK_HOME`, or approved environment variables

The app deliberately has no login page and does not store model credentials.

## Development

```powershell
./scripts/copy-grok-binary.ps1
npm run build
```

For a Tauri development run, install the dependencies declared in `package.json`, then run:

```powershell
npx tauri dev
```

## Architecture

1. Tauri validates a selected workspace and launches `grok.exe agent serve` on a random `127.0.0.1` port.
2. Each launch uses a new secret in the WebSocket URL; it is never persisted or logged by the UI.
3. React connects to the ACP server, initializes the agent, creates a session, and sends prompts.
4. The agent continues to own model calls, tools, file edits, and workspace behavior.

## Current MVP Scope

- Select a workspace folder.
- Start and stop the embedded agent service.
- Send ACP session prompts and receive streamed text events.
- Display basic tool activity.
- Build an NSIS-targeted Windows Tauri bundle configuration.

The ACP surface will need additional event mappings for detailed file diffs, permission prompts, and richer command output before production use.
