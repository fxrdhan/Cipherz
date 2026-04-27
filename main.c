#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define BLOCK_SIZE 8
#define KEY_SIZE 16
#define ROUNDS 8

static const uint8_t SBOX[16] = {0xE, 0x4, 0xD, 0x1, 0x2, 0xF, 0xB, 0x8,
                                 0x3, 0xA, 0x6, 0xC, 0x5, 0x9, 0x0, 0x7};

static uint32_t rotl32(uint32_t value, unsigned shift) {
  return (value << shift) | (value >> (32U - shift));
}

static uint32_t read_u32_be(const uint8_t *src) {
  return ((uint32_t)src[0] << 24) | ((uint32_t)src[1] << 16) |
         ((uint32_t)src[2] << 8) | (uint32_t)src[3];
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

static void generate_round_keys(const uint8_t key[KEY_SIZE],
                                uint32_t round_keys[ROUNDS]) {
  uint32_t a = read_u32_be(key);
  uint32_t b = read_u32_be(key + 4);
  uint32_t c = read_u32_be(key + 8);
  uint32_t d = read_u32_be(key + 12);

  for (int i = 0; i < ROUNDS; ++i) {
    uint32_t mix = rotl32(a ^ c, (unsigned)((i % 7) + 1)) +
                   rotl32(b ^ d, (unsigned)(((i + 2) % 7) + 1)) +
                   (0x9E3779B9u * (uint32_t)(i + 1));

    round_keys[i] =
        substitute_word(mix ^ rotl32(d, (unsigned)(((i + 4) % 9) + 1)));

    uint32_t next = a ^ rotl32(round_keys[i], 7) ^
                    (0xA5A5A5A5u + (uint32_t)i * 0x01010101u);
    a = b;
    b = c;
    c = d;
    d = next;
  }
}

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

static void copy_text_bytes(const char *text, uint8_t *dst, size_t dst_len) {
  memset(dst, 0, dst_len);

  for (size_t i = 0; i < dst_len && text[i] != '\0'; ++i) {
    dst[i] = (uint8_t)text[i];
  }
}

static void ofb_crypt(const uint8_t *input, uint8_t *output, size_t len,
                      const uint8_t key[KEY_SIZE],
                      const uint8_t iv[BLOCK_SIZE]) {
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
