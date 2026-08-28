#!/usr/bin/env bash
# vendor/ freshness guard — the LEADING (PR-time) gate. CODEC (fork-free) form.
#
# REDs iff the committed `vendor/` tree cannot satisfy the current `Cargo.lock`
# under the reproducible build's `--offline --locked` source-replacement config.
# This is the v0.74.0 failure class that hit the toolkit: a dep bump that updates
# Cargo.lock but forgets `cargo vendor vendor/`, so the release `--offline`
# reproducible build can't resolve the bumped dep and publishes NO musl binary.
# That gate is LAGGING (fires only at the release tag); this makes the same
# failure surface on the PR.
#
# Cheap by design: `cargo metadata` does FULL-workspace, all-target resolution
# with NO compile / NO musl toolchain / NO Docker. With vendored-sources
# replacement active, resolution validates EVERY Cargo.lock entry against vendor/
# regardless of target cfg (proven in the toolkit R0 — no musl-only false
# negative). Ported verbatim from mnemonic-toolkit:ci/repro/vendor-freshness.sh.
#
# THREE-BLOCK FORM, and the third block is NOT miniscript. This crate is still
# fork-free of miniscript (which resolves from crates.io), but P3 pinned
# `mnemonic-io-lib` by git rev out of `bg002h/mnemonic-engrave`, so Cargo.lock
# now carries exactly one `source = "git+…"` entry and the two-block config can
# no longer redirect it. The block list is therefore crates-io + that git source
# + vendored-sources.
#
# The rev is DERIVED from Cargo.lock rather than hard-coded, so bumping the pin
# in `crates/mk-cli/Cargo.toml` and re-running `cargo vendor vendor/` is the
# whole of the work — this gate follows automatically. It fails CLOSED if the
# derivation finds nothing, and closed again if a SECOND git source appears,
# because a block list that covers one of two sources would let the other reach
# the live host under `--offline` instead of REDing.
#
# SEE F-341. The TAG-TIME reproducible musl build passes its own block list,
# built by a REUSABLE workflow homed in `mnemonic-toolkit`, and that workflow can
# only ever emit a `rust-miniscript` git block. This gate and that one are now
# out of step, and the fix is not in this repo.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

# Derive the `mnemonic-io-lib` git source from Cargo.lock (authoritative,
# comment-free) so the block list auto-tracks the pin in
# crates/mk-cli/Cargo.toml.
# NOTE the `#<sha>` FRAGMENT. Cargo.lock writes
# `git+<url>?rev=<sha>#<sha>`, while the `[source."…"]` KEY cargo matches on is
# the same string WITHOUT that fragment -- exactly what `cargo vendor` prints.
# A pattern anchored on a closing quote after the rev matches nothing, and the
# fail-closed check below is what turned that into a RED instead of a config
# silently missing its git block. (It did. That is why this comment exists.)
IO_LIB_REV="$(grep -oE '^source = "git\+https://github\.com/bg002h/mnemonic-engrave\?rev=[0-9a-f]{40}' Cargo.lock \
  | head -1 | grep -oE '[0-9a-f]{40}' || true)"
IO_LIB_SOURCE="git+https://github.com/bg002h/mnemonic-engrave?rev=${IO_LIB_REV}"

# Fail CLOSED if the derivation found nothing while a git source is present: a
# missing block would let `--offline` mis-resolve (or reach the live host)
# instead of REDing, which is a FALSE GREEN on exactly the thing this gate
# exists to catch.
GIT_SOURCES="$(grep -cE '^source = "git\+' Cargo.lock || true)"
if [ "$GIT_SOURCES" != "0" ] && [ -z "$IO_LIB_REV" ]; then
  echo "::error::vendor-freshness: Cargo.lock has a git source this config cannot redirect." \
       "Only bg002h/mnemonic-engrave (mnemonic-io-lib) is known here. Add a per-source" \
       "[source] stanza for the new one before this gate can mean anything." >&2
  exit 1
fi
# ...and closed again on a SECOND git source, for the same reason.
if [ "${GIT_SOURCES:-0}" -gt 1 ]; then
  echo "::error::vendor-freshness: Cargo.lock now has ${GIT_SOURCES} git sources; this config" \
       "redirects exactly one. Add a stanza per source." >&2
  exit 1
fi

# Three-block source-replacement: crates-io + the mnemonic-io-lib git source ->
# vendored-sources -> committed vendor/.
SRC_CONFIG=( --config 'source.crates-io.replace-with="vendored-sources"' )
if [ -n "$IO_LIB_REV" ]; then
  SRC_CONFIG+=(
    --config "source.\"${IO_LIB_SOURCE}\".git=\"https://github.com/bg002h/mnemonic-engrave\""
    --config "source.\"${IO_LIB_SOURCE}\".rev=\"${IO_LIB_REV}\""
    --config "source.\"${IO_LIB_SOURCE}\".replace-with=\"vendored-sources\""
  )
fi
SRC_CONFIG+=( --config 'source.vendored-sources.directory="vendor"' )

echo "vendor-freshness: resolving Cargo.lock against committed vendor/ (offline, locked; mnemonic-io-lib rev ${IO_LIB_REV:-none}) ..."
if cargo metadata --format-version 1 --locked --offline "${SRC_CONFIG[@]}" >/dev/null; then
  echo "vendor-freshness: OK — vendor/ satisfies Cargo.lock."
else
  echo "::error::vendor/ is out of sync with Cargo.lock — the --offline --locked reproducible build" \
       "cannot resolve a dependency from the committed vendor/ tree. Run 'cargo vendor vendor/' and" \
       "commit the result (see docs/verify-reproducibility.md). This is the toolkit v0.74.0 release-CI" \
       "failure class, now caught at PR time." >&2
  exit 1
fi
