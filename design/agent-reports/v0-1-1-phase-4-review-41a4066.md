# mk-codec v0.1.1 Phase 4 review — release plumbing (commit 41a4066)

**Status:** DONE_WITH_CONCERNS
**Commit:** 41a4066 (release plumbing); cross-checked 8685608, 8df9910, 2417401, 1e42354, 59878ca
**Reviewer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**File(s):**
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/Cargo.toml`
- `/scratch/code/shibboleth/mnemonic-key/CHANGELOG.md`
- `/scratch/code/shibboleth/mnemonic-key/design/FOLLOWUPS.md`
- `/scratch/code/shibboleth/mnemonic-key/README.md`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/vectors.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/pipeline.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bin/gen_mk_vectors.rs`
- `/scratch/code/shibboleth/mnemonic-key/design/MILESTONE_v0_1_1.md`
**Role:** reviewer (code)

## Summary

Phase 4 release plumbing is structurally correct: Cargo bump applied (`0.1.0` → `0.1.1`), CHANGELOG `[0.1.1]` section landed with the correct date, FOLLOWUPS items marked `resolved`, full test suite green at 156 (149 unit + 3 round_trip + 4 vectors), and corpus SHA matches its pin. **Two concerns warrant a small pre-tag fixup commit:** (1) the `cross-chunk-hash-test-fixture-stability` resolution SHA cites 8685608 but the property advertised in the FOLLOWUPS resolution prose ("perturbs at the 5-bit-symbol layer past the chunked header") only became true in 8df9910 — the 8685608 fixture perturbed inside the header; (2) the `README.md` still describes the corpus as "8-vector" in two places, which will be visibly stale immediately upon tagging v0.1.1.

A handful of minor inconsistencies are also recorded; none block the tag, but bundling them into one fixup commit before tag is cheap insurance.

## Critical

(none)

## Important

### I-1 — `cross-chunk-hash-test-fixture-stability` SHA citation is misleading

**Where:** `design/FOLLOWUPS.md:143` — `Status: resolved 8685608` for the cross-chunk-hash perturbation test fixture.

**Problem:** The Status prose claims the test "perturbs at the 5-bit-symbol layer past the chunked header." That property was *not* satisfied by 8685608 — that commit moved the perturbation to char-indices 3..8 (chunked header chars 0..5 of the 8-symbol header). The fix that moved the perturbation to the post-header region (char-indices 11..16) landed in 8df9910 (per its commit body: "I-1 + I-2: ... Moved the perturbation to chars 11..16"). Per Phase 1 review (saved as `design/agent-reports/v0-1-1-phase-1-review-8685608.md`), this was even an Important finding — i.e., 8685608 alone did not satisfy the FOLLOWUPS resolution prose; only 8df9910 did.

**Recommended:** change `resolved 8685608` to `resolved 8685608 + 8df9910` (composite citation; the precedent already exists in similar Status conventions). Alternative: cite 8df9910 alone, since that's the commit where the test reaches the property described in the FOLLOWUPS prose.

**Sequencing:** This is a documentation-accuracy issue; not a wire-format or behavior issue. Fixing it before the tag is cheap and avoids a `git blame` archaeology trail later.

### I-2 — `README.md` describes corpus as 8-vector

**Where:**
- `README.md:10`: `> a canonical 8-vector conformance corpus.`
- `README.md:49`: `Rust crate, v0.1 working encode/decode round-trip + 8-vector conformance corpus`

**Problem:** Once `mk-codec-v0.1.1` is tagged, the v0.1.1 corpus has 17 clean + 22 negative = 39 vectors. The README's "v0.1 reference implementation shipped" framing reads as the most recent shipped state; "8-vector" is concretely wrong post-tag, and is the first thing a third-party implementer browsing the repo will read.

**Recommended:** update both occurrences to "39-vector (17 clean + 22 negative) conformance corpus" or similar. The README's `Status: v0.1 reference implementation shipped` framing remains accurate — the framing describes v0.1.x, not v0.1.0 specifically — but the corpus-count claim must reflect v0.1.1.

(Phase 4's milestone plan §4.5 doesn't list a README update, but the README isn't pinned to v0.1.0; it should track current shipped state. This is a fair Phase 4 inclusion.)

## Minor

### m-1 — Tier label `v0.2-nice-to-have` doesn't appear in the FOLLOWUPS rubric

**Where:** `design/FOLLOWUPS.md:126` — `error-variant-exhaustiveness-gate-strum` carries `Tier: v0.2-nice-to-have`. The rubric (line 17) lists tiers as `v0.1-blocker | v0.1-nice-to-have | v0.2 | pre-bip-submission | cross-repo | v1+ | external`. There is no `v0.2-nice-to-have`.

**Recommended:** either rename to `v0.2` (matches rubric exactly) or extend the rubric to include `v0.2-nice-to-have` (parallel to `v0.1-nice-to-have`). Either is fine; consistency is the point. Same wording reproduced in `CHANGELOG.md:38`.

### m-2 — Stale forward-reference comment in `tests/vectors.rs:153`

**Where:** `crates/mk-codec/tests/vectors.rs:153` — `// Phase 4 will tighten these to floor checks if v0.1.x adds vectors.`

**Problem:** Phase 4 is now done. The asserts at lines 154–155 are already floor checks (`>=`), so the comment is technically describing the current state, not future work. Reads slightly oddly post-Phase-4.

**Recommended:** rewrite as a forward-stable note, e.g., `// Floor-checks: v0.1.x can grow the corpus without breaking existing implementations consuming this harness.`

### m-3 — CHANGELOG omits the N17 reshape from 59878ca

**Where:** `CHANGELOG.md:24-28` — describes 22 negative vectors as "N1..N21, N23 — one per `Error` variant reachable from `decode`'s string-input path." Phase 3 fixup (59878ca) reshaped N17 from `UnexpectedEnd` (the original 1e42354 implementation) to `InvalidPathComponent` (LEB128 overflow), which actually achieved the variant-parity claim — without 59878ca, the N17 vector would not reach `InvalidPathComponent`.

**Problem:** the `Resolved (FOLLOWUPS)` line for `decoder-error-variant-parity` cites only 1e42354, not the N17 reshape that completed parity. A third-party implementer reading the CHANGELOG won't realize 59878ca was material to the resolution.

**Recommended:** either cite both SHAs in FOLLOWUPS (`resolved 1e42354 + 59878ca`) or add a CHANGELOG sub-bullet noting the N17 reshape. Low priority — the CHANGELOG aggregates the milestone, not individual fixup commits — but worth considering for Phase 3 review-trail integrity.

### m-4 — Wire-format invariant claim is technically correct but could be more precise

**Where:** `CHANGELOG.md:11-12` — `**Wire format byte-identical to v0.1.0**; existing v0.1.0 strings round-trip unchanged through the v0.1.1 decoder.`

**Observation:** Verified true. `Error::MixedHeaderTypes` is an API-behavior change, not a wire-format change. The 9 new clean vectors exercise indicators that already had encoder/decoder support in v0.1.0 (the `bytecode/path::round_trip_all_standard_paths` unit test cycled the full dictionary). The 22 negative vectors test rejection paths already present in v0.1.0. The only behavior delta is which `Error` variant is returned for two specific inputs (forward-direction `[SingleString, Chunked]` and reverse-direction `[Chunked, SingleString]`), which the `Notes` section already calls out.

**Suggestion (very minor):** could explicitly distinguish "wire format unchanged" (string output bytes) from "API behavior changes for header-mixing inputs" (the `MixedHeaderTypes` migration). The current `Notes` section does this implicitly; could lift it to a one-liner under `Changed`. Not blocking.

### m-5 — `Error::CardPayloadTooLarge` exemption rationale wording

**Where:** `CHANGELOG.md:27-28` — `Error::CardPayloadTooLarge is encoder-only and exempt from corpus coverage; documented in the exhaustiveness gate.`

**Observation:** Correct. The variant fires inside `split_into_chunks` on bytecode > 1693 B; the decoder cannot reach it because no string-list input can decode into a >1693-byte bytecode (the chunked-header total_chunks max is 32; max bytecode payload is bounded). A future encoder change could expose this if e.g. SPEC §3.6 Compact-65 lands; the exemption rationale should be tied to the wire format, not just "encoder-only" in casual prose.

**Suggestion:** if a follow-up commit lands, consider strengthening to "encoder-only at the v0.1 wire-format size bounds; re-evaluate if a future format extension changes the size envelope." Not blocking.

### m-6 — CHANGELOG date semantics

**Where:** `CHANGELOG.md:8` — `## [0.1.1] — 2026-04-29` (matches v0.1.0's date `2026-04-29`).

**Observation:** Both releases dated the same calendar day. Technically accurate (the commits all landed today), but slightly unusual to have two release entries on the same date. No action needed; just observed.

## Observations

### O-1 — Phase 4 milestone scope adherence

The milestone plan (§Phase 4) listed four tasks: Cargo bump, CHANGELOG, FOLLOWUPS status updates, tag. The commit landed the first three; tag deferred per established discipline (the user controls the remote push). The commit body explicitly notes this. Adherence: clean.

§4.5 final reconciliation lists "verify every minor item is either resolved or recorded back into FOLLOWUPS at an appropriate tier" — that step is the implicit close-out for this review. The minor items above are the ones that should be either fixed inline or recorded in FOLLOWUPS.

### O-2 — Tag readiness checklist

Verified state immediately before the tag operation:

- [x] `crates/mk-codec/Cargo.toml` `version = "0.1.1"` ✓
- [x] `Cargo.lock` updated (1 line; mk-codec entry version bumped) ✓
- [x] CHANGELOG `[0.1.1]` section present ✓
- [x] FOLLOWUPS items marked `resolved <SHA>` for all four targets ✓
- [x] Full test suite green at 156 ✓
- [x] Corpus SHA matches pin (`a91828ed2ecf5f0f17daa86f7df6493cb10d1837f474ff8798a48bc63a161023`) ✓
- [ ] (recommended) README corpus-count update from "8-vector" to current count
- [ ] (recommended) FOLLOWUPS `cross-chunk-hash-test-fixture-stability` SHA citation precision (8685608 → 8685608+8df9910 or 8df9910)
- [ ] (recommended) tier label `v0.2-nice-to-have` reconciled with rubric

### O-3 — `every_error_variant_has_negative_vector` runtime gate

The CHANGELOG (line 36-38) notes this is a runtime substring gate rather than the milestone-planned compile-time match, with the migration recorded as `error-variant-exhaustiveness-gate-strum` for v0.2. This was Phase 3 review's I-1 finding, applied as the FOLLOWUPS entry. The CHANGELOG transparency is appropriate.

### O-4 — Commit message quality

The 41a4066 commit body is well-structured: explicitly enumerates each FOLLOWUPS resolution with its closing SHA, distinguishes `wont-fix` from `resolved`, calls out the deferred tag operation. No issues.

## Recommended pre-tag actions

A single fixup commit before `git tag` would address:

1. **README.md** — both "8-vector" mentions updated to current count.
2. **design/FOLLOWUPS.md** — `cross-chunk-hash-test-fixture-stability` SHA: `resolved 8685608 + 8df9910` (or `8df9910` alone).
3. **design/FOLLOWUPS.md + CHANGELOG.md** — `v0.2-nice-to-have` tier label reconciled with the rubric.
4. *(optional)* **tests/vectors.rs:153** — comment rewrite from forward-reference to current-state framing.
5. *(optional)* **CHANGELOG.md** — note 59878ca's N17 reshape contribution to `decoder-error-variant-parity` resolution (or update FOLLOWUPS SHA citation analogously to (2)).

Items (1)-(3) are documentation-stability concerns: future readers of the v0.1.1 release artifact (README, CHANGELOG, FOLLOWUPS) hit them immediately. Items (4)-(5) are observational-grade; defer to FOLLOWUPS if not bundled.

After the fixup, `git tag -a mk-codec-v0.1.1 -m "..."` is safe to run.

## Conclusion

Phase 4 is technically correct and the code state is shippable. Tag-readiness is bounded by 2-3 small documentation accuracy issues that future readers will encounter; bundling them into one short fixup commit before the tag preserves audit trail integrity. No blockers.
