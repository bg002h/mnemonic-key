//! bughunt-1119 F-A (bh-mk-I3, bh-mk-I4): the two line readers of `mk encode`
//! (`--keys`/`--in` key files and `--from-md1-set` files) follow Brian's ruling
//! B1: a leading BOM and CRLF are normalised; every other CR, every other
//! line-break character and every other U+FEFF is refused at exit 2 naming the
//! line. One test per row of `design/FIX_mk_fa_line_readers.md` section 5; the
//! same bytes and expectations as `scripts/mk-fa-line-readers-rows.sh`.

use std::io::Write;
use std::process::{Command, Output, Stdio};

use assert_cmd::cargo::CommandCargoExt;

const X0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const X1: &str = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk";
const X84: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
const A: &str = "md1yqpqqxzq2qwfv8urt848e";
const B: &str = "md1ypfdsssj5qqcyxppggah9rg9v9gjh4";
/// The four-chunk keyed policy of `from_md1_set.rs` (KEYED_POLICY_A).
const KA: [&str; 4] = [
    "md1fj5r4pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapguu6w2y2jygprc",
    "md1fj5r4psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wnj3gvx34m0wnt",
    "md1fj5r4pshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuyz646z0r98kyg",
    "md1fj5r4pslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqrfrshqj9459eg",
];
const BOM: &str = "\u{feff}";
const CR: &str = "\r";
const LF: &str = "\n";
const LS: &str = "\u{2028}";
const NEL: &str = "\u{85}";

fn r0() -> String {
    format!("[73c5da0a/48'/0'/0'/2']{X0}")
}
fn r1() -> String {
    format!("[73c5da0a/48'/0'/1'/2']{X1}")
}

fn kcr(flag: &str, src: &str, n: usize) -> String {
    format!(
        "{flag} {src}:{n}: this line holds a carriage return (CR) with no line feed (LF) after it, so the text after the CR would be read as part of line {n} and a key record after it could be lost. Convert the file to LF or CRLF line endings."
    )
}
fn scr(path: &str, n: usize) -> String {
    format!(
        "--from-md1-set {path}:{n}: this line holds a carriage return (CR) with no line feed (LF) after it, so the text after the CR would be read as part of line {n} and an md1 string after it could be skipped. Convert the file to LF or CRLF line endings."
    )
}
fn kbom(flag: &str, src: &str, n: usize) -> String {
    format!(
        "{flag} {src}:{n}: this line holds a byte-order mark (U+FEFF), which is accepted only as the first character of the file. Joining two files can leave the second file's mark inside the result: if the mark starts the line, delete it; if other text comes before it on the line, replace it with a line break."
    )
}
fn sbom(path: &str, n: usize) -> String {
    format!(
        "--from-md1-set {path}:{n}: this line holds a byte-order mark (U+FEFF), which is accepted only as the first character of the file. Joining two files can leave the second file's mark inside the result: if the mark starts the line, delete it; if other text comes before it on the line, replace it with a line break."
    )
}
fn klb(flag: &str, src: &str, n: usize, what: &str) -> String {
    format!(
        "{flag} {src}:{n}: this line holds a {what}, a line break this reader does not split lines on, so the text after it would be read as part of line {n} and a key record after it could be lost. Replace it with an ordinary line end (LF or CRLF)."
    )
}
fn slb(path: &str, n: usize, what: &str) -> String {
    format!(
        "--from-md1-set {path}:{n}: this line holds a {what}, a line break this reader does not split lines on, so the text after it would be read as part of line {n} and an md1 string after it could be skipped. Replace it with an ordinary line end (LF or CRLF)."
    )
}

/// Run `mk` in `dir` (files are named relative, as the error texts print them),
/// with `stdin` bytes when given.
fn run(dir: &std::path::Path, args: &[&str], stdin: Option<&str>) -> Output {
    let mut c = Command::cargo_bin("mk").expect("mk binary");
    c.current_dir(dir)
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = c.spawn().unwrap();
    if let Some(s) = stdin {
        child.stdin.take().unwrap().write_all(s.as_bytes()).unwrap();
    }
    child.wait_with_output().unwrap()
}

fn keys(dir: &std::path::Path, flag: &str, file: &str, body: &str, json: bool) -> Output {
    std::fs::write(dir.join(file), body).unwrap();
    let mut a = vec!["encode", flag, file, "--policy-id-stub", "deadbeef"];
    if json {
        a.push("--json");
    }
    run(dir, &a, None)
}

fn set(dir: &std::path::Path, file: &str, body: &str) -> Output {
    std::fs::write(dir.join(file), body).unwrap();
    run(
        dir,
        &[
            "encode",
            "--xpub",
            X84,
            "--origin-path",
            "m/84'/0'/0'",
            "--origin-fingerprint",
            "73c5da0a",
            "--from-md1-set",
            file,
        ],
        None,
    )
}

fn want_set(dir: &std::path::Path) -> Vec<u8> {
    let o = set(dir, "s_plain.txt", &format!("{A}{LF}{B}{LF}"));
    assert_eq!(o.status.code(), Some(0));
    assert!(!o.stdout.is_empty());
    o.stdout
}

fn chunk(dir: &std::path::Path, file: &str, body: &str) -> Output {
    std::fs::write(dir.join(file), body).unwrap();
    run(
        dir,
        &[
            "encode",
            "--xpub",
            X0,
            "--origin-fingerprint",
            "73c5da0a",
            "--origin-path",
            "m/48'/0'/0'/2'",
            "--from-md1-set",
            file,
        ],
        None,
    )
}

/// A refused line encoding is an invalid input: exit 2 (exit-code spec §3).
fn refused(o: &Output, text: &str) {
    assert_eq!(
        o.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        o.stdout.is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&o.stdout)
    );
    assert_eq!(
        String::from_utf8(o.stderr.clone()).unwrap(),
        format!("error: {text}\n")
    );
}

fn cards(o: &Output, n: usize) {
    assert_eq!(
        o.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    let s = String::from_utf8_lossy(&o.stdout);
    assert!(s.contains(&format!("\"card_count\":{n}")), "stdout: {s}");
}

fn td() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn fa_i3_keys_file_bare_cr_after_comment_refused() {
    let d = td();
    let d = d.path();
    let o = keys(
        d,
        "--keys",
        "k_cr_comment.txt",
        &format!("{} # first{CR}{}{LF}", r0(), r1()),
        false,
    );
    refused(&o, &kcr("--keys", "k_cr_comment.txt", 1));
}

#[test]
fn fa_i3_keys_file_bare_cr_after_comment_refused_json() {
    let d = td();
    let d = d.path();
    let o = keys(
        d,
        "--keys",
        "k_cr_comment.txt",
        &format!("{} # first{CR}{}{LF}", r0(), r1()),
        true,
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(o.stderr.is_empty());
    let want = format!(
        "{{\"error\":{{\"details\":null,\"exit_code\":2,\"kind\":\"InvalidInput\",\"message\":\"{}\"}},\"schema_version\":1}}\n",
        kcr("--keys", "k_cr_comment.txt", 1)
    );
    assert_eq!(String::from_utf8(o.stdout).unwrap(), want);
}

#[test]
fn fa_i3_keys_stdin_bare_cr_after_comment_refused() {
    let d = td();
    let d = d.path();
    let o = run(
        d,
        &["encode", "--keys", "-", "--policy-id-stub", "deadbeef"],
        Some(&format!("{} # first{CR}{}{LF}", r0(), r1())),
    );
    refused(&o, &kcr("--keys", "<stdin>", 1));
}

#[test]
fn fa_i3_in_file_bare_cr_after_comment_refused() {
    let d = td();
    let d = d.path();
    let o = keys(
        d,
        "--in",
        "k_cr_comment.txt",
        &format!("{} # first{CR}{}{LF}", r0(), r1()),
        false,
    );
    refused(&o, &kcr("--in", "k_cr_comment.txt", 1));
}

#[test]
fn fa_i3_keys_trailing_bare_cr_refused() {
    let d = td();
    let d = d.path();
    let o = keys(
        d,
        "--keys",
        "k_trailing_cr.txt",
        &format!("{}{LF}{}{CR}", r0(), r1()),
        false,
    );
    refused(&o, &kcr("--keys", "k_trailing_cr.txt", 2));
}

#[test]
fn fa_i3_keys_bom_after_first_char_refused() {
    let d = td();
    let d = d.path();
    let o = keys(
        d,
        "--keys",
        "k_bom_mid.txt",
        &format!("{}{LF}{BOM}{}{LF}", r0(), r1()),
        false,
    );
    refused(&o, &kbom("--keys", "k_bom_mid.txt", 2));
}

#[test]
fn fa_i3_keys_line_separator_after_comment_refused() {
    let d = td();
    let d = d.path();
    let o = keys(
        d,
        "--keys",
        "k_ls_comment.txt",
        &format!("{} # first{LS}{}{LF}", r0(), r1()),
        false,
    );
    refused(
        &o,
        &klb("--keys", "k_ls_comment.txt", 1, "line separator (U+2028)"),
    );
}

#[test]
fn fa_i4_md1_set_leading_bom_normalised() {
    let d = td();
    let d = d.path();
    let want = want_set(d);
    let o = set(d, "s_bom.txt", &format!("{BOM}{A}{LF}{B}{LF}"));
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(o.stdout, want);
}

#[test]
fn fa_i4_md1_set_leading_bom_chunked_normalised() {
    let d = td();
    let d = d.path();
    let plain = format!("{}{LF}", KA.join(LF));
    let want = chunk(d, "c_plain.txt", &plain);
    assert_eq!(want.status.code(), Some(0));
    assert!(!want.stdout.is_empty());
    let o = chunk(d, "c_bom.txt", &format!("{BOM}{plain}"));
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(o.stdout, want.stdout);
}

#[test]
fn fa_i4_md1_set_bare_cr_after_comment_refused() {
    let d = td();
    let d = d.path();
    let o = set(d, "s_cr_comment.txt", &format!("# note{CR}{A}{LF}{B}{LF}"));
    refused(&o, &scr("s_cr_comment.txt", 1));
}

#[test]
fn fa_i4_md1_set_nel_after_comment_refused() {
    let d = td();
    let d = d.path();
    let o = set(
        d,
        "s_nel_comment.txt",
        &format!("# note{NEL}{A}{LF}{B}{LF}"),
    );
    refused(
        &o,
        &slb("s_nel_comment.txt", 1, "next-line character (U+0085)"),
    );
}

#[test]
fn fa_i4_md1_set_bom_after_first_char_refused() {
    let d = td();
    let d = d.path();
    let o = set(d, "s_bom_mid.txt", &format!("{A}{LF}{BOM}{B}{LF}"));
    refused(&o, &sbom("s_bom_mid.txt", 2));
}

#[test]
fn fa_i4_md1_set_cr_only_refused() {
    let d = td();
    let d = d.path();
    let o = set(d, "s_cr_only.txt", &format!("{A}{CR}{B}{CR}"));
    refused(&o, &scr("s_cr_only.txt", 1));
}

#[test]
fn fa_i4_md1_set_trailing_bare_cr_refused() {
    let d = td();
    let d = d.path();
    let o = set(d, "s_trailing_cr.txt", &format!("{A}{LF}{B}{CR}"));
    refused(&o, &scr("s_trailing_cr.txt", 2));
}

#[test]
fn fa_control_keys_lf_mints_two() {
    let d = td();
    let d = d.path();
    cards(
        &keys(
            d,
            "--keys",
            "k_lf.txt",
            &format!("{} # first{LF}{}{LF}", r0(), r1()),
            true,
        ),
        2,
    );
}

#[test]
fn fa_control_keys_crlf_mints_two() {
    let d = td();
    let d = d.path();
    cards(
        &keys(
            d,
            "--keys",
            "k_crlf.txt",
            &format!("{}{CR}{LF}{}{CR}{LF}", r0(), r1()),
            true,
        ),
        2,
    );
}

#[test]
fn fa_control_keys_leading_bom_mints_two() {
    let d = td();
    let d = d.path();
    cards(
        &keys(
            d,
            "--keys",
            "k_bom.txt",
            &format!("{BOM}{}{LF}{}{LF}", r0(), r1()),
            true,
        ),
        2,
    );
}

#[test]
fn fa_control_keys_cr_only_text_unchanged() {
    let d = td();
    let d = d.path();
    let o = keys(
        d,
        "--keys",
        "k_cr_only.txt",
        &format!("{}{CR}{}{CR}", r0(), r1()),
        false,
    );
    refused(
        &o,
        "--keys k_cr_only.txt: the file uses CR-only line endings (classic Mac), so every record after the first would be read as part of line 1. Convert it to LF or CRLF.",
    );
}

#[test]
fn fa_control_md1_set_crlf_equals_lf() {
    let d = td();
    let d = d.path();
    let want = want_set(d);
    let o = set(d, "s_crlf.txt", &format!("{A}{CR}{LF}{B}{CR}{LF}"));
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(o.stdout, want);
}
