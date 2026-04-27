# Persiapan Uji Perubahan Kode

File ini dibuat sebagai pegangan kalau dosen mengetes pemahaman dengan cara
meminta perubahan kecil pada kode, lalu menanyakan dampaknya.

Fokus utamanya bukan menghafal output, tetapi memahami:

- bagian kode mana yang berubah,
- apakah program masih bisa dikompilasi,
- apakah ciphertext berubah,
- apakah enkripsi dan dekripsi masih bisa round-trip,
- apakah ada risiko buffer, memori, atau input invalid.

Kode utama ada di `main.c`.

## Cara Menjawab Saat Dosen Mengubah Kode

Gunakan pola jawaban ini:

```text
Yang berubah ada di fungsi ...
Dampaknya ke ...
Program masih aman/tidak aman karena ...
Output ciphertext kemungkinan berubah/tidak berubah.
Round-trip masih bisa/tidak bisa jika enkripsi dan dekripsi memakai versi kode yang sama.
```

Contoh:

```text
Kalau SBOX diganti, yang berubah adalah hasil substitute_word.
Karena round_function memakai substitute_word, maka round key dan hasil block cipher
juga berubah. Ciphertext contoh pasti berubah. Namun dekripsi masih bisa balik
ke plaintext selama key, IV, dan versi program yang dipakai sama.
```

## Peta Cepat Dependensi Fungsi

| Bagian | Dipakai Oleh | Kalau Diubah |
| --- | --- | --- |
| `SBOX` | `substitute_word` | Round key dan keystream berubah |
| `rotl32` | `permute_word`, `generate_round_keys` | Difusi bit dan round key berubah |
| `read_u32_be` | `generate_round_keys`, `encrypt_block` | Cara byte dibaca jadi word berubah |
| `write_u32_be` | `encrypt_block` | Urutan byte output block berubah |
| `substitute_word` | `round_function`, `generate_round_keys` | Non-linearitas cipher berubah |
| `permute_word` | `round_function` | Penyebaran bit berubah |
| `round_function` | `encrypt_block` | Isi setiap ronde Feistel berubah |
| `generate_round_keys` | `ofb_crypt` | Semua round key berubah |
| `encrypt_block` | `ofb_crypt` | Keystream OFB berubah |
| `copy_text_bytes` | `main` | Cara key dan IV dari argumen dibentuk berubah |
| `ofb_crypt` | `main enc`, `main dec` | Inti enkripsi/dekripsi berubah |
| `hex_value` | `hex_to_bytes` | Validasi dan parsing hex berubah |
| `hex_to_bytes` | `main dec` | Cara ciphertext hex diubah ke byte berubah |
| `print_hex` | `main enc` | Format tampilan ciphertext berubah |

## Baseline Yang Harus Kamu Hafal

Build:

```bash
make
```

Enkripsi contoh 1 block parsial:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! "arif"
```

Output:

```text
332B5FA1
```

Enkripsi contoh 2 block:

```bash
./block_cipher enc KUNCI-1NGG121S ADA123!! "firdaus arif"
```

Output:

```text
343044A3CF442F1AB954B9C6
```

Dekripsi:

```bash
./block_cipher dec KUNCI-1NGG121S ADA123!! 343044A3CF442F1AB954B9C6
```

Output:

```text
firdaus arif
```

Jawaban inti:

```text
Panjang ciphertext sama dengan plaintext karena OFB tidak memakai padding.
Enkripsi dan dekripsi memakai fungsi ofb_crypt yang sama karena XOR bisa dibalik
dengan XOR yang sama.
```

## Skenario Perubahan Macro

### `ROUNDS` Diubah

Contoh:

```c
#define ROUNDS 4
```

Dampak:

- Program masih bisa dikompilasi.
- Jumlah ronde Feistel berkurang.
- Round key yang dibuat lebih sedikit.
- Keystream berubah, sehingga ciphertext contoh berubah.
- Round-trip masih bisa jika enkripsi dan dekripsi memakai program versi baru yang sama.
- Secara konsep keamanan/difusi lebih lemah karena data diproses lebih sedikit ronde.

Jawaban pendek:

```text
ROUNDS mengatur panjang array round_keys dan jumlah iterasi di generate_round_keys
serta encrypt_block. Kalau nilainya berubah, algoritma cipher berubah, jadi output
berubah. Tetapi OFB tetap bisa dekripsi selama key, IV, dan versi algoritmanya sama.
```

### `KEY_SIZE` Diubah

Contoh:

```c
#define KEY_SIZE 8
```

Dampak:

- Berbahaya kalau hanya macro yang diubah.
- `generate_round_keys` tetap membaca `key`, `key + 4`, `key + 8`, dan `key + 12`.
- Jika `KEY_SIZE` menjadi 8, pembacaan `key + 8` dan `key + 12` keluar dari buffer.
- Ini risiko undefined behavior.

Jawaban pendek:

```text
KEY_SIZE tidak bisa diubah sendirian. Key schedule saat ini didesain untuk 16 byte
karena dibagi menjadi empat word 32-bit. Kalau key dibuat 8 byte, generate_round_keys
juga harus didesain ulang.
```

Kalau `KEY_SIZE` dinaikkan menjadi 32:

- Program bisa saja tetap jalan.
- `copy_text_bytes` menyalin sampai 32 byte.
- Tetapi `generate_round_keys` saat ini hanya memakai 16 byte pertama.
- Jadi tambahan byte ke-17 sampai ke-32 tidak berpengaruh sebelum key schedule diubah.

### `BLOCK_SIZE` Diubah

Contoh:

```c
#define BLOCK_SIZE 16
```

Dampak:

- Tidak aman kalau hanya macro yang diubah.
- `encrypt_block` secara logika masih cipher 64-bit: membaca 4 byte kiri dan 4 byte kanan.
- Jika `BLOCK_SIZE` menjadi 16, `ofb_crypt` akan menganggap stream 16 byte, padahal
  `encrypt_block` hanya menulis 8 byte bermakna.
- Byte stream sisanya bisa tidak valid.

Jika `BLOCK_SIZE` menjadi 4:

- `encrypt_block` membaca `in + 4`, padahal blok hanya 4 byte.
- Ini out-of-bounds.

Jawaban pendek:

```text
BLOCK_SIZE terikat langsung dengan desain encrypt_block. Cipher ini 64-bit, jadi
BLOCK_SIZE harus 8. Kalau ingin 16 byte, encrypt_block harus didesain ulang menjadi
cipher 128-bit atau memakai dua block 64-bit.
```

## Skenario Perubahan S-Box Dan Operasi Bit

### Isi `SBOX` Diganti

Dampak:

- Program masih bisa dikompilasi jika tabel tetap 16 elemen.
- Hasil `substitute_word` berubah.
- `generate_round_keys` ikut berubah karena memakai `substitute_word`.
- `round_function` ikut berubah.
- Keystream dan ciphertext berubah.
- Round-trip tetap bisa dengan program versi yang sama.

Jawaban pendek:

```text
SBOX adalah tabel substitusi nibble. Kalau isinya berubah, non-linearitas cipher
berubah. Karena SBOX dipakai di key schedule dan round function, efeknya menyebar
ke round key dan keystream.
```

### `SBOX` Dibuat Identity

Contoh:

```c
static const uint8_t SBOX[16] = {
  0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7,
  0x8, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF
};
```

Dampak:

- Program masih jalan.
- Substitusi tidak benar-benar mengubah nilai.
- Cipher menjadi lebih lemah karena kehilangan bagian non-linear.
- Output tetap berubah dibanding baseline karena key schedule dan ronde berubah
  mengikuti SBOX baru.

Jawaban pendek:

```text
Secara struktur valid, tetapi secara konsep buruk karena SBOX identity tidak memberi
substitusi. Cipher masih menghasilkan output, tapi kualitas pencampuran datanya turun.
```

### Nilai `SBOX` Lebih Dari `0xF`

Dampak:

- Program mungkin tetap compile karena elemennya `uint8_t`.
- Tetapi `substitute_word` mengharapkan output 4-bit.
- Nilai lebih dari `0xF` bisa mengisi lebih dari satu nibble setelah digeser.
- Ini merusak konsep substitusi per nibble.

Jawaban pendek:

```text
SBOX sebaiknya berisi nilai 0 sampai 15 karena indeks dan outputnya sama-sama nibble.
Kalau lebih dari 0xF, hasil substitusi bisa melebar dan mengganggu posisi nibble lain.
```

### Rotasi Diubah Menjadi Shift Biasa

Contoh:

```c
return value << shift;
```

Dampak:

- Program masih bisa compile.
- Bit yang keluar dari sisi kiri hilang, tidak dimasukkan lagi ke kanan.
- Difusi lebih lemah.
- Keystream dan ciphertext berubah.
- Round-trip tetap bisa dengan versi program yang sama.

Jawaban pendek:

```text
Rotasi menjaga semua bit tetap ada, hanya posisinya berubah. Shift biasa membuang bit.
Jadi hasil difusi berbeda dan sebagian informasi bit hilang dari operasi tersebut.
```

### `rotl32(value, 0)` Dipakai

Dampak:

- Implementasi saat ini tidak aman untuk shift 0.
- Ekspresi `value >> (32U - shift)` menjadi `value >> 32`.
- Shift sebesar lebar tipe di C adalah undefined behavior.
- Pemanggilan yang ada sekarang aman karena shift selalu 1 sampai 19, tidak pernah 0.

Jawaban pendek:

```text
Fungsi rotl32 saat ini aman karena semua pemanggil memberi shift bukan 0 dan bukan 32.
Kalau ingin mendukung shift 0, fungsi perlu ditambah guard seperti `shift %= 32`.
```

## Skenario Perubahan Endianness

### `read_u32_be` Diganti Little-Endian

Dampak:

- Cara 4 byte dibentuk menjadi `uint32_t` berubah.
- Round key berubah karena key dibaca dengan urutan berbeda.
- Blok feedback juga dibaca berbeda di `encrypt_block`.
- Ciphertext contoh berubah.
- Round-trip masih bisa jika enkripsi dan dekripsi memakai versi baru yang sama.
- Ciphertext lama dari versi big-endian tidak cocok lagi dengan versi little-endian.

Jawaban pendek:

```text
Endianness menentukan urutan byte saat masuk ke word 32-bit. Kalau diubah,
algoritmanya berubah. Program masih bisa konsisten secara internal, tapi output
baseline tidak akan sama.
```

### `read_u32_be` Dan `write_u32_be` Tidak Konsisten

Dampak:

- Program bisa tetap berjalan.
- Tetapi representasi byte masuk dan byte keluar tidak lagi simetris secara konsep.
- Keystream berubah.
- Membandingkan dengan dokumentasi jadi tidak cocok.

Jawaban pendek:

```text
Keduanya sebaiknya dipertahankan konsisten. read mengubah byte ke word, write
mengubah word kembali ke byte. Kalau salah satu saja diubah, format internal
cipher berubah sebagian.
```

## Skenario Perubahan `encrypt_block`

### Swap Akhir Dihapus

Kode sekarang:

```c
write_u32_be(out, right);
write_u32_be(out + 4, left);
```

Jika diubah menjadi:

```c
write_u32_be(out, left);
write_u32_be(out + 4, right);
```

Dampak:

- Output block cipher berubah.
- Keystream OFB berubah.
- Ciphertext contoh berubah.
- Round-trip tetap bisa dengan versi program yang sama karena enkripsi dan dekripsi
  tetap menghasilkan keystream yang sama.

Jawaban pendek:

```text
Di project ini encrypt_block dipakai sebagai generator keystream OFB. Kalau swap akhir
diubah, keystream berubah. Tetapi dekripsi tetap bisa selama generate keystream-nya
sama dengan saat enkripsi.
```

### `next_left` Dan `next_right` Tidak Dipakai

Contoh salah:

```c
left = right;
right = left ^ round_function(right, round_keys[i]);
```

Dampak:

- Bug logika.
- Nilai `left` lama hilang sebelum dipakai menghitung `right` baru.
- Ronde Feistel tidak lagi sesuai rumus.
- Output berubah dan kualitas cipher rusak.

Jawaban pendek:

```text
next_left dan next_right dipakai supaya left lama dan right lama masih tersedia
sampai dua nilai baru selesai dihitung. Kalau left langsung ditimpa, rumus Feistel
berubah karena right baru memakai left yang sudah berubah.
```

### Fungsi `decrypt_block` Ditambahkan

Dampak:

- Untuk mode OFB, `decrypt_block` tidak diperlukan.
- OFB hanya membutuhkan enkripsi block cipher untuk membuat keystream.
- Dekripsi data dilakukan dengan XOR ciphertext dan keystream.

Jawaban pendek:

```text
Di OFB, block cipher hanya dipakai pada arah enkripsi untuk menghasilkan keystream.
Karena itu main dec tetap memanggil ofb_crypt, bukan decrypt_block.
```

## Skenario Perubahan `ofb_crypt`

### Feedback Diubah Dari `stream` Menjadi `input`

Kode sekarang:

```c
encrypt_block(feedback, stream, round_keys);
memcpy(feedback, stream, BLOCK_SIZE);
```

Jika feedback diubah dari input/ciphertext:

```c
memcpy(feedback, input + offset, BLOCK_SIZE);
```

Dampak:

- Itu bukan OFB lagi.
- Untuk block terakhir parsial, bisa membaca keluar dari input jika tetap copy
  `BLOCK_SIZE`.
- Enkripsi dan dekripsi tidak otomatis bisa memakai fungsi yang sama.
- Pada mode lain seperti CFB, aturan feedback enkripsi dan dekripsi harus hati-hati.

Jawaban pendek:

```text
OFB memakai output block cipher sebagai feedback, bukan plaintext dan bukan ciphertext.
Kalau feedback diganti input, sifat OFB hilang dan fungsi yang sama belum tentu bisa
dipakai untuk enkripsi dan dekripsi.
```

### `memcpy(feedback, stream, BLOCK_SIZE)` Dipindah Setelah XOR

Dampak:

- Untuk kode saat ini, hasilnya tetap sama.
- `stream` tidak berubah selama proses XOR.
- Feedback berikutnya tetap berisi stream yang sama.

Jawaban pendek:

```text
Kalau hanya dipindah setelah loop XOR, hasilnya tidak berubah karena stream tidak
dimodifikasi. Yang penting feedback berikutnya tetap stream, bukan input/output.
```

### Loop XOR Dipaksa Selalu 8 Byte

Contoh salah:

```c
for (size_t i = 0; i < BLOCK_SIZE; ++i) {
  output[offset + i] = input[offset + i] ^ stream[i];
}
```

Dampak:

- Berbahaya untuk plaintext yang panjangnya bukan kelipatan 8.
- Pada block terakhir parsial, program membaca dan menulis melewati panjang input/output.
- Contoh `"arif"` panjang 4 byte akan terkena out-of-bounds.

Jawaban pendek:

```text
Variabel chunk wajib dipakai karena block terakhir bisa kurang dari 8 byte. Kalau
selalu 8 byte, program tidak aman untuk input parsial.
```

### Padding Ditambahkan

Dampak:

- OFB sebenarnya tidak membutuhkan padding karena bekerja per byte.
- Jika padding ditambahkan, panjang ciphertext bisa lebih panjang dari plaintext.
- Dekripsi harus punya logika menghapus padding.
- Dokumentasi dan ekspektasi output harus berubah.

Jawaban pendek:

```text
Padding tidak diperlukan di OFB. Padding biasanya diperlukan di mode block yang harus
memproses block penuh. Di sini block terakhir cukup memakai sebagian keystream.
```

## Skenario Perubahan Key Dan IV

### `memset(dst, 0, dst_len)` Dihapus Dari `copy_text_bytes`

Dampak:

- Key atau IV pendek tidak lagi dipad dengan nol secara pasti.
- Sisa buffer berisi nilai lama/tidak terinisialisasi dari stack.
- Output bisa tidak deterministik.
- Ini bug serius.

Jawaban pendek:

```text
memset penting supaya key dan IV pendek punya sisa byte 0x00. Kalau dihapus,
sisa buffer bisa berisi data acak dari stack dan hasil enkripsi bisa berubah-ubah.
```

### Batas `i < dst_len` Dihapus

Dampak:

- Input key atau IV panjang bisa menulis melewati buffer.
- Ini buffer overflow.
- Program bisa crash atau merusak data lain di stack.

Jawaban pendek:

```text
Kondisi i < dst_len adalah pelindung buffer. Tanpa itu, argumen panjang bisa menulis
melewati ukuran key 16 byte atau IV 8 byte.
```

### Key Atau IV Pendek Ditolak

Dampak:

- Perilaku program berubah dari menerima input pendek menjadi validasi ketat.
- Ini bisa dibuat lebih eksplisit, tetapi contoh lama seperti `abc iv` tidak lagi valid.
- Perlu menambah pengecekan `strlen`.

Jawaban pendek:

```text
Saat ini key dan IV pendek diterima lalu dipad nol. Kalau ingin wajib tepat 16 dan
8 karakter, main perlu validasi panjang sebelum copy_text_bytes.
```

### IV Dibuat Konstan Di Kode

Dampak:

- Demo lebih sederhana, tetapi secara konsep lebih buruk.
- Jika key dan IV yang sama dipakai ulang, OFB menghasilkan keystream yang sama.
- Dua plaintext yang dienkripsi dengan keystream sama bisa bocor hubungan XOR-nya.

Jawaban pendek:

```text
IV sebaiknya tetap input karena IV membedakan keystream. Dalam OFB, key dan IV yang
sama menghasilkan keystream yang sama, jadi reuse IV dengan key sama tidak aman.
```

## Skenario Perubahan Hex

### `print_hex` Diganti Lowercase

Contoh:

```c
printf("%02x", data[i]);
```

Dampak:

- Ciphertext byte tidak berubah.
- Yang berubah hanya tampilan string hex.
- Dekripsi tetap bisa karena parser menerima `a-f` dan `A-F`.
- Output dokumentasi perlu disesuaikan.

Jawaban pendek:

```text
Ini hanya perubahan format output. Nilai byte ciphertext sama, hanya huruf A-F
ditampilkan kecil. Dekripsi tetap menerima lowercase.
```

### Validasi Panjang Genap Dihapus Dari `hex_to_bytes`

Kode penting:

```c
if ((hex_len % 2) != 0) {
  return NULL;
}
```

Dampak:

- Input seperti `ABC` tidak membentuk byte utuh.
- Loop membaca pasangan `hex[i]` dan `hex[i + 1]`.
- Pada digit terakhir, `hex[i + 1]` bisa melewati karakter yang valid.
- Ini bug parsing.

Jawaban pendek:

```text
Hex harus genap karena 2 karakter hex menjadi 1 byte. Kalau pengecekan dihapus,
input ganjil tidak punya pasangan lengkap untuk byte terakhir.
```

### Dukungan Huruf Kecil Dihapus

Dampak:

- Ciphertext lowercase tidak lagi diterima.
- Enkripsi tetap menghasilkan uppercase karena `print_hex` memakai `%02X`.
- Program menjadi lebih ketat, tetapi kurang fleksibel.

Jawaban pendek:

```text
Secara teknis boleh kalau hanya menerima uppercase, karena output enc memang uppercase.
Tapi parser saat ini sengaja lebih fleksibel dengan menerima lowercase juga.
```

### `argv[1] == "enc"` Dipakai

Contoh salah:

```c
if (argv[1] == "enc") {
```

Dampak:

- Ini bug di C.
- `==` membandingkan alamat pointer, bukan isi string.
- Harus memakai `strcmp`.

Jawaban pendek:

```text
String C dibandingkan dengan strcmp. Operator == hanya membandingkan alamat memori,
bukan teks yang dikandung string.
```

## Skenario Perubahan Output Dekripsi

### `fwrite` Diganti `printf("%s")`

Kode sekarang:

```c
fwrite(plaintext, 1, len, stdout);
```

Jika diganti:

```c
printf("%s\n", plaintext);
```

Dampak:

- Tidak aman.
- Buffer plaintext hasil dekripsi tidak diberi terminator `'\0'`.
- Jika plaintext berisi byte nol, output akan berhenti lebih awal.
- Jika tidak ada terminator, `printf("%s")` bisa membaca melewati buffer.

Jawaban pendek:

```text
fwrite dipakai karena plaintext diperlakukan sebagai byte array dengan panjang len,
bukan string C yang null-terminated. printf("%s") butuh terminator nol.
```

### `malloc(len == 0 ? 1 : len)` Diganti `malloc(len)`

Dampak:

- Untuk input kosong, `malloc(0)` boleh mengembalikan `NULL` tergantung implementasi.
- Program bisa salah menganggap itu gagal alokasi.
- Versi sekarang membuat alokasi minimal 1 byte agar kasus panjang 0 tetap aman.

Jawaban pendek:

```text
malloc(0) perilakunya implementation-defined. Karena program menganggap NULL sebagai
gagal, alokasi minimal 1 byte membuat plaintext kosong tetap aman.
```

## Skenario Pertanyaan Konsep Yang Sering Muncul

### Kenapa Dekripsi Tidak Memanggil `decrypt_block`?

Jawaban:

```text
Karena mode OFB tidak mendekripsi block cipher. Block cipher selalu dienkripsi untuk
menghasilkan keystream. Plaintext didapat dari ciphertext XOR keystream.
```

### Kenapa Ciphertext Sama Panjang Dengan Plaintext?

Jawaban:

```text
Karena setiap byte input hanya di-XOR dengan satu byte keystream. Tidak ada padding
dan tidak ada metadata tambahan.
```

### Apa Yang Terjadi Jika Key Benar Tapi IV Salah?

Jawaban:

```text
Keystream yang dihasilkan berbeda, jadi hasil dekripsi menjadi teks rusak. Di OFB,
key dan IV sama-sama menentukan keystream.
```

### Apa Yang Terjadi Jika Satu Digit Hex Ciphertext Diubah?

Jawaban:

```text
Satu digit hex adalah setengah byte. Setelah diparse menjadi byte, byte ciphertext
yang berubah akan menghasilkan byte plaintext yang berubah pada posisi itu. Karena
OFB tidak memakai ciphertext sebagai feedback, error tidak menyebar ke block berikutnya.
```

### Apa Yang Terjadi Jika Satu Digit IV Diubah?

Jawaban:

```text
IV adalah feedback awal. Jika IV berubah, stream pertama berubah, lalu feedback
berikutnya juga berubah karena berasal dari stream. Jadi banyak byte hasil dekripsi
akan berubah.
```

### Apa Yang Terjadi Jika Plaintext Sama, Key Sama, IV Sama?

Jawaban:

```text
Ciphertext akan sama karena keystream yang dihasilkan juga sama. Ini alasan IV
sebaiknya tidak dipakai ulang dengan key yang sama pada mode OFB.
```

### Apa Yang Terjadi Jika Plaintext Berbeda, Key Sama, IV Sama?

Jawaban:

```text
Keystream tetap sama. Karena ciphertext = plaintext XOR keystream, penggunaan
keystream yang sama bisa membocorkan hubungan antara dua plaintext.
```

## Latihan Cepat Dosen Mengubah Kode

Gunakan bagian ini untuk latihan jawab spontan.

| Pertanyaan Dosen | Jawaban Singkat |
| --- | --- |
| Kalau `ROUNDS` dari 8 jadi 10? | Output berubah, proses lebih banyak ronde, round-trip tetap bisa dengan versi sama. |
| Kalau `BLOCK_SIZE` dari 8 jadi 16? | Tidak cukup ubah macro; `encrypt_block` masih desain 64-bit, harus redesign. |
| Kalau `KEY_SIZE` dari 16 jadi 8? | Berbahaya; `generate_round_keys` masih baca empat word atau 16 byte. |
| Kalau `SBOX` diganti? | Round key dan keystream berubah, ciphertext berubah. |
| Kalau `print_hex` pakai `%02x`? | Hanya format huruf hex berubah jadi lowercase, byte tetap sama. |
| Kalau `hex_value` tidak menerima lowercase? | Dekripsi lowercase ditolak, uppercase tetap bisa. |
| Kalau `memset` di `copy_text_bytes` dihapus? | Key/IV pendek jadi tidak deterministik karena sisa buffer tidak nol. |
| Kalau `chunk` dihapus? | Block terakhir parsial bisa out-of-bounds. |
| Kalau `fwrite` diganti `printf("%s")`? | Tidak aman karena plaintext bukan string null-terminated. |
| Kalau IV salah saat dekripsi? | Keystream salah, plaintext rusak. |
| Kalau ciphertext diubah satu byte? | Plaintext berubah di byte itu; error tidak menyebar seperti CBC. |
| Kalau `strcmp` diganti `==`? | Salah karena membandingkan alamat pointer, bukan isi string. |

## Checklist Mental Sebelum Menjawab

Tanya diri sendiri:

```text
1. Perubahan ini menyentuh format input/output atau inti cipher?
2. Apakah perubahan hanya mengubah tampilan, atau mengubah byte sebenarnya?
3. Apakah buffer masih punya ukuran yang cocok?
4. Apakah block terakhir parsial masih aman?
5. Apakah enkripsi dan dekripsi masih menghasilkan keystream yang sama?
6. Apakah ciphertext lama masih kompatibel dengan algoritma baru?
```

Jawaban paling aman saat ragu:

```text
Kalau perubahan hanya mengubah tampilan, byte ciphertext tidak berubah.
Kalau perubahan menyentuh SBOX, round key, rotasi, Feistel, endian, atau OFB feedback,
keystream berubah sehingga ciphertext berubah.
Round-trip biasanya tetap bisa selama enkripsi dan dekripsi memakai versi algoritma
yang sama, kecuali perubahan itu merusak simetri OFB atau membuat bug memori.
```
