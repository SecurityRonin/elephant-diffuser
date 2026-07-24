//! # elephant-diffuser — the BitLocker Elephant Diffuser
//!
//! The BitLocker Elephant Diffuser as a standalone, dependency-free primitive:
//! Diffuser A, Diffuser B, and the per-sector-key XOR that together form the
//! diffuser stage of BitLocker's CBC-plus-diffuser sector cipher (encryption
//! methods `0x8000` and `0x8001`).
//!
//! The diffuser is **not** a cipher and holds no secret — it is a keyed,
//! invertible byte-mixing transform applied to a sector *after* AES-CBC
//! decryption (and *before* AES-CBC encryption), spreading each bit across the
//! whole sector. Every other primitive in BitLocker's cipher (AES, CBC, CCM,
//! SHA-256) has an audited RustCrypto crate; the Elephant Diffuser does not, so
//! it is the one documented exception to "never hand-roll crypto." The rotation
//! constants and cycle order follow the `dislocker` (`diffuser.c`) / `libbde`
//! reference; correctness is proven in situ by `bitlocker-core`'s Tier-1
//! `bdetogo.raw`-vs-`pybde` oracle (see `docs/validation.md`).
//!
//! ```
//! let mut sector = vec![0u8; 512];
//! let sector_key = [0u8; 32]; // caller-derived (BitLocker: AES-ECB over the offset with the TWEAK key)
//! elephant_diffuser::decrypt(&mut sector, &sector_key);
//! elephant_diffuser::encrypt(&mut sector, &sector_key); // exact inverse
//! assert_eq!(sector, vec![0u8; 512]);
//! ```

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Diffuser A rotation amounts (`Ra`), indexed by word position `i % 4`.
const RA: [u32; 4] = [9, 0, 13, 0];
/// Diffuser B rotation amounts (`Rb`), indexed by word position `i % 4`.
const RB: [u32; 4] = [0, 10, 0, 25];

/// Split a sector into little-endian 32-bit words. Trailing bytes that do not
/// fill a word are dropped from the diffused words (the per-sector-key XOR in
/// [`decrypt`]/[`encrypt`] still covers them), so the transform touches
/// `sector.len() / 4` words. Real BitLocker sectors are word-aligned, so a
/// sub-word remainder only arises for out-of-spec inputs the reference never
/// specifies.
fn to_words(sector: &[u8]) -> Vec<u32> {
    sector
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Write words back over the sector they came from.
fn from_words(words: &[u32], out: &mut [u8]) {
    for (i, w) in words.iter().enumerate() {
        // `words` was produced by `to_words(out)`, so `i*4 + 4 <= out.len()` for
        // every `i`; the `get_mut` guard keeps this panic-free regardless.
        if let Some(slot) = out.get_mut(i * 4..i * 4 + 4) {
            slot.copy_from_slice(&w.to_le_bytes());
        }
    }
}

/// `(i - k) mod n`, computed without unsigned underflow for any `k` and `n >= 1`.
/// For the real BitLocker case (`n >= 5`, i.e. sectors of 20+ bytes) this equals
/// the reference `(i + n - k) % n`; it additionally stays panic-free for the
/// 1..=4-word buffers a fuzzer can present.
#[inline]
fn back(i: usize, k: usize, n: usize) -> usize {
    (i + n - (k % n)) % n
}

/// Diffuser A, decryption direction: `d[i] += d[i-2] ^ ROL(d[i-5], Ra[i%4])`,
/// five cycles, indices ascending, modulo the word count.
fn diffuser_a_decrypt(sector: &mut [u8]) {
    let mut d = to_words(sector);
    let n = d.len();
    if n == 0 {
        return;
    }
    for _ in 0..5 {
        for i in 0..n {
            let a = d[back(i, 2, n)];
            let b = d[back(i, 5, n)].rotate_left(RA[i % 4]);
            d[i] = d[i].wrapping_add(a ^ b);
        }
    }
    from_words(&d, sector);
}

/// Diffuser B, decryption direction: `d[i] += d[i+2] ^ ROL(d[i+5], Rb[i%4])`,
/// three cycles, indices ascending, modulo the word count.
fn diffuser_b_decrypt(sector: &mut [u8]) {
    let mut d = to_words(sector);
    let n = d.len();
    if n == 0 {
        return;
    }
    for _ in 0..3 {
        for i in 0..n {
            let a = d[(i + 2) % n];
            let b = d[(i + 5) % n].rotate_left(RB[i % 4]);
            d[i] = d[i].wrapping_add(a ^ b);
        }
    }
    from_words(&d, sector);
}

/// Diffuser A, encryption direction — the inverse of [`diffuser_a_decrypt`]
/// (indices descending, `wrapping_sub`).
fn diffuser_a_encrypt(sector: &mut [u8]) {
    let mut d = to_words(sector);
    let n = d.len();
    if n == 0 {
        return;
    }
    for _ in 0..5 {
        for i in (0..n).rev() {
            let a = d[back(i, 2, n)];
            let b = d[back(i, 5, n)].rotate_left(RA[i % 4]);
            d[i] = d[i].wrapping_sub(a ^ b);
        }
    }
    from_words(&d, sector);
}

/// Diffuser B, encryption direction — the inverse of [`diffuser_b_decrypt`].
fn diffuser_b_encrypt(sector: &mut [u8]) {
    let mut d = to_words(sector);
    let n = d.len();
    if n == 0 {
        return;
    }
    for _ in 0..3 {
        for i in (0..n).rev() {
            let a = d[(i + 2) % n];
            let b = d[(i + 5) % n].rotate_left(RB[i % 4]);
            d[i] = d[i].wrapping_sub(a ^ b);
        }
    }
    from_words(&d, sector);
}

/// Decrypt one sector in place (the method-`0x8000` diffuser order): Diffuser B,
/// then Diffuser A, then XOR the 32-byte sector key. The exact inverse of
/// [`encrypt`]. Operates on a sector of any length and never panics.
pub fn decrypt(sector: &mut [u8], sector_key: &[u8; 32]) {
    diffuser_b_decrypt(sector);
    diffuser_a_decrypt(sector);
    for (i, b) in sector.iter_mut().enumerate() {
        *b ^= sector_key[i % 32];
    }
}

/// Encrypt one sector in place — the exact inverse of [`decrypt`]: XOR the
/// 32-byte sector key, then Diffuser A, then Diffuser B. Operates on a sector of
/// any length and never panics.
pub fn encrypt(sector: &mut [u8], sector_key: &[u8; 32]) {
    for (i, b) in sector.iter_mut().enumerate() {
        *b ^= sector_key[i % 32];
    }
    diffuser_a_encrypt(sector);
    diffuser_b_encrypt(sector);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        use std::fmt::Write;
        bytes.iter().fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
    }

    /// The same 512-byte pattern the source impl was captured against.
    fn sample_sector() -> Vec<u8> {
        (0..512u32)
            .map(|i| (i.wrapping_mul(31) ^ 0xA5) as u8)
            .collect()
    }

    /// Sector key = bytes 0x00..=0x1f.
    fn sample_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        for (i, b) in k.iter_mut().enumerate() {
            *b = i as u8;
        }
        k
    }

    // Tier-3 regression vector captured from the Tier-1-validated bitlocker-core
    // impl BEFORE extraction (rustc -O over the exact diffuser code). The
    // authoritative proof is bitlocker-core's in-situ 0x8000 oracle
    // (bdetogo.raw vs pybde) — see docs/validation.md.
    #[test]
    fn decrypt_matches_captured_regression_vector() {
        let mut buf = sample_sector();
        decrypt(&mut buf, &sample_key());
        assert_eq!(
            hex(&buf[..32]),
            "9649e3f15c8ecdb6fceb5a864f24e97596052689bf414d5c3137edb27dc43c6e"
        );
        assert_eq!(hex(&buf[496..512]), "21eafdd00ad4826068a2d7a8f28fcf97");
    }

    #[test]
    fn encrypt_decrypt_roundtrip_is_identity() {
        let key = sample_key();
        let orig = sample_sector();
        let mut buf = orig.clone();
        encrypt(&mut buf, &key);
        assert_ne!(buf, orig);
        decrypt(&mut buf, &key);
        assert_eq!(buf, orig);
    }

    #[test]
    fn empty_and_subword_inputs_do_not_panic() {
        let key = sample_key();
        let mut empty: [u8; 0] = [];
        decrypt(&mut empty, &key);
        encrypt(&mut empty, &key);
        let mut three = [1u8, 2, 3]; // < 1 word after chunks_exact -> no-op
        decrypt(&mut three, &key);
        encrypt(&mut three, &key);
    }

    // Real BitLocker sectors are >=512 bytes (128 words), so the diffuser is only
    // ever driven at n>=5. A fuzzer over arbitrary bytes, however, hits n in
    // 1..=4 where a naive `(i + n - 5)` index underflows. This locks in the
    // panic-free modular form for those word counts.
    #[test]
    fn tiny_sector_word_counts_do_not_panic() {
        let key = sample_key();
        for words in 1..=6usize {
            let mut buf = vec![0xABu8; words * 4];
            decrypt(&mut buf, &key);
            let mut buf2 = vec![0xABu8; words * 4];
            encrypt(&mut buf2, &key);
        }
    }
}
