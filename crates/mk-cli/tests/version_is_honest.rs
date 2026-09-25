//! F-677: a binary built from `main` reported `mk 0.13.0` while carrying flags
//! the 0.13.0 release does not have (`--in`, `--out`, `--keys`), so a main
//! build could pass for the release.
//!
//! The rule, checked against this crate's own CHANGELOG:
//! - a RELEASE version `X.Y.Z` has its own `## [X.Y.Z]` heading, and NO
//!   unreleased entries -- a non-empty `## [Unreleased]` beside a release
//!   version is exactly the F-677 state (0.13.0 plus 83 commits of changes);
//! - an UNRELEASED version is `X.Y.Z-dev`, and the `## [Unreleased]` section
//!   names `X.Y.Z` as the next release.
//!
//! What it cannot see: a change that lands after a release without a CHANGELOG
//! entry. That needs git, and is the release checklist's job.

use assert_cmd::Command;

const CHANGELOG: &str = include_str!("../CHANGELOG.md");

#[test]
fn the_version_is_either_a_released_heading_or_a_named_dev_build() {
    let v = env!("CARGO_PKG_VERSION");
    let unreleased: Option<&str> = CHANGELOG
        .split("\n## [Unreleased]")
        .nth(1)
        .and_then(|s| s.split("\n## [").next());
    match v.split_once('-') {
        None => {
            assert!(
                CHANGELOG.contains(&format!("\n## [{v}]")),
                "mk-cli {v} is a release version with no `## [{v}]` CHANGELOG heading"
            );
            assert!(
                unreleased.is_none_or(|s| s.trim().is_empty()),
                "mk-cli {v} is a release version, but `## [Unreleased]` lists \
                 changes it does not contain -- bump to the next `-dev` version"
            );
        }
        Some((base, pre)) => {
            assert_eq!(pre, "dev", "unexpected pre-release suffix in {v}");
            let unreleased = unreleased.expect("a -dev version needs a `## [Unreleased]` section");
            assert!(
                unreleased.contains(&format!("Next release: {base}")),
                "`## [Unreleased]` does not name {base} as the next release"
            );
            assert!(
                !CHANGELOG.contains(&format!("\n## [{base}]")),
                "{base} already has a release heading, so {v} is behind it"
            );
        }
    }
}

/// `mk --version` prints exactly the crate version, suffix included.
#[test]
fn mk_version_prints_the_crate_version() {
    let out = Command::cargo_bin("mk")
        .unwrap()
        .arg("--version")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        format!("mk {}", env!("CARGO_PKG_VERSION"))
    );
}
