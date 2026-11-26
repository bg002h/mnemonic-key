//! FIX_bh1119_mk_out_writes (bughunt-1119 bh-mk-I1, bh-mk-I6, T7): `mk encode`
//! output writes that fail closed and never replace an input. One test per TRACE
//! row of the FIX document (section 4), named exactly as its "test" column.
//!
//! Unix only: on Windows io-lib's `same_file` detects no hard link and no
//! redirected stdin/stdout, and the mode and read-only-directory rows are Unix
//! semantics. The FSIZE8 rows (o1, o2, o4, o20) are also Linux only (the file
//! size limit is set with `setrlimit` and SIGXFSZ is ignored, so a write past 8
//! bytes returns EFBIG, a stand-in for a full disk or quota).
#![cfg(unix)]

use std::fs::{File, OpenOptions};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use assert_cmd::cargo::CommandCargoExt;

/// The public testnet account key (mk-cli fixture `V15_84_TEST`), as one record.
const KEYS: &str = "[73c5da0a/84'/1'/0']tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X\n";
const MD1: &str = "md1yqpqqxzq2qwfv8urt848e\n";
const NOTE: &str = "note: stdout is watch-only — public keys only, cannot spend";
/// `R` of the FIX document: the X1 tail.
const R: &str = "so writing the card would replace the input mk is reading. Write to a new name.";
/// `S` of the FIX document: the X2 tail.
const S: &str = "so the card would go into the file mk reads its input from. A > redirect empties that file before mk starts, so check that it still holds what you expect. Write to a new name.";
const X3: &str = "--out F and standard output are the same file (the same path, a symlink or a hard link), so what mk prints on stdout would go into the file it writes. A > redirect empties that file before mk starts, so check that it still holds what you expect. Write to a new name.";

/// Where mk's standard output goes.
enum Out {
    Pipe,
    /// `>>name`
    Append(&'static str),
    /// `>name`
    Truncate(&'static str),
}

struct Env {
    dir: tempfile::TempDir,
}

impl Env {
    /// `keys` holds the one key record; `F` holds `PRIOR-CARD\n` (0600);
    /// `set.md1` holds a keyless md1 template.
    fn fresh() -> Env {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("keys"), KEYS).unwrap();
        std::fs::write(dir.path().join("F"), "PRIOR-CARD\n").unwrap();
        std::fs::set_permissions(dir.path().join("F"), PermissionsExt::from_mode(0o600)).unwrap();
        std::fs::write(dir.path().join("set.md1"), MD1).unwrap();
        Env { dir }
    }

    fn p(&self, name: &str) -> std::path::PathBuf {
        self.dir.path().join(name)
    }

    fn read(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.p(name)).unwrap_or_else(|e| panic!("read {name}: {e}"))
    }

    fn text(&self, name: &str) -> String {
        String::from_utf8(self.read(name)).unwrap()
    }

    fn mode(&self, name: &str) -> u32 {
        std::fs::metadata(self.p(name))
            .unwrap()
            .permissions()
            .mode()
            & 0o777
    }

    /// No `*.tmp` anywhere within two levels of the work directory.
    fn no_tmp(&self) -> bool {
        let tmp = |d: &Path| {
            std::fs::read_dir(d)
                .map(|it| {
                    it.flatten()
                        .any(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
                })
                .unwrap_or(false)
        };
        if tmp(self.dir.path()) {
            return false;
        }
        for e in std::fs::read_dir(self.dir.path()).unwrap().flatten() {
            if e.path().is_dir() && tmp(&e.path()) {
                return false;
            }
        }
        true
    }

    /// Runs `mk` in the work directory. `stdin` is a file name; `fsize8` sets a
    /// file-size limit of 8 bytes with SIGXFSZ ignored (EFBIG on a larger write).
    fn run(&self, args: &[&str], out: Out, stdin: Option<&str>, fsize8: bool) -> Output {
        let mut c = Command::cargo_bin("mk").expect("mk binary");
        c.current_dir(self.dir.path()).args(args);
        c.stdin(match stdin {
            Some(n) => Stdio::from(File::open(self.p(n)).unwrap()),
            None => Stdio::null(),
        });
        let to_file = match out {
            Out::Pipe => None,
            Out::Append(n) => Some(
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(self.p(n))
                    .unwrap(),
            ),
            Out::Truncate(n) => Some(File::create(self.p(n)).unwrap()),
        };
        c.stderr(Stdio::piped());
        match to_file {
            Some(f) => c.stdout(Stdio::from(f)),
            None => c.stdout(Stdio::piped()),
        };
        if fsize8 {
            // SAFETY: only async-signal-safe calls (setrlimit, signal) between fork and exec.
            unsafe {
                c.pre_exec(|| {
                    let lim = libc::rlimit {
                        rlim_cur: 8,
                        rlim_max: 8,
                    };
                    if libc::setrlimit(libc::RLIMIT_FSIZE, &lim) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
                    Ok(())
                });
            }
        }
        c.output().expect("run mk")
    }

    /// `mk encode --in keys --policy-id-stub deadbeef` plus `extra`.
    fn enc(&self, extra: &[&str], out: Out, fsize8: bool) -> Output {
        let mut a = vec!["encode", "--in", "keys", "--policy-id-stub", "deadbeef"];
        a.extend_from_slice(extra);
        self.run(&a, out, None, fsize8)
    }

    /// The reference artifact (the stdout card, with `extra` flags) from a clean run.
    fn reference(&self, extra: &[&str]) -> Vec<u8> {
        let o = self.enc(extra, Out::Pipe, false);
        assert_eq!(o.status.code(), Some(0), "reference run");
        o.stdout
    }
}

fn code(o: &Output) -> Option<i32> {
    o.status.code()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).trim_end().to_string()
}

fn out_s(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn efbig(file: &str) -> String {
    format!("mk: cannot write the output: {file}: File too large (os error 27)")
}

// --- I1: a failed --out write (R1, R3)

#[cfg(target_os = "linux")]
#[test]
fn bh_mk_i1_failed_replacement_preserves_old_card() {
    let e = Env::fresh();
    let o = e.enc(&["--out", "F"], Out::Pipe, true);
    assert_eq!(code(&o), Some(32), "{}", err(&o));
    assert_eq!(err(&o), format!("{}. F is unchanged.", efbig("F")));
    assert_eq!(e.text("F"), "PRIOR-CARD\n");
    assert!(e.no_tmp());
}

#[cfg(target_os = "linux")]
#[test]
fn i1_failed_write_leaves_an_absent_out_absent() {
    let e = Env::fresh();
    std::fs::remove_file(e.p("F")).unwrap();
    let o = e.enc(&["--out", "F"], Out::Pipe, true);
    assert_eq!(code(&o), Some(32), "{}", err(&o));
    assert_eq!(err(&o), format!("{}. Nothing was written.", efbig("F")));
    assert!(!e.p("F").exists());
    assert!(e.no_tmp());
}

#[test]
fn i1_missing_directory_prints_the_t6_line() {
    let e = Env::fresh();
    let o = e.enc(&["--out", "nodir/F"], Out::Pipe, false);
    assert_eq!(code(&o), Some(32), "{}", err(&o));
    assert_eq!(
        err(&o),
        "mk: cannot write the output: nodir/F: No such file or directory (os error 2). Nothing was written."
    );
    assert!(o.stdout.is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn i1_json_failed_write_keeps_stdout_empty() {
    let e = Env::fresh();
    let o = e.enc(&["--json", "--out", "F"], Out::Pipe, true);
    assert_eq!(code(&o), Some(32), "{}", err(&o));
    assert_eq!(err(&o), format!("{}. F is unchanged.", efbig("F")));
    assert!(o.stdout.is_empty(), "stdout: {}", out_s(&o));
    assert_eq!(e.text("F"), "PRIOR-CARD\n");
}

#[test]
fn i1_read_only_directory_refused_file_unchanged() {
    // Root ignores directory modes (ms #11 o6 does the same).
    if unsafe { libc::geteuid() } == 0 {
        return;
    }
    let e = Env::fresh();
    std::fs::create_dir(e.p("ro")).unwrap();
    std::fs::write(e.p("ro/F"), "PRIOR\n").unwrap();
    std::fs::set_permissions(e.p("ro/F"), PermissionsExt::from_mode(0o644)).unwrap();
    std::fs::set_permissions(e.p("ro"), PermissionsExt::from_mode(0o555)).unwrap();
    let o = e.enc(&["--out", "ro/F"], Out::Pipe, false);
    std::fs::set_permissions(e.p("ro"), PermissionsExt::from_mode(0o755)).unwrap();
    assert_eq!(code(&o), Some(32), "{}", err(&o));
    assert_eq!(
        err(&o),
        "mk: cannot write the output: ro/F: Permission denied (os error 13). ro/F is unchanged."
    );
    assert_eq!(e.text("ro/F"), "PRIOR\n");
}

#[cfg(target_os = "linux")]
#[test]
fn i1_stdio_alias_onto_a_file_names_no_state() {
    let e = Env::fresh();
    std::fs::write(e.p("s.txt"), "OLD BACKUP\n").unwrap();
    let o = e.enc(&["--out", "/dev/stdout"], Out::Append("s.txt"), true);
    assert_eq!(code(&o), Some(32), "{}", err(&o));
    let line = efbig("/dev/stdout");
    let s = e.text("s.txt");
    let got = err(&o);
    if got == line {
        // io-lib truncates behind a stdio alias: no state sentence may follow.
        assert_ne!(s, "OLD BACKUP\n", "s.txt unchanged but no sentence says so");
    } else if got == format!("{line}. /dev/stdout is unchanged.") {
        // An io-lib that appends leaves s.txt intact.
        assert_eq!(s, "OLD BACKUP\n", "the sentence is false of s.txt");
    } else {
        panic!("stderr is neither accepted line: {got}");
    }
}

// --- I6: --out names an input (R2)

fn x1(out: &str, input: &str) -> String {
    format!("error: --out {out} is the same file as {input}, {R}")
}

#[test]
fn i6_out_same_path_as_in_refuses() {
    let e = Env::fresh();
    let o = e.enc(&["--out", "keys"], Out::Pipe, false);
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x1("keys", "--in keys"));
    assert_eq!(e.text("keys"), KEYS);
    assert!(o.stdout.is_empty());
}

#[test]
fn i6_out_aliasing_source_refuses() {
    let e = Env::fresh();
    std::fs::hard_link(e.p("keys"), e.p("hl")).unwrap();
    let o = e.enc(&["--out", "hl"], Out::Pipe, false);
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x1("hl", "--in keys"));
    assert_eq!(e.text("keys"), KEYS);
}

#[test]
fn i6_out_symlink_to_in_refuses() {
    let e = Env::fresh();
    symlink("keys", e.p("sl")).unwrap();
    let o = e.enc(&["--out", "sl"], Out::Pipe, false);
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x1("sl", "--in keys"));
    assert_eq!(e.text("keys"), KEYS);
}

#[test]
fn i6_out_same_file_as_keys_refuses() {
    let e = Env::fresh();
    let o = e.run(
        &[
            "encode",
            "--keys",
            "keys",
            "--policy-id-stub",
            "deadbeef",
            "--out",
            "./keys",
        ],
        Out::Pipe,
        None,
        false,
    );
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x1("./keys", "--keys keys"));
    assert_eq!(e.text("keys"), KEYS);
}

#[test]
fn i6_out_same_file_as_md1_set_refuses() {
    let e = Env::fresh();
    let o = e.run(
        &[
            "encode",
            "--in",
            "keys",
            "--from-md1-set",
            "set.md1",
            "--out",
            "set.md1",
        ],
        Out::Pipe,
        None,
        false,
    );
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x1("set.md1", "--from-md1-set set.md1"));
    assert_eq!(e.text("set.md1"), MD1);
}

#[test]
fn i6_out_same_file_as_stdin_refuses() {
    let e = Env::fresh();
    let o = e.run(
        &[
            "encode",
            "--in",
            "-",
            "--policy-id-stub",
            "deadbeef",
            "--out",
            "keys",
        ],
        Out::Pipe,
        Some("keys"),
        false,
    );
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x1("keys", "standard input (--in -)"));
    assert_eq!(e.text("keys"), KEYS);
}

#[test]
fn i6_json_out_refusal_is_an_envelope() {
    let e = Env::fresh();
    let o = e.enc(&["--json", "--out", "keys"], Out::Pipe, false);
    assert_eq!(code(&o), Some(8));
    assert!(o.stderr.is_empty(), "stderr: {}", err(&o));
    let want = format!("\"message\":\"--out keys is the same file as --in keys, {R}\"");
    assert!(out_s(&o).contains(&want), "stdout: {}", out_s(&o));
    assert_eq!(e.text("keys"), KEYS);
}

// --- I6 widened to stdout (R4): encode without --out

fn x2(input: &str) -> String {
    format!("error: standard output is the same file as {input}, {S}")
}

#[test]
fn i6_stdout_appended_to_in_refuses() {
    let e = Env::fresh();
    let o = e.enc(&[], Out::Append("keys"), false);
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x2("--in keys"));
    assert_eq!(e.text("keys"), KEYS);
}

#[test]
fn i6_json_stdout_refusal_goes_to_stderr() {
    let e = Env::fresh();
    let o = e.enc(&["--json"], Out::Append("keys"), false);
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x2("--in keys"));
    assert_eq!(e.text("keys"), KEYS, "no envelope written into the input");
}

#[test]
fn i6_stdout_truncating_in_refuses() {
    let e = Env::fresh();
    let o = e.enc(&[], Out::Truncate("keys"), false);
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x2("--in keys"));
}

#[test]
fn i6_stdout_and_stdin_one_file_refuses() {
    let e = Env::fresh();
    let o = e.run(
        &["encode", "--in", "-", "--policy-id-stub", "deadbeef"],
        Out::Append("keys"),
        Some("keys"),
        false,
    );
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x2("standard input (--in -)"));
    assert_eq!(e.text("keys"), KEYS);
}

// --- controls (they pass before and after)

#[test]
fn control_out_new_name_is_the_stdout_card() {
    let e = Env::fresh();
    let card = e.reference(&[]);
    let o = e.enc(&["--out", "G"], Out::Pipe, false);
    assert_eq!(code(&o), Some(0), "{}", err(&o));
    assert_eq!(err(&o), NOTE);
    assert_eq!(e.read("G"), card);
    assert_eq!(e.mode("G"), 0o600);
    assert!(o.stdout.is_empty());
}

#[test]
fn control_out_symlink_replaces_its_target() {
    let e = Env::fresh();
    let card = e.reference(&[]);
    symlink("F", e.p("lk")).unwrap();
    let o = e.enc(&["--out", "lk"], Out::Pipe, false);
    assert_eq!(code(&o), Some(0), "{}", err(&o));
    assert_eq!(e.read("F"), card);
    assert!(std::fs::symlink_metadata(e.p("lk")).unwrap().is_symlink());
}

#[test]
fn control_out_overwrites_and_tightens() {
    let e = Env::fresh();
    let card = e.reference(&[]);
    std::fs::set_permissions(e.p("F"), PermissionsExt::from_mode(0o644)).unwrap();
    let o = e.enc(&["--out", "F"], Out::Pipe, false);
    assert_eq!(code(&o), Some(0), "{}", err(&o));
    assert_eq!(e.read("F"), card);
    assert_eq!(e.mode("F"), 0o600);
    assert!(e.no_tmp());
}

#[test]
fn control_stdout_pipe_is_not_an_input() {
    let e = Env::fresh();
    let card = e.reference(&[]);
    let o = e.enc(&[], Out::Pipe, false);
    assert_eq!(code(&o), Some(0), "{}", err(&o));
    assert_eq!(o.stdout, card);
    assert_eq!(e.text("keys"), KEYS);
}

// --- T7: --json --out writes both (R6); stdout must not be --out (R4, X3)

#[test]
fn i1_t7_json_out_writes_both() {
    let e = Env::fresh();
    let card = e.reference(&[]);
    let json = e.reference(&["--json"]);
    std::fs::remove_file(e.p("F")).unwrap();
    let o = e.enc(&["--json", "--out", "F"], Out::Pipe, false);
    assert_eq!(code(&o), Some(0), "{}", err(&o));
    assert_eq!(e.read("F"), card);
    assert_eq!(e.mode("F"), 0o600);
    assert_eq!(o.stdout, json);
}

#[test]
fn i1_t7_json_out_grouped_file_unbroken_json() {
    let e = Env::fresh();
    let card5 = e.reference(&["--group-size", "5"]);
    let json = e.reference(&["--json"]);
    std::fs::remove_file(e.p("F")).unwrap();
    let o = e.enc(
        &["--json", "--group-size", "5", "--out", "F"],
        Out::Pipe,
        false,
    );
    assert_eq!(code(&o), Some(0), "{}", err(&o));
    assert_eq!(e.read("F"), card5);
    assert_eq!(o.stdout, json);
}

#[test]
fn i6_json_out_stdout_appended_to_in_refuses() {
    let e = Env::fresh();
    let o = e.enc(&["--json", "--out", "G"], Out::Append("keys"), false);
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x2("--in keys"));
    assert_eq!(e.text("keys"), KEYS);
    assert!(!e.p("G").exists());
}

#[test]
fn i6_json_out_names_in_and_stdout_appends_to_it() {
    let e = Env::fresh();
    let o = e.enc(&["--json", "--out", "keys"], Out::Append("keys"), false);
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x2("--in keys"));
    assert_eq!(e.text("keys"), KEYS, "no envelope appended");
}

#[test]
fn i6_json_usage_error_never_appends_to_in() {
    let e = Env::fresh();
    let o = e.run(
        &[
            "encode",
            "--in",
            "keys",
            "--policy-id-stub",
            "zz",
            "--json",
            "--out",
            "G",
        ],
        Out::Append("keys"),
        None,
        false,
    );
    assert_eq!(code(&o), Some(8));
    assert_eq!(err(&o), x2("--in keys"));
    assert_eq!(e.text("keys"), KEYS);
}

#[test]
fn i1_t7_json_stdout_is_out_refused() {
    let e = Env::fresh();
    std::fs::remove_file(e.p("F")).unwrap();
    let o = e.enc(&["--json", "--out", "F"], Out::Truncate("F"), false);
    assert_eq!(code(&o), Some(8), "{}", err(&o));
    assert!(o.stderr.is_empty(), "stderr: {}", err(&o));
    let want = format!("\"message\":\"{X3}\"");
    assert!(e.text("F").contains(&want), "F: {}", e.text("F"));
}

#[test]
fn control_json_out_piped_writes_both() {
    let e = Env::fresh();
    let card = e.reference(&[]);
    let json = e.reference(&["--json"]);
    std::fs::remove_file(e.p("F")).unwrap();
    let o = e.enc(&["--json", "--out", "F"], Out::Pipe, false);
    assert_eq!(code(&o), Some(0), "{}", err(&o));
    assert_eq!(e.read("F"), card);
    assert_eq!(o.stdout, json);
}
