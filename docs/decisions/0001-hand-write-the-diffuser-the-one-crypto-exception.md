# 1. Hand-write the Elephant Diffuser — the one documented crypto exception

Date: 2026-07-24
Status: Accepted

## Context

BitLocker's CBC-plus-diffuser sector cipher (encryption methods `0x8000` and
`0x8001`) is built from five primitives: AES, CBC, CCM, SHA-256, and the Elephant
Diffuser. The first four are solved problems with mature, audited RustCrypto
crates (`aes`, `cbc`, `ccm`, `sha2`). The Elephant Diffuser has **no crate**: it
is a format-specific transform defined only by Microsoft's implementation and the
community reverse engineering in `dislocker` (`diffuser.c`) and `libbde`. A
consumer that wants to decrypt a diffuser-protected volume therefore has to
supply this one piece itself.

The fleet's binding rule (`~/.claude/CLAUDE.core.md` → "Never hand-roll a
cryptographic primitive") is that hand-rolled crypto is wrong, unaudited, and
usually side-channel-unsafe — reach for the vetted ecosystem crate. That same law
carves out an explicit exception: roll-your-own is justified "ONLY when no mature
crate exists at all — a format-specific codec with no ecosystem implementation
(e.g. MS Xpress-Huffman / [MS-XCA])."

## Decision

Implement the Elephant Diffuser by hand as the fleet's one documented exception to
"never hand-roll crypto," and keep the exception disciplined by isolating it in a
single crate that is reviewed, fuzzed, and validated against real data. The
implementation follows the `dislocker`/`libbde` reference verbatim (see ADR
[3](0003-format-fidelity-le-words-and-reference-constants.md)) rather than any
first-principles design. See `README.md` ("Why hand-written — the one documented
exception") and the crate-level docs in `src/lib.rs`.

The diffuser is additionally a *keyed diffusion transform, not a secret-dependent
cipher*: it branches on no secret and holds no key material of its own, so the
timing-side-channel concerns that make hand-rolled block ciphers dangerous do not
apply. That materially lowers the risk of taking the exception here.

## Consequences

The fleet gains a BitLocker decryption capability it could not otherwise assemble
from crates.io. The exception is bounded to exactly one small, auditable crate
rather than scattered inline in every consumer. Correctness cannot lean on an
audit that does not exist, so it is carried entirely by validation against an
independent oracle (ADR [6](0006-tier-1-validation-in-situ-via-libbde-oracle.md)):
this crate is correctness-validated against `libbde` on real data, **not**
independently security-audited, and the README/`docs/validation.md` state that
limit plainly rather than over-claiming.
