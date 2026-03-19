# Cipherz

Toolkit cipher blok kustom dengan dua implementasi: `C` sebagai dasar tingkat rendah yang bersih, dan `Rust` untuk CLI utama serta GUI desktop. Keduanya menjalankan desain cipher yang sama.

## Gambaran Umum

- ukuran blok `64-bit`
- ukuran kunci `128-bit`
- jaringan Feistel `8-round`
- fungsi ronde dengan `XOR`, nibble `S-Box`, rotasi, dan permutasi bit
- mode: `CBC`, `CFB`, `OFB`

## Komponen yang Tersedia

- CLI dalam `C`
- CLI dan GUI dalam `Rust`
- impor GUI untuk `.txt` dan `.md`
- ekspor GUI ke `.txt`
- alur benchmark dengan `CSV` dan dashboard `PNG`
- pengujian Rust yang menjaga kesetaraan output terhadap implementasi C

## Alasan Dua Implementasi

- `C` membuat cipher tetap mudah diperiksa pada level terendah.
- `Rust` adalah lapisan aplikasi utama: model memori lebih aman, ergonomi lebih rapi, dan mendukung GUI.

## Dokumentasi Implementasi

Untuk penjelasan implementasi yang lebih rinci, lihat [DOCUMENTATION.md](DOCUMENTATION.md). Dokumen tersebut menjabarkan detail cipher di level kode, terutama implementasi C pada `c/src/main.c`, alur CLI, mode operasi, dan perilaku GUI.

Kutipan ringkas dari dokumentasi:

> Implementasi ini memakai jaringan Feistel `64-bit`, fungsi ronde berbasis `XOR -> nibble S-Box -> rotasi/permutasi`, tiga mode operasi `CBC`, `CFB`, dan `OFB`, `PKCS#7` khusus untuk `CBC`, dukungan input CLI dari argumen atau `stdin`, serta opsi `--raw` untuk memproses ciphertext sebagai bytes mentah tanpa encoding hex.

File yang paling relevan untuk memahami implementasi:

- `DOCUMENTATION.md` untuk walkthrough detail per bagian kode.
- `c/src/main.c` untuk inti cipher, mode operasi, dan CLI implementasi `C`.
- `src/lib.rs` untuk implementasi cipher utama di `Rust`.
- `src/gpui_app.rs`, `src/gpui_app/app.rs`, dan `src/gpui_app/io.rs` untuk GUI desktop.
- `scripts/benchmark_metrics.py` untuk pipeline benchmark dan perbandingan performa.

## Instalasi

Instalasi standar:

Linux atau macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.sh | sh -s --
```

PowerShell Windows:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.ps1 -OutFile install.ps1
./install.ps1
```

Instalasi default tidak otomatis menjalankan aplikasi. Jika Anda ingin installer langsung membuka GUI setelah setup selesai, gunakan:

```bash
curl -fsSL https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.sh | sh -s -- --run-ui
```

Untuk PowerShell Windows, gunakan:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.ps1 -OutFile install.ps1
./install.ps1 -RunUI
```

## Kompilasi dan Eksekusi dari Kode Sumber

Clone repositori terlebih dahulu:

```bash
git clone https://github.com/fxrdhan/Cipherz.git
cd Cipherz
```

### Membangun Binary C

```bash
make
```

### Menjalankan Implementasi C

```bash
./block_cipher enc cbc "KAMSIS-KEY-2026!" "IV2026!!" "Firdaus Arif Ramadhani"
./block_cipher dec cbc "KAMSIS-KEY-2026!" "IV2026!!" "<ciphertext_hex>"
```

### Membangun Implementasi Rust

```bash
cargo build
```

### Menjalankan CLI Rust

```bash
cargo run --bin cipherz_cli -- enc cbc "KAMSIS-KEY-2026!" "IV2026!!" "Firdaus Arif Ramadhani"
cargo run --bin cipherz_cli -- dec cbc "KAMSIS-KEY-2026!" "IV2026!!" "<ciphertext_hex>"
```

### Menjalankan GUI Rust

```bash
cargo run --bin cipherz_gui
```

## Mode Operasi yang Didukung

Mode yang tersedia:

- `cbc`
- `cfb`
- `ofb`

## Benchmark dan Kinerja

Hasilkan benchmark dan dashboard perbandingan lengkap lewat skrip Python:

```bash
python3 scripts/benchmark_metrics.py
```

Benchmark memakai jalur `enc/dec --raw` agar ciphertext dipindahkan sebagai bytes mentah sehingga hasil `encrypt` dan `decrypt` tidak bias oleh overhead encoding hex.

| Skenario | C (MiB/s) | Rust (MiB/s) | Kenaikan Rust |
| --- | ---: | ---: | ---: |
| CBC Encrypt | 52.20 | 64.29 | 23.16% |
| CBC Decrypt | 55.80 | 66.96 | 20.00% |
| CFB Encrypt | 48.36 | 57.58 | 19.06% |
| CFB Decrypt | 53.35 | 68.99 | 29.32% |
| OFB Encrypt | 53.80 | 63.38 | 17.81% |
| OFB Decrypt | 55.56 | 65.16 | 17.28% |

Data terbaru memperlihatkan bahwa implementasi Rust tetap menjaga throughput yang lebih tinggi pada seluruh skenario utama, dengan uplift sekitar 17-29% dibanding implementasi C pada rentang ukuran data hingga `16 MiB`.

![Dashboard benchmark Cipherz](artifacts/benchmark/benchmark_dashboard.png)
