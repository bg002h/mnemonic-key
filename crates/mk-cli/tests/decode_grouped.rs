//! `mk decode` (and the shared `read_mk1_strings` intake) accepts a comma-grouped
//! mk1 card (mstring-grouping P3). mk-codec's decode tolerates NO separators and
//! the legacy `read_mk1_strings` only `.trim()`med edges, so a comma-grouped card
//! genuinely exercises the new `strip_display_separators` intake.

use std::process::Command;
use std::str::FromStr;

use assert_cmd::cargo::CommandCargoExt;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::KeyCard;

const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const V1_PATH: &str = "m/48'/0'/0'/2'";

fn comma5(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && i % 5 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Build the unbroken mk1 chunk strings for the V1 card via the codec (not the CLI).
fn unbroken_mk1() -> Vec<String> {
    let xpub = Xpub::from_str(V1_XPUB).unwrap();
    let fp = Fingerprint::from([0xaa, 0xbb, 0xcc, 0xdd]);
    let path = DerivationPath::from_str(V1_PATH).unwrap();
    let card = KeyCard::new(vec![[0x11u8, 0x22, 0x33, 0x44]], Some(fp), path, xpub);
    mk_codec::encode(&card).expect("encode")
}

fn decode_output(chunks: &[String]) -> std::process::Output {
    let mut cmd = Command::cargo_bin("mk").unwrap();
    cmd.arg("decode");
    for c in chunks {
        cmd.arg(c);
    }
    cmd.output().unwrap()
}

#[test]
fn decode_accepts_comma_grouped() {
    let chunks = unbroken_mk1();
    let grouped: Vec<String> = chunks.iter().map(|s| comma5(s)).collect();

    let plain = decode_output(&chunks);
    assert!(plain.status.success(), "unbroken decode should succeed");

    let g = decode_output(&grouped);
    assert!(
        g.status.success(),
        "comma-grouped decode must succeed; stderr={}",
        String::from_utf8_lossy(&g.stderr)
    );
    assert_eq!(
        g.stdout, plain.stdout,
        "grouped decode stdout must equal unbroken decode stdout"
    );
}
