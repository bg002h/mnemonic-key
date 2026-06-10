# Implementation review — rustfmt-drift chore (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Plan @ design/PLAN_rustfmt_drift_pin.md (R0 GREEN r3). Verdict: GREEN (0 Critical / 0 Important / 3 Minor — minors 1+2 folded post-review: vestigial matrix rustfmt component dropped, double blank line removed; minor 3 accepted as-is). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

1. **Vestigial `rustfmt` component in the matrix install** — `.github/workflows/ci.yml:45` still installs `components: clippy, rustfmt` in the 9-lane build matrix even though the matrix no longer runs fmt. Harmless (tiny install cost) but slightly at odds with the fmt-job comment's "build matrix no longer runs fmt". Optional cleanup: `components: clippy`.
2. **Double blank line** at `.github/workflows/ci.yml:59-60` (between the Clippy step and the fmt-job comment block). Cosmetic only; actionlint does not flag it.
3. **The 1.95.0 pin is written in 4 sites within ci.yml** (comment `:61`, job name `:70`, step name `:75`, `toolchain: :78`) plus CLAUDE.md:56 — a future deliberate bump must touch all five. The job comment's "bump it DELIBERATELY" covers the discipline but doesn't enumerate the sites. Acceptable as-is.

## Verdict

**GREEN** (0 Critical / 0 Important)

Evidence per review charge:

1. **Fmt diff is exactly mechanical, exactly the 5 predicted files.** `git diff --numstat crates/` = `cmd/mod.rs` (2/1), `output_advisory.rs` (6/3), `slip132.rs` (72/20), `tests/cli_output_class.rs` (13/5), `tests/cli_slip132.rs` (148/39) — sums to **+241 −68**, matching the FOLLOWUPS claim digit-for-digit. **Strongest proof:** fresh `git worktree` at HEAD `13821ad` + `cargo +1.95.0 fmt --all` → `diff -r --brief` against the working tree's `crates/` = **byte-identical** ("REPRO: fmt output IDENTICAL"). The diff is provably pure 1.95.0 rustfmt output — no hand edits, no semantic hunks, no string-literal content changes possible (manual hunk inspection confirms: all advisory/help strings re-wrapped intact, e.g. `output_advisory.rs:29-33`, `slip132.rs:78-117`).

2. **ci.yml conforms to D2 in full.** Matrix Rustfmt step removed (build job now ends at Clippy, `:57-58`); dedicated `fmt` job `:69-82` pinned `toolchain: 1.95.0` (full triple, parses as YAML string) + `components: rustfmt`, no rust-cache, **no `if:` filter** → fires on tag pushes (workflow `on.push.tags` `:17-19`), making `release-on-tag` `needs: [build, fmt, vectors-roundtrip]` (`:128`) satisfiable on tags; recurrence comment `:61-68` carries the deliberate-bump ritual, the FOLLOWUPS slug, and the rust-toolchain.toml file-beats-default warning (plan D2 + R0-r2 M7); header job list `:4-5` and permissions comment `:24` updated; vectors-roundtrip corpus path corrected to `crates/mk-codec/src/test_vectors/v0.1.json` (`:115`). `actionlint` **clean**. Full-file read confirms nothing else changed.

3. **Docs/FOLLOWUPS all true.** README.md:50 is path-only (`tests/vectors/` → `src/test_vectors/`), no scope creep; repo-wide grep confirms it was the only other *live* stale reference (remaining hits are `tests/vectors.rs` — a different, extant file — and historical CHANGELOG/plan-doc records). CLAUDE.md:56 rewrite is accurate (pin value, `cargo +1.95.0 fmt --all`, matrix-hijack rationale) and correctly replaces the stale rustup-shim tip (plan M4). FOLLOWUPS resolution (`design/FOLLOWUPS.md:330`): drift set = exactly the 5 files in the diff ✓; +241/−68 ✓; D2 deviation rationale matches plan R0-r1 I1 ✓; I3 bonus (vectors path) matches the ci.yml hunk ✓; all 4 cited artifacts exist on disk ✓.

4. **Verification runs, all green:** `cargo +1.95.0 fmt --check --all` CLEAN; `cargo +1.85.0 fmt --check --all` CLEAN (both toolchains installed — the one-pass-satisfies-MSRV-lane claim re-proved); `cargo test --workspace` — every `test result:` line `0 failed`; `cargo clippy --workspace --all-targets -- -D warnings` clean. Bonus: re-executed the vectors-roundtrip jq comparison against the corrected path with a freshly built `mk vectors --out` → **VECTORS-ROUNDTRIP OK** (40 vector files), confirming the unmasked job will go green, not red.

Tree left exactly as found.
