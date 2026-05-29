# R0 ARCHITECT REVIEW — `IMPLEMENTATION_PLAN_mk_test_hardening.md`

Opus feature-dev:code-reviewer. Mandatory pre-implementation plan-doc gate. Reviewed against live `mk-codec` (branch `mk-codec-test-hardening`, base `d9d2ed9`), rust-bitcoin **0.32.8** (`Cargo.lock:125-128`), proptest 1.x. Persisted (folds applied immediately; see fold log).

## Verification (CHECKS OUT)
- `ChildNumber::from_hardened_idx/from_normal_idx(u32) -> Result`, range `0..2^31` ✓. `DerivationPath: From<Vec<ChildNumber>>` ✓. `any::<[u8;32]>()`/`[u8;4]` (proptest Arbitrary for arrays ≤32) ✓. `prop_oneof!`/`.boxed()`/`prop_flat_map`/`prop::option::of`/`prop::collection::vec`/`prop_assume!`/`prop_assert_ne!`/`"\\PC*"` ✓.
- `KeyCard::new(Vec<[u8;4]>, Option<Fingerprint>, DerivationPath, Xpub)` (`key_card.rs:79-84`) ✓. `Xpub{..}` struct literal constructible from `tests/` (`round_trip.rs:26-33`) ✓. `Fingerprint::from([u8;4])`/`ChainCode::from([u8;32])`/`NetworkKind::{Main,Test}` ✓.
- `Error::{InvalidPolicyIdStubCount(:111), InvalidStringLength(usize)(:40), BchUncorrectable(String)(:58), CrossChunkHashMismatch(:97)}` re-exported (`lib.rs:50`) ✓. 256 stubs → `InvalidPolicyIdStubCount` before csid validation (`encode.rs:25-26`, `pipeline.rs:68`) ✓.
- 255-stub = 1096-byte bytecode → 21 chunks (< MAX 1692) ✓. Bands Regular 14..=93 / reserved 94..=95 / Long 96..=108 ✓. t=4 both codes ✓. q=0/p=1 → q↔p flip = exactly 1 symbol error ✓.
- **6-stub `multi_chunk_card` = 104-byte bytecode → 3 chunks (53/53/2): `strings[0]` data-part = 108 (Long), `strings.last()` = 25 (Regular)** ✓ (reconciled with the existing 1-stub `fixture_card_typical_chunked`). Char-index 11 = first fragment symbol past the 8-symbol header ✓.
- `decode→decode_string` length gate fires first (`pipeline.rs:128`) → T3b trim-to-97 → `InvalidStringLength(94)` deterministic ✓.
- `path.into_iter().copied()` on owned path compiles via autoref (borrow ends before move) ✓ (but see M3).

## CRITICAL — None.
Every snippet compiles against bitcoin 0.32.8 + proptest 1.x; no test flaky/vacuous/false-failing for its fixture.

## IMPORTANT
**I1** — `data_part_len(s) = len−3−8` is WRONG. `decode_string` feeds `data_part = &rest[1..]` (= `total−3`, header INCLUDED) to `bch_code_for_length` (`bch.rs:662,669`). Correct: `len−3`. Passes coincidentally for the 6-stub fixture (108 vs buggy 100, both in-band) but misrepresents the check and would misdirect the "iterate the constant" step. Fix the helper to `len−3`.
**I2** — T3b implementer note offers a never-correct `3+8+94=105` alternative and frames the definite `97` target as uncertain. The mapping is `total−3`; trim to 97 → data-part 94 → `InvalidStringLength(94)`. Remove the ambiguity; pin `InvalidStringLength(94)`.

## MINOR
**M1** — T2c `prop_assume!(strings.len() >= 1)` vacuous (encode always ≥1) + `clippy::len_zero` under `-D warnings`. Drop it.
**M2** — T4's first `stubs_255` line (`[u8;1].into()`) doesn't compile + shadows; plan note says delete but ships both. Remove the line from the plan text.
**M3** — owned `path.into_iter().copied()` may draw `clippy::into_iter_on_ref` under `-D warnings`. Use `path.as_ref().to_vec()`. (Watch; the impl note covers fallback.)
**M4** — `#![allow(dead_code)]` in `common/mod.rs` correct + necessary (subset use per test binary). No change.
**M5** — `proptest = "1"` pin cosmetic (matches ms-codec). No action.

## VERDICT: RED (0C / 2I / 5M)
No Critical — all APIs/variants/arities/band-arithmetic/chunk-counts verified true. RED solely on I1 (helper off by 8) + I2 (T3b note ambiguity). Mechanical fixes; fold + sweep M1/M2 (the real `-D warnings`/compile snags) + M3, re-dispatch R1.

---
## FOLD LOG (post-R0)
- I1 → `data_part_len` → `len.saturating_sub(3)` + corrected docstring (cites `bch.rs:662,669`).
- I2 → T3b: definitive 97→94→`InvalidStringLength(94)` value-pin; obsolete `105` note replaced with the verified mapping.
- M1 → T2c `prop_assume!` removed.
- M2 → T4 first `stubs_255` line + obsolete implementer note removed.
- M3 → `path.into_iter().copied()` → `path.as_ref().to_vec()` in `xpub_strategy` + all 3 fixtures (multi_chunk_card, T4 card, fixture_card).
- M4/M5 → no-action (confirmed correct/cosmetic).
