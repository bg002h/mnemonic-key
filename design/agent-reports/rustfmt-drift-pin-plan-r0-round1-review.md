# R0 round-1 architect review — PLAN_rustfmt_drift_pin (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). Plan @ design/PLAN_rustfmt_drift_pin.md, main @ 13821ad. Verdict: RED (0 Critical / 2 Important / 4 Minor). Review verbatim below.

---

## Critical

None.

## Important

**I1 — D2's stated mechanism is factually wrong (conclusion survives, but the prose is the FOLLOWUP-resolution rationale and must be corrected).**
Verified against `dtolnay/rust-toolchain@master` `action.yml` (fetched today): the action runs `rustup default ${{steps.parse.outputs.toolchain}}` — **not** `rustup override set`. Plan line `design/PLAN_rustfmt_drift_pin.md:19` claims "the dtolnay action sets a rustup override in CI (the file would be ignored where it matters)". Rustup's precedence is: `+toolchain` CLI > `RUSTUP_TOOLCHAIN` env > directory override (`rustup override set`) > **`rust-toolchain.toml` file** > default toolchain. Since the action only sets the *default*, a committed `rust-toolchain.toml` would NOT be ignored in CI — it would **win in every matrix lane**, silently hijacking all 9 jobs (the `stable`/`beta`/`1.85` lanes at `ci.yml:36` would all build/test/clippy with the pinned toolchain; the 1.85 MSRV lane stops testing 1.85; rustup auto-installs the pin per-job). The second clause at :19 ("while the matrix still checks fmt on moving stable/beta — recurrence unsolved") is therefore also wrong — the real failure mode is matrix vacuity, not unsolved recurrence. The verdict "rust-toolchain.toml is the WRONG tool" survives either way — it is in fact *stronger* (the file would destroy the compat matrix, not merely be ignored). **Fix:** rewrite :19 with the verified mechanism (`rustup default` + rust-toolchain.toml-beats-default precedence ⇒ matrix vacuity), and carry the corrected rationale into the FOLLOWUP resolution.

**I2 — the restructure silently removes fmt from the release gate.**
Today `release-on-tag` has `needs: [build, vectors-roundtrip]` (`ci.yml:107`), and Rustfmt runs *inside* `build` (`ci.yml:60-61`) — so a tag release is currently fmt-gated. D2 deletes the matrix Rustfmt step but never says to add the new `fmt` job to `release-on-tag`'s `needs`; a fmt-red tag push would still cut a GitHub release. **Fix:** `needs: [build, vectors-roundtrip, fmt]` (the `fmt` job already fires on tag pushes via `on.push.tags`, so this is one line).

## Minor

- **M1 — `<PIN>` form:** use the full `x.y.z` triple (measured today: local `stable` = **1.95.0**), not `1.95`. `rustup toolchain install 1.95` floats to the newest 1.95.z point release, and the action's parse step applies special arithmetic to `^1\.[0-9]+$` shorthands (benign for 95 today, but the full triple sidesteps both).
- **M2 — ci.yml comment drift:** the header job list (`ci.yml:4-5`) and the permissions comment (`ci.yml:23-24`, "build + vectors-roundtrip run with the default read-only token") enumerate jobs by name — update both when adding `fmt`. No `Swatinem/rust-cache` needed in the fmt job (no compilation).
- **M3 — record the measured drift set in the resolution.** Today's `cargo +stable(1.95.0) fmt --check --all` flags exactly 5 files: `crates/mk-cli/src/cmd/mod.rs`, `crates/mk-cli/src/output_advisory.rs`, `crates/mk-cli/src/slip132.rs`, `crates/mk-cli/tests/cli_output_class.rs`, `crates/mk-cli/tests/cli_slip132.rs`. Note the FOLLOWUP's own listings have drifted: the original Where (`repair.rs:65`, `cli_repair.rs:366`, `bch.rs:271`) and the re-measure's "plus … bch.rs" are all CLEAN under 1.95.0 — none of the slug's eponymous 3 files are in the current set.
- **M4 — CLAUDE.md:56** ("skip clippy/fmt gates if the shim errors") is a stale local-env tip that now conflicts with a pinned fmt gate; opportunistically reword or drop in the same commit.

## Verification results (the adversarial checks)

1. **ci.yml structure claims: VERIFIED.** Matrix `[stable, beta, '1.85'] × 3 OS` at `ci.yml:36-37`; Rustfmt step at `:60-61`; dtolnay@master at `:42`. Action behavior verified from source (not guessed) — see I1.
2. **Highest-risk check: CLEAR.** Nothing hashes or line-pins `.rs` sources. `vector_file_sha256_matches_pin` (`crates/mk-codec/tests/vectors.rs:55,108`) hashes only `src/test_vectors/v0.1.json`; `error_coverage.rs:134` reads the same JSON; `round_trip.rs:99-112` reads its own tempdir output; the only `include_str!` is the JSON corpus (`src/test_vectors/mod.rs:16`); the self-referential `slip132_version_bytes_match_slip0132` test declares an inline byte array (no file read). Line-number citations in `tests/bch_adversarial.rs:45,116` are comments, in files outside the drift set. Inspected the actual fmt diff: pure re-wrapping, every string literal byte-identical (e.g. `output_advisory.rs:26` advisory strings merely move into block arms).
3. **Changelog/tag gates: CLEAR.** `.github/workflows/` contains only `ci.yml` — no changelog-check or any other workflow. Precedent `47de269` (fix(ci), NO-BUMP) shipped the same shape cleanly.
4. **NO-BUMP call: SOUND** (formatting-only, suite + clippy as proof per D3). FOLLOWUPS citation live: the entry header is exactly `design/FOLLOWUPS.md:323`. No branch protection on `main` (gh api 404) → no required-check rename hazard.
5. **D1/D2 identity risk: EMPIRICALLY DISCHARGED.** In a scratch worktree at `13821ad`: applied `cargo +stable(1.95.0) fmt --all`, then `cargo +1.85.0 fmt --check --all` passes clean (exit 0, zero diffs) — the stable-formatted tree satisfies the MSRV lane today. Beta untested locally, but D2 makes any future beta divergence moot (beta no longer runs fmt), which is the plan's own correct argument at `:33`.

## Verdict

**NOT GREEN — 0 Critical / 2 Important.** Fold I1 (rewrite D2's mechanism with the verified `rustup default` + precedence facts) and I2 (`needs: [build, vectors-roundtrip, fmt]` on release-on-tag), plus the minors as desired, then re-dispatch. The plan's core design (one-shot pinned fmt + dedicated pinned `fmt` job, NO-BUMP chore) is correct and empirically validated; both Importants are prose/one-line-YAML folds.
