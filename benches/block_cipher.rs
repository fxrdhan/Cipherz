use std::hint::black_box;
use std::time::Instant;

use tugas_block_cipher_rust::{
    decrypt_message, derive_bytes, encrypt_message, CipherMode, BLOCK_SIZE, KEY_SIZE,
};

const SIZES: [usize; 3] = [1024, 64 * 1024, 1024 * 1024];
const MODES: [CipherMode; 3] = [CipherMode::Cbc, CipherMode::Cfb, CipherMode::Ofb];
const TARGET_TOTAL_BYTES: usize = 32 * 1024 * 1024;

fn make_plaintext(data_len: usize) -> Vec<u8> {
    let mut plaintext = vec![0u8; data_len];
    for (i, byte) in plaintext.iter_mut().enumerate() {
        *byte = (((i as u128) * 37 + 11) & 0xFF) as u8;
    }
    plaintext
}

fn iterations_for_size(data_len: usize) -> usize {
    (TARGET_TOTAL_BYTES / data_len).max(8)
}

fn mib_per_second(data_len: usize, iterations: usize, seconds: f64) -> f64 {
    ((data_len as f64) * (iterations as f64)) / (1024.0 * 1024.0 * seconds)
}

fn run_case(mode: CipherMode, data_len: usize, iterations: usize) -> Result<(), String> {
    let key = derive_bytes::<KEY_SIZE>("KAMSIS-KEY-2026!");
    let iv = derive_bytes::<BLOCK_SIZE>("IV2026!!");
    let plaintext = make_plaintext(data_len);

    let enc_start = Instant::now();
    let mut last_cipher = Vec::new();
    for iter in 0..iterations {
        let encrypted = encrypt_message(mode, black_box(&plaintext), &key, &iv);
        if iter + 1 == iterations {
            last_cipher = encrypted;
        }
    }
    let enc_seconds = enc_start.elapsed().as_secs_f64();

    let dec_start = Instant::now();
    let mut last_plain = Vec::new();
    for iter in 0..iterations {
        let decrypted = decrypt_message(mode, black_box(&last_cipher), &key, &iv)
            .ok_or_else(|| format!("dekripsi gagal pada mode {}", mode.name()))?;
        if iter + 1 == iterations {
            last_plain = decrypted;
        }
    }
    let dec_seconds = dec_start.elapsed().as_secs_f64();

    if last_plain != plaintext {
        return Err(format!("verifikasi gagal pada mode {}", mode.name()));
    }

    println!(
        "{:<4} {:>10} bytes | iter {:>6} | enc {:>8.2} MiB/s | dec {:>8.2} MiB/s | enc {:>8.4} s | dec {:>8.4} s",
        mode.name(),
        data_len,
        iterations,
        mib_per_second(data_len, iterations, enc_seconds),
        mib_per_second(data_len, iterations, dec_seconds),
        enc_seconds,
        dec_seconds,
    );

    Ok(())
}

fn main() {
    println!("Rust benchmark block cipher (cargo bench)");
    println!(
        "Target total data per case: {} MiB\n",
        TARGET_TOTAL_BYTES / (1024 * 1024)
    );

    for data_len in SIZES {
        let iterations = iterations_for_size(data_len);
        println!("Data size: {} bytes", data_len);
        for mode in MODES {
            if let Err(err) = run_case(mode, data_len, iterations) {
                eprintln!("Benchmark gagal: {err}");
                std::process::exit(1);
            }
        }
        println!();
    }
}
