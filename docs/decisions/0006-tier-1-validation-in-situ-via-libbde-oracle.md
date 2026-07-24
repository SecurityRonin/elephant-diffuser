# 6. Tier-1 validation in situ via the `libbde`/`pybde` oracle; in-tree tests are Tier-3

Date: 2026-07-24
Status: Accepted

## Context

The diffuser produces a value an independent oracle can check, so under the
fleet's Evidence-Based Rigor discipline a Tier-1 oracle is **mandatory** — a
self-authored round-trip would only prove self-consistency (the LZNT1 trap), never
correctness. But this crate is a pure primitive with no I/O: it cannot, by itself,
open a BitLocker volume. The obvious temptation — bundle a synthetic fixture and a
captured expected value and call the round-trip green — is exactly the tier-3
self-deception the discipline forbids as the *sole* validation of an
oracle-checkable value.

## Decision

Rely on the **in-situ Tier-1 oracle that already exists in the consuming repo**
rather than re-hosting one here. The authoritative proof is `bitlocker-forensic`'s
`core/tests/oracle_bdetogo.rs` (env-gated on `BDE_ORACLE_IMAGE`): it unlocks the
real dfvfs `bdetogo.raw` BitLocker To Go volume (method `0x8000`, AES-128-CBC +
Elephant Diffuser, published password `bde-TEST`) and asserts every decrypted
512-byte sector matches `pybde` (libbde 20240502) **byte-for-byte** by SHA-256. The
diffuser is on the critical path of that decryption, so any wrong constant, cycle
count, index, or stage order fails the oracle. The image is third-party
(log2timeline/dfvfs, Apache-2.0) and ground truth is an independent tool — a
genuine Tier-1 check. The oracle continuing to pass *after* extraction is the proof
this crate preserves the validated behaviour. See `docs/validation.md`.

The crate's own in-tree tests are labelled **Tier-3** honestly: the captured
regression vector's expected bytes were taken from the already-Tier-1-validated
`bitlocker-core` implementation before extraction, and the round-trip proves only
that `encrypt` and `decrypt` are inverses. They are fast regression scaffolding
*under* the Tier-1 oracle, not independent proof.

## Consequences

Correctness rests on real-world data checked by an independent reference, not on
fixtures this repo authored — the bar the discipline sets for an oracle-checkable
codec. The trade-off is that the authoritative check lives in another repo and is
env-gated (skips cleanly when `BDE_ORACLE_IMAGE` is absent), so this crate's own CI
runs only the Tier-3 regression/round-trip/robustness tests plus the fuzz target;
the Tier-1 guarantee is asserted where the volume can actually be opened.
`docs/validation.md` states the honest security posture: correctness-validated
against `libbde`, **not** independently security-audited.
