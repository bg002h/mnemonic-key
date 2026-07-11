//! Fuzz target: `bch_correct_regular` / `bch_correct_long` never bless an
//! invalid codeword.
//!
//! mk phase (the final phase) of the constellation stress-fuzz program
//! (Cycle C). The third target, alongside `mk1_decode` / `mk1_decode_single`
//! (which drive the whole `decode` pipeline). This one drills straight into
//! the BCH correction primitive — the F4 miscorrection-class guard
//! (constellation-eval §2 #8): the defensive re-verify at
//! `src/string_layer/bch.rs:451` (regular) / `:504` (long) that rejects any
//! proposed correction which is not actually a valid codeword.
//!
//! Input mapping: each fuzz byte becomes one 5-bit bech32 symbol (`b & 0x1F`),
//! so the whole input is a `data_with_checksum` candidate. The BCH code variant
//! is selected from the symbol count exactly as `decode_string` does
//! (`bch_code_for_length`): 14..=93 → regular, 96..=108 → long, everything else
//! skipped. Both bands are exercised in one target so libFuzzer's length
//! mutations flow between them.
//!
//! Oracles:
//! 1. Never-panic / clean-error (implicit: any panic/abort/OOM = libFuzzer
//!    failure).
//! 2. Ok ⇒ valid codeword: whenever `bch_correct_*` returns `Ok`, the corrected
//!    `data` MUST re-verify (residue == the per-HRP target constant, recomputed
//!    by an INDEPENDENT polymod that never calls `bch_verify_*` / `polymod_run`)
//!    AND `corrections_applied` MUST be ≤ 4 (the BCH t = 4 capacity). A blessed
//!    non-codeword — or a >4-substitution "repair" — is a REAL FINDING (the
//!    miscorrection class this program targets), so it aborts in-target.
#![no_main]

use libfuzzer_sys::fuzz_target;
use mk_codec::string_layer::bch::{
    GEN_LONG, GEN_REGULAR, LONG_MASK, LONG_SHIFT, POLYMOD_INIT, REGULAR_MASK, REGULAR_SHIFT,
    bch_correct_long, bch_correct_regular,
};
use mk_codec::{MK_LONG_CONST, MK_REGULAR_CONST};

/// `hrp_expand("mk")` — the fixed 5-bit prelude (see the codec's
/// `hrp_expand_mk_matches_spec` unit test).
const HRP_EXPAND_MK: [u8; 5] = [3, 3, 0, 13, 11];

/// Self-contained BIP-93 ms32 polymod, seeded with `POLYMOD_INIT`. Independent
/// of the codec's `polymod_run` / `bch_verify_*` so a correction-path bug
/// cannot also silence this oracle.
fn independent_polymod(values: &[u8], generator: &[u128; 5], shift: u32, mask: u128) -> u128 {
    let mut residue = POLYMOD_INIT;
    for &v in values {
        let b = residue >> shift;
        let mut next = ((residue & mask) << 5) ^ (v as u128);
        for (i, &g) in generator.iter().enumerate() {
            if (b >> i) & 1 != 0 {
                next ^= g;
            }
        }
        residue = next;
    }
    residue
}

fn independent_verify_regular(dwc: &[u8]) -> bool {
    let mut input = HRP_EXPAND_MK.to_vec();
    input.extend_from_slice(dwc);
    independent_polymod(&input, &GEN_REGULAR, REGULAR_SHIFT, REGULAR_MASK) == MK_REGULAR_CONST
}

fn independent_verify_long(dwc: &[u8]) -> bool {
    let mut input = HRP_EXPAND_MK.to_vec();
    input.extend_from_slice(dwc);
    independent_polymod(&input, &GEN_LONG, LONG_SHIFT, LONG_MASK) == MK_LONG_CONST
}

fuzz_target!(|data: &[u8]| {
    // One 5-bit symbol per byte; the count picks the BCH band (bch.rs
    // `bch_code_for_length`): 14..=93 regular, 96..=108 long.
    let symbols: Vec<u8> = data.iter().map(|&b| b & 0x1F).collect();
    let len = symbols.len();

    if (14..=93).contains(&len) {
        if let Ok(r) = bch_correct_regular("mk", &symbols) {
            assert!(
                independent_verify_regular(&r.data),
                "FINDING: bch_correct_regular blessed a NON-codeword (re-verify guard bypassed)"
            );
            assert!(
                r.corrections_applied <= 4,
                "FINDING: bch_correct_regular applied {} > 4 corrections",
                r.corrections_applied
            );
        }
    } else if (96..=108).contains(&len) {
        if let Ok(r) = bch_correct_long("mk", &symbols) {
            assert!(
                independent_verify_long(&r.data),
                "FINDING: bch_correct_long blessed a NON-codeword (re-verify guard bypassed)"
            );
            assert!(
                r.corrections_applied <= 4,
                "FINDING: bch_correct_long applied {} > 4 corrections",
                r.corrections_applied
            );
        }
    }
});
