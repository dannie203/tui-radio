# ==============================================================================
#  BOOMBOX-RS: Retro Cyberpunk Cassette Boombox & Worldwide Radio Explorer
#  Automated One-Line Installer for Windows (PowerShell 5.1+)
#  Repository: https://github.com/dannie203/tui-radio
# ==============================================================================

$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

# Enable TLS 1.2 & TLS 1.3 for secure web queries
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

$Repo = "dannie203/tui-radio"
$InstallDir = "$env:LOCALAPPDATA\Programs\Boombox"
$ZipUrl = "https://github.com/$Repo/releases/latest/download/boombox-rs-windows-x86_64.zip"

Write-Host ""
Write-Host "  ____   ____   ____  __  __ ____   ______  __" -ForegroundColor Yellow
Write-Host " | __ ) / __ \ / __ \|  \/  | __ ) / __ \ \/ /" -ForegroundColor Yellow
Write-Host " |  _ \| |  | | |  | | |\/| |  _ \| |  | |\  / " -ForegroundColor Yellow
Write-Host " | |_) | |__| | |__| | |  | | |_) | |__| |/  \ " -ForegroundColor Yellow
Write-Host " |____/ \____/ \____/|_|  |_|____/ \____//_/\_\ RX-505" -ForegroundColor Yellow
Write-Host " 📼 Automated Installer for Windows (v3.8.9+)" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray

# ------------------------------------------------------------------------------
# 1. Ensure Target Installation Directory
# ------------------------------------------------------------------------------
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

# ------------------------------------------------------------------------------
# 2. Check & Install Microsoft Visual C++ 2015-2022 Redistributable (x64)
# ------------------------------------------------------------------------------
$VcRuntimePath = "$env:SystemRoot\System32\vcruntime140.dll"
if (!(Test-Path $VcRuntimePath)) {
    Write-Host "⚙️ [1/5] Microsoft Visual C++ Runtime missing. Installing silently..." -ForegroundColor Cyan
    $VcTemp = "$env:TEMP\vc_redist.x64.exe"
    try {
        Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vc_redist.x64.exe" -OutFile $VcTemp -UseBasicParsing
        Start-Process -FilePath $VcTemp -ArgumentList "/install /quiet /norestart" -Wait
        Write-Host "  ✓ Visual C++ Redistributable installed." -ForegroundColor Green
    }
    catch {
        Write-Host "  ⚠️ Could not auto-install Visual C++. You can get it at: https://aka.ms/vs/17/release/vc_redist.x64.exe" -ForegroundColor Yellow
    }
    finally {
        if (Test-Path $VcTemp) { Remove-Item $VcTemp -Force -ErrorAction SilentlyContinue }
    }
} else {
    Write-Host "  ✓ [1/5] Visual C++ Runtime detected." -ForegroundColor Green
}

# ------------------------------------------------------------------------------
# 3. Install Boombox-RS Executable (Local file or Remote Release)
# ------------------------------------------------------------------------------
$ScriptDir = if ($MyInvocation.MyCommand.Path) { Split-Path -Parent $MyInvocation.MyCommand.Path } else { $null }
$LocalExe = if ($ScriptDir) { Join-Path $ScriptDir "boombox-rs.exe" } else { $null }

if ($LocalExe -and (Test-Path $LocalExe)) {
    Write-Host "📦 [2/5] Installing Boombox-RS from local directory..." -ForegroundColor Cyan
    Copy-Item $LocalExe -Destination "$InstallDir\boombox-rs.exe" -Force
} else {
    Write-Host "📥 [2/5] Downloading latest Boombox-RS release from GitHub..." -ForegroundColor Cyan
    $TempZip = "$env:TEMP\boombox-release-$([guid]::NewGuid().ToString().Substring(0,8)).zip"
    $TempExtract = "$env:TEMP\boombox-extract-$([guid]::NewGuid().ToString().Substring(0,8))"
    try {
        Invoke-WebRequest -Uri $ZipUrl -OutFile $TempZip -UseBasicParsing
        Expand-Archive -Path $TempZip -DestinationPath $TempExtract -Force

        $FoundExe = Get-ChildItem -Path $TempExtract -Filter "boombox-rs.exe" -Recurse | Select-Object -First 1
        if ($FoundExe) {
            Copy-Item $FoundExe.FullName -Destination "$InstallDir\boombox-rs.exe" -Force
        } else {
            throw "boombox-rs.exe not found in downloaded release zip."
        }

        # Also copy fallback.json or assets if present in archive
        $FoundFallback = Get-ChildItem -Path $TempExtract -Filter "fallback.json" -Recurse | Select-Object -First 1
        if ($FoundFallback) {
            $DataDir = "$InstallDir\data"
            if (!(Test-Path $DataDir)) { New-Item -ItemType Directory -Force -Path $DataDir | Out-Null }
            Copy-Item $FoundFallback.FullName -Destination "$DataDir\fallback.json" -Force
        }
        Write-Host "  ✓ Boombox-RS binary installed to $InstallDir" -ForegroundColor Green
    }
    catch {
        Write-Host "  ❌ Failed to download Boombox release: $_" -ForegroundColor Red
        throw $_
    }
    finally {
        if (Test-Path $TempZip) { Remove-Item $TempZip -Force -ErrorAction SilentlyContinue }
        if (Test-Path $TempExtract) { Remove-Item $TempExtract -Recurse -Force -ErrorAction SilentlyContinue }
    }
}

# ------------------------------------------------------------------------------
# 4. Check & Install Audio Engine: MPV (BẮT BUỘC để phát nhạc)
# ------------------------------------------------------------------------------
$HasMpv = (Get-Command "mpv" -ErrorAction SilentlyContinue) -or (Test-Path "$InstallDir\mpv.exe")
if ($HasMpv) {
    Write-Host "  ✓ [3/5] MPV Audio Engine detected." -ForegroundColor Green
} else {
    Write-Host "⚙️ [3/5] MPV Audio Engine is required. Attempting automated installation..." -ForegroundColor Cyan
    $InstalledMpv = $false

    # Option A: Install via WinGet
    if (Get-Command "winget" -ErrorAction SilentlyContinue) {
        Write-Host "  ▶ Installing shinchiro.mpv via Windows Package Manager (winget)..." -ForegroundColor DarkGray
        try {
            Start-Process -FilePath "winget" -ArgumentList "install --id shinchiro.mpv -e --accept-source-agreements --accept-package-agreements --silent" -NoNewWindow -Wait
            if ((Get-Command "mpv" -ErrorAction SilentlyContinue) -or (Test-Path "$env:LOCALAPPDATA\Microsoft\WindowsApps\mpv.exe")) {
                $InstalledMpv = $true
                Write-Host "  ✓ MPV installed successfully via winget." -ForegroundColor Green
            }
        } catch { }
    }

    # Option B: Download standalone portable MPV if winget not available or failed
    if (!$InstalledMpv -and (Get-Command "tar.exe" -ErrorAction SilentlyContinue)) {
        Write-Host "  ▶ Downloading standalone portable MPV build..." -ForegroundColor DarkGray
        $Mpv7z = "$env:TEMP\mpv-portable.7z"
        $MpvExtract = "$env:TEMP\mpv-extract-$([guid]::NewGuid().ToString().Substring(0,8))"
        try {
            # Direct link to shinchiro's stable git build
            $MpvUrl = "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20260903/mpv-x86_64-20260903-git-69e63f425a.7z"
            Invoke-WebRequest -Uri $MpvUrl -OutFile $Mpv7z -UseBasicParsing
            New-Item -ItemType Directory -Force -Path $MpvExtract | Out-Null
            & tar.exe -xf $Mpv7z -C $MpvExtract

            $FoundMpv = Get-ChildItem -Path $MpvExtract -Filter "mpv.exe" -Recurse | Select-Object -First 1
            if ($FoundMpv) {
                Copy-Item $FoundMpv.FullName -Destination "$InstallDir\mpv.exe" -Force
                $InstalledMpv = $true
                Write-Host "  ✓ Portable MPV installed into $InstallDir\mpv.exe." -ForegroundColor Green
            }
        }
        catch { }
        finally {
            if (Test-Path $Mpv7z) { Remove-Item $Mpv7z -Force -ErrorAction SilentlyContinue }
            if (Test-Path $MpvExtract) { Remove-Item $MpvExtract -Recurse -Force -ErrorAction SilentlyContinue }
        }
    }

    if (!$InstalledMpv) {
        Write-Host "  ⚠️ Could not auto-install MPV. You can install it manually via:" -ForegroundColor Yellow
        Write-Host "     winget install shinchiro.mpv" -ForegroundColor White
        Write-Host "     or place 'mpv.exe' directly inside: $InstallDir" -ForegroundColor White
    }
}

# ------------------------------------------------------------------------------
# 5. Check & Install Stream Helper: yt-dlp (Khuyến nghị cho YouTube / SoundCloud)
# ------------------------------------------------------------------------------
$HasYtDlp = (Get-Command "yt-dlp" -ErrorAction SilentlyContinue) -or (Test-Path "$InstallDir\yt-dlp.exe")
if ($HasYtDlp) {
    Write-Host "  ✓ [4/5] yt-dlp Stream Helper detected." -ForegroundColor Green
} else {
    Write-Host "📥 [4/5] Downloading portable yt-dlp stream helper..." -ForegroundColor Cyan
    try {
        Invoke-WebRequest -Uri "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe" -OutFile "$InstallDir\yt-dlp.exe" -UseBasicParsing
        Write-Host "  ✓ yt-dlp installed to $InstallDir\yt-dlp.exe" -ForegroundColor Green
    } catch {
        Write-Host "  ⚠️ Could not download yt-dlp. You can install it via: winget install yt-dlp" -ForegroundColor Yellow
    }
}

# ------------------------------------------------------------------------------
# 6. Check & Install Audio Transcoder: FFmpeg (Khuyến nghị cho Tape Recorder)
# ------------------------------------------------------------------------------
$HasFfmpeg = (Get-Command "ffmpeg" -ErrorAction SilentlyContinue) -or (Test-Path "$InstallDir\ffmpeg.exe")
if ($HasFfmpeg) {
    Write-Host "  ✓ [5/5] FFmpeg Transcoder detected." -ForegroundColor Green
} else {
    Write-Host "💡 [5/5] Notice: FFmpeg is optional (used for Cassette Tape Recording)." -ForegroundColor DarkGray
    if (Get-Command "winget" -ErrorAction SilentlyContinue) {
        Write-Host "  ▶ You can install it anytime with: winget install Gyan.FFmpeg" -ForegroundColor DarkGray
    }
}

# ------------------------------------------------------------------------------
# 7. Create Launchers (RUN-BOOMBOX.bat & boombox.cmd)
# ------------------------------------------------------------------------------
$BatContent = @"
@echo off
chcp 65001 >nul
title Boombox RX-505
cd /d "%~dp0"
set "PATH=%~dp0;%PATH%"
where wt.exe >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    start wt.exe --title "Boombox RX-505" "%~dp0boombox-rs.exe"
) else (
    start cmd.exe /k "chcp 65001 >nul && "%~dp0boombox-rs.exe""
)
"@
Set-Content -Path "$InstallDir\RUN-BOOMBOX.bat" -Value $BatContent -Encoding ASCII

$CmdContent = @"
@echo off
set "PATH=%~dp0;%PATH%"
"%~dp0boombox-rs.exe" %*
"@
Set-Content -Path "$InstallDir\boombox.cmd" -Value $CmdContent -Encoding ASCII

# ------------------------------------------------------------------------------
# 8. Create Uninstaller Script (uninstall.ps1)
# ------------------------------------------------------------------------------
$UninstallerContent = @"
`$ErrorActionPreference = 'SilentlyContinue'
Write-Host 'Uninstalling Boombox RX-505...' -ForegroundColor Yellow
`$InstallDir = "`$env:LOCALAPPDATA\Programs\Boombox"
`$DesktopShortcut = "`$([Environment]::GetFolderPath('Desktop'))\Boombox RX-505.lnk"
`$StartShortcut = "`$([Environment]::GetFolderPath('Programs'))\Boombox RX-505.lnk"

if (Test-Path `$DesktopShortcut) { Remove-Item `$DesktopShortcut -Force }
if (Test-Path `$StartShortcut) { Remove-Item `$StartShortcut -Force }

`$UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (`$UserPath -like "*`$InstallDir*") {
    `$NewPath = (`$UserPath.Split(';') | Where-Object { `$_ -ne `$InstallDir }) -join ';'
    [Environment]::SetEnvironmentVariable('Path', `$NewPath, 'User')
}

Write-Host 'To completely remove binary files, delete folder:' -ForegroundColor Cyan
Write-Host `$InstallDir -ForegroundColor White
Write-Host 'Boombox RX-505 shortcuts and environment variables removed.' -ForegroundColor Green
"@
Set-Content -Path "$InstallDir\uninstall.ps1" -Value $UninstallerContent -Encoding UTF8

# ------------------------------------------------------------------------------
# 9. Register InstallDir into User PATH Environment Variable
# ------------------------------------------------------------------------------
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "⚙️ Adding $InstallDir to User PATH..." -ForegroundColor Cyan
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
}

# ------------------------------------------------------------------------------
# 10. Create Desktop & Start Menu Shortcuts
# ------------------------------------------------------------------------------
try {
    $WshShell = New-Object -ComObject WScript.Shell

    # Desktop Shortcut
    $DesktopPath = [Environment]::GetFolderPath("Desktop")
    $Shortcut = $WshShell.CreateShortcut("$DesktopPath\Boombox RX-505.lnk")
    $Shortcut.TargetPath = "$InstallDir\RUN-BOOMBOX.bat"
    $Shortcut.WorkingDirectory = $InstallDir
    $Shortcut.IconLocation = "$InstallDir\boombox-rs.exe,0"
    $Shortcut.Description = "Boombox RX-505 Retro Music Player & Worldwide Radio"
    $Shortcut.Save()

    # Start Menu Shortcut
    $StartMenuPath = [Environment]::GetFolderPath("Programs")
    $StartShortcut = $WshShell.CreateShortcut("$StartMenuPath\Boombox RX-505.lnk")
    $StartShortcut.TargetPath = "$InstallDir\RUN-BOOMBOX.bat"
    $StartShortcut.WorkingDirectory = $InstallDir
    $StartShortcut.IconLocation = "$InstallDir\boombox-rs.exe,0"
    $StartShortcut.Description = "Boombox RX-505 Retro Music Player & Worldwide Radio"
    $StartShortcut.Save()
} catch {
    Write-Host "  ⚠️ Could not create desktop shortcuts automatically." -ForegroundColor Yellow
}

# ------------------------------------------------------------------------------
# Finished!
# ------------------------------------------------------------------------------
Write-Host ""
Write-Host "✨ BOOMBOX RX-505 has been successfully installed!" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Yellow
Write-Host "  ▶ Launch via Terminal:       boombox" -ForegroundColor Cyan
Write-Host "  ▶ Desktop Shortcut:          'Boombox RX-505' on your Desktop" -ForegroundColor Cyan
Write-Host "  ▶ Start Menu:                'Boombox RX-505' in Windows Search" -ForegroundColor Cyan
Write-Host "  ▶ Directory:                 $InstallDir" -ForegroundColor DarkGray
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Yellow
Write-Host ""
