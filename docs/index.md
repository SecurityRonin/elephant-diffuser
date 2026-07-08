# elephant-diffuser

The **BitLocker Elephant Diffuser** as a standalone, pure-Rust primitive: Diffuser
A, Diffuser B, and the sector-key XOR that together form the diffuser stage of
BitLocker's CBC-plus-diffuser sector cipher (encryption methods `0x8000` and
`0x8001`).

!!! info "Why a crate?"
    Every other primitive in BitLocker's cipher (AES, CBC, CCM, SHA-256) has an
    audited RustCrypto crate. The Elephant Diffuser does **not** — no ecosystem
    implementation exists — so it is the one documented exception to "never
    hand-roll crypto." Isolating it here, validated against `libbde` on real data,
    lets every consumer reuse one reviewed implementation instead of re-deriving
    the rotation constants and cycle order.

## What it is

The Elephant Diffuser is **not** a cipher and holds no secret: it is a keyed,
invertible byte-mixing transform applied to a sector after AES-CBC decryption (and
before AES-CBC encryption). It spreads each plaintext bit across the whole sector
so that a single ciphertext change cascades — a diffusion layer, not
confidentiality. Its inputs are the sector bytes and a 32-byte per-sector key the
caller derives (BitLocker derives it by AES-ECB over the sector offset with the
TWEAK key).

## API

```rust
// Decrypt: Diffuser B, then Diffuser A, then XOR the 32-byte sector key.
elephant_diffuser::decrypt(&mut sector, &sector_key);

// Encrypt (the exact inverse): XOR the sector key, then Diffuser A, then B.
elephant_diffuser::encrypt(&mut sector, &sector_key);
```

Both operate in place on a sector of any length (BitLocker uses 512-byte sectors;
trailing bytes shorter than a 32-bit word are XOR-mixed but not diffused, matching
the reference). Both are panic-free on arbitrary input — `#![forbid(unsafe_code)]`,
no `unwrap`/`expect` in production, every index computed with underflow-free
modular arithmetic.

## Trust but verify

The rotation constants (`Ra = {9,0,13,0}`, `Rb = {0,10,0,25}`), the five-cycle /
three-cycle counts, and the B-then-A-then-XOR order all follow the `dislocker`
(`diffuser.c`) / `libbde` reference. Correctness is proven **in situ**: the
extracted code still decrypts the real dfvfs `bdetogo.raw` volume byte-for-byte
against `pybde` inside `bitlocker-core`'s Tier-1 oracle. See
[Validation](validation.md).

[Privacy Policy](https://securityronin.github.io/elephant-diffuser/privacy/) · [Terms of Service](https://securityronin.github.io/elephant-diffuser/terms/) · © 2026 Security Ronin Ltd
