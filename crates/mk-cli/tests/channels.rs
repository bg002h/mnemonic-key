//! `--in` and `--out`: the file channels (P3, SPEC §6b).
//!
//! §6b rules three things this file gates:
//!
//!   * `--in FILE` reads the tool's **own input material** from a file — for
//!     `mk encode` that is a key list, for the reading verbs it is mk1 strings.
//!   * `--out FILE` writes the artifact to a file **created 0600**, never
//!     `std::fs::write` (F-244).
//!   * `--out` **OVERWRITES**, ruled by the operator 2026-08-26.
//!
//! **The 0644 case is the whole of F-244 and the only assertion here that can
//! fail against a plausible wrong implementation.** `OpenOptions::mode()` binds
//! on CREATE only, so a mode-on-create implementation leaves an existing
//! world-readable target exactly as it found it — and re-running a command over
//! its own previous output is the case an operator actually hits. A fresh-file
//! test alone passes against that bug.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

const XPUB: &str = "xpub6Den8YwXbKQvkwukmx7Uukicw4qDgMEPuuUkhMp3Rn557YSN2uVQnCMQNSfgDtennU9nES3Wbbmz1LAPBydhNpED8NU4mf1SFF41hM7vFrc";
const STUB: &str = "11223344";

/// A SINGLE-SIG account xpub, needed because `mk address` refuses a BIP-48
/// multisig-cosigner origin outright ("single-key addresses would not match the
/// wallet"). Measured -- the multisig card exits 64 there, which would have made
/// the equality below two refusals agreeing.
const XPUB_84: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";

/// Two real cosigner records, so the `--keys`/`--in` equivalence is measured on
/// a batch rather than on a single card.
const KEYFILE_BODY: &str = concat!(
    "[73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf\n",
    "[73c5da0a/48'/0'/1'/2']xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk\n",
);
const BATCH_STUB: &str = "5b48af35";

fn mk() -> Command {
    Command::cargo_bin("mk").expect("mk binary")
}

fn single_card_args() -> Vec<String> {
    card_args(XPUB, "m/48'/0'/0'/2'")
}

fn card_args(xpub: &str, path: &str) -> Vec<String> {
    [
        "encode",
        "--xpub",
        xpub,
        "--origin-fingerprint",
        "aabbccdd",
        "--origin-path",
        path,
        "--policy-id-stub",
        STUB,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn tmpdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[cfg(unix)]
fn mode_of(p: &std::path::Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p).expect("stat").permissions().mode() & 0o777
}

fn stdout_of(out: &std::process::Output) -> String {
    assert!(
        out.status.success(),
        "command failed ({:?}): {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).unwrap()
}

// ──────────────────────────────────────────────────────────────────────────
// --out
// ──────────────────────────────────────────────────────────────────────────

/// A fresh target is created 0600 and holds exactly what stdout would have.
#[cfg(unix)]
#[test]
fn out_creates_the_artifact_owner_only() {
    let d = tmpdir();
    let target = d.path().join("card.mk1");

    let piped = stdout_of(&mk().args(single_card_args()).output().unwrap());

    let out = mk()
        .args(single_card_args())
        .args(["--out", target.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(mode_of(&target), 0o600, "--out creates the file owner-only");
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        piped,
        "--out writes exactly what stdout carried"
    );
    assert!(
        out.stdout.is_empty(),
        "with --out the artifact does not ALSO go to stdout (SPEC §6b: stdout is \
         used when --out is not given); got {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// **THE F-244 GATE.** A pre-existing world-readable target is TIGHTENED, not
/// inherited.
///
/// `OpenOptions::mode()` does nothing when the file already exists, so an
/// implementation that sets the mode only on create leaves this at 0644 and
/// fails here while passing every other test in this file.
#[cfg(unix)]
#[test]
fn out_tightens_an_existing_world_readable_target() {
    use std::os::unix::fs::PermissionsExt;
    let d = tmpdir();
    let target = d.path().join("stale.mk1");
    std::fs::write(&target, "STALE CONTENT THAT MUST BE REPLACED\n").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(mode_of(&target), 0o644, "precondition");

    let out = mk()
        .args(single_card_args())
        .args(["--out", target.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        mode_of(&target),
        0o600,
        "an existing 0644 target must be tightened, not inherited (F-244)"
    );
    let body = std::fs::read_to_string(&target).unwrap();
    assert!(
        !body.contains("STALE"),
        "the old contents must be gone; got {body:?}"
    );
    assert!(body.starts_with("mk1"), "got {body:?}");
}

/// `--out` OVERWRITES rather than refusing (operator ruling, 2026-08-26), and
/// the overwrite TRUNCATES — a shorter artifact over a longer file must not
/// leave the old tail behind.
#[cfg(unix)]
#[test]
fn out_overwrites_and_truncates() {
    let d = tmpdir();
    let target = d.path().join("card.mk1");
    std::fs::write(&target, "x".repeat(64 * 1024)).unwrap();

    for pass in 0..2 {
        let out = mk()
            .args(single_card_args())
            .args(["--out", target.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "pass {pass} must not refuse an existing file; stderr={}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    // NOT `!body.contains('x')` -- `x` is in the codex32 alphabet and appears
    // inside real mk1 strings, so that assertion fails against a CORRECT
    // implementation. Measured: it did. Byte-equality with what stdout carries
    // is the assertion that actually distinguishes truncate from append-over.
    let body = std::fs::read_to_string(&target).unwrap();
    let piped = stdout_of(&mk().args(single_card_args()).output().unwrap());
    assert_eq!(
        body, piped,
        "truncated to exactly the artifact, not appended-over"
    );
    for line in body.lines() {
        assert!(line.starts_with("mk1"), "got {line:?}");
    }
}

/// `--out` suppresses stdout and NOTHING else: the stderr engraving card and
/// its advisory are unchanged.
#[test]
fn out_does_not_suppress_the_card() {
    let d = tmpdir();
    let target = d.path().join("card.mk1");
    let piped = mk().args(single_card_args()).output().unwrap();
    let filed = mk()
        .args(single_card_args())
        .args(["--out", target.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&piped.stderr),
        String::from_utf8_lossy(&filed.stderr),
        "--out must not change stderr"
    );
}

// ──────────────────────────────────────────────────────────────────────────
// --in
// ──────────────────────────────────────────────────────────────────────────

/// `mk encode --in <keys>` is byte-equal on stdout to `--keys <the same file>`.
#[test]
fn in_on_encode_equals_keys() {
    let d = tmpdir();
    let kf = d.path().join("cosigners.keys");
    std::fs::write(&kf, KEYFILE_BODY).unwrap();

    let via_keys = stdout_of(
        &mk()
            .args([
                "encode",
                "--keys",
                kf.to_str().unwrap(),
                "--policy-id-stub",
                BATCH_STUB,
            ])
            .output()
            .unwrap(),
    );
    let via_in = stdout_of(
        &mk()
            .args([
                "encode",
                "--in",
                kf.to_str().unwrap(),
                "--policy-id-stub",
                BATCH_STUB,
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(via_in, via_keys, "--in must route to the --keys reader");
    assert!(via_in.starts_with("mk1"));
}

/// Supplying both exits 64 with a message naming BOTH flags — an operator who
/// gave two input channels must be told which two.
#[test]
fn in_and_keys_together_are_refused_naming_both() {
    let d = tmpdir();
    let kf = d.path().join("cosigners.keys");
    std::fs::write(&kf, KEYFILE_BODY).unwrap();

    let out = mk()
        .args([
            "encode",
            "--in",
            kf.to_str().unwrap(),
            "--keys",
            kf.to_str().unwrap(),
            "--policy-id-stub",
            BATCH_STUB,
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64), "--in with --keys must refuse");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--in") && stderr.contains("--keys"),
        "the refusal must name BOTH channels; got {stderr:?}"
    );
}

/// A bad `--in` path is reported against `--in`, not against `--keys`.
///
/// Printing another flag's name out of this flag's mouth is the defect class
/// F-294 files against the shared crate's refusal text; it is just as wrong
/// inside one binary.
#[test]
fn in_errors_name_in_not_keys() {
    let out = mk()
        .args([
            "encode",
            "--in",
            "/nonexistent/keys/file",
            "--policy-id-stub",
            BATCH_STUB,
        ])
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Without this guard the test passes VACUOUSLY before the flag exists:
    // clap's own `unexpected argument '--in' found` contains "--in" and exits
    // 64, so every assertion below would hold against a binary that has no
    // --in at all. Measured -- this test was one of two that passed RED.
    assert!(
        !stderr.contains("unexpected argument"),
        "--in must be a real flag, not a clap parse error; got {stderr:?}"
    );
    assert!(
        stderr.contains("--in"),
        "the error must name --in; got {stderr:?}"
    );
    assert!(
        !stderr.contains("--keys"),
        "the error must NOT name --keys; got {stderr:?}"
    );
}

/// `--in FILE` on every reading verb is byte-equal to passing the same strings
/// positionally — on BOTH streams, so a verb that read the file and then
/// reported something different about it cannot pass.
#[test]
fn in_on_the_reading_verbs_equals_the_positional_run() {
    let d = tmpdir();
    let multisig = stdout_of(&mk().args(single_card_args()).output().unwrap());
    let singlesig = stdout_of(
        &mk()
            .args(card_args(XPUB_84, "m/84'/0'/0'"))
            .output()
            .unwrap(),
    );
    assert!(
        multisig.lines().count() > 1,
        "want a chunked card; got {multisig:?}"
    );

    let f_multi = d.path().join("multisig.mk1");
    let f_single = d.path().join("singlesig.mk1");
    std::fs::write(&f_multi, &multisig).unwrap();
    std::fs::write(&f_single, &singlesig).unwrap();

    // `derive` is the one verb that needs a second flag to reach the reader at
    // all; without it both runs are clap usage errors and the equality holds
    // vacuously. Measured: it did, and the two usage strings then differed
    // anyway, which is how it was caught.
    let extra: &[(&str, &[&str], bool)] = &[
        ("decode", &[], false),
        ("inspect", &[], false),
        ("verify", &[], false),
        ("repair", &[], false),
        ("address", &["--count", "1"], true),
        ("derive", &["--index", "0"], false),
    ];
    for (verb, more, single_sig) in extra {
        let (card, f) = if *single_sig {
            (&singlesig, &f_single)
        } else {
            (&multisig, &f_multi)
        };
        let strings: Vec<&str> = card.lines().collect();
        let mut positional = mk();
        positional.arg(verb);
        positional.args(*more);
        for s in &strings {
            positional.arg(s);
        }
        let a = positional.output().unwrap();
        assert_eq!(
            a.status.code(),
            Some(0),
            "{verb}: the positional control must SUCCEED, or the equality below is \
             two error messages agreeing; stderr={}",
            String::from_utf8_lossy(&a.stderr)
        );

        let b = mk()
            .args([verb, "--in", f.to_str().unwrap()])
            .args(*more)
            .output()
            .unwrap();

        assert_eq!(
            a.status.code(),
            b.status.code(),
            "{verb}: --in must exit as the positional run does"
        );
        assert_eq!(
            String::from_utf8_lossy(&a.stdout),
            String::from_utf8_lossy(&b.stdout),
            "{verb}: --in stdout must equal the positional run's"
        );
        assert_eq!(
            String::from_utf8_lossy(&a.stderr),
            String::from_utf8_lossy(&b.stderr),
            "{verb}: --in stderr must equal the positional run's"
        );
    }
}

/// `--in` accepts a GROUPED card, because that is what a human transcribing
/// from the engraving card writes down.
#[test]
fn in_accepts_a_grouped_card() {
    let d = tmpdir();
    let run = mk().args(single_card_args()).output().unwrap();
    let unbroken = stdout_of(&run);
    let card = String::from_utf8(run.stderr.clone()).unwrap();
    let mut grouped = String::new();
    for l in card.lines().filter(|l| l.starts_with("mk1")) {
        grouped.push_str(l);
        grouped.push('\n');
    }
    assert!(grouped.contains(' '), "precondition: the card is grouped");

    let g = d.path().join("grouped.mk1");
    let u = d.path().join("unbroken.mk1");
    std::fs::write(&g, &grouped).unwrap();
    std::fs::write(&u, &unbroken).unwrap();

    let a = stdout_of(
        &mk()
            .args(["decode", "--in", u.to_str().unwrap()])
            .output()
            .unwrap(),
    );
    let b = stdout_of(
        &mk()
            .args(["decode", "--in", g.to_str().unwrap()])
            .output()
            .unwrap(),
    );
    assert_eq!(a, b, "a grouped card must decode as the unbroken one does");
}

/// An empty `--in` is refused, and the refusal names channels that EXIST
/// (§6h). Before this entry the message could not mention `--in`, because
/// there was none.
#[test]
fn an_empty_in_is_refused_and_the_remedy_is_executable() {
    let d = tmpdir();
    let f = d.path().join("empty.mk1");
    std::fs::write(&f, "").unwrap();
    let out = mk()
        .args(["decode", "--in", f.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64), "usage error");
    let stderr = String::from_utf8_lossy(&out.stderr);
    // Same vacuity guard as above: clap's unknown-flag error names --in too.
    assert!(
        !stderr.contains("unexpected argument"),
        "--in must be a real flag, not a clap parse error; got {stderr:?}"
    );
    assert!(
        stderr.contains("expected at least one mk1 string"),
        "the tool's OWN empty-input refusal, not clap's; got {stderr:?}"
    );
    assert!(
        stderr.contains("--in"),
        "the remedy must name --in now that it exists; got {stderr:?}"
    );
}
