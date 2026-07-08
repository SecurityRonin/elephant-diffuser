# Validation

The Elephant Diffuser produces a value an independent oracle can check, so a
Tier-1 oracle is mandatory (a self-authored round-trip would only prove
self-consistency — the LZNT1 trap). This crate has one, borne by its origin.

## Tier-1 (authoritative) — in-situ against `libbde`/`pybde`

This code was **extracted from `bitlocker-core`**, where it is exercised by a
Tier-1 oracle: unlocking the real dfvfs `bdetogo.raw` BitLocker To Go volume
(method `0x8000`, AES-128-CBC + Elephant Diffuser, published password `bde-TEST`)
and asserting that every decrypted 512-byte sector matches `pybde` (libbde
20240502) **byte-for-byte** (SHA-256). The diffuser is on the critical path of
that decryption: if any rotation constant, cycle count, index, or the
B-then-A-then-XOR order were wrong, those sectors would not match.

That oracle lives in the consuming repo — `bitlocker-forensic`,
`core/tests/oracle_bdetogo.rs` (env-gated on `BDE_ORACLE_IMAGE`) — and continues
to pass after the extraction, which is the proof that this crate preserves the
validated behaviour. The image is third-party (log2timeline/dfvfs, Apache-2.0), not
authored by us; ground truth is an independent tool. That is a genuine Tier-1
check.

## Tier-3 (regression scaffolding) — the crate's own unit tests

The crate's in-tree tests are **Tier-3** — fixtures and expected values captured
from the already-validated implementation, useful as fast regression guards but
self-consistent by construction, not independent proof:

- **Captured regression vector** — a fixed 512-byte input and 32-byte sector key
  run through `decrypt`, with the expected first/last bytes captured from the
  Tier-1-validated `bitlocker-core` implementation *before* the code was moved
  (compiled with `rustc -O` over the exact source). It fails loudly if the
  transform ever drifts.
- **Round-trip** — `decrypt(encrypt(x)) == x` for a full sector; proves the two
  directions are exact inverses (self-consistency only).
- **Robustness** — empty, sub-word, and 1..=4-word buffers must not panic. Real
  BitLocker sectors are ≥512 bytes (128 words), so the diffuser is only ever
  driven at word-count ≥ 5; the fuzz target and these tests cover the smaller
  counts a naive back-index would underflow on.

## Fuzzing

`fuzz/fuzz_targets/fuzz_decrypt.rs` drives `decrypt` then `encrypt` over arbitrary
bytes and a fixed key; invariant: never panic, at any length.

## Honest security posture

`elephant-diffuser` is **correctness-validated against `libbde` on real data**, not
independently security-audited. It is a keyed diffusion transform, not a
secret-dependent cipher: it branches on no secret and holds no key material of its
own, so the usual timing-side-channel concerns of a block cipher do not apply to
it. Treat it as a faithful, reviewed implementation of a documented format
primitive — nothing more is claimed.
