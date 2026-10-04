<#
.SYNOPSIS
    قالب‌بندی کل پروژه.
.DESCRIPTION
    cargo fmt برای کد Rust و Prettier برای فرانت‌اند.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot

Write-Host '[تاهان] قالب‌بندی Rust...' -ForegroundColor Cyan
Push-Location $root
try {
    cargo fmt --all
} finally {
    Pop-Location
}

Write-Host '[تاهان] قالب‌بندی فرانت‌اند...' -ForegroundColor Cyan
$frontend = Join-Path $root 'frontend'
Push-Location $frontend
try {
    npm run format
} finally {
    Pop-Location
}

Write-Host '[تاهان] قالب‌بندی کامل شد.' -ForegroundColor Green
