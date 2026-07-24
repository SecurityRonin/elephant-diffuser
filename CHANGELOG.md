# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/SecurityRonin/elephant-diffuser/compare/elephant-diffuser-v0.1.0...elephant-diffuser-v0.1.1) - 2026-07-24

### Documentation

- reverse-write PRD + ADRs; mkdocs excludes governance docs (fleet standard)
- *(readme)* align robustness wording with fleet standard (fuzzed + panic-free-by-lint pairing)
- use verbatim Apache-2.0 license text

## [0.1.0]

### Added

- The BitLocker **Elephant Diffuser** as a standalone, dependency-free primitive,
  extracted from `bitlocker-core`:
  - `decrypt(sector, sector_key)` — Diffuser B, then Diffuser A, then XOR the
    32-byte sector key (the method-`0x8000` diffuser order).
  - `encrypt(sector, sector_key)` — the exact inverse: XOR, then Diffuser A, then
    Diffuser B.
  - Rotation constants `Ra = {9,0,13,0}`, `Rb = {0,10,0,25}` and the five- /
    three-cycle counts follow the `dislocker` (`diffuser.c`) / `libbde` reference.
  - `#![forbid(unsafe_code)]`, panic-free on any input length (underflow-free
    modular indexing), no `unwrap`/`expect` in production, zero dependencies.
  - Tier-1 validated **in situ** by `bitlocker-core`'s `bdetogo.raw`-vs-`pybde`
    oracle; a `cargo-fuzz` `decrypt`/`encrypt` target enforces "must not panic."
