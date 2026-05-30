# End-of-Cycle R0 Review — mk-codec 0.4.0 no-path support

Opus architect (feature-dev:code-reviewer), final gate before ff-merge + publish.
Full branch diff `main...HEAD` (`5c2bc8c`→`75c517e`). Persisted by controller.

## Confirmations (file:line)
1. **Guard ⟺ reconstruct inverse holds end-to-end.** Guard `encode.rs:38-48`
   (`expected_child = path_child.unwrap_or(Normal{0})`) is the exact inverse of decode
   `xpub_compact.rs:88-97` (`child_number = components.last().copied().unwrap_or(Normal{0})`).
   depth-0: encode accepts → `encode_path` emits `[0xFE,0x00]` → `decode_explicit_path`
   accepts count==0 → empty → reconstruct depth 0/Normal{0}. Proven by `depth0_card_round_trips`.
2. **No other path regressed.** All five child-from-path sites agree on `unwrap_or(Normal{0})`
   (`xpub_compact.rs`, `encode.rs`, `test_helpers.rs`, `tests/common/mod.rs`, `gen_mk_vectors.rs`).
   No production/decode/string-layer assumption of non-empty origin_path. Surviving
   `count==0`/"non-empty" references are about `policy_id_stubs` (≥1), not paths. No straggler.
3. **No weakened test / lint issue.** The 3 inverted tests carry real positive assertions;
   new `rejects_depth0_noncanonical_child` proves the guard still rejects non-round-trippable
   depth-0; proptest bijection now genuinely samples the empty arm.
4. **SemVer + mirrors.** mk-codec 0.4.0 / mk-cli 0.5.0 / pin "0.4.0" / Cargo.lock coherent.
   No new Error variant → `error_coverage.rs` + `mk-cli error.rs:133` kind map correctly
   untouched (carry XpubOriginPathMismatch from 0.3.2). No clap-flag change → no GUI/manual lockstep.
5. **Docs E1-E10 coherent.** No residual "1..=10"/"(or == 0)" for origin_path in SPEC_mk_v0_1.md;
   superseding blockquote + inline notes neutralize the contradicting claims in
   SPEC_mk_depth_child_enforcement.md.
6. **FOLLOWUP** `mk1-no-path-depth0-support` resolved `82c015e` + companion line present.

## CRITICAL — None.  ## IMPORTANT — None.
## MINOR (informational)
- Toolkit companion `mk1-wif-bundle-depth0-invalid-card` not yet in mnemonic-toolkit
  FOLLOWUPS — correct; Phase 3 (post-publish toolkit re-pin) creates it per the
  "both entries update when the action ships" convention.
- Resolution SHA `82c015e` (phase-0 code) precedes HEAD — documented pattern (SPEC §8).
- Corpus has no depth-0 fixture — SPEC §6 marked optional; proptest+round-trip cover it.

## VERDICT: GREEN (0C/0I) — clear to ff-merge to `main` and publish mk-codec 0.4.0 + mk-cli 0.5.0.
(Controller pre-gate run: mk-codec+mk-cli tests all green incl. proptest empty-arm, clippy -D warnings clean, `cargo +stable fmt --check` exit 0.)
