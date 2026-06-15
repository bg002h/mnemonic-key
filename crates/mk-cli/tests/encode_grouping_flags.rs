//! `mk encode` mstring display-grouping flags (P3). Default = space/5 print-once
//! (mk encode was UNBROKEN before — corrective). `--json` stays unbroken.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

const V1_XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";

fn encode(extra: &[&str]) -> std::process::Output {
    let mut cmd = Command::cargo_bin("mk").unwrap();
    cmd.args([
        "encode",
        "--xpub",
        V1_XPUB,
        "--origin-fingerprint",
        "aabbccdd",
        "--origin-path",
        "m/48'/0'/0'/2'",
        "--policy-id-stub",
        "11223344",
    ]);
    cmd.args(extra);
    cmd.output().unwrap()
}

fn first_line(out: &std::process::Output) -> String {
    String::from_utf8(out.stdout.clone())
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string()
}

#[test]
fn encode_default_groups_space_5() {
    let out = encode(&[]);
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let line = first_line(&out);
    assert_eq!(
        line.chars().nth(5),
        Some(' '),
        "default space/5; got {line:?}"
    );
    let unbroken: String = line.chars().filter(|c| *c != ' ').collect();
    assert!(
        unbroken.starts_with("mk1"),
        "space-stripped line starts with mk1; got {line:?}"
    );
}

#[test]
fn encode_unbroken_group_size_0() {
    let out = encode(&["--group-size", "0"]);
    assert!(out.status.success());
    let line = first_line(&out);
    assert!(
        !line.contains(' ') && !line.contains('-') && !line.contains(','),
        "unbroken; got {line:?}"
    );
    assert!(line.starts_with("mk1"));
}

#[test]
fn encode_separator_hyphen() {
    let out = encode(&["--separator", "hyphen"]);
    assert!(out.status.success());
    let line = first_line(&out);
    assert_eq!(
        line.chars().nth(5),
        Some('-'),
        "hyphen at idx 5; got {line:?}"
    );
}

#[test]
fn encode_rejects_bad_separator() {
    // mk maps clap parse errors to exit 64 (main.rs).
    let out = encode(&["--separator", "bogus"]);
    assert_eq!(out.status.code(), Some(64), "bad separator → exit 64");
}
