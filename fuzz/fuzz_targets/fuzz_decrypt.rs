#![no_main]
//! Fuzz the diffuser transforms over arbitrary bytes and an arbitrary sector
//! key. Invariant: must never panic — regardless of length (including 0 and
//! sub-word / 1..=4-word buffers, where a naive back-index would underflow).

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let key = [0x5au8; 32];
    let mut buf = data.to_vec();
    elephant_diffuser::decrypt(&mut buf, &key);
    elephant_diffuser::encrypt(&mut buf, &key);
});
