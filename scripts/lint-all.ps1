<#
.SYNOPSIS
    اجرای کنترل کیفیت کل پروژه.
.DESCRIPTION
    rustfmt (حالت بررسی)، cargo clippy و ESLint فرانت‌اند.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot

Write-Host '[تاهان] بررسی قالب‌بندی Rust...' -ForegroundColor Cyan
Push-Location $root
try {
    cargo fmt --all -- --check
} finally {
    Pop-Location
}

Write-Host '[تاهان] اجرای clippy...' -ForegroundColor Cyan
Push-Location $root
try {
    cargo clippy --workspace --all-targets -- -D warnings
} finally {
    Pop-Location
}

Write-Host '[تاهان] اجرای ESLint فرانت‌اند...' -ForegroundColor Cyan
$frontend = Join-Path $root 'frontend'
Push-Location $frontend
try {
    npm run lint
} finally {
    Pop-Location
}

Write-Host '[تاهان] کنترل کیفیت کامل شد.' -ForegroundColor Green
