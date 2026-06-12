//! Fuzz target: `mk_codec::decode` over a SINGLE whole-input string.
//!
//! mk phase (the final phase) of the constellation stress-fuzz program
//! (Cycle C).
//!
//! The complement to `mk1_decode`: instead of the sentinel splitter, the WHOLE
//! fuzz input is passed (utf8-lossy) as ONE `&str` — `decode(&[s])`. This
//! drives the string-layer header parse + the single-vs-chunked dispatch
//! (pipeline.rs:118-137) on unsplit bytes, which the splitter target's `\n`
//! framing never produces.
//!
//! NOTE on the fixed-point oracle's reachability here: real mk1 cards are
//! ALWAYS multi-chunk (minimal bytecode ~84 bytes > the 56-byte single-string
//! ceiling, pipeline.rs:73/:163), so `decode(&[single_chunk])` returns a CLEAN
//! `Err` ("received 1 chunks, header declares total_chunks = N") rather than
//! `Ok` for any real card chunk. The fixed-point branch is therefore rarely
//! (if ever) taken on a single string — but it stays wired (identical to
//! `mk1_decode`) so that IF some input ever does decode as a complete card via
//! the single-string path, the re-encode asymmetry is still checked. The
//! dominant value of this target is Oracle 1 (never-panic) over the unsplit
//! header-parse/dispatch surface.
//!
//! Oracles:
//! 1. Never-panic / clean-error (implicit: any panic/abort/OOM = libFuzzer
//!    failure).
//! 2. Decode -> re-encode fixed-point (R0 [Q5]/[I6]), via
//!    `encode_with_chunk_set_id(&card, FIXED_CSI)` — NOT `encode` (CSPRNG
//!    chunk_set_id, R0 [I2]). A re-encode `Err` on a decode-accepted value is
//!    a REAL FINDING and panics in-target (R0 [I6]).
#![no_main]

use libfuzzer_sys::fuzz_target;
use mk_codec::{decode, encode_with_chunk_set_id};

/// Fixed chunk_set_id for the re-encode oracle — see `mk1_decode` (R0 [I2]).
const FIXED_CSI: u32 = 0;

fuzz_target!(|data: &[u8]| {
    // mk1 strings are ASCII; the U+FFFD collapse just wastes a sliver of the
    // input space (R0 [M7]).
    let s = String::from_utf8_lossy(data);

    if let Ok(card) = decode(&[s.as_ref()]) {
        let reencoded = encode_with_chunk_set_id(&card, FIXED_CSI)
            .expect("FINDING: decode-accepted KeyCard failed to re-encode");
        let parts2: Vec<&str> = reencoded.iter().map(String::as_str).collect();
        let card2 = decode(&parts2).expect("FINDING: re-encoded mk1 card failed to decode");
        assert_eq!(
            card, card2,
            "FINDING: decode/re-encode/decode is not a fixed point"
        );
    }
});
