# Dokumentasi Teknis

Dokumen ini menjelaskan program `OFB Cipher CLI` dari awal sampai akhir. Fokusnya bukan hanya cara menjalankan program, tetapi juga cara membaca kode C-nya, bagaimana data bergerak di memori, bagaimana block cipher menghasilkan keystream, dan bagaimana mode `OFB` memakai keystream tersebut untuk enkripsi dan dekripsi.

Program utama berada di file `main.c`. Proses build diatur oleh `Makefile`. Binary hasil build bernama `block_cipher`.

## Tujuan Proyek

Tujuan proyek ini adalah membuat program command line sederhana untuk menunjukkan cara kerja enkripsi dan dekripsi menggunakan mode `Output Feedback` (`OFB`) di bahasa C.

Secara khusus, proyek ini bertujuan untuk:

- Mengimplementasikan block cipher kustom sebagai pembangkit keystream.
- Menunjukkan bagaimana mode `OFB` memakai `key` dan `IV` untuk menghasilkan keystream.
- Menunjukkan bahwa enkripsi dan dekripsi pada `OFB` dapat memakai fungsi yang sama karena sama-sama berbasis operasi `XOR`.
- Melatih pemrosesan data byte di C, termasuk buffer tetap, pointer, alokasi heap, dan konversi hexadecimal.
- Menyediakan contoh uji yang bisa dijalankan langsung melalui command line.

Ruang lingkup proyek:

- Input diproses dari argumen command line.
- Plaintext diperlakukan sebagai string teks biasa.
- Ciphertext enkripsi ditampilkan sebagai hexadecimal.
- Program dibuat untuk pembelajaran, bukan untuk keamanan produksi.

Dengan kata lain, fokus utama proyek bukan membuat algoritma kriptografi baru yang siap dipakai secara nyata, melainkan memahami alur teknis block cipher, mode operasi `OFB`, operasi bit, dan pengelolaan data di C.

## Ringkasan Sistem

Program ini menyediakan enkripsi dan dekripsi teks melalui command line. Mode operasi yang dipakai adalah `Output Feedback` atau `OFB`.

Secara sederhana, program bekerja seperti ini:

```text
key + IV -> block cipher -> keystream
input    -> XOR dengan keystream -> output
```

Pada enkripsi:

```text
plaintext XOR keystream = ciphertext
```

Pada dekripsi:

```text
ciphertext XOR keystream = plaintext
```

Karena operasi XOR bisa dibalik dengan XOR yang sama, fungsi `ofb_crypt` dapat dipakai untuk enkripsi dan dekripsi.

Karakteristik program:

| Komponen | Nilai |
| --- | --- |
| Bahasa | C |
| Standar C | C11 |
| Compiler default | `gcc` |
| Binary | `block_cipher` |
| Mode operasi | `OFB` |
| Ukuran blok | 8 byte / 64 bit |
| Ukuran key | 16 byte / 128 bit |
| Ukuran IV | 8 byte / 64 bit |
| Jumlah ronde Feistel | 8 |
| Output enkripsi | Hexadecimal uppercase |
| Input dekripsi | Hexadecimal valid dengan panjang genap |
| Padding | Tidak ada |
| Input file | Tidak ada |
| Output file | Tidak ada |
| Input `stdin` | Tidak ada |

Catatan penting:

- Program ini bersifat edukatif.
- Cipher yang dipakai adalah cipher kustom, bukan standar kriptografi resmi.
- Program tidak menyediakan autentikasi atau pengecekan integritas.
- Program tidak ditujukan sebagai pengganti pustaka kriptografi produksi.

## Struktur Proyek

Isi proyek yang relevan:

```text
main.c
Makefile
README.md
DOCUMENTATION.md
```

Peran file:

| File | Peran |
| --- | --- |
| `main.c` | Implementasi cipher, mode OFB, parser argumen, parser hex, dan fungsi `main` |
| `Makefile` | Instruksi build dan clean |
| `README.md` | Ringkasan singkat penggunaan |
| `DOCUMENTATION.md` | Dokumentasi teknis lengkap |

Program tidak memecah kode ke banyak file. Semua fungsi C berada di `main.c`, sehingga alur program dapat ditelusuri dari atas ke bawah dalam satu file.

## Istilah Dasar

Bagian ini menjelaskan istilah yang sering muncul di dokumen.

| Istilah | Arti |
| --- | --- |
| Bit | Unit data terkecil, bernilai `0` atau `1` |
| Byte | 8 bit |
| Nibble | 4 bit, setengah byte |
| Word 32-bit | Nilai integer berukuran 32 bit atau 4 byte |
| Block | Unit data tetap yang diproses block cipher |
| Key | Data rahasia yang memengaruhi round key dan keystream |
| IV | Initialization Vector, nilai awal feedback pada mode OFB |
| Keystream | Deretan byte hasil block cipher yang di-XOR dengan input |
| Feedback | State OFB yang menjadi input block cipher pada iterasi berikutnya |
| Hexadecimal | Penulisan angka basis 16 dengan karakter `0-9` dan `A-F` |

### `uint32_t`, `uint8_t`, dan `unsigned`

Tipe seperti `uint32_t` berasal dari header:

```c
#include <stdint.h>
```

Cara membaca `uint32_t`:

```text
u    = unsigned
int  = integer
32   = 32 bit
_t   = type
```

Jadi `uint32_t` berarti:

```text
unsigned integer 32-bit type
```

atau:

```text
tipe bilangan bulat tanpa tanda dengan ukuran tepat 32 bit
```

Perbandingan:

| Tipe | Arti |
| --- | --- |
| `uint8_t` | unsigned integer tepat 8 bit |
| `uint32_t` | unsigned integer tepat 32 bit |
| `int32_t` | signed integer tepat 32 bit |
| `unsigned` | keyword C untuk unsigned int |

`unsigned` adalah keyword standar C. Jika ditulis sendiri:

```c
unsigned shift;
```

maka artinya sama dengan:

```c
unsigned int shift;
```

`uint` berbeda dari `unsigned`. `uint` bukan tipe standar C. Biasanya `uint` hanya ada jika suatu project atau library membuat alias sendiri, misalnya:

```c
typedef unsigned int uint;
```

Di program ini, tipe yang dipakai adalah tipe standar seperti `uint8_t`, `uint32_t`, dan `unsigned`.

## Build

Build dilakukan dengan:

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

Compiler dan opsi build ditentukan di `Makefile`. Penjelasan detail `Makefile` ada di bagian akhir dokumen.

## Antarmuka Command Line

Format enkripsi:

```bash
./block_cipher enc <key16> <iv8> <plaintext>
```

Format dekripsi:

```bash
./block_cipher dec <key16> <iv8> <ciphertext_hex>
```

Catatan nama argumen:

- `key16` berarti program memakai buffer key internal berukuran 16 byte.
- `iv8` berarti program memakai buffer IV internal berukuran 8 byte.
- Nama tersebut tidak berarti program menolak input yang panjangnya tidak tepat 16 atau 8 byte.
- Jika input lebih pendek, sisa buffer diisi `0x00`.
- Jika input lebih panjang, byte setelah batas ukuran akan diabaikan.

Urutan argumen:

| Indeks | Isi |
| --- | --- |
| `argv[0]` | Nama program |
| `argv[1]` | Operasi: `enc` atau `dec` |
| `argv[2]` | Key teks |
| `argv[3]` | IV teks |
| `argv[4]` | Plaintext atau ciphertext hex |

Jumlah argumen harus tepat `5`. Jika tidak, program mencetak usage dan keluar dengan kode `1`.

Contoh enkripsi plaintext 2 block `firdaus arif`:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! "firdaus arif"
```

Program membaca:

| Argumen | Nilai | Makna |
| --- | --- | --- |
| `argv[1]` | `enc` | Operasi enkripsi |
| `argv[2]` | `KUNCI-1NGG121S` | Key |
| `argv[3]` | `ADA123!!` | IV |
| `argv[4]` | `firdaus arif` | Plaintext 12 byte / 2 block |

Jika plaintext memiliki spasi, plaintext perlu diapit tanda kutip agar shell mengirimnya sebagai satu argumen.

## Format Data

### Key

Argumen `key16` dibaca sebagai string C, lalu disalin ke buffer `key` berukuran 16 byte.

Nama `key16` dipakai untuk menandai ukuran buffer internal. Program tidak melakukan validasi bahwa key harus tepat 16 karakter. Key pendek tetap diterima dan dipad dengan `0x00`, sedangkan key panjang tetap diterima tetapi hanya 16 byte pertama yang dipakai.

Aturan:

- Maksimal 16 byte pertama dipakai.
- Jika key lebih pendek dari 16 byte, sisa buffer diisi `0x00`.
- Jika key lebih panjang dari 16 byte, byte setelah posisi ke-16 diabaikan.

Contoh key pendek:

```text
key16 = "abc"

buffer key:
61 62 63 00 00 00 00 00 00 00 00 00 00 00 00 00
```

`61`, `62`, dan `63` adalah kode ASCII untuk `a`, `b`, dan `c`.

Contoh key panjang:

```text
key16 = "1234567890ABCDEFGHIJKLMNOP"

byte yang dipakai:
31 32 33 34 35 36 37 38 39 30 41 42 43 44 45 46
```

Byte setelah 16 byte pertama tidak dipakai.

### IV

Argumen `iv8` dibaca sebagai string C, lalu disalin ke buffer `iv` berukuran 8 byte.

Nama `iv8` dipakai untuk menandai ukuran buffer IV internal. Program tidak melakukan validasi bahwa IV harus tepat 8 karakter. IV pendek tetap diterima dan dipad dengan `0x00`, sedangkan IV panjang tetap diterima tetapi hanya 8 byte pertama yang dipakai.

Aturan:

- Maksimal 8 byte pertama dipakai.
- Jika IV lebih pendek dari 8 byte, sisa buffer diisi `0x00`.
- Jika IV lebih panjang dari 8 byte, byte setelah posisi ke-8 diabaikan.

Contoh IV:

```text
iv8 = "ADA123!!"

buffer iv:
41 44 41 31 32 33 21 21
```

IV menjadi nilai awal `feedback` pada mode OFB. Dengan key yang sama, IV berbeda akan menghasilkan keystream berbeda.

### Plaintext

Plaintext pada mode `enc` berasal dari `argv[4]`.

Panjang plaintext dihitung dengan:

```c
strlen(argv[4])
```

Konsekuensinya:

- Plaintext diperlakukan sebagai string biasa.
- Byte `NUL` atau `0x00` tidak dapat menjadi bagian plaintext command line.
- Plaintext berisi spasi harus diapit tanda kutip.

Mode OFB tidak membutuhkan padding. Jika plaintext 13 byte, ciphertext juga 13 byte.

Untuk contoh tugas ini, dipakai dua plaintext agar perbedaan 1 block dan 2 block terlihat jelas:

```text
Contoh 1 block:
plaintext = "arif"
panjang   = 4 byte
block 1   = "arif" = 61 72 69 66

Contoh 2 block:
plaintext = "firdaus arif"
panjang   = 12 byte
block 1   = "firdaus " = 66 69 72 64 61 75 73 20
block 2   = "arif"     = 61 72 69 66
```

Pada contoh 2 block, block kedua disebut block parsial karena hanya berisi 4 byte. Program tetap membuat `keystream_2` berukuran 8 byte, tetapi yang dipakai untuk XOR hanya 4 byte pertama karena plaintext yang tersisa hanya 4 byte.

### Ciphertext Hex

Mode `enc` mencetak ciphertext sebagai hexadecimal uppercase.

Aturan ciphertext untuk mode `dec`:

- Panjang string hex harus genap.
- Karakter valid: `0-9`, `a-f`, `A-F`.
- Setiap dua karakter hex menjadi satu byte.

Contoh ciphertext untuk plaintext 2 block `firdaus arif`:

```text
ciphertext byte:
34 30 44 A3 CF 44 2F 1A B9 54 B9 C6

ciphertext_hex:
343044A3CF442F1AB954B9C6
```

Huruf kecil juga diterima saat dekripsi:

```text
343044a3cf442f1ab954b9c6
```

Nilainya sama dengan:

```text
343044A3CF442F1AB954B9C6
```

## Model OFB

Mode `OFB` mengubah block cipher menjadi generator keystream.

Rumus:

```text
feedback_0 = IV
stream_i = encrypt_block(feedback_i)
feedback_(i+1) = stream_i
output_i = input_i XOR stream_i
```

Makna rumus:

- Feedback awal berasal dari IV.
- Feedback dienkripsi oleh block cipher.
- Hasil enkripsi feedback menjadi stream.
- Stream di-XOR dengan input.
- Stream juga menjadi feedback berikutnya.

Sifat penting OFB:

- Enkripsi dan dekripsi memakai fungsi yang sama.
- Block cipher tidak langsung mengenkripsi plaintext.
- Feedback berasal dari output block cipher, bukan dari ciphertext.
- Panjang output sama dengan panjang input.
- Padding tidak diperlukan.
- Blok terakhir boleh kurang dari 8 byte.

Contoh input 13 byte:

```text
input:
byte 0..12

iterasi 1:
feedback_0 = IV
stream_0 = encrypt_block(feedback_0)
output byte 0..7 = input byte 0..7 XOR stream_0 byte 0..7

iterasi 2:
feedback_1 = stream_0
stream_1 = encrypt_block(feedback_1)
output byte 8..12 = input byte 8..12 XOR stream_1 byte 0..4
```

Pada iterasi kedua, hanya 5 byte pertama dari `stream_1` yang dipakai.

Kenapa enkripsi dan dekripsi sama:

```text
ciphertext = plaintext XOR stream
plaintext = ciphertext XOR stream
```

Karena:

```text
(plaintext XOR stream) XOR stream = plaintext
```

Syaratnya, dekripsi harus memakai key dan IV yang sama dengan enkripsi.

## Peta Fungsi di `main.c`

Urutan besar isi `main.c`:

1. Header dan macro.
2. Tabel `SBOX`.
3. Fungsi bit-level dan konversi word.
4. Fungsi ronde cipher.
5. Key schedule.
6. Enkripsi satu blok.
7. Konversi argumen teks ke byte.
8. Mode OFB.
9. Parser hex.
10. Printer hex.
11. Usage.
12. Fungsi `main`.

Alur pemanggilan utama:

```text
main
|-- copy_text_bytes
|-- enc
|   |-- ofb_crypt
|   |   |-- generate_round_keys
|   |   |-- encrypt_block
|   |   |   `-- round_function
|   |   |       |-- substitute_word
|   |   |       `-- permute_word
|   |   `-- XOR input dengan stream
|   `-- print_hex
`-- dec
    |-- hex_to_bytes
    |-- ofb_crypt
    |-- fwrite
    `-- free buffer
```

## Header dan Macro

Kode:

```c
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define BLOCK_SIZE 8
#define KEY_SIZE 16
#define ROUNDS 8
```

Tujuan header:

| Header | Dipakai untuk |
| --- | --- |
| `stdint.h` | `uint8_t`, `uint32_t` |
| `stdio.h` | `printf`, `fprintf`, `fwrite` |
| `stdlib.h` | `malloc`, `free` |
| `string.h` | `strlen`, `strcmp`, `memset`, `memcpy` |

Tujuan macro:

| Macro | Nilai | Makna |
| --- | --- | --- |
| `BLOCK_SIZE` | `8` | Ukuran blok 8 byte |
| `KEY_SIZE` | `16` | Ukuran key 16 byte |
| `ROUNDS` | `8` | Jumlah ronde Feistel |

Hubungan ukuran:

```text
BLOCK_SIZE = 8 byte = 64 bit
KEY_SIZE   = 16 byte = 128 bit
ROUNDS     = 8 ronde
```

Karena blok 8 byte dibagi dua, fungsi Feistel bekerja pada:

```text
4 byte kiri  = 32 bit
4 byte kanan = 32 bit
```

## Tabel S-Box

Kode:

```c
static const uint8_t SBOX[16] = {
    0xE, 0x4, 0xD, 0x1,
    0x2, 0xF, 0xB, 0x8,
    0x3, 0xA, 0x6, 0xC,
    0x5, 0x9, 0x0, 0x7
};
```

Tujuan:

`SBOX` adalah tabel substitusi untuk nilai 4-bit. Nilai 4-bit memiliki 16 kemungkinan, yaitu `0x0` sampai `0xF`, sehingga tabel memiliki 16 elemen.

Cara membaca tabel:

```text
input 0x0 -> output 0xE
input 0x1 -> output 0x4
input 0x2 -> output 0xD
...
input 0xF -> output 0x7
```

Substitusi berarti mengganti nilai lama dengan nilai baru berdasarkan tabel.

Contoh:

```text
input nibble  = 0x2
SBOX[0x2]     = 0xD
output nibble = 0xD
```

Jika byte `0x2F` dipandang sebagai dua nibble:

```text
0x2F = 0x2 dan 0xF
```

maka substitusinya:

```text
0x2 -> 0xD
0xF -> 0x7
```

Hasilnya:

```text
0x2F -> 0xD7
```

Catatan penting:

- Yang disubstitusi adalah nilai 4-bit, bukan karakter hex.
- Hex hanya cara menulis nilai agar ringkas.
- Secara bit, `0xF` berarti `1111`.

### Kebebasan Mendefinisikan S-Box

S-Box boleh didefinisikan sendiri selama cocok dengan cara program memakainya.

Syarat teknis:

- Jumlah elemen tetap 16.
- Setiap elemen sebaiknya berada pada rentang `0x0` sampai `0xF`.
- Idealnya semua nilai unik.

S-Box pada program ini adalah permutasi:

```text
E 4 D 1 2 F B 8 3 A 6 C 5 9 0 7
```

Semua nilai dari `0x0` sampai `0xF` muncul tepat satu kali.

Contoh S-Box lain yang valid secara struktur:

```c
static const uint8_t SBOX[16] = {
    0x6, 0xB, 0x0, 0x4,
    0xD, 0x2, 0xE, 0x8,
    0xF, 0x3, 0xA, 0x7,
    0x1, 0xC, 0x5, 0x9
};
```

Contoh yang sebaiknya dihindari:

```c
static const uint8_t SBOX[16] = {
    0x0, 0x1, 0x2, 0x3,
    0x4, 0x5, 0x6, 0x7,
    0x8, 0x9, 0xA, 0xB,
    0xC, 0xD, 0xE, 0xF
};
```

Itu disebut identity mapping karena input sama dengan output. Substitusi seperti itu hampir tidak memberi efek non-linear.

## Rotasi Kiri 32-bit

Kode:

```c
static uint32_t rotl32(uint32_t value, unsigned shift) {
    return (value << shift) | (value >> (32U - shift));
}
```

Tujuan:

`rotl32` melakukan rotasi kiri pada nilai 32-bit.

Rotasi kiri berarti bit digeser ke kiri, lalu bit yang keluar dari sisi kiri dimasukkan kembali ke sisi kanan. Jadi ini bukan sekadar shift biasa.

Cara membacanya:

```text
secara tujuan: rotate kiri
secara sintaks: shift kiri + shift kanan + OR
```

### `rotl32`

Header fungsi:

```c
static uint32_t rotl32(uint32_t value, unsigned shift)
```

Maknanya:

- `static`: fungsi lokal untuk file `main.c`.
- `uint32_t`: fungsi mengembalikan unsigned integer tepat 32 bit.
- `rotl32`: nama fungsi.
- `uint32_t value`: nilai 32-bit yang akan diputar.
- `unsigned shift`: jumlah posisi rotasi.

Statement utama:

```c
return (value << shift) | (value >> (32U - shift));
```

Alur baca:

1. `(value << shift)` membuat versi `value` yang digeser kiri.
2. `(32U - shift)` menghitung jarak geser kanan.
3. `(value >> (32U - shift))` mengambil bit yang keluar dari sisi kiri.
4. `|` menggabungkan dua hasil tersebut.
5. `return` mengembalikan hasil gabungan.

Contoh jika dipanggil:

```c
rotl32(value, 3)
```

Maka expression return menjadi:

```c
return (value << 3) | (value >> 29);
```

Ilustrasi 8-bit agar mudah dibaca:

```text
value awal      = 10110001
shift kiri 3    = 10001000
bit yang keluar = 101
masuk ke kanan  = 00000101
rotate kiri 3   = 10001101
```

Pada fungsi asli, lebar datanya 32 bit, bukan 8 bit.

Catatan teknis:

- Yang di-rotate adalah bit, bukan hex.
- Hex seperti `0x12345678` hanya representasi angka.
- `32U` adalah konstanta unsigned.
- Pemanggil di program ini selalu memberi `shift` aman, bukan `0` atau `32`.

## Pembacaan Word Big-Endian

Kode:

```c
static uint32_t read_u32_be(const uint8_t *src) {
    return ((uint32_t)src[0] << 24) |
           ((uint32_t)src[1] << 16) |
           ((uint32_t)src[2] << 8) |
           (uint32_t)src[3];
}
```

Tujuan:

`read_u32_be` membaca 4 byte dari array, lalu membentuk satu nilai `uint32_t` dengan urutan big-endian.

Big-endian berarti byte pertama menjadi byte paling tinggi.

```text
src[0] = byte paling tinggi
src[1] = byte tinggi berikutnya
src[2] = byte rendah berikutnya
src[3] = byte paling rendah
```

### `read_u32_be`

Parameter:

```c
const uint8_t *src
```

Maknanya:

- `src` adalah pointer ke byte.
- `const` berarti data yang ditunjuk tidak diubah oleh fungsi.
- Fungsi membaca `src[0]`, `src[1]`, `src[2]`, dan `src[3]`.

Statement utama:

```c
return ((uint32_t)src[0] << 24) |
       ((uint32_t)src[1] << 16) |
       ((uint32_t)src[2] << 8) |
       (uint32_t)src[3];
```

Alur per bagian:

```text
src[0] -> cast ke uint32_t -> shift kiri 24 bit
src[1] -> cast ke uint32_t -> shift kiri 16 bit
src[2] -> cast ke uint32_t -> shift kiri 8 bit
src[3] -> cast ke uint32_t -> tidak digeser
```

Susunan posisi:

```text
src[0] << 24  -> [src0][0000][0000][0000]
src[1] << 16  -> [0000][src1][0000][0000]
src[2] << 8   -> [0000][0000][src2][0000]
src[3]        -> [0000][0000][0000][src3]
```

Setelah digabung dengan `|`:

```text
[src0][src1][src2][src3]
```

Contoh:

```text
src[0] = 0x12
src[1] = 0x34
src[2] = 0x56
src[3] = 0x78

hasil = 0x12345678
```

Baris `return` ditulis beberapa baris agar mudah dibaca. Secara C, itu tetap satu statement karena baru selesai di tanda `;`.

Catatan:

Fungsi ini tidak bergantung pada endianness CPU. Word dibentuk manual dengan shift dan OR, sehingga hasilnya konsisten di mesin little-endian maupun big-endian.

## Penulisan Word Big-Endian

Kode:

```c
static void write_u32_be(uint8_t *dst, uint32_t value) {
    dst[0] = (uint8_t)(value >> 24);
    dst[1] = (uint8_t)(value >> 16);
    dst[2] = (uint8_t)(value >> 8);
    dst[3] = (uint8_t)value;
}
```

Tujuan:

`write_u32_be` melakukan kebalikan dari `read_u32_be`. Fungsi ini memecah satu word 32-bit menjadi 4 byte big-endian.

### `write_u32_be`

Parameter:

```c
uint8_t *dst
```

`dst` adalah pointer ke buffer tujuan. Karena tidak memakai `const`, fungsi boleh menulis ke buffer tersebut.

Parameter:

```c
uint32_t value
```

`value` adalah word 32-bit yang akan dipecah.

Statement assignment:

```c
dst[0] = (uint8_t)(value >> 24);
```

Alur bacanya:

```text
geser value ke kanan 24 bit
ambil 8 bit paling rendah dengan cast uint8_t
simpan ke dst[0]
```

Empat assignment membentuk pola:

```text
dst[0] = byte tertinggi dari value
dst[1] = byte tinggi berikutnya
dst[2] = byte rendah berikutnya
dst[3] = byte terendah dari value
```

Contoh:

```text
value = 0x12345678

dst[0] = 0x12
dst[1] = 0x34
dst[2] = 0x56
dst[3] = 0x78
```

Relasi dua fungsi:

```text
read_u32_be:  byte array -> uint32_t
write_u32_be: uint32_t -> byte array
```

## Substitusi Word 32-bit

Kode:

```c
static uint32_t substitute_word(uint32_t value) {
    uint32_t out = 0;

    for (int i = 0; i < 8; ++i) {
        uint32_t nibble = (value >> (i * 4)) & 0xFU;
        out |= ((uint32_t)SBOX[nibble]) << (i * 4);
    }

    return out;
}
```

Tujuan:

`substitute_word` mengganti setiap nibble di dalam word 32-bit memakai `SBOX`.

Karena:

```text
32 bit / 4 bit = 8 nibble
```

maka loop berjalan 8 kali.

### `substitute_word`

Inisialisasi:

```c
uint32_t out = 0;
```

`out` adalah word hasil. Awalnya nol agar posisi bit kosong sebelum diisi hasil substitusi.

Loop:

```c
for (int i = 0; i < 8; ++i)
```

Alur loop:

```text
mulai dari i = 0
selama i < 8, jalankan isi loop
setelah tiap iterasi, i dinaikkan 1
```

Ambil nibble:

```c
uint32_t nibble = (value >> (i * 4)) & 0xFU;
```

Alur bacanya:

1. `i * 4` menentukan posisi bit nibble.
2. `value >> (i * 4)` menggeser nibble target ke posisi paling rendah.
3. `& 0xFU` mengambil hanya 4 bit paling rendah.
4. Hasilnya disimpan sebagai `nibble`.

Masukkan hasil substitusi:

```c
out |= ((uint32_t)SBOX[nibble]) << (i * 4);
```

Alur bacanya:

1. `SBOX[nibble]` mengambil nilai pengganti.
2. Cast ke `uint32_t` agar aman digeser.
3. `<< (i * 4)` mengembalikan nibble ke posisi asal.
4. `out |= ...` memasukkan nibble tersebut ke `out`.

Posisi nibble:

```text
[nibble7][nibble6][nibble5][nibble4][nibble3][nibble2][nibble1][nibble0]
```

Contoh ringkas:

```text
value = 0x0000000F
nibble i=0 = 0xF
SBOX[0xF] = 0x7
out = 0x00000007
```

## Permutasi Word

Kode:

```c
static uint32_t permute_word(uint32_t value) {
    return rotl32(value, 3) ^ rotl32(value, 11) ^ rotl32(value, 19);
}
```

Tujuan:

`permute_word` menyebarkan perubahan bit melalui beberapa rotasi dan XOR.

### `permute_word`

Statement return:

```c
return rotl32(value, 3) ^ rotl32(value, 11) ^ rotl32(value, 19);
```

Alur bacanya:

```text
rotasi value ke kiri 3 bit
rotasi value ke kiri 11 bit
rotasi value ke kiri 19 bit
gabungkan ketiganya dengan XOR
kembalikan hasil
```

Secara konseptual:

```text
a = rotl32(value, 3)
b = rotl32(value, 11)
c = rotl32(value, 19)
output = a XOR b XOR c
```

Tujuan difusi:

- Bit dari posisi berbeda bercampur.
- Perubahan kecil pada input menyebar ke beberapa posisi output.
- Hasil fungsi ronde tidak hanya berupa substitusi nibble, tetapi juga penyebaran bit.

## Fungsi Ronde

Kode:

```c
static uint32_t round_function(uint32_t right, uint32_t round_key) {
    return permute_word(substitute_word(right ^ round_key));
}
```

Tujuan:

`round_function` adalah fungsi inti pada ronde Feistel. Fungsi ini menerima bagian kanan blok dan satu round key.

### `round_function`

Statement return:

```c
return permute_word(substitute_word(right ^ round_key));
```

Alur baca dari dalam ke luar:

1. `right ^ round_key`
   Data kanan dicampur dengan round key.

2. `substitute_word(...)`
   Hasil XOR masuk ke S-Box per nibble.

3. `permute_word(...)`
   Hasil substitusi disebar melalui rotasi dan XOR.

4. `return`
   Hasil akhir dikembalikan sebagai word 32-bit.

Alur data:

```text
right
XOR round_key
substitute_word
permute_word
output 32-bit
```

Fungsi ronde tidak perlu bisa dibalik secara langsung. Struktur Feistel tetap bekerja karena pembalikan block cipher dapat dilakukan dengan urutan round key terbalik. Program ini tidak membuat fungsi dekripsi block terpisah karena mode OFB hanya membutuhkan `encrypt_block`.

## Pembentukan Round Key

Kode:

```c
static void generate_round_keys(const uint8_t key[KEY_SIZE], uint32_t round_keys[ROUNDS]) {
    uint32_t a = read_u32_be(key);
    uint32_t b = read_u32_be(key + 4);
    uint32_t c = read_u32_be(key + 8);
    uint32_t d = read_u32_be(key + 12);

    for (int i = 0; i < ROUNDS; ++i) {
        uint32_t mix = rotl32(a ^ c, (unsigned)((i % 7) + 1)) +
                       rotl32(b ^ d, (unsigned)(((i + 2) % 7) + 1)) +
                       (0x9E3779B9u * (uint32_t)(i + 1));

        round_keys[i] = substitute_word(mix ^ rotl32(d, (unsigned)(((i + 4) % 9) + 1)));

        uint32_t next = a ^ rotl32(round_keys[i], 7) ^
                        (0xA5A5A5A5u + (uint32_t)i * 0x01010101u);
        a = b;
        b = c;
        c = d;
        d = next;
    }
}
```

Tujuan:

`generate_round_keys` mengubah key 16 byte menjadi 8 round key. Setiap round key berukuran 32 bit.

### `generate_round_keys`

Parameter:

```c
const uint8_t key[KEY_SIZE]
```

`key` adalah input 16 byte. Karena memakai `const`, fungsi tidak mengubah key.

Parameter:

```c
uint32_t round_keys[ROUNDS]
```

`round_keys` adalah array output. Fungsi mengisi 8 elemen di array ini.

Pemecahan key:

```c
uint32_t a = read_u32_be(key);
uint32_t b = read_u32_be(key + 4);
uint32_t c = read_u32_be(key + 8);
uint32_t d = read_u32_be(key + 12);
```

Alur bacanya:

```text
a = key byte 0..3
b = key byte 4..7
c = key byte 8..11
d = key byte 12..15
```

Ekspresi `key + 4` berarti alamat key digeser 4 byte ke depan.

Loop ronde:

```c
for (int i = 0; i < ROUNDS; ++i)
```

Karena `ROUNDS` bernilai 8, loop berjalan untuk:

```text
i = 0, 1, 2, 3, 4, 5, 6, 7
```

Pembentukan `mix`:

```c
uint32_t mix = rotl32(a ^ c, (unsigned)((i % 7) + 1)) +
               rotl32(b ^ d, (unsigned)(((i + 2) % 7) + 1)) +
               (0x9E3779B9u * (uint32_t)(i + 1));
```

Alur bacanya:

1. `a ^ c` mencampur word pertama dan ketiga.
2. `b ^ d` mencampur word kedua dan keempat.
3. Dua hasil XOR dirotasi dengan jarak yang bergantung pada `i`.
4. Konstanta `0x9E3779B9u * (i + 1)` menambah variasi per ronde.
5. Tiga bagian dijumlahkan menjadi `mix`.

Pembentukan round key:

```c
round_keys[i] = substitute_word(mix ^ rotl32(d, (unsigned)(((i + 4) % 9) + 1)));
```

Alur bacanya:

1. `d` dirotasi.
2. Hasil rotasi di-XOR dengan `mix`.
3. Hasil XOR masuk ke `substitute_word`.
4. Output disimpan ke `round_keys[i]`.

Update state:

```c
uint32_t next = a ^ rotl32(round_keys[i], 7) ^
                (0xA5A5A5A5u + (uint32_t)i * 0x01010101u);
a = b;
b = c;
c = d;
d = next;
```

Alur bacanya:

```text
buat word baru bernama next
geser a <- b
geser b <- c
geser c <- d
isi d dengan next
```

State key schedule berubah setiap ronde, sehingga round key berikutnya tidak hanya memakai susunan key awal yang sama.

Catatan:

- Key schedule deterministik. Key sama menghasilkan round key sama.
- Penjumlahan pada `uint32_t` mengikuti wrap around modulo 2^32.
- Konstanta dipakai untuk memberi variasi tetap antar ronde.

## Enkripsi Satu Blok

Kode:

```c
static void encrypt_block(const uint8_t in[BLOCK_SIZE], uint8_t out[BLOCK_SIZE],
                          const uint32_t round_keys[ROUNDS]) {
    uint32_t left = read_u32_be(in);
    uint32_t right = read_u32_be(in + 4);

    for (int i = 0; i < ROUNDS; ++i) {
        uint32_t next_left = right;
        uint32_t next_right = left ^ round_function(right, round_keys[i]);
        left = next_left;
        right = next_right;
    }

    write_u32_be(out, right);
    write_u32_be(out + 4, left);
}
```

Tujuan:

`encrypt_block` menerima 8 byte input dan menghasilkan 8 byte output melalui jaringan Feistel 8 ronde.

### `encrypt_block`

Pemecahan blok:

```c
uint32_t left = read_u32_be(in);
uint32_t right = read_u32_be(in + 4);
```

Alur:

```text
left  = in[0..3]
right = in[4..7]
```

Ronde Feistel:

```c
uint32_t next_left = right;
uint32_t next_right = left ^ round_function(right, round_keys[i]);
left = next_left;
right = next_right;
```

Rumus:

```text
L_(i+1) = R_i
R_(i+1) = L_i XOR F(R_i, K_i)
```

Variabel `next_left` dan `next_right` dipakai agar nilai `left` dan `right` lama tidak hilang sebelum keduanya selesai dihitung.

Output:

```c
write_u32_be(out, right);
write_u32_be(out + 4, left);
```

Output ditulis sebagai:

```text
right diikuti left
```

Urutan ini adalah swap akhir Feistel.

Catatan:

Pada mode OFB, fungsi ini tidak menerima plaintext langsung. Fungsi ini menerima `feedback` dan menghasilkan `stream`.

## Penyalinan Argumen Teks ke Buffer Tetap

Kode:

```c
static void copy_text_bytes(const char *text, uint8_t *dst, size_t dst_len) {
    memset(dst, 0, dst_len);

    for (size_t i = 0; i < dst_len && text[i] != '\0'; ++i) {
        dst[i] = (uint8_t)text[i];
    }
}
```

Tujuan:

`copy_text_bytes` menyalin argumen teks ke buffer byte berukuran tetap. Fungsi ini dipakai untuk membentuk key dan IV internal.

### `copy_text_bytes`

Parameter:

```c
const char *text
```

`text` adalah string input dari command line.

Parameter:

```c
uint8_t *dst
```

`dst` adalah buffer tujuan.

Parameter:

```c
size_t dst_len
```

`dst_len` adalah ukuran buffer tujuan.

Inisialisasi:

```c
memset(dst, 0, dst_len);
```

Semua byte di buffer tujuan diisi `0x00`.

Loop:

```c
for (size_t i = 0; i < dst_len && text[i] != '\0'; ++i)
```

Loop berjalan selama dua kondisi benar:

- `i < dst_len`: belum melewati ukuran buffer.
- `text[i] != '\0'`: belum sampai akhir string.

Penyalinan:

```c
dst[i] = (uint8_t)text[i];
```

Karakter dari string disalin sebagai byte.

Efek:

- Input pendek dipad dengan nol.
- Input panjang dipotong.
- Tidak ada penulisan melewati batas buffer.

Catatan:

Buffer `dst` bukan string C. Buffer ini tidak membutuhkan terminator `'\0'` karena dipakai sebagai byte array dengan panjang tetap.

## Mode OFB

Kode:

```c
static void ofb_crypt(const uint8_t *input, uint8_t *output, size_t len,
                      const uint8_t key[KEY_SIZE], const uint8_t iv[BLOCK_SIZE]) {
    uint32_t round_keys[ROUNDS];
    uint8_t feedback[BLOCK_SIZE];
    uint8_t stream[BLOCK_SIZE];

    generate_round_keys(key, round_keys);
    memcpy(feedback, iv, BLOCK_SIZE);

    for (size_t offset = 0; offset < len; offset += BLOCK_SIZE) {
        size_t chunk = len - offset;
        if (chunk > BLOCK_SIZE) {
            chunk = BLOCK_SIZE;
        }

        encrypt_block(feedback, stream, round_keys);
        memcpy(feedback, stream, BLOCK_SIZE);

        for (size_t i = 0; i < chunk; ++i) {
            output[offset + i] = input[offset + i] ^ stream[i];
        }
    }
}
```

Tujuan:

`ofb_crypt` adalah fungsi pemrosesan data utama. Fungsi ini dipakai untuk enkripsi dan dekripsi.

### `ofb_crypt`

Buffer lokal:

```c
uint32_t round_keys[ROUNDS];
uint8_t feedback[BLOCK_SIZE];
uint8_t stream[BLOCK_SIZE];
```

Makna:

- `round_keys`: 8 round key untuk block cipher.
- `feedback`: state OFB saat ini.
- `stream`: hasil `encrypt_block(feedback)`.

Inisialisasi:

```c
generate_round_keys(key, round_keys);
memcpy(feedback, iv, BLOCK_SIZE);
```

Alur:

```text
buat round key dari key
salin IV ke feedback
```

Loop utama:

```c
for (size_t offset = 0; offset < len; offset += BLOCK_SIZE)
```

`offset` bergerak dengan langkah 8 byte:

```text
0, 8, 16, ...
```

Hitung `chunk`:

```c
size_t chunk = len - offset;
if (chunk > BLOCK_SIZE) {
    chunk = BLOCK_SIZE;
}
```

`chunk` adalah jumlah byte yang diproses pada iterasi saat ini. Nilainya maksimal 8.

Membuat keystream:

```c
encrypt_block(feedback, stream, round_keys);
memcpy(feedback, stream, BLOCK_SIZE);
```

Alur:

```text
stream = encrypt_block(feedback)
feedback berikutnya = stream
```

XOR:

```c
output[offset + i] = input[offset + i] ^ stream[i];
```

Setiap byte input di-XOR dengan byte stream.

Contoh jika `len = 9`:

```text
iterasi 1:
offset = 0
chunk = 8
memproses byte 0..7

iterasi 2:
offset = 8
chunk = 1
memproses byte 8 saja
```

Catatan:

Karena `chunk` membatasi blok terakhir, fungsi tidak membaca melewati akhir input dan tidak menulis melewati akhir output.

## Konversi Karakter Hex

Kode:

```c
static int hex_value(char c) {
    if (c >= '0' && c <= '9') {
        return c - '0';
    }
    if (c >= 'a' && c <= 'f') {
        return 10 + (c - 'a');
    }
    if (c >= 'A' && c <= 'F') {
        return 10 + (c - 'A');
    }
    return -1;
}
```

Tujuan:

`hex_value` mengubah satu karakter hex menjadi nilai 0 sampai 15.

### `hex_value`

Percabangan pertama:

```c
if (c >= '0' && c <= '9')
```

Jika `c` adalah digit, fungsi mengembalikan:

```c
c - '0'
```

Contoh:

```text
'7' - '0' = 7
```

Percabangan kedua dan ketiga menangani huruf:

```text
'a'..'f' -> 10..15
'A'..'F' -> 10..15
```

Jika semua kondisi gagal:

```c
return -1;
```

Nilai `-1` menandakan karakter tidak valid.

Contoh:

```text
hex_value('0') = 0
hex_value('9') = 9
hex_value('A') = 10
hex_value('F') = 15
hex_value('f') = 15
hex_value('Z') = -1
```

Fungsi mengembalikan `int`, bukan `uint8_t`, karena perlu bisa mengembalikan `-1`.

## Konversi Hex ke Byte

Kode:

```c
static uint8_t *hex_to_bytes(const char *hex, size_t *out_len) {
    size_t hex_len = strlen(hex);

    if ((hex_len % 2) != 0) {
        return NULL;
    }

    *out_len = hex_len / 2;
    uint8_t *bytes = malloc(*out_len == 0 ? 1 : *out_len);
    if (bytes == NULL) {
        return NULL;
    }

    for (size_t i = 0; i < hex_len; i += 2) {
        int high = hex_value(hex[i]);
        int low = hex_value(hex[i + 1]);

        if (high < 0 || low < 0) {
            free(bytes);
            return NULL;
        }

        bytes[i / 2] = (uint8_t)((high << 4) | low);
    }

    return bytes;
}
```

Tujuan:

`hex_to_bytes` mengubah string hex menjadi buffer byte. Fungsi ini dipakai pada mode dekripsi.

### `hex_to_bytes`

Hitung panjang:

```c
size_t hex_len = strlen(hex);
```

Validasi panjang:

```c
if ((hex_len % 2) != 0) {
    return NULL;
}
```

Panjang hex harus genap karena dua karakter hex membentuk satu byte.

Tentukan panjang output:

```c
*out_len = hex_len / 2;
```

`out_len` adalah pointer. Ekspresi `*out_len` berarti isi variabel yang ditunjuk oleh pointer tersebut.

Alokasi:

```c
uint8_t *bytes = malloc(*out_len == 0 ? 1 : *out_len);
```

Jika output 0 byte, fungsi tetap meminta alokasi 1 byte agar tidak bergantung pada perilaku `malloc(0)`.

Loop:

```c
for (size_t i = 0; i < hex_len; i += 2)
```

Loop membaca dua karakter setiap iterasi.

Konversi pasangan:

```c
int high = hex_value(hex[i]);
int low = hex_value(hex[i + 1]);
```

Gabungkan dua nibble:

```c
bytes[i / 2] = (uint8_t)((high << 4) | low);
```

Contoh:

```text
hex = "7C"
high = 7
low = 12
byte = (7 << 4) | 12
byte = 0x7C
```

Jika karakter invalid ditemukan:

```c
free(bytes);
return NULL;
```

Buffer yang sudah dialokasikan dibebaskan sebelum fungsi gagal.

Catatan:

Fungsi ini mengembalikan pointer heap. Pemanggil wajib memanggil `free` setelah selesai memakai buffer.

## Pencetakan Hex

Kode:

```c
static void print_hex(const uint8_t *data, size_t len) {
    for (size_t i = 0; i < len; ++i) {
        printf("%02X", data[i]);
    }
    printf("\n");
}
```

Tujuan:

`print_hex` mencetak byte array sebagai hexadecimal uppercase.

### `print_hex`

Loop:

```c
for (size_t i = 0; i < len; ++i)
```

Loop berjalan dari byte pertama sampai byte terakhir.

Format:

```c
printf("%02X", data[i]);
```

Makna `%02X`:

| Bagian | Makna |
| --- | --- |
| `%` | Awal format specifier |
| `0` | Isi padding dengan nol |
| `2` | Lebar minimal 2 karakter |
| `X` | Cetak hex uppercase |

Contoh:

```text
0x0A -> 0A
0x7C -> 7C
0xFF -> FF
```

Setelah semua byte dicetak, fungsi mencetak newline:

```c
printf("\n");
```

Output dibuat tanpa spasi agar bisa langsung dipakai sebagai input dekripsi.

## Pesan Penggunaan

Kode:

```c
static void print_usage(const char *program) {
    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s enc <key16> <iv8> <plaintext>\n", program);
    fprintf(stderr, "  %s dec <key16> <iv8> <ciphertext_hex>\n", program);
}
```

Tujuan:

`print_usage` mencetak format penggunaan program.

### `print_usage`

Parameter:

```c
const char *program
```

`program` berasal dari `argv[0]`, sehingga usage mengikuti nama binary saat dipanggil.

Output memakai:

```c
fprintf(stderr, ...)
```

`stderr` dipakai karena usage muncul pada kondisi error. Output normal tetap berada di `stdout`.

## Fungsi `main`

Kode:

```c
int main(int argc, char *argv[]) {
    uint8_t key[KEY_SIZE];
    uint8_t iv[BLOCK_SIZE];

    if (argc != 5) {
        print_usage(argv[0]);
        return 1;
    }

    copy_text_bytes(argv[2], key, KEY_SIZE);
    copy_text_bytes(argv[3], iv, BLOCK_SIZE);

    if (strcmp(argv[1], "enc") == 0) {
        size_t len = strlen(argv[4]);
        uint8_t *ciphertext = malloc(len == 0 ? 1 : len);

        if (ciphertext == NULL) {
            return 1;
        }

        ofb_crypt((const uint8_t *)argv[4], ciphertext, len, key, iv);
        print_hex(ciphertext, len);
        free(ciphertext);
        return 0;
    }

    if (strcmp(argv[1], "dec") == 0) {
        size_t len = 0;
        uint8_t *ciphertext = hex_to_bytes(argv[4], &len);
        uint8_t *plaintext;

        if (ciphertext == NULL) {
            fprintf(stderr, "Ciphertext harus berupa hex valid.\n");
            return 1;
        }

        plaintext = malloc(len == 0 ? 1 : len);
        if (plaintext == NULL) {
            free(ciphertext);
            return 1;
        }

        ofb_crypt(ciphertext, plaintext, len, key, iv);
        fwrite(plaintext, 1, len, stdout);
        printf("\n");

        free(plaintext);
        free(ciphertext);
        return 0;
    }

    print_usage(argv[0]);
    return 1;
}
```

Tujuan:

`main` mengatur alur program dari argumen command line sampai output akhir.

### Cara Membaca Alur `main`

Buffer awal:

```c
uint8_t key[KEY_SIZE];
uint8_t iv[BLOCK_SIZE];
```

Program menyiapkan buffer key 16 byte dan IV 8 byte di stack.

Validasi jumlah argumen:

```c
if (argc != 5)
```

Jika jumlah argumen salah, program mencetak usage dan keluar dengan kode `1`.

Konversi key dan IV:

```c
copy_text_bytes(argv[2], key, KEY_SIZE);
copy_text_bytes(argv[3], iv, BLOCK_SIZE);
```

Key dan IV teks diubah menjadi byte array tetap.

Cabang enkripsi:

```c
if (strcmp(argv[1], "enc") == 0)
```

Jika operasi adalah `enc`:

1. Hitung panjang plaintext.
2. Alokasikan buffer ciphertext.
3. Jalankan `ofb_crypt`.
4. Cetak ciphertext sebagai hex.
5. Bebaskan buffer.
6. Keluar dengan kode `0`.

Cabang dekripsi:

```c
if (strcmp(argv[1], "dec") == 0)
```

Jika operasi adalah `dec`:

1. Ubah hex menjadi byte ciphertext.
2. Alokasikan buffer plaintext.
3. Jalankan `ofb_crypt`.
4. Cetak plaintext dengan `fwrite`.
5. Bebaskan buffer.
6. Keluar dengan kode `0`.

Jika operasi bukan `enc` atau `dec`, program mencetak usage dan keluar dengan kode `1`.

Catatan:

`fwrite` dipakai untuk dekripsi karena plaintext hasil dekripsi tidak diterminasi dengan `'\0'`. Fungsi ini mencetak berdasarkan panjang byte, bukan berdasarkan string C.

## Alur Enkripsi Lengkap

### Contoh 1 Block: `arif`

Command:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! "arif"
```

Alur ringkas:

1. `argv[4]` berisi plaintext `arif`.
2. Panjang plaintext adalah 4 byte.
3. Karena `BLOCK_SIZE` adalah 8 byte, plaintext ini hanya masuk ke block pertama.
4. Program menghasilkan `keystream_1` dari IV.
5. Empat byte plaintext di-XOR dengan 4 byte pertama dari `keystream_1`.
6. Karena plaintext sudah habis, program tidak membuat block kedua.

Nilai byte:

```text
keystream_1  = 52 59 36 C7 AE 31 5C 3A
plaintext_1  = 61 72 69 66
ciphertext_1 = 33 2B 5F A1
```

Empat byte terakhir `AE 31 5C 3A` dari `keystream_1` tidak dipakai pada contoh ini, karena plaintext `arif` hanya 4 byte.

Output contoh:

```text
332B5FA1
```

### Contoh 2 Block: `firdaus arif`

Command:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! "firdaus arif"
```

Alur:

1. `argc` dicek dan harus bernilai 5.
2. `argv[1]` adalah `enc`.
3. `argv[2]` disalin menjadi key 16 byte.
4. `argv[3]` disalin menjadi IV 8 byte.
5. `argv[4]` adalah plaintext `firdaus arif`.
6. Panjang plaintext adalah 12 byte.
7. Buffer ciphertext 12 byte dialokasikan.
8. `ofb_crypt` membuat round key.
9. Feedback awal diisi IV.
10. Iterasi pertama menghasilkan `keystream_1` dari IV.
11. Block plaintext pertama di-XOR dengan `keystream_1`.
12. Output block cipher pertama menjadi feedback untuk block kedua.
13. Iterasi kedua menghasilkan `keystream_2`.
14. Sisa plaintext pada block kedua di-XOR dengan 4 byte pertama dari `keystream_2`.
15. Ciphertext 12 byte dicetak sebagai hex uppercase.
16. Buffer ciphertext dibebaskan.

Pembagian block:

```text
block 1 plaintext = "firdaus " = 66 69 72 64 61 75 73 20
block 2 plaintext = "arif"     = 61 72 69 66
```

Nilai keystream dan ciphertext per block:

```text
keystream_1  = 52 59 36 C7 AE 31 5C 3A
plaintext_1  = 66 69 72 64 61 75 73 20
ciphertext_1 = 34 30 44 A3 CF 44 2F 1A

keystream_2  = D8 26 D0 A0 63 48 BE A8
plaintext_2  = 61 72 69 66
ciphertext_2 = B9 54 B9 C6
```

Empat byte terakhir `63 48 BE A8` dari `keystream_2` tidak dipakai pada contoh ini, karena plaintext sudah habis setelah 4 byte block kedua.

Output contoh:

```text
343044A3CF442F1AB954B9C6
```

## Alur Dekripsi Lengkap

Contoh:

```bash
./block_cipher dec KUNCI-1NGG121S ADA123!! 343044A3CF442F1AB954B9C6
```

Alur:

1. `argc` dicek dan harus bernilai 5.
2. `argv[1]` adalah `dec`.
3. `argv[2]` disalin menjadi key 16 byte.
4. `argv[3]` disalin menjadi IV 8 byte.
5. `argv[4]` dikonversi dari hex ke byte.
6. `343044A3CF442F1AB954B9C6` menjadi 12 byte ciphertext.
7. Buffer plaintext 12 byte dialokasikan.
8. `ofb_crypt` menghasilkan keystream yang sama karena key dan IV sama.
9. Ciphertext di-XOR dengan keystream.
10. Hasilnya adalah plaintext asli.
11. Plaintext dicetak dengan `fwrite`.
12. Buffer plaintext dan ciphertext dibebaskan.

Output contoh:

```text
firdaus arif
```

Jika key atau IV berbeda, program tetap menghasilkan output, tetapi output tidak akan menjadi plaintext asli.

## Pengelolaan Memori

Program memakai dua jenis penyimpanan:

| Jenis | Dipakai untuk |
| --- | --- |
| Stack | Buffer kecil berukuran tetap |
| Heap | Buffer yang ukurannya mengikuti input |

Buffer stack:

```text
key[16]
iv[8]
round_keys[8]
feedback[8]
stream[8]
```

Buffer heap:

```text
ciphertext pada enkripsi
ciphertext hasil hex_to_bytes pada dekripsi
plaintext pada dekripsi
```

Pola aman yang dipakai:

- Key dan IV disalin dengan batas ukuran.
- OFB memakai `chunk` agar blok terakhir aman.
- Parser hex memvalidasi panjang genap.
- Parser hex membebaskan buffer jika menemukan karakter invalid.
- Buffer heap dibebaskan setelah selesai dipakai.
- Jika alokasi plaintext gagal, ciphertext yang sudah dialokasikan dibebaskan.

Alokasi kosong:

```c
malloc(len == 0 ? 1 : len)
```

Pola ini menghindari ketergantungan pada perilaku `malloc(0)`. Walaupun alokasi 1 byte dilakukan, fungsi tetap memproses panjang 0 jika input kosong.

## Perilaku Error

Kondisi error:

| Kondisi | Respon |
| --- | --- |
| Jumlah argumen salah | Cetak usage, keluar `1` |
| Operasi bukan `enc` atau `dec` | Cetak usage, keluar `1` |
| Alokasi memori gagal | Keluar `1` |
| Hex panjang ganjil | Cetak error hex, keluar `1` |
| Hex mengandung karakter invalid | Cetak error hex, keluar `1` |

Pesan error eksplisit untuk hex:

```text
Ciphertext harus berupa hex valid.
```

Contoh panjang ganjil:

```bash
./block_cipher dec KUNCI-1NGG121S ADA123!! ABC
```

Contoh karakter invalid:

```bash
./block_cipher dec KUNCI-1NGG121S ADA123!! ZZ
```

Keduanya ditolak oleh parser hex.

## Makefile

Kode:

```makefile
CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -pedantic -O2
TARGET = block_cipher
SRC = main.c

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)

clean:
	rm -f $(TARGET)

.PHONY: all clean
```

Tujuan:

`Makefile` menyederhanakan proses build dan clean.

Variabel:

| Variabel | Makna |
| --- | --- |
| `CC` | Compiler, yaitu `gcc` |
| `CFLAGS` | Opsi compiler |
| `TARGET` | Nama binary |
| `SRC` | File sumber |

Opsi compiler:

| Opsi | Makna |
| --- | --- |
| `-std=c11` | Gunakan standar C11 |
| `-Wall` | Aktifkan warning umum |
| `-Wextra` | Aktifkan warning tambahan |
| `-pedantic` | Perketat kepatuhan standar C |
| `-O2` | Optimisasi tingkat 2 |

Target default:

```makefile
all: $(TARGET)
```

Jika pengguna menjalankan `make`, target `all` akan membangun `block_cipher`.

Target binary:

```makefile
$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)
```

Perintah efektifnya:

```bash
gcc -std=c11 -Wall -Wextra -pedantic -O2 -o block_cipher main.c
```

Target clean:

```makefile
clean:
	rm -f $(TARGET)
```

Target ini menghapus binary hasil build.

`.PHONY`:

```makefile
.PHONY: all clean
```

Baris ini memberi tahu `make` bahwa `all` dan `clean` adalah nama target perintah, bukan file biasa.

## Ringkasan Operasi Bit

Operator bit yang dipakai:

| Operator | Nama | Dipakai untuk |
| --- | --- | --- |
| `^` | XOR | Mencampur data, key, dan stream |
| `|` | OR | Menggabungkan bagian bit |
| `&` | AND | Mengambil nibble tertentu |
| `<<` | Shift kiri | Memindahkan byte/nibble ke posisi lebih tinggi |
| `>>` | Shift kanan | Mengambil byte/nibble atau membuat rotasi |

Operasi lain:

| Operasi | Dipakai untuk |
| --- | --- |
| `+` | Penjumlahan dalam key schedule |
| `*` | Perkalian konstanta ronde |
| `%` | Menghasilkan jarak rotasi yang berubah |

Fungsi memory/string:

| Fungsi | Dipakai untuk |
| --- | --- |
| `strlen` | Menghitung panjang string |
| `strcmp` | Membandingkan operasi `enc` dan `dec` |
| `memset` | Mengisi buffer dengan nol |
| `memcpy` | Menyalin IV dan feedback |
| `malloc` | Alokasi buffer heap |
| `free` | Membebaskan buffer heap |
| `fwrite` | Mencetak plaintext berdasarkan panjang byte |

## Kesimpulan Teknis

Program `OFB Cipher CLI` berhasil menunjukkan alur lengkap enkripsi dan dekripsi berbasis mode `OFB`.

Kesimpulan utama:

- Mode `OFB` mengubah block cipher menjadi generator keystream.
- Plaintext atau ciphertext tidak diproses langsung oleh block cipher, tetapi di-XOR dengan keystream.
- Enkripsi dan dekripsi memakai fungsi `ofb_crypt` yang sama karena sifat operasi `XOR`.
- Panjang output sama dengan panjang input, sehingga padding tidak diperlukan.
- Implementasi memakai jaringan Feistel 8 ronde sebagai block cipher kustom.
- Key 16 byte dan IV 8 byte dipakai sebagai buffer internal; input pendek dipad nol dan input panjang dipotong.
- Program sudah menangani beberapa kondisi error dasar, terutama jumlah argumen dan format ciphertext hex.

Batasan utama:

- Cipher yang dipakai adalah cipher kustom dan belum dianalisis secara kriptografis.
- Program tidak menyediakan autentikasi, sehingga perubahan ciphertext tidak dapat dideteksi otomatis.
- IV sebaiknya tidak dipakai ulang dengan key yang sama pada penggunaan OFB, karena reuse key dan IV dapat menghasilkan keystream yang sama.
- Program cocok untuk demonstrasi konsep dan pembelajaran teknis, bukan sebagai pengganti pustaka kriptografi produksi.

## Checklist Verifikasi

Bagian ini berisi checklist yang bisa dipakai untuk memastikan program berjalan sesuai dokumentasi. Checklist ini juga berguna sebagai pegangan saat demo atau saat menjawab pertanyaan teknis.

### Checklist Dasar

Build program:

```bash
make
```

Hasil yang diharapkan:

```text
block_cipher berhasil dibuat tanpa error kompilasi
```

Enkripsi contoh 1 block:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! "arif"
```

Output yang diharapkan:

```text
332B5FA1
```

Enkripsi contoh 2 block:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! "firdaus arif"
```

Output yang diharapkan:

```text
343044A3CF442F1AB954B9C6
```

Dekripsi contoh:

```bash
./block_cipher dec KUNCI-1NGG121S ADA123!! 343044A3CF442F1AB954B9C6
```

Output yang diharapkan:

```text
firdaus arif
```

Input hex invalid:

```bash
./block_cipher dec KUNCI-1NGG121S ADA123!! ZZ
```

Output error yang diharapkan:

```text
Ciphertext harus berupa hex valid.
```

### Checklist Tambahan

Dekripsi dengan hex huruf kecil:

```bash
./block_cipher dec KUNCI-1NGG121S ADA123!! 343044a3cf442f1ab954b9c6
```

Output yang diharapkan:

```text
firdaus arif
```

Tujuan uji:

```text
Membuktikan parser hex menerima A-F dan a-f.
```

Input hex panjang ganjil:

```bash
./block_cipher dec KUNCI-1NGG121S ADA123!! ABC
```

Output error yang diharapkan:

```text
Ciphertext harus berupa hex valid.
```

Tujuan uji:

```text
Membuktikan ciphertext hex harus memiliki jumlah karakter genap.
```

Plaintext kosong:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! ""
```

Output yang diharapkan:

```text
Hanya newline, tidak ada karakter hex.
```

Tujuan uji:

```text
Membuktikan input panjang 0 tetap aman diproses dan tidak menyebabkan error.
```

Key dan IV pendek:

```bash
./block_cipher enc abc iv "arif"
```

Output yang diharapkan:

```text
1D21FA5A
```

Tujuan uji:

```text
Membuktikan key pendek dan IV pendek diterima, lalu sisa buffer dipad dengan 0x00.
```

Key dan IV panjang:

```bash
./block_cipher enc 1234567890ABCDEFGHIJKLMNOP 1234567890 "arif"
```

Output yang diharapkan:

```text
40574999
```

Tujuan uji:

```text
Membuktikan key panjang hanya memakai 16 byte pertama dan IV panjang hanya memakai 8 byte pertama.
```

Round-trip enkripsi lalu dekripsi:

```bash
cipher=$(./block_cipher enc KUNCI-1NGG121S ADA123!! "firdaus arif")
./block_cipher dec KUNCI-1NGG121S ADA123!! "$cipher"
```

Output yang diharapkan:

```text
firdaus arif
```

Tujuan uji:

```text
Membuktikan ciphertext hasil enkripsi bisa dikembalikan menjadi plaintext awal dengan key dan IV yang sama.
```

### Ringkasan Skenario Uji

| Skenario | Hal yang Dibuktikan |
| --- | --- |
| Build | Program dapat dikompilasi dengan `Makefile` |
| Enkripsi 1 block | Block parsial bisa diproses tanpa padding |
| Enkripsi 2 block | Block penuh dan block parsial bisa diproses berurutan |
| Dekripsi | `ofb_crypt` dapat membalik ciphertext menjadi plaintext |
| Hex lowercase | Parser menerima `a-f` dan `A-F` |
| Hex invalid | Parser menolak karakter non-hex |
| Hex panjang ganjil | Parser menolak input yang tidak membentuk byte utuh |
| Plaintext kosong | Alokasi kosong dan panjang 0 ditangani aman |
| Key/IV pendek | Input pendek dipad dengan `0x00` |
| Key/IV panjang | Input panjang dipotong sesuai ukuran buffer internal |
