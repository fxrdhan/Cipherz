#!/usr/bin/env sh
set -eu

REPO_OWNER="fxrdhan"
REPO_NAME="Cipherz"
BRANCH="main"
INSTALL_DIR="Cipherz"
FORCE=0

usage() {
    cat <<'EOF'
Usage: sh install.sh [--branch <branch>] [--dir <install_dir>] [--force]

Downloads the latest project archive from GitHub without using git clone.

Options:
  --branch <branch>    GitHub branch to download (default: main)
  --dir <install_dir>  Target directory (default: Cipherz)
  --force              Overwrite target directory if it already exists
  -h, --help           Show this help message
EOF
}

need_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        printf 'Error: required command not found: %s\n' "$1" >&2
        exit 1
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

printf 'Downloading %s (%s)...\n' "$REPO_NAME" "$BRANCH"
curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"
tar -xzf "$ARCHIVE_PATH" -C "$EXTRACT_DIR"

set -- "$EXTRACT_DIR"/*

if [ "$#" -eq 0 ] || [ ! -d "$1" ]; then
    printf 'Error: downloaded archive did not contain a project directory.\n' >&2
    exit 1
fi

mv "$1" "$INSTALL_DIR"

TARGET_ABS="$(cd "$(dirname "$INSTALL_DIR")" && pwd)/$(basename "$INSTALL_DIR")"

printf 'Installed to %s\n' "$TARGET_ABS"
printf 'Next steps:\n'
printf '  cd %s\n' "$INSTALL_DIR"
printf '  cargo build\n'
