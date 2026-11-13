# Build report: mk output sink (STOPPED before push)

Start head 1e112ce3ac59315a743772714f8a38c337690725. Branch fix/mk-output-sink.

**⚠ STOP: CI job `fmt` step "Shared CI files, and no code names a docs file" (`bash ci/code-docs-check.sh`) exits 1 on this head.** The two hits are in the base-committed `scripts/mk-output-sink-fix-gate.sh` (L26 names `design/FIX_mk_output_sink.md`, L105 names `design/FOLLOWUPS.md`). The build changed neither the script nor `ci/`. Resolution (a `- ` rule in `ci/docs-only-paths.txt`, an entry in `ci/code-docs-exempt.txt`, or rewording the script) is a decision; not made. Not pushed, no PR.

## Applied
- RED commit: `rows-combined-i3.diff` (crates/mk-cli/tests/output_errors.rs).
- GREEN commit 2f05088: `fix.diff` (Cargo.lock, mk-cli CHANGELOG.md, Cargo.toml, main.rs, out.rs, process_hardening.rs) and the `diff followups` block (design/FOLLOWUPS.md). The document holds no `<COMMIT>` placeholder, so no substitution commit.

## RED (section 6 commands as printed; logs red-*.log, red-set.log)
Each of the six rows exited 100 with its `## expect:` substring present; the scan row exited 0. Whole file: `7 tests run: 1 passed, 6 failed`.

## GREEN
`7 tests run: 7 passed`; workspace `406 tests run: 406 passed`.

## Gate (logs under /scratch/cache/build-fix-output-sink/mk/, summary.txt)
rc 0: s7-nextest (406/406), s7-doc, s7-fmt, s7-clippy, s7-rows (7/7), ci-build, ci-test (406/406), ci-doctest, ci-clippy, ci-fmt (1.95.0), ci-docsonly-test, ci-vecsum, ci-freebsd, rel-test (406/406), rel-doc, ci-vectors (debug build, not --release), rel-smoke (debug build).
rc 1: ci-shared (sha256 check, code-docs-check.test.sh and docs-only checks OK; code-docs-check.sh FAIL, see STOP).

## Left to CI
macOS and Windows legs, release.yml cross/aarch64 builds, assemble/sign, musl (tag only), docs-only.sh (needs GitHub context), fuzz-smoke, vendor-freshness (not run), release-mode vectors-roundtrip and smoke (run on debug builds here).

Counts: Critical 0, Important 1 (the STOP), Minor 0.

## Finish (2026-11-13)
Ruling (coordinator brief build-fix-output-sink-finish.md): the local gate script is run by hand and by no CI step, so it is listed in `ci/code-docs-exempt.txt`, following that file's own precedent. Commit 6c4ea3d appends the ruled entry (that file alone; the script, the document and `.github/` are untouched).
Check commands (logs f-*.log under /scratch/cache/build-fix-output-sink/mk/): `sha256sum --check --strict ci/shared-ci.sha256` rc 0; `bash ci/code-docs-check.test.sh` rc 0; `bash ci/code-docs-check.sh` rc 0 (98 files scanned); `bash ci/docs-only.test.sh` rc 0.
The STOP above is closed. All other results above are from the first builder's logs. The draft PR is open (see the PR for its number); it names toolkit companion PR bg002h/mnemonic-toolkit#158 (merged, toolkit master 67470572) and does not merge before it.

## Whole-diff fold (2026-11-13)
Start head e6d533f4. Folded: the access-mode-3 port of the ms fix, and the s2 review's I1.
- Port: RED e47c538 `no_access_mode_stdout_exits_1_with_the_line`: `1 test run: 0 passed, 1 failed, 199 skipped` (left: Some(0), right: Some(1)). GREEN 8f361b2 (probe `!matches!(fl & libc::O_ACCMODE, libc::O_WRONLY | libc::O_RDWR)`, doc comment): `1 test run: 1 passed, 199 skipped`.
- I1 b8f2131 `read_write_stdout_keeps_the_output_and_exits_0` (row verbatim from the report): RED on a scratch plant (the probe with `| libc::O_RDWR` removed, reverted, never committed): `1 test run: 0 passed, 1 failed`, left: Some(1), right: Some(0); GREEN on the head: `1 test run: 1 passed, 200 skipped`.
- Filed c242666 (deferred-minor, owner mk-output-sink-followups): `mk-output-sink-wd-m1-flush-unpinned`, `-m2-closed-early-return-unpinned`, `-m3-sigxfsz-file-size-limit` (run here: `ulimit -f 1; mk gui-schema > file` exits 153, so it holds), `-n1-advisory-after-failed-stdout` (s1 Nit 1). Not filed: s2 N1 (already filed), s1 Nit 2.
- Gate (logs /scratch/cache/fold-output-sink-accmode/mk/logs/): rc 0 for nextest (408 tests run: 408 passed; was 406), doc tests, `cargo +1.95.0 fmt --check --all`, clippy `--workspace --all-targets -D warnings`, rows file (9 tests run: 9 passed; was 7), `ci/code-docs-check.sh` (98 files scanned), freebsd `cargo check --target x86_64-unknown-freebsd -p mk-cli`.
- The FIX document (counts 7 / 406) and its gate script are unedited; they predate these rows.
- STOPs: none.
