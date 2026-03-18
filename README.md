# Cipherz

Custom block cipher toolkit with two implementations: `C` for a clean low-level baseline, and `Rust` for the main CLI and desktop GUI. Both run the same cipher design, and in the current benchmark set Rust is also the faster implementation.

## Core

- `64-bit` block size
- `128-bit` key size
- `8-round` Feistel network
- round function with `XOR`, nibble `S-Box`, rotation, and bit permutation
- modes: `CBC`, `CFB`, `OFB`

## Included

- CLI in `C`
- CLI and GUI in `Rust`
- GUI import for `.txt` and `.md`
- GUI export to `.txt`
- benchmark pipeline with `CSV` and dashboard `PNG`
- Rust tests that lock output parity against the C implementation

## Why Two Implementations?

- `C` keeps the cipher easy to inspect at the lowest level.
- `Rust` is the primary app layer: safer memory model, cleaner ergonomics, and GUI support.
- Keeping both makes regression checks and performance comparisons straightforward.

## Install

Standard install:

Linux or macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.sh | sh
```

Windows PowerShell:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.ps1 -OutFile install.ps1
./install.ps1
```

Install and launch the GUI in one step:

```bash
curl -fsSL https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.sh | sh -s -- --run-ui
```

```powershell
./install.ps1 -RunUI
```

Default install does not auto-launch the app. Use `--run-ui` or `-RunUI` when you want the installer to open the GUI immediately after setup.

## Build From Source

Build the C binary:

```bash
make
```

Run C:

```bash
./block_cipher enc cbc "KAMSIS-KEY-2026!" "IV2026!!" "halo dunia"
./block_cipher dec cbc "KAMSIS-KEY-2026!" "IV2026!!" "<ciphertext_hex>"
```

Build Rust:

```bash
cargo build
```

Run Rust CLI:

```bash
cargo run -- enc cbc "KAMSIS-KEY-2026!" "IV2026!!" "halo dunia"
cargo run -- dec cbc "KAMSIS-KEY-2026!" "IV2026!!" "<ciphertext_hex>"
```

Run Rust GUI:

```bash
cargo run -- ui
```

Valid modes:

- `cbc`
- `cfb`
- `ofb`

## Benchmark

Run the built-in benchmark:

```bash
./block_cipher bench
cargo run -- bench
```

Generate the full comparison dashboard:

```bash
python3 scripts/benchmark_metrics.py
```

Across the committed benchmark set, Rust leads in every tested mode and operation. Average throughput is about `1.44x` higher overall, and the `4 MiB` runs below show a `1.17x` to `1.47x` advantage depending on mode.

![Cipherz benchmark dashboard](artifacts/benchmark/benchmark_dashboard.png)

### 4 MiB Snapshot

| Mode | Operation | C (MiB/s) | Rust (MiB/s) | Rust/C | C (ms) | Rust (ms) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| CBC | Encrypt | 116.94 | 167.61 | 1.43x | 34.204 | 23.866 |
| CBC | Decrypt | 140.99 | 207.02 | 1.47x | 28.371 | 19.322 |
| CFB | Encrypt | 119.13 | 139.71 | 1.17x | 33.578 | 28.631 |
| CFB | Decrypt | 144.19 | 195.05 | 1.35x | 27.741 | 20.508 |
| OFB | Encrypt | 126.31 | 166.21 | 1.32x | 31.668 | 24.066 |
| OFB | Decrypt | 137.39 | 181.59 | 1.32x | 29.114 | 22.028 |

## Test

```bash
cargo test
```
