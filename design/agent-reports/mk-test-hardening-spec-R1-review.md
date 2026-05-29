# R1 ARCHITECT REVIEW — `SPEC_mk_codec_test_hardening.md`

Sonnet feature-dev:code-reviewer, R1 fold-verify. Reviewed against live source (`mnemonic-key` `crates/mk-codec` @ origin/main `d9d2ed9`) + toolkit consumer. Persisted verbatim.

## I1 fold (scrutinized hardest) — §3 strategy rewrite
- **(a) Buildability CONFIRMED.** `tests/round_trip.rs:16-34` is an integration test that already constructs `bitcoin::bip32::Xpub { network, depth, parent_fingerprint, child_number, public_key, chain_code }` by struct literal from outside the crate, and passes — proving all fields are `pub` and constructible from `tests/`. The strategy as rewritten is buildable. The `test_helpers.rs:22-40` precedent is `pub(crate)`, but §3 cites the *technique*, not the function — accurate and unambiguous.
- **(b) depth/child consistency CONFIRMED** (`depth := path.len()`, `child := path.last()`); §1.1 seam sidestepped, no contradiction.
- **(c)** old `Xpriv::new_master`/"mirrors synthetic_xpub" contradiction fully removed; rewrite internally coherent (direct construction, prop_filter for ~2⁻¹²⁸ invalid scalar, network from the network axis).

## I2 fold — repair.rs path
All citation sites (§1:17, §5:77, §10:129) carry `mnemonic-toolkit/crates/mnemonic-toolkit/src/repair.rs:1001` + comment `:997-1000`. Correct. (R0's "§9" was a counting slip — §9 has no repair.rs ref.) `Mk1IndelOracle` at `:1001` with the 4-line doc comment `:997-1000` confirmed.

## M1–M5 folds
- M1 — §10 relabels `pipeline.rs:288` as the 5-burst test comment; guard = `chunk.rs:189-201`. Correct.
- M2 — §7 gate adds `-- -D warnings` + CI cite (`.github/workflows/ci.yml:58`). Sound.
- M3 — §5 carries the deterministic(T3a/b)-vs-randomized(T2c) assertion-rationale paragraph. Sound.
- M4 — §4 T2a requires a long-band (96..=108) chunk (reuse the 255-stub card) + run against both regular- and long-band chunks. Sound.
- M5 — §3 gitignore → `**/proptest-regressions/` (glob). Correct for the nested path.

## Drift hunt
No contradiction §3↔§1.1/§7/§8/§9. New citation `test_helpers.rs:22-40` resolves (function spans exactly 22-40, direct construction, `pub(crate)` note accurate). Assertion logic (T2c `!= Ok(original)`, T3a `is_err()`, T3b `InvalidStringLength`) internally consistent across §4/§5/§7/§8/§9. §9 R0 agenda still valid post-fold.

## Critical: None.  Important: None.  Minor: None.

## VERDICT: GREEN (0C / 0I)
All folds present, correct, sound. The I1 rewrite is buildable (proven), depth/child-consistent, contradiction-free. I2 complete. No fold-introduced drift. Spec ready for implementation (writing-plans → plan-doc R0 → per-phase TDD).
