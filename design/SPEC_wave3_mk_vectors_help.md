# SPEC — W3-3: mk-cli `vectors --pretty` help-text correction (PATCH + crates.io publish)

**Repo:** `/scratch/code/shibboleth/mnemonic-key` (mk-cli) · **Default branch:** `main` (RE-CONFIRMED: `git rev-parse --abbrev-ref HEAD` = `main`; md+mk use `main`, toolkit+ms use `master`)
**Source SHA at write time:** `f3c8cea` (HEAD of `main`)
**Semver:** PATCH `0.10.1` → `0.10.2` · **Ship:** FF to `main` + tag `mk-cli-v0.10.2` + `cargo publish -p mk-cli` (orchestrator dry-runs first)
**Latest published/tagged:** `mk-cli-v0.10.1` (RE-VERIFIED via `git tag -l 'mk-cli*' | sort -V` — do NOT use `tail` of an unsorted `grep`, which truncates and appears to top out at v0.9.0; tag + Cargo.toml + Cargo.lock all agree at 0.10.1). **Idempotency guard: `git tag -l 'mk-cli-v0.10.2'` MUST be EMPTY before tagging (verified empty at write time).**

This is a PURE help-text / doc-comment correction. **No functional behavior changes anywhere** — the source already honors `--pretty` under `--out`. The slug is `mk-vectors-pretty-out-help-mismatch` (lives in toolkit `design/FOLLOWUPS.md:2397-2405`).

---

## Change 1 — Reword the `--pretty` clap doc-comment (THE FIX)

**File:** `crates/mk-cli/src/cmd/vectors.rs` · **Line 22** (re-verified against current source `f3c8cea`)

### Current behavior
The doc-comment on `pub pretty: bool` (this IS the clap-derive `--help` text) reads (line 22):
```rust
    /// Indent the JSON output for human readability. Ignored when `--out` is supplied.
    #[arg(long)]
    pub pretty: bool,
```
This is **WRONG**. The per-fixture write path `write_per_fixture_files` (vectors.rs:70-74, re-verified) branches:
```rust
let body = if pretty {
    serde_json::to_string_pretty(v)
} else {
    serde_json::to_string(v)
}
```
so `--pretty` **IS honored** when `--out` is supplied (each per-fixture file is pretty-printed). The help text claims the opposite.

### Exact edit
Replace line 22 with a source-truth statement. Suggested wording (implementer may tighten, but must affirm `--pretty` applies under `--out`):
```rust
    /// Indent the JSON output for human readability. Also applies to each
    /// per-fixture file when `--out` is supplied.
```
Constraints:
- Single doc-comment line(s) on `pub pretty: bool`; clap renders it verbatim into `mk vectors --help`.
- **MUST remain a non-empty `///` doc** (clap-derive requires it as help text; `[workspace.lints.rust] missing_docs = "warn"` @ Cargo.toml:11-12 also applies to the package — an empty/removed doc would both change help output AND risk tripping `missing_docs`). The reworded `///` keeps `pub pretty` documented → `missing_docs` stays satisfied.
- Must NOT change the flag name `--pretty` or its `#[arg(long)]` attribute (changing the name would re-fire flag-NAME gates downstream — out of scope and forbidden here).
- The text must no longer assert any "ignored / silently ignored when --out" claim.

### Why this is safe
- It is clap-derive help text → a malformed doc-comment is a **compile error**, caught by `cargo build -p mk-cli` (Change-1 verification).
- `mk gui-schema` does **NOT** emit help text (gui_schema.rs:73-74 skips the auto-generated `help` field; the emitted Flag struct = name/required/kind/choices). So this edit produces **zero** change to gui-schema JSON → mk-cli `gui_schema.rs` tests and the GUI `schema_mirror`/`smoke-gui-schema-mk` gates are unaffected.

---

## Change 2 — TDD hardening test (pins the corrected contract)

**File:** `crates/mk-cli/tests/round_trip.rs` (append a new `#[test]`)
**Rationale:** No test in any of the 3 repos currently exercises `--pretty` + `--out` together — which is exactly why this prose drift persisted undetected. The existing `vectors_subcommand_no_path_dep` (round_trip.rs:99-126, re-verified) runs `vectors --out` WITHOUT `--pretty` and asserts file count (`>= 8`) + parseability + a `name` field on each. Add a positive test proving `--pretty` is honored under `--out` (pretty JSON contains newlines/indentation), locking the documented contract.

> **rustfmt note (HARD GATE — see verification matrix):** This change adds ~30 lines of HAND-WRITTEN test code. CI runs `cargo fmt --check --all` on a **pinned 1.95.0** toolchain (ci.yml:68-84); if the new code is not rustfmt-clean the `fmt` job goes RED, and because `release-on-tag` has `needs: [build, fmt, vectors-roundtrip]` (ci.yml:130) the GitHub release for tag `mk-cli-v0.10.2` will NOT be created. The new test code MUST be rustfmt-clean. This repo has **NO `mlock.rs`** (`find . -name mlock.rs` is EMPTY) so the g6 fmt-exemption is N/A — a plain `cargo fmt --check --all` is both safe and required.

### TDD order
This is the bug-fix lock. Write the test FIRST and confirm it PASSES against the *current* (already-correct) source behavior — this is a **characterization/contract test** pinning the behavior the reworded doc now correctly describes (the doc was the bug, not the code; the test guards against a future regression of the code AND documents the contract the help text now states).

### Test sketch (implementer finalizes)
Uses the SAME invocation pattern as the existing `vectors_subcommand_no_path_dep` test: `Command` here is **`std::process::Command`** (imported at round_trip.rs:5) extended by the **`CommandCargoExt`** trait (`use assert_cmd::cargo::CommandCargoExt;` at round_trip.rs:8), which provides `Command::cargo_bin("mk")`. `tempfile` and `serde_json` are already dev-deps and in scope (`tempfile::tempdir()` is used at line 100). Do NOT introduce `assert_cmd::Command` — the file does not use it; stay consistent with the existing imports.
```rust
/// `mk vectors --pretty --out <DIR>` must pretty-print each per-fixture file
/// (proving --pretty IS honored under --out; the help text documents this).
#[test]
fn vectors_pretty_out_writes_indented_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut cmd = Command::cargo_bin("mk").expect("mk binary");
    let out = cmd
        .args(["vectors", "--pretty", "--out"])
        .arg(dir.path())
        .output()
        .expect("invoke mk vectors --pretty --out");
    assert!(out.status.success(), "mk vectors --pretty --out failed: {:?}", out);

    let json_files: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read tempdir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    assert!(!json_files.is_empty(), "expected ≥1 fixture file");

    // Pretty output for any object/array fixture contains a newline + indent.
    let mut saw_indented = false;
    for entry in &json_files {
        let body = std::fs::read_to_string(entry.path()).unwrap();
        // valid JSON
        let _: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        if body.contains("\n  ") {
            saw_indented = true;
        }
    }
    assert!(
        saw_indented,
        "expected at least one --pretty --out file to be indented (newline + 2-space); \
         --pretty must be honored under --out"
    );
}
```
Notes:
- **Style consistency:** the existing test uses `{:?}` positional format (`"mk vectors failed: {:?}", out` at line 107), so the sketch above uses `{:?}` to match. `{out:?}` inline-capture also compiles; either is fine, but rustfmt (1.95.0) must leave it untouched — see the rustfmt note above.
- `serde_json::to_string_pretty` emits 2-space indentation with `\n` separators; checking for `"\n  "` is robust for any non-empty object/array fixture. The existing test asserts the corpus has ≥8 fixtures each carrying a `name` field (i.e. they are JSON objects, not bare scalars), so the `"\n  "` indentation check WILL match — no bare-scalar fallback is needed for this corpus. (If a future fixture were a bare scalar, relax to comparing byte-length: pretty `--out` > compact `--out` for the same fixture name.)
- Keep using the existing imports already at the top of `round_trip.rs` (`std::process::Command` + `assert_cmd::cargo::CommandCargoExt`); no new `use` is required (`tempfile`/`serde_json` already in scope).
- Integration-test fns are not flagged by `missing_docs`, but keeping the `///` doc on the test fn is harmless and consistent.

---

## Change 3 — Version bump

**File:** `crates/mk-cli/Cargo.toml` · the `version = "..."` line
```
version = "0.10.1"   →   version = "0.10.2"
```

**File:** `Cargo.lock` (root) · the `[[package]] name = "mk-cli"` block (version field at line 540-541, re-verified at 0.10.1)
```
version = "0.10.1"   →   version = "0.10.2"
```
Prefer regenerating the lock deterministically: after the Cargo.toml edit run `cargo update -p mk-cli --precise 0.10.2` OR simply `cargo build -p mk-cli` (which rewrites the lock entry). Verify the diff touches ONLY the mk-cli version line (no incidental dep churn).

**Version sites — EXHAUSTIVE (verified):**
- `crates/mk-cli/Cargo.toml` ✅ (only `version = "..."` site)
- root `Cargo.lock` ✅
- ❌ NO README version site (grep of `crates/mk-cli/README.md` + root `README.md` for `0.10`/`mk-cli-v`/`version` is EMPTY — mk-cli READMEs carry no pinned version string)
- ❌ NO `fuzz/Cargo.lock` mk-cli entry (verified: `fuzz/Cargo.lock` has no `name = "mk-cli"`)
- ❌ NO install.sh / workflow self-pin in THIS repo for mk-cli (the pins live in the toolkit repo and stay at v0.8.0 — DO NOT touch, see CI gates)

---

## Change 4 — CHANGELOG

**File:** `crates/mk-cli/CHANGELOG.md` — prepend a new `## [0.10.2]` section directly above the existing `## [0.10.1] — 2026-06-21` heading (line 10). Match the existing Keep-a-Changelog format.

```markdown
## [0.10.2] — 2026-06-22

**SemVer-PATCH — help-text correction only. No flag, no API, no wire/behavior change; `mk-codec` untouched.**

- **`mk vectors --pretty` help text corrected.** The clap doc-comment on `--pretty`
  (`src/cmd/vectors.rs`) previously claimed it was "Ignored when `--out` is supplied".
  That was wrong: `write_per_fixture_files` honors `--pretty`, pretty-printing each
  per-fixture file written under `--out`. The help text now states `--pretty` applies
  to the per-fixture files as well. Behavior is unchanged (the code was already correct);
  this fixes the documented contract. A new `vectors_pretty_out_writes_indented_files`
  test pins it. Closes FOLLOWUP `mk-vectors-pretty-out-help-mismatch`.
```
(Implementer may adjust prose; the `## [0.10.2] — 2026-06-22` header and the PATCH framing are required.)

---

## Atomicity / ordering

Single atomic commit on a feature branch (do NOT commit on `main` directly per repo convention; branch first), then FF to `main`:

1. Edit Change 1 (vectors.rs:22 doc-comment).
2. Add Change 2 (new test in round_trip.rs).
3. Edit Change 3 (Cargo.toml 0.10.2) → `cargo build -p mk-cli` to sync the Cargo.lock mk-cli version AND confirm the reworded doc-comment + new test code compile (single build covers both).
4. Edit Change 4 (CHANGELOG).
5. **`cargo +1.95.0 fmt --check --all`** — MUST be clean (pinned 1.95.0 toolchain; this is the exact CI fmt gate, ci.yml:68-84; no `mlock.rs` exclusion in this repo). If RED, run `cargo +1.95.0 fmt --all` and re-stage.
6. `cargo test -p mk-cli` (FULL package suite) + **`cargo clippy --workspace --all-targets -- -D warnings`** (mirrors CI exactly, ci.yml:57-58; `[workspace.lints.clippy] all = "warn"` @ Cargo.toml:14-15 is promoted to DENY by `-D warnings`) — both GREEN.
7. Stage paths EXPLICITLY (no `git add -A`): `vectors.rs`, `round_trip.rs`, `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`.
8. Commit. Trailer: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.
9. FF to `main` → confirm `git tag -l 'mk-cli-v0.10.2'` is EMPTY (idempotency) → tag `mk-cli-v0.10.2` (annotated) → `cargo publish -p mk-cli --dry-run` (orchestrator) → `cargo publish -p mk-cli`.

There is no split-push ordering hazard in THIS repo (no sibling-pin-check workflow here; that gate lives in the toolkit). Single commit + single tag + single publish. The tag push fires CI `release-on-tag` (ci.yml:126), which creates the GitHub release ONLY after `build`, `fmt`, and `vectors-roundtrip` all pass (`needs: [build, fmt, vectors-roundtrip]`, ci.yml:130) — hence steps 5 and 6 above are load-bearing for the release, not just local hygiene.

---

## FOLLOWUP flips (at ship)

- **Toolkit `design/FOLLOWUPS.md:2397-2405`** — flip slug `mk-vectors-pretty-out-help-mismatch` Status `open` → `resolved` (lands in the toolkit doc lane, NOT this mk-cli commit — decoupled).
- **Add Companion entries** in mk-cli's FOLLOWUPS record and mnemonic-gui's FOLLOWUPS, cross-citing the toolkit slug (the slug self-notes both are currently missing). Cross-repo lockstep DISCIPLINE, not gate-enforced.
- These FOLLOWUP flips + the toolkit/GUI prose-mirror edits are in their OWN lanes/PRs per the orchestrator decision (no CI gate compares help prose across repos). This mk-cli lane ships independently.

---

## Verification matrix (HOW to verify each gate)

| Gate | Type | Verify | Expected |
|---|---|---|---|
| `cargo build -p mk-cli` | LOCAL | run in repo root | exit 0 (reworded clap doc-comment + new test compile) |
| `cargo test -p mk-cli` (FULL suite) | LOCAL (mirrors CI `cargo test --workspace`, ci.yml:55) | run whole package, not `--test <one>` | all green incl. new test; per MEMORY: never targeted |
| **`cargo +1.95.0 fmt --check --all`** | **LOCAL — mirrors CI `fmt` job (ci.yml:68-84, pinned 1.95.0)** | run BEFORE commit; pinned 1.95.0 toolchain; **no `mlock.rs` in this repo so NO fmt exclusion** | exit 0 (new ~30-line test must be rustfmt-clean). **HARD SHIP GATE: a RED fmt blocks `release-on-tag` (needs:[…,fmt,…], ci.yml:130) → no GitHub release for the tag.** |
| **`cargo clippy --workspace --all-targets -- -D warnings`** | **LOCAL — mirrors CI build job clippy step EXACTLY (ci.yml:57-58)** | run in repo root; `-D warnings` promotes `[workspace.lints.clippy] all = "warn"` (Cargo.toml:14-15) to DENY | 0 warnings. A clippy warning in the new test (needless borrow / redundant clone in the read-dir chain) would pass a bare local clippy but RED CI build → blocks `release-on-tag`. Do NOT use a no-deny local clippy. |
| `vectors-roundtrip` | **CI-ONLY (ci.yml:86-124, `needs: build`)** | builds mk-cli --release, runs `mk vectors --out` (compact, NO `--pretty`), reconstructs via `jq -s -S 'sort_by(.name)'` and structurally diffs against pinned `crates/mk-codec/src/test_vectors/v0.1.json` | **NOT re-fired.** Lane changes only a doc-comment + adds a test; does NOT alter `write_per_fixture_files` write behavior; the gate uses the `--out` COMPACT path (not `--pretty`) and `jq` normalizes JSON whitespace, so pretty-vs-compact is irrelevant. ASSERTED NON-FIRING. |
| `release-on-tag` | CI-ONLY (ci.yml:126-147) | `needs: [build, fmt, vectors-roundtrip]`; on tag push creates GH release | Fires on `mk-cli-v0.10.2` tag; succeeds only if all three needs are GREEN. |
| `gui_schema.rs` (in-package) | LOCAL (via full suite) | `mk gui-schema` skips auto-`help` field (gui_schema.rs:73-74) | PASS unchanged; spot: `mk gui-schema \| grep -c pretty` unchanged |
| `missing_docs = "warn"` | LOCAL (Cargo.toml:11-12, applies to package) | reworded `--pretty` stays a non-empty `///` doc | PASS unchanged (clap-derive requires the doc as help text; the edit cannot accidentally drop it) |
| `version_help_exit_codes.rs::help_flag_exits_zero_and_prints_help` | LOCAL (via full suite) | asserts only top-level `Usage:`+`mk` | PASS; never reads vectors prose |
| help-golden / trycmd / insta | NONE | `grep -rln 'Ignored when' crates/` post-edit | only fixed in vectors.rs; no golden test trips |
| `cargo publish -p mk-cli --dry-run` | crates.io preflight | orchestrator runs first | exit 0 (mk-codec 0.4.0 already published) |
| toolkit manual lint / sibling-pin-check | CI-ONLY (toolkit repo) | DECOUPLED — pin stays v0.8.0; lint checks flag NAME not prose | NOT re-fired; DO NOT bump toolkit pin (G1-B trap) |
| GUI schema_mirror / smoke-gui-schema-mk | CI-ONLY (gui repo) | DECOUPLED — gates flag-NAMES; mk gui-schema has no help text | NOT re-fired; no flag name changes |

**Hard CI-gate discipline note:** Two same-repo CI gates that a plain local build does NOT reproduce are now first-class in the matrix: (1) the pinned-1.95.0 **`fmt`** gate (the new hand-written test code is the live risk; this repo has no `mlock.rs`, so a plain `cargo +1.95.0 fmt --check --all` is correct and required), and (2) the CI build job's **clippy with `-D warnings`** (the spec's prior local clippy omitted `-D warnings` and was therefore NOT representative). Both feed `release-on-tag` via `needs`, so a RED in either silently blocks the GitHub release for the tag. The CI-ONLY `vectors-roundtrip` gate is named and asserted non-firing. The one cross-repo trap to avoid (per the Wave-2 G1-B revert lesson) is opportunistically bumping the toolkit's mk-cli pin from v0.8.0 to "pull in" the new `--help` — that would re-fire the toolkit sibling-pin-check + the gui-schema-coverage CI-only gate for zero benefit, because no toolkit/GUI gate reads help prose. DO NOT bump the toolkit pin.

---

## Risk: LOW
Pure doc/help-text + version + changelog + one positive test. No funds path, no wire format, no flag-name change, no g6/mlock coupling (this repo has no `mlock.rs`), no MSRV tension, no manual flag-mirror trap. The two in-repo CI gates a plain local build would miss — the pinned-1.95.0 `fmt` gate and the `-D warnings` clippy gate — are now explicit pre-commit steps (steps 5-6) and matrix rows, so the "LOW" framing no longer leans on an unrepresentative local clippy. The only ship-cost beyond the commit is the normal crates.io PATCH publish (required so the corrected `--help` reaches `cargo install` users).