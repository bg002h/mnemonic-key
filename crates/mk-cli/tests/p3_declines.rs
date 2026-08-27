//! **P3's declines, asserted** — the `mk` half of the plan's last row.
//!
//! No code builds these. They pin what P3 ruled `mk` must KEEP, so that a later
//! phase cannot delete any of it as tidying, and so that an over-eager adoption
//! of the shared IO crate goes red here rather than being discovered on a plate.
//!
//! Each assertion names the ruling it comes from. P3's boundary table declines
//! ten of the crate's eleven items for `mk`; three of those declines are
//! observable from outside the binary and are the ones below.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

const XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const PATH: &str = "m/48'/0'/0'/2'";
const STUB: &str = "11223344";

fn mk() -> Command {
    Command::cargo_bin("mk").expect("mk binary")
}

fn encode() -> std::process::Output {
    mk().args([
        "encode",
        "--xpub",
        XPUB,
        "--origin-fingerprint",
        "aabbccdd",
        "--origin-path",
        PATH,
        "--policy-id-stub",
        STUB,
    ])
    .output()
    .unwrap()
}

/// **`exit::write_block` / `channel::destination` are DECLINED (§6e).**
///
/// §6e retracted the generalisation of `me`'s write gate in as many words: "the
/// terminal gate stays scoped to `me`'s binary container", justified by
/// binary-in-a-scrollback and by nothing else. `mk1` strings are short printable
/// ASCII a human must READ in order to engrave them.
///
/// So `mk encode` writing to an ordinary, world-readable destination must still
/// succeed. An adoption of `write_block` that imported `me`'s
/// `Destination::Terminal` refusal — or `me`'s mode-0644 refusal, which exits 2
/// there — reds here.
#[test]
fn mk_encode_does_not_refuse_a_world_readable_destination() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("scrollback.txt");
        std::fs::write(&p, "").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();

        let f = std::fs::OpenOptions::new().write(true).open(&p).unwrap();
        let out = mk()
            .args([
                "encode",
                "--xpub",
                XPUB,
                "--origin-fingerprint",
                "aabbccdd",
                "--origin-path",
                PATH,
                "--policy-id-stub",
                STUB,
            ])
            .stdout(f)
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "mk has no write gate on stdout (SPEC §6e); stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(std::fs::read_to_string(&p).unwrap().starts_with("mk1"));
    }
}

/// **The argv refusal is DECLINED for `mk` (§4).**
///
/// §4 exempts watch-only material by name and quotes `mt`'s own shipped refusal
/// doing so: "md and mk DO take their strings as arguments; md1/mk1 are
/// watch-only, so a leak there costs privacy rather than the money."
///
/// Adding one is a ruling, and P3 does not make it. So an xpub on argv still
/// mints, and nothing warns about argv.
#[test]
fn mk_still_takes_watch_only_material_on_argv_without_refusing() {
    let out = encode();
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("on argv"),
        "mk has no argv refusal or advisory (SPEC §4); got {stderr:?}"
    );
}

/// **`mk decode`'s stdout shape is out of scope BY NAME (§6a).**
///
/// §6a scopes the stdout rule to `encode` and says of `decode`: "explicitly out
/// of scope, and named so rather than left ambiguous… a breaking change to a
/// machine-readable surface with no funds-safety argument behind it". The
/// labelled five-field table scripts read today is still there, unchanged by the
/// entries that reshaped `encode`.
#[test]
fn mk_decode_still_emits_its_five_labelled_fields() {
    let card = String::from_utf8(encode().stdout).unwrap();
    let mut cmd = mk();
    cmd.arg("decode");
    for line in card.lines() {
        cmd.arg(line);
    }
    let out = cmd.output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    for label in [
        "xpub:",
        "origin_fingerprint:",
        "origin_path:",
        "policy_id_stubs:",
        "chunks:",
    ] {
        assert!(
            stdout.lines().any(|l| l.starts_with(label)),
            "decode must still emit `{label}`; got {stdout:?}"
        );
    }
}

/// **The shared display-grouping corpus is UNTOUCHED**, and the assertion is the
/// digest rather than the file's presence.
///
/// §6c removes `hyphen` and `comma` from `--separator`, one layer ABOVE this
/// file: the corpus's consumer maps each keyword to a `char` inside the test and
/// calls `render_grouped`, which takes a `char` and has no keyword vocabulary.
/// The corpus keeps its hyphen and comma rows, and its four copies across
/// `descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret` and
/// `mnemonic-toolkit` stay byte-identical — three of them CI-pinned to this
/// digest. A separator fix applied one layer too deep changes this number.
#[test]
fn the_display_grouping_corpus_still_hashes_to_7147b0ec() {
    use bitcoin::hashes::{Hash, sha256};

    const EXPECTED: &str = "7147b0ecc8cf175c41b2ade612d8dc4c6e523974f39188485ee68b2f99cc10ad";
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../design/display-grouping-vectors.tsv"
    );
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    assert_eq!(
        sha256::Hash::hash(&bytes).to_string(),
        EXPECTED,
        "the shared conformance corpus must not change"
    );

    // ...and it still carries the rows §6c retires at the CLI layer, which is
    // what proves the removal did not reach the codec layer.
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("hyphen"), "corpus keeps its hyphen rows");
    assert!(text.contains("comma"), "corpus keeps its comma rows");
}
