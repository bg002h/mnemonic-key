//! Creating the file `--out` writes into (SPEC §6b, F-244).
//!
//! **Byte-for-byte duplicate of `mnemonic_io_lib::write::write_private`** at
//! `bg002h/mnemonic-engrave` rev `6c24e62`, for the same reason
//! `output_advisory.rs` duplicates the toolkit's advisory: `mk-cli` cannot take
//! the dependency.
//!
//! P3's boundary table rules that `mk` ADOPTs that crate item, reached by a git
//! rev pin. Measured while executing it, the pin is not available to this repo:
//!
//!   * `mnemonic-key` commits a `vendor/` tree, and `ci/repro/vendor-freshness.sh`
//!     fails CLOSED on any git source in `Cargo.lock`. That half IS fixable here
//!     -- `cargo vendor` adds one 80K directory with zero churn, and the
//!     three-block offline resolve exits 0.
//!   * The half that is not: `mk`'s reproducible-musl gate is a REUSABLE
//!     workflow homed in a fourth repository,
//!     `bg002h/mnemonic-toolkit/.github/workflows/reproducible-musl-build.yml@6e37b18`,
//!     called from `.github/workflows/musl-binaries.yml`. It can redirect
//!     exactly one git source, `rust-miniscript`, interpolated from its
//!     `miniscript_rev` input. An `mnemonic-engrave` git source cannot be
//!     redirected there without editing that shared workflow -- which
//!     `descriptor-mnemonic` calls too. That is a cross-repo join, and the gate
//!     is LAGGING: it fires at the next `mk-cli-v*` tag, not on this push.
//!
//! So the pin would leave a release-time gate that cannot pass, months after
//! anyone holds this cycle's context. The behaviour §6b and F-244 require does
//! not depend on where the function lives, and this file makes it identical.
//!
//! **Swap this for the crate** when `mnemonic-io-lib` reaches crates.io (a
//! registry source passes both gates unchanged) or when the shared workflow
//! gains a generic git-source redirect. Filed as F-341.
//!
//! ## Why `0o600` is a constant and not a parameter
//!
//! The mode a tool creates *its own* output at is not the question the crate
//! declines to settle (`me` rules `0o044` disqualifying, `mt` rules `0o077` --
//! that is a disagreement about somebody else's file). A file `mk` creates for
//! an artifact is owner-only, and a parameter added speculatively would be the
//! first place a caller could weaken it by accident.
//!
//! ## F-244 -- the half that is easy to leave out
//!
//! `OpenOptions::mode()` binds **on create only**. Open an existing file with it
//! and the mode on disk is untouched, so `--out stale.mk1` over a target already
//! at `0644` would leave it at `0644` -- and that is the case an operator
//! re-running a command actually hits.
//!
//! The `set_permissions` call below is made on the **open file** rather than on
//! the path. A path-based `set_permissions` names the file a second time, and
//! between the two calls the name can be made to point somewhere else; a handle
//! cannot be redirected once it is open.

/// Write `bytes` to `path`, creating or truncating it owner-only.
///
/// `truncate(true)` is load-bearing and not a stylistic echo of
/// `std::fs::write`: without it a **shrinking** overwrite -- a two-chunk card
/// written over a five-chunk one -- leaves the tail of the old file in place,
/// so the target holds chunks from two different cards.
///
/// On non-Unix the mode calls compile out and the create/truncate semantics
/// remain. The threat model is POSIX: mode bits do not mean the same thing
/// elsewhere, and pretending otherwise would be worse than saying so.
pub fn write_private(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut f = opts.open(path)?;
    // F-244. `opts.mode()` above did nothing if `path` already existed, so the
    // mode is set a second time on the OPEN FILE -- see the module header for
    // why the handle and not the path.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    f.write_all(bytes)
}

#[cfg(all(test, unix))]
mod tests {
    use super::write_private;
    use std::os::unix::fs::PermissionsExt;

    fn mode_of(p: &std::path::Path) -> u32 {
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    /// **THE GATE, and the pre-existing target is the whole of it.**
    ///
    /// The fresh-file half passes under any implementation that sets a mode at
    /// all, so on its own it proves nothing. The `0644` half is what fails
    /// against `OpenOptions::mode()` alone.
    ///
    /// It also pins the CONTENTS, because a function that tightened the mode and
    /// wrote nothing would satisfy a permissions-only test.
    #[test]
    fn an_existing_world_readable_target_is_tightened_not_inherited() {
        let d = tempfile::tempdir().unwrap();

        let fresh = d.path().join("fresh.mk1");
        write_private(&fresh, b"mk1fresh\n").unwrap();
        assert_eq!(mode_of(&fresh), 0o600);
        assert_eq!(std::fs::read(&fresh).unwrap(), b"mk1fresh\n");

        let stale = d.path().join("stale.mk1");
        std::fs::write(&stale, b"OLD OLD OLD OLD OLD\n").unwrap();
        std::fs::set_permissions(&stale, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(mode_of(&stale), 0o644, "precondition");

        write_private(&stale, b"mk1new\n").unwrap();
        assert_eq!(mode_of(&stale), 0o600, "F-244: tightened, not inherited");
        // And truncated: the old, LONGER content leaves no tail behind.
        assert_eq!(std::fs::read(&stale).unwrap(), b"mk1new\n");
    }
}
