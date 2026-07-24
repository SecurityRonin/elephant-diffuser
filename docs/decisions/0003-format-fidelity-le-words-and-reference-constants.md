# 3. Format fidelity — little-endian words, reference constants, B→A→XOR order

Date: 2026-07-24
Status: Accepted

## Context

The Elephant Diffuser is defined by behaviour, not by a public specification. Any
implementation must reproduce Microsoft's exact byte mixing or the decrypted
sectors will not match a real volume. The load-bearing structural choices are: how
a sector is split into machine words, the two sets of rotation constants, the
per-diffuser cycle counts, and the order in which Diffuser A, Diffuser B, and the
sector-key XOR are applied. Every one of these is a place a plausible-but-wrong
guess would compile, pass a self-authored round-trip, and still produce garbage
against `libbde` (the "LZNT1 trap").

## Decision

Mirror the `dislocker` (`diffuser.c`) / `libbde` reference exactly:

- **Endianness.** Split the sector into **little-endian** 32-bit words
  (`src/lib.rs::to_words`, `u32::from_le_bytes`). Real BitLocker sectors are
  word-aligned (512+ bytes) and the production caller passes a block-aligned length
  (`bitlocker-forensic` `core/src/crypto.rs:332` hands `buf[..plain_len]` with
  `plain_len` a multiple of 16), so a trailing sub-word remainder never arises in
  practice and the reference does not specify one. Should an out-of-spec length
  present one (e.g. a fuzzer's), this crate drops it from the diffused words while
  the per-sector-key XOR still covers it — a defined, panic-free choice of this
  crate's, not reference-matched behaviour.
- **Rotation constants.** `Ra = {9, 0, 13, 0}` and `Rb = {0, 10, 0, 25}`, indexed
  by word position `i % 4` (`RA`/`RB` in `src/lib.rs`).
- **Cycle counts.** Diffuser A runs **five** cycles, Diffuser B **three**
  (`diffuser_a_*` / `diffuser_b_*`).
- **Neighbour offsets.** Diffuser A mixes `d[i-2]` and `ROL(d[i-5], Ra)`; Diffuser
  B mixes `d[i+2]` and `ROL(d[i+5], Rb)`.
- **Stage order.** `decrypt` applies Diffuser B, then Diffuser A, then XOR the
  32-byte sector key; `encrypt` is the exact inverse (XOR, then Diffuser A, then
  Diffuser B). This is the method-`0x8000` order.

The `encrypt` direction is implemented as the literal inverse of `decrypt`:
descending index order and `wrapping_sub` where `decrypt` ascends and
`wrapping_add`. See `README.md` and `CHANGELOG.md` (0.1.0), which record the
constants and order against the reference.

## Consequences

The transform is faithful to the only ground truth that exists — the deployed
BitLocker format — and that fidelity is provable, not asserted: the in-situ oracle
(ADR [6](0006-tier-1-validation-in-situ-via-libbde-oracle.md)) would fail on any
wrong constant, cycle count, offset, or stage order, since the diffuser sits on
the critical path of the `bdetogo.raw` decryption. The constants are format facts,
not tunables; changing any of them is a correctness regression, not a
configuration choice.
