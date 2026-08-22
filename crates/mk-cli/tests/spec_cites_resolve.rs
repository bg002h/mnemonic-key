//! Every `SPEC §x.y` citation in the source must resolve to a real heading.
//!
//! This class has recurred twice. A 2026-06-10 audit found four `SPEC §3.5.1`
//! cites pointing at a section that does not exist and repointed them; on
//! 2026-08-21 an independent review found a fifth (`§3.5.4`), and sweeping the
//! whole crate found EIGHT. The mk SPEC's §3.5 is "Origin path encoding" and
//! has no subsections at all, so the entire `§3.5.x` family referred to
//! nothing — and several cited CLI surfaces the format spec never covered.
//!
//! Fixing the one instance a reviewer happened to notice is what left seven
//! behind the first time. This test is the check that makes the class stay
//! closed, so it costs a command instead of a discipline.
//!
//! Cross-document cites (`md SPEC §8.1`) are deliberately out of scope: they
//! name a different repository's spec, which this test cannot resolve.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Section numbers the mk SPEC actually defines, from its headings.
fn spec_sections() -> HashSet<String> {
    let spec = std::fs::read_to_string(repo_root().join("design/SPEC_mk_v0_1.md"))
        .expect("SPEC_mk_v0_1.md must be readable");
    let mut out = HashSet::new();
    for line in spec.lines() {
        let t = line.trim_start_matches('#').trim_start();
        let t = t.strip_prefix('§').unwrap_or(t);
        if !line.starts_with('#') {
            continue;
        }
        let num: String = t
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let num = num.trim_end_matches('.').to_string();
        if !num.is_empty() {
            out.insert(num);
        }
    }
    assert!(out.contains("3.3"), "sanity: the spec defines §3.3");
    assert!(
        !out.contains("3.5.4"),
        "sanity: the spec does NOT define §3.5.4"
    );
    out
}

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/mk-cli.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repo root")
        .to_path_buf()
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).expect("readable dir").flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rust_sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn every_spec_citation_resolves_to_a_real_section() {
    let sections = spec_sections();
    let mut files = Vec::new();
    rust_sources(&repo_root().join("crates"), &mut files);
    assert!(files.len() > 10, "sanity: found {} sources", files.len());

    let mut bad: Vec<String> = Vec::new();
    for f in &files {
        // This file's own documentation necessarily quotes the cites it exists
        // to forbid.
        if f.file_name().is_some_and(|n| n == "spec_cites_resolve.rs") {
            continue;
        }
        let text = std::fs::read_to_string(f).expect("readable source");
        for (i, line) in text.lines().enumerate() {
            // A line that DESCRIBES a retired cite has to quote it. Opt out
            // explicitly rather than by guessing at phrasing -- the first
            // draft used a phrase blocklist and still flagged two of its own
            // explanatory comments.
            if line.contains("SPEC-CITE-EXEMPT") {
                continue;
            }
            // Only THIS spec's cites. A bare `SPEC §x` or an explicit
            // `SPEC_mk_v0_1.md §x`. Anything else -- `md SPEC §8.1`,
            // `SPEC_v0_11_wire_format.md §1.4` -- names a different document,
            // often in another repository, which this test cannot resolve and
            // must not flag. (Both forms are present and correct in-tree; the
            // first draft of this test reported them as phantoms.)
            for pat in ["SPEC §", "SPEC_mk_v0_1.md §"] {
                for (idx, _) in line.match_indices(pat) {
                    // `md SPEC §` / `SPEC_other.md §` end with a different token.
                    let before = &line[..idx];
                    if pat == "SPEC §"
                        && (before.trim_end().ends_with("md") || before.ends_with('_'))
                    {
                        continue;
                    }
                    let rest = &line[idx + pat.len()..];
                    let num: String = rest
                        .chars()
                        .take_while(|c| c.is_ascii_digit() || *c == '.')
                        .collect();
                    let num = num.trim_end_matches('.');
                    if num.is_empty() {
                        continue;
                    }
                    if !sections.contains(num) {
                        bad.push(format!(
                            "{}:{} cites SPEC §{num}, which the SPEC does not define",
                            f.strip_prefix(repo_root()).unwrap_or(f).display(),
                            i + 1
                        ));
                    }
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "phantom SPEC citations:\n  {}",
        bad.join("\n  ")
    );
}
