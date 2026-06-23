## R0 Review — W3-3 mk-cli `vectors --pretty` help-text correction

**Verdict: GREEN — 0 Critical / 0 Important / 3 Minor. CLEARED TO IMPLEMENT.**

Source verified against CURRENT `main` HEAD `f3c8cea` in `/scratch/code/shibboleth/mnemonic-key`. Every load-bearing claim re-grepped; all check out. The 3 Minors are informational and none gate the start of coding.

### Adversarial scrutiny — the hard questions answered

**(a) Does each change do what it claims WITHOUT breaking a CI gate — especially CI-ONLY gates?**
- **GUI `schema_mirror` (CI-only, gui repo) — PROVEN unaffected.** I read `mnemonic-gui/tests/schema_mirror.rs`: it compares ONLY flag-NAME sets (`schema_flag_names` maps `f.name`; upstream side extracts `--<flag>` tokens or `gui-schema` JSON flag names). It never reads the `help:` field. And `mk gui-schema`'s `Flag` struct (`crates/mk-cli/src/cmd/gui_schema.rs:54-60`) has NO `help` field — only `name/required/kind/choices`; `flag_from_arg` (104-117) emits only those. So `mk gui-schema` is provably help-text-free, and the `--pretty` flag NAME is unchanged. The hand-maintained GUI `help:` strings (mk.rs `VECTORS_FLAGS` ~274-276) are pure prose, gated by nothing — correctly deferred to a separate GUI lane. Spec claim (a) is correct.
- **`vectors-roundtrip` (CI-only) — asserted non-firing, VERIFIED correct.** ci.yml:86-124 runs `mk vectors --out` (compact, no `--pretty`) and reconstructs via `jq -s -S 'sort_by(.name)'` vs the pinned corpus. `jq` normalizes whitespace, the gate uses the COMPACT path, and the lane changes neither `write_per_fixture_files` write behavior nor the corpus. Non-firing confirmed.
- **`fmt` (pinned 1.95.0, ci.yml:68-84) — real risk, correctly elevated.** ~30 lines of hand-written test code; `cargo +1.95.0 fmt --check --all` is the exact gate. No `mlock.rs` in this repo (`find . -name mlock.rs` EMPTY) → g6 exemption N/A → a plain `--all` is correct and required. `release-on-tag` has `needs: [build, fmt, vectors-roundtrip]` (ci.yml:130) so a RED fmt blocks the GH release. Steps 5-6 make this a pre-commit gate. Correct.
- **clippy `-D warnings` (ci.yml:57-58) — correctly added** as a representative local step (bare local clippy would miss the deny promotion of `[workspace.lints.clippy] all = "warn"`, Cargo.toml:14-15).
- **ms g6-invariant / sibling-pin-check (toolkit) — N/A.** Those gates live in the toolkit, not here. No `mlock.rs`, no `install.sh`/workflow self-pin for mk-cli in this repo. The one cross-repo trap (opportunistically bumping the toolkit's mk-cli pin from v0.8.0) is explicitly forbidden by the spec — correctly avoiding the Wave-2 G1-B class of break.

**(b) Are golden snapshots captured from the pinned binary, not hand-written?** N/A by design — this lane adds NO golden snapshot. It adds one positive characterization test that shells out to `Command::cargo_bin("mk")` (the freshly built binary) and asserts a structural property (pretty output contains `"\n  "`). No hand-written golden to drift. I empirically confirmed the corpus has 40 object fixtures and `to_string_pretty` yields `"\n  "`, so the assertion holds against the live binary's output. This is the correct shape — it self-derives from the binary rather than freezing a snapshot.

**(c) Is the pin de-stale ATOMIC + md-leg correctly EXCLUDED?** There is NO pin de-stale in this lane (the spec correctly does NOT touch any sibling pin; toolkit's mk-cli pin stays v0.8.0). `md-codec` stays at 0.34.0 (Cargo.toml:24, untouched) and `mk-codec` at 0.4.0 (Cargo.toml:20). md-leg fully excluded. Single atomic commit → FF → single tag → single publish; no split-push hazard in this repo (no sibling-pin-check here).

**(d) Is `mlock.rs` correctly excluded from fmt?** N/A — this repo has NO `mlock.rs` (verified empty). The spec correctly states a plain `cargo +1.95.0 fmt --check --all` is both safe and required. No g6 coupling.

**(e) SemVer + version sites complete?** PATCH 0.10.1→0.10.2 is correct: help-text-only, no API/flag/wire change, `mk-codec` untouched. Version sites verified EXHAUSTIVE: `crates/mk-cli/Cargo.toml:3` (0.10.1) and `Cargo.lock:541` (0.10.1) are the ONLY two sites. Confirmed empty: no README version string, no `fuzz/Cargo.lock` mk-cli entry, no install.sh/workflow self-pin. CHANGELOG prepend point (`## [0.10.1] — 2026-06-21` at line 10) is correct. Tag idempotency guard satisfied: `git tag -l 'mk-cli-v0.10.2'` is EMPTY; latest tag is `mk-cli-v0.10.1` (confirmed via `sort -V`).

**(f) No scope creep into deferred items?** Clean. No md-leg, no W3-4/W3-5, no export-wallet, no flag rename, no `--pretty` attribute change. The toolkit FOLLOWUPS flip + mk-cli/GUI companion entries + GUI prose-mirror are correctly assigned to OWN decoupled lanes, not this commit.

### Spot-checks
- vectors.rs:22 currently reads the WRONG `Ignored when --out is supplied` (confirmed via grep). `write_per_fixture_files` pretty branch at vectors.rs:70-74 honors `--pretty`. The reworded `///` stays non-empty → `missing_docs = "warn"` (Cargo.toml:11-12) stays satisfied; a malformed doc-comment is a compile error → caught by `cargo build`.
- Test imports: `round_trip.rs` uses `std::process::Command` (line 5) + `assert_cmd::cargo::CommandCargoExt` (line 8); `tempfile::tempdir()` at line 100; `serde_json` used at line 123. The spec's instruction to reuse these and NOT pull in `assert_cmd::Command` (which `version_help_exit_codes.rs` uses, a different file) is correct.
- MSRV: build matrix pins `1.85` (Cargo.toml:7 / ci.yml:36). The new test uses only long-stable APIs (`std::fs::read_dir`, `tempfile`, `serde_json`, closures) → no MSRV tension.
- ci.yml line citations (55, 57-58, 68, 84, 126, 130) all EXACT.

### Minors (non-blocking)
1. The fmt job runs a co-located `design/` checksum-pin step (ci.yml:74-75) the spec omits; not a risk here (lane touches no `design/`), but don't let an incidental `design/` edit ride along.
2. Toolkit slug citations (`vectors.rs:23`, GUI `mk.rs:208`) are stale snapshots; the spec correctly re-targets the live line (vectors.rs:22). The deferred GUI lane should re-grep (`--pretty` help is ~mk.rs:274-276).
3. Cosmetic: `write_per_fixture_files` spans 53-83; the cited branch lines 70-74 are exact, but the phrasing could read as the whole fn being at 70-74.

**Gate: 0C/0I — GREEN. Cleared to implement.**