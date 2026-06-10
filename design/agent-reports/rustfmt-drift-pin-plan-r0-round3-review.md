# R0 round-3 architect review — PLAN_rustfmt_drift_pin (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 3, post-fold verification of round 2). main @ 13821ad. Verdict: GREEN (0 Critical / 0 Important / 1 new Minor M8; I3 + M5/M6/M7 all FOLDED-OK with zero drift). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

- **M8 (NEW, same move-class as I3, non-blocking) — `README.md:50` still links the pre-move corpus path.** `[`crates/mk-codec/tests/vectors/v0.1.json`](crates/mk-codec/tests/vectors/v0.1.json)` is a live markdown link to a path that does not exist (404 on GitHub since `33f2ca2`, 2026-05-15). Verified this is the ONLY remaining live-surface stale reference: a full-repo grep for `tests/vectors` outside `src/test_vectors` hits only (a) `ci.yml:94` (= I3, folded), (b) this README link, and (c) historical artifacts that legitimately describe the old layout — resolved FOLLOWUPS entries (`mk-cli-vector-corpus-inlined` is `resolved 33f2ca2`; its prose is a correct historical record), CHANGELOG entries, frozen design/agent-report docs, and `tests/vectors.rs` self-references (that FILE still exists; only the JSON moved — `VECTOR_FILE` at `vectors.rs:55` correctly reads `src/test_vectors/v0.1.json`). Doc-only, gates nothing, blocks nothing. Cheapest disposal: one more line in the same chore commit (the README sentence's "v0.1.1" crate-version staleness is a separate, larger README-currency question — do NOT scope-creep into it; fixing just the link path is safe). Acceptable alternative: file a FOLLOWUP. Either way this does not block implementation.

## Fold-verification

- **I3 → D2(d) (`:22`) + `:30` date: FOLDED CORRECTLY AND COMPLETELY — and this round EXECUTED the unmasked job to prove the fold is also SUFFICIENT.**
  - Path: `ci.yml:94` re-verified live = `crates/mk-codec/tests/vectors/v0.1.json`; that path does not exist; `crates/mk-codec/src/test_vectors/v0.1.json` exists (34,162 bytes, the SHA-pinned corpus). Plan's target path is exact.
  - Mechanism: all present at `:22` — stale pre-move path, `33f2ca2` move attribution, same-move-class link to `47de269`, the masking chain (`needs: build` → skipped on every run since the fmt red; `ci.yml:66` re-verified), "last green run 2026-05-13 PREDATES the move / has NEVER executed against the moved corpus", the unmask consequence (next CI red + release-blocking via (c)).
  - Same-commit instruction: present verbatim ("Fix in the same commit: `ci.yml:94` → `crates/mk-codec/src/test_vectors/v0.1.json`").
  - Date correction: `:30` now reads "first all-green mnemonic-key run since 2026-05-13 (`eb7e8b1` — every later run was red or had skipped jobs; R0-r2 I3 corrected the date)". Spot-checked `gh run list`: every run 2026-05-30 → 2026-06-10 is `failure`, consistent.
  - **SUFFICIENCY (new evidence, closes the residual round-2 unknown):** the job had never run against the moved corpus AND mk-cli v0.8.0 changed stub computation since — so a path fix alone could have unmasked a SECOND red (corpus/output mismatch). Executed the job's exact logic locally at `13821ad`: `cargo build -p mk-cli --release` → `mk vectors --out` (40 files) → the `ci.yml:93-95` jq diff against `crates/mk-codec/src/test_vectors/v0.1.json` → **"vectors-roundtrip OK"**. D2(d) is the one-line fix the plan claims; nothing further hides behind it.
- **M5 → `:14`: FOLDED CORRECTLY.** Now reads "the local default toolchain is 1.97.0-nightly (rustfmt 1.9.0-nightly)" — toolchain/rustfmt versioning no longer conflated. Local `rustup toolchain list` re-confirms nightly is the active default (the hazard D1 guards against is real).
- **M6 → `:29`: FOLDED CORRECTLY.** Conditional phrasing gone; now "mnemonic-key has NO other workflow at all — `.github/workflows/` contains only ci.yml (verified twice; R0-r2 M6)". Re-verified third time: `ls .github/workflows/` → `ci.yml` only.
- **M7 → `:20`: FOLDED CORRECTLY.** D2(b)'s job-comment text now carries the defense-in-depth clause: "a committed rust-toolchain.toml would override this pin too (rustup file-beats-default precedence) — keep the repo free of one (R0-r2 M7)". Consistent with D2's I1 rationale at `:19` and the `:40` non-goal.

## Whole-plan final scan (round-3 re-measurements, all confirm the plan)

- **PIN = 1.95.0 still correct:** local stable = `rustc 1.95.0 (59807616e 2026-04-14)`, rustfmt `1.9.0-stable` same commit hash — matches `:15`.
- **Drift set re-measured at `13821ad`:** `cargo +stable fmt --check --all` flags exactly the plan's 5 files (`mk-cli/src/cmd/mod.rs`, `src/output_advisory.rs`, `src/slip132.rs`, `tests/cli_output_class.rs`, `tests/cli_slip132.rs`); none of the slug's eponymous 3 (`repair.rs`, `cli_repair.rs`, `bch.rs`). `:27` exact.
- **D1's one-pass claim PROVEN, not just inherited from the FOLLOWUP:** in a throwaway worktree at `13821ad`, `cargo +stable(1.95.0) fmt --all` → diff = **exactly 5 files, +241 −68** (the plan's `:3` figures to the line) → `cargo +stable fmt --check` clean AND `cargo +1.85.0 fmt --check --all` clean. The 1.85 MSRV lane accepts the 1.95.0-formatted bytes today. (beta unverifiable locally; post-D2 beta no longer runs fmt, and `:36` already records this as irrelevant — acceptable residual, CI arbitrates.)
- **No other masked reds behind the fmt cure:** `cargo +stable test --workspace` all green (157 + 15 + 10 + … , 0 failed) and `cargo +stable clippy --workspace --all-targets -- -D warnings` clean at `13821ad` — consistent with CI history (lanes fail at the Rustfmt step only, post-`47de269`).
- **Citations re-verified live:** `design/FOLLOWUPS.md:323` is still the entry header; `CLAUDE.md:56` is still the shim tip; `ci.yml` `:4-5` header job list, `:17-19` tag triggers, `:23-24` permissions comment, `:36-37` matrix, `:60-61` Rustfmt step, `:66` needs, `:107` release needs — all exact.
- Nothing else found mis-stated, masked, or missing.

## Verdict

**GREEN — 0 Critical / 0 Important / 1 Minor (M8, optional one-liner or FOLLOWUP).** All four round-2 findings are folded faithfully with zero fold-drift; I3's fold is verified not merely present but *sufficient* (the unmasked `vectors-roundtrip` job passes end-to-end against the corrected path with the live v0.8.0 binary). The plan's central de-risking claims were re-proven empirically this round: one `+1.95.0` fmt pass produces exactly the recorded 5-file/+241−68 diff and is byte-acceptable to the 1.85 lane; test + clippy are green underneath the fmt red. Implementation may begin.
