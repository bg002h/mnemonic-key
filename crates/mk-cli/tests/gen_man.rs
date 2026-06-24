//! Integration tests for `mk gen-man`.
//!
//! `gen-man` self-emits roff man pages from the compiled clap `Command` tree
//! via `clap_mangen::generate_to(Cli::command(), out_dir)` — the bare, naive
//! call with NO pre-`.build()` (a pre-build poisons the output with a `help`
//! pseudo-subcommand shadow tree; see SPEC §2 / C-1).
//!
//! The pages are clap-generated, hence binary-faithful by construction — there
//! is no content-fidelity gate (SPEC §1 non-goals). These tests assert the
//! page-SET shape, not prose.

use std::collections::BTreeSet;
use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

/// Run `mk gen-man --out <dir>` into a fresh temp dir and return the sorted
/// set of produced `*.1` filenames.
fn generate_pages(dir: &std::path::Path) -> BTreeSet<String> {
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .arg("gen-man")
        .arg("--out")
        .arg(dir)
        .output()
        .expect("invoke mk gen-man");
    assert!(
        out.status.success(),
        "mk gen-man failed: status={:?} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let mut names = BTreeSet::new();
    for entry in std::fs::read_dir(dir).expect("read out dir") {
        let entry = entry.expect("dir entry");
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".1") {
            names.insert(name);
        }
    }
    names
}

/// `gen-man` produces a non-empty set of `*.1` pages.
#[test]
fn gen_man_produces_non_empty_page_set() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pages = generate_pages(tmp.path());
    assert!(!pages.is_empty(), "expected at least one *.1 page");
}

/// Every produced page is a non-empty file carrying a `.TH` roff header.
#[test]
fn every_page_has_a_th_header() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pages = generate_pages(tmp.path());
    for name in &pages {
        let body = std::fs::read_to_string(tmp.path().join(name)).expect("read page");
        assert!(!body.is_empty(), "page {name} is empty");
        assert!(
            body.contains(".TH"),
            "page {name} lacks a .TH roff header; head:\n{}",
            &body[..body.len().min(200)]
        );
    }
}

/// The root page `mk.1` exists.
#[test]
fn root_page_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pages = generate_pages(tmp.path());
    assert!(
        pages.contains("mk.1"),
        "expected root page mk.1 in {pages:?}"
    );
}

/// EXACT page-set walk: the produced set equals the set derived from the
/// live `gui-schema` subcommand list (the same unbuilt clap tree, minus the
/// auto-`help` and `is_hide_set()` subs — mk hides none) plus the new
/// `gen-man` page and the root. `mk` has NO nested subcommands, so every page
/// is either the root `mk.1` or a top-level `mk-<sub>.1`.
///
/// Derives the expected set from the live tree (via `gui-schema`) — NO magic
/// integer baked into the test (SPEC §3 / M-2). NOTE: `gui-schema` excludes
/// `gui-schema` itself from its JSON list, but the man tree DOES emit a
/// `mk-gui-schema.1` page (it is visible / not `is_hide_set`), so the
/// expected set re-adds `gui-schema` and `gen-man`.
#[test]
fn exact_page_set_matches_live_command_tree() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let produced = generate_pages(tmp.path());

    // Live subcommand names from `mk gui-schema` (excludes `gui-schema`).
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let schema_out = cmd.arg("gui-schema").output().expect("invoke gui-schema");
    assert!(schema_out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&schema_out.stdout).expect("gui-schema JSON");
    let subs = v["subcommands"].as_array().expect("subcommands array");

    let mut expected: BTreeSet<String> = BTreeSet::new();
    expected.insert("mk.1".to_string());
    for s in subs {
        let n = s["name"].as_str().expect("name string");
        expected.insert(format!("mk-{n}.1"));
    }
    // `gui-schema` excludes itself from its JSON, but it is a visible
    // subcommand → it DOES get a man page. `gen-man` is new and likewise visible.
    expected.insert("mk-gui-schema.1".to_string());
    expected.insert("mk-gen-man.1".to_string());

    assert_eq!(
        produced, expected,
        "produced man-page set does not match the live command tree"
    );
}

/// NEGATIVE canary (SPEC §8 / C-1): the naive `generate_to` call (no pre-build)
/// must emit ZERO `*-help*.1` shadow pages. A `*-help.1` or `*-help-*.1` file
/// is the tripwire for an accidental future pre-`.build()` regression.
#[test]
fn no_help_shadow_pages() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let pages = generate_pages(tmp.path());
    for name in &pages {
        let stem = name.strip_suffix(".1").unwrap_or(name);
        assert!(
            stem != "mk-help" && !stem.contains("-help-") && !stem.ends_with("-help"),
            "unexpected help shadow page {name:?} — a pre-.build() regression (C-1)"
        );
    }
}
