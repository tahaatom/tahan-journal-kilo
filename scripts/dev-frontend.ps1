<#
.SYNOPSIS
    شروع سرور توسعه فرانت‌اند (Vite).
.DESCRIPTION
    سرور توسعه Vite را در پورت 5173 راه‌اندازی می‌کند.
    اپلیکیشن Tahan در حالت توسعه به این آدرس متصل می‌شود.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$frontend = Join-Path $root 'frontend'

Write-Host '[تاهان] شروع سرور توسعه فرانت‌اند در http://localhost:5173 ...' -ForegroundColor Cyan
Push-Location $frontend
try {
    npm run dev
} finally {
    Pop-Location
}
