//! The invalid-artifact exit code: 2 → 1, and `repair` keeps its 2 (SPEC §6f).
//!
//! **This is the only exit code the uniformity cycle changes, and it is
//! funds-adjacent, so both directions are asserted here rather than one.**
//!
//! §6f's table gives `md`, `ms` and `mnemonic` an invalid-artifact of **1** and a
//! repair-uncorrectable of **2**. `mk` produced **2** for both, out of a single
//! `CliError::Codec(_) | CliError::MdCodec(_)` arm — so the ruling as written
//! ("mk's invalid-artifact 2 becomes 1") would also have moved `repair`'s code
//! and broken a parity the other three CLIs hold (F-291).
//!
//! **`md` splits by VERB, not by error kind**, and that is what is copied:
//! `md repair` returns `Ok(2)` from a bare `Err(e) =>` arm covering *any* codec
//! error out of the correcting decode, while `md decode` is 1. A bypass narrowed
//! to the BCH-uncorrectable variant would send an HRP-swapped card on `repair`
//! to 1 — which is why the HRP case is asserted below and not only the
//! uncorrectable one. The two candidate implementations are indistinguishable
//! without it.
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
// The move: invalid artifact 2 → 1.
// ──────────────────────────────────────────────────────────────────────────

/// Every non-`repair` verb reports an invalid artifact as **1**.
#[test]
fn an_invalid_artifact_exits_1_on_the_reading_verbs() {
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
            1,
            "mk {verb} <{label}> must exit 1 (SPEC §6f invalid artifact)"
        );
    }
}

/// `mk encode --from-md1 <not an md1>` goes through `CliError::MdCodec`, the
/// other half of the arm that moved, and must land on 1 too.
#[test]
fn an_invalid_md1_binding_exits_1() {
    assert_eq!(
        code_of(&[
            "encode",
            "--xpub",
            V1_XPUB,
            "--origin-path",
            V1_PATH,
            "--from-md1",
            "md1notavalidcard",
        ]),
        1,
        "an md1 the binding cannot read is an invalid artifact, not a usage error"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// What must NOT move: repair's 2, on BOTH shapes.
// ──────────────────────────────────────────────────────────────────────────

/// **THE ASSERTION THAT SEPARATES THE TWO CANDIDATE IMPLEMENTATIONS.**
///
/// The same string, on two verbs: `repair` is 2 and everything else is 1. That
/// is `md`'s split, and it is a split by VERB. A bypass scoped to the
/// BCH-uncorrectable variant passes the `uncorrectable` row and fails the
/// `HRP-swapped` one, because an HRP mismatch is an invalid artifact that
/// happens to arrive at `repair`.
#[test]
fn repair_is_2_where_the_other_verbs_are_1_on_the_same_input() {
    for (artifact, label) in [
        (uncorrectable(), "uncorrectable"),
        (hrp_swapped(), "HRP-swapped"),
    ] {
        assert_eq!(
            code_of(&["repair", &artifact]),
            2,
            "mk repair <{label}> must STAY 2 (cross-CLI parity with md/ms/mnemonic)"
        );
        assert_eq!(
            code_of(&["decode", &artifact]),
            1,
            "mk decode <{label}> must be 1 — same string, different verb"
        );
    }
}

/// The control that says the bypass did not swallow the SUCCESS paths: a valid
/// card still repairs at 0, and a single correctable substitution still exits 5.
#[test]
fn repair_success_codes_are_untouched() {
    let chunks = valid_chunks();
    assert_eq!(
        code_of(&["repair", &chunks[1]]),
        0,
        "an already-valid card still exits 0"
    );
    assert_eq!(
        code_of(&["repair", &flip_at(&chunks[1], 5)]),
        5,
        "one correctable substitution still exits 5 (REPAIR_APPLIED)"
    );
}

/// The clap split is untouched: a usage error is still 64, not 1 or 2.
///
/// §6f records the 2-versus-64 disagreement across the constellation and
/// declines to resolve it; this phase must not resolve it by accident while
/// moving a neighbouring code.
#[test]
fn the_usage_error_code_is_untouched() {
    assert_eq!(
        code_of(&["decode"]),
        64,
        "no artifact at all is a usage error"
    );
    assert_eq!(
        code_of(&["encode", "--xpub", V1_XPUB]),
        64,
        "a missing required flag is still clap's 64"
    );
}

/// **SPEC §6b: `--json` is UNCHANGED and out of scope this cycle** — and the
/// bypass is exactly where it could have been changed by accident.
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
/// returns, not the 1 the `CliError` now maps to — otherwise a `--json` consumer
/// reads one number and its shell reads another.
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
