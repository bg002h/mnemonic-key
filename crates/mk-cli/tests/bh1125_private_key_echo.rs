//! Bug hunt 2026-11-25, mk#4 — a mis-pasted extended PRIVATE key was echoed
//! in full in the error text (and inside the `--json` error envelope).
//!
//! An xprv/tprv (or a SLIP-0132 private form) where an xpub belongs is now
//! refused with a fixed message that names the mistake and never quotes the
//! value, on every route (`--xpub`, a key file, `mk verify --xpub`). Other
//! unparsable key text is no longer quoted in full either.

use std::process::{Command, Output};

use assert_cmd::cargo::CommandCargoExt;
use bitcoin::base58;

/// BIP-32 test vector 1 master key (public test data, not a real key).
const XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";
const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";

fn mk() -> Command {
    Command::cargo_bin("mk").expect("mk binary")
}

fn desc(o: &Output) -> String {
    format!(
        "code={:?} stdout={:?} stderr={:?}",
        o.status.code(),
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// Re-version a base58check extended key (same key material).
fn reversion(key: &str, ver: [u8; 4]) -> String {
    let mut data = base58::decode_check(key).unwrap();
    data[..4].copy_from_slice(&ver);
    base58::encode_check(&data)
}

/// No 12-character window of the key's body (past its 4-char prefix)
/// appears anywhere in the output.
fn assert_not_echoed(o: &Output, key: &str) {
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    let body: Vec<char> = key.chars().skip(4).collect();
    for w in body.windows(12) {
        let w: String = w.iter().collect();
        assert!(!all.contains(&w), "key text {w:?} echoed: {}", desc(o));
    }
}

fn encode_xpub(key: &str, json: bool) -> Output {
    let mut args = vec![
        "encode",
        "--xpub",
        key,
        "--origin-path",
        "m",
        "--origin-fingerprint",
        "3442193e",
        "--policy-id-stub",
        "aabbccdd",
    ];
    if json {
        args.push("--json");
    }
    mk().args(args).output().unwrap()
}

#[test]
fn encode_xpub_with_xprv_is_refused_without_echo() {
    for json in [false, true] {
        let o = encode_xpub(XPRV, json);
        assert_eq!(o.status.code(), Some(64), "{}", desc(&o));
        assert_not_echoed(&o, XPRV);
        let all = format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        assert!(all.contains("extended PRIVATE key"), "{}", desc(&o));
    }
}

#[test]
fn every_private_version_is_refused_without_echo() {
    for ver in [
        [0x04, 0x35, 0x83, 0x94], // tprv
        [0x04, 0x9D, 0x78, 0x78], // yprv
        [0x04, 0xB2, 0x43, 0x0C], // zprv
        [0x02, 0x95, 0xB0, 0x05], // Yprv
        [0x02, 0xAA, 0x7A, 0x99], // Zprv
        [0x04, 0x4A, 0x4E, 0x28], // uprv
        [0x04, 0x5F, 0x18, 0xBC], // vprv
        [0x02, 0x42, 0x85, 0xB5], // Uprv
        [0x02, 0x57, 0x50, 0x48], // Vprv
    ] {
        let key = reversion(XPRV, ver);
        let o = encode_xpub(&key, false);
        assert_eq!(o.status.code(), Some(64), "{key}: {}", desc(&o));
        assert_not_echoed(&o, &key);
        assert!(
            String::from_utf8_lossy(&o.stderr).contains("extended PRIVATE key"),
            "{key}: {}",
            desc(&o)
        );
    }
}

/// A mistyped xprv (its checksum broken) is still recognised by its prefix.
#[test]
fn mistyped_xprv_is_refused_without_echo() {
    let mut c: Vec<char> = XPRV.chars().collect();
    c[40] = if c[40] == 'a' { 'b' } else { 'a' };
    let key: String = c.into_iter().collect();
    let o = encode_xpub(&key, false);
    assert_eq!(o.status.code(), Some(64), "{}", desc(&o));
    assert_not_echoed(&o, &key);
    assert!(String::from_utf8_lossy(&o.stderr).contains("extended PRIVATE key"));
}

#[test]
fn key_file_with_xprv_is_refused_without_echo() {
    let dir = tempfile::tempdir().unwrap();
    for (name, line) in [
        ("origin.txt", format!("[3442193e/]{XPRV}\n")),
        ("path.txt", format!("[3442193e/84'/0'/0']{XPRV}\n")),
        ("bare.txt", format!("{XPRV}\n")),
        ("suffix.txt", format!("[3442193e/84'/0'/0']{XPRV}/0/*\n")),
        ("nopath.txt", format!("[3442193e]{XPRV}\n")),
    ] {
        let p = dir.path().join(name);
        std::fs::write(&p, line).unwrap();
        for json in [false, true] {
            let mut args = vec![
                "encode",
                "--in",
                p.to_str().unwrap(),
                "--policy-id-stub",
                "aabbccdd",
            ];
            if json {
                args.push("--json");
            }
            let o = mk().args(args).output().unwrap();
            assert_eq!(o.status.code(), Some(64), "{name}: {}", desc(&o));
            assert_not_echoed(&o, XPRV);
        }
    }
}

#[test]
fn verify_xpub_with_xprv_is_refused_without_echo() {
    let card = mk()
        .args([
            "encode",
            "--xpub",
            V2_84_MAIN,
            "--origin-path",
            "m/84'/0'/0'",
            "--origin-fingerprint",
            "73c5da0a",
            "--policy-id-stub",
            "aabbccdd",
        ])
        .output()
        .unwrap();
    assert_eq!(card.status.code(), Some(0), "{}", desc(&card));
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("card.txt");
    std::fs::write(&p, &card.stdout).unwrap();
    for json in [false, true] {
        let mut args = vec!["verify", "--in", p.to_str().unwrap(), "--xpub", XPRV];
        if json {
            args.push("--json");
        }
        let o = mk().args(args).output().unwrap();
        assert_eq!(o.status.code(), Some(64), "{}", desc(&o));
        assert_not_echoed(&o, XPRV);
    }
}

/// Other unparsable key text: still refused (exit 64, "invalid xpub"), and
/// not quoted in full.
#[test]
fn other_invalid_key_text_is_not_quoted_in_full() {
    // A WIF private key (BIP-32 TV1-unrelated, well-known test WIF).
    let wif = "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ";
    let o = encode_xpub(wif, false);
    assert_eq!(o.status.code(), Some(64), "{}", desc(&o));
    assert!(String::from_utf8_lossy(&o.stderr).contains("invalid xpub"));
    assert_not_echoed(&o, wif);
}

/// A good xpub still encodes.
#[test]
fn a_public_key_still_encodes() {
    let o = mk()
        .args([
            "encode",
            "--xpub",
            V2_84_MAIN,
            "--origin-path",
            "m/84'/0'/0'",
            "--origin-fingerprint",
            "73c5da0a",
            "--policy-id-stub",
            "aabbccdd",
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", desc(&o));
}

/// Review r1 I1: a key-file record is never quoted back, whatever it holds.
/// A WIF line, a BIP-39 phrase line, a `[fp/path]WIF/0/*` line and a
/// `[WIF]xpub` line each get a shape error naming the file and line only,
/// in text and in `--json`.
#[test]
fn key_file_record_is_never_quoted() {
    let wif = "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ";
    let phrase = "legal winner thank year wave sausage worth useful legal winner thank yellow";
    let dir = tempfile::tempdir().unwrap();
    for (name, line, secret) in [
        ("wif.txt", wif.to_string(), wif),
        ("phrase.txt", phrase.to_string(), phrase),
        ("suffix.txt", format!("[3442193e/84'/0'/0']{wif}/0/*"), wif),
        ("origin.txt", format!("[{wif}]{V2_84_MAIN}"), wif),
        ("empty.txt", format!("[3442193e/{wif}]"), wif),
    ] {
        let p = dir.path().join(name);
        std::fs::write(&p, format!("{line}\n")).unwrap();
        for json in [false, true] {
            let mut args = vec![
                "encode",
                "--in",
                p.to_str().unwrap(),
                "--policy-id-stub",
                "aabbccdd",
            ];
            if json {
                args.push("--json");
            }
            let o = mk().args(args).output().unwrap();
            assert_eq!(o.status.code(), Some(64), "{name}: {}", desc(&o));
            let all = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            );
            assert!(all.contains(":1:"), "{name}: names the line: {}", desc(&o));
            if secret == phrase {
                for w in ["legal winner", "sausage", "yellow"] {
                    assert!(!all.contains(w), "{name}: phrase echoed: {}", desc(&o));
                }
            } else {
                assert_not_echoed(&o, secret);
            }
        }
    }
}
