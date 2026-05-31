# R1 Re-Review — IMPLEMENTATION_PLAN_mk_derive_address.md

Reviewer: feature-dev:code-reviewer (opus). Re-review after the R0 1C/2I/4M fold
(`mk-derive-address-plan-R0-review.md`). Read the plan, the GREEN SPEC, the R0 record, and verified
every cited source against ground truth.

## Critical — None.
- **C1 RESOLVED** — `main.rs:62-67` confirmed (clap `Err` → `DisplayHelp`/`DisplayVersion` = SUCCESS,
  else `ExitCode::from(64)`). Plan pins clap parse errors to exit 64 in Task 1.3 (count/range conflict),
  Task 2.2 (path/index group), and the self-review checklist. Every surviving "exit 2" is corrective
  negation, not a residual assertion.

## Important — None.
- **I1 RESOLVED + technically correct** — the fixture-construction note (codec invariant
  `xpub.depth == origin_path.len()` + terminal child, `encode.rs:41`) is accurate; account fixtures
  genuinely liftable from `test_vectors/v0.1.json` (depth-3 44'/49'/84'/86', testnet tpub, 48'/87'
  multisig, non-standard purposes all present); the leaf recipe `acct.derive_pub(&secp, &p("m/0/5"))`
  → depth 5, child `Normal{5}`, exactly satisfying the invariant against `origin_path = m/84'/0'/0'/0/5`.
- **I2 RESOLVED** — Task 4.1 targets `crates/mk-cli/CHANGELOG.md` (heading `## [0.6.0] — <date>`),
  explicitly NOT the root `/CHANGELOG.md` (mk-codec, unchanged). Both files confirmed distinct.

## Minor — verified
- M2 (SPEC §4 count rationale corrected: manual excludes gui-schema, 6+2=8); M3 (plan line 11 secp
  precedent = toolkit `verify_message.rs:55`, not mk-cli verify.rs); M4 (Task 5.3 notes the real
  GUI header(v0.3.1)-vs-pin(v0.4.2) skew, bump both to v0.6.0).

## Fold-drift sweep — clean
No new contradiction (I1 note consistent with §5 mapping + Task 1.2); no residual "exit 2" assertion;
code snippets (enums, `infer_address_type`, `render_address`, `secp_verify`) unchanged from the
R0-verified versions (`KnownHrp` spellings, builder signatures, multisig-before-`len()` order all
intact); no internal inconsistency.

**VERDICT: GREEN (0C/0I)**

---
Post-R1: folded the reviewer's one non-blocking note — SPEC §5 tests 5 & 12 now state the clap
conflict/group errors → exit 64 explicitly (aligning the test sketches with the already-normative
§3.2 + the plan). Both gates (SPEC + plan) GREEN — implementation may begin.
