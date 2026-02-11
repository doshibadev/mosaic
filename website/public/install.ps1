# Mosaic Installer (Windows PowerShell)

$ErrorActionPreference = 'Stop'

# Define installation paths
$InstallDir = "$env:LOCALAPPDATA\Mosaic\bin"
$ExeName = "mosaic.exe"
$Repo = "doshibadev/mosaic"
$AssetName = "mosaic-windows-amd64.exe"

Write-Host "Installing Mosaic for Windows..."

# Create directory
if (-not (Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Download latest release
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$AssetName"
$OutputPath = Join-Path $InstallDir $ExeName

Write-Host "Downloading from $DownloadUrl..."
try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $OutputPath
}
catch {
    Write-Error "Failed to download Mosaic. Please check your internet connection or if the release exists."
    exit 1
}

# Add to PATH (User Environment Variable)
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Host "Added $InstallDir to your User PATH."
    Write-Host "You may need to restart your terminal for changes to take effect."
} else {
    Write-Host "Path already configured."
}

Write-Host ""
Write-Host "Mosaic installed successfully!" -ForegroundColor Green
Write-Host "Run 'mosaic --help' to get started."
