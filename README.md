# OFB Cipher CLI in C

OFB Cipher CLI adalah program command line berbasis bahasa C untuk enkripsi dan dekripsi teks dengan mode operasi `Output Feedback` (`OFB`). Program memakai block cipher kustom berukuran blok 64-bit, kunci 128-bit, dan jaringan Feistel 8 ronde sebagai fungsi pembangkit keystream.

Mode `OFB` menghasilkan keystream dari `IV` melalui proses enkripsi block cipher. Keystream tersebut di-XOR dengan plaintext untuk menghasilkan ciphertext, atau di-XOR dengan ciphertext untuk menghasilkan plaintext. Proses enkripsi dan dekripsi memakai alur komputasi yang sama.

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

Contoh enkripsi:

```bash
./block_cipher enc KAMSIS-KEY-2026 IV2026!! "halo"
```

Contoh output:

```text
7CBBFA5F
```

Contoh dekripsi:

```bash
./block_cipher dec KAMSIS-KEY-2026 IV2026!! 7CBBFA5F
```

Contoh output:

```text
halo
```

## Format Input

Argumen `key16` diproses sebagai byte teks. Maksimal 16 byte pertama dipakai sebagai kunci. Jika panjang kurang dari 16 byte, sisa byte kunci bernilai `0x00`.

Argumen `iv8` diproses sebagai byte teks. Maksimal 8 byte pertama dipakai sebagai IV. Jika panjang kurang dari 8 byte, sisa byte IV bernilai `0x00`.

Argumen `plaintext` diproses sebagai teks dari satu argumen command line. Teks berisi spasi perlu diapit kutip.

Argumen `ciphertext_hex` wajib berupa hexadecimal dengan jumlah karakter genap. Karakter hex huruf besar dan huruf kecil diterima.

## Format Output

Perintah `enc` menghasilkan ciphertext dalam hexadecimal uppercase dan newline.

Perintah `dec` menghasilkan plaintext hasil dekripsi dan newline.

