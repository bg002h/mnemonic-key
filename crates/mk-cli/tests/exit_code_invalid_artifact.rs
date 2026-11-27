//! The invalid-artifact exit code is 2 on every verb, `repair` included
//! (constellation exit-code spec §3: INVALID).
//!
//! History: the uniformity cycle (SPEC §6f) moved `mk`'s invalid artifact
//! from 2 to 1 and kept `repair`'s 2 through a bypass, splitting by VERB as
//! `md` did. The exit-code spec makes an invalid input 2 everywhere, so the
//! split is gone: the same string is 2 on `decode` and on `repair`. The HRP
//! case is still asserted beside the uncorrectable one, so a later narrowing
//! of `repair`'s atomic-failure arm to the BCH variant alone would be caught.
//!
//! **What must NOT move:** `CliError::SetReassemblyMismatch` stays **2**. It is
//! the miscorrection rejection — the funds fix — and it is pinned by four tests
//! in `cli_mk1_repair_reverify.rs` plus the table test in `src/error.rs`.

use std::str::FromStr;

use assert_cmd::cargo::CommandCargoExt;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::KeyCard;
use mk_codec::string_layer::bch::ALPHABET;
use std::process::Command;

const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const V1_PATH: &str = "m/48'/0'/0'/2'";

fn valid_chunks() -> Vec<String> {
    let xpub = Xpub::from_str(V1_XPUB).unwrap();
    let fp = Fingerprint::from([0xaa, 0xbb, 0xcc, 0xdd]);
    let path = DerivationPath::from_str(V1_PATH).unwrap();
    let card = KeyCard::new(vec![[0x11u8, 0x22, 0x33, 0x44]], Some(fp), path, xpub);
    mk_codec::encode(&card).expect("encode V1 KeyCard")
}

/// Flip the bech32 char at `pos` in the data part (mirrors `cli_repair.rs`).
fn flip_at(chunk: &str, pos: usize) -> String {
    let sep = chunk.rfind('1').unwrap();
    let (prefix, rest) = chunk.split_at(sep + 1);
    let mut chars: Vec<char> = rest.chars().collect();
    let alphabet = std::str::from_utf8(ALPHABET).unwrap();
    let idx = alphabet.find(chars[pos]).unwrap();
    chars[pos] = alphabet.chars().nth((idx + 1) % 32).unwrap();
    let mut out = String::from(prefix);
    out.extend(chars);
    out
}

/// Five substitutions exceed the t=4 correction capacity.
fn uncorrectable() -> String {
    let chunks = valid_chunks();
    [3usize, 11, 19, 27, 35]
        .iter()
        .fold(chunks[1].clone(), |acc, &p| flip_at(&acc, p))
}

/// An mk1 whose HRP is swapped to `ms1`: the data part stays intact and
/// parseable, and the HRP-bound polymod fires `InvalidHrp`. **This is an
/// INVALID ARTIFACT, not an uncorrectable one** — the case that decides how wide
/// `repair`'s bypass has to be.
fn hrp_swapped() -> String {
    valid_chunks()[1].replacen("mk1", "ms1", 1)
}

fn code_of(args: &[&str]) -> i32 {
    let out = Command::cargo_bin("mk")
        .expect("mk binary")
        .args(args)
        .output()
        .expect("invoke mk");
    out.status.code().expect("exited normally")
}

// ──────────────────────────────────────────────────────────────────────────
// An invalid artifact is 2 (INVALID).
// ──────────────────────────────────────────────────────────────────────────

/// Every reading verb reports an invalid artifact as **2**.
#[test]
fn an_invalid_artifact_exits_2_on_the_reading_verbs() {
    let bad = uncorrectable();
    let hrp = hrp_swapped();
    for (verb, artifact, label) in [
        ("decode", bad.as_str(), "uncorrectable"),
        ("decode", hrp.as_str(), "HRP-swapped"),
        ("decode", "notanmk1card", "garbage"),
        ("inspect", bad.as_str(), "uncorrectable"),
        ("inspect", hrp.as_str(), "HRP-swapped"),
        ("verify", bad.as_str(), "uncorrectable"),
        ("verify", hrp.as_str(), "HRP-swapped"),
    ] {
        assert_eq!(
            code_of(&[verb, artifact]),
            2,
            "mk {verb} <{label}> must exit 2 (exit-code spec: INVALID)"
        );
    }
}

/// `mk encode --from-md1 <not an md1>` goes through `CliError::MdCodec`, the
/// other half of the arm, and must land on 2 too.
#[test]
fn an_invalid_md1_binding_exits_2() {
    assert_eq!(
        code_of(&[
            "encode",
            "--xpub",
            V1_XPUB,
            "--origin-path",
            V1_PATH,
            "--privacy-preserving",
            "--from-md1",
            "md1notavalidcard",
        ]),
        2,
        "an md1 the binding cannot read is an invalid artifact, not a usage error"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// repair's 2, on BOTH shapes, is the same 2 the other verbs give.
// ──────────────────────────────────────────────────────────────────────────

/// The same string, on two verbs: both are 2. An HRP mismatch is an invalid
/// artifact that happens to arrive at `repair`; it must not be read as
/// anything else there.
#[test]
fn repair_and_decode_agree_on_the_same_invalid_input() {
    for (artifact, label) in [
        (uncorrectable(), "uncorrectable"),
        (hrp_swapped(), "HRP-swapped"),
    ] {
        assert_eq!(
            code_of(&["repair", &artifact]),
            2,
            "mk repair <{label}> must be 2 (cross-CLI parity with md/ms/mnemonic)"
        );
        assert_eq!(
            code_of(&["decode", &artifact]),
            2,
            "mk decode <{label}> must be 2 — same string, different verb"
        );
    }
}

/// The control that says the bypass did not swallow the SUCCESS paths: a valid
/// card still repairs at 0, and a single correctable substitution exits 16.
#[test]
fn repair_success_codes() {
    let chunks = valid_chunks();
    assert_eq!(
        code_of(&["repair", &chunks[1]]),
        0,
        "an already-valid card still exits 0"
    );
    assert_eq!(
        code_of(&["repair", &flip_at(&chunks[1], 5)]),
        16,
        "one correctable substitution exits 16 (REPAIRED)"
    );
}

/// A usage error is 64 (exit-code spec decision 2); naming no input channel
/// at all is a missing required argument, still 64 (decision 3).
#[test]
fn usage_is_64_and_no_input_is_64() {
    assert_eq!(
        code_of(&["decode"]),
        64,
        "no input channel named at all is a missing argument, USAGE"
    );
    assert_eq!(
        code_of(&["encode", "--xpub", V1_XPUB]),
        64,
        "a missing required flag is still clap's 64"
    );
}

/// **SPEC §6b: `--json` is UNCHANGED** — and `repair`'s atomic-failure
/// branch is exactly where it could have been changed by accident.
///
/// A bare `eprintln!` + `return Ok(2)` deletes this envelope silently, and no
/// exit-code assertion notices. Measured before the change, this stdout line was
/// byte-for-byte what is asserted below.
///
/// **`md repair` — the shape being transplanted — DOES have `--json`, and it
/// drops its envelope on exactly this path.** Measured:
/// `md repair --json <a card the correcting decode rejects>` exits 2 with an
/// EMPTY stdout and `md: repair: …` on stderr. So copying `md`'s bypass verbatim
/// would have made `mk` match `md` by removing behaviour `mk` already had, in a
/// cycle whose §6b says `--json` is unchanged. Filed against `md`, not fixed
/// here.
///
/// `exit_code` inside the envelope must be **2**, the code the process actually
/// returns (spec R14) — otherwise a `--json` consumer reads one number and its
/// shell reads another.
#[test]
fn repair_json_error_envelope_survives_the_bypass() {
    let out = Command::cargo_bin("mk")
        .expect("mk binary")
        .args(["repair", "--json", &uncorrectable()])
        .output()
        .expect("invoke mk");
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("--json must still emit an envelope on stdout ({e}); got {stdout:?}")
    });
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["error"]["kind"], "BchUncorrectable");
    assert_eq!(
        v["error"]["exit_code"], 2,
        "the envelope must report the code the process exits with"
    );
    assert!(
        out.stderr.is_empty(),
        "--json keeps the report on ONE stream; got {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
}
