# 2. Standalone, zero-dependency crate extracted from bitlocker-core

Date: 2026-07-24
Status: Accepted

## Context

The diffuser began life inside `bitlocker-core`, on the critical path of that
crate's `0x8000` decryption. Other consumers that need the transform — a forensic
mounter, a carver, a re-encryption tool — would each have to reach into
`bitlocker-core` or, worse, copy the algorithm. Copy-paste of a hand-written
crypto primitive is exactly the drift-and-divergence failure the fleet's DRY and
"prefer our own crates" disciplines exist to prevent: N copies of an unaudited
transform, each free to acquire its own bug.

The transform itself is pure computation over a byte slice. It needs no I/O, no
allocation beyond a scratch `Vec<u32>` per sector, and no other crate's types.

## Decision

Extract the diffuser into its own crate, `elephant-diffuser`, with **zero
dependencies** (`[dependencies]` in `Cargo.toml` is empty) so every consumer
shares one reviewed implementation. See `CHANGELOG.md` (0.1.0, "extracted from
`bitlocker-core`") and the GREEN implementation commit `e6347de`.

Two sub-decisions follow from the leaf-primitive nature:

- **Naming.** The crate is a single flat primitive, not a reader/analyzer pair, so
  the `-core`/`-forensic` split of the crate-structure standard does not apply. It
  takes the bare descriptive name `elephant-diffuser` — self-describing on
  crates.io, in the same shape as other fleet leaves (`safe-read`).
- **No `safe-read` dependency.** The fleet routes integer field reads through
  `safe-read`, but that crate handles *fixed-width fields parsed from untrusted
  headers*. Here the input is a whole sector split into little-endian words by
  `chunks_exact(4)` (`src/lib.rs::to_words`), which is infallible by construction
  and yields no partial-read/overflow hazard. Pulling in `safe-read` would add a
  dependency for a guarantee `chunks_exact` already provides, defeating the
  zero-dependency goal. Robustness of the indexing is handled instead by ADR
  [4](0004-forbid-unsafe-and-panic-free-modular-indexing.md).

## Consequences

A consumer depends on exactly the primitive it needs, decoupled from
`bitlocker-core`'s reader surface. The single crate is the one place the
hand-written exception (ADR [1](0001-hand-write-the-diffuser-the-one-crypto-exception.md))
is reviewed and fuzzed. Zero dependencies keep the crate trivially auditable. It
is **not** `no_std`/no-alloc: `to_words` (`src/lib.rs:40`) collects each sector
into a scratch `Vec<u32>` and the crate carries no `#![no_std]` attribute, so it
declares the single `cryptography` crates.io category (`categories =
["cryptography"]`). The cost is one more crate to publish and version; the
in-situ oracle in `bitlocker-forensic` continues to prove the extracted code
preserves the validated behaviour (ADR
[6](0006-tier-1-validation-in-situ-via-libbde-oracle.md)).
