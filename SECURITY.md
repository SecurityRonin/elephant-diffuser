# Security Policy

`elephant-diffuser` implements the BitLocker Elephant Diffuser — a keyed byte-mixing
transform applied to **untrusted sector bytes** from BitLocker (BDE) volumes,
including images acquired from compromised or actively hostile systems. Hostile
input is the expected case. Robustness against arbitrary sector content and length
is a core design goal, and we take reports of crashes, hangs, or memory-safety
issues seriously.

## Supported versions

| Version | Supported |
|---|---|
| 0.1.x   | ✅ — current development line |
| < 0.1   | ❌ — pre-release, unsupported |

## Reporting a vulnerability

**Do not open a public GitHub issue for a security vulnerability.**

Report privately, by either:

- **GitHub Security Advisories** — open a private advisory on the
  [`elephant-diffuser` repository](https://github.com/SecurityRonin/elephant-diffuser/security/advisories/new), or
- **Email** — [albert@securityronin.com](mailto:albert@securityronin.com).

Please include:

- the affected version and target triple,
- a minimal reproducing input (a fuzz corpus entry is ideal),
- the observed behaviour (panic, hang, excessive allocation, wrong output) and the
  expected behaviour.

We aim to acknowledge a report within a few business days and to coordinate
disclosure once a fix is available.

## Security posture

- **`#![forbid(unsafe_code)]`** — no `unsafe`, anywhere.
- **No panics on any input** — the transform runs on a sector of any length,
  including empty and sub-word buffers; every index is computed with
  underflow-free modular arithmetic, so no length can drive an out-of-bounds
  access or an arithmetic overflow.
- **No `unwrap`/`expect` in production** — `clippy::unwrap_used`/`expect_used` are
  denied outside tests.
- **Continuous fuzzing** with [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz):
  `decrypt`/`encrypt` over arbitrary bytes, invariant "must not panic," smoke-run
  on every push/PR and fuzzed for ten minutes weekly.

## Scope of the primitive

The Elephant Diffuser holds no secret and branches on no secret; it is a diffusion
layer, not a confidentiality cipher. `elephant-diffuser` is correctness-validated
against `libbde` on real data (see [`docs/validation.md`](docs/validation.md)), not
independently security-audited.
