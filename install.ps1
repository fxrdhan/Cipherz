param(
    [string]$Branch = "main",
    [string]$InstallDir = "Cipherz",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$RepoOwner = "fxrdhan"
$RepoName = "Cipherz"
$DownloadUrl = "https://github.com/$RepoOwner/$RepoName/archive/refs/heads/$Branch.zip"

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("cipherz-install-" + [Guid]::NewGuid().ToString("N"))
$archivePath = Join-Path $tempRoot "repo.zip"
$extractPath = Join-Path $tempRoot "extract"

New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null
New-Item -ItemType Directory -Force -Path $extractPath | Out-Null

try {
    if ([System.IO.Path]::IsPathRooted($InstallDir)) {
        $resolvedInstallDir = $InstallDir
    } else {
        $resolvedInstallDir = Join-Path (Get-Location) $InstallDir
    }

    if (Test-Path $resolvedInstallDir) {
        if (-not $Force) {
            throw "Target already exists: $resolvedInstallDir. Use -Force to overwrite it."
        }

        Remove-Item -Recurse -Force $resolvedInstallDir
    }

    $parentDir = Split-Path -Parent $resolvedInstallDir
    if ($parentDir) {
        New-Item -ItemType Directory -Force -Path $parentDir | Out-Null
    }

    Write-Host "Downloading $RepoName ($Branch)..."
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $archivePath
    Expand-Archive -Path $archivePath -DestinationPath $extractPath -Force

    $sourceDir = Get-ChildItem -Path $extractPath -Directory | Select-Object -First 1
    if (-not $sourceDir) {
        throw "Downloaded archive did not contain a project directory."
    }

    Move-Item -Path $sourceDir.FullName -Destination $resolvedInstallDir

    Write-Host "Installed to $resolvedInstallDir"
    Write-Host "Next steps:"
    Write-Host "  Set-Location '$resolvedInstallDir'"
    Write-Host "  cargo build"
}
finally {
    if (Test-Path $tempRoot) {
        Remove-Item -Recurse -Force $tempRoot
    }
}
