# Build report: mk bug-hunt FIX + toolkit manual lockstep

No STOP. All gates green. Two judgment notes below (Tier values, flag-lint baseline).

## Commits (not pushed)
- mk `fix/bughunt`: c592b92 (GREEN: spec + production + fmt), 6e2d22b (followups).
- toolkit `fix/bughunt-mk-manual` (worktree /scratch/code/shibboleth/tk-worktrees/bughunt-mk-manual, from origin/master f6355e22): 0637fca "manual: mk bug-hunt lockstep (44-mk-cli.md)" (5 rows).

## RED / GREEN (logs in design/agent-reports/build-mk-bughunt-logs/, uncommitted)
- RED `red.log`: 8 run, 3 passed, 5 failed, 391 skipped (as §4).
- Step rows: `s2.log` row 05 x2 pass; `s3-5.log` rows 01/03/04 (4 tests) pass; `mustgreen.log` 11/11.
- Gate: `full.log` 399 run, 399 passed; `filter.log` 8 run, 8 passed; `doc.log` ok (0 doctests); `fmt.log` clean; `clippy.log` clean (-D warnings).
- Mutation gate not run (controller's).

## Toolkit manual job (local)
- `ci/doc-gate-guard.test.sh`: pass. `cargo build --bin mnemonic`: ok.
- `make audit` (this build's mk 0.14.0-dev): exit 2 from lint only. markdownlint 0 errors; the failures are `(unreleased: mk-cli after 0.13.0: ...)` markers "does not name the pinned release (0.14.0)". Same failure set on unchanged f6355e22 (`tk-audit-base.log`): PRE-EXISTING, no new failure.
- `ci/doc-flag-lint.test.sh` (needs `*_BIN` env): 11 passed, 4 failed (baseline, R4 reverse, R6 reverse, G0), identical on f6355e22 (`tk-flaglint-base.log`): PRE-EXISTING (same 4 names; marker-line order differs between runs only).
- `make pdf` skipped (not run; not part of the brief's list).
- Note: a stray toolkit `target/` build dir exists in the tk worktree (untracked/ignored); harmless.

## Deviations / judgment
- FOLLOWUPS entries use `**Tier:**` values from the file's tier list: `v1+` for the M1 and stdout items, `cross-repo` for the GUI item; the brief gave owners but no tier, so these are my pick. Change if wrong.
- Gate also confirms `cargo fmt` reformatted the §3.3 insertions as review N1 predicted.

399/399 nextest; filter 8/8; RED 8 run / 3 passed / 5 failed.
