# elephant-diffuser

[![Crates.io](https://img.shields.io/crates/v/elephant-diffuser.svg)](https://crates.io/crates/elephant-diffuser)
[![Docs.rs](https://img.shields.io/docsrs/elephant-diffuser?label=docs.rs)](https://docs.rs/elephant-diffuser)
[![Rust 1.81+](https://img.shields.io/badge/rust-1.81%2B-blue.svg)](https://www.rust-lang.org)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Sponsor](https://img.shields.io/badge/sponsor-h4x0r-ea4aaa?logo=githubsponsors)](https://github.com/sponsors/h4x0r)

[![CI](https://github.com/SecurityRonin/elephant-diffuser/actions/workflows/ci.yml/badge.svg)](https://github.com/SecurityRonin/elephant-diffuser/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/badge/coverage-100%25%20lines-brightgreen.svg)](docs/validation.md)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance)
[![Security advisories](https://img.shields.io/badge/advisories-clean-success.svg)](https://rustsec.org)

**The BitLocker Elephant Diffuser as a standalone, dependency-free Rust primitive —
the one part of BitLocker's sector cipher no RustCrypto crate provides, validated
byte-for-byte against `libbde` on a real volume.**

BitLocker's CBC-plus-diffuser methods (`0x8000`, `0x8001`) wrap AES-CBC in a keyed
byte-mixing layer — the Elephant Diffuser. AES, CBC, CCM and SHA-256 all have
audited crates; the diffuser does not. This crate is that missing piece, extracted
from `bitlocker-core` so every consumer shares one reviewed implementation.

```rust
use elephant_diffuser::{decrypt, encrypt};

let mut sector = vec![0u8; 512];
let sector_key = [0u8; 32]; // caller-derived (BitLocker: AES-ECB over the offset with the TWEAK key)

// Decrypt: Diffuser B, then Diffuser A, then XOR the sector key.
decrypt(&mut sector, &sector_key);

// Encrypt is the exact inverse: XOR, then Diffuser A, then Diffuser B.
encrypt(&mut sector, &sector_key);
```

## What it is (and isn't)

The Elephant Diffuser is a **keyed, invertible diffusion transform**, not a cipher:
it holds no secret and provides no confidentiality. Applied to a sector *after*
AES-CBC decryption (and before AES-CBC encryption), it spreads each bit across the
whole sector so a one-bit change cascades. Confidentiality comes from AES; this is
the diffusion layer around it.

- `decrypt(sector, sector_key)` — Diffuser B → Diffuser A → XOR the 32-byte sector key.
- `encrypt(sector, sector_key)` — the exact inverse: XOR → Diffuser A → Diffuser B.

Both operate in place on a sector of any length (BitLocker uses 512-byte sectors).

## Why hand-written — the one documented exception

The fleet rule is *never hand-roll crypto — use an audited crate*. The Elephant
Diffuser is the exception, because **no crate exists**: it is a format-specific
transform defined only by Microsoft's implementation and the community reverse
engineering in `dislocker` (`diffuser.c`) and `libbde`. The rotation constants
(`Ra = {9,0,13,0}`, `Rb = {0,10,0,25}`), the five- and three-cycle counts, and the
B-then-A-then-XOR order follow that reference. Isolating it in one crate — reviewed,
fuzzed, and validated against real data — is how the exception stays disciplined.

## Trust but verify

- **`#![forbid(unsafe_code)]`, panic-free, zero dependencies.** No `unwrap`/`expect`
  in production; every index is computed with underflow-free modular arithmetic, so
  the transform never panics on any input length. A `cargo-fuzz` target drives
  `decrypt`/`encrypt` over arbitrary bytes with the "must not panic" invariant.
- **Tier-1 validated, in situ.** The authoritative proof is `bitlocker-core`'s
  oracle: this code decrypts the real dfvfs `bdetogo.raw` volume byte-for-byte
  against `pybde`. It is **correctness-validated against `libbde` on real data,
  not independently security-audited** — a keyed diffusion transform, not a
  secret-branching cipher. See [`docs/validation.md`](docs/validation.md).

[Privacy Policy](https://securityronin.github.io/elephant-diffuser/privacy/) · [Terms of Service](https://securityronin.github.io/elephant-diffuser/terms/) · © 2026 Security Ronin Ltd
