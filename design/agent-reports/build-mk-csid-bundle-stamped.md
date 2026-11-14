**Resolved by the Follow-on below: the STOP (Important) was cured by the 0.16.0-dev bump; no open Critical or Important.**

# Build report: mk FU1, bundle-stamped chunk-set id stays silent (Option A)

- Model: Claude Sonnet 5.5, claude-sonnet-5-5
- Base: 89192ac706e0dc6d5a4c68f83a56ccc4ecf5d904 (origin/main, unmoved at start; `git apply --check` of the diff passed)
- Head: see the final summary (the report is committed last)
- PR: see the final summary

## RED (commit e94833f, unmodified src)
    FAIL verify_json_names_a_bundle_stamped_id
    FAIL bundle_card_is_silent_on_every_read_verb
    FAIL repair_of_a_bundle_card_does_not_warn
    FAIL bundle_cards_at_slots_1_16_31_are_silent
    PASS bundle_card_inspect_still_prints_the_stamped_id   (guard)
    PASS a_pin_one_past_the_last_slot_still_warns          (guard)
    Summary 6 tests run: 2 passed, 4 failed   (log: /scratch/cache/mk-csid-logs/red.log)

## GREEN (commit 7a6d698)
    Summary 6 tests run: 6 passed, 0 skipped   (log: /scratch/cache/mk-csid-logs/green.log)
`cargo fmt` (pinned 1.85.0) changed nothing.

## Tests that changed
None edited; one file added (`crates/mk-cli/tests/csid_bundle_stamped.rs`).

## Text deltas
SPEC contract 2 insert (silent) and contract 4 `bundle_stamped`; BIP :327 append; `crates/mk-cli/CHANGELOG.md` [Unreleased] ### Changed (the brief's "CHANGELOG.md" for mk-cli); FOLLOWUPS Q4 entry (status `fixed in this PR`).
Note: the grade gives the contract 4 delta only as "add the `bundle_stamped` field"; I extended the JSON example with `"bundle_stamped": false` and one clause saying it is true for a contract-2 id and that `matches` keeps its literal meaning. Wording is mine, from the grade's Q2 ruling; see STOPs.

## STOPs
- Contract 4 wording (above): the grade did not give exact sentence text. Minor; coordinator to confirm or reword.

## Gate
Logs in /scratch/cache/mk-csid-logs/. Toolchain 1.85.0 (pinned); fmt job on 1.95.0.
- ci build: `cargo build --workspace` rc 0; `cargo nextest run --workspace --no-fail-fast` **415 run, 414 passed, 1 FAILED** (below); `cargo test --workspace --doc` rc 0; `cargo clippy --workspace --all-targets -- -D warnings` rc 0
- release.yml test job: `cargo nextest run --locked --workspace --no-fail-fast` 415 run, 414 passed, 1 FAILED (same test); `cargo test --locked --workspace --doc` rc 0
- ci fmt job: docs-only.test.sh rc 0; shared-ci sha + code-docs-check(.test) rc 0; vector checksum pin ok; `cargo +1.95.0 fmt --check --all` rc 0
- vectors-roundtrip: release build + `mk vectors --out` + jq diff: VEC_OK
- freebsd `cargo check --target x86_64-unknown-freebsd -p mk-cli`: rc 0
- NOT RUN: musl legs (no musl-gcc here; aarch64 leg needs an arm runner); macOS/Windows build legs and release.yml `build mk` matrix; the stable/beta toolchain legs of ci build; fuzz-smoke and vendor-freshness (path filters not touched by this diff); mutation gate (coordinator).

## STOP (Important): an existing test changes outcome
`mk-cli::version_is_honest the_version_is_either_a_released_heading_or_a_named_dev_build` FAILS on the head:
"mk-cli 0.15.0 is a release version, but `## [Unreleased]` lists changes it does not contain -- bump to the next `-dev` version".
Cause: the brief's required CHANGELOG entry under `[Unreleased]` while the crate version is the release `0.15.0`. The test passes on main (empty Unreleased). The cure is a version bump to a `-dev` version (0.15.1-dev or 0.16.0-dev; `[Unreleased]` must name the next release, and Cargo.lock follows), but the brief forbids a version bump and the grade does not decide the number. Undecided: the coordinator must choose. Because of this I did NOT push and did NOT open the PR (it would be red). Branch is local only.

**⚠ Important: 1 (above)**
Counts: Critical 0, Important 1, Minor 1 (contract 4 wording)

## Follow-on (version bump, re-gate, push, draft PR)
- Model: Claude Sonnet 5.5, claude-sonnet-5-5. Base 89192ac706e0dc6d5a4c68f83a56ccc4ecf5d904 (origin/main unmoved before and after the push). PR: #26 (draft), https://github.com/bg002h/mnemonic-key/pull/26
- Bump commit a5d6f1e2acffd00f3f044948de63c98565fabcfb: mk-cli 0.16.0-dev; Cargo.lock diff is the one mk-cli version line.
- Beyond the brief: the first re-gate still failed `version_is_honest` ("`## [Unreleased]` does not name 0.16.0 as the next release"). The test and precedent 2f05088 require a `Next release: 0.16.0` line under `[Unreleased]`; I added exactly that line to `crates/mk-cli/CHANGELOG.md` and amended it into the bump commit (never pushed before). The brief did not mention it; the coordinator may check it.
- Gate on the bump head (logs /scratch/cache/mk-csid-logs/fu2/): build rc 0; `cargo nextest run --workspace --no-fail-fast` 415 run, 415 passed; locked release.yml form 415 run, 415 passed; doc-tests both forms rc 0; clippy -D warnings rc 0; docs-only.test, shared-ci sha, code-docs-check, vector checksum, `cargo +1.95.0 fmt --check` rc 0; vectors-roundtrip ok; FreeBSD `cargo check` rc 0.
- NOT RUN: as above (musl, macOS/Windows, release.yml build matrix, stable/beta legs, fuzz-smoke, vendor-freshness, mutation gate).
- SPEC contract 4 wording stands for the diff review.
- STOPs: none.

Counts: Critical 0, Important 0, Minor 1 (contract 4 wording)
