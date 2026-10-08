# msvc-env.ps1 — فعال‌سازی محیط کامپایلر MSVC در نشست PowerShell جاری (ویندوز).
#
# چرا لازم است؟
#   موتور ذخیره‌سازی از SQLCipher با OpenSSL بسته‌بندی‌شده (vendored) استفاده
#   می‌کند. ساخت OpenSSL روی ویندوز به کامپایلر و سرصفحه‌ها/کتابخانه‌های MSVC
#   نیاز دارد که تنها پس از اجرای vcvarsall در محیط تنظیم می‌شوند.
#
# استفاده (dot-source):
#   . .\scripts\msvc-env.ps1
#   cargo build -p aria-storage-engine
#
# در CI لینوکس این اسکریپت لازم نیست (OpenSSL با gcc ساخته می‌شود).

$ErrorActionPreference = "Stop"

function Get-VcVarsAllPath {
    $candidates = @()
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $install = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($install) {
            $candidates += (Join-Path $install "VC\Auxiliary\Build\vcvarsall.bat")
        }
    }
    foreach ($root in @(
            "${env:ProgramFiles(x86)}\Microsoft Visual Studio",
            "${env:ProgramFiles}\Microsoft Visual Studio")) {
        if (Test-Path $root) {
            $candidates += Get-ChildItem -Path $root -Recurse -Filter "vcvarsall.bat" -ErrorAction SilentlyContinue |
                Select-Object -ExpandProperty FullName
        }
    }
    $candidates | Where-Object { $_ -and (Test-Path $_) } | Select-Object -First 1
}

function Import-MsvcEnvironment {
    param([string]$Arch = "x64")

    if (-not $IsWindows -and $PSVersionTable.PSEdition -eq "Core") {
        Write-Verbose "محیط غیرویندوزی؛ MSVC لازم نیست."
        return
    }

    if (Get-Command cl.exe -ErrorAction SilentlyContinue) {
        return
    }

    $vcvarsall = Get-VcVarsAllPath
    if (-not $vcvarsall) {
        Write-Warning "vcvarsall.bat یافت نشد؛ ساخت SQLCipher/OpenSSL ممکن است شکست بخورد."
        return
    }

    $lines = & cmd /c "`"$vcvarsall`" $Arch >nul 2>&1 && set"
    foreach ($line in $lines) {
        if ($line -match '^([^=]+)=(.*)$') {
            $name = $matches[1]
            $value = $matches[2]
            Set-Item -Path "Env:$name" -Value $value
        }
    }
    Write-Verbose "محیط MSVC فعال شد: $vcvarsall ($Arch)"
}

Import-MsvcEnvironment