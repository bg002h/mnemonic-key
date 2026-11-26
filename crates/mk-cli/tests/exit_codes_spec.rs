//! Behavioural rows for the constellation exit-code spec (mnemonic-engrave
//! `design/SPEC_constellation_exit_codes.md` §3, §7b) that no older test file
//! pins: `repair`'s decode check (decision 6), input-file I/O (32), and the
//! usage-before-input order (R2).

use assert_cmd::Command;

fn vector(name: &str) -> Vec<String> {
    let d: serde_json::Value = serde_json::from_str(mk_codec::test_vectors::V0_1_JSON).unwrap();
    let v = d["vectors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"].as_str().unwrap().starts_with(name))
        .unwrap_or_else(|| panic!("no vector {name}"));
    v["input"]["strings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

fn mk(args: &[&str]) -> std::process::Output {
    Command::cargo_bin("mk")
        .unwrap()
        .args(args)
        .output()
        .unwrap()
}

/// Decision 6: `mk repair` used to exit 0 on a clean, complete set that
/// `mk decode` rejects (N12, an unsupported version). A complete set that
/// does not decode is now 2, with nothing printed, like `decode`.
#[test]
fn repair_of_a_clean_set_decode_rejects_is_2_and_prints_nothing() {
    let n12 = vector("N12_");
    let mut a = vec!["repair"];
    a.extend(n12.iter().map(String::as_str));
    let o = mk(&a);
    assert_eq!(o.status.code(), Some(2));
    assert!(
        o.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&o.stdout)
    );
    let mut d = vec!["decode"];
    d.extend(n12.iter().map(String::as_str));
    assert_eq!(mk(&d).status.code(), Some(2), "decode agrees");
}

/// Decision 6: a string that is no mk1 chunk at all (N6, a reserved card
/// type in its header) is still printed, with an error line, and exits 2.
#[test]
fn repair_prints_a_string_that_is_no_chunk_and_sets_2() {
    let n6 = vector("N6_");
    let o = mk(&["repair", &n6[0]]);
    assert_eq!(o.status.code(), Some(2));
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(out.contains(&n6[0]), "{out}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("is not an mk1 chunk"), "{err}");
}

/// An input file that cannot be read is the system's failure: 32 (was 64).
#[test]
fn a_missing_input_file_is_32() {
    let o = mk(&["decode", "--in", "/nonexistent-mk-exit-spec/x.txt"]);
    assert_eq!(o.status.code(), Some(32));
    let o = mk(&[
        "encode",
        "--keys",
        "/nonexistent-mk-exit-spec/keys.txt",
        "--policy-id-stub",
        "11223344",
    ]);
    assert_eq!(o.status.code(), Some(32));
}

/// Text that is not UTF-8 is an invalid input (2), not an I/O failure.
#[test]
fn a_non_utf8_input_file_is_2() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("bad.txt");
    std::fs::write(&p, b"mk1\xff\xfe\n").unwrap();
    let o = mk(&["decode", "--in", p.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2));
}

/// R2: a bad setting is 64 before any input is read, even with no input.
#[test]
fn a_bad_setting_is_64_before_the_input_is_read() {
    assert_eq!(mk(&["derive", "--path", "m/0'"]).status.code(), Some(64));
    assert_eq!(mk(&["address", "--range", "5,2"]).status.code(), Some(64));
    // encode: a missing fingerprint choice is 64 before --from-md1 is read.
    let o = mk(&[
        "encode",
        "--xpub",
        "xpub",
        "--origin-path",
        "m/84'/0'/0'",
        "--from-md1",
        "md1notacard",
    ]);
    assert_eq!(o.status.code(), Some(64));
}

/// The JSON error envelope carries the exit status (R14) and names the
/// class in `kind`.
#[test]
fn json_envelope_carries_the_class() {
    let o = mk(&["decode", "--json", "--in", "/nonexistent-mk-exit-spec/x"]);
    assert_eq!(o.status.code(), Some(32));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["error"]["exit_code"], 32);
    assert_eq!(v["error"]["kind"], "IoError");
    let o = mk(&["decode", "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["error"]["exit_code"], 2);
    assert_eq!(v["error"]["kind"], "InvalidInput");
}
