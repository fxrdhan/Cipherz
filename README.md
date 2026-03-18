# Cipherz

Toolkit cipher blok kustom dengan dua implementasi: `C` sebagai dasar tingkat rendah yang bersih, dan `Rust` untuk CLI utama serta GUI desktop. Keduanya menjalankan desain cipher yang sama, dan pada kumpulan benchmark saat ini Rust juga menjadi implementasi yang lebih cepat.

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

## Instalasi

Instalasi standar:

Linux atau macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.sh | sh
```

PowerShell Windows:

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/fxrdhan/Cipherz/main/install.ps1 -OutFile install.ps1
./install.ps1
```

Instalasi default tidak otomatis menjalankan aplikasi. Gunakan `--run-ui` atau `-RunUI` jika Anda ingin installer langsung membuka GUI setelah proses setup selesai.

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
./block_cipher enc cbc "KAMSIS-KEY-2026!" "IV2026!!" "halo dunia"
./block_cipher dec cbc "KAMSIS-KEY-2026!" "IV2026!!" "<ciphertext_hex>"
```

### Membangun Implementasi Rust

```bash
cargo build
```

### Menjalankan CLI Rust

```bash
cargo run -- enc cbc "KAMSIS-KEY-2026!" "IV2026!!" "halo dunia"
cargo run -- dec cbc "KAMSIS-KEY-2026!" "IV2026!!" "<ciphertext_hex>"
```

### Menjalankan GUI Rust

```bash
cargo run -- ui
```

## Mode Operasi yang Didukung

Mode yang tersedia:

- `cbc`
- `cfb`
- `ofb`

## Benchmark dan Kinerja

Jalankan benchmark bawaan:

```bash
./block_cipher bench
cargo run -- bench
```

Hasilkan dashboard perbandingan lengkap:

```bash
python3 scripts/benchmark_metrics.py
```

Rust unggul di setiap mode dan operasi yang diuji. Throughput rata-rata secara keseluruhan sekitar `1.44x` lebih tinggi, dan pengujian `4 MiB` di bawah menunjukkan keunggulan `1.17x` hingga `1.47x` tergantung modenya.

![Dashboard benchmark Cipherz](artifacts/benchmark/benchmark_dashboard.png)

### Ringkasan Hasil 4 MiB

| Mode | Operasi | C (MiB/s) | Rust (MiB/s) | Rust/C | C (ms) | Rust (ms) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| CBC | Enkripsi | 116.94 | 167.61 | 1.43x | 34.204 | 23.866 |
| CBC | Dekripsi | 140.99 | 207.02 | 1.47x | 28.371 | 19.322 |
| CFB | Enkripsi | 119.13 | 139.71 | 1.17x | 33.578 | 28.631 |
| CFB | Dekripsi | 144.19 | 195.05 | 1.35x | 27.741 | 20.508 |
| OFB | Enkripsi | 126.31 | 166.21 | 1.32x | 31.668 | 24.066 |
| OFB | Dekripsi | 137.39 | 181.59 | 1.32x | 29.114 | 22.028 |
