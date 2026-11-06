# SPDX-License-Identifier: MIT
#
# Fahras installer check: shows the Arabic wizard, installs silently for the
# current user and checks the result, updates a running copy the way Fahras
# does, or uninstalls and checks it is gone.

param(
    [Parameter(Mandatory)] [string] $Setup,
    [Parameter(Mandatory)] [string] $Evidence,
    [Parameter(Mandatory)] [ValidateSet('install', 'update', 'uninstall')] [string] $Step
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing

New-Item -ItemType Directory -Force -Path $Evidence | Out-Null
$results = Join-Path $Evidence 'installer.txt'
$failures = 0
$app = Join-Path $env:LOCALAPPDATA 'Programs\Fahras'
$exe = Join-Path $app 'bin\Fahras.exe'
$shortcut = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Fahras.lnk'
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\{03512DC5-FE1A-451D-A96E-15B5EB7C4D48}_is1'

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

function Wait-Until([scriptblock] $condition, [int] $seconds = 60) {
    $deadline = (Get-Date).AddSeconds($seconds)
    while ((Get-Date) -lt $deadline) {
        if (& $condition) { return $true }
        Start-Sleep -Milliseconds 500
    }
    return $false
}

if ($Step -eq 'install') {
    Log "--- Installer: $(Split-Path -Leaf $Setup) ---"

    $wizard = Start-Process -FilePath $Setup -ArgumentList '/LANG=ar' -PassThru
    Start-Sleep -Seconds 8
    Shot 'installer-01-arabic-wizard'
    Get-Process | Where-Object { $_.Path -like '*\is-*.tmp\*' } | Stop-Process -Force -ErrorAction SilentlyContinue
    if (-not $wizard.HasExited) { Stop-Process -Id $wizard.Id -Force -ErrorAction SilentlyContinue }
    Start-Sleep -Seconds 2

    # A folder that already holds someone else's files keeps them on a first install.
    $foreign = Join-Path $app 'share\keep-me.txt'
    New-Item -ItemType Directory -Force -Path (Split-Path $foreign) | Out-Null
    Set-Content $foreign 'not Fahras'

    $log = Join-Path $Evidence 'installer-install.log'
    $install = Start-Process -FilePath $Setup -Wait -PassThru -ArgumentList @(
        '/VERYSILENT', '/SUPPRESSMSGBOXES', '/CURRENTUSER', '/NORESTART', "/LOG=`"$log`"")
    Check 'installer exits successfully' ($install.ExitCode -eq 0)
    Check 'Fahras.exe is installed under the user profile' (Test-Path $exe)
    Check "a first install leaves other files in the folder alone" (Test-Path $foreign)
    Check 'Start menu shortcut exists' (Test-Path $shortcut)
    # The PE header's subsystem: 2 is a windowed program, 3 opens a console beside it.
    $bytes = [System.IO.File]::ReadAllBytes($exe)
    $subsystem = [BitConverter]::ToUInt16($bytes, [BitConverter]::ToInt32($bytes, 0x3C) + 0x5C)
    Check 'Fahras opens without a console window' ($subsystem -eq 2)
    $entry = Get-ItemProperty -Path $uninstallKey -ErrorAction SilentlyContinue
    Check 'listed in Installed apps as Fahras' ($entry -and $entry.DisplayName -eq 'Fahras')
    if ($entry) { Log "      version: $($entry.DisplayVersion)" }

    $out = Join-Path $Evidence 'installed-version.txt'
    $version = Start-Process -FilePath $exe -ArgumentList '--version' -Wait -PassThru -NoNewWindow -RedirectStandardOutput $out
    $text = Get-Content $out -Raw -ErrorAction SilentlyContinue
    Log "      --version: $($text.Trim())"
    Check 'installed copy runs' ($version.ExitCode -eq 0 -and $text -like 'Fahras *')
} elseif ($Step -eq 'update') {
    Log '--- Update a running copy ---'
    Log '      starting the installed copy'
    $running = Start-Process -FilePath $exe -PassThru
    Check 'installed copy opens' (Wait-Until { $running.Refresh(); $running.MainWindowHandle -ne [IntPtr]::Zero })
    $before = Get-Date
    # The arguments src/fahras/windows_install.rs passes.
    Log '      running the installer over it'
    # Not -Wait: it also waits for the Fahras the installer reopens.
    $update = Start-Process -FilePath $Setup -PassThru -ArgumentList @(
        '/SILENT', '/SP-', '/SUPPRESSMSGBOXES', '/NORESTART', '/CLOSEAPPLICATIONS',
        '/FORCECLOSEAPPLICATIONS', '/RELAUNCH=1', "/LOG=`"$(Join-Path $Evidence 'installer-update.log')`"")
    $finished = $update.WaitForExit(180000)
    Log "      installer finished: $finished"
    if (-not $finished) { Shot 'installer-02-update-stuck'; Stop-Process -Id $update.Id -Force -ErrorAction SilentlyContinue }
    Check 'update installer exits successfully' ($finished -and $update.ExitCode -eq 0)
    Check 'the running copy was closed' (Wait-Until { $running.HasExited } 30)
    $relaunched = $null
    $opened = Wait-Until {
        $script:relaunched = Get-Process -Name Fahras -ErrorAction SilentlyContinue |
            Where-Object { $_.StartTime -gt $before -and $_.Path -eq $exe } | Select-Object -First 1
        [bool] $script:relaunched
    } 60
    Check 'Fahras opens again after the update' $opened
    Start-Sleep -Seconds 5
    Shot 'installer-02-reopened-after-update'
    Get-Process -Name Fahras -ErrorAction SilentlyContinue | Stop-Process -Force
} else {
    Log '--- Uninstaller ---'
    $entry = Get-ItemProperty -Path $uninstallKey -ErrorAction SilentlyContinue
    Check 'uninstall entry exists' ([bool] $entry)
    if ($entry) {
        $uninstaller = $entry.UninstallString.Trim('"')
        Start-Process -FilePath $uninstaller -Wait -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART'
        # The uninstaller hands off to a copy of itself in %TEMP% and returns early.
        $gone = Wait-Until { -not (Test-Path $exe) -and -not (Test-Path $uninstallKey) }
        Check 'program files are removed' ($gone -and -not (Test-Path $exe))
        Check 'Start menu shortcut is removed' (-not (Test-Path $shortcut))
        Check 'Installed apps entry is removed' (-not (Test-Path $uninstallKey))
        Check 'settings are kept' (Test-Path (Join-Path $env:LOCALAPPDATA 'strata\settings.toml'))
        foreach ($class in 'Directory', 'Drive', 'Directory\Background') {
            Check "Open in Fahras is gone from $class" (-not (Test-Path "HKCU:\Software\Classes\$class\shell\Fahras"))
        }
        foreach ($class in 'Directory', 'Drive') {
            $verb = (Get-ItemProperty -Path "HKCU:\Software\Classes\$class\shell" -ErrorAction SilentlyContinue).'(default)'
            Check "$class opens in File Explorer again" ($verb -ne 'Fahras')
        }
    }
}

Log "Failures: $failures"
exit $failures
