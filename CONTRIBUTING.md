# Contributing to elephant-diffuser

Thanks for your interest in improving `elephant-diffuser`. This crate implements a
BitLocker format primitive that runs on untrusted sector bytes, so correctness and
robustness are not negotiable. The bar is high and the workflow is strict — please
read this before opening a pull request.

## Test-Driven Development is mandatory

Every code change follows strict Red-Green-Refactor, and the RED and GREEN steps
land as **two separate commits**:

1. **RED** — write the failing test(s) first. They must define the expected
   behaviour and actually fail. Commit them alone. This commit is the verifiable
   proof that the test was written first.
2. **GREEN** — write the minimal implementation that makes the tests pass.
   Commit it separately.
3. **REFACTOR** — clean up while keeping every test green.

A single combined commit is not accepted.

Because the diffuser produces a value an independent oracle can check, correctness
is proven **against real data, not fixtures we authored**: the authoritative check
is `bitlocker-core`'s Tier-1 `bdetogo.raw`-vs-`pybde` oracle, which exercises this
code end-to-end. A change here must keep that oracle green.

## Quality gates

All of the following must pass locally and in CI before a PR can merge:

```bash
cargo fmt --all -- --check                       # formatting
cargo clippy --all-targets -- -D warnings        # lints, warnings denied
cargo deny check                                 # license / advisory / source policy
cargo test --all-features                        # unit tests
cargo llvm-cov --lib --show-missing-lines        # 100% line coverage
```

- **Coverage** — 100% line coverage is enforced; a provably-unreachable defensive
  arm may be annotated `// cov:unreachable: <invariant>` and is then exempt, but no
  other line may be left uncovered.
- **Fuzzing** — the `fuzz_decrypt` target's invariant is "must not panic." A change
  must keep it green; a panic found by the fuzzer is fixed and pinned as a
  regression test.

## Robustness expectations

- No panics on any input — the transform must handle a sector of any length,
  including empty and sub-word buffers; use checked or underflow-free arithmetic.
- Keep `#![forbid(unsafe_code)]` intact.
- Keep diffs minimal — change only the lines the task requires.

## Reporting security issues

Do not open a public issue for a security vulnerability. See
[SECURITY.md](SECURITY.md) for the private reporting process.
