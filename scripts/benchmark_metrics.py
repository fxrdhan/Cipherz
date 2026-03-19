#!/usr/bin/env python3
import argparse
import csv
import shutil
import statistics
import subprocess
import sys
import time
from collections import defaultdict
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.lines import Line2D


REPO_ROOT = Path(__file__).resolve().parents[1]
C_ROOT = REPO_ROOT / "c"
RUST_ROOT = REPO_ROOT
BUILD_ROOT = REPO_ROOT / ".benchmark_build"
C_BINARY_PATH = BUILD_ROOT / "block_cipher_c"
RUST_TARGET_DIR = BUILD_ROOT / "rust_target"
ARTIFACT_ROOT = REPO_ROOT / "artifacts" / "benchmark"
CSV_PATH = ARTIFACT_ROOT / "benchmark_results.csv"
SUMMARY_PATH = ARTIFACT_ROOT / "benchmark_summary.csv"
PNG_PATH = ARTIFACT_ROOT / "benchmark_dashboard.png"
KEY_TEXT = "KAMSIS-KEY-2026!"
IV_TEXT = "IV2026!!"
PLAINTEXT_PATTERN = b"Firdaus Arif Ramadhani | "

DEFAULT_SIZES = [
    1024,
    4 * 1024,
    16 * 1024,
    64 * 1024,
    256 * 1024,
    1024 * 1024,
    4 * 1024 * 1024,
    16 * 1024 * 1024,
    256 * 1024 * 1024,
]

MODES = ["cbc", "cfb", "ofb"]
OPERATIONS = ["encrypt", "decrypt"]
COLORS = {
    "cbc": "#1d4ed8",
    "cfb": "#dc2626",
    "ofb": "#16a34a",
}
IMPLEMENTATIONS = {
    "c": {
        "label": "C",
        "root": C_ROOT,
        "executable": C_BINARY_PATH,
        "linestyle": "-",
        "marker": "o",
    },
    "rust": {
        "label": "Rust",
        "root": RUST_ROOT,
        "executable": RUST_TARGET_DIR / "release" / "cipherz_cli",
        "linestyle": "--",
        "marker": "s",
    },
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Benchmark block cipher C, Rust, atau keduanya."
    )
    parser.add_argument(
        "implementation",
        nargs="?",
        choices=["c", "rust", "both"],
        default="both",
        help="implementasi yang dibenchmark (default: both)",
    )
    parser.add_argument(
        "--samples",
        type=int,
        default=1,
        help="jumlah sampel per ukuran data (default: 1)",
    )
    parser.add_argument(
        "--target-mib",
        type=int,
        default=8,
        help="target total data per ukuran dalam MiB (default: 8)",
    )
    parser.add_argument(
        "--max-iterations",
        type=int,
        default=8,
        help="batas maksimum iterasi per ukuran (default: 8)",
    )
    return parser.parse_args()


def selected_implementations(choice: str) -> list[str]:
    if choice == "both":
        return ["c", "rust"]
    return [choice]


def ensure_binary(implementation: str) -> None:
    spec = IMPLEMENTATIONS[implementation]
    BUILD_ROOT.mkdir(exist_ok=True)
    ARTIFACT_ROOT.mkdir(parents=True, exist_ok=True)

    if implementation == "c":
        tool = "gcc"
        build_cmd = [
            tool,
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-pedantic",
            "-O2",
            "-o",
            str(C_BINARY_PATH),
            str(C_ROOT / "src" / "main.c"),
        ]
        cwd = REPO_ROOT
    else:
        tool = "cargo"
        build_cmd = [
            tool,
            "build",
            "--release",
            "--target-dir",
            str(RUST_TARGET_DIR),
            "--bin",
            "cipherz_cli",
        ]
        cwd = RUST_ROOT

    if shutil.which(tool) is None:
        raise RuntimeError(
            f"{tool} tidak ditemukan untuk build implementasi {implementation}."
        )

    subprocess.run(build_cmd, cwd=cwd, check=True)

    executable = spec["executable"]
    if not executable.exists():
        raise RuntimeError(
            f"Executable untuk implementasi {implementation} tidak ditemukan: {executable}"
        )


def choose_iterations(size_bytes: int, target_mib: int, max_iterations: int) -> int:
    target_total_bytes = target_mib * 1024 * 1024
    iterations = max(1, target_total_bytes // size_bytes)
    return min(iterations, max_iterations)


def generate_plaintext(size_bytes: int) -> bytes:
    repeats = (size_bytes + len(PLAINTEXT_PATTERN) - 1) // len(PLAINTEXT_PATTERN)
    return (PLAINTEXT_PATTERN * repeats)[:size_bytes]


def run_cli_command(
    implementation: str,
    args: list[str],
    stdin_bytes: bytes,
    trim_text_output: bool = False,
) -> tuple[float, bytes]:
    spec = IMPLEMENTATIONS[implementation]
    started_at = time.perf_counter()
    result = subprocess.run(
        [str(spec["executable"]), *args],
        cwd=spec["root"],
        check=True,
        input=stdin_bytes,
        capture_output=True,
    )
    elapsed = time.perf_counter() - started_at
    output = result.stdout.rstrip(b"\r\n") if trim_text_output else result.stdout
    return elapsed, output


def benchmark_mode(
    implementation: str,
    mode: str,
    plaintext: bytes,
    iterations: int,
) -> list[dict[str, str]]:
    enc_total_seconds = 0.0
    dec_total_seconds = 0.0
    last_ciphertext = b""

    for _ in range(iterations):
        elapsed, last_ciphertext = run_cli_command(
            implementation,
            ["enc", mode, KEY_TEXT, IV_TEXT, "-", "--raw"],
            plaintext,
        )
        enc_total_seconds += elapsed

    last_plaintext = b""
    for _ in range(iterations):
        elapsed, last_plaintext = run_cli_command(
            implementation,
            ["dec", mode, KEY_TEXT, IV_TEXT, "-", "--raw"],
            last_ciphertext,
        )
        dec_total_seconds += elapsed

    if last_plaintext != plaintext:
        raise RuntimeError(
            f"Verifikasi benchmark gagal untuk implementasi {implementation} mode {mode}."
        )

    total_bytes = len(plaintext) * iterations
    enc_throughput = total_bytes / (1024.0 * 1024.0 * enc_total_seconds)
    dec_throughput = total_bytes / (1024.0 * 1024.0 * dec_total_seconds)

    return [
        {
            "mode": mode,
            "operation": "encrypt",
            "data_bytes": str(len(plaintext)),
            "iterations": str(iterations),
            "total_seconds": f"{enc_total_seconds:.6f}",
            "throughput_mib_s": f"{enc_throughput:.6f}",
            "avg_ms_per_iteration": f"{(enc_total_seconds * 1000.0) / iterations:.6f}",
        },
        {
            "mode": mode,
            "operation": "decrypt",
            "data_bytes": str(len(plaintext)),
            "iterations": str(iterations),
            "total_seconds": f"{dec_total_seconds:.6f}",
            "throughput_mib_s": f"{dec_throughput:.6f}",
            "avg_ms_per_iteration": f"{(dec_total_seconds * 1000.0) / iterations:.6f}",
        },
    ]


def collect_rows(
    implementations: list[str],
    sizes: list[int],
    samples: int,
    target_mib: int,
    max_iterations: int,
) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for implementation in implementations:
        print(f"\n== Benchmark {IMPLEMENTATIONS[implementation]['label']} ==")
        for size_bytes in sizes:
            iterations = choose_iterations(size_bytes, target_mib, max_iterations)
            plaintext = generate_plaintext(size_bytes)
            for sample in range(1, samples + 1):
                print(
                    f"{implementation} | {size_bytes} bytes x {iterations} iterations | sample {sample}/{samples}"
                )
                for mode in MODES:
                    sample_rows = benchmark_mode(
                        implementation,
                        mode,
                        plaintext,
                        iterations,
                    )
                    for row in sample_rows:
                        row["implementation"] = implementation
                        row["sample"] = str(sample)
                        rows.append(row)
    return rows


def write_csv(rows: list[dict[str, str]]) -> None:
    fieldnames = [
        "implementation",
        "sample",
        "mode",
        "operation",
        "data_bytes",
        "iterations",
        "total_seconds",
        "throughput_mib_s",
        "avg_ms_per_iteration",
    ]
    with CSV_PATH.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def aggregate_rows(
    rows: list[dict[str, str]],
    implementations: list[str],
    sizes: list[int],
) -> list[dict[str, float | int | str]]:
    grouped: dict[tuple[str, str, str, int], list[dict[str, str]]] = defaultdict(list)
    for row in rows:
        key = (
            row["implementation"],
            row["mode"],
            row["operation"],
            int(row["data_bytes"]),
        )
        grouped[key].append(row)

    summary: list[dict[str, float | int | str]] = []
    for implementation in implementations:
        for mode in MODES:
            for operation in OPERATIONS:
                for size_bytes in sizes:
                    samples = grouped[(implementation, mode, operation, size_bytes)]
                    throughput_values = [
                        float(item["throughput_mib_s"]) for item in samples
                    ]
                    total_values = [float(item["total_seconds"]) for item in samples]
                    latency_values = [
                        float(item["avg_ms_per_iteration"]) for item in samples
                    ]
                    iterations = int(samples[0]["iterations"])
                    summary.append(
                        {
                            "implementation": implementation,
                            "mode": mode,
                            "operation": operation,
                            "data_bytes": size_bytes,
                            "iterations": iterations,
                            "samples": len(samples),
                            "throughput_mean": statistics.fmean(throughput_values),
                            "throughput_stddev": statistics.stdev(throughput_values)
                            if len(throughput_values) > 1
                            else 0.0,
                            "throughput_min": min(throughput_values),
                            "throughput_max": max(throughput_values),
                            "total_seconds_mean": statistics.fmean(total_values),
                            "latency_ms_mean": statistics.fmean(latency_values),
                            "latency_ms_stddev": statistics.stdev(latency_values)
                            if len(latency_values) > 1
                            else 0.0,
                        }
                    )
    return summary


def write_summary(summary: list[dict[str, float | int | str]]) -> None:
    fieldnames = [
        "implementation",
        "mode",
        "operation",
        "data_bytes",
        "iterations",
        "samples",
        "throughput_mean",
        "throughput_stddev",
        "throughput_min",
        "throughput_max",
        "total_seconds_mean",
        "latency_ms_mean",
        "latency_ms_stddev",
    ]
    with SUMMARY_PATH.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(summary)


def format_size_label(size_bytes: int) -> str:
    if size_bytes >= 1024 * 1024:
        return f"{size_bytes // (1024 * 1024)} MiB"
    if size_bytes >= 1024:
        return f"{size_bytes // 1024} KiB"
    return f"{size_bytes} B"


def extract_metric(
    summary: list[dict[str, float | int | str]],
    implementation: str,
    operation: str,
    metric_key: str,
    spread_key: str,
) -> dict[str, tuple[list[int], list[float], list[float]]]:
    metric: dict[str, tuple[list[int], list[float], list[float]]] = {}
    for mode in MODES:
        rows = [
            row
            for row in summary
            if row["implementation"] == implementation
            and row["mode"] == mode
            and row["operation"] == operation
        ]
        rows.sort(key=lambda row: int(row["data_bytes"]))
        sizes = [int(row["data_bytes"]) for row in rows]
        means = [float(row[metric_key]) for row in rows]
        spreads = [float(row[spread_key]) for row in rows]
        metric[mode] = (sizes, means, spreads)
    return metric


def setup_axis(ax, title: str, ylabel: str, sizes: list[int]) -> None:
    ax.set_title(title, fontsize=14, weight="bold", pad=10)
    ax.set_xscale("log", base=2)
    ax.set_xticks(sizes)
    ax.set_xticklabels([format_size_label(size) for size in sizes], rotation=0)
    ax.set_xlabel("Data size per iteration")
    ax.set_ylabel(ylabel)
    ax.grid(True, which="major", color="#dbe3ef", linewidth=0.8)
    ax.grid(True, which="minor", axis="y", color="#eef2f7", linewidth=0.5)
    ax.set_facecolor("#fbfdff")


def render_dashboard(
    summary: list[dict[str, float | int | str]],
    implementations: list[str],
    sizes: list[int],
    samples: int,
) -> None:
    plt.style.use("seaborn-v0_8-whitegrid")
    fig, axes = plt.subplots(2, 2, figsize=(18, 11), dpi=180)
    fig.subplots_adjust(
        top=0.80, bottom=0.08, left=0.06, right=0.98, hspace=0.24, wspace=0.10
    )

    if len(implementations) == 1:
        title = (
            f"Block Cipher Benchmark - {IMPLEMENTATIONS[implementations[0]]['label']}"
        )
    else:
        title = "Block Cipher Benchmark - C vs Rust"
    fig.suptitle(title, fontsize=22, fontweight="bold", y=0.975)

    charts = [
        (
            "encrypt",
            "throughput_mean",
            "throughput_stddev",
            "Encryption Throughput",
            "MiB/s",
        ),
        (
            "decrypt",
            "throughput_mean",
            "throughput_stddev",
            "Decryption Throughput",
            "MiB/s",
        ),
        (
            "encrypt",
            "latency_ms_mean",
            "latency_ms_stddev",
            "Encryption Latency",
            "ms / iteration",
        ),
        (
            "decrypt",
            "latency_ms_mean",
            "latency_ms_stddev",
            "Decryption Latency",
            "ms / iteration",
        ),
    ]

    for ax, (operation, metric_key, spread_key, title_text, ylabel) in zip(
        axes.flatten(), charts
    ):
        setup_axis(ax, title_text, ylabel, sizes)
        for implementation in implementations:
            metric = extract_metric(
                summary, implementation, operation, metric_key, spread_key
            )
            spec = IMPLEMENTATIONS[implementation]
            for mode in MODES:
                mode_sizes, means, spreads = metric[mode]
                lower = [
                    max(0.0, mean - spread) for mean, spread in zip(means, spreads)
                ]
                upper = [mean + spread for mean, spread in zip(means, spreads)]
                ax.plot(
                    mode_sizes,
                    means,
                    marker=spec["marker"],
                    markersize=5,
                    linewidth=2.2,
                    linestyle=spec["linestyle"],
                    color=COLORS[mode],
                )
                ax.fill_between(
                    mode_sizes, lower, upper, color=COLORS[mode], alpha=0.06
                )

    mode_handles = [
        Line2D(
            [0], [0], color=COLORS[mode], linewidth=2.2, marker="o", label=mode.upper()
        )
        for mode in MODES
    ]
    mode_legend = fig.legend(
        handles=mode_handles,
        loc="upper center",
        ncol=len(MODES),
        frameon=False,
        bbox_to_anchor=(0.28, 0.935),
        fontsize=11,
        title="Mode",
    )
    fig.add_artist(mode_legend)

    impl_handles = [
        Line2D(
            [0],
            [0],
            color="#111827",
            linewidth=2.2,
            linestyle=IMPLEMENTATIONS[item]["linestyle"],
            marker=IMPLEMENTATIONS[item]["marker"],
            label=IMPLEMENTATIONS[item]["label"],
        )
        for item in implementations
    ]
    fig.legend(
        handles=impl_handles,
        loc="upper center",
        ncol=len(implementations),
        frameon=False,
        bbox_to_anchor=(0.77, 0.935),
        fontsize=11,
        title="Implementation",
    )

    fig.savefig(PNG_PATH, dpi=180)
    plt.close(fig)


def print_summary(
    summary: list[dict[str, float | int | str]],
    implementations: list[str],
) -> None:
    print("\nRingkasan throughput rata-rata (MiB/s):")
    for implementation in implementations:
        label = IMPLEMENTATIONS[implementation]["label"]
        print(f"- {label}")
        for mode in MODES:
            enc_values = [
                float(row["throughput_mean"])
                for row in summary
                if row["implementation"] == implementation
                and row["mode"] == mode
                and row["operation"] == "encrypt"
            ]
            dec_values = [
                float(row["throughput_mean"])
                for row in summary
                if row["implementation"] == implementation
                and row["mode"] == mode
                and row["operation"] == "decrypt"
            ]
            print(
                f"  {mode.upper()}: enc avg {statistics.fmean(enc_values):.2f}, dec avg {statistics.fmean(dec_values):.2f}"
            )


def main() -> int:
    args = parse_args()
    if args.samples <= 0:
        sys.stderr.write("Error: --samples harus lebih dari 0\n")
        return 1
    if args.target_mib <= 0:
        sys.stderr.write("Error: --target-mib harus lebih dari 0\n")
        return 1
    if args.max_iterations <= 0:
        sys.stderr.write("Error: --max-iterations harus lebih dari 0\n")
        return 1

    implementations = selected_implementations(args.implementation)
    sizes = DEFAULT_SIZES

    try:
        for implementation in implementations:
            ensure_binary(implementation)

        rows = collect_rows(
            implementations,
            sizes,
            args.samples,
            args.target_mib,
            args.max_iterations,
        )
        write_csv(rows)
        summary = aggregate_rows(rows, implementations, sizes)
        write_summary(summary)
        render_dashboard(summary, implementations, sizes, args.samples)
        print_summary(summary, implementations)
        print(f"\nCSV sample   : {CSV_PATH}")
        print(f"CSV summary  : {SUMMARY_PATH}")
        print(f"PNG dashboard: {PNG_PATH}")
        return 0
    except subprocess.CalledProcessError as exc:
        if exc.stdout:
            sys.stderr.write(exc.stdout.decode("utf-8", errors="replace"))
        if exc.stderr:
            sys.stderr.write(exc.stderr.decode("utf-8", errors="replace"))
        return exc.returncode
    except Exception as exc:
        sys.stderr.write(f"Error: {exc}\n")
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
