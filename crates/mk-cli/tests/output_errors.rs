//! What mk does when stdout cannot take its output (crash sweep F1 and the
//! family rule, grade-mk-crash-sweep-1; manual `44-mk-cli.md` "Exit codes").
//!
//! - A reader that closed the pipe only stops the output: no text, no
//!   signal, the exit code is the verb's own.
//! - Any other write error (`/dev/full`, ENOSPC), from a verb or `--help`:
//!   "mk: cannot write the output: <error>" on stderr, exit 1; a verb that
//!   was already failing keeps its code.
//! - No file of the crate but `src/out.rs` names stdout.
#![cfg(target_os = "linux")]

use std::fs::OpenOptions;
use std::os::fd::{FromRawFd, OwnedFd};
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Output, Stdio};

const XPUB: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
const ENCODE: [&str; 9] = [
    "encode",
    "--xpub",
    XPUB,
    "--origin-path",
    "m/84'/0'/0'",
    "--origin-fingerprint",
    "aabbccdd",
    "--policy-id-stub",
    "11223344",
];
const FULL: &str = "mk: cannot write the output: No space left on device (os error 28)";

fn mk() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mk"))
}

fn card() -> Vec<String> {
    let o = mk().args(ENCODE).output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    String::from_utf8(o.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

/// The card with one character of its first string changed: repair exits 5.
fn damaged() -> Vec<String> {
    let mut c = card();
    let mut b = c[0].clone().into_bytes();
    b[12] = if b[12] == b'q' { b'p' } else { b'q' };
    c[0] = String::from_utf8(b).unwrap();
    c
}

fn args(verb: &[&str], card: &[String]) -> Vec<String> {
    verb.iter()
        .map(|s| s.to_string())
        .chain(card.iter().cloned())
        .collect()
}

fn closed_pipe() -> OwnedFd {
    let mut fds = [0i32; 2];
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    unsafe { libc::close(fds[0]) }; // the reader is gone before mk starts
    unsafe { OwnedFd::from_raw_fd(fds[1]) }
}

fn on_full(a: &[String]) -> Output {
    let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
    mk().args(a)
        .stdout(Stdio::from(full))
        .stderr(Stdio::piped())
        .output()
        .unwrap()
}

fn on_closed(a: &[String], stderr_too: bool) -> Output {
    let err = if stderr_too {
        Stdio::from(closed_pipe())
    } else {
        Stdio::piped()
    };
    mk().args(a)
        .stdout(Stdio::from(closed_pipe()))
        .stderr(err)
        .output()
        .unwrap()
}

fn full_case(a: &[String], code: i32) {
    let o = on_full(a);
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(!err.contains("panicked"), "{a:?}: panic: {err}");
    assert_eq!(
        o.status.code(),
        Some(code),
        "{a:?}: status {:?}: {err}",
        o.status
    );
    assert_eq!(err.lines().last(), Some(FULL), "{a:?}: stderr: {err}");
}

fn closed_case(a: &[String], code: i32, stderr_too: bool) {
    let o = on_closed(a, stderr_too);
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(
        o.status.signal(),
        None,
        "{a:?}: killed by a signal: {:?}",
        o.status
    );
    assert_eq!(
        o.status.code(),
        Some(code),
        "{a:?}: status {:?}: {err}",
        o.status
    );
    assert!(
        !err.contains("panicked") && !err.contains("cannot write"),
        "{a:?}: {err}"
    );
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn full_stdout_exits_1_with_the_line() {
    let c = card();
    for a in [
        s(&["gui-schema"]),
        s(&["vectors"]),
        s(&["vectors", "--pretty"]),
        s(&ENCODE),
        args(&["decode"], &c),
        args(&["decode", "--json"], &c),
        args(&["inspect"], &c),
        args(&["verify"], &c),
        args(&["repair"], &c),
        args(&["address", "--count", "2"], &c),
        args(&["derive", "--path", "m/0/5"], &c),
        s(&["--help"]),
        s(&["--version"]),
        s(&["decode", "--help"]),
    ] {
        full_case(&a, 1);
    }
}

#[test]
fn full_stdout_after_a_repair_exits_1_not_5() {
    let d = damaged();
    let o = mk().args(args(&["repair"], &d)).output().unwrap();
    assert_eq!(o.status.code(), Some(5), "fixture: repair should exit 5");
    full_case(&args(&["repair"], &d), 1);
}

#[test]
fn full_stdout_keeps_a_failing_code() {
    // A usage error in --json mode prints its envelope on stdout (exit 64).
    full_case(
        &args(&["address", "--json", "--count", "1000001"], &card()),
        64,
    );
    // A content mismatch in --json mode prints its envelope on stdout (exit 4).
    const OTHER: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
    full_case(&args(&["verify", "--json", "--xpub", OTHER], &card()), 4);
}

#[test]
fn closed_reader_exits_with_the_verbs_code() {
    let c = card();
    for a in [
        s(&["gui-schema"]),
        s(&["vectors"]),
        s(&ENCODE),
        args(&["address", "--count", "2"], &c),
        s(&["--help"]),
        s(&["--version"]),
    ] {
        closed_case(&a, 0, false);
    }
    closed_case(&args(&["repair"], &damaged()), 5, false);
}

#[test]
fn closed_reader_on_stdout_and_stderr_exits_0() {
    // encode prints its stderr notes after the card; `vectors --out` prints
    // "wrote N vector file(s)" with `eprintln!`.
    closed_case(&s(&ENCODE), 0, true);
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path().to_str().unwrap();
    closed_case(&s(&["vectors", "--out", d]), 0, true);
}

#[test]
fn only_out_rs_names_stdout() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    let mut stack = vec![dir];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") && !p.ends_with("src/out.rs") {
                let t = std::fs::read_to_string(&p).unwrap();
                for n in ["stdout(", "std::print", "std::eprint", "core::print"] {
                    if t.contains(n) {
                        bad.push(format!("{}: {n}", p.display()));
                    }
                }
            }
        }
    }
    assert!(bad.is_empty(), "stdout named outside src/out.rs: {bad:?}");
}

/// std's stdout reports EBADF as a successful write, so a stdout opened
/// read-only (`mk gui-schema 1</dev/null`) lost the output and exited 0.
#[test]
fn read_only_stdout_exits_1_with_the_line() {
    const BADF: &str = "mk: cannot write the output: Bad file descriptor (os error 9)";
    let on_read_only = |a: &[String]| {
        let ro = std::fs::File::open("/dev/null").unwrap();
        mk().args(a)
            .stdout(Stdio::from(ro))
            .stderr(Stdio::piped())
            .output()
            .unwrap()
    };
    let c = card();
    for (a, code) in [
        (s(&["gui-schema"]), 1),
        (s(&ENCODE), 1),
        (args(&["decode"], &c), 1),
        (s(&["--help"]), 1),
        (s(&["--version"]), 1),
        (args(&["repair"], &damaged()), 1),
        // A failing verb keeps its code (the --json usage envelope, 64).
        (args(&["address", "--json", "--count", "1000001"], &c), 64),
    ] {
        let o = on_read_only(&a);
        let err = String::from_utf8_lossy(&o.stderr);
        assert_eq!(o.status.code(), Some(code), "{a:?}: {err}");
        assert_eq!(err.lines().last(), Some(BADF), "{a:?}: {err}");
    }
    // A verb that writes nothing to stdout keeps its code and prints no line.
    let dir = tempfile::tempdir().unwrap();
    let o = on_read_only(&s(&["vectors", "--out", dir.path().to_str().unwrap()]));
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(0), "{err}");
    assert!(!err.contains("cannot write"), "{err}");
}

/// Linux accepts access mode 3 (`O_WRONLY | O_RDWR`): the fd can neither read
/// nor write, and a write gets EBADF, which std reports as a success.
#[test]
fn no_access_mode_stdout_exits_1_with_the_line() {
    const BADF: &str = "mk: cannot write the output: Bad file descriptor (os error 9)";
    let fd = unsafe { libc::open(c"/dev/null".as_ptr(), libc::O_ACCMODE) };
    assert!(fd >= 0);
    let o = mk()
        .args(["gui-schema"])
        .stdout(Stdio::from(unsafe { OwnedFd::from_raw_fd(fd) }))
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    let err = String::from_utf8_lossy(&o.stderr);
    assert_eq!(o.status.code(), Some(1), "{err}");
    assert_eq!(err.lines().last(), Some(BADF), "{err}");
}
