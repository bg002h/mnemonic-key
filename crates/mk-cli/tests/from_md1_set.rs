//! `mk encode --from-md1-set FILE` (P3, SPEC §10).
//!
//! §10's acceptance pipeline is `md encode --out wallet.md1` and then
//! `mk encode --from-md1-set wallet.md1`, so this flag reads what `md` wrote.
//!
//! **The load-bearing property is EQUIVALENCE**, exactly as `--keys` is to a
//! per-key loop: `--from-md1-set FILE` must bind the stub the way the repeated
//! `--from-md1` it replaces does, byte for byte. A flag that merely "worked"
//! could bind a different stub and mint a card that engraves fine and is refused
//! at recovery.
//!
//! **And it must be independent of which era wrote the file.** Today `md encode`
//! prints a `chunk-set-id:` header on stdout and groups its output space-5;
//! after the `md` branch lands it does neither. This flag skips every line that
//! is not an md1 string and strips display separators, so both eras' files work
//! — which is what lets it be built and gated before `md` changes at all.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

/// A REAL keyed wallet-policy md1, four chunks, minted by the primary
/// `md encode --force-chunked` — the same fixture `template_id_stub.rs` uses.
/// A keyed policy is 246 data symbols against a regular-code cap of 80, so it
/// cannot arrive any other way.
const KEYED_POLICY_A: [&str; 4] = [
    "md1fj5r4pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapguu6w2y2jygprc",
    "md1fj5r4psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wnj3gvx34m0wnt",
    "md1fj5r4pshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuyz646z0r98kyg",
    "md1fj5r4pslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqrfrshqj9459eg",
];

/// A second, distinct keyed policy: the same two keys at threshold 1. Eight
/// strings across the two files are TWO cards, not eight.
const KEYED_POLICY_B: [&str; 4] = [
    "md1f8h9cpspq2tvyyy4qqxppsq2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapggcqjxy8deuaca",
    "md1f8h9cpsf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2w445dju4r5jkq8",
    "md1f8h9cpshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuz9lsyct426why",
    "md1f8h9cpslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqtjjhsvxyyzfma",
];

/// `@0` of both policies. `mk encode` refuses to stamp a card with a KEYED
/// policy's stub unless the xpub is one of that policy's cosigners, so the
/// fixture has to be a real member.
const V1_XPUB: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const V1_FP: &str = "73c5da0a";
const V1_PATH: &str = "m/48'/0'/0'/2'";

fn mk() -> Command {
    Command::cargo_bin("mk").expect("mk binary")
}

fn base() -> Vec<String> {
    [
        "encode",
        "--xpub",
        V1_XPUB,
        "--origin-fingerprint",
        V1_FP,
        "--origin-path",
        V1_PATH,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn run(extra: &[String]) -> std::process::Output {
    mk().args(base()).args(extra).output().unwrap()
}

fn stdout_of(out: &std::process::Output) -> String {
    assert!(
        out.status.success(),
        "mk encode failed ({:?}): {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn repeated(chunks: &[&str]) -> Vec<String> {
    let mut v = Vec::new();
    for c in chunks {
        v.push("--from-md1".to_string());
        v.push((*c).to_string());
    }
    v
}

fn write(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    p
}

fn set_args(paths: &[&std::path::Path]) -> Vec<String> {
    let mut v = Vec::new();
    for p in paths {
        v.push("--from-md1-set".to_string());
        v.push(p.to_str().unwrap().to_string());
    }
    v
}

/// THE test: the flag is byte-equal to the repeated flag it replaces.
#[test]
fn from_md1_set_equals_four_repeated_from_md1() {
    let d = tempfile::tempdir().unwrap();
    let f = write(
        d.path(),
        "wallet.md1",
        &format!("{}\n", KEYED_POLICY_A.join("\n")),
    );

    let via_repeat = stdout_of(&run(&repeated(&KEYED_POLICY_A)));
    let via_set = stdout_of(&run(&set_args(&[&f])));

    assert_eq!(
        via_set, via_repeat,
        "--from-md1-set must bind the same stub"
    );
    assert!(via_set.starts_with("mk1"));
}

/// **The header-tolerance assertion is what makes this entry independent of the
/// `md` branch**, and it is testable now: today's `md encode --force-chunked`
/// prints `chunk-set-id: 0x…` on stdout ahead of the artifact, and after the
/// `md` branch lands it will not. Both files must work.
#[test]
fn from_md1_set_skips_a_chunk_set_id_header() {
    let d = tempfile::tempdir().unwrap();
    let clean = write(
        d.path(),
        "clean.md1",
        &format!("{}\n", KEYED_POLICY_A.join("\n")),
    );
    let with_header = write(
        d.path(),
        "headered.md1",
        &format!("chunk-set-id: 0x6386e\n{}\n", KEYED_POLICY_A.join("\n")),
    );

    assert_eq!(
        stdout_of(&run(&set_args(&[&with_header]))),
        stdout_of(&run(&set_args(&[&clean]))),
        "a non-md1 line must be skipped, not bound"
    );
}

/// Blank lines, `#` comments and stray prose are skipped too — a file a human
/// annotated is still a wallet file.
#[test]
fn from_md1_set_skips_blanks_comments_and_prose() {
    let d = tempfile::tempdir().unwrap();
    let clean = write(
        d.path(),
        "clean.md1",
        &format!("{}\n", KEYED_POLICY_A.join("\n")),
    );
    let messy = write(
        d.path(),
        "messy.md1",
        &format!(
            "# our reasonably complex wallet\n\nchunk-set-id: 0x6386e\n{}\n\nnote: keep this safe\n",
            KEYED_POLICY_A.join("\n")
        ),
    );
    assert_eq!(
        stdout_of(&run(&set_args(&[&messy]))),
        stdout_of(&run(&set_args(&[&clean])))
    );
}

/// A GROUPED file and an UNGROUPED one bind identically, so the flag never has
/// to know which era wrote it.
#[test]
fn from_md1_set_accepts_grouped_and_ungrouped_alike() {
    let d = tempfile::tempdir().unwrap();
    let ungrouped = write(
        d.path(),
        "ungrouped.md1",
        &format!("{}\n", KEYED_POLICY_A.join("\n")),
    );
    let mut body = String::new();
    for c in KEYED_POLICY_A {
        for (i, ch) in c.chars().enumerate() {
            if i > 0 && i % 5 == 0 {
                body.push(' ');
            }
            body.push(ch);
        }
        body.push('\n');
    }
    assert!(body.contains(' '), "precondition: grouped");
    let grouped = write(d.path(), "grouped.md1", &body);

    assert_eq!(
        stdout_of(&run(&set_args(&[&grouped]))),
        stdout_of(&run(&set_args(&[&ungrouped])))
    );
}

/// Repeatable, and it composes with `--from-md1`: two policies are TWO stubs on
/// one card, in the order given, however they were supplied.
#[test]
fn from_md1_set_is_repeatable_and_composes() {
    let d = tempfile::tempdir().unwrap();
    let a = write(
        d.path(),
        "a.md1",
        &format!("{}\n", KEYED_POLICY_A.join("\n")),
    );
    let b = write(
        d.path(),
        "b.md1",
        &format!("{}\n", KEYED_POLICY_B.join("\n")),
    );

    let mut both = repeated(&KEYED_POLICY_A);
    both.extend(repeated(&KEYED_POLICY_B));
    let via_repeat = stdout_of(&run(&both));

    assert_eq!(
        stdout_of(&run(&set_args(&[&a, &b]))),
        via_repeat,
        "two --from-md1-set files equal eight repeated --from-md1"
    );

    // MIXING the two channels composes by FLAG, not by argv position:
    // `--policy-id-stub` values first, then `--from-md1`, then
    // `--from-md1-set`. That is the order `mk encode` already used for the first
    // two, and clap does not preserve inter-flag argv order without asking for
    // it. It is asserted rather than left implicit because stub ORDER is on the
    // wire -- a different order is a different card -- so a reader must be able
    // to predict it. (`mk verify` compares stubs as a multiset, so a card minted
    // in either order still verifies, with a note.)
    let mut mixed = set_args(&[&a]);
    mixed.extend(repeated(&KEYED_POLICY_B));
    let mut b_then_a = repeated(&KEYED_POLICY_B);
    b_then_a.extend(repeated(&KEYED_POLICY_A));
    assert_eq!(
        stdout_of(&run(&mixed)),
        stdout_of(&run(&b_then_a)),
        "--from-md1 binds before --from-md1-set, whatever their argv positions"
    );
    assert_ne!(
        stdout_of(&run(&mixed)),
        via_repeat,
        "...and that really is a different card, so the order is worth pinning"
    );
}

/// **A file with no md1 in it is REFUSED, not silently skipped to nothing.**
///
/// "Skip every line that is not an md1 string" is what makes the flag tolerant
/// of a header — and, unguarded, it is also what would make a mistyped path to a
/// README bind zero stubs. `mk encode` would then fall through to
/// "at least one of --policy-id-stub or --from-md1 is required" and name flags
/// the operator did supply, or worse, mint against some other stub they did.
#[test]
fn from_md1_set_with_no_md1_lines_is_refused_naming_the_file() {
    let d = tempfile::tempdir().unwrap();
    let empty = write(d.path(), "readme.txt", "# no cards here\n\njust prose\n");
    let out = run(&set_args(&[&empty]));
    assert_eq!(
        out.status.code(),
        Some(64),
        "a file with no md1 strings is a usage error"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("readme.txt"),
        "the refusal must name the FILE, not just the flag; got {stderr:?}"
    );
    assert!(
        stderr.contains("--from-md1-set"),
        "and the flag; got {stderr:?}"
    );
}

/// An unreadable path is reported against `--from-md1-set`.
#[test]
fn from_md1_set_bad_path_names_the_flag() {
    let out = run(&[
        "--from-md1-set".to_string(),
        "/nonexistent/wallet.md1".to_string(),
    ]);
    assert_ne!(out.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("unexpected argument"),
        "--from-md1-set must be a real flag, not a clap parse error; got {stderr:?}"
    );
    assert!(stderr.contains("--from-md1-set"), "got {stderr:?}");
}
