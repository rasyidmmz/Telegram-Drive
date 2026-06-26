# PowerShell script to download MPV binary sidecar for local Tauri development
$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$binDir = Join-Path $PSScriptRoot "bin"
if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Path $binDir -Force
}

$targetFile = Join-Path $binDir "mpv-x86_64-pc-windows-msvc.exe"
if (Test-Path $targetFile) {
    Write-Host "MPV sidecar already exists: $targetFile"
    Exit 0
}

Write-Host "Downloading MPV archive..."
$tempZip = Join-Path $env:TEMP "mpv-temp.zip"

# Official stable zip release
$url = "https://github.com/mpv-player/mpv/releases/download/v0.41.0/mpv-v0.41.0-x86_64-pc-windows-msvc.zip"
curl.exe -L -o $tempZip $url

Write-Host "Extracting MPV binary..."
$tempExtractDir = Join-Path $env:TEMP "mpv-temp-extract"
if (Test-Path $tempExtractDir) {
    Remove-Item -Path $tempExtractDir -Recurse -Force
}
New-Item -ItemType Directory -Path $tempExtractDir -Force

Expand-Archive -Path $tempZip -DestinationPath $tempExtractDir -Force

$extractedMpv = Join-Path $tempExtractDir "mpv.exe"
if (-not (Test-Path $extractedMpv)) {
    $foundFile = Get-ChildItem -Path $tempExtractDir -Filter "mpv.exe" -Recurse | Select-Object -First 1
    if ($foundFile) {
        $extractedMpv = $foundFile.FullName
    } else {
        $extractedMpv = $null
    }
}

if ($extractedMpv) {
    Move-Item -Path $extractedMpv -Destination $targetFile -Force
    Write-Host "Successfully configured MPV sidecar: $targetFile"
} else {
    Write-Error "Could not find mpv.exe in the extracted files."
}

# Cleanup
Remove-Item -Path $tempZip -Force -ErrorAction SilentlyContinue
Remove-Item -Path $tempExtractDir -Recurse -Force -ErrorAction SilentlyContinue
