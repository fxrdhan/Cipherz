use std::fmt::Write as _;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use rfd::FileDialog;

use crate::{BLOCK_SIZE, KEY_SIZE};

pub(super) fn validate_key_iv(key_text: &str, iv_text: &str) -> Option<String> {
    if key_text.len() != KEY_SIZE {
        return Some(format!("Key must be exactly {} characters.", KEY_SIZE));
    }

    if iv_text.len() != BLOCK_SIZE {
        return Some(format!("IV must be exactly {} characters.", BLOCK_SIZE));
    }

    None
}

pub(super) fn random_complex_string(len: usize) -> String {
    const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
    const LOWER: &[u8] = b"abcdefghijkmnopqrstuvwxyz";
    const DIGIT: &[u8] = b"23456789";
    const SYMBOL: &[u8] = b"!@#$%^&*()-_=+[]{}?";
    const ALL: &[u8] =
        b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%^&*()-_=+[]{}?";

    let mut chars = Vec::with_capacity(len);
    let required_sets = [UPPER, LOWER, DIGIT, SYMBOL];

    for charset in required_sets {
        if chars.len() < len {
            chars.push(random_charset_char(charset));
        }
    }

    while chars.len() < len {
        chars.push(random_charset_char(ALL));
    }

    for i in (1..chars.len()).rev() {
        let swap_idx = random_index(i + 1);
        chars.swap(i, swap_idx);
    }

    chars.into_iter().collect()
}

fn random_charset_char(charset: &[u8]) -> char {
    charset[random_index(charset.len())] as char
}

fn random_index(upper_bound: usize) -> usize {
    if upper_bound <= 1 {
        return 0;
    }

    (random_u64() % upper_bound as u64) as usize
}

fn random_u64() -> u64 {
    let mut bytes = [0u8; 8];
    if let Ok(mut random) = std::fs::File::open("/dev/urandom")
        && random.read_exact(&mut bytes).is_ok()
    {
        return u64::from_ne_bytes(bytes);
    }

    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    seed ^= (std::process::id() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    seed ^= seed << 13;
    seed ^= seed >> 7;
    seed ^= seed << 17;
    seed
}

pub(super) fn save_output_with_dialog(prefix: &str, content: &str) -> std::io::Result<()> {
    if content.is_empty() {
        return Ok(());
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let default_name = format!("{prefix}_{timestamp}.txt");
    let start_dir = std::env::current_dir().ok();

    let mut dialog = FileDialog::new().add_filter("Text file", &["txt"]);
    if let Some(dir) = start_dir {
        dialog = dialog.set_directory(dir);
    }

    if let Some(path) = dialog.set_file_name(&default_name).save_file() {
        std::fs::write(path, content)?;
    }

    Ok(())
}

pub(super) fn import_text_with_dialog() -> std::io::Result<Option<String>> {
    let start_dir = std::env::current_dir().ok();

    let mut dialog = FileDialog::new()
        .add_filter("Text and Markdown", &["txt", "md"])
        .add_filter("Text", &["txt"])
        .add_filter("Markdown", &["md"]);
    if let Some(dir) = start_dir {
        dialog = dialog.set_directory(dir);
    }

    let Some(path) = dialog.pick_file() else {
        return Ok(None);
    };

    Ok(Some(std::fs::read_to_string(path)?))
}

fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(10 + (c - b'a')),
        b'A'..=b'F' => Some(10 + (c - b'A')),
        _ => None,
    }
}

pub(super) fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
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

pub(super) fn hex_string(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 2);
    for byte in data {
        let _ = write!(&mut out, "{byte:02X}");
    }
    out
}
