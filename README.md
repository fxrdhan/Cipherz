# OFB Cipher CLI in C

OFB Cipher CLI adalah program command line berbasis bahasa C untuk enkripsi dan dekripsi teks dengan mode operasi `Output Feedback` (`OFB`). Program memakai block cipher kustom berukuran blok 64-bit, kunci 128-bit, dan jaringan Feistel 8 ronde sebagai fungsi pembangkit keystream.

Mode `OFB` menghasilkan keystream dari `IV` melalui proses enkripsi block cipher. Keystream tersebut di-XOR dengan plaintext untuk menghasilkan ciphertext, atau di-XOR dengan ciphertext untuk menghasilkan plaintext. Proses enkripsi dan dekripsi memakai alur komputasi yang sama.

Untuk penjabaran kode `main.c` dan dokumentasi teknis yang lebih lengkap, lihat [`DOCUMENTATION.md`](DOCUMENTATION.md).

## Spesifikasi

| Komponen | Nilai |
| --- | --- |
| Bahasa | C |
| Versi C | C11 |
| Compiler default | `gcc` |
| Ukuran blok | 8 byte / 64 bit |
| Ukuran kunci | 16 byte / 128 bit |
| Ukuran IV | 8 byte / 64 bit |
| Jumlah ronde | 8 |
| Mode operasi | `OFB` |
| Format ciphertext | Hexadecimal uppercase |

## Operasi Dasar

Implementasi cipher memakai beberapa operasi dasar berikut:

- `XOR`: mencampur input dengan round key dan mencampur data dengan keystream.
- `S-Box`: mengganti setiap nibble 4-bit memakai tabel substitusi.
- `Rotasi bit`: memutar word 32-bit ke kiri untuk difusi bit.
- `Permutasi/difusi`: menggabungkan beberapa hasil rotasi dengan `XOR`.

Operasi `OFB` pada data:

```text
feedback_0 = IV
keystream_i = encrypt_block(feedback_i)
feedback_(i+1) = keystream_i
output_i = input_i XOR keystream_i
```

Karena `OFB` bekerja sebagai stream mode, panjang ciphertext sama dengan panjang plaintext. Padding tidak digunakan.

Contoh utama pada project ini memakai dua plaintext:

```text
Contoh 1 block:
plaintext = "arif"
block 1   = "arif"          (4 byte, block parsial)

Contoh 2 block:
plaintext = "firdaus arif"
block 1   = "firdaus "      (8 byte, block penuh)
block 2   = "arif"          (4 byte, block parsial)
```

## Build

```bash
make
```

Hasil build:

```text
./block_cipher
```

Membersihkan binary:

```bash
make clean
```

## Penggunaan

Enkripsi:

```bash
./block_cipher enc <key16> <iv8> <plaintext>
```

Dekripsi:

```bash
./block_cipher dec <key16> <iv8> <ciphertext_hex>
```

Contoh enkripsi 1 block:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! "arif"
```

Contoh output:

```text
332B5FA1
```

Contoh enkripsi 2 block:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! "firdaus arif"
```

Contoh output:

```text
343044A3CF442F1AB954B9C6
```

Contoh dekripsi:

```bash
./block_cipher dec KUNCI-1NGG121S ADA123!! 343044A3CF442F1AB954B9C6
```

Contoh output:

```text
firdaus arif
```

## Format Input

Argumen `key16` diproses sebagai byte teks. Maksimal 16 byte pertama dipakai sebagai kunci. Jika panjang kurang dari 16 byte, sisa byte kunci bernilai `0x00`.

Argumen `iv8` diproses sebagai byte teks. Maksimal 8 byte pertama dipakai sebagai IV. Jika panjang kurang dari 8 byte, sisa byte IV bernilai `0x00`.

Argumen `plaintext` diproses sebagai teks dari satu argumen command line. Teks berisi spasi perlu diapit kutip.

Argumen `ciphertext_hex` wajib berupa hexadecimal dengan jumlah karakter genap. Karakter hex huruf besar dan huruf kecil diterima.

## Format Output

Perintah `enc` menghasilkan ciphertext dalam hexadecimal uppercase dan newline.

Perintah `dec` menghasilkan plaintext hasil dekripsi dan newline.
