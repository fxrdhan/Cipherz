#!/usr/bin/env sh
set -eu

REPO_OWNER="fxrdhan"
REPO_NAME="Cipherz"
BRANCH="main"
INSTALL_DIR="Cipherz"
FORCE=0
BUILD_RUST=1
BUILD_C=0
RUN_UI=0

usage() {
    cat <<'EOF'
Usage: sh install.sh [options]

Downloads the latest project archive from GitHub without using git clone.
By default it also builds the Rust project in release mode.

Options:
  --branch <branch>    GitHub branch to download (default: main)
  --dir <install_dir>  Target directory (default: Cipherz)
  --force              Overwrite target directory if it already exists
  --source-only        Download source only, skip all build steps
  --build-c            Also build the C CLI via make
  --run-ui             Launch the Rust GUI after the build succeeds
  -h, --help           Show this help message
EOF
}

need_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'Error: required command not found: %s\n' "$1" >&2
        exit 1
    fi
}

docs_url_for_os() {
    case "$(uname -s)" in
        Darwin)
            printf '%s\n' 'https://zed.dev/docs/development/macos'
            ;;
        Linux)
            printf '%s\n' 'https://zed.dev/docs/development/linux'
            ;;
        *)
            printf '%s\n' 'https://zed.dev/docs/development'
            ;;
    esac
}

normalize_arch() {
    case "$1" in
        x86_64|amd64)
            printf '%s\n' 'x86_64'
            ;;
        aarch64|arm64)
            printf '%s\n' 'aarch64'
            ;;
        *)
            printf '%s\n' "$1"
            ;;
    esac
}

prebuilt_asset_name() {
    os_name="$(uname -s)"
    arch_name="$(normalize_arch "$(uname -m)")"

    case "$os_name" in
        Linux)
            printf 'Cipherz-linux-%s.tar.gz\n' "$arch_name"
            ;;
        Darwin)
            printf 'Cipherz-macos-%s.tar.gz\n' "$arch_name"
            ;;
        *)
            return 1
            ;;
    esac
}

install_prebuilt_release() {
    asset_name="$(prebuilt_asset_name)" || return 1
    asset_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest/download/${asset_name}"
    asset_archive="$TMP_ROOT/$asset_name"
    prebuilt_extract="$TMP_ROOT/prebuilt"

    mkdir -p "$prebuilt_extract"

    printf 'Trying prebuilt release asset %s...\n' "$asset_name"
    if ! curl -fsSL "$asset_url" -o "$asset_archive"; then
        printf 'Prebuilt asset not available for this OS/arch. Falling back to source build.\n'
        return 1
    fi

    tar -xzf "$asset_archive" -C "$prebuilt_extract"
    set -- "$prebuilt_extract"/*
    if [ "$#" -eq 0 ] || [ ! -d "$1" ]; then
        printf 'Error: prebuilt archive did not contain an application directory.\n' >&2
        exit 1
    fi

    mv "$1" "$INSTALL_DIR"
    return 0
}

ensure_rust_toolchain() {
    if command -v cargo >/dev/null 2>&1 && command -v rustc >/dev/null 2>&1; then
        return
    fi

    printf 'Rust toolchain not found. Installing via rustup...\n'
    curl -fsSL https://sh.rustup.rs | sh -s -- -y

    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck disable=SC1090
        . "$HOME/.cargo/env"
    fi

    if ! command -v cargo >/dev/null 2>&1 || ! command -v rustc >/dev/null 2>&1; then
        printf 'Error: Rust toolchain installation finished but cargo/rustc are still unavailable.\n' >&2
        exit 1
    fi
}

ensure_gui_prereqs() {
    DOCS_URL="$(docs_url_for_os)"
    need_cmd cmake

    case "$(uname -s)" in
        Darwin)
            if ! xcode-select -p >/dev/null 2>&1; then
                printf 'Error: Xcode Command Line Tools are required to build the GUI.\n' >&2
                printf 'See: %s\n' "$DOCS_URL" >&2
                exit 1
            fi
            ;;
        Linux)
            need_cmd pkg-config
            if ! command -v cc >/dev/null 2>&1 \
                && ! command -v gcc >/dev/null 2>&1 \
                && ! command -v clang >/dev/null 2>&1; then
                printf 'Error: a C compiler is required to build the GUI on Linux.\n' >&2
                printf 'See: %s\n' "$DOCS_URL" >&2
                exit 1
            fi
            ;;
    esac
}

build_rust_project() {
    printf 'Building Rust project (release)...\n'
    (
        cd "$INSTALL_DIR"
        cargo build --release
    )
}

build_c_project() {
    need_cmd make

    if ! command -v cc >/dev/null 2>&1 \
        && ! command -v gcc >/dev/null 2>&1 \
        && ! command -v clang >/dev/null 2>&1; then
        printf 'Error: a C compiler is required to build the C CLI.\n' >&2
        exit 1
    fi

    printf 'Building C CLI...\n'
    (
        cd "$INSTALL_DIR"
        make
    )
}

run_gui_app() {
    case "$(uname -s)" in
        Linux)
            if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
                printf 'Error: no graphical session detected. Set DISPLAY or WAYLAND_DISPLAY before using --run-ui.\n' >&2
                exit 1
            fi
            ;;
    esac

    printf 'Launching GUI app...\n'
    if [ -x "$INSTALL_DIR/cipherz_gui" ]; then
        (
            cd "$INSTALL_DIR"
            ./cipherz_gui
        )
    else
        (
            cd "$INSTALL_DIR"
            cargo run --release -- ui
        )
    fi
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --branch)
            [ "$#" -ge 2 ] || {
                printf 'Error: missing value for %s\n' "$1" >&2
                exit 1
            }
            BRANCH="$2"
            shift 2
            ;;
        --dir)
            [ "$#" -ge 2 ] || {
                printf 'Error: missing value for %s\n' "$1" >&2
                exit 1
            }
            INSTALL_DIR="$2"
            shift 2
            ;;
        --force)
            FORCE=1
            shift
            ;;
        --source-only)
            BUILD_RUST=0
            BUILD_C=0
            RUN_UI=0
            shift
            ;;
        --build-c)
            BUILD_C=1
            shift
            ;;
        --run-ui)
            RUN_UI=1
            BUILD_RUST=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'Error: unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

need_cmd curl
need_cmd tar
need_cmd mktemp
need_cmd dirname
need_cmd basename
need_cmd pwd

TMP_ROOT="$(mktemp -d 2>/dev/null || mktemp -d -t cipherz-install)"
ARCHIVE_PATH="$TMP_ROOT/repo.tar.gz"
EXTRACT_DIR="$TMP_ROOT/extract"
TARGET_PARENT="$(dirname "$INSTALL_DIR")"

cleanup() {
    rm -rf "$TMP_ROOT"
}

trap cleanup EXIT INT TERM

mkdir -p "$EXTRACT_DIR" "$TARGET_PARENT"

if [ -e "$INSTALL_DIR" ]; then
    if [ "$FORCE" -ne 1 ]; then
        printf 'Error: target already exists: %s\n' "$INSTALL_DIR" >&2
        printf 'Use --force to overwrite it.\n' >&2
        exit 1
    fi
    rm -rf "$INSTALL_DIR"
fi

DOWNLOAD_URL="https://codeload.github.com/${REPO_OWNER}/${REPO_NAME}/tar.gz/refs/heads/${BRANCH}"

USED_PREBUILT=0
if [ "$BRANCH" = "main" ] && [ "$BUILD_C" -eq 0 ] && [ "$BUILD_RUST" -eq 1 ]; then
    if install_prebuilt_release; then
        USED_PREBUILT=1
    fi
fi

if [ "$USED_PREBUILT" -eq 0 ]; then
    printf 'Downloading %s (%s)...\n' "$REPO_NAME" "$BRANCH"
    curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"
    tar -xzf "$ARCHIVE_PATH" -C "$EXTRACT_DIR"

    set -- "$EXTRACT_DIR"/*

    if [ "$#" -eq 0 ] || [ ! -d "$1" ]; then
        printf 'Error: downloaded archive did not contain a project directory.\n' >&2
        exit 1
    fi

    mv "$1" "$INSTALL_DIR"
fi

TARGET_ABS="$(cd "$(dirname "$INSTALL_DIR")" && pwd)/$(basename "$INSTALL_DIR")"

printf 'Installed to %s\n' "$TARGET_ABS"

if [ "$USED_PREBUILT" -eq 0 ] && { [ "$BUILD_RUST" -eq 1 ] || [ "$RUN_UI" -eq 1 ]; }; then
    ensure_rust_toolchain
    ensure_gui_prereqs
    build_rust_project
fi

if [ "$USED_PREBUILT" -eq 0 ] && [ "$BUILD_C" -eq 1 ]; then
    build_c_project
fi

if [ "$RUN_UI" -eq 1 ]; then
    run_gui_app
    exit 0
fi

printf 'Next steps:\n'
printf '  cd %s\n' "$INSTALL_DIR"
if [ "$USED_PREBUILT" -eq 1 ]; then
    printf '  ./cipherz_gui\n'
else
    printf '  cargo run -- ui\n'
fi
