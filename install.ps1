# ==============================================================================
#  BOOMBOX-RS: Retro Cyberpunk Cassette Boombox & Worldwide Radio Explorer
#  Automated One-Line Installer for Windows (PowerShell)
#  Repository: https://github.com/dannie203/tui-radio
# ==============================================================================

$ErrorActionPreference = "Stop"

$Repo = "dannie203/tui-radio"
$InstallDir = "$env:LOCALAPPDATA\Programs\Boombox"
$ZipUrl = "https://github.com/$Repo/releases/latest/download/boombox-rs-windows-x86_64.zip"

Write-Host ""
Write-Host "  ____   ____   ____  __  __ ____   ______  __" -ForegroundColor Yellow
Write-Host " | __ ) / __ \ / __ \|  \/  | __ ) / __ \ \/ /" -ForegroundColor Yellow
Write-Host " |  _ \| |  | | |  | | |\/| |  _ \| |  | |\  / " -ForegroundColor Yellow
Write-Host " | |_) | |__| | |__| | |  | | |_) | |__| |/  \ " -ForegroundColor Yellow
Write-Host " |____/ \____/ \____/|_|  |_|____/ \____//_/\_\ RX-505" -ForegroundColor Yellow
Write-Host " 📼 Automated Installer for Windows" -ForegroundColor Cyan
Write-Host ""

# 1. Create Installation Directory
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

# 2. Download and Extract Latest Windows Release
$TempZip = "$env:TEMP\boombox-release-$([guid]::NewGuid().ToString().Substring(0,8)).zip"
$TempExtract = "$env:TEMP\boombox-extract-$([guid]::NewGuid().ToString().Substring(0,8))"

try {
    Write-Host "📥 Downloading latest Boombox-RS from GitHub..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $ZipUrl -OutFile $TempZip -UseBasicParsing

    Write-Host "📦 Extracting executable..." -ForegroundColor Cyan
    Expand-Archive -Path $TempZip -DestinationPath $TempExtract -Force
    Copy-Item "$TempExtract\boombox-rs.exe" -Destination "$InstallDir\boombox-rs.exe" -Force
}
finally {
    # 3. Clean up all temporary files immediately
    if (Test-Path $TempZip) { Remove-Item $TempZip -Force -ErrorAction SilentlyContinue }
    if (Test-Path $TempExtract) { Remove-Item $TempExtract -Recurse -Force -ErrorAction SilentlyContinue }
}

# 4. Check / Download yt-dlp.exe portable helper if missing
$YtDlpPath = "$InstallDir\yt-dlp.exe"
if (!(Get-Command "yt-dlp" -ErrorAction SilentlyContinue) -and !(Test-Path $YtDlpPath)) {
    Write-Host "📥 Downloading portable yt-dlp stream helper..." -ForegroundColor Cyan
    try {
        Invoke-WebRequest -Uri "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe" -OutFile $YtDlpPath -UseBasicParsing
    } catch {
        Write-Host "⚠️  Could not auto-download yt-dlp. You can install it via: winget install yt-dlp" -ForegroundColor Yellow
    }
}

# 5. Check MPV dependency
if (!(Get-Command "mpv" -ErrorAction SilentlyContinue) -and !(Test-Path "$InstallDir\mpv.exe")) {
    Write-Host "💡 Notice: 'mpv' was not detected in PATH." -ForegroundColor Yellow
    Write-Host "   You can install it easily with: winget install mpv.net" -ForegroundColor Cyan
}

# 6. Add InstallDir to User PATH Environment Variable
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "⚙️ Adding $InstallDir to User PATH..." -ForegroundColor Cyan
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
}

# 7. Create Start Menu and Desktop Shortcuts
$WshShell = New-Object -ComObject WScript.Shell

# Desktop Shortcut
$DesktopPath = [Environment]::GetFolderPath("Desktop")
$Shortcut = $WshShell.CreateShortcut("$DesktopPath\Boombox RX-505.lnk")
$Shortcut.TargetPath = "$InstallDir\boombox-rs.exe"
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "Boombox RX-505 Retro Music Player & Radio"
$Shortcut.Save()

# Start Menu Shortcut
$StartMenuPath = [Environment]::GetFolderPath("Programs")
$StartShortcut = $WshShell.CreateShortcut("$StartMenuPath\Boombox RX-505.lnk")
$StartShortcut.TargetPath = "$InstallDir\boombox-rs.exe"
$StartShortcut.WorkingDirectory = $InstallDir
$StartShortcut.Description = "Boombox RX-505 Retro Music Player & Radio"
$StartShortcut.Save()

Write-Host ""
Write-Host "✨ BOOMBOX RX-505 installed successfully!" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Yellow
Write-Host "  ▶ Launch via Command Prompt / PowerShell:  boombox-rs" -ForegroundColor Cyan
Write-Host "  ▶ Desktop Shortcut:                        'Boombox RX-505' on your Desktop" -ForegroundColor Cyan
Write-Host "  ▶ Start Menu:                              'Boombox RX-505' in Windows Search" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Yellow
Write-Host ""
