#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import shutil
import stat
import tarfile
import tempfile
import zipfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Package prebuilt Cipherz binaries into a release archive."
    )
    parser.add_argument("--platform", required=True, choices=["linux", "macos", "windows"])
    parser.add_argument("--arch", required=True)
    parser.add_argument("--release-dir", required=True)
    parser.add_argument("--output-dir", required=True)
    return parser.parse_args()


def make_executable(path: Path) -> None:
    mode = path.stat().st_mode
    path.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def write_text(path: Path, content: str) -> None:
    path.write_text(content, encoding="utf-8", newline="\n")


def create_launcher_files(bundle_dir: Path, platform: str) -> None:
    if platform == "windows":
        write_text(
            bundle_dir / "run-gui.bat",
            "@echo off\r\nstart \"\" \"%~dp0cipherz_gui.exe\"\r\n",
        )
        write_text(
            bundle_dir / "run-cli.bat",
            "@echo off\r\n\"%~dp0cipherz_cli.exe\" %*\r\n",
        )
        return

    write_text(
        bundle_dir / "run-gui.sh",
        "#!/usr/bin/env sh\nset -eu\nDIR=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\nexec \"$DIR/cipherz_gui\" \"$@\"\n",
    )
    write_text(
        bundle_dir / "run-cli.sh",
        "#!/usr/bin/env sh\nset -eu\nDIR=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\nexec \"$DIR/cipherz_cli\" \"$@\"\n",
    )
    make_executable(bundle_dir / "run-gui.sh")
    make_executable(bundle_dir / "run-cli.sh")


def package_archive(bundle_dir: Path, platform: str, output_path: Path) -> Path:
    if platform == "windows":
        with zipfile.ZipFile(output_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for file_path in bundle_dir.rglob("*"):
                archive.write(file_path, file_path.relative_to(bundle_dir.parent))
        return output_path

    with tarfile.open(output_path, "w:gz") as archive:
        archive.add(bundle_dir, arcname=bundle_dir.name)
    return output_path


def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[1]
    release_dir = Path(args.release_dir).resolve()
    output_dir = Path(args.output_dir).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    binary_suffix = ".exe" if args.platform == "windows" else ""
    binaries = [
        ("cipherz_gui", release_dir / f"cipherz_gui{binary_suffix}"),
        ("cipherz_cli", release_dir / f"cipherz_cli{binary_suffix}"),
    ]

    missing = [name for name, path in binaries if not path.exists()]
    if missing:
        raise FileNotFoundError(f"missing release binaries: {', '.join(missing)}")

    archive_ext = "zip" if args.platform == "windows" else "tar.gz"
    archive_name = f"Cipherz-{args.platform}-{args.arch}.{archive_ext}"
    output_path = output_dir / archive_name

    with tempfile.TemporaryDirectory(prefix="cipherz-package-") as tmp_dir:
        tmp_root = Path(tmp_dir)
        bundle_dir = tmp_root / f"Cipherz-{args.platform}-{args.arch}"
        bundle_dir.mkdir(parents=True, exist_ok=True)

        for _, source_path in binaries:
            target_path = bundle_dir / source_path.name
            shutil.copy2(source_path, target_path)
            if args.platform != "windows":
                make_executable(target_path)

        shutil.copy2(repo_root / "README.md", bundle_dir / "README.md")
        create_launcher_files(bundle_dir, args.platform)
        package_archive(bundle_dir, args.platform, output_path)

    print(output_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
