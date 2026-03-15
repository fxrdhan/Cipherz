# Tugas Block Cipher C

Implementasi ini dibuat tanpa library kriptografi. Cipher yang dipakai adalah cipher blok sederhana berbasis jaringan Feistel dengan:

- ukuran blok 64 bit (8 byte)
- ukuran kunci 128 bit (16 byte)
- 8 ronde
- dua fungsi dasar utama: XOR dan substitusi S-Box
- fungsi tambahan rotasi/permutasi bit pada round function

Mode operasi yang tersedia:

- CBC
- CFB
- OFB
- CTR

Empat mode operasi yang dipakai adalah `CBC`, `CFB`, `OFB`, dan `CTR`.

## Kaitan dengan PPT

- konsep block cipher: slide 4
- mode operasi block cipher: slide 7-12
- jaringan Feistel: slide 15-17
- fungsi substitusi dan XOR ala DES: slide 25-30

Implementasi ini tidak menyalin DES atau AES penuh, tetapi mengikuti ide yang dijelaskan di slide: data dibagi per blok, diproses per ronde, memakai XOR, substitusi, dan perubahan posisi bit.

## Build

```bash
make
```

## Menjalankan demo

```bash
./block_cipher demo
```

Demo akan menampilkan hasil enkripsi dan dekripsi untuk semua mode yang dipakai.

## Menjalankan benchmark

```bash
./block_cipher bench
```

Benchmark akan mengukur performa enkripsi dan dekripsi untuk `CBC`, `CFB`, `OFB`, dan `CTR`.

Untuk menghasilkan metrik komprehensif dan line chart dalam Python:

```bash
python3 benchmark_metrics.py
```

Output yang dihasilkan:

- `benchmark_results.csv`
- `benchmark_summary.csv`
- `benchmark_dashboard.png`

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

Catatan:

- key memakai 16 karakter pertama
- IV/nonce memakai 8 karakter pertama
- CBC memakai padding PKCS#7
- CFB, OFB, dan CTR diproses sebagai stream berbasis blok sehingga tidak memerlukan padding

## Struktur program

- `encrypt_block()` dan `decrypt_block()` mengerjakan enkripsi/dekripsi satu blok dengan Feistel
- `round_function()` memakai XOR, S-Box, dan rotasi bit
- `encrypt_message()` dan `decrypt_message()` menangani mode operasi

## Catatan akademik

Cipher ini bersifat edukatif untuk memenuhi tugas kuliah. Ini bukan pengganti algoritma standar seperti AES untuk sistem produksi.
