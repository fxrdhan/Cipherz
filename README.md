# OFB Cipher CLI in C

OFB Cipher CLI adalah program command line berbasis bahasa C untuk enkripsi dan dekripsi teks dengan mode operasi `Output Feedback` (`OFB`). Program memakai block cipher kustom berukuran blok 64-bit, kunci 128-bit, dan jaringan Feistel 8 ronde sebagai fungsi pembangkit keystream.

Mode `OFB` menghasilkan keystream dari `IV` melalui proses enkripsi block cipher. Keystream tersebut di-XOR dengan plaintext untuk menghasilkan ciphertext, atau di-XOR dengan ciphertext untuk menghasilkan plaintext. Proses enkripsi dan dekripsi memakai alur komputasi yang sama.

## Struktur File

```text
.
|-- Makefile
|-- README.md
`-- main.c
```

Peran file:

- `main.c`: implementasi block cipher, mode `OFB`, parsing argumen, encoding hex, dan output command line.
- `Makefile`: konfigurasi build dengan `gcc`.
- `README.md`: dokumentasi penggunaan dan ringkasan implementasi.

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

## Penjabaran Kode `main.c`

Bagian ini menjabarkan seluruh isi `main.c` dari awal sampai akhir. Setiap potongan kode ditampilkan berurutan sesuai posisi di file.

### 1. Header, konstanta, dan S-Box

```c
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define BLOCK_SIZE 8
#define KEY_SIZE 16
#define ROUNDS 8

static const uint8_t SBOX[16] = {
    0xE, 0x4, 0xD, 0x1,
    0x2, 0xF, 0xB, 0x8,
    0x3, 0xA, 0x6, 0xC,
    0x5, 0x9, 0x0, 0x7
};
```

Header yang dipakai:

- `stdint.h`: menyediakan tipe integer berukuran tetap seperti `uint8_t` dan `uint32_t`.
- `stdio.h`: menyediakan fungsi input-output seperti `printf`, `fprintf`, dan `fwrite`.
- `stdlib.h`: menyediakan alokasi memori seperti `malloc` dan `free`.
- `string.h`: menyediakan operasi string dan memori seperti `strlen`, `strcmp`, `memset`, dan `memcpy`.

Konstanta program:

- `BLOCK_SIZE` bernilai `8`, berarti satu blok cipher berukuran 8 byte atau 64 bit.
- `KEY_SIZE` bernilai `16`, berarti kunci internal berukuran 16 byte atau 128 bit.
- `ROUNDS` bernilai `8`, berarti jaringan Feistel berjalan sebanyak 8 ronde.

`SBOX` adalah tabel substitusi 16 elemen untuk mengganti nilai nibble 4-bit. Nilai input `0x0` sampai `0xF` dipetakan ke nilai baru sesuai indeks tabel.

### 2. Rotasi dan konversi word 32-bit

```c
static uint32_t rotl32(uint32_t value, unsigned shift) {
    return (value << shift) | (value >> (32U - shift));
}

static uint32_t read_u32_be(const uint8_t *src) {
    return ((uint32_t)src[0] << 24) |
           ((uint32_t)src[1] << 16) |
           ((uint32_t)src[2] << 8) |
           (uint32_t)src[3];
}

static void write_u32_be(uint8_t *dst, uint32_t value) {
    dst[0] = (uint8_t)(value >> 24);
    dst[1] = (uint8_t)(value >> 16);
    dst[2] = (uint8_t)(value >> 8);
    dst[3] = (uint8_t)value;
}
```

`rotl32` melakukan rotasi kiri pada integer 32-bit. Bit yang keluar dari sisi kiri dimasukkan kembali ke sisi kanan.

`read_u32_be` membaca 4 byte dari array lalu membentuk satu `uint32_t` dengan urutan big-endian. Byte pertama menjadi bagian paling tinggi.

`write_u32_be` melakukan kebalikan dari `read_u32_be`. Nilai `uint32_t` dipecah menjadi 4 byte big-endian dan ditulis ke array tujuan.

### 3. Substitusi, permutasi, dan fungsi ronde

```c
static uint32_t substitute_word(uint32_t value) {
    uint32_t out = 0;

    for (int i = 0; i < 8; ++i) {
        uint32_t nibble = (value >> (i * 4)) & 0xFU;
        out |= ((uint32_t)SBOX[nibble]) << (i * 4);
    }

    return out;
}

static uint32_t permute_word(uint32_t value) {
    return rotl32(value, 3) ^ rotl32(value, 11) ^ rotl32(value, 19);
}

static uint32_t round_function(uint32_t right, uint32_t round_key) {
    return permute_word(substitute_word(right ^ round_key));
}
```

`substitute_word` memproses satu word 32-bit sebagai 8 nibble. Setiap nibble diambil dengan shift dan mask `0xFU`, diganti memakai `SBOX`, lalu ditempatkan kembali ke posisi semula.

`permute_word` menghasilkan difusi bit melalui tiga rotasi kiri. Hasil rotasi dengan jarak `3`, `11`, dan `19` digabung memakai `XOR`.

`round_function` adalah fungsi inti per ronde Feistel. Nilai bagian kanan (`right`) dicampur dengan `round_key` memakai `XOR`, masuk ke substitusi S-Box, lalu masuk ke permutasi.

### 4. Pembentukan round key

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

`generate_round_keys` mengubah kunci 16 byte menjadi 8 round key 32-bit. Kunci dibagi menjadi empat word 32-bit: `a`, `b`, `c`, dan `d`.

Pada setiap ronde:

- `a ^ c` dan `b ^ d` mencampur bagian-bagian kunci.
- Hasil campuran dirotasi dengan jarak yang berubah mengikuti indeks ronde.
- Konstanta `0x9E3779B9u` ditambahkan untuk memberi variasi antar ronde.
- `round_keys[i]` dibentuk dari campuran tersebut, digabung dengan rotasi `d`, lalu disubstitusi memakai `SBOX`.
- Variabel `a`, `b`, `c`, dan `d` digeser seperti register kecil agar ronde berikutnya memakai keadaan kunci yang berbeda.

### 5. Enkripsi satu blok

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

`encrypt_block` memproses 8 byte input menjadi 8 byte output. Input dibagi menjadi dua bagian 32-bit: `left` dan `right`.

Setiap ronde Feistel melakukan langkah berikut:

- Bagian kanan lama menjadi bagian kiri baru.
- Bagian kiri lama di-XOR dengan hasil `round_function` dari bagian kanan lama.
- Hasil XOR menjadi bagian kanan baru.

Setelah 8 ronde, bagian `right` ditulis lebih dulu, lalu bagian `left`. Urutan akhir ini adalah swap standar pada struktur Feistel.

### 6. Penyalinan teks ke buffer byte tetap

```c
static void copy_text_bytes(const char *text, uint8_t *dst, size_t dst_len) {
    memset(dst, 0, dst_len);

    for (size_t i = 0; i < dst_len && text[i] != '\0'; ++i) {
        dst[i] = (uint8_t)text[i];
    }
}
```

`copy_text_bytes` mengisi buffer tujuan dengan nol terlebih dahulu, lalu menyalin byte teks sampai batas ukuran tujuan atau sampai akhir string.

Fungsi ini dipakai untuk membentuk kunci 16 byte dan IV 8 byte dari argumen command line. Jika argumen lebih pendek dari ukuran buffer, sisa byte tetap bernilai `0x00`. Jika argumen lebih panjang, byte setelah batas buffer tidak dipakai.

### 7. Proses OFB untuk enkripsi dan dekripsi

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

`ofb_crypt` menjalankan mode `OFB`. Fungsi ini dipakai untuk enkripsi dan dekripsi karena operasi OFB simetris.

Alur di dalam fungsi:

- `generate_round_keys` membentuk round key dari kunci.
- `feedback` diisi dengan `IV`.
- Setiap iterasi memproses maksimal 8 byte.
- `encrypt_block(feedback, stream, round_keys)` menghasilkan blok keystream.
- `feedback` berikutnya diisi dengan `stream`, bukan ciphertext.
- Setiap byte input di-XOR dengan keystream untuk menghasilkan output.

Variabel `chunk` menjaga blok terakhir tetap sesuai panjang data. Jika sisa data kurang dari 8 byte, hanya byte yang tersedia yang diproses.

### 8. Konversi satu karakter hex

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

`hex_value` mengubah satu karakter hexadecimal menjadi nilai integer 0 sampai 15.

Karakter yang diterima:

- `0` sampai `9`
- `a` sampai `f`
- `A` sampai `F`

Karakter selain itu menghasilkan `-1`, yang menandakan input hex tidak valid.

### 9. Konversi string hex ke byte

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

`hex_to_bytes` mengubah ciphertext hex menjadi array byte untuk proses dekripsi.

Pemeriksaan yang dilakukan:

- Panjang hex harus genap karena 2 karakter hex membentuk 1 byte.
- Alokasi memori dilakukan sebesar jumlah byte hasil konversi.
- Setiap pasangan karakter hex dikonversi menjadi nibble atas (`high`) dan nibble bawah (`low`).
- Jika salah satu karakter tidak valid, memori dibebaskan dan fungsi mengembalikan `NULL`.

Ekspresi `(high << 4) | low` menggabungkan dua nibble menjadi satu byte.

### 10. Output hex dan pesan penggunaan

```c
static void print_hex(const uint8_t *data, size_t len) {
    for (size_t i = 0; i < len; ++i) {
        printf("%02X", data[i]);
    }
    printf("\n");
}

static void print_usage(const char *program) {
    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s enc <key16> <iv8> <plaintext>\n", program);
    fprintf(stderr, "  %s dec <key16> <iv8> <ciphertext_hex>\n", program);
}
```

`print_hex` mencetak setiap byte sebagai dua digit hexadecimal uppercase. Format `%02X` memastikan byte seperti `0x0A` dicetak sebagai `0A`, bukan `A`.

`print_usage` mencetak format perintah ke `stderr`. Fungsi ini dipanggil saat jumlah argumen salah atau perintah bukan `enc` maupun `dec`.

### 11. Awal fungsi `main`

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
```

`main` adalah titik masuk program.

Argumen yang diharapkan berjumlah 5:

```text
argv[0] = nama program
argv[1] = perintah enc atau dec
argv[2] = key16
argv[3] = iv8
argv[4] = plaintext atau ciphertext_hex
```

Jika jumlah argumen tidak sesuai, format penggunaan dicetak dan program keluar dengan kode `1`.

Setelah validasi jumlah argumen, `argv[2]` disalin ke buffer `key` berukuran 16 byte dan `argv[3]` disalin ke buffer `iv` berukuran 8 byte.

### 12. Cabang enkripsi

```c
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
```

Cabang ini berjalan jika argumen perintah bernilai `enc`.

Langkah enkripsi:

- Panjang plaintext dihitung dengan `strlen`.
- Buffer `ciphertext` dialokasikan sepanjang plaintext.
- Jika alokasi gagal, program keluar dengan kode `1`.
- `ofb_crypt` memproses plaintext menjadi ciphertext.
- Ciphertext dicetak sebagai hex uppercase.
- Memori ciphertext dibebaskan.
- Program keluar dengan kode `0`.

Ukuran output byte sama dengan ukuran plaintext karena mode OFB tidak memakai padding.

### 13. Cabang dekripsi

```c
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
```

Cabang ini berjalan jika argumen perintah bernilai `dec`.

Langkah dekripsi:

- Ciphertext hex dari `argv[4]` dikonversi ke byte dengan `hex_to_bytes`.
- Jika hex tidak valid, pesan error dicetak ke `stderr`.
- Buffer `plaintext` dialokasikan sepanjang ciphertext byte.
- Jika alokasi gagal, buffer ciphertext dibebaskan sebelum keluar.
- `ofb_crypt` memproses ciphertext menjadi plaintext.
- Plaintext dicetak ke `stdout` memakai `fwrite`.
- Newline dicetak setelah plaintext.
- Memori plaintext dan ciphertext dibebaskan.
- Program keluar dengan kode `0`.

OFB memakai operasi yang sama untuk enkripsi dan dekripsi. Perbedaan cabang ini hanya format input dan output: dekripsi menerima hex dan mencetak teks.

### 14. Perintah tidak dikenal dan akhir program

```c
    print_usage(argv[0]);
    return 1;
}
```

Bagian ini dijalankan jika `argv[1]` bukan `enc` atau `dec`. Format penggunaan dicetak, lalu program keluar dengan kode `1`.

Kurung kurawal terakhir menutup fungsi `main`.

## Penjabaran Kode `Makefile`

Bagian ini menjabarkan seluruh isi `Makefile`.

### 1. Konfigurasi build

```makefile
CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -pedantic -O2
TARGET = block_cipher
SRC = main.c
```

`CC` menentukan compiler yang dipakai, yaitu `gcc`.

`CFLAGS` berisi opsi compiler:

- `-std=c11`: memakai versi C11.
- `-Wall`: mengaktifkan peringatan umum.
- `-Wextra`: mengaktifkan peringatan tambahan.
- `-pedantic`: meminta compiler mengikuti aturan bahasa C dengan lebih ketat.
- `-O2`: mengaktifkan optimisasi level 2.

`TARGET` adalah nama binary hasil build, yaitu `block_cipher`.

`SRC` adalah file sumber yang dikompilasi, yaitu `main.c`.

### 2. Target build utama

```makefile
all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)
```

Target `all` bergantung pada `$(TARGET)`. Saat perintah `make` dijalankan, target ini membangun binary `block_cipher`.

Target `$(TARGET)` bergantung pada `$(SRC)`. Jika `main.c` berubah, perintah kompilasi dijalankan:

```text
gcc -std=c11 -Wall -Wextra -pedantic -O2 -o block_cipher main.c
```

### 3. Target pembersihan dan deklarasi phony

```makefile
clean:
	rm -f $(TARGET)

.PHONY: all clean
```

Target `clean` menghapus binary `block_cipher`.

`.PHONY` menandai `all` dan `clean` sebagai nama target, bukan nama file. Deklarasi ini menjaga perintah tetap berjalan sesuai target walaupun ada file bernama sama.

## Batasan

- Hanya mode `OFB`.
- Tidak membaca input dari `stdin`.
- Tidak memakai file input atau file output.
- Tidak memakai padding.
- Tidak menyediakan opsi raw bytes.
- Cipher bersifat edukatif dan tidak ditujukan sebagai pengganti pustaka kriptografi resmi.
