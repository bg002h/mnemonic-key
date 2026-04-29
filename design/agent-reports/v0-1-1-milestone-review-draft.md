# Review — `design/MILESTONE_v0_1_1.md` (uncommitted draft)

- **Reviewer:** Opus code-reviewer subagent
- **Date:** 2026-04-29
- **Artifact:** planning markdown (`design/MILESTONE_v0_1_1.md`); not a code change
- **Cross-checks:**
  - `design/FOLLOWUPS.md`
  - `design/SPEC_mk_v0_1.md` §§2.4–2.5, §3.5, §4
  - `crates/mk-codec/src/error.rs` (variant set, rustdoc)
  - `crates/mk-codec/src/string_layer/pipeline.rs::decode` (pipeline rejection sites)
  - `crates/mk-codec/src/string_layer/chunk.rs` (reassembly errors)
  - `crates/mk-codec/src/string_layer/header.rs::from_5bit_symbols`
  - `crates/mk-codec/src/bytecode/path.rs` (testnet table 0x11–0x17)
  - `crates/mk-codec/src/bin/gen_mk_vectors.rs` (current `FixtureSpec`/schema=1 emit)
  - `crates/mk-codec/tests/vectors.rs` (`every_vector_round_trips`, `schema_metadata_pinned`)
- **Test baseline (verified):** `cargo test -p mk-codec` → 147 unit + 3 round_trip + 3 vectors = 153, matches the plan's "v0.1.0 baseline" row.

## Top-line

The plan is **structurally sound and largely commit-ready**. The cut-line between "patch polish + corpus" and "review-grade audit" is defensible; the phase ordering is correct; the schema-bump strategy is the right additive choice. There are no critical errors. There is **one important gap** (an `Error` variant the negative-vector table omits), one **important phase-ordering nit** (the `MixedHeaderTypes` plumbing change in Phase 1 silently shifts behaviour for a long-standing assertion in `tests/pipeline.rs` and *also* forces an update to `decode_rejects_empty_input` if Phase 1 chooses to repurpose `ChunkedHeaderMalformed("empty input")` semantically — see below), a couple of **important** rustdoc and exhaustiveness-test caveats, and a handful of **minor** wording / reference fixes. With those addressed, the plan is good to commit and execute.

---

## Critical

_None._

---

## Important

### I-1. The negative-vector table is missing one `Error` variant — the empty-input path

**Where:** `MILESTONE_v0_1_1.md:259-282` (Task 3.2 "Mandatory coverage" table) vs `crates/mk-codec/src/string_layer/pipeline.rs:117-121` and `tests/pipeline.rs::decode_rejects_empty_input` (`pipeline.rs:328-330`).

`pipeline::decode` rejects an empty input list with `Err(Error::ChunkedHeaderMalformed("empty input string list"))`. After Phase 1 lands `MixedHeaderTypes`, this site **stays** `ChunkedHeaderMalformed` (the empty-list case is not a header-type disagreement — it's just "no input"), but the negative-vector table only assigns N9 to the chunk-index-OOB form of `ChunkedHeaderMalformed`. The plan should either:

- (a) add an N9b/N23 row covering the empty-list trigger so the harness exercises every distinct call-site that emits `ChunkedHeaderMalformed`, or
- (b) explicitly note that `ChunkedHeaderMalformed` is a `String`-parameterized variant covering *multiple* distinct rejection conditions, and pick one canonical trigger per (variant × condition) pair if exhaustive coverage is the intent.

The exhaustiveness gate proposed in Step 3.3.2 (`every_error_variant_has_negative_vector`) only checks **variant** coverage, not **call-site** coverage, so the gap won't fire as a CI failure even though it's a real corpus gap. md-codec's "30 negative vectors, one per Error variant" precedent (cited in FOLLOWUPS' `decoder-error-variant-parity` entry) is variant-level, so (a) is consistent with the contract — recommend adding one row to the table, e.g.:

```
| N23 | `ChunkedHeaderMalformed` (empty input) | empty `&[]` passed to decode |
```

### I-2. Phase 1's `MixedHeaderTypes` plumbing changes the variant returned by *both* mixed-header paths, but only one site is enumerated — verify the second site exists

**Where:** `MILESTONE_v0_1_1.md:102` ("replace the two `ChunkedHeaderMalformed("...")` sites") vs the actual `pipeline.rs::decode` body (`pipeline.rs:135-139`).

In the current pipeline, the only call-site for "header type disagreement" is `pipeline.rs:135-139` (`first_is_single` → `parsed.len() != 1`). The reverse case `[Chunked, SingleString, …]` is **not** an explicit early reject in `pipeline.rs`; it falls through into `reassemble_from_chunks`, which then trips a different `ChunkedHeaderMalformed` (the "missing chunk_index" / "duplicate chunk_index" / "total_chunks mismatch" branches at `chunk.rs:153-188`) because the embedded SingleString chunk's `from_5bit_symbols` produced a `SingleString` header that never enters the chunk pipeline. (Or, more precisely: the SingleString string's parsed header in the `parsed: Vec<(StringLayerHeader, Vec<u8>)>` won't be matched by the `into_iter().map(...)` in the Chunked branch, but reading the code at `pipeline.rs:146-149` shows the `.map` will still pack a `ChunkFragment { header: SingleString, .. }` into the chunks vec, and `reassemble_from_chunks::is_chunked()` checks at `chunk.rs:111-118` will surface a `ChunkedHeaderMalformed("first chunk header is not Chunked")` or similar.)

The plan says **two** sites need replacing; in the actual code the second site is buried in `reassemble_from_chunks` and isn't actually labelled "mixed header types" — replacing it would require changing chunk.rs's reassembly check to detect the `SingleString-after-Chunked` case before its existing chunked-shape checks fire. The plan's Phase 1 task list does not capture this complexity. Recommend:

- Re-confirm by grep'ing `ChunkedHeaderMalformed` call-sites in `pipeline.rs` + `chunk.rs` and listing each one. Phase 1.2.3 should explicitly enumerate which call-sites get migrated and which keep the existing variant.
- Add a test under Step 1.2.1 that supplies `[Chunked, SingleString]` (reverse direction) and either asserts `MixedHeaderTypes` (if the plan extends the migration into chunk.rs) or asserts the existing `ChunkedHeaderMalformed` (if the plan scopes the migration to the pipeline.rs site only). Currently the plan is silent on which behaviour holds for the reverse case — the CHANGELOG's claim that `MixedHeaderTypes` covers "header-type disagreement across multi-string inputs" should match the implementation.

### I-3. The exhaustiveness gate at Step 3.3.2 conflicts with `Error` being `#[non_exhaustive]`

**Where:** `MILESTONE_v0_1_1.md:295` ("uses `strum::EnumIter` (or a manual exhaustive list, since `Error` is `#[non_exhaustive]`)").

The plan acknowledges the tension but doesn't resolve it. `#[non_exhaustive]` blocks **external** exhaustive matching; `strum::EnumIter` is a derive that runs **inside** the crate, so it can still iterate every variant the crate itself defines — this is fine. But the plan also says "or a manual exhaustive list," which is the brittle path: every time a new variant is added, two files (the enum + the test) must be updated together, with no compiler-enforced linkage.

Recommendation: pick one. If `strum` is acceptable as a dev-dependency (`[dev-dependencies] strum = { version = "0.27", features = ["derive"] }`), use `EnumIter` and have the test iterate the variants via the derive; the test then catches any new variant whose negative vector is missing. md-codec uses this pattern. If `strum` is not desired, the test should pattern-match exhaustively on `Error` *inside* the crate (which the `#[non_exhaustive]` attribute permits) so the compiler enforces "all variants enumerated" via match-arm-coverage warnings — that's just as strong as `EnumIter` and adds no dependency. Either choice is good; the manual-list option should be removed from the plan.

### I-4. The CHANGELOG note about `Error::MixedHeaderTypes` understates the wire-equivalent observability concern

**Where:** `MILESTONE_v0_1_1.md:354` ("precise discriminator; previously surfaced as `ChunkedHeaderMalformed`").

This is correct as stated, but the *mitigation* in the Risks section (`MILESTONE_v0_1_1.md:411`) only covers the message-text-fragility concern. The deeper concern: **`Error::Display` rendering changes**, which means **negative vectors that pin the `expected_error` rendered string** (Phase 3) need to be regenerated against the new variant text the moment Phase 1 lands. That's an internal sequencing point — the negative vectors won't be authored until Phase 3 — so it's not a real bug, but it should be called out in the risks section that **the Phase 3 corpus must be regenerated against post-Phase-1 `Error::Display` strings**, never against pre-Phase-1 strings. Otherwise N10 (the `MixedHeaderTypes` vector) would carry the wrong `expected_error` field and the SHA pin would be stale.

Recommend adding a one-line risk note to that effect, plus: third-party validators that pinned `expected_error` against an *internal* (non-published) version of the v0.1.0 corpus will see drift. (No third party has done this for v0.1.0 since the v0.1.0 corpus has no `expected_error` field at all — but worth noting in the migration prose for v0.1.1 → future releases.)

---

## Minor

### m-1. Test count math: Phase 1 might add **2** tests, not 1

**Where:** `MILESTONE_v0_1_1.md:399-401` (test count table); cross-ref Step 1.2.1.

Step 1.2.1 prescribes "add a test in `pipeline::tests` that supplies `[SingleString_string, Chunked_string]` and asserts `Err(MixedHeaderTypes)`." If I-2's recommendation lands and the reverse case `[Chunked, SingleString]` also gets a test, Phase 1 adds 2 tests, not 1, and the After-Phase-1 unit count is 149, not 148. Also, Step 1.1 *replaces* the existing `decode_rejects_perturbed_cross_chunk_hash` test rather than adding a new one (1.1.2 says "Delete the brittle byte-flip variant"), which is +0. So depending on I-2's resolution, the table row should be 148 or 149.

### m-2. Phase 2 corpus-size determinism note is good but missing one detail

**Where:** `MILESTONE_v0_1_1.md:190` (Step 2.2.2 "Verify byte-determinism").

Re-running the generator **and confirming SHA-256 stays stable** is the right discipline, but the v0.1.0 generator at `gen_mk_vectors.rs:281` defaults `output_path` to `crates/mk-codec/tests/vectors/v0.1.json`. If the user is in a worktree or runs with `--output` against a tmp path, the SHA pin in `tests/vectors.rs` won't be updated automatically. Recommend the verify step include an explicit `diff` between the regenerated file and the on-disk file before updating the SHA pin (or a `cmp -s`), so a stale generator output doesn't trick the workflow.

### m-3. `wont-fix` rationale for `encode-with-chunk-set-id-singlestring-silent-ignore` is correct but should explicitly cite the latent-bug condition

**Where:** `MILESTONE_v0_1_1.md:108-112` (Task 1.3); cross-ref `pipeline.rs:65-70` (the function rustdoc) and `pipeline.rs:73-82` (the SingleString branch which never consults `chunk_set_id`).

Per SPEC §2.4 (`SPEC_mk_v0_1.md:85`), every conforming v0.1 KeyCard chunks (smallest bytecode = 80 bytes > 56). So the `SingleString` branch in `encode_with_chunk_set_id` is **dead code under the v0.1 wire format**. The closure rationale is correct.

However: the Task 1.3 closure prose says "if a future format extension lands a smaller bytecode (e.g., Compact-65 per §3.6), this item should be re-opened." That's a subtle but real latent-bug surface. Recommend strengthening to:

> _"...if a future format extension lands a smaller bytecode (e.g., Compact-65 per §3.6, which would drop xpub.version + xpub.parent_fingerprint and bring some bytecodes below 56 bytes), this item MUST be re-opened **before the format extension ships** — the silent-drop semantics is friendly today but masks an Encode-side determinism bug under any wire-format that makes SingleString reachable. A pending FOLLOWUPS-or-equivalent gate should be checked at any future smaller-bytecode design pass."_

The status in FOLLOWUPS should also list a sequencing requirement: "re-open before any future smaller-bytecode wire-format extension."

### m-4. Phase 1.1 BCH-distance argument is sound but the test can be more rigorous than the plan suggests

**Where:** `MILESTONE_v0_1_1.md:78` ("perturb at 5-bit position P with magnitude M such that (M, M, M, M, M) at neighbouring positions ... cannot factor as a degree-≤4 BCH error locator polynomial").

The plan's BCH-distance reasoning is structurally correct. BCH `t = 4` covers any error pattern whose syndromes admit a degree-≤4 error-locator polynomial. A 5-symbol burst is **provably outside the correction radius** iff the syndromes' polynomial fit is rank-≥5 (i.e., no degree-≤4 locator polynomial satisfies all syndromes). For *most* random 5-symbol bursts this holds; for *some* (specifically, those that happen to coincide with a 4-error pattern in the dual code), the BCH decoder will correct *to a different valid codeword* — different from both the original and the perturbed input. The plan's "pin a specific 4-symbol perturbation" prose is slightly off: the threshold is **5 substitutions**, not 4, since `t = 4` covers up to 4 substitutions exactly. A 5-symbol perturbation always *exceeds* the correction radius (decoder either corrects to the wrong codeword or returns `BchUncorrectable`); a 4-symbol perturbation is *within* the radius and gets silently un-flipped. The plan should clarify:

- The perturbation magnitude is `5+ symbol substitutions`, not `>4 BCH-distance`.
- The verification discipline: after re-encoding, run the decoder and confirm one of {`Err(CrossChunkHashMismatch)`, `Err(BchUncorrectable)`}; either is acceptable. The current test (`pipeline.rs:302-305`) already accepts only `CrossChunkHashMismatch`, so the new test should accept either to avoid being equally brittle in the other direction.

This is rigour-not-correctness; the plan as written would land a working test, just one with the wrong stopping condition under some inputs.

### m-5. Schema-2 migration detail: `expected_error: null` vs absent field

**Where:** `MILESTONE_v0_1_1.md:225-226` ("Clean vectors keep `expected_error: null` (or omit the field — schema-2 readers handle both)").

The `gen_mk_vectors.rs` emit code (current) sorts keys via `serde_json::Map`'s `BTreeMap` backing. Adding an optional `expected_error` field that's *sometimes present, sometimes absent* breaks the byte-determinism property the corpus depends on (see `gen_mk_vectors.rs:18-22`). Concretely: a clean vector with the field omitted vs `null` produces different JSON bytes, hence a different SHA-256 pin. Recommend the plan **mandate** one shape — either always emit `expected_error: null` for clean vectors (consistent shape, harness checks `is_null()`), or always emit `kind: "clean"` / `kind: "negative"` discriminator with `expected_error` only present in negative variants. The "additive optional field" framing is fine for Phase 3's harness logic but the **emit** side needs a deterministic rule.

The draft picks the additive-field approach (correct call IMO — a `kind` discriminator is over-engineered for a 2-state distinction), but the plan should explicitly say "**always emit `expected_error: null` for clean vectors**" so byte-determinism is preserved.

### m-6. Path-dictionary table mapping verified ✓ — plan matches `bytecode/path.rs::29-46`

**Where:** `MILESTONE_v0_1_1.md:160-169` (Phase 2 missing-dictionary-entries table).

I cross-checked all 9 entries against the source-of-truth in `crates/mk-codec/src/bytecode/path.rs` (the `STD_PATHS` static at path.rs:31-46). All testnet path strings match: 0x11→`m/44'/1'/0'`, 0x12→`m/49'/1'/0'`, 0x13→`m/84'/1'/0'`, 0x14→`m/86'/1'/0'`, 0x17→`m/87'/1'/0'`. Mainnet entries 0x01/0x02/0x04/0x06 also match the SPEC §3.5 table at SPEC_mk_v0_1.md:203-208. No bugs.

(Note: the plan correctly skips 0x16 per the cross-repo `md-path-dictionary-0x16-gap` deferral; the SPEC §3.5 footnote at SPEC_mk_v0_1.md:214 confirms encoders MUST NOT emit 0x16 in v0.1.)

### m-7. Branch-from-tag instruction at Task 0.1 is correct but the workflow note should mention the working tree

**Where:** `MILESTONE_v0_1_1.md:60` (`git checkout -b feature/v0.1.1-implementation mk-codec-v0.1.0`).

Tag exists per task description (commit `f4de16c`). The current branch is `feature/v0.1.0-implementation`, which has post-tag commits (`21efbea` = `docs: add CLAUDE.md for next-session context loading`). Branching from the tag means **dropping** that CLAUDE.md from the v0.1.1 branch's history. If the team wants CLAUDE.md to persist on the v0.1.1 branch (which it should, since CLAUDE.md is repo-meta not code), the branch should be cut from the post-tag commit on the v0.1.0 branch (or the CLAUDE.md should be cherry-picked into the v0.1.1 branch as Task 0.0). Recommend the plan resolve this — either branch from `feature/v0.1.0-implementation` (after the docs commit) and tag retroactively, or branch from the tag and cherry-pick.

---

## Observations

### O-1. The `decoder-error-variant-parity` exhaustiveness invariant should outlive v0.1.1

**Observation:** Phase 3.3.2's `every_error_variant_has_negative_vector` test is a CI gate per the plan. Recommend the plan call out that this gate persists into v0.2+; any new `Error` variant in a future release lands together with its negative vector or breaks CI. The plan implies this but doesn't say it explicitly. (No action needed; just noting.)

### O-2. The "out of scope" rationale for the analytical audit items is well-argued

**Observation:** The plan's argument for splitting "review-grade analytical artifacts" (`nums-structural-audit`, `hrp-mk-collision-check`, `bip-cross-reference-completeness`) into a separate `MILESTONE_v0_1_pre_bip_audit.md` milestone is sound. Each of those items wants a single domain-expert review pass over the whole package (Andrew Poelstra for NUMS structure per FOLLOWUPS:69, SLIP-0173 maintainers for HRP collision per FOLLOWUPS:75, BIP editors for cross-references per FOLLOWUPS:84), not piecemeal landing as patch-release sub-tasks. Splitting them out is the right call.

### O-3. Phase 4 final-reconciliation step is good; consider adding a "smoke test against a real Bitcoin xpub"

**Observation:** Step 4.5 lists agent-report consolidation + memory update. v0.1.0's release plumbing did not include an integration test against a real, externally-anchored xpub (e.g., a published BIP test vector). Not a blocker for v0.1.1, but worth tracking as a v0.2 or pre-bip-submission item: "round-trip a known BIP 32 / BIP 84 published xpub through the encoder and verify the decoded `KeyCard` matches the published derivation." The current corpus uses synthetic xpubs (`gen_mk_vectors.rs::synthetic_xpub` at line 179) — fine for wire-conformance but skips the cross-implementation interop check.

### O-4. The plan's "patch-version semver discipline" promise interacts well with the `expected_error` schema-bump

**Observation:** Bumping the *vector-corpus schema* from 1 → 2 while the *wire format* stays at v0.1 is a clean separation. The vector corpus and the Rust crate version aren't lockstep-coupled — third parties can validate against schema-1 corpora produced by older generators or schema-2 corpora produced by v0.1.1 with the same wire format. The CHANGELOG's "schema-1 corpora remain readable" claim (`MILESTONE_v0_1_1.md:357`) is a forward-compatibility promise the harness in Step 3.3.1 can structurally honour (`null` or absent `expected_error` → clean-vector path). Good design.

### O-5. The decision to deliver `decoder-error-variant-parity` (a pre-BIP-submission item) in v0.1.1 is well-motivated

**Observation:** The plan moves a `pre-bip-submission` item into the patch release, on grounds that "negative vectors are a natural extension of the corpus and don't require external review." This is the right call — variant parity is a code-and-test discipline, not an analytical artifact. Doing it now (rather than at the pre-BIP gate) means the corpus is exercised at every CI run from v0.1.1 onward, surfacing any decoder rejection-path drift early. The plan's selective movement of corpus-shaped pre-BIP items into v0.1.1 while leaving review-shaped pre-BIP items out is principled.

---

## Recommended action

**Commit-with-fixups.** The plan is structurally sound and I would not block it from being committed. The important findings (I-1 through I-4) are all wording / table-row / risk-section additions that take 5–10 minutes each to fold in; none require re-thinking the milestone shape. The minor findings (m-1 through m-7) are mostly precision improvements and should be folded in too, but none would block execution. Suggested order:

1. Address I-2 first (verify the `MixedHeaderTypes` plumbing scope by grep-listing the actual call-sites; either expand Phase 1 to cover both directions or scope the doc to one direction).
2. Then I-1 (add the empty-input row to the negative-vector table).
3. Then I-3 (resolve `strum` vs manual-exhaustive for the gate).
4. Then m-1 through m-7 in any order.
5. Then I-4 (add the risk note about Phase 3 vector regeneration depending on Phase 1 `Error::Display` strings).

After fold-in, the plan is ready to commit and execute under the established per-phase-opus-review workflow.

(End of review.)
