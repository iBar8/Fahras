# SPDX-License-Identifier: MIT
#
# Fahras installer check: shows the Arabic wizard, installs silently for the
# current user and checks the result, or uninstalls and checks it is gone.

param(
    [Parameter(Mandatory)] [string] $Setup,
    [Parameter(Mandatory)] [string] $Evidence,
    [Parameter(Mandatory)] [ValidateSet('install', 'uninstall')] [string] $Step
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

    $log = Join-Path $Evidence 'installer-install.log'
    $install = Start-Process -FilePath $Setup -Wait -PassThru -ArgumentList @(
        '/VERYSILENT', '/SUPPRESSMSGBOXES', '/CURRENTUSER', '/NORESTART', "/LOG=`"$log`"")
    Check 'installer exits successfully' ($install.ExitCode -eq 0)
    Check 'Fahras.exe is installed under the user profile' (Test-Path $exe)
    Check 'Start menu shortcut exists' (Test-Path $shortcut)
    $entry = Get-ItemProperty -Path $uninstallKey -ErrorAction SilentlyContinue
    Check 'listed in Installed apps as Fahras' ($entry -and $entry.DisplayName -eq 'Fahras')
    if ($entry) { Log "      version: $($entry.DisplayVersion)" }

    $out = Join-Path $Evidence 'installed-version.txt'
    $version = Start-Process -FilePath $exe -ArgumentList '--version' -Wait -PassThru -NoNewWindow -RedirectStandardOutput $out
    $text = Get-Content $out -Raw -ErrorAction SilentlyContinue
    Log "      --version: $($text.Trim())"
    Check 'installed copy runs' ($version.ExitCode -eq 0 -and $text -like 'Fahras *')
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
    }
}

Log "Failures: $failures"
exit $failures
