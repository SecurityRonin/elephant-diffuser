//! # elephant-diffuser — the BitLocker Elephant Diffuser
//!
//! RED stub: the public API exists but performs no transform yet, so the
//! regression-vector and round-trip tests fail. The GREEN commit fills in the
//! real Diffuser A / Diffuser B / sector-key-XOR implementation.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Decrypt one sector in place. RED stub — does nothing yet.
pub fn decrypt(_sector: &mut [u8], _sector_key: &[u8; 32]) {}

/// Encrypt one sector in place. RED stub — does nothing yet.
pub fn encrypt(_sector: &mut [u8], _sector_key: &[u8; 32]) {}

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
