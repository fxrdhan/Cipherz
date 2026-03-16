use std::fmt::Write as _;
use std::io::{self, Write};
use std::time::Instant;

pub mod gpui_app;

pub const BLOCK_SIZE: usize = 8;
pub const KEY_SIZE: usize = 16;
const ROUNDS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CipherMode {
    Cbc,
    Cfb,
    Ofb,
}

impl CipherMode {
    fn parse(text: &str) -> Option<Self> {
        match text {
            "cbc" => Some(Self::Cbc),
            "cfb" => Some(Self::Cfb),
            "ofb" => Some(Self::Ofb),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Cbc => "CBC",
            Self::Cfb => "CFB",
            Self::Ofb => "OFB",
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Self::Cbc => "cbc",
            Self::Cfb => "cfb",
            Self::Ofb => "ofb",
        }
    }

    fn uses_padding(self) -> bool {
        matches!(self, Self::Cbc)
    }
}

const NIBBLE_SBOX: [u8; 16] = [
    0xE, 0x4, 0xD, 0x1, 0x2, 0xF, 0xB, 0x8, 0x3, 0xA, 0x6, 0xC, 0x5, 0x9, 0x0, 0x7,
];

fn rotl32(value: u32, shift: u32) -> u32 {
    value.rotate_left(shift)
}

fn read_u32_be(src: &[u8]) -> u32 {
    u32::from_be_bytes(src[..4].try_into().expect("slice length must be 4"))
}

fn write_u32_be(dst: &mut [u8], value: u32) {
    dst[..4].copy_from_slice(&value.to_be_bytes());
}

fn substitute_word(value: u32) -> u32 {
    let mut out = 0u32;
    for i in 0..8 {
        let nibble = ((value >> (i * 4)) & 0x0F) as usize;
        out |= u32::from(NIBBLE_SBOX[nibble]) << (i * 4);
    }
    out
}

fn permute_word(value: u32) -> u32 {
    rotl32(value, 3) ^ rotl32(value, 11) ^ rotl32(value, 19)
}

fn round_function(right: u32, round_key: u32) -> u32 {
    let mixed = right ^ round_key;
    let substituted = substitute_word(mixed);
    permute_word(substituted)
}

fn generate_round_keys(key: &[u8; KEY_SIZE]) -> [u32; ROUNDS] {
    let mut round_keys = [0u32; ROUNDS];
    let mut a = read_u32_be(&key[0..4]);
    let mut b = read_u32_be(&key[4..8]);
    let mut c = read_u32_be(&key[8..12]);
    let mut d = read_u32_be(&key[12..16]);

    for (i, slot) in round_keys.iter_mut().enumerate() {
        let idx = i as u32;
        let mix = rotl32(a ^ c, ((i % 7) + 1) as u32)
            .wrapping_add(rotl32(b ^ d, (((i + 2) % 7) + 1) as u32))
            .wrapping_add(0x9E37_79B9u32.wrapping_mul(idx + 1));

        *slot = substitute_word(mix ^ rotl32(d, (((i + 4) % 9) + 1) as u32));

        let next =
            a ^ rotl32(*slot, 7) ^ 0xA5A5_A5A5u32.wrapping_add(idx.wrapping_mul(0x0101_0101));
        a = b;
        b = c;
        c = d;
        d = next;
    }

    round_keys
}

fn encrypt_block(input: &[u8; BLOCK_SIZE], round_keys: &[u32; ROUNDS]) -> [u8; BLOCK_SIZE] {
    let mut left = read_u32_be(&input[0..4]);
    let mut right = read_u32_be(&input[4..8]);

    for &round_key in round_keys {
        let next_left = right;
        let next_right = left ^ round_function(right, round_key);
        left = next_left;
        right = next_right;
    }

    let mut out = [0u8; BLOCK_SIZE];
    write_u32_be(&mut out[0..4], right);
    write_u32_be(&mut out[4..8], left);
    out
}

fn decrypt_block(input: &[u8; BLOCK_SIZE], round_keys: &[u32; ROUNDS]) -> [u8; BLOCK_SIZE] {
    let mut right = read_u32_be(&input[0..4]);
    let mut left = read_u32_be(&input[4..8]);

    for &round_key in round_keys.iter().rev() {
        let prev_right = left;
        let prev_left = right ^ round_function(left, round_key);
        right = prev_right;
        left = prev_left;
    }

    let mut out = [0u8; BLOCK_SIZE];
    write_u32_be(&mut out[0..4], left);
    write_u32_be(&mut out[4..8], right);
    out
}

fn xor_block(dst: &mut [u8; BLOCK_SIZE], src: &[u8; BLOCK_SIZE]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d ^= *s;
    }
}

pub fn derive_bytes<const N: usize>(text: &str) -> [u8; N] {
    let mut dst = [0u8; N];
    let bytes = text.as_bytes();
    let len = bytes.len().min(N);
    dst[..len].copy_from_slice(&bytes[..len]);
    dst
}

fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(10 + (c - b'a')),
        b'A'..=b'F' => Some(10 + (c - b'A')),
        _ => None,
    }
}

fn pkcs7_pad(src: &[u8]) -> Vec<u8> {
    let padding = BLOCK_SIZE - (src.len() % BLOCK_SIZE);
    let mut out = Vec::with_capacity(src.len() + padding);
    out.extend_from_slice(src);
    out.extend(std::iter::repeat_n(padding as u8, padding));
    out
}

fn pkcs7_unpad(src: &[u8]) -> Option<Vec<u8>> {
    if src.is_empty() || !src.len().is_multiple_of(BLOCK_SIZE) {
        return None;
    }

    let padding = usize::from(*src.last()?);
    if padding == 0 || padding > BLOCK_SIZE {
        return None;
    }

    if !src[src.len() - padding..]
        .iter()
        .all(|&byte| usize::from(byte) == padding)
    {
        return None;
    }

    Some(src[..src.len() - padding].to_vec())
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let bytes = hex.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return None;
    }

    let mut out = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = hex_value(chunk[0])?;
        let low = hex_value(chunk[1])?;
        out.push((high << 4) | low);
    }
    Some(out)
}

fn hex_string(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        let _ = write!(&mut out, "{:02X}", byte);
    }
    out
}

pub fn encrypt_message(
    mode: CipherMode,
    plaintext: &[u8],
    key: &[u8; KEY_SIZE],
    iv: &[u8; BLOCK_SIZE],
) -> Vec<u8> {
    let round_keys = generate_round_keys(key);
    let input = if mode.uses_padding() {
        pkcs7_pad(plaintext)
    } else {
        plaintext.to_vec()
    };

    let mut output = vec![0u8; input.len()];
    let mut feedback = *iv;

    match mode {
        CipherMode::Cbc => {
            for offset in (0..input.len()).step_by(BLOCK_SIZE) {
                let mut block = [0u8; BLOCK_SIZE];
                block.copy_from_slice(&input[offset..offset + BLOCK_SIZE]);
                xor_block(&mut block, &feedback);
                let encrypted = encrypt_block(&block, &round_keys);
                output[offset..offset + BLOCK_SIZE].copy_from_slice(&encrypted);
                feedback = encrypted;
            }
        }
        CipherMode::Cfb => {
            let mut offset = 0usize;
            while offset < input.len() {
                let chunk = (input.len() - offset).min(BLOCK_SIZE);
                let stream = encrypt_block(&feedback, &round_keys);
                for i in 0..chunk {
                    output[offset + i] = input[offset + i] ^ stream[i];
                }
                feedback[..chunk].copy_from_slice(&output[offset..offset + chunk]);
                if chunk < BLOCK_SIZE {
                    feedback[chunk..].copy_from_slice(&stream[chunk..]);
                }
                offset += chunk;
            }
        }
        CipherMode::Ofb => {
            let mut offset = 0usize;
            while offset < input.len() {
                let chunk = (input.len() - offset).min(BLOCK_SIZE);
                feedback = encrypt_block(&feedback, &round_keys);
                for i in 0..chunk {
                    output[offset + i] = input[offset + i] ^ feedback[i];
                }
                offset += chunk;
            }
        }
    }

    output
}

pub fn decrypt_message(
    mode: CipherMode,
    ciphertext: &[u8],
    key: &[u8; KEY_SIZE],
    iv: &[u8; BLOCK_SIZE],
) -> Option<Vec<u8>> {
    let round_keys = generate_round_keys(key);
    if mode.uses_padding() && !ciphertext.len().is_multiple_of(BLOCK_SIZE) {
        return None;
    }

    let mut temp = vec![0u8; ciphertext.len()];
    let mut feedback = *iv;

    match mode {
        CipherMode::Cbc => {
            for offset in (0..ciphertext.len()).step_by(BLOCK_SIZE) {
                let mut block = [0u8; BLOCK_SIZE];
                block.copy_from_slice(&ciphertext[offset..offset + BLOCK_SIZE]);
                let mut decrypted = decrypt_block(&block, &round_keys);
                xor_block(&mut decrypted, &feedback);
                temp[offset..offset + BLOCK_SIZE].copy_from_slice(&decrypted);
                feedback = block;
            }
        }
        CipherMode::Cfb => {
            let mut offset = 0usize;
            while offset < ciphertext.len() {
                let chunk = (ciphertext.len() - offset).min(BLOCK_SIZE);
                let stream = encrypt_block(&feedback, &round_keys);
                for i in 0..chunk {
                    temp[offset + i] = ciphertext[offset + i] ^ stream[i];
                }
                feedback[..chunk].copy_from_slice(&ciphertext[offset..offset + chunk]);
                if chunk < BLOCK_SIZE {
                    feedback[chunk..].copy_from_slice(&stream[chunk..]);
                }
                offset += chunk;
            }
        }
        CipherMode::Ofb => {
            let mut offset = 0usize;
            while offset < ciphertext.len() {
                let chunk = (ciphertext.len() - offset).min(BLOCK_SIZE);
                feedback = encrypt_block(&feedback, &round_keys);
                for i in 0..chunk {
                    temp[offset + i] = ciphertext[offset + i] ^ feedback[i];
                }
                offset += chunk;
            }
        }
    }

    if mode.uses_padding() {
        pkcs7_unpad(&temp)
    } else {
        Some(temp)
    }
}

fn parse_positive_int(text: &str) -> Option<usize> {
    let parsed = text.parse::<usize>().ok()?;
    if parsed == 0 || parsed > 100_000_000 {
        return None;
    }
    Some(parsed)
}

fn parse_size_value(text: &str) -> Option<usize> {
    let parsed = text.parse::<u128>().ok()?;
    if parsed == 0 || parsed > usize::MAX as u128 {
        return None;
    }
    Some(parsed as usize)
}

fn print_usage(program: &str) {
    println!("Usage:");
    println!("  {program} bench");
    println!("  {program} benchcsv <data_bytes> <iterations>");
    println!("  {program} enc <mode> <key16> <iv8> <plaintext>");
    println!("  {program} dec <mode> <key16> <iv8> <ciphertext_hex>\n");
    println!("Modes: cbc, cfb, ofb");
    println!("Catatan:");
    println!("- Key diambil dari 16 karakter pertama.");
    println!("- IV diambil dari 8 karakter pertama.");
    println!("- CBC memakai padding PKCS#7.");
}

fn run_benchmark_internal(data_len: usize, iterations: usize, csv_output: bool) -> i32 {
    let key_text = "KAMSIS-KEY-2026!";
    let iv_text = "IV2026!!";
    let key = derive_bytes::<KEY_SIZE>(key_text);
    let iv = derive_bytes::<BLOCK_SIZE>(iv_text);
    let modes = [
        CipherMode::Cbc,
        CipherMode::Cfb,
        CipherMode::Ofb,
    ];

    let mut plaintext = vec![0u8; data_len];
    for (i, byte) in plaintext.iter_mut().enumerate() {
        *byte = (((i as u128) * 37 + 11) & 0xFF) as u8;
    }

    if csv_output {
        println!(
            "mode,operation,data_bytes,iterations,total_seconds,throughput_mib_s,avg_ms_per_iteration"
        );
    } else {
        println!("Benchmark block cipher");
        println!("Data per iterasi : {data_len} bytes");
        println!("Jumlah iterasi   : {iterations}\n");
    }

    for mode in modes {
        let enc_start = Instant::now();
        let mut last_cipher = Vec::new();
        for iter in 0..iterations {
            let encrypted = encrypt_message(mode, &plaintext, &key, &iv);
            if iter + 1 == iterations {
                last_cipher = encrypted;
            }
        }
        let enc_seconds = enc_start.elapsed().as_secs_f64();

        let dec_start = Instant::now();
        let mut last_plain = Vec::new();
        for iter in 0..iterations {
            let Some(decrypted) = decrypt_message(mode, &last_cipher, &key, &iv) else {
                eprintln!("Benchmark dekripsi gagal pada mode {}", mode.name());
                return 1;
            };
            if iter + 1 == iterations {
                last_plain = decrypted;
            }
        }
        let dec_seconds = dec_start.elapsed().as_secs_f64();

        if last_plain != plaintext {
            eprintln!("Verifikasi benchmark gagal pada mode {}", mode.name());
            return 1;
        }

        let total_bytes = (data_len as f64) * (iterations as f64);
        let enc_mib_s = total_bytes / (1024.0 * 1024.0 * enc_seconds);
        let dec_mib_s = total_bytes / (1024.0 * 1024.0 * dec_seconds);

        if csv_output {
            println!(
                "{},{},{},{},{:.6},{:.6},{:.6}",
                mode.slug(),
                "encrypt",
                data_len,
                iterations,
                enc_seconds,
                enc_mib_s,
                (enc_seconds * 1000.0) / (iterations as f64)
            );
            println!(
                "{},{},{},{},{:.6},{:.6},{:.6}",
                mode.slug(),
                "decrypt",
                data_len,
                iterations,
                dec_seconds,
                dec_mib_s,
                (dec_seconds * 1000.0) / (iterations as f64)
            );
        } else {
            println!("[{}]", mode.name());
            println!(
                "  Enkripsi : {:.4} s total | {:.2} MiB/s",
                enc_seconds, enc_mib_s
            );
            println!(
                "  Dekripsi : {:.4} s total | {:.2} MiB/s\n",
                dec_seconds, dec_mib_s
            );
        }
    }

    0
}

fn run_benchmark() -> i32 {
    run_benchmark_internal(1024 * 1024, 200, false)
}

pub fn run_cli() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    let program = args.first().map(String::as_str).unwrap_or("block_cipher");

    if args.len() == 2 && args[1] == "bench" {
        return run_benchmark();
    }

    if args.len() == 4 && args[1] == "benchcsv" {
        let Some(data_len) = parse_size_value(&args[2]) else {
            eprintln!("Argumen benchcsv tidak valid.");
            print_usage(program);
            return 1;
        };
        let Some(iterations) = parse_positive_int(&args[3]) else {
            eprintln!("Argumen benchcsv tidak valid.");
            print_usage(program);
            return 1;
        };

        return run_benchmark_internal(data_len, iterations, true);
    }

    if args.len() != 6 {
        print_usage(program);
        return 1;
    }

    let Some(mode) = CipherMode::parse(&args[2]) else {
        eprintln!("Mode tidak dikenal: {}", args[2]);
        print_usage(program);
        return 1;
    };

    let key = derive_bytes::<KEY_SIZE>(&args[3]);
    if args[4] == "-" {
        eprintln!("Mode {} memerlukan IV 8 karakter.", args[2]);
        return 1;
    }
    let iv = derive_bytes::<BLOCK_SIZE>(&args[4]);

    if args[1] == "enc" {
        let encrypted = encrypt_message(mode, args[5].as_bytes(), &key, &iv);
        println!("{}", hex_string(&encrypted));
        return 0;
    }

    if args[1] == "dec" {
        let Some(ciphertext) = hex_to_bytes(&args[5]) else {
            eprintln!("Ciphertext hex tidak valid.");
            return 1;
        };

        let Some(decrypted) = decrypt_message(mode, &ciphertext, &key, &iv) else {
            eprintln!("Dekripsi gagal. Periksa mode, key, IV, atau padding.");
            return 1;
        };

        let mut stdout = io::stdout().lock();
        if stdout.write_all(&decrypted).is_err() || stdout.write_all(b"\n").is_err() {
            eprintln!("Gagal menulis output.");
            return 1;
        }
        return 0;
    }

    print_usage(program);
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_ciphertexts_match_c_version() {
        let plaintext = b"Tugas block cipher tanpa library kriptografi";
        let key = derive_bytes::<KEY_SIZE>("KAMSIS-KEY-2026!");
        let iv = derive_bytes::<BLOCK_SIZE>("IV2026!!");

        let cases = [
            (
                CipherMode::Cbc,
                "1D772D20CC1BB06322AA854304670B3317E57FC1352695EC943B5D2FA39DD119C1967B69EABA418E3E79FAC424ED9463",
            ),
            (
                CipherMode::Cfb,
                "E4B82563C6FDEF1DB2CD63E28669AF8A29EB2332D001DDA024CC06FD1E689EF51A490EB2D603384E09A0745B",
            ),
            (
                CipherMode::Ofb,
                "E4B82563C6FDEF1DCCFF1ECB9DB6730836A9C6B7DEDCA8DA37A6792A8C8650A661A74E8BE5D75A8966092D54",
            ),
        ];

        for (mode, expected_hex) in cases {
            let encrypted = encrypt_message(mode, plaintext, &key, &iv);
            let decrypted = decrypt_message(mode, &encrypted, &key, &iv).expect("decrypt failed");
            assert_eq!(hex_string(&encrypted), expected_hex);
            assert_eq!(decrypted, plaintext);
        }
    }

    #[test]
    fn rejects_invalid_padding() {
        let key = derive_bytes::<KEY_SIZE>("KAMSIS-KEY-2026!");
        let iv = derive_bytes::<BLOCK_SIZE>("IV2026!!");
        assert!(decrypt_message(CipherMode::Cbc, &[0u8; BLOCK_SIZE], &key, &iv).is_none());
    }
}
