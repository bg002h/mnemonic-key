# PLAN — clear the rustfmt-drift CI red + pin the fmt gate (chore, NO-BUMP)

**Cycle:** mnemonic-key chore commit (no version bump, no tag — CI-hygiene + mechanical fmt; mk-cli/mk-codec semantics untouched) · **Source SHA:** `13821ad` (main, in sync) · **Resolves:** `design/FOLLOWUPS.md:323` `rustfmt-drift-fn-signature-collapse-3-files` (re-measured 2026-06-10 in the v0.8.0 cycle: 5 files / ~+241 −68; **stable + beta + 1.85 fail IDENTICALLY on all 3 OSes** → one fmt pass satisfies every current CI lane).

## Problem

`cargo fmt --check --all` runs inside ALL NINE matrix jobs (`ci.yml:36-37` toolchain [stable, beta, '1.85'] × 3 OS; Rustfmt step `:60-61`) and is the repo's sole CI red. Two layers:
1. **The drift itself** — origin/main was formatted by an older rustfmt; current toolchains collapse short multi-line fn signatures.
2. **The recurrence vector** — the gate runs with whatever `stable`/`beta` resolve to on run day; every future rustfmt style change re-reds the repo (this exact failure mode, recurring).

## Design

### D1 — one-shot `cargo fmt --all`, formatted by the PINNED toolchain (not local nightly)

The fix bytes MUST come from the toolchain the gate will pin (D2) — the local default toolchain is 1.97.0-nightly (rustfmt 1.9.0-nightly), which is NOT one of the CI lanes (R0-r2 M5) and may format differently (its drift set already differs from the FOLLOWUP's tri-toolchain measurement). Procedure: `rustup toolchain install <PIN> --component rustfmt` → `cargo +<PIN> fmt --all` → verify `cargo +<PIN> fmt --check --all` clean. `<PIN>` = the exact current stable version as a FULL x.y.z triple (R0-r1 M1; measured today: **1.95.0** — shorthand `1.95` floats point releases and trips the action's shorthand arithmetic). Since the FOLLOWUP measured stable≡beta≡1.85 agreement on these files, the result also satisfies the remaining matrix lanes — CI is the final arbiter.

### D2 — recurrence fix: a DEDICATED pinned `fmt` job; remove the per-matrix Rustfmt step

- `rust-toolchain.toml` is the WRONG tool here — corrected mechanism (R0-r1 I1, verified against the action's source): `dtolnay/rust-toolchain` runs `rustup default <toolchain>`, and rustup precedence puts a `rust-toolchain.toml` file ABOVE the default — so a committed file would WIN in every matrix lane, silently hijacking all 9 jobs (the 1.85 MSRV lane would stop testing 1.85; matrix vacuity, not mere ignorance). (This refines the FOLLOWUP's "recommend rust-toolchain.toml" — its alternative clause "or document a fixed fmt toolchain" is what we implement, mechanically enforced; carry this corrected rationale into the FOLLOWUP resolution.)
- ci.yml changes: (a) DELETE the Rustfmt step from the 9-job matrix (it is 9× redundant and the recurrence vector — clippy/build/test stay per-lane); (b) ADD a dedicated `fmt` job: ubuntu-latest, `dtolnay/rust-toolchain@master` with `toolchain: <PIN>` + `components: rustfmt`, running `cargo fmt --check --all`. Comment in the job documents: "<PIN> is the canonical formatting toolchain; bump it DELIBERATELY (re-run cargo +<new> fmt --all in the same commit) — do not float stable here (recurrence: rustfmt-drift-fn-signature-collapse-3-files); a committed rust-toolchain.toml would override this pin too (rustup file-beats-default precedence) — keep the repo free of one (R0-r2 M7)."
- (c) **release gate (R0-r1 I2):** `release-on-tag`'s `needs: [build, vectors-roundtrip]` (`ci.yml:107`) gains `fmt` — today fmt rides inside `build`, so deleting the matrix step without this would silently un-gate releases from fmt.
- (d) **unmask-proofing (R0-r2 I3):** `ci.yml:94`'s `vectors-roundtrip` step pins the STALE pre-move corpus path `crates/mk-codec/tests/vectors/v0.1.json` (moved to `src/test_vectors/` in `33f2ca2`; same move-class as the `47de269` .gitattributes fix, this site missed). The job has been `skipped` on every run since the fmt red began (needs: build) — the last green run 2026-05-13 PREDATES the move, so it has NEVER executed against the moved corpus; curing fmt would unmask it as the next CI red AND block all releases via (c). Fix in the same commit: `ci.yml:94` → `crates/mk-codec/src/test_vectors/v0.1.json`.
- Net effect: formatting is checked exactly once per push (and still gates releases), against a toolchain that only changes when a human bumps it together with a reformat.

### D3 — ritual

- FOLLOWUPS: resolve `rustfmt-drift-fn-signature-collapse-3-files` (record the D2 deviation from the entry's rust-toolchain.toml lean + the corrected I1 rationale + the MEASURED current drift set — exactly 5 files under 1.95.0: `mk-cli/src/cmd/mod.rs`, `src/output_advisory.rs`, `src/slip132.rs`, `tests/cli_output_class.rs`, `tests/cli_slip132.rs`; NONE of the slug's eponymous 3 files are in it anymore — R0-r1 M3).
- ci.yml comment hygiene (R0-r1 M2): update the header job list (`:4-5`) + the permissions comment (`:23-24`) for the new `fmt` job; no rust-cache in the fmt job (no compilation). CLAUDE.md:56's stale "skip clippy/fmt gates if the shim errors" tip — reword to point at the pinned fmt job (M4).
- NO version bump / tag / CHANGELOG release entry (CI-only + mechanical fmt; precedent: the toolkit's CI-only NO-BUMP commits). mnemonic-key has NO other workflow at all — `.github/workflows/` contains only ci.yml (verified twice; R0-r2 M6).
- Verification: full `cargo test --workspace` + `cargo clippy --workspace --all-targets -- -D warnings` locally (fmt must not change behavior — it can't, but the suite run is the proof), `cargo +<PIN> fmt --check --all` clean, actionlint on ci.yml if available; push → ALL CI jobs green (the 9-matrix + new fmt job) = the first all-green mnemonic-key run since 2026-05-13 (`eb7e8b1` — every later run was red or had skipped jobs; R0-r2 I3 corrected the date).

## Risks

- Local-nightly contamination (mitigated by D1's +<PIN> discipline; double-check `git diff` touches only formatting — no semantic hunks).
- The fmt diff may exceed the FOLLOWUP's 5-file estimate if stable moved since 2026-06-10 morning (fine — whatever +<PIN> produces is canonical).
- beta lane could in principle disagree with <PIN> on some future file — irrelevant post-D2 (beta no longer runs fmt).

## Non-goals

Toolkit-side fmt posture (no fmt gate there — different, deliberate); reformatting beyond `cargo fmt --all`; MSRV changes; rust-toolchain.toml.
