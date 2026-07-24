param(
    [ValidateSet('terminal', 'desktop', 'both')]
    [string]$Target = 'both'
)

$ErrorActionPreference = 'Stop'
$workspaceRoot = Split-Path -Parent $PSScriptRoot
$desktopManifest = Join-Path $workspaceRoot 'crates\codegen\xai-grok-desktop\Cargo.toml'

function Get-TargetTriple {
    $output = rustc -vV | Select-String '^host:'
    return ($output -split ':\s+')[1].Trim()
}

Push-Location $workspaceRoot
try {
    if ($Target -in @('terminal', 'both')) {
        cargo build --release -p xai-grok-pager-bin
    }

    if ($Target -in @('desktop', 'both')) {
        $tauriCli = Get-Command cargo-tauri -ErrorAction SilentlyContinue
        if (-not $tauriCli) {
            Write-Host "Tauri CLI not found; building terminal-only fallback." -ForegroundColor Yellow
            Write-Host "Install the Tauri CLI to produce the full desktop bundle:" -ForegroundColor Yellow
            Write-Host "  cargo install tauri-cli --version '^2.0'" -ForegroundColor Yellow
            cargo build --release -p xai-grok-desktop
        }
        else {
            $triple = Get-TargetTriple
            $terminalSrc = Join-Path $workspaceRoot 'target\release\xai-grok-pager.exe'
            $terminalDstDir = Join-Path $workspaceRoot 'crates\codegen\xai-grok-desktop\bin'
            $terminalDst = Join-Path $terminalDstDir "xai-grok-pager-$triple.exe"
            if (-not (Test-Path $terminalSrc)) {
                throw "Terminal binary not found at $terminalSrc. Build with 'terminal' first."
            }
            New-Item -ItemType Directory -Force -Path $terminalDstDir | Out-Null
            Copy-Item -Path $terminalSrc -Destination $terminalDst -Force
            cargo tauri build --manifest-path $desktopManifest
        }
    }
}
finally {
    Pop-Location
}
