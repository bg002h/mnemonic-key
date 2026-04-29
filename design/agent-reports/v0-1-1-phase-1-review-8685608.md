# Review — v0.1.1 Phase 1 (commit 8685608)

**Status:** DONE_WITH_CONCERNS
**Commit:** 8685608 (`fix(mk-codec phase 1.1): clear v0.1-nice-to-have backlog`)
**Reviewer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**File(s):**
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/error.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/pipeline.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/chunk.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/header.rs` (cross-check)
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/bch.rs` (cross-check)
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/key_card.rs` (cross-check)
- `/scratch/code/shibboleth/mnemonic-key/design/FOLLOWUPS.md`
- `/scratch/code/shibboleth/mnemonic-key/design/MILESTONE_v0_1_1.md`
- `/scratch/code/shibboleth/mnemonic-key/design/agent-reports/v0-1-1-milestone-review-draft.md`
**Role:** reviewer (code)
**Test status (verified):** 149 unit + 3 round_trip + 3 vectors = 155 tests pass; 0 ignored.

## Summary

Phase 1 lands cleanly at the variant + plumbing + FOLLOWUPS-closure level. Tests pass, the new `Error::MixedHeaderTypes` variant is well-named with accurate rustdoc, both call-site migrations match the milestone plan, and the wont-fix closure for `encode-with-chunk-set-id-singlestring-silent-ignore` is sound with a usable sequencing requirement. There are **two important issues to fix before Phase 2** — both in Task 1.1 (the new perturbation test): the in-code comment block describing where the perturbation lands is wrong relative to the actual code, and the test's accept-error set may be too narrow given where the perturbation actually lands. There is **one important rustdoc-staleness issue** in `pipeline::decode` carried over from the v0.1.0 docs that was missed during the Task 1.2 migration. None of these block proceeding to Phase 2 in principle, but they should be folded into a Phase 1 fixup commit before Phase 2 to keep the audit trail tidy.

---

## Critical

_None._

---

## Important

### I-1. Task 1.1 test: in-code comment describes a different perturbation region than the code actually perturbs

**Where:** `crates/mk-codec/src/string_layer/pipeline.rs:301-322`.

The comment block at lines 301-311 claims:

> _"Perturb 5 consecutive characters in the LAST chunk's data part, starting at position 11 within the chunk's string — i.e., 8 symbols into the data part (past the 8-symbol chunked header). This places the burst inside the bytecode-fragment region; for the typical 84-byte card, the last chunk is the regular-code chunk holding bytecode tail + 4-byte cross-chunk hash, so the perturbation falls within the cross-chunk-hash region with high probability and exercises that rejection path."_

The code at `pipeline.rs:316-321`:

```rust
for c in chars.iter_mut().take(8).skip(3) {
    *c = if *c == 'q' { 'p' } else { 'q' };
}
```

`take(8).skip(3)` iterates indices `3..8` (5 chars). The `mk1` HRP+separator occupy indices 0..2, so the perturbation lands at the **first 5 chars of the data part**. For a chunked encoding, data-part positions 0..7 are the 8-symbol chunked header (`version`, `type`, four `chunk_set_id` symbols, `total_chunks`, `chunk_index`). Indices 3..8 within the *string* therefore map to data-part positions 0..4 — the first 5 chunked-header symbols (`version`, `type`, three of the four `chunk_set_id` symbols).

**The comment claims the burst lies in the bytecode-fragment / cross-chunk-hash region; in reality it lies entirely inside the chunked header.** Either the comment is wrong or the code is wrong, but they disagree. This matters because the documented property under test ("perturbation falls within the cross-chunk-hash region with high probability") is not what the code actually exercises.

**Recommended fix:** decide whether the test wants header-region or fragment-region perturbation, then align comment + code. The test name `decode_rejects_5_symbol_burst_in_last_chunk_data_part` is direction-neutral and doesn't constrain the choice. If the intent is "any 5-symbol burst in the data part," the current code is fine and the comment needs trimming. If the intent is "burst in the bytecode-fragment region exercising the cross-chunk-hash path specifically," the iteration bounds need to skip past the 8-symbol header (e.g., `chars.iter_mut().take(11+5).skip(11)` to perturb data-part indices 8..12, the first 5 fragment symbols). Either is acceptable; the choice should be made consciously.

### I-2. Task 1.1 test: accept-error set may be too narrow given the actual perturbation region

**Where:** `crates/mk-codec/src/string_layer/pipeline.rs:328-334`.

The test asserts:

```rust
match decode(&parts) {
    Err(Error::CrossChunkHashMismatch) | Err(Error::BchUncorrectable(_)) => (),
    other => panic!(...),
}
```

**Given that the burst (per I-1's analysis) actually lands in the chunked header**, BCH outcomes split as follows:

1. BCH returns `Err(BchUncorrectable)` (5 errors > t=4 → most common outcome). Test passes. ✓
2. BCH "corrects" to a wrong-but-valid codeword (5 errors fit a degree-≤4 locator polynomial in some pathological pattern → rare but possible). The "corrected" data part now contains a chunked header whose fields may be:
   - `version != 0` → `Err(UnsupportedVersion(_))` from `header::from_5bit_symbols:125`. **Not in accept set.**
   - `type` ∈ `{0x02..=0x1F}` → `Err(UnsupportedCardType(_))` from `header::from_5bit_symbols:174`. **Not in accept set.**
   - `total_chunks == 0` or `> MAX_CHUNKS` → `Err(ChunkedHeaderMalformed(_))` from `header::from_5bit_symbols:155`. **Not in accept set.**
   - `chunk_index >= total_chunks` → `Err(ChunkedHeaderMalformed(_))` from `header::from_5bit_symbols:160`. **Not in accept set.**
   - `chunk_set_id` mutated to a value different from chunk[0]'s csid → `Err(ChunkSetIdMismatch)` from `chunk.rs:150`. **Not in accept set.**
   - Header decodes cleanly (csid happens to match, all field values valid) → falls through to bytecode-fragment decoding; if the bytecode is corrupted, ultimately `CrossChunkHashMismatch`. ✓ In accept set.

The test currently passes empirically (BCH gives up on this specific fixture — outcome 1), but the property being asserted is "5-symbol burst is rejected via either of two specific variants." Under the actual perturbation region (header), a wrong-but-valid BCH correction has 5 distinct legitimate rejection paths, only one of which is in the accept set.

This is the **same brittleness class as the v0.1.0 test that the milestone explicitly aims to remove** (the v0.1.0 test pinned to `CrossChunkHashMismatch` only; future fixture changes could mask the test). The replacement test improves robustness by accepting two variants instead of one, but if I-1's region is preserved, the accept set should be widened or the implementation should be adjusted to keep the burst inside the bytecode-fragment region (where header-decode rejection paths can't fire).

**Recommended fix:** if I-1 is resolved by moving the burst into the fragment region (post-header), the current accept set is correct. If the burst region stays in the header, the accept set should be widened to `{BchUncorrectable, CrossChunkHashMismatch, UnsupportedVersion, UnsupportedCardType, ChunkedHeaderMalformed, ChunkSetIdMismatch}` — or, equivalently, `Err(_) ⇒ ()` with a sharper rejection assertion (e.g., "any rejection is acceptable; the property is _that_ it was rejected, not _how_").

The first option (move the burst) is preferable: it keeps the test's stated property tight ("perturbation in the cross-chunk-hash region exercises the BCH-decode + cross-chunk-hash paths") and matches the milestone plan §1.1.1 verbatim, which says "modify **at least 5 payload symbols** that lie inside the cross-chunk-hash region."

The fact that all 149 tests pass on the current commit does not invalidate the concern — BCH's behavior on a specific 5-error pattern is deterministic, but it's brittle: a future fixture change (different `chunk_set_id`, different bytecode size, different padding) could shift outcome 1 to outcome 2 with a header-decode rejection variant the accept set doesn't cover, silently breaking the test.

### I-3. `pipeline::decode` rustdoc still cites `ChunkedHeaderMalformed` for the mixed-header rejection

**Where:** `crates/mk-codec/src/string_layer/pipeline.rs:114-115`.

```rust
/// Mixing `SingleString` and `Chunked` headers across a multi-string
/// input is rejected with [`Error::ChunkedHeaderMalformed`].
```

After Task 1.2's plumbing migration, the relevant call-site at `pipeline.rs:137` returns `Error::MixedHeaderTypes` (and the matching reverse-direction site at `chunk.rs:176` likewise). The rustdoc claim is now stale — external consumers reading the public-API docs would expect `ChunkedHeaderMalformed` and pattern-match against the wrong variant.

**Recommended fix:** update the rustdoc to:

```rust
/// Mixing `SingleString` and `Chunked` headers across a multi-string
/// input is rejected with [`Error::MixedHeaderTypes`]. (An empty input
/// list is rejected with [`Error::ChunkedHeaderMalformed`] — that's the
/// "no input at all" case, distinct from "header types disagree.")
```

Cheap, no code change, but matters for the public-API doc surface. The CHANGELOG copy in `MILESTONE_v0_1_1.md:354` correctly cites `MixedHeaderTypes`, which makes the source-doc staleness slightly more glaring.

---

## Minor

### m-1. Task 1.1 test name slightly understates the property

**Where:** `crates/mk-codec/src/string_layer/pipeline.rs:269` — `decode_rejects_5_symbol_burst_in_last_chunk_data_part`.

The test name is accurate but flat — the BCH-distance argument (the property is "5 substitutions exceed t=4") is the actual invariant the test enforces. A name like `decode_rejects_5_symbol_burst_exceeding_bch_t4_capacity` would be more self-documenting; the current name is fine but not as illuminating about *why* 5 is the magic number. Optional rename, not blocking.

### m-2. The test's BCH-distance discipline argument in the comment is correct, but understates which code variant applies to each chunk

**Where:** `crates/mk-codec/src/string_layer/pipeline.rs:276-277`.

```
// BCH(108,93,8) and BCH(93,80,8) both cover up to 4 substitutions
// exactly (`t = 4`); a 5-symbol burst always exceeds the
// correction radius.
```

This is correct. The typical 84-byte card produces 88 stream bytes split into chunks of 53 + 35 bytes. The first chunk's data part is `8-symbol header + ceil(53*8/5) = 8 + 85 = 93 symbols`, which gets the long code (BCH(108,93,8) — total data-part length = 93+15=108). The second (last) chunk's data part is `8 + ceil(35*8/5) = 8 + 56 = 64 symbols`, which gets the regular code (BCH(93,80,8) — total = 64+13=77). So **the test perturbs the last (regular-code) chunk specifically**, and 5 errors > regular-code t=4. The comment doesn't articulate which chunk's BCH code is in play; a one-line clarification would tie the BCH-distance argument tighter to the test's specific shape.

### m-3. `synthetic_singlestring` helper byte-alignment proof is implicit; consider a debug-assert

**Where:** `crates/mk-codec/src/string_layer/pipeline.rs:343-350`.

The helper relies on `bytes_to_5bit` always padding with zero, so `five_bit_to_bytes` round-trips byte-aligned. For 8-byte inputs (= 64 bits), this produces 13 5-bit symbols with 1 zero pad bit. The 2-symbol header + 13-symbol payload = 15-symbol data-part, which lands in regular-code territory (14..=93). All correct.

But if a future test passes a bytecode whose `bytes_to_5bit` produces a non-byte-aligned tail (e.g., a 2-byte input → 4 5-bit symbols with 4 pad bits — wait, 4 zero pad bits is fine since `five_bit_to_bytes` requires the residual bits to be zero, and `bytes_to_5bit` always zero-pads). In fact, the property always holds because the `bytes_to_5bit`/`five_bit_to_bytes` pair is round-trip safe by construction (`bch.rs::79-103`). So the helper is correct under any byte input. A `debug_assert!` after the encode call to verify the round-trip would be defensive belt-and-braces but not necessary.

### m-4. FOLLOWUPS closure prose: minor formatting inconsistency

**Where:** `design/FOLLOWUPS.md:149-150`.

The `**Status:**` line in the closure entry includes both a `wont-fix — <reason>` clause and a closing-commit citation in the same line:

```
- **Status:** `wont-fix — moot per SPEC §2.4 (...).` Closed during v0.1.1 Phase 1 Task 1.3 (...). The smallest valid v0.1 bytecode is 80 bytes ...
```

The format rubric at FOLLOWUPS.md:16 specifies `wont-fix — <one-line reason>`. The current entry stuffs three sentences after the status code, which is informative but breaks the one-line discipline. Easy fix: move the elaboration to a new `**Closure note:**` line (or fold into `**What:**`), and keep `**Status:**` to the rubric form. Not blocking; the entry is readable as-is and the additional detail is useful.

### m-5. `decode_rejects_chunked_then_singlestring` chunk-count check trace is sound but worth calling out explicitly

**Where:** `crates/mk-codec/src/string_layer/pipeline.rs:367-383`.

The trace is: chunks[0] is Chunked with `total_chunks=2` (from encoder); chunks[1] is replaced with synthetic SingleString. In `reassemble_from_chunks`, `chunks.len()=2 == total_usize=2` so the count-check at `chunk.rs:131` passes. Then the loop reaches `chunks[1]`, hits the SingleString match arm, and returns `MixedHeaderTypes`. ✓

The test comment at lines 376-379 documents this correctly. One subtle robustness concern: the test relies on the typical 84-byte fixture producing exactly 2 chunks. If a future fixture change produced 3+ chunks, replacing only `strings[1]` would leave `strings[2..]` as Chunked, and the code path would be: chunks[0] Chunked → slot[0]; chunks[1] SingleString → MixedHeaderTypes. **Still correct** — the SingleString arm fires before any later Chunked arm. So the test is structurally robust to fixture-size drift.

### m-6. Reverse-direction synthetic-singlestring fixture data is arbitrary; deterministic input would be slightly cleaner

**Where:** `crates/mk-codec/src/string_layer/pipeline.rs:359, 380`.

The forward-direction test uses `&[0x42u8; 8]`; the reverse-direction test uses `&[0xAAu8; 8]`. Different bytes for no test-discrimination reason — both are arbitrary bytecode that's never decoded. Either pick one (e.g., a shared `SYNTHETIC_BYTECODE: &[u8; 8] = &[0x00; 8]` const) or document why they differ. Cosmetic, not blocking.

### m-7. The test count delta in the commit message matches the milestone plan; verified

**Where:** commit message + `MILESTONE_v0_1_1.md:399-401`.

Commit message: "Test count: 147 → 149 unit tests (+2 from Task 1.2's forward+reverse tests; Task 1.1 replaces the existing test count-neutral)." Milestone test-count table predicts 149 after Phase 1. `cargo test -p mk-codec` confirms 149 unit tests. ✓

---

## Observations

### O-1. Task 1.2 plumbing migration matches the milestone plan exactly

The milestone plan §1.2.3 enumerated three call-sites:
- `pipeline.rs:137` (forward direction) → migrate to `MixedHeaderTypes`. **Done.** ✓
- `chunk.rs:171` (reverse direction) → migrate to `MixedHeaderTypes`. **Done** (now at line 176). ✓
- `chunk.rs:124` (defense-in-depth, first-chunk-SingleString) → keep as `ChunkedHeaderMalformed`. **Done** (preserved at line 124). ✓

The `error::tests::static_variants_render` test (at error.rs:240-273) was updated to include the new variant per plan step 1.2.5. The CHANGELOG-bound observation that `Error` is `#[non_exhaustive]` makes this addition exhaustive-match-safe for external consumers. ✓

### O-2. `MixedHeaderTypes` rustdoc is well-shaped and accurate

The variant rustdoc at error.rs:82-91 distinguishes itself from `ChunkedHeaderMalformed` clearly (the former is "headers disagree across the list"; the latter covers within-set malformations). Both call-sites match the rustdoc claim verbatim. The Display string `"mixed string-layer header types in input list"` is a precise, diagnostic message. ✓

### O-3. Plan vs implementation drift — the milestone scope did anticipate the I-1/I-2 issue and noted it

Re-reading the milestone review draft I-2 and m-4, the original plan-review correctly flagged the BCH-distance argument as needing rigor improvements, and the resolution there (5+ symbol perturbation, accept set = {`CrossChunkHashMismatch`, `BchUncorrectable`}) is what the implementation followed. The plan was **silent on** which 5 symbols to perturb (header vs fragment vs hash region); the implementation chose the front of the data part (which falls in the header), but the in-code comment writes as if the choice was the bytecode-fragment region. The disconnect is a Task-1.1 implementation issue, not a plan defect.

### O-4. Task 1.3 wont-fix sequencing requirement is clear and discoverable

The FOLLOWUPS entry now reads: "if a future format extension lands a smaller bytecode (e.g., the Compact-65 mode discussed in SPEC §3.6, ...), this item MUST be re-opened **before the format extension ships**. ... Any future smaller-bytecode design pass (or a Compact-65-shaped FOLLOWUPS entry) MUST cite this requirement and re-open the issue."

This is the right shape. A future Compact-65 designer who greps FOLLOWUPS for `Compact-65`, `smaller bytecode`, or even just `singlestring` would find this entry. The MUST-cite requirement gives reviewers a clear gate: if a future smaller-bytecode design pass doesn't re-open this, the review can flag it as a missed prerequisite. ✓

### O-5. The plan didn't list, but Phase 1 surfaces, no test-discipline gaps

Phase 1's test inventory (per the milestone plan):
- Task 1.1: replace one test (count-neutral). ✓
- Task 1.2: add two tests (forward + reverse direction). ✓
- Task 1.2.5: update `error::tests::static_variants_render`. ✓

What the plan did NOT list, and Phase 1 (correctly) did NOT add: a test for the `chunk.rs:124` defense-in-depth path. That's correct because the path is documented as unreachable from `pipeline::decode` (which intercepts the all-SingleString case at `pipeline.rs:135`); a direct unit test would have to call `reassemble_from_chunks` directly with a SingleString-headed chunks vec, which is testable but not part of the Phase 1 scope. There is no FOLLOWUPS entry for it, which is correct — defense-in-depth code paths don't need user-facing test coverage to maintain their invariant.

### O-6. The plan's Phase 1 scope did not specify which 5 symbols to perturb (header vs fragment); see I-1/I-2

This is the only **plan defect** I found. The milestone review had an opportunity to be more prescriptive about the perturbation region; it specified "5+ symbols in the cross-chunk-hash region" (per `MILESTONE_v0_1_1.md:79-86`), which is a precise instruction. The Phase 1 implementation perturbed the header region instead, with a comment that incorrectly claims it perturbed the fragment region. So the chain is:

1. Plan (I-2 of plan review) says "5+ symbols in the cross-chunk-hash region."
2. Implementation puts 5 symbols in the chunked-header region.
3. Implementation comment says it's in the cross-chunk-hash region.

Steps 2 and 3 disagree with each other and with step 1. Fix: align all three (recommend keeping plan + comment intact, fix code to perturb post-header). See I-1/I-2 above.

---

## Recommended action

**Proceed to Phase 2 after a Phase 1 fixup commit** addressing I-1, I-2, and I-3. The fixup is small (rustdoc edit + comment-or-code adjustment in the new perturbation test), should be 5–10 minutes of work, and lets the audit trail cleanly attribute all important findings to a Phase 1 fixup commit named per the workflow convention (`style/fix(mk-codec phase 1.1): apply Phase 1 review fixes (commit 8685608 review)`).

Minor items m-1..m-7 should be folded into the same fixup commit where straightforward (m-1 rename, m-2 comment expansion, m-4 FOLLOWUPS reformat); m-3, m-5, m-6, m-7 are observational and don't need code changes.

The commit-with-fixups pattern is consistent with the per-phase opus-review discipline established on v0.1.0 (Phases 1–7), and Phase 1's report saves cleanly to `design/agent-reports/v0-1-1-phase-1-review-8685608.md` per the established naming convention.

(End of review.)
