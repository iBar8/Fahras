# SPDX-License-Identifier: MIT
#
# Fahras Windows smoke test: starts the packaged app on the runner's desktop,
# drives it with real keyboard input, checks every file operation on disk and
# takes screenshots. Writes evidence (screenshots, results.txt, app logs) to
# -Evidence. Exits non-zero if any check fails.

param(
    [Parameter(Mandatory)] [string] $App,
    [Parameter(Mandatory)] [string] $Evidence
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class Win {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int cmd);
}
'@

New-Item -ItemType Directory -Force -Path $Evidence | Out-Null
$results = Join-Path $Evidence 'results.txt'
Set-Content -Path $results -Value "Fahras Windows smoke test - $(Get-Date -Format o)" -Encoding utf8
$failures = 0

function Log([string] $line) {
    Write-Host $line
    Add-Content -Path $results -Value $line -Encoding utf8
}

function Check([string] $name, [bool] $ok) {
    if ($ok) { Log "PASS  $name" } else { Log "FAIL  $name"; $script:failures++ }
}

function Shot([string] $name) {
    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save((Join-Path $Evidence "$name.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose(); $bitmap.Dispose()
}

function Keys([string] $keys, [int] $settle = 1500) {
    Focus-App
    [System.Windows.Forms.SendKeys]::SendWait($keys)
    Start-Sleep -Milliseconds $settle
}

$script:process = $null
function Focus-App {
    $script:process.Refresh()
    $handle = $script:process.MainWindowHandle
    if ($handle -ne [IntPtr]::Zero) {
        [Win]::ShowWindow($handle, 3) | Out-Null   # maximize
        [Win]::SetForegroundWindow($handle) | Out-Null
    }
    Start-Sleep -Milliseconds 300
}

function Wait-Until([scriptblock] $condition, [int] $seconds = 20) {
    $deadline = (Get-Date).AddSeconds($seconds)
    while ((Get-Date) -lt $deadline) {
        if (& $condition) { return $true }
        Start-Sleep -Milliseconds 500
    }
    return $false
}

function Start-Fahras([string] $folder, [string] $label) {
    $log = Join-Path $Evidence "app-$label.log"
    $env:RUST_LOG = 'strata=debug'
    $script:process = Start-Process -FilePath (Join-Path $App 'bin\Fahras.exe') `
        -ArgumentList "`"$folder`"" -PassThru `
        -RedirectStandardError $log -RedirectStandardOutput (Join-Path $Evidence "app-$label.out")
    $opened = Wait-Until {
        $script:process.Refresh()
        $script:process.HasExited -or $script:process.MainWindowHandle -ne [IntPtr]::Zero
    } 60
    Check "[$label] window opens" ($opened -and -not $script:process.HasExited)
    if (-not $script:process.HasExited) {
        Log "      window title: '$($script:process.MainWindowTitle)'"
    } else {
        Log "      exited with code $($script:process.ExitCode)"
    }
    Start-Sleep -Seconds 6
    Focus-App
}

function Stop-Fahras {
    if ($script:process -and -not $script:process.HasExited) {
        Stop-Process -Id $script:process.Id -Force
        $script:process.WaitForExit(10000) | Out-Null
    }
}

# Opens `$path` through the location bar (Ctrl+L), then focuses the listing.
function Go([string] $path) {
    Keys '^l' 800
    Keys '^a' 300
    Keys $path 500
    Keys '{ENTER}' 2500
    Keys '{ESC}' 500
}

function Run-Suite([string] $root, [string] $label) {
    $ops = @{
        rename = Join-Path $root 'rename'
        copy = Join-Path $root 'copy'
        copied = Join-Path $root 'copy-destination'
        move = Join-Path $root 'move'
        moved = Join-Path $root 'move-destination'
        delete = Join-Path $root 'delete'
        arabic = Join-Path $root 'عربي'
    }
    foreach ($folder in $ops.Values) { New-Item -ItemType Directory -Force -Path $folder | Out-Null }
    Set-Content (Join-Path $ops.rename 'alpha.txt') 'alpha'
    Set-Content (Join-Path $ops.copy 'beta.txt') 'beta'
    Set-Content (Join-Path $ops.move 'gamma.txt') 'gamma'
    Set-Content (Join-Path $ops.delete 'delta.txt') 'delta'
    Set-Content (Join-Path $ops.arabic 'تقرير الشهر.txt') 'تقرير'
    Set-Content (Join-Path $ops.arabic 'notes.md') '# notes'

    Start-Fahras $root $label
    if ($script:process.HasExited) { return }
    Shot "$label-01-window"

    Go 'C:\'
    Shot "$label-02-drive-c"
    Go $ops.arabic
    Shot "$label-03-arabic-names"
    Set-Content (Join-Path $ops.arabic 'created-outside.txt') 'live refresh'
    Start-Sleep -Seconds 4
    Shot "$label-03b-created-outside"

    Go $ops.rename
    Keys '^a' 500
    Keys '{F2}' 1000
    Keys '^a' 300
    Keys 'renamed.txt' 300
    Keys '{ENTER}' 2500
    Shot "$label-04-renamed"
    Check "[$label] rename alpha.txt -> renamed.txt" (
        (Test-Path (Join-Path $ops.rename 'renamed.txt')) -and -not (Test-Path (Join-Path $ops.rename 'alpha.txt')))

    Go $ops.copy
    Keys '^a' 500
    Keys '^c' 800
    Go $ops.copied
    Keys '^v' 3000
    Shot "$label-05-copied"
    Check "[$label] copy beta.txt" (
        (Test-Path (Join-Path $ops.copied 'beta.txt')) -and (Test-Path (Join-Path $ops.copy 'beta.txt')))
    Check "[$label] copied contents match" (
        (Get-Content (Join-Path $ops.copied 'beta.txt') -ErrorAction SilentlyContinue) -eq 'beta')

    Go $ops.move
    Keys '^a' 500
    Keys '^x' 800
    Go $ops.moved
    Keys '^v' 3000
    Shot "$label-06-moved"
    Check "[$label] move gamma.txt" (
        (Test-Path (Join-Path $ops.moved 'gamma.txt')) -and -not (Test-Path (Join-Path $ops.move 'gamma.txt')))

    Go $ops.delete
    Keys '^a' 500
    Keys '{DELETE}' 3000
    Shot "$label-07-deleted"
    Check "[$label] delete delta.txt to the Recycle Bin" (-not (Test-Path (Join-Path $ops.delete 'delta.txt')))

    Check "[$label] still running after all operations" (-not $script:process.HasExited)
    Stop-Fahras
}

$settingsDir = Join-Path $env:LOCALAPPDATA 'strata'
New-Item -ItemType Directory -Force -Path $settingsDir | Out-Null

Log '--- English ---'
Set-Content (Join-Path $settingsDir 'settings.toml') 'language = "en"' -Encoding utf8
Run-Suite 'C:\fahras-smoke-en' 'en'

Log '--- Arabic ---'
Set-Content (Join-Path $settingsDir 'settings.toml') 'language = "ar"' -Encoding utf8
Run-Suite 'C:\fahras-smoke-ar' 'ar'

$recycled = @((New-Object -ComObject Shell.Application).NameSpace(10).Items() | ForEach-Object { $_.Name })
Log "Recycle Bin: $($recycled -join ', ')"
Check "deleted files are in the Recycle Bin" (@($recycled | Where-Object { $_ -like 'delta*' }).Count -ge 2)
Log "Failures: $failures"
exit $failures
