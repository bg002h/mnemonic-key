# v0.2.0 Phase 1 review — strum-driven exhaustiveness gate

- **Commit:** `901596a` — `feat(test): strum-driven exhaustiveness gate for negative vectors`
- **Branch:** `feature/v0.2.0`
- **Reviewer:** code-reviewer subagent (model: opus)
- **Date:** 2026-04-29
- **Predecessor reports:** `v0-1-1-phase-3-review-1e42354.md` (origin of strum follow-up)
- **Files in scope:**
  - `crates/mk-codec/Cargo.toml` (+strum 0.26 dev-dep)
  - `crates/mk-codec/tests/error_coverage.rs` (NEW, 216 lines)
  - `crates/mk-codec/tests/vectors.rs` (-65 / +6, removed runtime substring gate)
  - `Cargo.lock` (strum dependency tree pinned)

## TL;DR

**Proceed to Phase 2.** Phase 1 is a clean, faithful adaptation of md-codec's pattern with one small UX improvement (the "exempt-but-leaked" anti-contradiction guard in the forward gate). All 22 source `Error` variants are mirrored case-for-case; every parameterized-variant prefix is a literal prefix of the actual `#[error("...")]` rendering; the `CardPayloadTooLarge` exemption is structurally justified (encoder-only via `split_into_chunks`, decoder uses `reassemble_from_chunks`). Test count, clippy, and fmt all clean. The only follow-up action is administrative: flip the `error-variant-exhaustiveness-gate-strum` FOLLOWUPS entry from `open` to `resolved 901596a` (Suggestion S-1 below).

## 1. `ErrorVariantName` ↔ `Error` parity

Verified every variant in `crates/mk-codec/src/error.rs::Error` (22 total) appears in the mirror enum at `tests/error_coverage.rs::ErrorVariantName`. Both sides match case-for-case in name; iteration order in the mirror also matches the declaration order in the source (a nice readability bonus, though not required for correctness).

| # | Source variant | Mirror variant |
|---|---|---|
| 1 | `InvalidHrp` | `InvalidHrp` |
| 2 | `MixedCase` | `MixedCase` |
| 3 | `InvalidStringLength` | `InvalidStringLength` |
| 4 | `InvalidChar` | `InvalidChar` |
| 5 | `BchUncorrectable` | `BchUncorrectable` |
| 6 | `UnsupportedCardType` | `UnsupportedCardType` |
| 7 | `MalformedPayloadPadding` | `MalformedPayloadPadding` |
| 8 | `ChunkSetIdMismatch` | `ChunkSetIdMismatch` |
| 9 | `ChunkedHeaderMalformed` | `ChunkedHeaderMalformed` |
| 10 | `MixedHeaderTypes` | `MixedHeaderTypes` |
| 11 | `CrossChunkHashMismatch` | `CrossChunkHashMismatch` |
| 12 | `UnsupportedVersion` | `UnsupportedVersion` |
| 13 | `ReservedBitsSet` | `ReservedBitsSet` |
| 14 | `InvalidPolicyIdStubCount` | `InvalidPolicyIdStubCount` |
| 15 | `InvalidPathIndicator` | `InvalidPathIndicator` |
| 16 | `PathTooDeep` | `PathTooDeep` |
| 17 | `InvalidPathComponent` | `InvalidPathComponent` |
| 18 | `InvalidXpubVersion` | `InvalidXpubVersion` |
| 19 | `InvalidXpubPublicKey` | `InvalidXpubPublicKey` |
| 20 | `UnexpectedEnd` | `UnexpectedEnd` |
| 21 | `TrailingBytes` | `TrailingBytes` |
| 22 | `CardPayloadTooLarge` | `CardPayloadTooLarge` |

**No drift.** Pass.

## 2. `display_prefix()` correctness

Cross-checked every `match` arm in `display_prefix()` (lines 84-107) against the corresponding `#[error("...")]` attribute in `error.rs`. For the parameterized variants the user specifically called out, the prefix is a literal prefix of the actual `Display` rendering — verified up to the colon or first variable substitution:

| Variant | `#[error]` template | `display_prefix()` | Prefix valid |
|---|---|---|---|
| `InvalidHrp` | `"invalid HRP: {0}"` | `"invalid HRP"` | yes (truncates before `:`) |
| `InvalidStringLength` | `"invalid data-part length: {0}"` | `"invalid data-part length"` | yes |
| `InvalidChar` | `"invalid character {ch} at position {position}"` | `"invalid character"` | yes (truncates before space + var) |
| `BchUncorrectable` | `"BCH uncorrectable: {0}"` | `"BCH uncorrectable"` | yes |
| `UnsupportedCardType` | `"unsupported card type: 0x{0:02x}"` | `"unsupported card type"` | yes |
| `ChunkedHeaderMalformed` | `"chunked-header malformed: {0}"` | `"chunked-header malformed"` | yes |
| `UnsupportedVersion` | `"unsupported version: {0}"` | `"unsupported version"` | yes |
| `InvalidPathIndicator` | `"invalid path indicator byte: 0x{0:02x}"` | `"invalid path indicator byte"` | yes |
| `PathTooDeep` | `"path too deep: {0} components (max 10)"` | `"path too deep"` | yes |
| `InvalidPathComponent` | `"invalid path component: {0}"` | `"invalid path component"` | yes |
| `InvalidXpubVersion` | `"invalid xpub version: 0x{0:08x}"` | `"invalid xpub version"` | yes |
| `InvalidXpubPublicKey` | `"invalid xpub public key: {0}"` | `"invalid xpub public key"` | yes |
| `CardPayloadTooLarge` | `"card payload too large: bytecode_len = {bytecode_len} > max_supported = {max_supported}"` | `"card payload too large"` | yes |

Static (unparameterized) variants verified as exact-match prefixes (i.e., the prefix equals the full Display string):

| Variant | Static `Display` | `display_prefix()` | Equal |
|---|---|---|---|
| `MixedCase` | `"mixed case in input string"` | `"mixed case"` | proper prefix (yes) |
| `MalformedPayloadPadding` | `"malformed payload padding (5-bit symbols don't byte-align)"` | `"malformed payload padding"` | yes |
| `ChunkSetIdMismatch` | `"chunk_set_id mismatch across chunks"` | `"chunk_set_id mismatch"` | yes |
| `MixedHeaderTypes` | `"mixed string-layer header types in input list"` | `"mixed string-layer header types"` | yes |
| `CrossChunkHashMismatch` | `"cross-chunk integrity hash mismatch"` | `"cross-chunk integrity hash mismatch"` | yes (full equality) |
| `ReservedBitsSet` | `"reserved bits set in bytecode header"` | `"reserved bits set"` | yes |
| `InvalidPolicyIdStubCount` | `"policy_id_stub_count must be >= 1"` | `"policy_id_stub_count must be >= 1"` | yes (full equality) |
| `UnexpectedEnd` | `"unexpected end of bytecode"` | `"unexpected end of bytecode"` | yes (full equality) |
| `TrailingBytes` | `"trailing bytes after xpub"` | `"trailing bytes after xpub"` | yes (full equality) |

**All 22 prefixes are valid `starts_with` candidates against the actual `Display` rendering.** Pass.

One observation worth recording for future maintenance (no fix needed): six variants (`CrossChunkHashMismatch`, `InvalidPolicyIdStubCount`, `UnexpectedEnd`, `TrailingBytes`, `MalformedPayloadPadding` come close) have prefixes equal to the full Display string. If a future variant whose Display rendering is itself a prefix of another variant's Display were added, the strict `starts_with` discipline would silently mis-attribute. None of the current 22 variants share a non-empty common prefix, so this is theoretical only. Not actionable for v0.2.0.

## 3. `is_exempt()` rationale — `CardPayloadTooLarge` encoder-only claim

Searched for `CardPayloadTooLarge` emit sites:

```
src/error.rs:156                              ← variant declaration
src/string_layer/chunk.rs:60                  ← only `Err(...)` emit site
src/string_layer/chunk.rs:250                 ← unit-test assertion (in #[cfg(test)])
src/bin/gen_mk_vectors.rs:346                 ← comment reiterating exemption
```

The single emit site is `split_into_chunks` (chunk.rs:60). Tracing callers:

- `split_into_chunks` is called only from `encode_with_chunk_set_id` (pipeline.rs:97), `gen_mk_vectors.rs` (lines 395, 681), and the chunk.rs test module. None of these are `decode`-reachable.
- `decode` (pipeline.rs:118) calls `reassemble_from_chunks`, not `split_into_chunks`. The reassembly side has its own bounds (chunk-count and per-fragment length checks); pathological oversized inputs surface as `ChunkedHeaderMalformed` or `CrossChunkHashMismatch`, not `CardPayloadTooLarge`.

The exemption rationale in `is_exempt()` (line 116-122) is accurate: input chunked stream is bounded by `MAX_CHUNKS=32 × 53-byte fragments = 1696 bytes` minus the 4-byte hash, exactly matching the encoder's emit ceiling at chunk.rs:21 (`MAX_CHUNKABLE_BYTECODE = 32*53 - 4 = 1692`). Decoder cannot construct a stream large enough to trip the encoder-side ceiling.

**Encoder-only claim verified.** Pass.

Bonus observation on the exemption gate's design: the forward test (lines 144-160) actively asserts that an exempt variant's prefix is *not* present in any corpus vector (a "leaked" exemption is treated as a contradiction). This is a small but real improvement over md-codec's pattern, where exemption was — to my reading — a one-way bypass. The contradiction guard catches the case where someone adds an `is_exempt()` rationale and *also* a corpus vector for the same variant; one of the two must go. Nice.

## 4. `every_negative_vector_maps_to_a_known_variant` — walk-through

Picked three corpus negatives the user called out:

- **N1** (`expected_error = "invalid HRP: bt"`): matches `ErrorVariantName::InvalidHrp` prefix `"invalid HRP"`. Match.
- **N17** (`expected_error = "invalid path component: LEB128 overflow at shift 35"`): matches `ErrorVariantName::InvalidPathComponent` prefix `"invalid path component"`. Match. (N.B.: the v0.1.1 Phase 3 fixup commit `59878ca` reshaped this vector to actually trigger `InvalidPathComponent`; before that fixup it surfaced as `UnexpectedEnd`. The mirror-enum gate would have caught the original drift earlier in the cycle.)
- **N23** (`expected_error = "chunked-header malformed: empty input string list"`): matches `ErrorVariantName::ChunkedHeaderMalformed` prefix `"chunked-header malformed"`. Match.

Reverse-direction test (lines 188-216) is straightforward: collects `prefixes: Vec<&'static str>`, iterates corpus, skips clean vectors (where `expected_error == null`), and any negative whose `expected_error` doesn't `starts_with` any prefix is reported as an "orphan" with vector name + the offending string. The error message is helpful and points the maintainer at the two correct fix locations (vector or `ErrorVariantName`/`display_prefix`).

`cargo test -p mk-codec --test error_coverage` confirms both gates pass on the current corpus.

**Pass.**

## 5. `vectors.rs` cleanup correctness

The remaining tests in `tests/vectors.rs` are intact:

- `vector_file_sha256_matches_pin` (line 95)
- `schema_metadata_pinned` (line 108)
- `every_vector_round_trips` (line 124, with its `exercise_clean_vector` / `exercise_negative_vector` helpers)

The bottom-of-file pointer comment (lines 242-247) correctly redirects readers to `tests/error_coverage.rs`. No dangling imports — the deleted gate's only unique imports were the runtime variant-coverage helpers, all of which moved. `cargo build --tests -p mk-codec` succeeds with no `unused_imports` warnings (verified via `cargo test -p mk-codec` clean output above).

One nit (Suggestion S-2 below): the pointer comment says "as of v0.1.2" but the file SHA pin and schema version still reference v0.1.1's corpus (`schema = 2`, `family_token = "mk-codec 0.1"`). Under the v0.2.0 milestone this is fine — the corpus extends in Phase 2 — but the comment's tense ("as of v0.1.2") is slightly forward-looking. If you'd rather keep the comment passive, "lives in `tests/error_coverage.rs`" without the version stamp avoids the issue.

**Pass.**

## 6. Mirror-enum hazard model vs. md-codec's parallel

Read `descriptor-mnemonic/crates/md-codec/tests/error_coverage.rs` mentally from the doc rationale embedded in `tests/error_coverage.rs:31-33` and the `Why deferred` section of FOLLOWUPS lines 132-134. The hazard is identical: the mirror enum's variant names must match the source enum case-for-case, and the compiler doesn't enforce it because `#[non_exhaustive]` blocks integration-test exhaustive matching. The maintenance discipline is the same on both sides ("update the mirror, then the corpus or `is_exempt()` follows automatically").

mk-codec **reproduces** the md-codec hazard model and **does not improve** it (other than the small contradiction-guard improvement noted in §3). That's the right call: the v0.2.0 plan's goal was alignment with md-codec, not divergence. Future improvements (e.g., a build-script that codegens the mirror enum from a `#[non_exhaustive]` source via `syn` parsing, or moving the gate into `crate::error::tests` per FOLLOWUPS option 2) can apply uniformly to both repos when the shared `mc-codex32` extraction lands (closure Q-9 trigger).

For Phase 1: **no improvement needed; alignment is the right call.** Pass.

## 7. Test-count delta

`cargo test -p mk-codec` output:

```
test result: ok. 149 passed; 0 failed; 0 ignored        (lib unit)
test result: ok. 2 passed; 0 failed; 0 ignored          (error_coverage)
test result: ok. 3 passed; 0 failed; 0 ignored          (round_trip)
test result: ok. 3 passed; 0 failed; 0 ignored          (vectors)
                                                        ─────
total:                                          157 tests
```

Pre-Phase-1 was 156 (149 unit + 0 error_coverage + 3 round_trip + 4 vectors — the deleted gate was the 4th vectors test). Post-Phase-1 is 157 (149 unit + 2 error_coverage + 3 round_trip + 3 vectors). Net +1 as advertised. **Pass.**

## 8. strum 0.26 vs. 0.27

The dev-dep is pinned at 0.26 to align with md-codec's `crates/md-codec/Cargo.toml`. The latest published strum is 0.27 (verified via `Cargo.lock` ecosystem; not separately polled here). Per the project's "don't adopt a higher bar than md1" principle (CLAUDE.md cross-repo coordination section) and the FOLLOWUPS entry's resolution recipe ("This is the path md-codec uses for its `error_coverage` test"), 0.26 alignment is the right call for Phase 1.

I'm not aware of any security advisory or correctness fix in 0.27 that would affect the `EnumIter` derive macro's behavior for this use case. (`EnumIter` is a thin proc-macro that emits `impl IntoIterator` over unit variants; semantics are stable across 0.25/0.26/0.27.) The derive features used (`features = ["derive"]`) are present in both versions.

**Recommendation:** keep at 0.26 for Phase 1 alignment with md-codec. If/when md-codec bumps to 0.27, mk-codec can follow in lockstep — perhaps a small `cross-repo` FOLLOWUPS entry to track that as a coordinated bump, but it's not blocking v0.2.0. **Pass.**

## 9. Anything missing?

Two administrative items, both Suggestions:

### S-1 (Suggestion) — Flip the FOLLOWUPS status for `error-variant-exhaustiveness-gate-strum`

`design/FOLLOWUPS.md` line 135 still reads `Status: open`. Phase 1's commit message says "Closes `error-variant-exhaustiveness-gate-strum`" and Phase 1 implements option (1) of the two-option resolution recipe. Recommend flipping the entry to:

```
Status: resolved 901596a (v0.2.0 Phase 1) — adopted option (1)
        from the resolution recipe (strum 0.26 dev-dep + EnumIter
        on a hand-written mirror enum at tests/error_coverage.rs).
        Option (2) (move into src/error.rs unit-test module) is
        not pursued; the mirror-enum + is_exempt() pair gives the
        same compile-time safety with the same maintenance gesture
        as md-codec's pattern.
```

This is a docs-only follow-up and can land as part of the Phase 4 "release plumbing" commit (or sooner — the user's call).

### S-2 (Suggestion, low-priority) — Pointer comment tense in `tests/vectors.rs`

The pointer comment at `tests/vectors.rs:242-247` says "as of v0.1.2" but the work landed on the v0.2.0 branch. Not wrong (v0.2.0 supersedes v0.1.2 in the milestone hierarchy and the gate genuinely existed for the brief v0.1.2 window the user is squashing into v0.2.0), but inconsistent with the v0.2.0 framing. Two clean options:

- Drop the version stamp: "lives in `tests/error_coverage.rs`" (timeless).
- Update to v0.2.0: "as of v0.2.0".

Either is fine; neither is blocking.

## Issues by tier

- **Critical (must fix before Phase 2):** none.
- **Important (should fix before Phase 2):** none.
- **Suggestions (nice to have):**
  - S-1: flip FOLLOWUPS `error-variant-exhaustiveness-gate-strum` status from `open` to `resolved 901596a`.
  - S-2: tense polish on `tests/vectors.rs:242-247` pointer comment (drop or update the version stamp).

## Top-line conclusion

**Proceed to Phase 2 (V18 — 0x16 testnet nested-segwit vector).** Phase 1's strum-driven gate is a faithful, correct port of md-codec's pattern with one small UX improvement (the leaked-exemption contradiction guard). All 22 source `Error` variants are mirrored case-for-case; every parameterized prefix is a valid literal prefix of the actual `Display` rendering; the lone `CardPayloadTooLarge` exemption is structurally justified (decoder calls `reassemble_from_chunks`, encoder calls `split_into_chunks`, the only `Err(CardPayloadTooLarge)` site). 157 tests pass, clippy + fmt clean, vectors.rs cleanup leaves the remaining tests intact. The two follow-ups are administrative (FOLLOWUPS status flip, pointer-comment tense polish) and can land alongside Phase 4 or whenever convenient.
