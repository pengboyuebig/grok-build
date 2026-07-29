param([string]$Source = "D:\Program Files (x86)\grok-build-main\grok-build-main\target\release\xai-grok-pager.exe")
$destination = Join-Path $PSScriptRoot "..\src-tauri\resources\grok.exe"
if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "找不到 Grok 二进制文件：$Source" }
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
Copy-Item -LiteralPath $Source -Destination $destination -Force
Write-Output "Copied Grok binary to $destination"
