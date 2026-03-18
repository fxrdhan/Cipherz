#include <ctype.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define BLOCK_SIZE 8
#define KEY_SIZE 16
#define ROUNDS 8

typedef enum {
    MODE_CBC,
    MODE_CFB,
    MODE_OFB,
    MODE_INVALID
} CipherMode;

typedef struct {
    uint8_t *data;
    size_t len;
} Buffer;

static const uint8_t NIBBLE_SBOX[16] = {
    0xE, 0x4, 0xD, 0x1,
    0x2, 0xF, 0xB, 0x8,
    0x3, 0xA, 0x6, 0xC,
    0x5, 0x9, 0x0, 0x7
};

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

static uint32_t substitute_word(uint32_t value) {
    uint32_t out = 0;
    for (int i = 0; i < 8; ++i) {
        uint32_t nibble = (value >> (i * 4)) & 0xFU;
        out |= ((uint32_t)NIBBLE_SBOX[nibble]) << (i * 4);
    }
    return out;
}

static uint32_t permute_word(uint32_t value) {
    return rotl32(value, 3) ^ rotl32(value, 11) ^ rotl32(value, 19);
}

static uint32_t round_function(uint32_t right, uint32_t round_key) {
    uint32_t mixed = right ^ round_key;
    uint32_t substituted = substitute_word(mixed);
    return permute_word(substituted);
}

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

        {
            uint32_t next = a ^ rotl32(round_keys[i], 7) ^ (0xA5A5A5A5u + (uint32_t)i * 0x01010101u);
            a = b;
            b = c;
            c = d;
            d = next;
        }
    }
}

static void encrypt_block(const uint8_t in[BLOCK_SIZE], uint8_t out[BLOCK_SIZE], const uint32_t round_keys[ROUNDS]) {
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

static void decrypt_block(const uint8_t in[BLOCK_SIZE], uint8_t out[BLOCK_SIZE], const uint32_t round_keys[ROUNDS]) {
    uint32_t right = read_u32_be(in);
    uint32_t left = read_u32_be(in + 4);

    for (int i = ROUNDS - 1; i >= 0; --i) {
        uint32_t prev_right = left;
        uint32_t prev_left = right ^ round_function(left, round_keys[i]);
        right = prev_right;
        left = prev_left;
    }

    write_u32_be(out, left);
    write_u32_be(out + 4, right);
}

static void xor_block(uint8_t *dst, const uint8_t *src) {
    for (size_t i = 0; i < BLOCK_SIZE; ++i) {
        dst[i] ^= src[i];
    }
}

static void derive_bytes(const char *text, uint8_t *dst, size_t size) {
    size_t len = strlen(text);
    memset(dst, 0, size);
    if (len > size) {
        len = size;
    }
    memcpy(dst, text, len);
}

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

static Buffer clone_buffer(const uint8_t *src, size_t len) {
    Buffer out = {NULL, 0};
    out.data = (uint8_t *)malloc(len == 0 ? 1 : len);
    if (out.data == NULL) {
        return out;
    }
    if (len > 0) {
        memcpy(out.data, src, len);
    }
    out.len = len;
    return out;
}

static Buffer pkcs7_pad(const uint8_t *src, size_t len) {
    Buffer out = {NULL, 0};
    size_t padding = BLOCK_SIZE - (len % BLOCK_SIZE);
    if (padding == 0) {
        padding = BLOCK_SIZE;
    }

    out.data = (uint8_t *)malloc(len + padding);
    if (out.data == NULL) {
        return out;
    }

    memcpy(out.data, src, len);
    for (size_t i = 0; i < padding; ++i) {
        out.data[len + i] = (uint8_t)padding;
    }
    out.len = len + padding;
    return out;
}

static Buffer pkcs7_unpad(const uint8_t *src, size_t len) {
    Buffer out = {NULL, 0};
    if (len == 0 || (len % BLOCK_SIZE) != 0) {
        return out;
    }

    uint8_t padding = src[len - 1];
    if (padding == 0 || padding > BLOCK_SIZE) {
        return out;
    }

    for (size_t i = 0; i < padding; ++i) {
        if (src[len - 1 - i] != padding) {
            return out;
        }
    }

    return clone_buffer(src, len - padding);
}

static Buffer hex_to_bytes(const char *hex) {
    Buffer out = {NULL, 0};
    size_t len = strlen(hex);

    if ((len % 2) != 0) {
        return out;
    }

    out.data = (uint8_t *)malloc(len / 2 == 0 ? 1 : len / 2);
    if (out.data == NULL) {
        return out;
    }

    for (size_t i = 0; i < len; i += 2) {
        int high = hex_value(hex[i]);
        int low = hex_value(hex[i + 1]);
        if (high < 0 || low < 0) {
            free(out.data);
            out.data = NULL;
            return out;
        }
        out.data[i / 2] = (uint8_t)((high << 4) | low);
    }

    out.len = len / 2;
    return out;
}

static void print_hex(const uint8_t *data, size_t len) {
    for (size_t i = 0; i < len; ++i) {
        printf("%02X", data[i]);
    }
    printf("\n");
}

static CipherMode parse_mode(const char *text) {
    if (strcmp(text, "cbc") == 0) {
        return MODE_CBC;
    }
    if (strcmp(text, "cfb") == 0) {
        return MODE_CFB;
    }
    if (strcmp(text, "ofb") == 0) {
        return MODE_OFB;
    }
    return MODE_INVALID;
}

static const char *mode_name(CipherMode mode) {
    switch (mode) {
        case MODE_CBC:
            return "CBC";
        case MODE_CFB:
            return "CFB";
        case MODE_OFB:
            return "OFB";
        default:
            return "INVALID";
    }
}

static const char *mode_slug(CipherMode mode) {
    switch (mode) {
        case MODE_CBC:
            return "cbc";
        case MODE_CFB:
            return "cfb";
        case MODE_OFB:
            return "ofb";
        default:
            return "invalid";
    }
}

static int mode_uses_padding(CipherMode mode) {
    return mode == MODE_CBC;
}

static Buffer encrypt_message(CipherMode mode, const uint8_t *plaintext, size_t len,
                              const uint8_t key[KEY_SIZE], const uint8_t iv[BLOCK_SIZE]) {
    Buffer input = {NULL, 0};
    Buffer output = {NULL, 0};
    uint32_t round_keys[ROUNDS];
    uint8_t feedback[BLOCK_SIZE];
    uint8_t stream[BLOCK_SIZE];

    generate_round_keys(key, round_keys);

    if (mode_uses_padding(mode)) {
        input = pkcs7_pad(plaintext, len);
    } else {
        input = clone_buffer(plaintext, len);
    }

    if (input.data == NULL) {
        return output;
    }

    output.data = (uint8_t *)malloc(input.len == 0 ? 1 : input.len);
    if (output.data == NULL) {
        free(input.data);
        return output;
    }
    output.len = input.len;

    memcpy(feedback, iv, BLOCK_SIZE);

    if (mode == MODE_CBC) {
        for (size_t offset = 0; offset < input.len; offset += BLOCK_SIZE) {
            uint8_t block[BLOCK_SIZE];
            memcpy(block, input.data + offset, BLOCK_SIZE);
            xor_block(block, feedback);
            encrypt_block(block, output.data + offset, round_keys);
            memcpy(feedback, output.data + offset, BLOCK_SIZE);
        }
    } else if (mode == MODE_CFB) {
        for (size_t offset = 0; offset < input.len; offset += BLOCK_SIZE) {
            size_t chunk = input.len - offset;
            if (chunk > BLOCK_SIZE) {
                chunk = BLOCK_SIZE;
            }
            encrypt_block(feedback, stream, round_keys);
            for (size_t i = 0; i < chunk; ++i) {
                output.data[offset + i] = input.data[offset + i] ^ stream[i];
            }
            memcpy(feedback, output.data + offset, chunk);
            if (chunk < BLOCK_SIZE) {
                memcpy(feedback + chunk, stream + chunk, BLOCK_SIZE - chunk);
            }
        }
    } else if (mode == MODE_OFB) {
        for (size_t offset = 0; offset < input.len; offset += BLOCK_SIZE) {
            size_t chunk = input.len - offset;
            if (chunk > BLOCK_SIZE) {
                chunk = BLOCK_SIZE;
            }
            encrypt_block(feedback, feedback, round_keys);
            for (size_t i = 0; i < chunk; ++i) {
                output.data[offset + i] = input.data[offset + i] ^ feedback[i];
            }
        }
    }

    free(input.data);
    return output;
}

static Buffer decrypt_message(CipherMode mode, const uint8_t *ciphertext, size_t len,
                              const uint8_t key[KEY_SIZE], const uint8_t iv[BLOCK_SIZE]) {
    Buffer temp = {NULL, 0};
    Buffer output = {NULL, 0};
    uint32_t round_keys[ROUNDS];
    uint8_t feedback[BLOCK_SIZE];
    uint8_t stream[BLOCK_SIZE];

    generate_round_keys(key, round_keys);

    if (mode_uses_padding(mode) && ((len % BLOCK_SIZE) != 0)) {
        return output;
    }

    temp.data = (uint8_t *)malloc(len == 0 ? 1 : len);
    if (temp.data == NULL) {
        return output;
    }
    temp.len = len;
    memcpy(feedback, iv, BLOCK_SIZE);

    if (mode == MODE_CBC) {
        for (size_t offset = 0; offset < len; offset += BLOCK_SIZE) {
            uint8_t block[BLOCK_SIZE];
            decrypt_block(ciphertext + offset, block, round_keys);
            xor_block(block, feedback);
            memcpy(temp.data + offset, block, BLOCK_SIZE);
            memcpy(feedback, ciphertext + offset, BLOCK_SIZE);
        }
    } else if (mode == MODE_CFB) {
        for (size_t offset = 0; offset < len; offset += BLOCK_SIZE) {
            size_t chunk = len - offset;
            if (chunk > BLOCK_SIZE) {
                chunk = BLOCK_SIZE;
            }
            encrypt_block(feedback, stream, round_keys);
            for (size_t i = 0; i < chunk; ++i) {
                temp.data[offset + i] = ciphertext[offset + i] ^ stream[i];
            }
            memcpy(feedback, ciphertext + offset, chunk);
            if (chunk < BLOCK_SIZE) {
                memcpy(feedback + chunk, stream + chunk, BLOCK_SIZE - chunk);
            }
        }
    } else if (mode == MODE_OFB) {
        for (size_t offset = 0; offset < len; offset += BLOCK_SIZE) {
            size_t chunk = len - offset;
            if (chunk > BLOCK_SIZE) {
                chunk = BLOCK_SIZE;
            }
            encrypt_block(feedback, feedback, round_keys);
            for (size_t i = 0; i < chunk; ++i) {
                temp.data[offset + i] = ciphertext[offset + i] ^ feedback[i];
            }
        }
    }

    if (mode_uses_padding(mode)) {
        output = pkcs7_unpad(temp.data, temp.len);
        free(temp.data);
        return output;
    }

    return temp;
}

static void free_buffer(Buffer *buffer) {
    if (buffer->data != NULL) {
        free(buffer->data);
        buffer->data = NULL;
    }
    buffer->len = 0;
}

static double now_seconds(void) {
    struct timespec ts;
    timespec_get(&ts, TIME_UTC);
    return (double)ts.tv_sec + ((double)ts.tv_nsec / 1000000000.0);
}

static int parse_positive_int(const char *text, int *value) {
    char *end = NULL;
    long parsed = strtol(text, &end, 10);
    if (end == text || *end != '\0' || parsed <= 0 || parsed > 100000000L) {
        return 0;
    }
    *value = (int)parsed;
    return 1;
}

static int parse_size_value(const char *text, size_t *value) {
    char *end = NULL;
    unsigned long long parsed = strtoull(text, &end, 10);
    if (end == text || *end != '\0' || parsed == 0ULL) {
        return 0;
    }
    *value = (size_t)parsed;
    return 1;
}

static void print_usage(const char *program) {
    printf("Usage:\n");
    printf("  %s bench\n", program);
    printf("  %s benchcsv <data_bytes> <iterations>\n", program);
    printf("  %s enc <mode> <key16> <iv8> <plaintext>\n", program);
    printf("  %s dec <mode> <key16> <iv8> <ciphertext_hex>\n\n", program);
    printf("Modes: cbc, cfb, ofb\n");
    printf("Catatan:\n");
    printf("- Key diambil dari 16 karakter pertama.\n");
    printf("- IV diambil dari 8 karakter pertama.\n");
    printf("- CBC memakai padding PKCS#7.\n");
}

static int run_benchmark_internal(size_t data_len, int iterations, int csv_output) {
    const char *key_text = "KAMSIS-KEY-2026!";
    const char *iv_text = "IV2026!!";
    uint8_t key[KEY_SIZE];
    uint8_t iv[BLOCK_SIZE];
    uint8_t *plaintext = NULL;
    CipherMode modes[] = {MODE_CBC, MODE_CFB, MODE_OFB};

    derive_bytes(key_text, key, KEY_SIZE);
    derive_bytes(iv_text, iv, BLOCK_SIZE);

    plaintext = (uint8_t *)malloc(data_len);
    if (plaintext == NULL) {
        fprintf(stderr, "Gagal mengalokasikan buffer benchmark.\n");
        return 1;
    }

    for (size_t i = 0; i < data_len; ++i) {
        plaintext[i] = (uint8_t)((i * 37U + 11U) & 0xFFU);
    }

    if (csv_output) {
        printf("mode,operation,data_bytes,iterations,total_seconds,throughput_mib_s,avg_ms_per_iteration\n");
    } else {
        printf("Benchmark block cipher\n");
        printf("Data per iterasi : %zu bytes\n", data_len);
        printf("Jumlah iterasi   : %d\n\n", iterations);
    }

    for (size_t i = 0; i < sizeof(modes) / sizeof(modes[0]); ++i) {
        double enc_start;
        double enc_end;
        double dec_start;
        double dec_end;
        double enc_mb_s;
        double dec_mb_s;
        Buffer last_cipher = {NULL, 0};
        Buffer last_plain = {NULL, 0};

        enc_start = now_seconds();
        for (int iter = 0; iter < iterations; ++iter) {
            Buffer encrypted = encrypt_message(modes[i], plaintext, data_len, key, iv);
            if (encrypted.data == NULL) {
                fprintf(stderr, "Benchmark enkripsi gagal pada mode %s\n", mode_name(modes[i]));
                free(plaintext);
                return 1;
            }
            if (iter == iterations - 1) {
                last_cipher = encrypted;
            } else {
                free_buffer(&encrypted);
            }
        }
        enc_end = now_seconds();

        dec_start = now_seconds();
        for (int iter = 0; iter < iterations; ++iter) {
            Buffer decrypted = decrypt_message(modes[i], last_cipher.data, last_cipher.len, key, iv);
            if (decrypted.data == NULL) {
                fprintf(stderr, "Benchmark dekripsi gagal pada mode %s\n", mode_name(modes[i]));
                free_buffer(&last_cipher);
                free(plaintext);
                return 1;
            }
            if (iter == iterations - 1) {
                last_plain = decrypted;
            } else {
                free_buffer(&decrypted);
            }
        }
        dec_end = now_seconds();

        if (last_plain.len != data_len || memcmp(last_plain.data, plaintext, data_len) != 0) {
            fprintf(stderr, "Verifikasi benchmark gagal pada mode %s\n", mode_name(modes[i]));
            free_buffer(&last_cipher);
            free_buffer(&last_plain);
            free(plaintext);
            return 1;
        }

        enc_mb_s = ((double)data_len * (double)iterations) / (1024.0 * 1024.0 * (enc_end - enc_start));
        dec_mb_s = ((double)data_len * (double)iterations) / (1024.0 * 1024.0 * (dec_end - dec_start));

        if (csv_output) {
            printf("%s,encrypt,%zu,%d,%.6f,%.6f,%.6f\n",
                   mode_slug(modes[i]),
                   data_len,
                   iterations,
                   enc_end - enc_start,
                   enc_mb_s,
                   ((enc_end - enc_start) * 1000.0) / (double)iterations);
            printf("%s,decrypt,%zu,%d,%.6f,%.6f,%.6f\n",
                   mode_slug(modes[i]),
                   data_len,
                   iterations,
                   dec_end - dec_start,
                   dec_mb_s,
                   ((dec_end - dec_start) * 1000.0) / (double)iterations);
        } else {
            printf("[%s]\n", mode_name(modes[i]));
            printf("  Enkripsi : %.4f s total | %.2f MiB/s\n", enc_end - enc_start, enc_mb_s);
            printf("  Dekripsi : %.4f s total | %.2f MiB/s\n\n", dec_end - dec_start, dec_mb_s);
        }

        free_buffer(&last_cipher);
        free_buffer(&last_plain);
    }

    free(plaintext);
    return 0;
}

static int run_benchmark(void) {
    return run_benchmark_internal(1024 * 1024, 200, 0);
}

int main(int argc, char *argv[]) {
    uint8_t key[KEY_SIZE];
    uint8_t iv[BLOCK_SIZE];

    if (argc == 2 && strcmp(argv[1], "bench") == 0) {
        return run_benchmark();
    }

    if (argc == 4 && strcmp(argv[1], "benchcsv") == 0) {
        size_t data_len = 0;
        int iterations = 0;

        if (!parse_size_value(argv[2], &data_len) || !parse_positive_int(argv[3], &iterations)) {
            fprintf(stderr, "Argumen benchcsv tidak valid.\n");
            print_usage(argv[0]);
            return 1;
        }

        return run_benchmark_internal(data_len, iterations, 1);
    }

    if (argc != 6) {
        print_usage(argv[0]);
        return 1;
    }

    CipherMode mode = parse_mode(argv[2]);
    if (mode == MODE_INVALID) {
        fprintf(stderr, "Mode tidak dikenal: %s\n", argv[2]);
        print_usage(argv[0]);
        return 1;
    }

    derive_bytes(argv[3], key, KEY_SIZE);
    if (strcmp(argv[4], "-") == 0) {
        fprintf(stderr, "Mode %s memerlukan IV 8 karakter.\n", argv[2]);
        return 1;
    }
    derive_bytes(argv[4], iv, BLOCK_SIZE);

    if (strcmp(argv[1], "enc") == 0) {
        Buffer encrypted = encrypt_message(mode, (const uint8_t *)argv[5], strlen(argv[5]), key, iv);
        if (encrypted.data == NULL) {
            fprintf(stderr, "Enkripsi gagal.\n");
            return 1;
        }
        print_hex(encrypted.data, encrypted.len);
        free_buffer(&encrypted);
        return 0;
    }

    if (strcmp(argv[1], "dec") == 0) {
        Buffer ciphertext = hex_to_bytes(argv[5]);
        if (ciphertext.data == NULL) {
            fprintf(stderr, "Ciphertext hex tidak valid.\n");
            return 1;
        }

        Buffer decrypted = decrypt_message(mode, ciphertext.data, ciphertext.len, key, iv);
        free_buffer(&ciphertext);

        if (decrypted.data == NULL) {
            fprintf(stderr, "Dekripsi gagal. Periksa mode, key, IV, atau padding.\n");
            return 1;
        }

        printf("%.*s\n", (int)decrypted.len, decrypted.data);
        free_buffer(&decrypted);
        return 0;
    }

    print_usage(argv[0]);
    return 1;
}
