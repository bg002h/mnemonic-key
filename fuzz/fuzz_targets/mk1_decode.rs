//! Fuzz target: `mk_codec::decode` over a multi-chunk mk1 card.
//!
//! mk phase (the final phase) of the constellation stress-fuzz program
//! (Cycle C).
//!
//! Structured multi-chunk input uses a SENTINEL-BYTE splitter (R0 [M2]): split
//! the fuzz input on `\n` (0x0A, outside the bech32 alphabet/HRP/separator)
//! into <=8 parts (truncate excess), utf8-lossy each, then call
//! `mk_codec::decode(&parts)`. A libFuzzer insert/delete then moves ONE chunk
//! boundary locally instead of re-shearing all of them. `decode` takes
//! `&[&str]` (key_card.rs:115), so the parts are borrowed `&str` views into
//! owned `String`s.
//!
//! Real mk1 cards are ALWAYS multi-chunk: the minimal bytecode (~84 bytes,
//! dominated by the 73-byte compact xpub) exceeds the 56-byte single-string
//! ceiling (`SINGLE_STRING_LONG_BYTES`; pipeline.rs:73/:163), so this
//! splitter target is the one that can drive `decode` to an `Ok(KeyCard)`.
//!
//! Oracles:
//! 1. Never-panic / clean-error (implicit: any panic/abort/OOM = libFuzzer
//!    failure).
//! 2. Decode -> re-encode fixed-point (R0 [Q5]/[I6]): on `Ok(card)`,
//!    `encode_with_chunk_set_id(&card, FIXED_CSI)` then `decode` again and
//!    assert the recovered `KeyCard` equals the first (`KeyCard` derives
//!    `PartialEq, Eq`, key_card.rs:23). The re-encode MUST use
//!    `encode_with_chunk_set_id` with a FIXED chunk_set_id, NOT `encode` —
//!    `encode` draws a fresh 20-bit chunk_set_id from the system CSPRNG
//!    (key_card.rs:95-100 / pipeline.rs:56), whose nondeterminism would break
//!    crash reproducibility and coverage stability (R0 [I2]). `KeyCard` does
//!    not carry the chunk_set_id (it is wire-only), so a fixed value yields a
//!    sound value-level fixed point. A re-encode `Err` on a decode-accepted
//!    value is a REAL FINDING — the decode/encode-asymmetry class the charter
//!    targets — so it panics in-target rather than being swallowed (R0 [I6]).
#![no_main]

use libfuzzer_sys::fuzz_target;
use mk_codec::{decode, encode_with_chunk_set_id};

/// Maximum chunk count accepted by the splitter; extra parts are dropped.
const MAX_PARTS: usize = 8;

/// Fixed chunk_set_id for the re-encode oracle (any 20-bit value works; 0 is
/// the simplest). Using a fixed value keeps the multi-chunk re-encode
/// byte-deterministic so a crashing input always reproduces (R0 [I2]).
const FIXED_CSI: u32 = 0;

fuzz_target!(|data: &[u8]| {
    // mk1 strings are ASCII; the U+FFFD collapse from lossy conversion just
    // wastes a sliver of input space (R0 [M7]).
    let owned: Vec<String> = data
        .split(|&b| b == b'\n')
        .take(MAX_PARTS)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect();
    let parts: Vec<&str> = owned.iter().map(String::as_str).collect();

    if let Ok(card) = decode(&parts) {
        // Re-encode the accepted card with a FIXED chunk_set_id, decode again,
        // and assert the round trip is a fixed point. Err on either step is a
        // finding.
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
