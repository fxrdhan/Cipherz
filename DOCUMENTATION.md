# OFB Cipher CLI in C: Dokumentasi Teknis

Dokumen ini menjelaskan implementasi `OFB Cipher CLI` secara lengkap dan rinci. Pembahasan mencakup struktur file, spesifikasi program, alur data, desain block cipher, mode operasi `OFB`, pemrosesan argumen command line, encoding hexadecimal, pengelolaan memori, proses build, serta batasan implementasi.

Program terdiri dari satu source file C utama (`main.c`) dan satu `Makefile`. Binary yang dihasilkan bernama `block_cipher`. Binary menerima perintah enkripsi (`enc`) dan dekripsi (`dec`) melalui argumen command line.

## Ringkasan Sistem

Program menyediakan enkripsi dan dekripsi teks dengan mode operasi `Output Feedback` (`OFB`). Mode `OFB` memakai block cipher sebagai generator keystream. Keystream tersebut di-XOR dengan input untuk menghasilkan output.

Karakteristik utama:

- Bahasa implementasi: C.
- Versi bahasa: C11.
- Compiler default: `gcc`.
- Binary hasil build: `block_cipher`.
- Mode operasi: `OFB`.
- Ukuran blok: 8 byte.
- Ukuran kunci: 16 byte.
- Ukuran IV: 8 byte.
- Jumlah ronde Feistel: 8 ronde.
- Output enkripsi: hexadecimal uppercase.
- Input dekripsi: hexadecimal valid dengan panjang genap.
- Input plaintext: satu argumen command line.
- Padding: tidak digunakan.
- Input dari `stdin`: tidak digunakan.
- File input-output: tidak digunakan.

## Antarmuka Command Line

Program memakai format perintah berikut:

```bash
./block_cipher enc <key16> <iv8> <plaintext>
./block_cipher dec <key16> <iv8> <ciphertext_hex>
```

Urutan argumen di dalam `argv`:

```text
argv[0] = nama program
argv[1] = operasi: enc atau dec
argv[2] = key16
argv[3] = iv8
argv[4] = plaintext atau ciphertext_hex
```

Jumlah argumen wajib `5`. Jika jumlah argumen berbeda, program mencetak pesan penggunaan dan keluar dengan kode `1`.

## Format Data

### Key

Argumen `key16` dibaca sebagai string C. Program menyalin byte dari string tersebut ke buffer `key` berukuran 16 byte.

Aturan pemrosesan:

- Maksimal 16 byte pertama dipakai.
- Jika string lebih pendek dari 16 byte, sisa buffer diisi `0x00`.
- Jika string lebih panjang dari 16 byte, byte setelah posisi ke-16 diabaikan.

### IV

Argumen `iv8` dibaca sebagai string C. Program menyalin byte dari string tersebut ke buffer `iv` berukuran 8 byte.

Aturan pemrosesan:

- Maksimal 8 byte pertama dipakai.
- Jika string lebih pendek dari 8 byte, sisa buffer diisi `0x00`.
- Jika string lebih panjang dari 8 byte, byte setelah posisi ke-8 diabaikan.

### Plaintext

Plaintext pada mode `enc` berasal dari `argv[4]`. Panjang plaintext dihitung dengan `strlen`, sehingga plaintext diperlakukan sebagai string biasa. Karena input berasal dari argumen command line, byte `NUL` (`0x00`) tidak dapat menjadi bagian plaintext.

### Ciphertext

Ciphertext pada mode `enc` dicetak sebagai hexadecimal uppercase. Setiap byte ciphertext menjadi dua karakter hex.

Ciphertext pada mode `dec` wajib berupa string hex dengan aturan:

- Panjang string harus genap.
- Karakter valid: `0-9`, `a-f`, `A-F`.
- Setiap dua karakter hex dikonversi menjadi satu byte.

## Model OFB

Mode `OFB` mengubah block cipher menjadi generator keystream. Block cipher tidak langsung mengenkripsi plaintext. Block cipher mengenkripsi `feedback`, lalu hasilnya menjadi `stream`. Stream kemudian di-XOR dengan input.

Rumus:

```text
feedback_0 = IV
stream_i = encrypt_block(feedback_i)
feedback_(i+1) = stream_i
output_i = input_i XOR stream_i
```

Sifat penting:

- Enkripsi dan dekripsi memakai fungsi yang sama.
- Feedback berasal dari output block cipher, bukan dari ciphertext.
- Panjang output sama dengan panjang input.
- Padding tidak diperlukan.
- Blok terakhir boleh lebih pendek dari 8 byte.

## Struktur `main.c`

File `main.c` dapat dibagi menjadi beberapa kelompok:

1. Header dan konstanta.
2. Tabel substitusi `SBOX`.
3. Primitive bit dan konversi word.
4. Fungsi ronde cipher.
5. Key schedule.
6. Enkripsi blok Feistel.
7. Konversi key dan IV dari argumen.
8. Mode `OFB`.
9. Parser hexadecimal.
10. Printer hexadecimal.
11. Pesan penggunaan.
12. Fungsi `main`.

## Header dan Konstanta

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

Header:

- `stdint.h` menyediakan tipe integer berukuran tetap. Tipe ini penting karena cipher membutuhkan ukuran byte dan word yang konsisten.
- `stdio.h` menyediakan fungsi output seperti `printf`, `fprintf`, dan `fwrite`.
- `stdlib.h` menyediakan `malloc` dan `free` untuk alokasi memori dinamis.
- `string.h` menyediakan `strlen`, `strcmp`, `memset`, dan `memcpy`.

Konstanta:

- `BLOCK_SIZE` bernilai `8`. Nilai ini berarti satu blok internal berukuran 8 byte atau 64 bit.
- `KEY_SIZE` bernilai `16`. Nilai ini berarti kunci internal berukuran 16 byte atau 128 bit.
- `ROUNDS` bernilai `8`. Nilai ini berarti proses Feistel berjalan sebanyak 8 ronde.

Ketiga konstanta dipakai sebagai ukuran tetap di seluruh fungsi. Ukuran tetap membuat alur memori dapat ditelusuri langsung dari source code.

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

`SBOX` adalah tabel substitusi untuk nilai 4-bit. Karena 4 bit memiliki 16 kemungkinan nilai, ukuran tabel adalah 16 elemen.

Cara membaca tabel:

```text
input 0x0 -> output 0xE
input 0x1 -> output 0x4
input 0x2 -> output 0xD
...
input 0xF -> output 0x7
```

S-Box memberi unsur non-linear pada cipher. Tanpa substitusi, operasi cipher akan didominasi oleh operasi linear seperti `XOR`, rotasi, dan shift. Substitusi membuat relasi antara input dan output tidak hanya berupa kombinasi linear bit.

Deklarasi memakai `static const`:

- `static` membatasi visibilitas tabel hanya di file `main.c`.
- `const` menandakan isi tabel tidak berubah saat program berjalan.

## Rotasi Kiri 32-bit

Kode:

```c
static uint32_t rotl32(uint32_t value, unsigned shift) {
    return (value << shift) | (value >> (32U - shift));
}
```

Fungsi `rotl32` melakukan rotasi kiri terhadap nilai 32-bit.

Perbedaan rotasi dan shift biasa:

- Shift kiri membuang bit yang keluar dari sisi kiri.
- Rotasi kiri memasukkan kembali bit yang keluar dari sisi kiri ke sisi kanan.

Contoh konseptual:

```text
value = ABCD dalam susunan bit
rotl32(value, n) = bit digeser n posisi ke kiri, bit yang keluar masuk kembali dari kanan
```

Fungsi ini menggabungkan dua operasi:

- `(value << shift)` menghasilkan bagian utama yang bergeser ke kiri.
- `(value >> (32U - shift))` mengambil bit yang keluar dari sisi kiri.
- Operator `|` menggabungkan kedua bagian.

Nilai `shift` yang dipakai dalam kode selalu berada pada rentang aman, seperti `3`, `7`, `11`, `19`, atau nilai kecil dari ekspresi modulo.

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

Fungsi `read_u32_be` membaca 4 byte dan membentuk satu nilai `uint32_t`.

Urutan big-endian:

```text
src[0] = byte paling tinggi
src[1] = byte tinggi berikutnya
src[2] = byte rendah berikutnya
src[3] = byte paling rendah
```

Contoh:

```text
src = {0x12, 0x34, 0x56, 0x78}
hasil = 0x12345678
```

Setiap byte di-cast ke `uint32_t` sebelum shift agar operasi dilakukan pada tipe 32-bit. Tanpa cast, promosi integer tetap terjadi, tetapi cast eksplisit membuat maksud kode lebih jelas.

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

Fungsi `write_u32_be` memecah satu `uint32_t` menjadi 4 byte.

Contoh:

```text
value = 0x12345678
dst[0] = 0x12
dst[1] = 0x34
dst[2] = 0x56
dst[3] = 0x78
```

Cast ke `uint8_t` mengambil 8 bit paling rendah dari hasil shift. Fungsi ini menjadi kebalikan dari `read_u32_be`.

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

Fungsi `substitute_word` membagi word 32-bit menjadi 8 nibble. Satu nibble adalah 4 bit. Karena 32 bit dibagi 4 bit menghasilkan 8 bagian, loop berjalan dari `i = 0` sampai `i = 7`.

Langkah per iterasi:

1. `value >> (i * 4)` menggeser nibble target ke posisi paling rendah.
2. `& 0xFU` mengambil hanya 4 bit paling rendah.
3. `SBOX[nibble]` mengganti nilai nibble dengan nilai substitusi.
4. `<< (i * 4)` mengembalikan nibble hasil substitusi ke posisi awal.
5. `out |= ...` memasukkan nibble hasil substitusi ke word output.

Nilai `out` dimulai dari `0` agar setiap posisi bit kosong sebelum diisi hasil substitusi.

## Permutasi Word

Kode:

```c
static uint32_t permute_word(uint32_t value) {
    return rotl32(value, 3) ^ rotl32(value, 11) ^ rotl32(value, 19);
}
```

Fungsi `permute_word` menghasilkan difusi bit. Nilai input dirotasi dengan tiga jarak berbeda, lalu hasilnya digabung dengan `XOR`.

Tujuan difusi:

- Perubahan kecil pada input dapat memengaruhi beberapa posisi bit pada output.
- Bit dari posisi berbeda bercampur dalam satu word.
- Output fungsi ronde tidak hanya mengubah posisi bit secara langsung, tetapi juga menggabungkan beberapa versi rotasi.

Jarak rotasi `3`, `11`, dan `19` dipilih sebagai jarak yang berbeda dan tidak sejajar dengan batas nibble atau byte.

## Fungsi Ronde

Kode:

```c
static uint32_t round_function(uint32_t right, uint32_t round_key) {
    return permute_word(substitute_word(right ^ round_key));
}
```

`round_function` adalah fungsi inti dalam jaringan Feistel. Fungsi ini menerima bagian kanan blok (`right`) dan satu round key (`round_key`).

Urutan proses:

1. `right ^ round_key`: mencampur data dengan round key.
2. `substitute_word(...)`: memberi substitusi nibble dengan S-Box.
3. `permute_word(...)`: menyebarkan perubahan bit melalui rotasi dan XOR.

Output fungsi ini berukuran 32 bit dan dipakai untuk menghasilkan bagian kanan baru pada ronde Feistel.

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

Fungsi `generate_round_keys` menerima kunci 16 byte dan menghasilkan 8 round key. Setiap round key berukuran 32 bit.

### Pemecahan kunci

Kunci 16 byte dibagi menjadi empat word:

```text
a = key[0..3]
b = key[4..7]
c = key[8..11]
d = key[12..15]
```

Semua word dibaca dengan urutan big-endian melalui `read_u32_be`.

### Campuran per ronde

Variabel `mix` dibentuk dari:

- `rotl32(a ^ c, (i % 7) + 1)`.
- `rotl32(b ^ d, ((i + 2) % 7) + 1)`.
- Konstanta ronde `0x9E3779B9u * (i + 1)`.

Ekspresi `a ^ c` dan `b ^ d` mencampur word yang berbeda dari kunci. Rotasi dengan jarak berubah membuat setiap ronde memakai pola bit berbeda.

Konstanta `0x9E3779B9u` adalah konstanta 32-bit yang sering dipakai dalam desain pencampuran integer. Dalam kode ini, konstanta tersebut memberi variasi deterministik antar ronde.

### Round key

Round key dibentuk dengan:

```text
round_keys[i] = substitute_word(mix XOR rotl32(d, ((i + 4) % 9) + 1))
```

Nilai `d` ikut dirotasi dan dicampur dengan `mix`. Hasilnya disubstitusi dengan S-Box agar round key tidak hanya berasal dari operasi linear.

### Pergeseran state key schedule

Setelah round key dibuat, state internal key schedule digeser:

```text
next = a XOR rotl32(round_keys[i], 7) XOR (0xA5A5A5A5 + i * 0x01010101)
a = b
b = c
c = d
d = next
```

Pola ini membuat ronde berikutnya memakai kombinasi word berbeda. Variabel `a`, `b`, `c`, dan `d` berperan sebagai register 32-bit internal untuk proses key schedule.

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

Fungsi `encrypt_block` menerima 8 byte input dan menghasilkan 8 byte output.

### Pemecahan blok

Blok 8 byte dibagi menjadi dua bagian 32-bit:

```text
left  = in[0..3]
right = in[4..7]
```

Kedua bagian dibaca dengan format big-endian.

### Ronde Feistel

Setiap ronde mengikuti pola:

```text
next_left = right
next_right = left XOR round_function(right, round_key)
left = next_left
right = next_right
```

Dengan pola ini, bagian kanan lama berpindah ke kiri, sementara bagian kanan baru berasal dari bagian kiri lama yang dicampur dengan hasil fungsi ronde.

### Output blok

Setelah 8 ronde, output ditulis dengan urutan:

```text
out[0..3] = right
out[4..7] = left
```

Urutan ini melakukan swap akhir. Swap ini lazim pada jaringan Feistel karena setiap ronde sudah menukar posisi kiri dan kanan.

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

Fungsi `copy_text_bytes` dipakai untuk membentuk key dan IV dari argumen command line.

Langkah:

1. `memset(dst, 0, dst_len)` mengisi seluruh buffer tujuan dengan nol.
2. Loop menyalin byte teks selama indeks masih kurang dari `dst_len`.
3. Loop berhenti lebih awal jika karakter akhir string `'\0'` ditemukan.

Efek:

- Input yang lebih pendek dari buffer menghasilkan padding nol.
- Input yang lebih panjang dari buffer terpotong pada batas buffer.
- Tidak ada penulisan melewati ukuran buffer karena kondisi loop selalu memeriksa `i < dst_len`.

Fungsi ini tidak menambahkan terminator string ke `dst` sebagai string C. Buffer `dst` dipakai sebagai byte array untuk cipher, bukan sebagai string.

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

Fungsi `ofb_crypt` adalah inti pemrosesan data. Fungsi yang sama dipakai untuk enkripsi dan dekripsi.

### Buffer lokal

Buffer lokal:

- `round_keys`: menyimpan 8 round key 32-bit.
- `feedback`: menyimpan state feedback 8 byte.
- `stream`: menyimpan output block cipher 8 byte untuk satu blok OFB.

### Inisialisasi

```text
generate_round_keys(key, round_keys)
feedback = IV
```

Round key dibentuk satu kali pada awal fungsi. Feedback awal berasal dari IV.

### Loop per blok

Loop berjalan dengan langkah `BLOCK_SIZE`:

```text
offset = 0, 8, 16, ...
```

Variabel `chunk` menentukan jumlah byte yang diproses pada iterasi tersebut. Jika sisa data lebih dari 8 byte, `chunk = 8`. Jika sisa data kurang dari 8 byte, `chunk` berisi jumlah byte sisa.

### Pembentukan keystream

```text
stream = encrypt_block(feedback)
feedback = stream
```

Pada OFB, feedback baru adalah output block cipher. Ciphertext tidak masuk ke feedback. Hal ini membedakan OFB dari CFB.

### XOR input dengan stream

```text
output[offset + i] = input[offset + i] XOR stream[i]
```

Operasi ini berlaku sama untuk plaintext dan ciphertext:

- Enkripsi: plaintext XOR keystream = ciphertext.
- Dekripsi: ciphertext XOR keystream = plaintext.

Sifat XOR membuat proses dapat dibalik:

```text
(plaintext XOR stream) XOR stream = plaintext
```

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

Fungsi `hex_value` menerima satu karakter dan menghasilkan nilai 0 sampai 15.

Rentang valid:

- `'0'` sampai `'9'` menghasilkan 0 sampai 9.
- `'a'` sampai `'f'` menghasilkan 10 sampai 15.
- `'A'` sampai `'F'` menghasilkan 10 sampai 15.

Jika karakter tidak termasuk rentang tersebut, fungsi mengembalikan `-1`.

Nilai `-1` dipakai oleh `hex_to_bytes` sebagai tanda bahwa input dekripsi tidak valid.

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

Fungsi `hex_to_bytes` dipakai pada cabang dekripsi.

### Pemeriksaan panjang

```text
if ((hex_len % 2) != 0) return NULL
```

Dua karakter hex membentuk satu byte. Panjang ganjil tidak dapat dikonversi secara lengkap.

### Alokasi output

```text
*out_len = hex_len / 2
bytes = malloc(*out_len == 0 ? 1 : *out_len)
```

Jika input hex kosong, jumlah byte hasil adalah 0. Alokasi tetap memakai ukuran 1 untuk menghindari perilaku implementasi yang berbeda pada `malloc(0)`.

### Loop konversi

Loop membaca pasangan karakter:

```text
hex[i]     = nibble atas
hex[i + 1] = nibble bawah
```

Setiap pasangan digabung menjadi byte:

```text
byte = (high << 4) | low
```

Jika `high` atau `low` bernilai negatif, input mengandung karakter non-hex. Dalam kondisi tersebut, buffer yang sudah dialokasikan dibebaskan, lalu fungsi mengembalikan `NULL`.

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

Fungsi `print_hex` mencetak byte array sebagai hexadecimal uppercase.

Format `%02X`:

- `X` mencetak angka hexadecimal uppercase.
- `2` meminta lebar minimal 2 karakter.
- `0` mengisi posisi kosong dengan nol.

Contoh:

```text
0x0A -> 0A
0x7C -> 7C
0xFF -> FF
```

Newline dicetak setelah seluruh byte selesai agar output command line rapi.

## Pesan Penggunaan

Kode:

```c
static void print_usage(const char *program) {
    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s enc <key16> <iv8> <plaintext>\n", program);
    fprintf(stderr, "  %s dec <key16> <iv8> <ciphertext_hex>\n", program);
}
```

Fungsi `print_usage` mencetak format penggunaan ke `stderr`. Parameter `program` berasal dari `argv[0]`, sehingga nama program pada pesan mengikuti cara binary dipanggil.

Output diarahkan ke `stderr` karena pesan ini muncul pada kondisi error argumen atau operasi tidak dikenal.

## Fungsi `main`

Kode lengkap fungsi `main`:

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

### Alokasi key dan IV

```c
uint8_t key[KEY_SIZE];
uint8_t iv[BLOCK_SIZE];
```

`key` berada di stack dengan ukuran 16 byte. `iv` berada di stack dengan ukuran 8 byte. Ukuran keduanya berasal dari macro agar konsisten dengan fungsi lain.

### Validasi jumlah argumen

```c
if (argc != 5) {
    print_usage(argv[0]);
    return 1;
}
```

Program hanya menerima format tetap dengan 4 argumen setelah nama program. Kode keluar `1` menandakan eksekusi gagal.

### Konversi key dan IV

```c
copy_text_bytes(argv[2], key, KEY_SIZE);
copy_text_bytes(argv[3], iv, BLOCK_SIZE);
```

Argumen key dan IV dikonversi menjadi byte array tetap. Pemanggilan ini terjadi sebelum cabang `enc` dan `dec` karena kedua operasi membutuhkan key dan IV.

### Cabang enkripsi

```c
if (strcmp(argv[1], "enc") == 0) {
    ...
}
```

`strcmp` mengembalikan 0 jika string sama. Jika `argv[1]` adalah `enc`, program menjalankan alur enkripsi.

Langkah detail:

1. `strlen(argv[4])` menghitung panjang plaintext.
2. `malloc(len == 0 ? 1 : len)` membuat buffer ciphertext.
3. Jika alokasi gagal, program keluar dengan kode `1`.
4. `ofb_crypt` menghasilkan ciphertext.
5. `print_hex` mencetak ciphertext sebagai hex.
6. `free(ciphertext)` membebaskan memori.
7. `return 0` menandakan sukses.

### Cabang dekripsi

```c
if (strcmp(argv[1], "dec") == 0) {
    ...
}
```

Jika `argv[1]` adalah `dec`, program menjalankan alur dekripsi.

Langkah detail:

1. `hex_to_bytes(argv[4], &len)` mengubah hex menjadi byte.
2. Jika hasil `NULL`, pesan error hex dicetak dan program keluar.
3. Buffer plaintext dialokasikan sepanjang ciphertext byte.
4. Jika alokasi gagal, ciphertext dibebaskan sebelum keluar.
5. `ofb_crypt` mengubah ciphertext menjadi plaintext.
6. `fwrite` mencetak plaintext berdasarkan panjang byte, bukan berdasarkan terminator string.
7. `printf("\n")` menambahkan newline.
8. `free(plaintext)` dan `free(ciphertext)` membebaskan memori.
9. `return 0` menandakan sukses.

Penggunaan `fwrite` membuat plaintext hasil dekripsi dicetak sesuai panjang yang dihitung, walaupun tidak ditambah terminator `'\0'`.

### Operasi tidak dikenal

Jika perintah bukan `enc` atau `dec`, eksekusi sampai ke bagian akhir:

```c
print_usage(argv[0]);
return 1;
```

Program mencetak format penggunaan dan keluar dengan kode gagal.

## Alur Enkripsi Lengkap

Contoh perintah:

```bash
./block_cipher enc KAMSIS-KEY-2026 IV2026!! "halo"
```

Alur internal:

1. `argc` bernilai 5, sehingga validasi jumlah argumen lolos.
2. `argv[1]` bernilai `enc`.
3. `argv[2]` dikonversi menjadi key 16 byte.
4. `argv[3]` dikonversi menjadi IV 8 byte.
5. Panjang plaintext `halo` adalah 4 byte.
6. Buffer ciphertext 4 byte dialokasikan.
7. `ofb_crypt` membentuk round key dari key.
8. Feedback awal diisi IV.
9. `encrypt_block` mengenkripsi feedback menjadi keystream 8 byte.
10. Empat byte plaintext di-XOR dengan empat byte pertama keystream.
11. Ciphertext dicetak sebagai hex uppercase.
12. Memori ciphertext dibebaskan.

Output contoh:

```text
7CBBFA5F
```

## Alur Dekripsi Lengkap

Contoh perintah:

```bash
./block_cipher dec KAMSIS-KEY-2026 IV2026!! 7CBBFA5F
```

Alur internal:

1. `argc` bernilai 5, sehingga validasi jumlah argumen lolos.
2. `argv[1]` bernilai `dec`.
3. `argv[2]` dikonversi menjadi key 16 byte.
4. `argv[3]` dikonversi menjadi IV 8 byte.
5. `argv[4]` dikonversi dari hex menjadi byte ciphertext.
6. Hex `7CBBFA5F` menjadi 4 byte ciphertext.
7. Buffer plaintext 4 byte dialokasikan.
8. `ofb_crypt` membentuk keystream sama seperti proses enkripsi karena key dan IV sama.
9. Ciphertext di-XOR dengan keystream.
10. Hasil XOR menghasilkan plaintext asli.
11. Plaintext dicetak dengan `fwrite`.
12. Memori plaintext dan ciphertext dibebaskan.

Output contoh:

```text
halo
```

## Pengelolaan Memori

Program memakai dua jenis penyimpanan:

- Stack untuk buffer berukuran tetap seperti `key`, `iv`, `round_keys`, `feedback`, dan `stream`.
- Heap untuk buffer dengan ukuran mengikuti input seperti `ciphertext` dan `plaintext`.

### Buffer stack

Buffer stack:

```text
key[16]
iv[8]
round_keys[8]
feedback[8]
stream[8]
```

Semua akses buffer stack dibatasi oleh macro ukuran atau variabel `chunk`.

### Buffer heap pada enkripsi

Pada enkripsi:

```c
uint8_t *ciphertext = malloc(len == 0 ? 1 : len);
```

Ukuran alokasi mengikuti panjang plaintext. Setelah dipakai, buffer dibebaskan:

```c
free(ciphertext);
```

### Buffer heap pada dekripsi

Pada dekripsi, `hex_to_bytes` mengalokasikan ciphertext byte. Setelah itu `main` mengalokasikan plaintext.

Jika alokasi plaintext gagal, ciphertext dibebaskan sebelum keluar:

```c
if (plaintext == NULL) {
    free(ciphertext);
    return 1;
}
```

Pada jalur sukses, dua buffer dibebaskan:

```c
free(plaintext);
free(ciphertext);
```

### Pembatasan akses blok terakhir

Pada mode OFB:

```c
size_t chunk = len - offset;
if (chunk > BLOCK_SIZE) {
    chunk = BLOCK_SIZE;
}
```

Loop XOR memakai `i < chunk`, sehingga blok terakhir yang kurang dari 8 byte tetap aman. Tidak ada pembacaan melewati akhir input dan tidak ada penulisan melewati akhir output selama pemanggil menyediakan buffer output sepanjang `len`.

## Perilaku Error

Kondisi error:

- Jumlah argumen salah.
- Perintah bukan `enc` atau `dec`.
- Alokasi memori gagal.
- Ciphertext hex memiliki panjang ganjil.
- Ciphertext hex mengandung karakter non-hex.

Kode keluar:

- `0`: operasi berhasil.
- `1`: operasi gagal.

Pesan error yang eksplisit:

- Format penggunaan untuk argumen salah atau perintah tidak dikenal.
- `Ciphertext harus berupa hex valid.` untuk input hex dekripsi yang tidak valid.

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

## Penjelasan Makefile

### Variabel build

```makefile
CC = gcc
CFLAGS = -std=c11 -Wall -Wextra -pedantic -O2
TARGET = block_cipher
SRC = main.c
```

`CC` menentukan compiler. Nilainya `gcc`.

`CFLAGS` menentukan opsi compiler:

- `-std=c11`: menggunakan C11.
- `-Wall`: mengaktifkan peringatan umum.
- `-Wextra`: mengaktifkan peringatan tambahan.
- `-pedantic`: memperketat kepatuhan terhadap aturan C.
- `-O2`: mengaktifkan optimisasi tingkat 2.

`TARGET` menentukan nama binary hasil build. `SRC` menentukan file sumber.

### Target default

```makefile
all: $(TARGET)
```

Target `all` adalah target default. Target ini bergantung pada `block_cipher`.

### Target binary

```makefile
$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)
```

Jika `main.c` berubah, target `block_cipher` dibangun ulang. Perintah yang dijalankan:

```text
gcc -std=c11 -Wall -Wextra -pedantic -O2 -o block_cipher main.c
```

### Target clean

```makefile
clean:
	rm -f $(TARGET)
```

Target `clean` menghapus binary `block_cipher`. Opsi `-f` membuat perintah tetap berhasil walaupun file belum ada.

### Deklarasi phony

```makefile
.PHONY: all clean
```

`all` dan `clean` dideklarasikan sebagai target phony agar `make` memperlakukan keduanya sebagai perintah, bukan sebagai file biasa.

## Batasan Implementasi

- Mode operasi hanya `OFB`.
- Tidak ada mode `CBC` atau `CFB`.
- Tidak ada padding.
- Tidak ada pembacaan dari `stdin`.
- Tidak ada penulisan ke file.
- Tidak ada opsi raw bytes.
- Plaintext dari command line tidak dapat memuat byte `NUL`.
- Key dan IV berasal dari argumen teks.
- Cipher kustom ini bersifat edukatif dan tidak ditujukan sebagai pengganti pustaka kriptografi resmi.

## Ringkasan Alur Fungsi

```text
main
|-- validasi argc
|-- copy_text_bytes key
|-- copy_text_bytes iv
|-- enc
|   |-- strlen plaintext
|   |-- malloc ciphertext
|   |-- ofb_crypt
|   |   |-- generate_round_keys
|   |   |-- encrypt_block feedback
|   |   |-- XOR input dengan stream
|   |-- print_hex
|   `-- free ciphertext
`-- dec
    |-- hex_to_bytes
    |-- malloc plaintext
    |-- ofb_crypt
    |   |-- generate_round_keys
    |   |-- encrypt_block feedback
    |   `-- XOR input dengan stream
    |-- fwrite plaintext
    |-- free plaintext
    `-- free ciphertext
```

## Ringkasan Operasi Bit

Operasi bit yang muncul dalam implementasi:

- `^`: XOR untuk pencampuran data.
- `|`: OR untuk menggabungkan bagian bit.
- `&`: AND untuk mengambil nibble.
- `<<`: shift kiri untuk memindahkan byte atau nibble ke posisi tinggi.
- `>>`: shift kanan untuk membaca byte, nibble, atau bagian rotasi.

Operasi aritmetika yang muncul:

- `+`: penjumlahan dalam key schedule.
- `*`: perkalian konstanta ronde dengan indeks.
- `%`: modulo untuk memilih jarak rotasi yang berubah per ronde.

Operasi memori yang muncul:

- `memset`: mengisi buffer dengan nol.
- `memcpy`: menyalin IV atau stream antar buffer.
- `malloc`: mengalokasikan buffer input-output dinamis.
- `free`: membebaskan buffer dinamis.

## Catatan Keamanan Memori

Implementasi menghindari penulisan melewati buffer melalui beberapa pola:

- Penyalinan key dan IV memakai batas `dst_len`.
- Pemrosesan OFB memakai `chunk` untuk membatasi blok terakhir.
- Output enkripsi dialokasikan sepanjang plaintext.
- Output dekripsi dialokasikan sepanjang ciphertext byte.
- Parser hex memeriksa panjang genap sebelum membaca pasangan karakter.
- Parser hex membebaskan buffer jika menemukan karakter tidak valid.

Program tetap memiliki batas penggunaan karena input berasal dari argumen command line dan plaintext dihitung dengan `strlen`. Data biner bebas byte `NUL` tidak menjadi target pemrosesan antarmuka ini.
