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
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class ShellWin {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int cmd);
    [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr hWnd, int index);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(int flags, int dx, int dy, int data, IntPtr extra);
    [DllImport("user32.dll")] static extern IntPtr SendMessage(IntPtr hWnd, int msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] static extern int GetMenuItemCount(IntPtr menu);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetMenuString(IntPtr menu, uint item, StringBuilder text, int max, uint flags);
    [DllImport("user32.dll")] static extern uint GetMenuState(IntPtr menu, uint item, uint flags);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassName(IntPtr hWnd, StringBuilder name, int max);
    delegate bool EnumProc(IntPtr hWnd, IntPtr data);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc callback, IntPtr data);
    const uint ByPosition = 0x400, Separator = 0x800, Hilite = 0x80;

    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr hWnd, out Rect rect);

    // Where the newest open menu is on screen.
    public static Rect MenuBounds() {
        Rect bounds = new Rect();
        EnumWindows((hWnd, data) => {
            var name = new StringBuilder(64);
            GetClassName(hWnd, name, 64);
            if (name.ToString() == "#32768" && IsWindowVisible(hWnd)) {
                GetWindowRect(hWnd, out bounds);
                return false;
            }
            return true;
        }, IntPtr.Zero);
        return bounds;
    }

    // The open popup menus, oldest first: Windows draws each in a "#32768" window.
    public static List<IntPtr> PopupMenus() {
        var menus = new List<IntPtr>();
        EnumWindows((hWnd, data) => {
            var name = new StringBuilder(64);
            GetClassName(hWnd, name, 64);
            if (name.ToString() == "#32768" && IsWindowVisible(hWnd)) {
                IntPtr menu = SendMessage(hWnd, 0x1E1, IntPtr.Zero, IntPtr.Zero); // MN_GETHMENU
                if (menu != IntPtr.Zero) menus.Insert(0, menu);
            }
            return true;
        }, IntPtr.Zero);
        return menus;
    }

    public static string[] Items(IntPtr menu) {
        int count = GetMenuItemCount(menu);
        var items = new string[Math.Max(count, 0)];
        for (int i = 0; i < count; i++) {
            var text = new StringBuilder(256);
            GetMenuString(menu, (uint)i, text, 256, ByPosition);
            items[i] = (GetMenuState(menu, (uint)i, ByPosition) & Separator) != 0 ? "-" : text.ToString().Replace("&", "");
        }
        return items;
    }

    public static string Highlighted(IntPtr menu) {
        var items = Items(menu);
        for (int i = 0; i < items.Length; i++) {
            if ((GetMenuState(menu, (uint)i, ByPosition) & Hilite) != 0) return items[i];
        }
        return null;
    }
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
# Moves the keyboard highlight in the newest open menu to the item named $name.
function Highlight-Menu-Item([string] $name) {
    for ($step = 0; $step -lt 40; $step++) {
        $menu = [ShellWin]::PopupMenus() | Select-Object -Last 1
        if ($menu -and [ShellWin]::Highlighted($menu) -like $name) { return $true }
        Keys '{DOWN}' 250
    }
    return $false
}

foreach ($lang in 'en', 'ar') {
    $archive = Join-Path $menuFolder 'report.7z'
    Remove-Item $archive -ErrorAction SilentlyContinue
    Write-Settings @{ language = $lang; explorer_skin = 'windows11'; browser_mode = 'list' }
    $process = Start-Fahras $menuFolder "menu-$lang"
    # The folder's only file is selected; Shift+F10 opens its menu at the file,
    # not at the pointer, which waits in the top corner.
    [ShellWin]::SetCursorPos(1010, 3) | Out-Null
    Keys '+{F10}' 2500
    $menu = [ShellWin]::PopupMenus() | Select-Object -First 1
    Check "[$lang] Shift+F10 on a file opens the Windows menu" ([bool] $menu)
    Shot "explorer-05-native-menu-$lang" | Out-Null
    if ($menu) {
        $bounds = [ShellWin]::MenuBounds()
        Log "      menu at: $($bounds.Left),$($bounds.Top)"
        Check "[$lang] the keyboard opens the menu at the file" ($bounds.Top -gt 100 -and $bounds.Top -lt 400)
        if (Highlight-Menu-Item 'Send to') {
            Keys '{RIGHT}' 2000
            $targets = @([ShellWin]::Items(([ShellWin]::PopupMenus() | Select-Object -Last 1)) | Where-Object { $_ -and $_ -ne '-' })
            Log "      Send to: $($targets -join ' | ')"
            Shot "explorer-05b-send-to-$lang" | Out-Null
            Check "[$lang] Send to lists its places" ($targets.Count -ge 2)
            Keys '{ESC}{ESC}' 800
        } else {
            Check "[$lang] the menu has Send to" $false
        }
        Keys '+{F10}' 2500
        $menu = [ShellWin]::PopupMenus() | Select-Object -First 1
    }
    if ($menu) {
        $names = [ShellWin]::Items($menu)
        Log "      menu: $($names -join ' | ')"
        Check "[$lang] the menu has 7-Zip's entries" ($names -contains '7-Zip')
        if (Highlight-Menu-Item '7-Zip') {
            Keys '{RIGHT}' 1500
            $submenu = [ShellWin]::PopupMenus() | Select-Object -Last 1
            Log "      7-Zip: $(([ShellWin]::Items($submenu)) -join ' | ')"
            Shot "explorer-06-7zip-submenu-$lang" | Out-Null
            $found = Highlight-Menu-Item 'Add to "report.7z"'
            Check "[$lang] 7-Zip offers Add to report.7z" $found
            if ($found) {
                Keys '{ENTER}' 1000
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
function Click([int] $x, [int] $y) {
    [ShellWin]::SetCursorPos($x, $y) | Out-Null
    Start-Sleep -Milliseconds 150
    [ShellWin]::mouse_event(0x2, 0, 0, 0, [IntPtr]::Zero)
    [ShellWin]::mouse_event(0x4, 0, 0, 0, [IntPtr]::Zero)
    Start-Sleep -Milliseconds 1500
}

# The switch's knob, grey when off and accent-coloured when on: the first bright
# pixel at the row's far end, right (or left, right to left) of the text.
function Find-Switch([bool] $rightToLeft) {
    $bitmap = Screen
    $columns = if ($rightToLeft) { 20..140 } else { ($bitmap.Width - 140)..($bitmap.Width - 20) }
    try {
        foreach ($y in 150..400) {
            foreach ($x in $columns) {
                $pixel = $bitmap.GetPixel($x, $y)
                if ([math]::Max($pixel.R, [math]::Max($pixel.G, $pixel.B)) -gt 150) { return @(($x + 4), ($y + 6)) }
            }
        }
    } finally { $bitmap.Dispose() }
    return $null
}

# Settings search shows the one row; its switch is found on screen and clicked.
function Flip-Folder-Switch([string] $lang) {
    Write-Settings @{ language = $lang; explorer_skin = 'windows11'; browser_mode = 'list' }
    $process = Start-Fahras $folder "default-$lang"
    Keys '^,' 2500
    Keys '+{TAB}' 600
    Set-Clipboard -Value 'Open folders in Fahras'
    Keys '^v' 1500
    Shot "explorer-07-settings-$lang" | Out-Null
    $switch = Find-Switch ($lang -eq 'ar')
    Log "      switch at: $switch"
    if ($switch) { Click $switch[0] $switch[1] }
    Shot "explorer-08-switched-$lang" | Out-Null
    Keys '{ESC}' 500
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
