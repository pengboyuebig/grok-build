param(
    [string]$Root = (Join-Path $PSScriptRoot '..')
)

$ErrorActionPreference = 'Stop'
$Root = Resolve-Path $Root

$uiRoot = Join-Path $Root 'crates\codegen\xai-grok-desktop\ui\src'
$rustRoot = Join-Path $Root 'crates\codegen\xai-grok-desktop\src'

$forbiddenUiPatterns = @(
    'dangerouslySetInnerHTML',
    'innerHTML',
    'outerHTML',
    'document\.write',
    'eval\(',
    'new Function',
    'style=\{\{'
)

$forbiddenRustPatterns = @(
    'Command::new\("cmd"',
    'Command::new\(''cmd''',
    'powershell',
    'cmd\.exe'
)

$violations = @()

if (Test-Path $uiRoot) {
    $uiFiles = Get-ChildItem -Path $uiRoot -Recurse -Include *.tsx, *.ts
    foreach ($file in $uiFiles) {
        $content = Get-Content -Path $file.FullName -Raw
        foreach ($pattern in $forbiddenUiPatterns) {
            if ($content -match $pattern) {
                $violations += "UI forbidden pattern '$pattern' found in $($file.FullName)"
            }
        }
    }
}

if (Test-Path $rustRoot) {
    $rustFiles = Get-ChildItem -Path $rustRoot -Recurse -Include *.rs
    foreach ($file in $rustFiles) {
        $content = Get-Content -Path $file.FullName -Raw
        foreach ($pattern in $forbiddenRustPatterns) {
            if ($content -match $pattern) {
                $violations += "Rust shell-spawn pattern '$pattern' found in $($file.FullName)"
            }
        }
    }
}

if ($violations.Count -gt 0) {
    Write-Host "Security scan failed with $($violations.Count) violation(s):" -ForegroundColor Red
    foreach ($violation in $violations) {
        Write-Host "  - $violation" -ForegroundColor Red
    }
    exit 1
}

Write-Host "Security scan passed." -ForegroundColor Green
