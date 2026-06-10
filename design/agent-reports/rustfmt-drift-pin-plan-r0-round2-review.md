# R0 round-2 architect review — PLAN_rustfmt_drift_pin (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold verification). main @ 13821ad. Verdict: RED (0 Critical / 1 NEW Important I3 / 3 Minor; all 6 round-1 findings FOLDED-OK with zero drift). Review verbatim below.

---

## Critical

None.

## Important

**I3 (NEW) — the plan's verification gate ("push → ALL CI jobs green") is unachievable as written: `vectors-roundtrip` carries a latent stale corpus path that is currently MASKED by the fmt red, and fixing fmt will unmask it.**
Evidence chain, all verified live:
- `ci.yml:94` pins `crates/mk-codec/tests/vectors/v0.1.json` — that path **does not exist**. The corpus moved to `crates/mk-codec/src/test_vectors/v0.1.json` in `33f2ca2` (2026-05-15, "promote test_vectors to public API"); `.gitattributes:5` even documents the move ("the corpus moved from tests/vectors/ to src/test_vectors/") — the same move whose stale-path fallout was just fixed in `47de269`, but `ci.yml:94` was missed.
- Why nobody has seen it fail: `vectors-roundtrip` has `needs: build` (`ci.yml:66`), and every run since 2026-06-01 is `build: failure` (fmt) → `vectors-roundtrip: skipped` (confirmed on run 27278774264: all 9 build lanes failure, vectors-roundtrip **skipped**). The last fully-green run is 2026-05-13 (`eb7e8b1`) — **before** the move. `vectors-roundtrip` has never executed against the moved corpus.
- What happens post-fmt-fix: build goes green → vectors-roundtrip runs → `pinned=$(jq -S ... crates/mk-codec/tests/vectors/v0.1.json)` under `set -eu` fails on the missing file → job red. The plan's D3 gate at `:29` ("ALL CI jobs green") fails, and — because `release-on-tag` has `needs: [build, vectors-roundtrip, fmt]` post-D2(c) — every future tag release is also blocked.
**Fix (one line, same file the plan already edits):** D2 gains (d): correct `ci.yml:94` to `crates/mk-codec/src/test_vectors/v0.1.json`. This is in-scope for a CI-hygiene chore and is *required* to meet the plan's own stated verification. Also correct the `:29` claim "first all-green run since 2026-06-01" — the 2026-06-01 runs were red; the last green run is 2026-05-13.

## Minor

- **M5 — `:15` wording:** "the local dev rustfmt is `1.97.0-nightly`" conflates toolchain and rustfmt versioning — the local default *toolchain* is `1.97.0-nightly`; its rustfmt self-reports `1.9.0-nightly (52b6e2c208)`. The argument is unaffected (it is indeed not a CI lane); reword to "the local default toolchain is 1.97.0-nightly (rustfmt 1.9.0-nightly)" for a persisted-doc-accurate record.
- **M6 — `:28` "verify — it's tag-gated if present"** can be tightened: round 1 already verified `.github/workflows/` contains only `ci.yml` (re-confirmed this round) — no changelog gate exists at all. Drop the conditional phrasing.
- **M7 — defense-in-depth note for D2's comment:** the fmt job relies on the action's `rustup default <PIN>` with no `rust-toolchain.toml` present (verified: none in the repo, and it's a stated non-goal). Worth one clause in the job comment that a future committed `rust-toolchain.toml` would override the pin in this job too — same precedence fact as D2's rationale, cheap to record where the next editor will look.

## Fold-verification

- **I1 → `:19`: FOLDED CORRECTLY, and the corrected mechanism RE-VERIFIED against the action source fetched this round.** `dtolnay/rust-toolchain@master` `action.yml` runs `rustup toolchain install ...` then `- run: rustup default ${{steps.parse.outputs.toolchain}}` — `rustup default`, not an override; the action sets no `RUSTUP_TOOLCHAIN` env. Rustup precedence (`+toolchain` > `RUSTUP_TOOLCHAIN` > directory override > **`rust-toolchain.toml` file** > default) puts a committed file above the default, and ci.yml's cargo invocations are all bare `cargo` — so the file would win in all 9 lanes ⇒ matrix vacuity, exactly as `:19` now states. The "carry into FOLLOWUP resolution" instruction is present. No fold-drift.
- **I2 → `:21` (D2(c)): FOLDED CORRECTLY.** `needs: [build, vectors-roundtrip]` re-verified at `ci.yml:107`; the plan adds `fmt`. The "fmt job already fires on tag pushes" premise re-verified: `on.push.tags` covers `mk-cli-v*`/`mk-codec-v*` (`ci.yml:17-19`) and the planned fmt job carries no `if:` filter. (Note `release-on-tag`'s own `if:` at `:105` only gates the release job, not its dependencies.)
- **M1 → `:15`: FOLDED CORRECTLY + RE-VERIFIED.** PIN documented as full triple `1.95.0`; local stable re-measured = `rustc 1.95.0` (`cargo +stable fmt` = rustfmt 1.9.0-stable, same commit hash `59807616e`). Action source confirms the shorthand hazard: `^1\.[0-9]+$` inputs hit the arithmetic branch and `rustup toolchain install 1.95` floats point releases; `1.95.0` bypasses both.
- **M2 → `:27`: FOLDED CORRECTLY.** Header job list confirmed at `ci.yml:4-5`, permissions comment job enumeration at `:23-24`; no-rust-cache note present.
- **M3 → `:26`: FOLDED CORRECTLY + RE-MEASURED THIS ROUND.** `cargo +stable(1.95.0) fmt --check --all` at `13821ad` flags exactly the 5 recorded files; none of the slug's eponymous 3 appear. The FOLLOWUPS citation `design/FOLLOWUPS.md:323` is still the live entry-header line.
- **M4 → `:27`: FOLDED CORRECTLY.** `CLAUDE.md:56` re-verified as the "skip clippy/fmt gates if the shim errors" tip; plan rewords it toward the pinned fmt job.

## Verdict

**NOT GREEN — 0 Critical / 1 Important (new I3) / 3 Minor.** All six round-1 findings are folded faithfully with zero fold-drift, and both corrected claims (I1 mechanism, I2 needs-line) re-verify against the live action source and ci.yml. The blocker is new and was invisible to round 1's frame: the stale `ci.yml:94` vector-corpus path is currently shadowed by the very fmt red this plan removes — `vectors-roundtrip` has been `skipped` on every run since the corpus moved (last green run 2026-05-13 predates the move), so curing fmt converts the masked path bug into the repo's next CI red and blocks the plan's own all-green gate plus all future tag releases. Fold = one line in the same file (D2(d): `tests/vectors/` → `src/test_vectors/` at `ci.yml:94`) plus the `:29` "since 2026-06-01" correction, then re-dispatch. Everything else in the plan is verified sound.
