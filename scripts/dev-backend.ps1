<#
.SYNOPSIS
    شروع حلقه توسعه بک‌اند (کرنل Rust).
.DESCRIPTION
    در فاز ۰: اجرای cargo check برای کل ورک‌اسپیس.
    در فازهای بعدی: حلقه نظارتی کرنل.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot

Write-Host '[تاهان] بررسی کرنل با cargo check...' -ForegroundColor Cyan
Push-Location $root
try {
    cargo check --workspace --all-targets
} finally {
    Pop-Location
}

Write-Host '[تاهان] بررسی بک‌اند کامل شد.' -ForegroundColor Green
