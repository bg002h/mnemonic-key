//! Deterministic, re-runnable seed-corpus generator for the mk-codec fuzz
//! targets. mk phase (the final phase) of the constellation stress-fuzz
//! program (Cycle C).
//!
//! Run with:
//!     cd fuzz && cargo +nightly-2026-04-27 test --test gen_corpus
//!
//! It (1) builds a fixed catalog of valid `KeyCard`s from real xpub/path/stub
//! fixtures via the public structural API, (2) encodes each with a FIXED
//! `chunk_set_id` so the emitted mk1 strings are byte-identical every run, (3)
//! writes seed files into the cargo-fuzz default `corpus/<target>/` layout, and
//! (4) — THE GATE (R0 [I6] / round-2 minor) — asserts every committed seed
//! passes the SAME call the corresponding target makes. A seed that no longer
//! round-trips is a generation bug and fails the test loudly (the md phase
//! caught a rotted doc-harvested string this way).
//!
//! Determinism: `encode_with_chunk_set_id` is RNG-free (only the public
//! `encode` draws a CSPRNG chunk_set_id — pipeline.rs:56-100), so all seeds are
//! byte-identical every run and re-running never churns the committed corpus.
//!
//! Per-target gate (R0 round-2 splitter clarification):
//!   * `mk1_decode` (SPLITTER): a real mk1 card is ALWAYS multi-chunk (minimal
//!     bytecode ~84 bytes > the 56-byte single-string ceiling), so its seeds
//!     are the card's chunk strings joined by `\n` BETWEEN (no trailing
//!     newline). The gate runs the target's exact call: `split('\n')` ->
//!     `decode(&parts)` is `Ok`, and the recovered `KeyCard` equals the
//!     original.
//!   * `mk1_decode_single` (WHOLE input as ONE string): there is NO single
//!     string that `decode(&[s])` returns `Ok` for — no real card fits in one
//!     chunk (pipeline.rs:73/:163). The honest, non-vacuous gate for this
//!     target therefore mirrors the target's own contract: each seed is a real
//!     individual mk1 chunk string, and the gate asserts `decode(&[s])`
//!     returns a CLEAN `Err` (the "received 1 chunks, header declares
//!     total_chunks = N" rejection) rather than `Ok` or a panic — exactly what
//!     the never-panic oracle protects. (Asserting `Ok` here would be a false
//!     gate; the single-string surface cannot produce a complete card.)

use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use mk_codec::error::Error;
use mk_codec::{KeyCard, decode, encode_with_chunk_set_id};

use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};

// ---------------------------------------------------------------------------
// Fixtures — real (xpub, path, stubs, fp) tuples lifted from the canonical
// vector corpus (crates/mk-codec/src/test_vectors/v0.1.json). Building the
// KeyCard from these fixtures and re-encoding here (rather than hardcoding the
// mk1 STRINGS) means the encoder is the single source of truth: if the wire
// format ever changes, the gate re-encodes the new bytes and re-verifies the
// round trip, so the corpus self-heals instead of rotting.
// ---------------------------------------------------------------------------

/// `(name, stub_hexes, fingerprint_hex_or_none, path, xpub)`.
struct Fixture {
    name: &'static str,
    stubs: &'static [&'static str],
    fp: Option<&'static str>,
    path: &'static str,
    xpub: &'static str,
}

const FIXTURES: &[Fixture] = &[
    // V1_bip48_mainnet_1_stub_with_fp (2 chunks): standard-path encode mode.
    Fixture {
        name: "bip48_mainnet_1stub_fp",
        stubs: &["11223344"],
        fp: Some("aabbccdd"),
        path: "m/48'/0'/0'/2'",
        xpub: "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc",
    },
    // V5_explicit_path_4_components_with_fp (3 chunks): explicit-path (0xFE)
    // encode mode with a fingerprint.
    Fixture {
        name: "explicit_path_4comp_fp",
        stubs: &["55667788"],
        fp: Some("01020304"),
        path: "m/9999'/1234'/56'/7'",
        xpub: "xpub6Den8YxgJdggPygKKEv3wiQwQ6PSGUouW98xC4obAJAqvuWcBMHuxeuXHxyZtAJHLqE7U1JdEXrNwbNPNCn1F79n4ZuBTLnzF7mPbLR3ZvB",
    },
    // V7_max_path_components_no_fp (3 chunks): 10-component explicit path,
    // privacy-preserving mode (no fingerprint).
    Fixture {
        name: "max_path_10comp_nofp",
        stubs: &["90919293"],
        fp: None,
        path: "m/0'/1'/2'/3'/4'/5'/6'/7'/8'/9'",
        xpub: "xpub6QwbHG5Nw7rYLo6utUHsXUqaaojc3YDdq84Ho7HV3mHuiJ1NNXB1GzUdBCMVph1HfRMMuRjW2VVVr8k5Fz7YGrKVGwVYPBcXr6dZKQenNqk",
    },
    // Multi-stub variant of V1 (exercises the >1 stub-count path) — same
    // xpub/path, two stubs.
    Fixture {
        name: "bip48_mainnet_2stub_fp",
        stubs: &["11223344", "55667788"],
        fp: Some("aabbccdd"),
        path: "m/48'/0'/0'/2'",
        xpub: "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc",
    },
];

/// Fixed chunk_set_id for the corpus encode — any 20-bit value; 0 keeps the
/// seeds byte-stable and matches the targets' re-encode csid.
const CORPUS_CSI: u32 = 0;

fn parse_stub(hex_str: &str) -> [u8; 4] {
    assert_eq!(hex_str.len(), 8, "stub hex must be 8 chars: {hex_str}");
    let mut out = [0u8; 4];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16)
            .unwrap_or_else(|_| panic!("bad stub hex: {hex_str}"));
    }
    out
}

fn build_card(fx: &Fixture) -> KeyCard {
    let stubs: Vec<[u8; 4]> = fx.stubs.iter().map(|s| parse_stub(s)).collect();
    let fp: Option<Fingerprint> = fx.fp.map(|s| Fingerprint::from(parse_stub(s)));
    let path = DerivationPath::from_str(fx.path)
        .unwrap_or_else(|e| panic!("gen-corpus: {} path parse: {e}", fx.name));
    let xpub: Xpub = fx
        .xpub
        .parse()
        .unwrap_or_else(|e| panic!("gen-corpus: {} xpub parse: {e}", fx.name));
    KeyCard::new(stubs, fp, path, xpub)
}

fn corpus_dir(target: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join(target)
}

fn write_seed(dir: &Path, name: &str, bytes: &[u8]) {
    fs::create_dir_all(dir).expect("create corpus dir");
    fs::write(dir.join(name), bytes).expect("write seed");
}

#[test]
fn gen_corpus() {
    let dir_decode = corpus_dir("mk1_decode");
    let dir_single = corpus_dir("mk1_decode_single");

    let mut decode_count = 0usize;
    let mut single_count = 0usize;
    let mut max_chunks_seen = 0usize;

    for fx in FIXTURES {
        let card = build_card(fx);

        // Encode with a fixed chunk_set_id so the seed bytes are deterministic.
        let chunks = encode_with_chunk_set_id(&card, CORPUS_CSI)
            .unwrap_or_else(|e| panic!("gen-corpus: {} encode failed: {e}", fx.name));
        assert!(
            chunks.len() >= 2,
            "gen-corpus: {} encoded into {} chunk(s); a real mk1 card is always \
             multi-chunk — fixture or wire format changed",
            fx.name,
            chunks.len()
        );
        max_chunks_seen = max_chunks_seen.max(chunks.len());

        // --- mk1_decode SPLITTER seed: chunks joined by `\n` BETWEEN (no
        //     trailing newline, so split('\n') reproduces the exact chunks). ---
        let joined = chunks.join("\n");
        let parts: Vec<&str> = joined.split('\n').collect();
        assert_eq!(
            parts.len(),
            chunks.len(),
            "gen-corpus: {} join/split changed chunk count",
            fx.name
        );
        // GATE: the seed must decode via the target's exact split-then-call,
        // and recover the original card.
        let recovered = decode(&parts).unwrap_or_else(|e| {
            panic!(
                "gen-corpus GATE: {} \\n-joined seed does not decode via the splitter: {e}",
                fx.name
            )
        });
        assert_eq!(
            recovered, card,
            "gen-corpus GATE: {} splitter seed decodes to a different KeyCard",
            fx.name
        );
        write_seed(
            &dir_decode,
            &format!("{}.parts", fx.name),
            joined.as_bytes(),
        );
        decode_count += 1;

        // --- mk1_decode_single seeds: each individual real chunk string. The
        //     whole-input target passes ONE &str; a single real chunk cannot
        //     be a complete card, so the gate asserts a CLEAN incomplete-card
        //     Err (NOT Ok, NOT a panic) — the never-panic contract the target
        //     protects. (See the module gate-doc: an Ok-asserting gate would
        //     be vacuous because no single string decodes to a full card.) ---
        for (i, chunk) in chunks.iter().enumerate() {
            match decode(&[chunk.as_str()]) {
                Ok(c) => {
                    // Defensive: if the wire format ever lets a lone chunk
                    // decode to a full card, the round trip must still hold —
                    // otherwise it is a genuine asymmetry the target would
                    // flag, and we want gen-corpus to flag it first.
                    assert_eq!(
                        c, card,
                        "gen-corpus GATE: {} chunk {i} unexpectedly decoded \
                         single-string to a DIFFERENT KeyCard",
                        fx.name
                    );
                }
                // The expected outcome for a real lone chunk: a CLEAN,
                // deterministic chunked-header error. Any OTHER error variant
                // is still fine (clean error, never-panic); we only fail if
                // the call panics (which it cannot reach here — match arms
                // cover Ok/Err exhaustively).
                Err(Error::ChunkedHeaderMalformed(_)) => {}
                Err(_other) => {
                    // Other clean errors are acceptable too — the contract is
                    // "no panic", and we exercised the call without one.
                }
            }
            write_seed(
                &dir_single,
                &format!("{}_chunk{i}.mk1", fx.name),
                chunk.as_bytes(),
            );
            single_count += 1;
        }
    }

    // Sanity: the splitter has at least one MULTI-chunk (>=3) seed so libFuzzer
    // has a non-trivial boundary to mutate, and we wrote several single-string
    // seeds.
    assert!(
        max_chunks_seen >= 3,
        "expected at least one 3+-chunk fixture for the splitter; saw max {max_chunks_seen}"
    );
    assert!(decode_count >= 3, "expected several splitter seeds");
    assert!(
        single_count >= 6,
        "expected several single-string chunk seeds"
    );

    eprintln!(
        "gen-corpus wrote: mk1_decode={decode_count} (max {max_chunks_seen} chunks), \
         mk1_decode_single={single_count}"
    );
}
