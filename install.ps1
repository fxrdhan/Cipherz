param(
    [string]$Branch = "main",
    [string]$InstallDir = "Cipherz",
    [switch]$Force,
    [switch]$SourceOnly,
    [switch]$BuildC,
    [switch]$RunUI
)

$ErrorActionPreference = "Stop"

$RepoOwner = "fxrdhan"
$RepoName = "Cipherz"
$DownloadUrl = "https://github.com/$RepoOwner/$RepoName/archive/refs/heads/$Branch.zip"
$PathSeparator = [System.IO.Path]::PathSeparator

function Get-DocsUrl {
    if ($IsWindows) {
        return "https://zed.dev/docs/development/windows"
    }

    if ($IsMacOS) {
        return "https://zed.dev/docs/development/macos"
    }

    if ($IsLinux) {
        return "https://zed.dev/docs/development/linux"
    }

    return "https://zed.dev/docs/development"
}

function Add-CargoToPath {
    $cargoBin = Join-Path $HOME ".cargo/bin"
    if (Test-Path $cargoBin) {
        $env:PATH = "$cargoBin$PathSeparator$env:PATH"
    }
}

function Ensure-RustToolchain {
    if ((Get-Command cargo -ErrorAction SilentlyContinue) -and (Get-Command rustc -ErrorAction SilentlyContinue)) {
        return
    }

    Write-Host "Rust toolchain not found. Installing via rustup..."

    if ($IsWindows) {
        $rustupInstaller = Join-Path $tempRoot "rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInstaller
        & $rustupInstaller -y | Out-Host
    }
    else {
        $rustupScript = Join-Path $tempRoot "rustup-init.sh"
        Invoke-WebRequest -Uri "https://sh.rustup.rs" -OutFile $rustupScript
        & sh $rustupScript -y | Out-Host
    }

    Add-CargoToPath

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -or -not (Get-Command rustc -ErrorAction SilentlyContinue)) {
        throw "Rust toolchain installation finished but cargo/rustc are still unavailable."
    }
}

function Assert-Command {
    param([string]$Name)

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command not found: $Name"
    }
}

function Ensure-GuiPrereqs {
    $docsUrl = Get-DocsUrl
    Assert-Command cmake

    if ($IsMacOS) {
        if (-not (Get-Command xcode-select -ErrorAction SilentlyContinue)) {
            throw "xcode-select was not found. See: $docsUrl"
        }

        & xcode-select -p | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "Xcode Command Line Tools are required to build the GUI. See: $docsUrl"
        }
    }

    if ($IsLinux) {
        Assert-Command pkg-config
        if (-not (Get-Command cc -ErrorAction SilentlyContinue) `
            -and -not (Get-Command gcc -ErrorAction SilentlyContinue) `
            -and -not (Get-Command clang -ErrorAction SilentlyContinue)) {
            throw "A C compiler is required to build the GUI on Linux. See: $docsUrl"
        }
    }

    if ($IsWindows) {
        if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue) `
            -and -not (Get-Command clang.exe -ErrorAction SilentlyContinue)) {
            Write-Warning "No MSVC/Clang compiler was found in PATH. Cargo build may fail. See: $docsUrl"
        }
    }
}

function Build-RustProject {
    Write-Host "Building Rust project (release)..."
    Push-Location $resolvedInstallDir
    try {
        & cargo build --release
    }
    finally {
        Pop-Location
    }
}

function Build-CProject {
    Assert-Command make

    if (-not (Get-Command cc -ErrorAction SilentlyContinue) `
        -and -not (Get-Command gcc -ErrorAction SilentlyContinue) `
        -and -not (Get-Command clang -ErrorAction SilentlyContinue)) {
        throw "A C compiler is required to build the C CLI."
    }

    Write-Host "Building C CLI..."
    Push-Location $resolvedInstallDir
    try {
        & make
    }
    finally {
        Pop-Location
    }
}

function Run-GuiApp {
    if ($IsLinux -and -not $env:DISPLAY -and -not $env:WAYLAND_DISPLAY) {
        throw "No graphical session detected. Set DISPLAY or WAYLAND_DISPLAY before using -RunUI."
    }

    Write-Host "Launching GUI app..."
    Push-Location $resolvedInstallDir
    try {
        & cargo run --release -- ui
    }
    finally {
        Pop-Location
    }
}

$tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("cipherz-install-" + [Guid]::NewGuid().ToString("N"))
$archivePath = Join-Path $tempRoot "repo.zip"
$extractPath = Join-Path $tempRoot "extract"

New-Item -ItemType Directory -Force -Path $tempRoot | Out-Null
New-Item -ItemType Directory -Force -Path $extractPath | Out-Null

if ($RunUI) {
    $SourceOnly = $false
}

try {
    if ([System.IO.Path]::IsPathRooted($InstallDir)) {
        $resolvedInstallDir = $InstallDir
    }
    else {
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

    if (-not $SourceOnly) {
        Add-CargoToPath
        Ensure-RustToolchain
        Ensure-GuiPrereqs
        Build-RustProject
    }

    if ($BuildC) {
        Build-CProject
    }

    if ($RunUI) {
        Run-GuiApp
        exit 0
    }

    Write-Host "Next steps:"
    Write-Host "  Set-Location '$resolvedInstallDir'"
    Write-Host "  cargo run -- ui"
}
finally {
    if (Test-Path $tempRoot) {
        Remove-Item -Recurse -Force $tempRoot
    }
}
