# Build report: mk-remedies, the `mk address` multisig pointer (item R)

- Base: origin/main cea6c70bc940616aa302a33efff253932113ace4 (did not move before the build).
- Branch: fix/mk-remedies. Head: see the PR and the final commit (this report is the last commit).
- PR: filled in below after the push.

## RED
Row `multisig_advisory_prints_the_md_address_route` (crates/mk-cli/tests/cli_address.rs), run on the unmodified base:
`cargo nextest run --locked -j 8 -p mk-cli --test cli_address multisig_advisory_prints_the_md_address_route`
Result: `Summary [   0.008s] 1 test run: 0 passed, 1 failed, 15 skipped` (left: "mnemonic verify-bundle", right: "md address <MD1>... --from-mk1-file FILE").

## GREEN
Same file after applying mk-fix.diff and the SPEC copy: `Summary [   0.023s] 16 tests run: 16 passed, 0 skipped`.

## Executed evidence (pointer row, grader's md binary)
`bash /scratch/cache/grade-md-mk-remedies/rows/mk_pointer_row.sh <worktree>/target/debug/mk`:
```
mk address rc=64
printed: md address <MD1>... --from-mk1-file FILE
ran: md address md1ypfds3cf... rc=0 bc1q3x9sy9cfx7y90rcscpqhad37fwzwk4ernn802cs9737ngmq224wspuk4z8 note: composed wallet id c147b29e · policy shape id aad0e0e0
PASS
```

## Tests that changed
None existing pinned the old text (grep for `verify-bundle` / `descriptor tooling` in tests: none). One row added.

## Other files
CHANGELOG (crates/mk-cli, Unreleased, Changed), design/SPEC_mk_derive_address.md:43, design/FOLLOWUPS.md (new `policy-id-stub-256-refused` deferred-minor; p2tr measurement appended to F3 entry `crash-sweep-mk-address-json-memory`; new `mk-mutants-audit-owed`).
The SPEC copy no longer mentions "(M6)" and the `mnemonic convert --to address` hatch: that is the grade's text verbatim.

## STOPs
None.

## Gate (logs in /scratch/cache/mk-remedies-logs/, run on the head before this report commit; the report is the only later change)
The diff touches code, so no docs-only skip applies. Result lines:
- ci.yml fmt job: docs-only.test rc=0; shared-ci files + code-docs-check rc=0; conformance checksum rc=0; `cargo +1.95.0 fmt --check --all` rc=0
- ci.yml build job (run once, on the pinned 1.85.0 toolchain): `cargo build --workspace` rc=0; `cargo nextest run --workspace --no-fail-fast`: `409 tests run: 409 passed (1 slow), 0 skipped`; `cargo test --workspace --doc` rc=0; `cargo clippy --workspace --all-targets -- -D warnings` rc=0
- ci.yml vectors-roundtrip: `cargo build -p mk-cli --release` rc=0; `mk vectors --out`; `vectors-roundtrip OK`
- ci.yml freebsd-compile-gate: `cargo check --target x86_64-unknown-freebsd -p mk-cli` rc=0
- release.yml test job: `cargo nextest run --locked --workspace --no-fail-fast`: `409 tests run: 409 passed (1 slow), 0 skipped`; `cargo test --locked --workspace --doc` rc=0
- fuzz-smoke.yml: triggers on pull_request paths fuzz/** and crates/mk-codec/src/**; this diff touches neither; does not run.
- vendor-freshness.yml: paths Cargo.*, vendor/**, ci/repro/**; not touched; does not run.
- musl-binaries.yml: tag/dispatch only; does not run.

## NOT RUN
- ci.yml build matrix legs stable, beta, macos, windows (only 1.85.0 on Linux run locally; the nextest/doc/clippy lines above are that leg).
- ci.yml musl-check (x86_64 and aarch64 musl; needs musl-tools and an arm64 runner).
- release.yml build matrix (macos, windows, cross targets; remote runners) and test job on macos/windows.
- ci.yml release-on-tag: tag only.
- cargo-nextest 0.9.140 locally, workflow pins 0.9.146.

Counts: Critical 0, Important 0, Minor 0 (STOPs 0).
