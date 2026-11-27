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

/// Final review M-2 ruling: a string whose chunk header does not parse
/// (N6, a reserved card type; N9) is no chunk of the format. Clean, it is 2
/// with nothing printed; corrected, it has no oracle and is printed at 22.
#[test]
fn repair_of_a_string_that_is_no_chunk_is_2_clean_and_22_corrected() {
    for name in ["N6_", "N9_"] {
        let n = vector(name);
        let o = mk(&["repair", &n[0]]);
        assert_eq!(o.status.code(), Some(2), "{name}");
        assert!(
            o.stdout.is_empty(),
            "{name}: {}",
            String::from_utf8_lossy(&o.stdout)
        );
        let err = String::from_utf8_lossy(&o.stderr);
        assert!(err.contains("is not an mk1 chunk"), "{name}: {err}");
    }
    let n6 = vector("N6_");
    let bad = flip(&n6[0], 20);
    let o = mk(&["repair", &bad]);
    assert_eq!(
        o.status.code(),
        Some(22),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(String::from_utf8_lossy(&o.stdout).contains(&n6[0]));
    let o = mk(&["repair", "--json", &bad]);
    assert_eq!(o.status.code(), Some(22));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(
        v["not_a_card"]["message"]
            .as_str()
            .unwrap()
            .contains("not an mk1 chunk")
    );
}

/// Final review M-4 (R11): every verb's help states the bits it can set.
#[test]
fn every_verb_help_has_an_exit_status_line() {
    for verb in [
        "encode",
        "decode",
        "verify",
        "inspect",
        "repair",
        "address",
        "derive",
        "vectors",
        "gui-schema",
        "gen-man",
    ] {
        let o = mk(&[verb, "--help"]);
        assert_eq!(o.status.code(), Some(0), "{verb}");
        let out = String::from_utf8_lossy(&o.stdout);
        assert!(out.contains("Exit status:"), "{verb}: {out}");
    }
    let out = String::from_utf8_lossy(&mk(&["repair", "--help"]).stdout).to_string();
    for code in ["16", "22", "2 ", "32", "64"] {
        assert!(out.contains(code), "repair help names {code}: {out}");
    }
}

/// Final review M-7: a key that is not an xpub is not echoed back (an xprv
/// given as --xpub must not reach stderr or a JSON envelope on stdout).
#[test]
fn an_xprv_given_as_xpub_is_not_echoed() {
    const XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";
    for json in [false, true] {
        let mut a = vec![
            "encode",
            "--xpub",
            XPRV,
            "--origin-path",
            "m/84'/0'/0'",
            "--policy-id-stub",
            "11223344",
            "--privacy-preserving",
        ];
        if json {
            a.push("--json");
        }
        let o = mk(&a);
        assert_eq!(o.status.code(), Some(2));
        let all = format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
        assert!(!all.contains(&XPRV[4..20]), "the key is echoed: {all}");
        assert!(all.contains("--xpub") && all.contains("0488ade4"), "{all}");
    }
    let d: serde_json::Value = serde_json::from_str(mk_codec::test_vectors::V0_1_JSON).unwrap();
    let v1: Vec<String> = d["vectors"][0]["expected"]["strings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    let o = mk(&["verify", "--xpub", XPRV, &v1[0], &v1[1]]);
    assert_eq!(o.status.code(), Some(2));
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(!all.contains(&XPRV[4..20]), "the key is echoed: {all}");
    assert!(
        all.contains("0488ade4"),
        "verify reached the --xpub check: {all}"
    );
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
    assert_eq!(o.status.code(), Some(64));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["error"]["exit_code"], 64);
    assert_eq!(v["error"]["kind"], "UsageError");
}

/// Decision 3: a required argument that is missing stays 64. No positional,
/// no `--in` and no `-` is the command line alone (test 1), refused before
/// anything is read (R2), on every verb that reads mk1 strings.
#[test]
fn no_input_channel_at_all_is_64() {
    for verb in [
        vec!["decode"],
        vec!["inspect"],
        vec!["verify"],
        vec!["repair"],
        vec!["address"],
        vec!["derive", "--path", "0/0"],
    ] {
        let o = mk(&verb);
        assert_eq!(
            o.status.code(),
            Some(64),
            "{verb:?}: stderr={}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
}

/// A named channel that yields nothing is an empty input: 2 (decision 3).
#[test]
fn a_named_channel_that_is_empty_is_2() {
    let o = Command::cargo_bin("mk")
        .unwrap()
        .args(["decode", "-"])
        .write_stdin("")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("empty.txt");
    std::fs::write(&f, "\n\n").unwrap();
    let o = mk(&["decode", "--in", f.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2));
}

/// I2 ruling: a string that is structurally a chunk (a chunked header, a
/// set of one) follows the complete-set rule: corrected and failing its
/// decode is 2 with nothing printed.
#[test]
fn repair_of_a_corrected_chunked_set_of_one_that_fails_is_2_and_prints_nothing() {
    use mk_codec::string_layer::bch::{bytes_to_5bit, encode_5bit_to_string};
    use mk_codec::string_layer::header::{StringLayerHeader, VERSION_V0_1};
    let header = StringLayerHeader::Chunked {
        version: VERSION_V0_1,
        chunk_set_id: 0x12345,
        total_chunks: 1,
        chunk_index: 0,
    };
    let mut data = header.to_5bit_symbols();
    data.extend(bytes_to_5bit(&[0xFF; 10]));
    let valid = encode_5bit_to_string(&data).unwrap();
    assert!(mk_codec::decode(&[valid.as_str()]).is_err());
    let bad = flip(&valid, 5);
    let o = mk(&["repair", &bad]);
    assert_eq!(
        o.status.code(),
        Some(2),
        "stderr={}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(o.stdout.is_empty());
}

fn flip(s: &str, pos: usize) -> String {
    const ALPHABET: &str = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let sep = s.rfind('1').unwrap();
    let mut c: Vec<char> = s.chars().collect();
    let i = sep + 1 + pos;
    let cur = c[i];
    c[i] = ALPHABET.chars().find(|&a| a != cur).unwrap();
    c.into_iter().collect()
}
