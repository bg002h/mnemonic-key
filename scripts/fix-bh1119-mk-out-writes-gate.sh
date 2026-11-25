#!/usr/bin/env bash
# Local gate for design/FIX_bh1119_mk_out_writes.md (bh-mk-I1, bh-mk-I6, T7), section 7.
# Run once on the final head, output to a file, then grep it:
#   MK_BASE=<mk binary built before the fix> scripts/fix-bh1119-mk-out-writes-gate.sh > LOG 2>&1; grep -E '^(GATE|FAIL)' LOG
#
# What it does: every step below, each with CARGO_BUILD_JOBS=6 and a host-load
# wait first; one line "GATE <step> OK" or "FAIL <step>" per step; exit 1 if any failed.
#   pin check (cargo search lists 0.1.3); full workspace nextest (expects 552 tests run: 552 passed);
#   mk-codec gen-vectors nextest and clippy; doc tests; fmt on 1.95.0; clippy -D warnings;
#   the 28 bh1119_mk_out_writes tests (28 tests run: 28 passed); rows green on this build and red on MK_BASE;
#   the ci fmt-job steps (docs-only test, shared-ci sha, code-docs check, display-grouping sha);
#   the vendor-freshness workflow steps; the vectors-roundtrip job with a debug build (never --release);
#   cargo check for x86_64-unknown-freebsd.
# What it does NOT cover: the nightly build matrix (stable/beta/1.85 legs), the musl legs (musl-tools), the
# macOS/Windows nightly legs (the R3 non-Unix branch is compiled only there), the toolkit manual, and
# coordinator/scripts/mutation-gate.sh (run by the coordinator before review).
set -uo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS=6
cd "$(git rev-parse --show-toplevel)" || exit 2
: "${CARGO_TARGET_DIR:?set CARGO_TARGET_DIR (never /tmp)}"
: "${MK_BASE:?set MK_BASE to an mk binary built before the fix}"
case "$CARGO_TARGET_DIR" in /tmp*) echo "refusing /tmp" >&2; exit 2;; esac
VEC=${VEC_OUT:-$CARGO_TARGET_DIR/vectors-out}
FAILED=0
wait_load() { until [ "$(cut -d' ' -f1 /proc/loadavg | cut -d. -f1)" -lt 30 ]; do sleep 30; done; }
step() { # name, command...
  local n=$1; shift; wait_load
  if "$@"; then echo "GATE $n OK"; else echo "FAIL $n"; FAILED=1; fi
}
counted() { # name, want-regex, command...  (a short or empty count is a FAIL)
  local n=$1 want=$2; shift 2; wait_load
  local out; out=$("$@" 2>&1); local rc=$?
  echo "$out" | tail -4
  if [ $rc = 0 ] && echo "$out" | grep -Eq "$want"; then echo "GATE $n OK"; else echo "FAIL $n"; FAILED=1; fi
}
vectors_roundtrip() {
  cargo build --locked -q -p mk-cli || return 1
  rm -rf "$VEC"; mkdir -p "$VEC"
  "$CARGO_TARGET_DIR/debug/mk" vectors --out "$VEC" || return 1
  local pinned rebuilt
  pinned=$(jq -S '.vectors | sort_by(.name)' crates/mk-codec/src/test_vectors/v0.1.json)
  rebuilt=$(jq -s -S 'sort_by(.name)' "$VEC"/*.json)
  [ "$pinned" = "$rebuilt" ]
}

step pin-0.1.3 bash -c 'cargo search mnemonic-io-lib --limit 1 | grep -q "^mnemonic-io-lib = \"0.1.3\""'
counted nextest-workspace '552 tests run: 552 passed' cargo nextest run --locked --workspace --build-jobs 6 -j 6 --no-fail-fast
step codec-gen-vectors-nextest cargo nextest run --locked -p mk-codec --features gen-vectors --build-jobs 6 -j 6 --no-fail-fast
step codec-gen-vectors-clippy cargo clippy --locked -p mk-codec --features gen-vectors --all-targets -- -D warnings
step doc-tests cargo test --locked --workspace --doc
step fmt-1.95.0 cargo +1.95.0 fmt --check --all
step clippy cargo clippy --locked --workspace --all-targets -- -D warnings
counted bh1119-tests '28 tests run: 28 passed' cargo nextest run --locked -p mk-cli --build-jobs 6 -j 6 -E 'binary(bh1119_mk_out_writes)'
step rows-green bash scripts/mk-out-writes-rows.sh green
step rows-red-on-base env MK="$MK_BASE" bash scripts/mk-out-writes-rows.sh red
step docs-only-test bash ci/docs-only.test.sh
step shared-ci-sha sha256sum --check --strict ci/shared-ci.sha256
step code-docs-check-test bash ci/code-docs-check.test.sh
step code-docs-check bash ci/code-docs-check.sh
step display-grouping-sha bash -c 'cd design && sha256sum -c display-grouping-vectors.tsv.sha256'
step shared-vendor-sha sha256sum --check --strict ci/repro/shared-vendor.sha256
step vendor-freshness bash ci/repro/vendor-freshness.sh
step vendor-freshness-selftest bash ci/repro/vendor-freshness.test.sh
step vectors-roundtrip vectors_roundtrip
if rustup target list --installed | grep -q '^x86_64-unknown-freebsd$'; then
  step freebsd-check cargo check --locked --target x86_64-unknown-freebsd -p mk-cli
else
  echo "GATE freebsd-check NOT RUN (target not installed)"
fi
[ "$FAILED" = 0 ] && echo "GATE ALL OK" || { echo "FAIL gate"; exit 1; }
