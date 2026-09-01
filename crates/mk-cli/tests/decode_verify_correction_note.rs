//! Diagnostics followup `mk-decode-silent-correction-reporting`
//! (`design/FOLLOWUPS.md`): `mk decode` and `mk verify` apply BCH
//! error-correction to a damaged mk1 chunk silently -- the card comes back
//! looking pristine even though it consumed some of its `t = 4` per-chunk
//! correction budget. This file proves both verbs now emit a stderr note
//! naming the per-chunk correction count when correction fired, and stay
//! SILENT when it didn't -- non-fatal in both cases (exit code and stdout
//! unchanged, mirrors the R2 `chunk_set_id` warning's own placement --
//! `csid_verification.rs`).
//!
//! Fixture: the V1 KeyCard (same canonical fixture as `cli_repair.rs`),
//! minted via `mk_codec::encode` directly for self-contained
//! reproducibility (chunk 0 = long code, chunk 1 = regular code, per that
//! file's own comment). Chunk 0 gets 2 single-symbol flips, chunk 1 gets 1
//! -- each `flip_at` call is a single deterministic substitution, so 2 and 1
//! stay well inside the `t <= 4` correction radius and the whole set still
//! reassembles to the SAME card as the undamaged input.

use std::str::FromStr;

use assert_cmd::Command;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::KeyCard;
use mk_codec::string_layer::bch::ALPHABET;

const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const V1_PATH: &str = "m/48'/0'/0'/2'";

/// Encode a canonical V1 KeyCard and return all chunks (mirrors
/// `cli_repair.rs::generate_valid_mk1_chunks`).
fn generate_valid_mk1_chunks() -> Vec<String> {
    let xpub = Xpub::from_str(V1_XPUB).unwrap();
    let fp = Fingerprint::from([0xaa, 0xbb, 0xcc, 0xdd]);
    let path = DerivationPath::from_str(V1_PATH).unwrap();
    let stub = [0x11u8, 0x22, 0x33, 0x44];
    let card = KeyCard::new(vec![stub], Some(fp), path, xpub);
    mk_codec::encode(&card).expect("encode V1 KeyCard")
}

/// Flip the bech32 character at data-part position `pos` to the next
/// alphabet char (cyclic) -- a single deterministic substitution. Mirrors
/// `cli_repair.rs::flip_at`.
fn flip_at(chunk: &str, pos: usize) -> String {
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let was = chars[pos];
    let alphabet_str = std::str::from_utf8(ALPHABET).unwrap();
    let was_idx = alphabet_str.find(was).unwrap();
    let new_idx = (was_idx + 1) % 32;
    chars[pos] = alphabet_str.chars().nth(new_idx).unwrap();
    let mut out = String::from(prefix);
    for c in chars {
        out.push(c);
    }
    out
}

fn flip_many(chunk: &str, positions: &[usize]) -> String {
    positions
        .iter()
        .fold(chunk.to_string(), |acc, &p| flip_at(&acc, p))
}

/// Damaged-but-in-budget fixture: chunk 0 (long code) gets 2 flips, chunk 1
/// (regular code) gets 1. Positions are clear of the 8-symbol chunked
/// header and of each other. Returns `(damaged_chunks, clean_chunks)`.
fn damaged_and_clean_chunks() -> (Vec<String>, Vec<String>) {
    let clean = generate_valid_mk1_chunks();
    assert_eq!(
        clean.len(),
        2,
        "fixture must chunk into 2 or this test proves nothing"
    );
    let damaged0 = flip_many(&clean[0], &[20, 50]);
    let damaged1 = flip_at(&clean[1], 20);
    assert_ne!(damaged0, clean[0]);
    assert_ne!(damaged1, clean[1]);
    (vec![damaged0, damaged1], clean)
}

fn run(verb: &str, extra: &[&str], strings: &[String]) -> std::process::Output {
    let mut cmd = Command::cargo_bin("mk").unwrap();
    cmd.arg(verb);
    cmd.args(extra);
    for s in strings {
        cmd.arg(s);
    }
    cmd.output().expect("invoke mk")
}

#[test]
fn decode_damaged_input_warns_with_per_chunk_counts_same_card() {
    let (damaged, clean) = damaged_and_clean_chunks();

    let d = run("decode", &[], &damaged);
    assert_eq!(
        d.status.code(),
        Some(0),
        "a genuine in-budget correction must still succeed; stderr={}",
        String::from_utf8_lossy(&d.stderr)
    );
    let d_stderr = String::from_utf8_lossy(&d.stderr).to_string();
    assert!(
        d_stderr.contains("chunk 0: 2 correction(s)"),
        "stderr must name chunk 0's count; stderr={d_stderr:?}"
    );
    assert!(
        d_stderr.contains("chunk 1: 1 correction(s)"),
        "stderr must name chunk 1's count; stderr={d_stderr:?}"
    );
    assert!(
        d_stderr.contains("max 4 per chunk"),
        "stderr must name the t=4 ceiling so budget consumption is legible; stderr={d_stderr:?}"
    );
    assert!(
        d_stderr.contains("mk repair"),
        "stderr must point at `mk repair` for the per-position detail; stderr={d_stderr:?}"
    );

    let c = run("decode", &[], &clean);
    assert_eq!(c.status.code(), Some(0));
    let d_stdout = String::from_utf8(d.stdout).unwrap();
    let c_stdout = String::from_utf8(c.stdout).unwrap();
    assert_eq!(
        d_stdout, c_stdout,
        "damaged-but-corrected input must decode to the SAME card as clean input"
    );

    let c_stderr = String::from_utf8_lossy(&c.stderr).to_string();
    assert!(
        !c_stderr.contains("BCH error-correction repaired"),
        "undamaged input must stay silent; stderr={c_stderr:?}"
    );
}

#[test]
fn verify_damaged_input_warns_with_per_chunk_counts_exit_unchanged() {
    let (damaged, clean) = damaged_and_clean_chunks();

    let d = run("verify", &[], &damaged);
    let c = run("verify", &[], &clean);
    assert_eq!(
        d.status.code(),
        c.status.code(),
        "the note must not change verify's exit code"
    );
    assert_eq!(d.status.code(), Some(0));

    let d_stderr = String::from_utf8_lossy(&d.stderr).to_string();
    assert!(
        d_stderr.contains("chunk 0: 2 correction(s)"),
        "stderr={d_stderr:?}"
    );
    assert!(
        d_stderr.contains("chunk 1: 1 correction(s)"),
        "stderr={d_stderr:?}"
    );
    assert!(d_stderr.contains("max 4 per chunk"), "stderr={d_stderr:?}");
    assert!(d_stderr.contains("mk repair"), "stderr={d_stderr:?}");

    let c_stderr = String::from_utf8_lossy(&c.stderr).to_string();
    assert!(
        !c_stderr.contains("BCH error-correction repaired"),
        "clean input must stay silent; stderr={c_stderr:?}"
    );

    let d_stdout = String::from_utf8(d.stdout).unwrap();
    let c_stdout = String::from_utf8(c.stdout).unwrap();
    assert_eq!(
        d_stdout, c_stdout,
        "the note is stderr-only; verify's stdout verdict must be identical"
    );
}
