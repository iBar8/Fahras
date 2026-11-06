# SPDX-License-Identifier: MIT
#
# Fahras as a Windows file manager, on the installed copy: "Open in Fahras" in
# Explorer, the Explorer layout beside real File Explorer, the native
# right-click menu with 7-Zip's entries, and opening folders in Fahras by
# default from Settings. Writes screenshots and explorer.txt to -Evidence;
# exits non-zero if a check fails.

param(
    [Parameter(Mandatory)] [string] $App,
    [Parameter(Mandatory)] [string] $Evidence
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing, UIAutomationClient, UIAutomationTypes
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ShellWin {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int cmd);
    [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr hWnd, int index);
}
'@

New-Item -ItemType Directory -Force -Path $Evidence | Out-Null
$results = Join-Path $Evidence 'explorer.txt'
Set-Content -Path $results -Value "Fahras in the Windows shell - $(Get-Date -Format o)" -Encoding utf8
$failures = 0
$exe = Join-Path $App 'bin\Fahras.exe'
$settingsDir = Join-Path $env:LOCALAPPDATA 'strata'
$settingsFile = Join-Path $settingsDir 'settings.toml'
New-Item -ItemType Directory -Force -Path $settingsDir | Out-Null

$folder = 'C:\fahras-explorer'
foreach ($name in 'Documents', 'Pictures', 'Reports') { New-Item -ItemType Directory -Force -Path (Join-Path $folder $name) | Out-Null }
foreach ($name in 'notes.txt', 'budget.csv', 'photo.png', 'تقرير.txt') { Set-Content (Join-Path $folder $name) 'sample' }
$menuFolder = 'C:\fahras-menu'
New-Item -ItemType Directory -Force -Path $menuFolder | Out-Null
Set-Content (Join-Path $menuFolder 'report.txt') 'archive me'

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
    $path = Join-Path $Evidence "$name.png"
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bitmap.Dispose()
    return $path
}

function Side-By-Side([string] $left, [string] $right, [string] $name) {
    $a = [System.Drawing.Image]::FromFile($left)
    $b = [System.Drawing.Image]::FromFile($right)
    $canvas = New-Object System.Drawing.Bitmap ($a.Width + $b.Width + 16), ([math]::Max($a.Height, $b.Height))
    $graphics = [System.Drawing.Graphics]::FromImage($canvas)
    $graphics.Clear([System.Drawing.Color]::FromArgb(40, 40, 40))
    $graphics.DrawImage($a, 0, 0, $a.Width, $a.Height)
    $graphics.DrawImage($b, $a.Width + 16, 0, $b.Width, $b.Height)
    $canvas.Save((Join-Path $Evidence "$name.png"), [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose(); $canvas.Dispose(); $a.Dispose(); $b.Dispose()
}

function Fahras-Windows {
    @(Get-Process -Name Fahras -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero })
}

function Stop-Fahras {
    Get-Process -Name Fahras -ErrorAction SilentlyContinue | Stop-Process -Force
    Wait-Until { -not (Get-Process -Name Fahras -ErrorAction SilentlyContinue) } 15 | Out-Null
}

function Toml-Value($value) {
    if ($value -is [array]) {
        return '[' + (($value | ForEach-Object { '"' + $_ + '"' }) -join ', ') + ']'
    }
    return '"' + $value + '"'
}

function Write-Settings([hashtable] $settings) {
    $lines = foreach ($entry in $settings.GetEnumerator()) { $entry.Key + ' = ' + (Toml-Value $entry.Value) }
    Set-Content -Path $settingsFile -Value ($lines -join "`n") -Encoding utf8
}

function Start-Fahras([string] $path, [string] $label) {
    $env:RUST_LOG = 'strata=debug'
    $process = Start-Process -FilePath $exe -ArgumentList "`"$path`"" -PassThru `
        -RedirectStandardError (Join-Path $Evidence "explorer-$label.log") -RedirectStandardOutput (Join-Path $Evidence "explorer-$label.out")
    $opened = Wait-Until { $process.Refresh(); $process.HasExited -or $process.MainWindowHandle -ne [IntPtr]::Zero } 60
    Check "[$label] Fahras opens" ($opened -and -not $process.HasExited)
    Start-Sleep -Seconds 5
    Focus $process.MainWindowHandle
    return $process
}

function Focus([IntPtr] $handle) {
    [ShellWin]::ShowWindow($handle, 3) | Out-Null
    [ShellWin]::SetForegroundWindow($handle) | Out-Null
    Start-Sleep -Milliseconds 800
}

function Keys([string] $keys, [int] $settle = 800) {
    [System.Windows.Forms.SendKeys]::SendWait($keys)
    Start-Sleep -Milliseconds $settle
}

function Explorer-Window([string] $path) {
    $shell = New-Object -ComObject Shell.Application
    foreach ($window in $shell.Windows()) {
        try {
            if ($window.Document.Folder.Self.Path -eq $path) { return $window }
        } catch { }
    }
    return $null
}

# ---------------------------------------------------------------- Open in Fahras
Log '--- "Open in Fahras" in File Explorer ---'
foreach ($class in 'Directory', 'Drive', 'Directory\Background') {
    $command = (Get-ItemProperty -Path "HKCU:\Software\Classes\$class\shell\Fahras\command" -ErrorAction SilentlyContinue).'(default)'
    Log "      $class command: $command"
    Check "Explorer offers Open in Fahras on $class" ($command -like "*Fahras.exe*")
}
$shell = New-Object -ComObject Shell.Application
$verbs = @($shell.NameSpace((Split-Path $folder)).ParseName((Split-Path -Leaf $folder)).Verbs())
Log "      folder menu: $(($verbs | ForEach-Object { $_.Name -replace '&', '' }) -join ' | ')"
$verb = $verbs | Where-Object { ($_.Name -replace '&', '') -eq 'Open in Fahras' } | Select-Object -First 1
Check 'the folder right-click menu lists Open in Fahras' ([bool] $verb)
if ($verb) {
    Stop-Fahras
    $verb.DoIt()
    $opened = Wait-Until { @(Fahras-Windows | Where-Object { $_.MainWindowTitle -like '*fahras-explorer*' }).Count -gt 0 } 60
    Check 'Open in Fahras opens that folder in Fahras' $opened
    Start-Sleep -Seconds 3
    Shot 'explorer-01-open-in-fahras' | Out-Null
    Stop-Fahras
}
$driveVerbs = @($shell.NameSpace(17).ParseName('C:\').Verbs() | ForEach-Object { $_.Name -replace '&', '' })
Check 'the drive right-click menu lists Open in Fahras' ($driveVerbs -contains 'Open in Fahras')

# ------------------------------------------------ The Explorer layout beside Explorer
Log '--- Fahras beside File Explorer ---'
$explorerShot = $null
Start-Process explorer.exe -ArgumentList "`"$folder`""
if (Wait-Until { [bool] (Explorer-Window $folder) } 30) {
    $window = Explorer-Window $folder
    Focus ([IntPtr] $window.HWND)
    Start-Sleep -Seconds 2
    $explorerShot = Shot 'explorer-02-file-explorer'
    $window.Quit()
}
Check 'File Explorer opens the same folder for comparison' ([bool] $explorerShot)

$layouts = @(
    @{ lang = 'en'; skin = 'windows11' },
    @{ lang = 'en'; skin = 'windows10' },
    @{ lang = 'ar'; skin = 'windows11' }
)
foreach ($layout in $layouts) {
    $label = "$($layout.lang)-$($layout.skin)"
    Write-Settings @{
        language = $layout.lang; mode = 'theme'; theme = 'github'; explorer_skin = $layout.skin
        browser_mode = 'list'; list_column_order = @('name', 'modified', 'type', 'size', 'mode'); list_hidden_columns = @('mode')
    }
    $process = Start-Fahras $folder $label
    $style = [ShellWin]::GetWindowLong($process.MainWindowHandle, -16)
    Log ("      window style: 0x{0:x8}" -f $style)
    # WS_CAPTION, WS_SYSMENU, WS_MINIMIZEBOX and WS_MAXIMIZEBOX: the system title bar.
    Check "[$label] uses the Windows title bar with minimize, maximize and close" (($style -band 0x00CB0000) -eq 0x00CB0000)
    $fahrasShot = Shot "explorer-03-fahras-$label"
    if ($explorerShot) { Side-By-Side $fahrasShot $explorerShot "explorer-04-beside-file-explorer-$label" }
    Stop-Process -Id $process.Id -Force
    Start-Sleep -Seconds 1
}

# --------------------------------------------------------- The native right-click menu
Log '--- Right-clicking a file shows the Windows menu ---'
$root = [System.Windows.Automation.AutomationElement]::RootElement
$menuCondition = New-Object System.Windows.Automation.PropertyCondition ([System.Windows.Automation.AutomationElement]::ClassNameProperty, '#32768')
$itemCondition = New-Object System.Windows.Automation.PropertyCondition ([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::MenuItem)
function Menus { @($root.FindAll([System.Windows.Automation.TreeScope]::Children, $menuCondition)) }
function Menu-Items($menu) { @($menu.FindAll([System.Windows.Automation.TreeScope]::Descendants, $itemCondition)) }

foreach ($lang in 'en', 'ar') {
    $archive = Join-Path $menuFolder 'report.7z'
    Remove-Item $archive -ErrorAction SilentlyContinue
    Write-Settings @{ language = $lang; explorer_skin = 'windows11'; browser_mode = 'list' }
    $process = Start-Fahras $menuFolder "menu-$lang"
    # The folder's only file is selected; Shift+F10 opens its menu.
    Keys '+{F10}' 2500
    $menu = Menus | Select-Object -First 1
    Check "[$lang] Shift+F10 on a file opens the Windows menu" ([bool] $menu)
    Shot "explorer-05-native-menu-$lang" | Out-Null
    if ($menu) {
        $names = @(Menu-Items $menu | ForEach-Object { $_.Current.Name })
        Log "      menu: $($names -join ' | ')"
        $sevenZip = Menu-Items $menu | Where-Object { $_.Current.Name -eq '7-Zip' } | Select-Object -First 1
        Check "[$lang] the menu has 7-Zip's entries" ([bool] $sevenZip)
        if ($sevenZip) {
            $sevenZip.GetCurrentPattern([System.Windows.Automation.ExpandCollapsePattern]::Pattern).Expand()
            Start-Sleep -Seconds 2
            $add = Menus | ForEach-Object { Menu-Items $_ } | Where-Object { $_.Current.Name -like 'Add to "report.7z"' } | Select-Object -First 1
            Shot "explorer-06-7zip-submenu-$lang" | Out-Null
            Check "[$lang] 7-Zip offers Add to report.7z" ([bool] $add)
            if ($add) {
                $add.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
                Check "[$lang] choosing it creates the archive" (Wait-Until { Test-Path $archive } 60)
            }
        }
    }
    Keys '{ESC}{ESC}' 500
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
}

# ------------------------------------------------- Opening folders in Fahras by default
Log '--- Settings: open folders in Fahras ---'
function Folder-Verb([string] $class) {
    (Get-ItemProperty -Path "HKCU:\Software\Classes\$class\shell" -ErrorAction SilentlyContinue).'(default)'
}
# Settings search shows the one row; four Tabs reach its switch and Space flips it.
function Flip-Folder-Switch([string] $lang) {
    Write-Settings @{ language = $lang; explorer_skin = 'windows11'; browser_mode = 'list' }
    $process = Start-Fahras $folder "default-$lang"
    Keys '^,' 2500
    Keys '+{TAB}' 600
    Set-Clipboard -Value 'Open folders in Fahras'
    Keys '^v' 1500
    Shot "explorer-07-settings-$lang" | Out-Null
    Keys '{TAB}{TAB}{TAB}{TAB}' 800
    Keys ' ' 1500
    Shot "explorer-08-switched-$lang" | Out-Null
    Keys '{ESC}' 500
    Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
}

Flip-Folder-Switch 'en'
Check 'turning it on makes Fahras open folders' (((Folder-Verb 'Directory') -eq 'Fahras') -and ((Folder-Verb 'Drive') -eq 'Fahras'))
Stop-Fahras
Start-Process (Join-Path $folder 'Documents')
Check 'double-clicking a folder in Windows opens it in Fahras' (Wait-Until { @(Fahras-Windows | Where-Object { $_.MainWindowTitle -like 'Documents*' }).Count -gt 0 } 60)
Start-Sleep -Seconds 3
Shot 'explorer-09-folder-opened-in-fahras' | Out-Null
Stop-Fahras

Flip-Folder-Switch 'ar'
Check 'turning it off in Arabic gives folders back to File Explorer' ((Folder-Verb 'Directory') -ne 'Fahras')
Start-Process (Join-Path $folder 'Pictures')
$backInExplorer = Wait-Until { [bool] (Explorer-Window (Join-Path $folder 'Pictures')) } 30
Check 'double-clicking a folder opens File Explorer again' $backInExplorer
if ($backInExplorer) { (Explorer-Window (Join-Path $folder 'Pictures')).Quit() }

# Left on, so the uninstall check sees Fahras hand folders back.
Flip-Folder-Switch 'en'
Check 'turned back on before uninstalling' ((Folder-Verb 'Directory') -eq 'Fahras')
Stop-Fahras

Log "Failures: $failures"
exit $failures
