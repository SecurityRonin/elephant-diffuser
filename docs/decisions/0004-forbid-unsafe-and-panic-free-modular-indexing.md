# 4. `forbid(unsafe)` + panic-free modular indexing for any word count

Date: 2026-07-24
Status: Accepted

## Context

The diffuser transforms attacker-controllable sector bytes, so it falls under the
fleet's Paranoid Gatekeeper standard: never panic, never read out of bounds. Two
concrete hazards exist. First, the reference algorithm's back-references are
written as `(i + n - 5) % n`; for the real BitLocker case a sector is ≥512 bytes
(128 words, `n ≥ 5`) and this is fine, but a fuzzer driving `decrypt`/`encrypt`
over arbitrary bytes can present `n` in `1..=4`, where `i + n - 5` underflows
`usize` and panics. Second, this crate — unlike the mmap-backed container readers
(`ewf`, `memory-forensic`) that must downgrade to `unsafe_code = "deny"` — needs
no `unsafe` at all: it touches only owned slices and a scratch `Vec`.

## Decision

- **`#![forbid(unsafe_code)]`** (`src/lib.rs`, mirrored by `unsafe_code = "forbid"`
  in `Cargo.toml`). `forbid` (not `deny`) is chosen because there is no mmap or
  other justified `unsafe` site to allow, so the crate earns the stronger,
  un-overridable posture and the `unsafe forbidden` badge honestly.
- **Panic-free by lint:** `unwrap_used` and `expect_used` are `deny`; production
  code uses neither. Tests opt out via `#![cfg_attr(test, allow(...))]`.
- **Underflow-free modular indexing:** back-references go through
  `fn back(i, k, n) = (i + n - (k % n)) % n` (`src/lib.rs`), which equals the
  reference `(i + n - k) % n` for the real `n ≥ 5` case and additionally stays
  panic-free for the `1..=4`-word buffers a fuzzer can present. `from_words` guards
  its write with `get_mut` so it cannot panic even though the index is provably in
  range. `wrapping_add`/`wrapping_sub` make the arithmetic total.
- A `cargo-fuzz` target (`fuzz/fuzz_targets/fuzz_decrypt.rs`) drives
  `decrypt` then `encrypt` over arbitrary bytes with the "must not panic"
  invariant, and unit tests
  (`empty_and_subword_inputs_do_not_panic`, `tiny_sector_word_counts_do_not_panic`)
  lock in the small-`n` behaviour.

## Consequences

The transform is total: it returns for every input length without panicking or
reading out of bounds, so it is safe to expose directly to untrusted sector data.
The `forbid(unsafe)` posture is provable and badge-able. The `back()` helper adds a
tiny cost (a `% n` on `k`) that never matters at real sector sizes but buys
fuzz-safety at the sub-sector sizes real volumes never produce — a deliberate
robustness-over-micro-optimisation trade, consistent with defense-in-depth.
