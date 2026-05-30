# R0 Review — SPEC_mk_depth_child_enforcement.md

Opus architect, mandatory pre-impl R0 gate. Branch `mk-depth-child-enforcement`. Verified against live `mk-codec` @ `998f3c9` + bitcoin 0.32. Persisted by controller (review agent had no Write tool).

## Headline confirmations (file:line)
- **§2.1 sole chokepoint AIRTIGHT.** `XpubCompact::from_xpub` called exactly once in production (`encode.rs:44`); both encode entries call `encode_bytecode` first (`pipeline.rs:57,68`); `encode_bytecode_stream` operates on opaque bytes; vector-gen routes through `encode_bytecode` + reuses the `[len-73..]` tail (no re-serialize). No 2nd emission path. Guard goes before `encode.rs:44`, after `encode_path` `:43`; `encode_bytecode` returns `Result` (`:21`).
- **§2.2 semantics VERIFIED.** `Xpub::depth: u8`, `child_number: ChildNumber` (enum Normal/Hardened, hardened structural, Eq+Copy → `==` exact inverse of `reconstruct_xpub` `components.last().copied()` `xpub_compact.rs:92-95`); no u32-normalize; `.into_iter().last().copied()`→`Option` is the empty-path-safe analog to reconstruct's `.expect()` (`:95`). `KeyCard.origin_path: DerivationPath`, fields `pub` (`key_card.rs:42`).
- **§2.3 variant VERIFIED.** `error.rs` `#[non_exhaustive]` (`:18`), NOT alphabetized, bytecode-layer group banner `:99` (`TrailingBytes` :146, `UnexpectedEnd` :142). `error.rs` does NOT import ChildNumber (only `use thiserror::Error;` :12) → impl adds `use bitcoin::bip32::ChildNumber;`. `ChildNumber: Display` → `{xpub_child}` compiles; `Option<ChildNumber>: Debug` → `{path_child:?}` compiles.
- **§2.3 precedent STRONGER than cited:** retired `FingerprintFlagMismatch` at `error.rs:231-236` ("structurally undetectable in the decoder … reframed as an encoder-side invariant") — direct in-codebase precedent.
- **§2.4 no-elided-origin VERIFIED (make-or-break):** `path.rs` full-path both modes (standard-table deref `lookup_indicator` :60-65, table :38-55; explicit LEB128 every component `:85-98`); depth-0 unrepresentable (`decode_explicit_path` rejects count==0 → PathTooDeep(0) `:114`). NO valid card can have `xpub.depth != component_count(origin_path)`. NO false-positive.
- **§3 SPEC_mk_v0_1.md citations ALL EXACT:** :257/263/265/292/301, +:237 cap=10, :285 PathTooDeep. **§4 FOLLOWUPs EXACT:** mk :284, toolkit :3335; toolkit check `synthesize.rs:497`.
- **SemVer:** Cargo.toml 0.3.1 (:3), bitcoin 0.32 (:31) → 0.3.2 PATCH correct.
- **Tests:** `round_trip_full_xpub_depth_4` `xpub_compact.rs:144`; `synthetic_xpub` `test_helpers.rs:22` derives depth/child from path; mismatched cards hand-buildable via `Xpub` pub fields. No mk-codec CHANGELOG.md (Glob empty) — §6 "if absent, skip" correct.

## CRITICAL — None.   ## IMPORTANT — None.
## MINOR
- **M1** §2.4 cite `path.rs:38-55` is the table literal; the deref logic is `lookup_indicator` `:60-65` — cite both for precision.
- **M2** §1 reconstruct line `:85` is fn-decl (fine); body extraction `:92-95`. No action.
- **M3** ensure §3.6:263 + §4:301 edits land mutually-consistent in the same PR (both assert enforcement; no residual "impossible by construction"). Already flagged in §7 Phase 1.

## Test-coverage: all 5 cells meaningful/buildable/non-vacuous. Cell 2 (child-mismatch-at-equal-depth) is the highest-value (depth-only checks miss it). Optional 6th: standard-table card with hand-set child ≠ dictionary terminal (e.g. indicator 0x05 = m/48'/0'/0'/2' but child=1') rejected — plan-author discretion.

## Narrative-drift: none. Faithful transcription of the GREEN design; no off-by-N.

## VERDICT: GREEN (0C / 0I / 3M)
Clear to proceed to the plan-doc / implementation. The 3 MINORs are citation-precision nits that don't gate.
