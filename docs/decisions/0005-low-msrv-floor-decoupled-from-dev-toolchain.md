# 5. Low, CI-verified MSRV floor decoupled from the pinned dev toolchain

Date: 2026-07-24
Status: Accepted

## Context

`elephant-diffuser` is a **published library** — a leaf primitive meant to be
linked by `bitlocker-core` and any other consumer that needs BitLocker's diffuser.
The fleet's MSRV policy (`~/.claude/CLAUDE.core.md` → "Rust MSRV & Toolchain
Policy") separates the *dev toolchain* (what the fleet builds and lints with, a
single current stable) from the *declared MSRV* (a downstream-facing compatibility
promise). Published libraries keep a **low, CI-verified MSRV** because raising it
narrows the crates.io audience; apps and binaries instead pin MSRV to the dev
toolchain. This crate's `rust-toolchain.toml` pins the fleet dev toolchain
(`1.96.0`), which is a build-with version, not a promise.

## Decision

Declare a low MSRV floor of **`rust-version = "1.81"`** in `Cargo.toml`,
independent of the `1.96.0` dev-toolchain pin in `rust-toolchain.toml`. The floor
is a compatibility guarantee to consumers; the pin is only what CI and local
development build with. The crate uses no post-1.81 language feature, so the floor
can stay low. Raise it only if a genuinely newer-Rust feature is ever needed, and
never merely to match the toolchain.

Rationale reconstructed from structure and the fleet MSRV policy; the specific
choice of `1.81` rather than the `1.75`/`1.80` values the policy cites as examples
is not recorded in the git history, so the exact floor value is treated as a
low-as-feasible floor rather than a documented requirement.

## Consequences

The crate stays consumable by older toolchains, which matters for a widely-linkable
primitive. The two-number split means a contributor builds with 1.96.0 while the
promise to downstream stays at 1.81; a CI MSRV job must verify the crate actually
compiles at the declared floor so the promise is real and not aspirational. Because
the floor value's precise origin is undocumented, a future reviewer may lower it
toward `1.75`/`1.80` if broader reach is wanted and nothing in the crate blocks it.
