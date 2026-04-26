# Simple OFB Cipher CLI

Versi branch ini sengaja dibuat kecil: satu program command line dalam bahasa C dan satu mode operasi saja, yaitu `OFB`.

OFB dipilih karena paling sederhana dari mode `CBC`, `CFB`, dan `OFB` yang ada di proyek utama. Enkripsi dan dekripsi memakai proses yang sama: blok cipher menghasilkan keystream, lalu data di-XOR dengan keystream tersebut. Karena itu branch ini tidak memakai padding dan tidak punya mode tambahan.

## Build

```bash
make
```

## Pakai

```bash
./block_cipher enc <key16> <iv8> <plaintext>
./block_cipher dec <key16> <iv8> <ciphertext_hex>
```

Contoh:

```bash
./block_cipher enc KAMSIS-KEY-2026 IV2026!! "halo"
./block_cipher dec KAMSIS-KEY-2026 IV2026!! <ciphertext_hex>
```

Key diambil maksimal 16 byte pertama, IV diambil maksimal 8 byte pertama. Jika lebih pendek, sisanya diisi nol.
