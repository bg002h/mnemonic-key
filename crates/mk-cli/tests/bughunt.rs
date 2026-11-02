//! Bug-hunt triage rows (mk-cli). Each test asserts the CORRECT behaviour.

use std::process::{Command, Output};

use assert_cmd::cargo::CommandCargoExt;

const V1_XPUB: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";

const KEYED_POLICY_A: [&str; 4] = [
    "md1fj5r4pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapguu6w2y2jygprc",
    "md1fj5r4psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wnj3gvx34m0wnt",
    "md1fj5r4pshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuyz646z0r98kyg",
    "md1fj5r4pslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqrfrshqj9459eg",
];

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

/// Hunt #1. SPEC §6: "Encoders SHOULD expose this [omitting the fingerprint]
/// as an explicit choice rather than a default." With neither
/// `--origin-fingerprint` nor `--privacy-preserving`, the card must not be
/// minted silently. (Open product question: exact message / exit code; only
/// the absence of a silent success is asserted.)
#[test]
fn bughunt_mk_01_encode_without_fingerprint_flag_not_silent() {
    let o = mk()
        .args([
            "encode",
            "--xpub",
            V1_XPUB,
            "--origin-path",
            "m/48'/0'/0'/2'",
            "--policy-id-stub",
            "deadbeef",
        ])
        .output()
        .unwrap();
    assert!(
        !o.status.success(),
        "minted a fingerprint-less card with no fingerprint flag and no \
         --privacy-preserving: {}",
        desc(&o)
    );
}

/// Hunt #3. `--count 2147483648` must be a fast usage error (exit 64), not an
/// 8 GiB allocation. The child runs under `ulimit -v` (2 GiB) so an unfixed
/// binary aborts (no exit code) instead of hogging memory. The cap value is
/// an open product question; only "this request is refused" is asserted.
#[test]
fn bughunt_mk_03_address_huge_count_is_usage_error() {
    use mk_codec::KeyCard;
    use std::str::FromStr;
    let kc = KeyCard::new(
        vec![[0xde, 0xad, 0xbe, 0xef]],
        Some(bitcoin::bip32::Fingerprint::from([0x73, 0xc5, 0xda, 0x0a])),
        bitcoin::bip32::DerivationPath::from_str("m/84'/0'/0'").unwrap(),
        bitcoin::bip32::Xpub::from_str(V2_84_MAIN).unwrap(),
    );
    let strings = mk_codec::encode(&kc).unwrap();
    let bin = std::path::PathBuf::from(env!("CARGO_BIN_EXE_mk"));
    for flags in [
        vec!["--count", "2147483648"],
        vec!["--range", "0,2147483647"],
    ] {
        let o = Command::new("timeout")
            .arg("60")
            .arg("sh")
            .arg("-c")
            .arg("ulimit -v 2097152; exec \"$0\" \"$@\"")
            .arg(&bin)
            .arg("address")
            .args(&strings)
            .args(&flags)
            .output()
            .unwrap();
        assert_eq!(
            o.status.code(),
            Some(64),
            "{flags:?} not refused with a usage error: {}",
            desc(&o).chars().take(400).collect::<String>()
        );
    }
}

/// Hunt #4. `--from-md1-set` documents that every line that is not an md1
/// string is skipped; a line starting with non-ASCII characters must be
/// skipped, not panic.
#[test]
fn bughunt_mk_04_from_md1_set_non_ascii_line_skipped() {
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("set.txt");
    let mut body = String::from("\u{e9}\u{e9} note\n");
    for c in KEYED_POLICY_A {
        body.push_str(c);
        body.push('\n');
    }
    std::fs::write(&f, body).unwrap();
    let o = mk()
        .args([
            "encode",
            "--xpub",
            V1_XPUB,
            "--origin-fingerprint",
            "73c5da0a",
            "--origin-path",
            "m/48'/0'/0'/2'",
            "--from-md1-set",
            f.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", desc(&o));
}
