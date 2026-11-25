//! `mk encode` prints each mk1 string ONCE, on stdout (Brian 2026-11-22,
//! reversing D4 of mnemonic-engrave `design/SPEC_constellation_cli_uniformity.md`;
//! `design/FIX_mk_single_print.md`).
//!
//! The grouped stderr engraving card, and its `group size:` / `separator:`
//! lines, are gone. `--group-size N` groups STDOUT on request (default 0,
//! ungrouped), as `mt encode` and `mnemonic bundle` do, and `--out FILE`
//! receives exactly the bytes stdout would have carried. `--keys` with two or
//! more records labels each card on stderr by its line range, so a duplicate
//! record stays visible (F-311) without a second copy of any string.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

/// mk-cli's `V15_84_TEST` fixture: a testnet account xpub at m/84'/1'/0',
/// master fingerprint 73c5da0a. Public material only.
const TPUB: &str = "tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X";
const STUB: &str = "deadbeef";
const X0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const X1: &str = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk";
const NOTE: &str = "note: stdout is watch-only \u{2014} public keys only, cannot spend\n";

fn mk() -> Command {
    Command::cargo_bin("mk").expect("mk binary")
}

fn encode(extra: &[&str]) -> std::process::Output {
    let mut args = vec![
        "encode",
        "--xpub",
        TPUB,
        "--origin-fingerprint",
        "73c5da0a",
        "--origin-path",
        "m/84'/1'/0'",
        "--policy-id-stub",
        STUB,
    ];
    args.extend_from_slice(extra);
    mk().args(&args).output().expect("run mk")
}

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).expect("utf8")
}

/// Every 5 characters a space: the `--group-size 5` form of one line.
fn grouped5(s: &str) -> String {
    let c: Vec<char> = s.chars().collect();
    c.chunks(5)
        .map(|g| g.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn encode_prints_each_string_once() {
    let o = encode(&[]);
    assert_eq!(o.status.code(), Some(0));
    let out = text(&o.stdout);
    let err = text(&o.stderr);
    assert_eq!(out.lines().count(), 2, "two mk1 strings; got {out:?}");
    for l in out.lines() {
        assert!(
            l.starts_with("mk1") && !l.contains(' '),
            "ungrouped mk1; got {l:?}"
        );
    }
    assert_eq!(err, NOTE, "stderr carries only the watch-only note");
}

#[test]
fn encode_group_size_groups_stdout() {
    let plain = text(&encode(&[]).stdout);
    let o = encode(&["--group-size", "5"]);
    assert_eq!(o.status.code(), Some(0));
    let want: String = plain.lines().map(|l| grouped5(l) + "\n").collect();
    assert_eq!(text(&o.stdout), want, "stdout grouped by 5 with a space");
    assert_eq!(text(&o.stderr), NOTE);
}

#[test]
fn encode_out_writes_the_stdout_bytes() {
    let grouped = text(&encode(&["--group-size", "5"]).stdout);
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("card.mk1");
    let o = encode(&["--group-size", "5", "--out", f.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(text(&o.stdout), "", "stdout is empty under --out");
    assert_eq!(std::fs::read_to_string(&f).unwrap(), grouped);
    assert_eq!(text(&o.stderr), NOTE);
}

#[test]
fn encode_keys_labels_each_card_on_stderr() {
    // Two distinct records. The labels are the card boundary: cards of one key
    // file can share a chunk-set id, so the strings alone do not show it.
    let dir = tempfile::tempdir().unwrap();
    let kf = dir.path().join("keys.txt");
    std::fs::write(
        &kf,
        format!("[73c5da0a/48'/0'/0'/2']{X0}\n[73c5da0a/48'/0'/1'/2']{X1}\n"),
    )
    .unwrap();
    let o = mk()
        .args([
            "encode",
            "--keys",
            kf.to_str().unwrap(),
            "--policy-id-stub",
            STUB,
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let out = text(&o.stdout);
    assert_eq!(
        out.lines().count(),
        5,
        "two cards of two and three strings; got {out:?}"
    );
    assert_eq!(
        text(&o.stderr),
        format!("card 1 of 2: lines 1-2\ncard 2 of 2: lines 3-5\n{NOTE}")
    );
}
