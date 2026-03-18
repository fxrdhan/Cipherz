# Tugas Block Cipher C dan Rust

Repo ini berisi implementasi block cipher edukatif yang dibuat tanpa library kriptografi siap pakai. Cipher yang dipakai adalah block cipher sederhana berbasis jaringan Feistel dengan:

- ukuran blok 64 bit (8 byte)
- ukuran kunci 128 bit (16 byte)
- 8 ronde
- fungsi dasar utama XOR dan substitusi S-Box
- fungsi tambahan rotasi dan permutasi bit pada round function

Mode operasi yang saat ini diimplementasikan:

- CBC
- CFB
- OFB

Implementasi ini tidak menyalin DES atau AES penuh, tetapi mengikuti konsep block cipher, mode operasi, Feistel network, XOR, dan substitusi yang dibahas di materi kuliah.

## Isi repo

Repo ini memuat dua implementasi yang setara secara inti algoritma:

- implementasi C CLI di `c/src/main.c`
- implementasi Rust CLI dan GUI di `src/lib.rs`, `src/main.rs`, dan `src/gpui_app.rs`
- tooling benchmark di `scripts/benchmark_metrics.py`
- artefak hasil benchmark di `artifacts/benchmark/`

Input yang didukung saat ini adalah teks langsung:

- argumen command line pada implementasi C dan Rust
- field teks pada GUI Rust

Input file `.txt` atau file biner belum diimplementasikan pada versi repo ini.

## Install Tanpa Git

Installer sekarang bisa:

- download source dari GitHub tanpa `git clone`
- build Rust project otomatis dalam mode `release`
- opsional build CLI C
- opsional langsung menjalankan GUI Rust

Untuk Linux atau macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.sh | sh
```

Untuk langsung build lalu menjalankan GUI:

```bash
curl -fsSL https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.sh | sh -s -- --run-ui
```

Untuk PowerShell:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.ps1 -OutFile install.ps1
./install.ps1
```

Untuk langsung build lalu menjalankan GUI di PowerShell:

```powershell
./install.ps1 -RunUI
```

Opsi yang didukung:

- `install.sh`: `--branch`, `--dir`, `--force`, `--source-only`, `--build-c`, `--run-ui`
- `install.ps1`: `-Branch`, `-InstallDir`, `-Force`, `-SourceOnly`, `-BuildC`, `-RunUI`

Catatan:

- installer akan mencoba memasang Rust toolchain via `rustup` jika `cargo` belum ada
- dependency GUI tingkat OS tetap harus tersedia; kalau kurang, installer akan berhenti dan menunjuk ke docs resmi Zed atau GPUI untuk OS terkait
- `--run-ui` butuh sesi grafis aktif, jadi tidak akan jalan di environment headless

## Kaitan dengan PPT

- konsep block cipher: slide 4
- mode operasi block cipher: slide 7-12
- jaringan Feistel: slide 15-17
- fungsi substitusi dan XOR ala DES: slide 25-30

## Build implementasi C

```bash
make
```

Binary yang dihasilkan:

```bash
./block_cipher
```

## Build implementasi Rust

```bash
cargo build
```

Menjalankan CLI Rust:

```bash
cargo run -- enc cbc "KAMSIS-KEY-2026!" "IV2026!!" "halo dunia"
```

Menjalankan GUI Rust:

```bash
cargo run -- ui
```

## Menjalankan manual

Enkripsi:

```bash
./block_cipher enc <mode> <key16> <iv8> <plaintext>
```

Dekripsi:

```bash
./block_cipher dec <mode> <key16> <iv8> <ciphertext_hex>
```

Contoh:

```bash
./block_cipher enc cbc "KAMSIS-KEY-2026!" "IV2026!!" "halo dunia"
./block_cipher dec cbc "KAMSIS-KEY-2026!" "IV2026!!" "<hex hasil enkripsi>"
```

Mode yang valid:

- `cbc`
- `cfb`
- `ofb`

Catatan:

- key diambil dari 16 karakter pertama
- IV diambil dari 8 karakter pertama
- CBC memakai padding PKCS#7
- CFB dan OFB diproses secara stream berbasis blok sehingga tidak memerlukan padding

## Benchmark

Benchmark bawaan pada CLI C:

```bash
./block_cipher bench
```

Benchmark bawaan pada CLI Rust:

```bash
cargo run -- bench
```

Untuk menghasilkan metrik komprehensif dan dashboard benchmark C vs Rust:

```bash
python3 scripts/benchmark_metrics.py
```

Script benchmark saat ini hanya mengukur mode yang benar-benar tersedia di kode:

- `CBC`
- `CFB`
- `OFB`

Output yang dihasilkan:

- `artifacts/benchmark/benchmark_results.csv`
- `artifacts/benchmark/benchmark_summary.csv`
- `artifacts/benchmark/benchmark_dashboard.png`

## Pengujian

Menjalankan test Rust:

```bash
cargo test
```

Test yang ada saat ini memverifikasi:

- kesesuaian ciphertext hasil implementasi Rust terhadap implementasi C
- validasi kegagalan padding untuk mode CBC

## Struktur program

- `encrypt_block()` dan `decrypt_block()` mengerjakan enkripsi dan dekripsi satu blok dengan Feistel
- `round_function()` memakai XOR, S-Box, dan rotasi atau permutasi bit
- `encrypt_message()` dan `decrypt_message()` menangani mode operasi
- `scripts/benchmark_metrics.py` membandingkan throughput dan latensi implementasi C dan Rust

## Catatan akademik

Cipher ini bersifat edukatif untuk memenuhi tugas kuliah. Implementasi ini bukan pengganti algoritma standar seperti AES untuk sistem produksi.
