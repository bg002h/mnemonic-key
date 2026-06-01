# A2 PLAN R0 review — mk SLIP-0132 acceptance

**PLAN:** `design/IMPLEMENTATION_PLAN_mk_slip0132_acceptance.md`
**SPEC (R0 GREEN):** `design/SPEC_mk_slip0132_acceptance.md`; SPEC R0 review `design/agent-reports/mk-slip0132-spec-R0-review.md`
**Source SHA reviewed:** mk `main`/branch `mk-slip0132-acceptance` HEAD `fc2341b`; toolkit `master` (lockstep target).
**Reviewer:** opus architect (R0, adversarial). Verified against LIVE source + empirical bitcoin-0.32 type/behavior probes + an empirical `-D warnings` dead_code probe. Phase-2 lesson (test-invocation/fixture fidelity) given extra scrutiny.

## Verdict: RED (1C / 1I)

The plan is byte-correct (all 8 version arms match the CI-tested toolkit table), the predicate compiles + behaves correctly against real bitcoin 0.32 (hardenedness + short-path no-panic empirically confirmed), the fixtures round-trip the real `V2_84_MAIN`/`V1_48_MULTISIG` corpus xpubs to identical canonical keys, and the lockstep count/SemVer/manual targets all verify. BUT the plan carries one **Critical** factual error that produces a CI-failing dead-code build (`parse_xpub` is orphaned once both its callers are rewired — the plan's "keep it for a non-card caller" rationale is false; there is no other caller), and one **Important** under-specification of the dead-code fix + the watch-only literal byte. Fold both, re-dispatch.

---

## Critical

- **C1 — `parse_xpub` becomes dead code and FAILS CI's `-D warnings` once A1+A3 rewire its only two callers; the plan's stated rationale to keep it is factually wrong.**
  - **Evidence (live source):** `grep -rn parse_xpub crates/` returns exactly three hits beyond the definition: `cmd/encode.rs:11` (import) + `:85` (call), and `cmd/verify.rs:11` (import) + `:53` (call). The definition is `cmd/mod.rs:57` `pub fn parse_xpub`. There are **zero** other callers (no re-export, no test reference). A1 Step 6 replaces the encode call+import; A3 Step 3 replaces the verify call+import. After A3, `parse_xpub` has no remaining reference. The plan's `detect_and_normalize` does NOT call `parse_xpub` — it uses a private `from_str` closure (PLAN :146-148) — so it is genuinely orphaned.
  - **Why it's CI-fatal:** mk-cli is a **binary-only** crate (`main.rs:15-17` `[[bin]] name="mk"`; `cmd` is a *private* module `mod cmd;` at `main.rs:9`), so a `pub fn` with no in-crate caller is dead-code. CI runs `cargo clippy --workspace --all-targets -- -D warnings` (`.github/workflows/ci.yml:58`). Empirically confirmed: an unused `pub fn` in a private module of a bin crate yields `error: function ... is never used` / `= note: -D dead-code implied by -D warnings` → build fails. (Precedent in this very crate: `output_advisory.rs:16` and `error.rs:28` carry explicit `#[allow(dead_code)]` for exactly this bin-only-crate condition.)
  - **The plan's wrong claim:** A1 Step 5 (PLAN :172) says "keep `parse_xpub` for any non-card caller; the new helper adds the note + check". There is **no non-card caller** — both callers are the encode/verify card paths the plan rewires.
  - **Timing nuance (why this still trips the executor):** after A1 only encode is rewired; `verify.rs:53` keeps `parse_xpub` alive, so A1 Step 7 clippy passes. The function only goes dead after A3 Step 3, and A3 Step 5 clippy would surface it — but by then the plan has explicitly instructed the executor to KEEP the now-dead function, so the executor hits an unexplained clippy failure and must improvise a fix mid-phase with no specified remedy. This is precisely the "would this actually compile/pass as written" defect class Phase-2 flagged.
  - **Fix:** Delete `pub fn parse_xpub` (`cmd/mod.rs:57-59`) as part of A3 Step 3 (the step that removes its last caller), and drop the dangling `parse_xpub` from both `use crate::cmd::{…}` lists (`encode.rs:11`, `verify.rs:11`). Stage `cmd/mod.rs` in the A3 commit. Correct the A1 Step 5 rationale text to "`parse_xpub_normalized` REPLACES `parse_xpub` at both (only) call sites; `parse_xpub` is removed in A3 once verify is rewired." (Alternatively, if a future non-card caller is genuinely wanted, route `parse_xpub_normalized`'s canonical branch through `parse_xpub` so it stays live — but the cleaner option is deletion, matching the actual call graph.)

## Important

- **I1 — A1 lands `slip132.rs` (≈6 `pub` items) but only Task A1 Step 7 clippy must stay GREEN; the plan must guarantee EVERY `pub` item is reached by non-test code at the A1 commit, and the dead-code remedy for `parse_xpub` (C1) must be specified in the step that orphans it, not left to improvisation.** Two concrete sub-points:
  - **(a) `slip132.rs` pub-item reachability at A1:** the encode wiring (`parse_xpub_normalized` → `detect_and_normalize` → on `Some(variant)`: `label()`, `canonical_label()`, `path_matches()`, on mismatch `mismatch_help()`) reaches `detect_and_normalize`, `label`, `canonical_label`, `path_matches`, `mismatch_help`, and the `Slip132Variant` enum — ALL via the encode path, including the error branch (a call on an error branch IS a use; dead-code only fires on never-referenced items). This is sound **provided** `parse_xpub_normalized` (added in A1 Step 5, used by encode in A1 Step 6) is committed in the SAME A1 commit as `slip132.rs` — the plan does co-locate them (A1 Step 8 stages `slip132.rs` + `mod.rs` + `encode.rs` together), so A1 is internally clean. Confirm no `pub` item in `slip132.rs` is unreachable: `Slip132Variant` (constructed in `detect_and_normalize`), `label`/`canonical_label` (called in `parse_xpub_normalized`'s note), `path_matches` (called in the predicate), `mismatch_help` (called on the refuse branch), `detect_and_normalize` (called by `parse_xpub_normalized`). All reachable. ✓ — keep this guarantee explicit in A1 Step 7.
  - **(b) the C1 remedy belongs in A3 Step 3 text.** As written, A3 Step 3 only says "Update the `use parse_xpub` → `parse_xpub_normalized` at `:11`" — it does NOT instruct deleting the now-orphaned `pub fn parse_xpub`. Add the deletion + the `cmd/mod.rs` staging to A3 Step 3 and A3 Step 6's `git add` (currently A3 Step 6 stages only `verify.rs` + the test file — it MUST also stage `crates/mk-cli/src/cmd/mod.rs`).
  - **(c) watch-only literal byte (M2 cell, A3 Step 4):** the live advisory text is `note: stdout is watch-only \u{2014} public keys only, cannot spend` (`output_advisory.rs:31`) — the separator is U+2014 EM DASH, not a hyphen. The plan's A3 Step 4 prose writes the literal `note: stdout is watch-only — public keys only, cannot spend` (em-dash). The integration cell MUST assert the em-dash byte (`\u{2014}` or the literal `—`), not `-`/`--`, or the `stderr.contains(...)` check silently never matches and the ordering assertion is vacuous. State this explicitly in A3 Step 4 (mirror the byte from `output_advisory.rs:31`). The toolkit's cross-repo byte-parity precedent (`—` == `\u{2014}`) applies.

---

## Minor

- **M-a — verify `want_path` reuse (A3 Step 3) is mechanically correct but under-pinned.** Live `verify.rs:84-93` parses a LOCAL `let want = parse_derivation_path(expected)?;` inside `if let Some(expected) = &args.origin_path { … }` and compares `want != card.origin_path`. The plan hoists `want_path: Option<DerivationPath>` before the xpub parse (`:53`) and passes `want_path.as_ref()` to `parse_xpub_normalized`. To "not parse twice," the executor must then change the `:84-93` block to consume the hoisted `want_path` (e.g. `if let Some(want) = &want_path { if want != &card.origin_path { … } }`) and drop the inner `parse_derivation_path`. The plan says "reuse `want_path`" but doesn't show the rewritten `:84-93` block. Spell it out so the executor doesn't leave a double-parse OR a borrow/move error. (Not blocking — it's a small, well-bounded edit; just pin it.)

- **M-b — `parse_derivation_path` failure for a malformed `--origin-path` on verify now occurs BEFORE the BCH decode** once the path is hoisted above `:50`'s `mk_codec::decode`. Today a bad `--origin-path` on verify errors only at `:85` (after decode). Hoisting moves that UsageError earlier. Behavior change is benign (still exit 64, still UsageError, just ordered before the decode) and arguably better, but worth a one-line note so a reviewer/transcript diff isn't surprised. No test asserts the old ordering (verified: no verify test feeds a malformed `--origin-path`). Non-blocking.

- **M-c — `#![allow(missing_docs)]` already suppresses missing_docs crate-wide (`main.rs:7`).** The plan repeatedly says "doc every pub item" to avoid `missing_docs` (A1 Step 7; carried from SPEC §10). That belt-and-suspenders is harmless and good hygiene, but the actual `-D warnings` risk for `slip132.rs` is **dead_code**, not missing_docs (the latter is allowed at the crate root). Reframe the A1 Step 7 gate note to "every `slip132.rs` pub item reachable from the encode path (dead_code)" — that is the real gate. Non-blocking.

---

## Spec-coverage matrix (§ → task → ✓/gap)

| SPEC § | Requirement | Plan task | Status |
|---|---|---|---|
| §1 Goal | accept ypub/zpub/…+testnet, normalize, note, refuse-on-mismatch | A1 (`slip132.rs` + `parse_xpub_normalized`) | ✓ |
| §2 version table (10 entries) | 8 non-canonical arms + 2 canonical fall-through | A1 Step 3 match arms | ✓ (all 8 bytes match toolkit; see verification table) |
| §3 `detect_and_normalize` | decode→swap→reencode→from_str; None for canonical; fall-through for unknown | A1 Step 3 | ✓ (empirically round-trips; fall-through preserves today's error) |
| §4 stderr note (exit 0) | one `note:` line, per-variant label, none for canonical | A1 Step 5 (`parse_xpub_normalized` eprintln) | ✓ |
| §5 mismatch refusal (exit 64), ACTIONABLE | `UsageError` + `mismatch_help` naming both sides + fix | A1 (`mismatch_help`) + A2/A3 cells | ✓ |
| §6 encode vs verify boundary | encode always has path; verify path-OPTIONAL, skip predicate when absent | A1 (encode, `Some(&path)`) + A3 (verify, `want_path.as_ref()`) | ✓ |
| §7 exit codes | mismatch=64 (UsageError), unknown-version=existing from_str error | A2/A3 cells assert 64; fall-through preserves | ✓ (error.rs:85 UsageError→64 confirmed) |
| §8 lockstep | mk-codec untouched; mk-cli MINOR 0.7.0; no GUI/flag-coverage; 3 toolkit pins; toolkit PATCH | A4 (bump) + B1 (3 pins + manual + 0.38.4) | ✓ |
| §9 test plan | unit byte-parity+predicate; integration happy/mismatch/multisig/canonical/verify | A1-A3 cells | ✓ |
| §10 footguns | two-wall intercept, re-checksum, short-path guard, depth/child interaction, verify-no-path, canonical-no-check, doc/dead-code | A1-A3 design | ✓ (short-path + hardenedness empirically verified; dead-code gap = C1) |
| §11 R0 fold M1/M2/M3 + precedent | exit-64 doc+assert; stderr-order cell; HARDENED predicate; byte-parity unit | A2 Step 1 (byte-parity), A3 Step 4 (M2), A1 predicate (M3), A2/A3 (M1) | ✓ (M2 cell needs em-dash byte — I1c) |

No spec gaps. All §1-§11 land in a concrete task.

## Code/API & invocation verification (✓/✗)

- **8 SLIP-0132 version-byte match arms (PLAN :153-160) vs toolkit `slip0132.rs:82-90` (CI-tested):** ypub `049D7CB2` ✓, zpub `04B24746` ✓, Ypub `0295B43F` ✓, Zpub `02AA7ED3` ✓, upub `044A5262` ✓, Upub `024289EF` ✓, vpub `045F1CF6` ✓, Vpub `02575483` ✓. Canonical targets `XPUB_MAINNET=[04,88,B2,1E]` / `TPUB_TESTNET=[04,35,87,CF]` ✓. PLAN test const `ZPUB_V=[04,B2,47,46]` ✓. **ALL 8 + 2 + test-const MATCH.**
- **`ChildNumber::Hardened { index }` (bitcoin 0.32.100 bip32.rs:133-135):** variant + field name correct; `index` is the BARE number (`from_hardened_idx(index)→Hardened{index}`, line 156-158; bare 84 not 84|2^31). Predicate `*index == 49|84|48|1|2` compares bare → ✓.
- **`path.as_ref() → &[ChildNumber]`:** `impl AsRef<[ChildNumber]> for DerivationPath` (bip32.rs:332) → ✓.
- **Predicate compiles + behaves (empirical probe):** `m/84'/0'/0'`→zpub TRUE; `m/84/0/0` (unhardened)→FALSE (M3 ✓); `m/49'/0'/0'`→zpub FALSE; `m/48'/0'/0'/2'`→Zpub-multisig TRUE; `m/48'/0'/0'/1'`→Zpub-multisig FALSE; `m/48'/0'` (2-comp)→FALSE, NO PANIC (short-path guard ✓). `c.get(3)→None→matches!` false. ✓
- **`to_slip132` inverse-swap + normalize round-trip (empirical, real fixtures):** `V2_84_MAIN`(depth-3)→zpub→normalize→byte-identical canonical Xpub, sig `"zpub"`, depth 3 ✓; `V1_48_MULTISIG`(depth-4)→Zpub→normalize→identical Xpub, sig `"Zpub"`, depth 4 ✓. base58check re-checksum correct (SPEC §10). ✓
- **Fixtures exist at asserted paths/depths:** `cli_address.rs:17` `V2_84_MAIN` (byte-identical to PLAN :31) used at `m/84'/0'/0'` (lines 78/89/96/110…); `cli_address.rs:19` `V1_48_MULTISIG` used at `m/48'/0'/0'/2'` (line 123). Both pass the codec depth/child guard at those paths (existing passing tests). ✓
- **encode wiring (PLAN :196-200):** `encode.rs:84` `let path = parse_derivation_path(&args.origin_path)?;` then `:85` `let xpub = parse_xpub(&args.xpub)?;` → replace `:85` with `parse_xpub_normalized(&args.xpub, Some(&path))?`. `Some(&path)` is `Option<&DerivationPath>` — matches the helper sig. ✓ `--origin-path` is required `String` (`encode.rs:27-28`). ✓
- **verify wiring (PLAN :243-252):** `verify.rs:31` `origin_path: Option<String>`, `:23` `xpub: Option<String>`, xpub parsed `:53`, existing path content-match `:84-93`. Hoist `want_path` before `:53`, pass `.as_ref()`. ✓ (reuse needs the `:84-93` block rewrite — M-a).
- **`parse_xpub_normalized` eprintln idiom:** matches mk-cli's stderr idiom — encode uses `writeln!(stderr,…)`/the advisory uses `writeln!`; verify/main use `eprintln`-class via `emit_error`. `eprintln!` is consistent with the crate's stderr usage. ✓
- **`parse_xpub` dead-code after rewire:** 2 callers only (encode:85, verify:53), both rewired → ORPHANED → CI `-D warnings` FAIL. **✗ → C1.**
- **Happy-path determinism (PLAN :57):** zpub-derived mk1 stdout == canonical-xpub-derived mk1 stdout — normalize yields byte-identical Xpub, `mk_codec::encode` deterministic, note is stderr-only (encode.rs note + advisory both stderr), stdout = mk1 strings only → equality holds. ✓
- **exit-code asserts:** UsageError→64 (`error.rs:85`); ContentMismatch→4 (`error.rs:84`); M1 cell asserts 64 (distinct from 4). ✓ clap parse errors→64 (`main.rs:71`) — irrelevant here (valid args). ✓
- **stderr-ordering (M2, A3 Step 4):** SLIP-0132 note fires at parse (inside `parse_xpub_normalized`, called `encode.rs:85`) BEFORE stdout; watch-only advisory fires `encode.rs:97-100` AFTER stdout. Two distinct stderr `note:` lines, SLIP-0132 first. `verify.rs` has NO `output_advisory` call (inert — confirmed) so the M2 cell is correctly encode-scoped. ✓ (cell must assert the em-dash byte — I1c).
- **bin-crate unit-test target / test deps:** `#[cfg(test)] mod tests` already lives in `src/` files (`process_hardening.rs`, `cmd/derive_support.rs`) run via the bin target (`cargo test --workspace` / per-crate `-p mk-cli`); A2 Step 2's "re-grep how mk-cli runs unit tests" is satisfied — bin-target, same as Phase 2. `assert_cmd` + `bitcoin` + `bitcoin::base58` available in `tests/` (cli_address.rs:11-13 idiom; round_trip.rs:8 `std::process::Command`+`CommandCargoExt` idiom = exactly what PLAN :27-28 uses). ✓

## Lockstep (Phase B) verification

- **3 mk-cli pin sites at `mk-cli-v0.6.1`:** `scripts/install.sh:41` ✓, `.github/workflows/manual.yml:77` ✓, `.github/workflows/quickstart.yml:71` (the `run: cargo install … --tag mk-cli-v0.6.1 mk-cli` line) ✓. → bump all 3 to `mk-cli-v0.7.0`.
- **md/ms pins untouched:** separate `component_info` lines `descriptor-mnemonic-md-cli-v0.6.2` (install.sh:35) + `ms-cli-v0.5.0` (install.sh:38) — not matched by an mk bump. ✓ `sibling-pin-check.yml` gates 3-site consistency.
- **No GUI schema change:** `--xpub` VALUES widened, NAME unchanged → no `schema_mirror` (flag-NAME gate) change, no manual flag-coverage change. ✓
- **Manual chapter:** `docs/manual/src/40-cli-reference/44-mk-cli.md` exists (16.5KB), contains NO ypub/zpub today → net-new prose (B1 Step 2). Correct chapter. ✓
- **Toolkit version + README markers:** `crates/mnemonic-toolkit/Cargo.toml` version `0.38.3`→`0.38.4` ✓; README markers `<!-- toolkit-version: 0.38.3 -->` at `README.md:13` AND `crates/mnemonic-toolkit/README.md:9` (both must update — `readme_version_current` gate) ✓; CHANGELOG `[0.38.3]` entry present → add `[0.38.4]`. ✓
- **No CI-gated mk transcript ingests ypub/zpub** (B1 Step 4 re-sweep) — consistent with SPEC; verify at impl (no expected re-capture).

## SemVer / scope

- mk-cli MINOR `0.6.1 → 0.7.0` (current `Cargo.toml:3` = `0.6.1`) — purely additive (every SLIP-0132 prefix was wholly REFUSED before; no previously-accepted input changes behavior). ✓
- mk-codec UNTOUCHED (normalization lives entirely in mk-cli; `KeyCard::new` receives a canonical `Xpub` exactly as today; no new `mk_codec::Error`). ✓
- Toolkit re-pin = PATCH. ✓ No scope creep observed (the only structural change beyond `slip132.rs`+wiring is the C1-mandated `parse_xpub` deletion, which is in-scope cleanup).

## Notes

- **Strongest signals:** (1) all 8 version bytes are byte-for-byte the toolkit's CI-tested table (round-trips published SLIP-0132/BIP-84 vectors); (2) the predicate + normalize were empirically compiled AND behavior-tested against real bitcoin 0.32 and the real `V2_84_MAIN`/`V1_48_MULTISIG` fixtures — hardenedness (M3), short-path no-panic, and byte-identical key round-trip all PASS; (3) the dead-code failure mode was empirically reproduced under `-D warnings`.
- **The single blocker is C1** — a one-line factual error in A1 Step 5's rationale that, followed literally, leaves `parse_xpub` orphaned and fails CI clippy at A3. The fix is mechanical (delete the orphan in A3, stage `cmd/mod.rs`). I1 packages the dead-code remedy placement + the em-dash literal + the explicit pub-item-reachability guarantee.
- After folding C1 + I1 (+ the three Minors), this plan is execution-ready: spec coverage is complete, the core is byte-correct and empirically validated, and the lockstep/SemVer all verify against live source.
- Re-dispatch the architect after the fold (CLAUDE.md: reviewer-loop continues after every fold — folds can introduce drift, especially around the A3 `git add` set and the verify `:84-93` rewrite).
