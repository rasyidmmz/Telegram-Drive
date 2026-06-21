$ErrorActionPreference = 'Stop'

function Get-Sha256([string]$Path) {
    $stream = [IO.File]::OpenRead($Path)
    $sha256 = [Security.Cryptography.SHA256]::Create()
    try {
        return [BitConverter]::ToString($sha256.ComputeHash($stream)).Replace('-', '')
    }
    finally {
        $sha256.Dispose()
        $stream.Dispose()
    }
}

$archiveUrl = 'https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20260610/mpv-x86_64-20260610-git-304426c.7z'
$archiveHash = 'FACAC536BAA73C7B925771AF5E39A3C9CB16B8D75B59A6E9800DE89799DFFCA7'
$executableHash = 'B0BB2DC1928E6D86CC26D950815C80C977440081E814C6A46E93F6E9E99C276D'
$target = Join-Path $PSScriptRoot '..\src-tauri\binaries\mpv-x86_64-pc-windows-msvc.exe'
$target = [IO.Path]::GetFullPath($target)

if (Test-Path -LiteralPath $target) {
    $currentHash = Get-Sha256 $target
    if ($currentHash -eq $executableHash) {
        Write-Host 'Pinned mpv sidecar is already prepared.'
        exit 0
    }
}

$sevenZip = (Get-Command 7z -ErrorAction SilentlyContinue).Source
if (-not $sevenZip) {
    $sevenZip = 'C:\Program Files\7-Zip\7z.exe'
}
if (-not (Test-Path -LiteralPath $sevenZip)) {
    throw '7-Zip is required to prepare the pinned mpv sidecar.'
}

$tempBase = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$tempRoot = Join-Path $tempBase ('telegram-drive-mpv-' + [Guid]::NewGuid().ToString('N'))
$archive = Join-Path $tempRoot 'mpv.7z'
$extracted = Join-Path $tempRoot 'mpv.exe'

try {
    New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null
    Invoke-WebRequest -Uri $archiveUrl -OutFile $archive

    $downloadedHash = Get-Sha256 $archive
    if ($downloadedHash -ne $archiveHash) {
        throw "mpv archive hash mismatch: $downloadedHash"
    }

    & $sevenZip e $archive mpv.exe "-o$tempRoot" -y | Out-Null
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $extracted)) {
        throw 'Failed to extract mpv.exe from the pinned archive.'
    }

    $actualExecutableHash = Get-Sha256 $extracted
    if ($actualExecutableHash -ne $executableHash) {
        throw "mpv executable hash mismatch: $actualExecutableHash"
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
    Move-Item -LiteralPath $extracted -Destination $target -Force
    Write-Host "Prepared pinned mpv sidecar at $target"
}
finally {
    $resolvedTempRoot = [IO.Path]::GetFullPath($tempRoot)
    if ($resolvedTempRoot.StartsWith($tempBase, [StringComparison]::OrdinalIgnoreCase) -and
        (Test-Path -LiteralPath $resolvedTempRoot)) {
        Remove-Item -LiteralPath $resolvedTempRoot -Recurse -Force
    }
}
