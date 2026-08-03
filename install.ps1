# QRShare Windows Installer - Production Edition
# Auto-detects architecture, downloads release, verifies checksum, extracts, and updates user PATH.

$ErrorActionPreference = "Stop"

# Repository details
$repo = "bharadwajsanket/QRShare"

Write-Host ""
Write-Host "  ██████╗  ██████╗  ███████╗██╗  ██╗  █████╗  ██████╗  ███████╗" -ForegroundColor Cyan
Write-Host " ██╔═══██╗ ██╔══██╗ ██╔════╝██║  ██║ ██╔══██╗ ██╔══██╗ ██╔════╝" -ForegroundColor Cyan
Write-Host " ██║   ██║ ██████╔╝ ███████╗███████║ ███████║ ██████╔╝ █████╗  " -ForegroundColor Cyan
Write-Host " ██║ ▄ ██║ ██╔══██╗ ╚════██║██╔══██║ ██╔══██║ ██╔══██╗ ██╔══╝  " -ForegroundColor Cyan
Write-Host " ╚██████╔╝ ██║  ██║ ███████║██║  ██║ ██║  ██║ ██║  ██║ ███████╗" -ForegroundColor Cyan
Write-Host "  ╚═══██╔╝  ╚═╝  ╚═╝ ╚══════╝╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚══════╝" -ForegroundColor Cyan
Write-Host ""
Write-Host "         QRShare Windows Installer — Production Edition" -ForegroundColor Cyan -BackgroundColor Black
Write-Host "         ==============================================" -ForegroundColor Cyan
Write-Host ""

# 1. Check OS Version
if (-not [System.Environment]::OSVersion.VersionString.Contains("Windows")) {
    Write-Error "This installer supports Windows systems only."
}

# 2. Detect Architecture
Write-Host "Detecting host architecture..." -ForegroundColor Gray
$arch = "x86_64"
if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64" -or $env:PROCESSOR_ARCHITEW6432 -eq "ARM64") {
    $arch = "arm64"
    Write-Host "-> Architecture: Windows ARM64" -ForegroundColor Green
} else {
    Write-Host "-> Architecture: Windows x64 (x86_64)" -ForegroundColor Green
}

# 3. Resolve Version Tag
Write-Host "Fetching latest version tag..." -ForegroundColor Gray
$latestTag = "v1.5.4" # Fallback
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $releasesUrl = "https://api.github.com/repos/$repo/releases/latest"
    $response = Invoke-RestMethod -Uri $releasesUrl -Headers @{ "User-Agent" = "QRShare-Installer" } -TimeoutSec 10
    if ($response.tag_name) {
        $latestTag = $response.tag_name
        Write-Host "-> Target version resolved: $latestTag" -ForegroundColor Green
    }
} catch {
    Write-Host "-> Connection to GitHub API failed. Defaulting to: $latestTag" -ForegroundColor Yellow
}

# 4. Configure Paths
$binName = "qrshare-windows-$arch.zip"
$downloadUrl = "https://github.com/$repo/releases/download/$latestTag/$binName"
$checksumUrl = "https://github.com/$repo/releases/download/$latestTag/SHA256SUMS"

$tmpDir = Join-Path $env:TEMP "qrshare-install-$(New-Guid)"
$null = New-Item -ItemType Directory -Path $tmpDir -Force

$zipPath = Join-Path $tmpDir $binName
$checksumPath = Join-Path $tmpDir "SHA256SUMS"

# 5. Download Release Zip & Checksums
try {
    Write-Host "Downloading release archive..." -ForegroundColor Gray
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing

    Write-Host "Downloading release checksums..." -ForegroundColor Gray
    Invoke-WebRequest -Uri $checksumUrl -OutFile $checksumPath -UseBasicParsing
} catch {
    Remove-Item -Recururse -Path $tmpDir -ErrorAction SilentlyContinue
    Write-Error "Failed to download release assets. Please check your internet connection."
}

# 6. Verify Checksum
Write-Host "Verifying archive hash integrity..." -ForegroundColor Gray
try {
    # Extract expected hash from SHA256SUMS
    $expectedHashLine = Get-Content $checksumPath | Select-String -Pattern $binName
    if (-not $expectedHashLine) {
        throw "Checksum for $binName not found in SHA256SUMS."
    }
    $expectedHash = ($expectedHashLine -split "\s+")[0].Trim().ToUpper()

    # Calculate actual hash
    $actualHash = (Get-FileHash -Path $zipPath -Algorithm SHA256).Hash.ToUpper()

    if ($expectedHash -ne $actualHash) {
        throw "Checksum mismatch. Expected: $expectedHash, Got: $actualHash"
    }
    Write-Host "-> Hash verification passed (SHA256 match)" -ForegroundColor Green
} catch {
    Remove-Item -Recurse -Path $tmpDir -ErrorAction SilentlyContinue
    Write-Error "Verification FAILED: $_"
}

# 7. Extract Executable
Write-Host "Extracting release binary..." -ForegroundColor Gray
try {
    Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force
    Write-Host "-> Extraction complete" -ForegroundColor Green
} catch {
    Remove-Item -Recurse -Path $tmpDir -ErrorAction SilentlyContinue
    Write-Error "Failed to extract ZIP archive: $_"
}

# 8. Install Executable
$installDir = Join-Path $env:USERPROFILE ".qrshare\bin"
$destPath = Join-Path $installDir "qrshare.exe"

Write-Host "Installing executable to $destPath..." -ForegroundColor Gray
try {
    $null = New-Item -ItemType Directory -Path $installDir -Force
    Copy-Item -Path (Join-Path $tmpDir "qrshare.exe") -Destination $destPath -Force
    Write-Host "-> Binary installed successfully" -ForegroundColor Green
} catch {
    Remove-Item -Recurse -Path $tmpDir -ErrorAction SilentlyContinue
    Write-Error "Failed to copy binary to destination: $_"
}

# 9. Verify Installation
Write-Host "Verifying target execution compatibility..." -ForegroundColor Gray
try {
    $helpCheck = Start-Process -FilePath $destPath -ArgumentList "--help" -NoNewWindow -PassThru -Wait
    if ($helpCheck.ExitCode -ne 0) {
        throw "Verification process returned exit code $($helpCheck.ExitCode)"
    }
    Write-Host "-> Execution validation passed" -ForegroundColor Green
} catch {
    Remove-Item -Recurse -Path $tmpDir -ErrorAction SilentlyContinue
    Write-Error "Binary verification failed. The executable is not running on this host system."
}

# Cleanup Temp folder
Remove-Item -Recurse -Path $tmpDir -ErrorAction SilentlyContinue

# 10. Update Environment PATH
Write-Host "Checking PATH environment configuration..." -ForegroundColor Gray
$pathUpdated = $false
try {
    $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -split ";" -notcontains $installDir) {
        # Update user-level path variable
        $newUserPath = $userPath.TrimEnd(';') + ";" + $installDir
        [System.Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
        
        # Also update current session's path variable
        $env:Path += ";" + $installDir
        $pathUpdated = $true
        Write-Host "-> User PATH variable updated" -ForegroundColor Green
    } else {
        Write-Host "-> Directory already in PATH" -ForegroundColor Green
    }
} catch {
    Write-Host "Warning: Failed to update PATH environment variable: $_" -ForegroundColor Yellow
}

# 11. Completion Summary
Write-Host ""
Write-Host "┌────────────────────────────────────────────────────────┐" -ForegroundColor Green
Write-Host "│ Installation Completed Successfully!                  │" -ForegroundColor Green
Write-Host "├────────────────────────────────────────────────────────┤" -ForegroundColor Green
Write-Host "│ Binary Path:  $($destPath.PadRight(40)) │" -ForegroundColor Green
Write-Host "│ Version:      $($latestTag.PadRight(40)) │" -ForegroundColor Green
Write-Host "│ Platform:     Windows ($($arch.PadRight(32))) │" -ForegroundColor Green
Write-Host "└────────────────────────────────────────────────────────┘" -ForegroundColor Green
Write-Host ""

if ($pathUpdated) {
    Write-Host "⚠️  Notice: Added $installDir to your User PATH variable." -ForegroundColor Yellow
    Write-Host "Please restart your terminal or shell session to enable 'qrshare' commands." -ForegroundColor Cyan
} else {
    Write-Host "You can now run 'qrshare --help' in any terminal to verify command bindings." -ForegroundColor Cyan
}
Write-Host ""
