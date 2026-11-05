# SPDX-License-Identifier: MIT
#
# Fahras look check on Windows: every app icon reaches the window and taskbar
# and survives a restart, both Explorer skins render in English and Arabic,
# and Settings changes the icon and skin live. Writes screenshots and
# look.txt to -Evidence; exits non-zero if a check fails.

param(
    [Parameter(Mandatory)] [string] $App,
    [Parameter(Mandatory)] [string] $Evidence
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class LookWin {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int cmd);
    [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr hWnd, int msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern IntPtr GetClassLongPtr(IntPtr hWnd, int index);
}
'@

New-Item -ItemType Directory -Force -Path $Evidence | Out-Null
$results = Join-Path $Evidence 'look.txt'
Set-Content -Path $results -Value "Fahras look check - $(Get-Date -Format o)" -Encoding utf8
$failures = 0
$settingsDir = Join-Path $env:LOCALAPPDATA 'strata'
$settingsFile = Join-Path $settingsDir 'settings.toml'
New-Item -ItemType Directory -Force -Path $settingsDir | Out-Null
$folder = 'C:\fahras-look'
New-Item -ItemType Directory -Force -Path $folder | Out-Null
foreach ($name in 'Documents', 'Reports', 'notes.txt', 'تقرير.txt') {
    $path = Join-Path $folder $name
    if ($name -like '*.txt') { Set-Content $path 'sample' } else { New-Item -ItemType Directory -Force -Path $path | Out-Null }
}

function Log([string] $line) {
    Write-Host $line
    Add-Content -Path $results -Value $line -Encoding utf8
}

function Check([string] $name, [bool] $ok) {
    if ($ok) { Log "PASS  $name" } else { Log "FAIL  $name"; $script:failures++ }
}

function Wait-Until([scriptblock] $condition, [int] $seconds = 30) {
    $deadline = (Get-Date).AddSeconds($seconds)
    while ((Get-Date) -lt $deadline) {
        if (& $condition) { return $true }
        Start-Sleep -Milliseconds 500
    }
    return $false
}

function Screen {
    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $graphics.Dispose()
    return $bitmap
}

function Shot([string] $name) {
    $bitmap = Screen
    $bitmap.Save((Join-Path $Evidence "$name.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $bitmap.Dispose()
}

$script:process = $null
function Focus-App {
    $script:process.Refresh()
    $handle = $script:process.MainWindowHandle
    if ($handle -ne [IntPtr]::Zero) {
        [LookWin]::ShowWindow($handle, 3) | Out-Null
        [LookWin]::SetForegroundWindow($handle) | Out-Null
    }
    Start-Sleep -Milliseconds 300
}

function Keys([string] $keys, [int] $settle = 800) {
    Focus-App
    [System.Windows.Forms.SendKeys]::SendWait($keys)
    Start-Sleep -Milliseconds $settle
}

# A null $settings keeps the saved settings file, as a restart does.
function Start-Fahras($settings, [string] $label) {
    if ($settings) {
        $lines = $settings.GetEnumerator() | ForEach-Object { "$($_.Key) = `"$($_.Value)`"" }
        Set-Content -Path $settingsFile -Value ($lines -join "`n") -Encoding utf8
    }
    $script:process = Start-Process -FilePath (Join-Path $App 'bin\Fahras.exe') -ArgumentList "`"$folder`"" -PassThru `
        -RedirectStandardError (Join-Path $Evidence "look-$label.log") -RedirectStandardOutput (Join-Path $Evidence "look-$label.out")
    $opened = Wait-Until { $script:process.Refresh(); $script:process.HasExited -or $script:process.MainWindowHandle -ne [IntPtr]::Zero } 60
    Check "[$label] window opens" ($opened -and -not $script:process.HasExited)
    Start-Sleep -Seconds 5
    Focus-App
    Start-Sleep -Seconds 1
}

function Stop-Fahras {
    if ($script:process -and -not $script:process.HasExited) {
        Stop-Process -Id $script:process.Id -Force
        $script:process.WaitForExit(10000) | Out-Null
    }
}

# The window's big icon, as Windows shows it on the taskbar and in Alt+Tab.
function Window-Icon([string] $name) {
    $script:process.Refresh()
    $handle = $script:process.MainWindowHandle
    $icon = [LookWin]::SendMessage($handle, 0x7F, [IntPtr]1, [IntPtr]::Zero)
    if ($icon -eq [IntPtr]::Zero) { $icon = [LookWin]::GetClassLongPtr($handle, -14) }
    if ($icon -eq [IntPtr]::Zero) { return $null }
    $path = Join-Path $Evidence "icon-$name.png"
    $bitmap = [System.Drawing.Icon]::FromHandle($icon).ToBitmap()
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bitmap.Dispose()
    return (Get-FileHash $path).Hash
}

# The sidebar's background, sampled in its empty lower part.
function Sidebar-Color([bool] $rightToLeft) {
    $bitmap = Screen
    $x = if ($rightToLeft) { $bitmap.Width - 40 } else { 40 }
    $color = $bitmap.GetPixel($x, [int]($bitmap.Height * 0.72))
    $bitmap.Dispose()
    return $color
}

function Near([System.Drawing.Color] $color, [string] $hex) {
    $expected = [System.Drawing.ColorTranslator]::FromHtml($hex)
    return ([math]::Abs($color.R - $expected.R) -le 4) -and ([math]::Abs($color.G - $expected.G) -le 4) -and ([math]::Abs($color.B - $expected.B) -le 4)
}

function Hex([System.Drawing.Color] $color) { '#{0:x2}{1:x2}{2:x2}' -f $color.R, $color.G, $color.B }

Log '--- Each app icon on the window and taskbar, after a restart ---'
$hashes = @{}
foreach ($icon in 'manila', 'forest', 'ink', 'clay') {
    Start-Fahras @{ language = 'en'; app_icon = $icon } "icon-$icon"
    $hashes[$icon] = Window-Icon $icon
    Log "      $icon window icon: $($hashes[$icon])"
    Check "[$icon] the window has an icon" ([bool] $hashes[$icon])
    Shot "look-icon-$icon"
    Stop-Fahras
}
Check 'each app icon looks different on the taskbar' (@($hashes.Values | Where-Object { $_ } | Sort-Object -Unique).Count -eq 4)

Log '--- Explorer skins, English and Arabic, light and dark ---'
# Sidebar colors from src/ui/theme/fahras_skin.rs: Windows 11 uses the surface,
# Windows 10 the background.
$skins = @(
    @{ label = 'en-windows11-light'; lang = 'en'; skin = 'windows11'; theme = 'github'; sidebar = '#f3f3f3' },
    @{ label = 'en-windows10-light'; lang = 'en'; skin = 'windows10'; theme = 'github'; sidebar = '#ffffff' },
    @{ label = 'en-windows11-dark'; lang = 'en'; skin = 'windows11'; theme = 'github-dark'; sidebar = '#202020' },
    @{ label = 'ar-windows11-light'; lang = 'ar'; skin = 'windows11'; theme = 'github'; sidebar = '#f3f3f3' },
    @{ label = 'ar-windows10-dark'; lang = 'ar'; skin = 'windows10'; theme = 'github-dark'; sidebar = '#191919' }
)
foreach ($case in $skins) {
    Start-Fahras @{ language = $case.lang; mode = 'theme'; theme = $case.theme; explorer_skin = $case.skin } $case.label
    $color = Sidebar-Color ($case.lang -eq 'ar')
    Log "      $($case.label) sidebar: $(Hex $color)"
    Check "[$($case.label)] sidebar uses Explorer's $($case.sidebar)" (Near $color $case.sidebar)
    Shot "look-skin-$($case.label)"
    Stop-Fahras
}

# Settings search shows only the matching row; four Tabs pass the clear
# button, the page entry and the close button to reach its choice.
function Choose-In-Settings([string] $query, [int] $down, [string] $shot) {
    Keys '^,' 2500
    Keys '+{TAB}' 600
    Keys '^a' 300
    Keys $query 1500
    Keys '{TAB}{TAB}{TAB}{TAB}' 800
    Keys ' ' 1200
    Shot "$shot-menu"
    Keys ('{DOWN}' * $down) 600
    Keys '{ENTER}' 1500
    Shot "$shot-chosen"
    Keys '{ESC}' 600
    Keys '{ESC}' 1500
}

function Saved([string] $line) {
    (Get-Content $settingsFile -Raw -ErrorAction SilentlyContinue) -match [regex]::Escape($line)
}

Log '--- Live change from Settings, then a restart ---'
foreach ($lang in 'en', 'ar') {
    $rtl = $lang -eq 'ar'
    Start-Fahras @{ language = $lang; mode = 'theme'; theme = 'github'; app_icon = 'manila' } "live-$lang"
    $before = Sidebar-Color $rtl
    Check "[live-$lang] starts on the Manila icon" ((Window-Icon "live-$lang-before") -eq $hashes['manila'])

    Choose-In-Settings 'skin' 2 "look-live-$lang-01-skin"
    Check "[live-$lang] Windows 11 skin is saved" (Saved 'explorer_skin = "windows11"')
    $after = Sidebar-Color $rtl
    Log "      sidebar $(Hex $before) -> $(Hex $after)"
    Check "[live-$lang] the open window takes the Windows 11 skin" (Near $after '#f3f3f3')
    Shot "look-live-$lang-02-skinned"

    Choose-In-Settings 'manila' 1 "look-live-$lang-03-icon"
    Check "[live-$lang] Forest icon is saved" (Saved 'app_icon = "forest"')
    Check "[live-$lang] the window and taskbar show Forest at once" ((Window-Icon "live-$lang-after") -eq $hashes['forest'])
    Shot "look-live-$lang-04-icon"
    Stop-Fahras

    Start-Fahras $null "live-$lang-restart"
    Check "[live-$lang] Forest icon after a restart" ((Window-Icon "live-$lang-restart") -eq $hashes['forest'])
    Check "[live-$lang] Windows 11 skin after a restart" (Near (Sidebar-Color $rtl) '#f3f3f3')
    Shot "look-live-$lang-05-restarted"
    Stop-Fahras
}

Log "Failures: $failures"
exit $failures
