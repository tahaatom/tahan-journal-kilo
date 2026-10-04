<#
.SYNOPSIS
    اجرای تمام تست‌ها.
.DESCRIPTION
    تست‌های کرنل Rust و بیلد فرانت‌اند (به‌عنوان تست سلامت فرانت‌اند در فاز ۰).
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot

Write-Host '[تاهان] اجرای تست‌های کرنل...' -ForegroundColor Cyan
Push-Location $root
try {
    cargo test --workspace
} finally {
    Pop-Location
}

Write-Host '[تاهان] بیلد فرانت‌اند (تست سلامت)...' -ForegroundColor Cyan
$frontend = Join-Path $root 'frontend'
Push-Location $frontend
try {
    npm run build
} finally {
    Pop-Location
}

Write-Host '[تاهان] همه تست‌ها موفق.' -ForegroundColor Green
