# R0 Review — IMPLEMENTATION_PLAN_mk_depth_child_enforcement.md

Opus architect, mandatory pre-impl R0 gate. Branch `mk-depth-child-enforcement`, base `main` `998f3c9`. Verified against live mk-codec + bitcoin 0.32.8 (Cargo.lock:141) + the R0-GREEN SPEC. Persisted by controller (review agent had no Write tool).

## Headline confirmations (file:line)
- **Task 0.1 variant COMPILES.** `error.rs` has only `use thiserror::Error;` (`:12`) — `ChildNumber` add needed, not duplicate. `#[non_exhaustive]` (`:18`); last variant `CardPayloadTooLarge` (`:156-161`), append after valid. `ChildNumber: Display` → `{xpub_child}`; `Option<ChildNumber>: Debug` → `{path_child:?}`. The `\`-continued multi-line `#[error]` literal is already used by `CardPayloadTooLarge` (`:153-155`) — no thiserror gotcha. Non-alphabetical placement correct (grouped enum, banner `:99`); plan flags "do not fix ordering."
- **Task 0.3 guard COMPILES + CORRECT.** `card.origin_path.into_iter()` yields `&ChildNumber` (3 live callsites use the idiom: `test_helpers.rs:26`, `path.rs:91`, `xpub_compact.rs:87`); `.count()`/`.last().copied()`→`Option<ChildNumber>` typecheck. `Some(card.xpub.child_number) != path_child` = exact inverse of reconstruct (`xpub_compact.rs:92-95`). `path_depth as u8`: workspace lints `clippy::all="warn"` ONLY (`Cargo.toml:14-15`), no pedantic → `cast_possible_truncation` does NOT fire under `-D warnings` (and `len() as u8` already used unflagged at `path.rs:92` etc.).
- **Top placement SOUND.** After stub checks (`encode.rs:22-27`), before `BytecodeHeader` (`:29`). Same fn = 100% emission coverage. Runs before `encode_path` (`:43`) → empty-path never reaches it (`encode_path` on empty → `0xFE,0x00`, no panic — so the move is defensive, harmless either way). Preserves `rejects_zero_stubs` (zero-stub hits `InvalidPolicyIdStubCount` at `:23` first).
- **Task 0.2 tests GATE + COMPILE.** Fixture `m/48'/0'/0'/2'` (`encode.rs:57`) = depth 4, terminal `Hardened{2}` → `depth=3` / `child=Hardened{1}` genuine mismatches. `DerivationPath::from_str("m")` = empty path (confirmed rust-bitcoin 0.32); `synthetic_xpub(empty)` = depth 0/`Normal{0}` (`.unwrap_or(Normal{0})` `test_helpers.rs:28-31`) → reject via `Some(Normal{0})!=None`, `path_child:None`. `m/44'/0'/0'/0/5` = 5 comps, NOT in `STANDARD_PATHS` (`path.rs:38-55`), aligned → encodes Ok (no false-positive). `matches!` patterns valid (trailing-comma form mirrors `rejects_zero_stubs` `:93-96`). TDD: 3 reject cells FAIL pre-guard (`matches!(Ok,Err)`=false), aligned PASSES. test-mod needs `ChildNumber` add (not duplicate, `:53`).
- **Non-regression VERIFIED crate-wide.** All fixtures derive depth/child from path → aligned. The real-xpub-string vector corpus (`vectors.rs:89-93,179`) is aligned-by-construction via `gen_mk_vectors::synthetic_xpub` from the same origin_path (`gen_mk_vectors.rs:972-977,1013-1015`); negatives route through `decode` not `encode_bytecode`. No test hand-builds a misaligned card.
- **Task 1.1 SPEC citations EXACT** (re-grepped): `:263`/`:265`/`:257`/`:292`/`:301`; before-texts verbatim; internal consistency holds (both "impossible by construction" occurrences :263+:301 edited; none left).
- **Tasks 1.2/2.1 EXACT:** FOLLOWUP `:284` (`Status: open` `:290`); `Cargo.toml:3` = 0.3.1→0.3.2 PATCH correct; no CHANGELOG (skip right).

## CRITICAL — None.   ## IMPORTANT — None.
## MINOR
- **M1** Cell 3 empty-path rejects via the CHILD clause (depth `0!=0` is false); comment could clarify. No action; cell gates correctly.
- **M2** Guard code-comment "SPEC §2.2" refers to the design-SPEC; the rule lands in `SPEC_mk_v0_1.md` §4. Optional: cite `SPEC_mk_v0_1.md §4` in the shipped comment (folded by controller).
- **M3** Task 1.2 `<SHA>` placeholder filled from the Phase-0 commit at the end-of-cycle sweep (recurring "agents forget FOLLOWUP flip" lesson) — ensure substitution.

## VERDICT: GREEN (0C / 0I / 3M)
Clear to implement. Code compiles against live types; 3 reject cells genuinely gate; aligned cells + the aligned-by-construction vector corpus stay green; guard regresses no existing test. The 3 MINORs are comment/hygiene nits.
