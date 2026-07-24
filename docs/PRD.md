# elephant-diffuser — Design, Purpose & Scope

*A library design/scope doc, not a PRD. `elephant-diffuser` is a leaf primitive a
developer links, not a tool an examiner runs, so it has no product requirements —
only a purpose, a scope boundary, and the decisions that shaped it. The
load-bearing decisions live as ADRs under [`docs/decisions/`](decisions/); this
doc frames what the crate is for and where its edges are. Every claim below is
grounded in a same-session read of `src/lib.rs`, `Cargo.toml`, `README.md`, and
`docs/validation.md`.*

## Purpose

BitLocker's CBC-plus-diffuser sector cipher (encryption methods `0x8000` and
`0x8001`) is built from five primitives: AES, CBC, CCM, SHA-256, and the **Elephant
Diffuser**. The first four have mature, audited RustCrypto crates. The Elephant
Diffuser does not — it is a format-specific transform defined only by Microsoft's
implementation and the `dislocker`/`libbde` reverse engineering. `elephant-diffuser`
is that missing piece: a standalone, dependency-free implementation of the
diffuser stage, extracted from `bitlocker-core` so every consumer shares one
reviewed, fuzzed, oracle-validated copy instead of re-deriving a hand-written
crypto primitive N times (ADR [1](decisions/0001-hand-write-the-diffuser-the-one-crypto-exception.md),
ADR [2](decisions/0002-standalone-zero-dependency-crate-extracted-from-bitlocker-core.md)).

## What it does

Two in-place, symmetric operations over a sector byte slice:

- `decrypt(sector, sector_key)` — Diffuser B, then Diffuser A, then XOR the 32-byte
  sector key (the method-`0x8000` diffuser order).
- `encrypt(sector, sector_key)` — the exact inverse: XOR, then Diffuser A, then
  Diffuser B.

Both take a `&mut [u8]` of any length and a `&[u8; 32]` sector key, operate in
place, and never panic. Internally the sector is split into little-endian 32-bit
words; Diffuser A runs five cycles with rotation constants `Ra = {9,0,13,0}`,
Diffuser B three cycles with `Rb = {0,10,0,25}` — all following the reference
implementation (ADR [3](decisions/0003-format-fidelity-le-words-and-reference-constants.md)).

## Users

Rust consumers on the BitLocker decryption path — principally `bitlocker-core` and
any forensic mounter, carver, or re-encryption tool that needs the diffuser stage.
The audience is developers linking a crate, not analysts running a binary; there is
no CLI, GUI, or MCP surface.

## Scope

- The keyed, invertible **diffusion transform** and the per-sector-key XOR, for
  BitLocker encryption methods `0x8000` and `0x8001`.
- Faithful reproduction of the deployed BitLocker format, provable byte-for-byte
  against `libbde` on a real volume (ADR [6](decisions/0006-tier-1-validation-in-situ-via-libbde-oracle.md)).
- A total, panic-free, `#![forbid(unsafe_code)]`, zero-dependency implementation
  suitable for exposing directly to untrusted sector bytes (ADR [4](decisions/0004-forbid-unsafe-and-panic-free-modular-indexing.md)).

## Non-goals

- **Not a cipher and not confidentiality.** The diffuser holds no secret and
  provides no confidentiality; it spreads bits across a sector so a one-bit change
  cascades. Confidentiality comes from the AES-CBC layer the caller supplies.
- **Not AES/CBC/CCM/SHA-256.** Those primitives are the caller's job (via
  RustCrypto); this crate is only the piece with no ecosystem crate.
- **Not key derivation.** The caller supplies the 32-byte sector key already
  derived (BitLocker: AES-ECB over the byte offset with the TWEAK key). This crate
  does not derive, cache, or hold key material.
- **Not a security-audited cryptographic module.** It is correctness-validated, not
  independently security-audited — see the honest posture below.

## Validation approach

The diffuser produces an oracle-checkable value, so a Tier-1 oracle is mandatory.
The authoritative proof is in the consuming repo: `bitlocker-forensic`'s
`bdetogo.raw`-vs-`pybde` oracle decrypts a real BitLocker To Go volume and asserts
every sector matches `libbde` byte-for-byte; the diffuser is on that critical path.
The crate's own in-tree tests (captured regression vector, round-trip, small-`n`
robustness) are Tier-3 regression scaffolding under that oracle, and a `cargo-fuzz`
target enforces "must not panic" over arbitrary bytes. Full detail:
[`docs/validation.md`](validation.md) and ADR
[6](decisions/0006-tier-1-validation-in-situ-via-libbde-oracle.md).

## Honest security posture

`elephant-diffuser` is **correctness-validated against `libbde` on real data, not
independently security-audited.** It is a keyed diffusion transform, not a
secret-dependent cipher: it branches on no secret and holds no key material of its
own, so the timing-side-channel concerns of a block cipher do not apply. Treat it
as a faithful, reviewed implementation of a documented format primitive — nothing
more is claimed.
