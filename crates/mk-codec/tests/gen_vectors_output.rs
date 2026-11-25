//! Output-path rows for the `gen_mk_vectors` maintainer tool (bh-mk-I16;
//! `design/FIX_mk_gen_vectors_overwrite.md` section 4). The tool writes the
//! vectors corpus and then `csid_ext_v0.1.json` beside it; these rows pin
//! what happens when the two paths are the same file.
//!
//! Compiled only with `--features gen-vectors` (the binary needs it).
#![cfg(feature = "gen-vectors")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mk_codec::test_vectors::csid_ext::CSID_EXT_JSON;
use mk_codec::test_vectors::V0_1_JSON;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// A fresh, empty directory under `CARGO_TARGET_TMPDIR`, one per row.
fn fresh_dir(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("gen_vectors_output_{name}"));
    if d.exists() {
        fs::remove_dir_all(&d).expect("clear row directory");
    }
    fs::create_dir_all(&d).expect("create row directory");
    d
}

/// Runs the generator with `args`, current directory `dir`.
fn run(dir: &Path, args: &[&str]) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_gen_mk_vectors"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run gen_mk_vectors");
    Run {
        code: out.status.code(),
        stdout: String::from_utf8(out.stdout).expect("utf8 stdout"),
        stderr: String::from_utf8(out.stderr).expect("utf8 stderr"),
    }
}

fn json_len(text: &str, key: &str) -> usize {
    let v: serde_json::Value = serde_json::from_str(text).expect("corpus parses");
    v[key].as_array().expect("array").len()
}

fn n_vectors() -> usize {
    json_len(V0_1_JSON, "vectors")
}

fn n_rows() -> usize {
    json_len(CSID_EXT_JSON, "rows")
}

fn t1(p: &str) -> String {
    format!(
        "gen_mk_vectors: refused: --output {p} is named csid_ext_v0.1.json, the csid_ext file this tool writes beside it; nothing was written\n"
    )
}

fn t2(p: &str, c: &str) -> String {
    format!(
        "gen_mk_vectors: error: {p} no longer holds the {} vectors after {c} was written; it is the same file as {c} or was changed meanwhile\n",
        n_vectors()
    )
}

fn assert_t2(r: &Run, p: &Path, c: &Path) {
    assert_eq!(r.code, Some(1));
    assert_eq!(r.stdout, "");
    assert_eq!(
        r.stderr,
        t2(&p.display().to_string(), &c.display().to_string())
    );
}

#[test]
fn gen_vectors_refuses_output_named_csid_ext() {
    let d = fresh_dir("r1");
    let p = d.join("csid_ext_v0.1.json");
    fs::write(&p, "sentinel\n").unwrap();
    let r = run(&d, &["--output", p.to_str().unwrap()]);
    assert_eq!(r.code, Some(64));
    assert_eq!(r.stdout, "");
    assert_eq!(r.stderr, t1(&p.display().to_string()));
    assert_eq!(fs::read_to_string(&p).unwrap(), "sentinel\n");
    let names: Vec<_> = fs::read_dir(&d)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("csid_ext_v0.1.json")]);
}

#[test]
fn gen_vectors_refuses_bare_csid_ext_name() {
    let d = fresh_dir("r2");
    let r = run(&d, &["--output", "csid_ext_v0.1.json"]);
    assert_eq!(r.code, Some(64));
    assert_eq!(r.stdout, "");
    assert_eq!(r.stderr, t1("csid_ext_v0.1.json"));
    assert_eq!(fs::read_dir(&d).unwrap().count(), 0);
}

#[test]
fn gen_vectors_reports_hard_link_alias() {
    let d = fresh_dir("r3");
    let c = d.join("csid_ext_v0.1.json");
    let p = d.join("v.json");
    fs::write(&c, "x\n").unwrap();
    fs::hard_link(&c, &p).unwrap();
    let r = run(&d, &["--output", p.to_str().unwrap()]);
    assert_t2(&r, &p, &c);
    assert_eq!(fs::read_to_string(&c).unwrap(), CSID_EXT_JSON);
}

#[cfg(unix)]
#[test]
fn gen_vectors_reports_symlink_alias() {
    let d = fresh_dir("r4");
    let c = d.join("csid_ext_v0.1.json");
    let p = d.join("v.json");
    fs::write(&c, "x\n").unwrap();
    std::os::unix::fs::symlink("csid_ext_v0.1.json", &p).unwrap();
    let r = run(&d, &["--output", p.to_str().unwrap()]);
    assert_t2(&r, &p, &c);
    assert_eq!(fs::read_to_string(&c).unwrap(), CSID_EXT_JSON);
}

#[cfg(unix)]
#[test]
fn gen_vectors_reports_reverse_symlink_alias() {
    let d = fresh_dir("r5");
    let c = d.join("csid_ext_v0.1.json");
    let p = d.join("v.json");
    fs::write(&p, "x\n").unwrap();
    std::os::unix::fs::symlink("v.json", &c).unwrap();
    let r = run(&d, &["--output", p.to_str().unwrap()]);
    assert_t2(&r, &p, &c);
    assert_eq!(fs::read_to_string(&p).unwrap(), CSID_EXT_JSON);
}

#[test]
fn gen_vectors_writes_both_corpora() {
    let d = fresh_dir("c1");
    let p = d.join("v0.1.json");
    let c = d.join("csid_ext_v0.1.json");
    let r = run(&d, &["--output", p.to_str().unwrap()]);
    assert_eq!(r.code, Some(0));
    assert_eq!(r.stdout, "");
    assert_eq!(
        r.stderr,
        format!(
            "wrote {} vectors to {} ({} bytes)\nwrote {} csid_ext rows to {} ({} bytes)\n",
            n_vectors(),
            p.display(),
            V0_1_JSON.len(),
            n_rows(),
            c.display(),
            CSID_EXT_JSON.len()
        )
    );
    assert_eq!(fs::read_to_string(&p).unwrap(), V0_1_JSON);
    assert_eq!(fs::read_to_string(&c).unwrap(), CSID_EXT_JSON);
}
