# Phase 5 Code Review — String Layer

**Commit:** `12c54f8`
**Branch:** `feature/v0.1.0-implementation`
**Reviewer:** `superpowers:code-reviewer` (Opus, dispatched 2026-04-29)
**Plan reference:** `design/IMPLEMENTATION_PLAN_mk_v0_1.md` §"Phase 5 — String layer (BCH + chunking, forked from md-codec) (TDD)"
**Spec ground truth:** `design/SPEC_mk_v0_1.md` §§2.4, 2.5, 2.6, 4 (validity rules 11–14)
**Closure design:** `docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`
**Saved by:** controller (subagent could not write to disk in this run)

---

## Status

**DONE_WITH_CONCERNS** — one important + several minor; no critical blockers.

## Top-line conclusions

The Phase 5 work is solid and faithful to the spec. The BCH fork is mechanically clean (only HRP, target constants, module paths, and `BchUncorrectable`-string changes; the polynomial/syndrome math is byte-for-byte preserved from md-codec). String-layer header packing matches SPEC §2.5 modulo the `total_chunks` off-by-one, which is the right pragmatic resolution since the spec is internally inconsistent ("5 bits, range 1..=32"); the FOLLOWUPS entry to clarify the spec is correctly placed. Cross-chunk-hash, chunked reassembly, and pipeline determinism are all correct. CSPRNG-by-default for `chunk_set_id` is consistent with closure Q-5 wording. 150 tests pass locally.

One **important** finding: the BIP draft still says `total_chunks (5 bits, range 1..=32)` (`bip/bip-mnemonic-key.mediawiki:168`) without documenting the `count - 1` wire encoding. SPEC §2.5 has the same gap. The FOLLOWUPS entry tracks this for pre-bip-submission; a third-party implementer reading only the BIP today would miss the off-by-one. Recommend either landing the spec wording fix in Phase 6 alongside the vector corpus (so vectors anchor the convention) or adding a clarifying parenthetical now.

Recommendation: proceed to Phase 6 after applying the minor fixes inline.

---

## Critical
None.

## Important

### I-1. BIP/SPEC `total_chunks` wire-encoding text is silent on the off-by-one

- Files: `design/SPEC_mk_v0_1.md:106`, `bip/bip-mnemonic-key.mediawiki:168`
- Both currently say `total_chunks (5 bits, range 1..=32)`. The implementation (`crates/mk-codec/src/string_layer/header.rs:88` and `:146`) encodes `count - 1` on the wire — the only way 32 distinct values fit in a 5-bit field. This is correct, defensible (matches md1 convention if md1 has a similar capacity claim), and the off-by-one decode is symmetric in `to_5bit_symbols`/`from_5bit_symbols`. The FOLLOWUPS entry `chunked-header-total-chunks-wire-encoding-clarification` records the cleanup — but the decision is implicitly load-bearing for *anyone* writing an interoperating decoder. Two paths forward, in priority order:
  1. **Recommended (cheap, fits Phase 5 fixup commit):** add a one-sentence note in SPEC §2.5 and BIP "Chunked header" — e.g., "*Wire encoding:* `total_chunks` is encoded as `count − 1` (5-bit field 0..=31 maps to semantic range 1..=32)." This closes the FOLLOWUPS item now and removes pre-bip-submission risk.
  2. Or reduce `MAX_CHUNKS` to 31 so the literal SPEC wording becomes correct without needing a wire-encoding note. Not recommended — it gratuitously cuts capacity for no gain.

The reviewer guidance asked: "Is this the right fix, or should the SPEC say '1..=31' instead, or should `MAX_CHUNKS` be reduced to 31?" The implementation choice (encode `count-1`, semantic range 1..=32) is the right one — preserves capacity and matches the closure-locked claim of "32 chunks per card." The fix is in the spec text, not the code.

## Minor / Suggestions

### M-1. `decode_string` second-pass `case_check` redundancy

- `crates/mk-codec/src/string_layer/bch.rs:653` and `:656`: code calls `case_check` then unconditionally lower-cases the entire string. The lowercased string is then split and HRP-checked. This is mostly fine, but `s.to_lowercase()` allocates even for all-lowercase input; consider `s.to_ascii_lowercase()` (cheaper, in-place-byte semantics, sufficient since the alphabet is ASCII). Pure perf nit; the fork inherits this from md-codec.

### M-2. `from_5bit_symbols` never produces wire `total_chunks > 31`

- `crates/mk-codec/src/string_layer/header.rs:154`: the defensive `total_chunks > MAX_CHUNKS` check is unreachable under the current `& 0x1F` masking (max wire = 31, +1 = 32 = `MAX_CHUNKS`). The comment correctly notes this is a future-proofing guard. Consider either (a) leaving as-is with the existing comment, or (b) removing the check and relying on a `debug_assert!` since the path is structurally unreachable. Either is fine; the current version errs on the side of safety, which is consistent with the rest of the codebase.

### M-3. `decode_rejects_perturbed_cross_chunk_hash` test pattern is brittle

- `crates/mk-codec/src/string_layer/pipeline.rs:272-306`: the test perturbs the last byte of the last chunk's *fragment*, then re-encodes. The comment correctly notes "the BCH layer will correct it to *something*" — but the test relies on the perturbed last fragment not happening to land somewhere that the BCH t=4 correction silently un-perturbs into a CRC-valid bytecode. With the current fixture this works, but a future fixture change could mask the test. Consider perturbing in 5-bit-symbol space *after* re-encoding, or pinning a specific cross-chunk hash byte position that is BCH-distance > 4 from any valid codeword in the chunk's data part. Low priority.

### M-4. `KeyCard::new` documentation gap

- `crates/mk-codec/src/key_card.rs:62`: the constructor doc reads cleanly, but it does not assert any invariant on `policy_id_stubs.len() >= 1`. The bytecode encoder later rejects empty-stub vectors with `InvalidPolicyIdStubCount`. Consider documenting that calling code is responsible for non-empty stubs (or alternatively returning `Result<Self>` with the check in the constructor). Currently the validation is shifted to encode-time, which is consistent with md-codec's pattern but worth noting in the doc.

### M-5. Pipeline `decode` SingleString-with-extra-strings error message

- `crates/mk-codec/src/string_layer/pipeline.rs:137-139`: a decoder that gets `[SingleString_string, Chunked_string, ...]` returns `ChunkedHeaderMalformed("multiple strings supplied with SingleString header")`. The variant name suggests a chunked-set issue; the actual condition is "first string was SingleString, but more strings were supplied." Consider a more specific variant name (e.g., add a `MixedHeaderTypes` discriminator) or accept the slight semantic drift. Low priority — the error is reachable only through user error and the message text is clear.

### M-6. `encode_with_chunk_set_id` silently ignores override on SingleString path

- `crates/mk-codec/src/string_layer/pipeline.rs:67`: doc explicitly notes "single-string encodings have no `chunk_set_id` field, so the value is silently ignored." This is friendly behavior, but a Phase-6 vector regenerator that pins chunk_set_id and asserts `s1 == s2` will not detect a SingleString-vs-Chunked drift if the cutoff changes. Consider returning `Err(Error::ChunkedHeaderMalformed("chunk_set_id supplied but encoding is SingleString"))` when the override is non-`None` and the bytecode lands in single-string territory. Or document that the test harness should assert chunked-vs-single explicitly before pinning. Minor.

### M-7. `bch_decode.rs` test imports `MK_LONG_CONST`/`MK_REGULAR_CONST` from `crate::consts`

- `crates/mk-codec/src/string_layer/bch_decode.rs:602-606`: the cross-module import path is fine, but slightly inconsistent with the production code in `bch.rs` which imports from the same crate root. Cosmetic; no fix needed.

## Observations / Confirmations

- **BCH fork fidelity (Q1).** Diff between `crates/md-codec/src/encoding.rs` and `mk-codec/src/string_layer/bch.rs` is exactly: HRP comments, `MD_*` → `MK_*` constants imported from `crate::consts`, module-path adjustments (`crate::encoding` → `crate::string_layer`, `pub(in crate::encoding)` → `pub(in crate::string_layer)`), the swap of `encode_string(header, payload)` for `encode_5bit_to_string(data_5bit)`, `BchUncorrectable` upgraded to `String`-parameterized with cause text, dropped `MD_*_CONST` doc-comments, dropped the duplicate NUMS reproducer (correctly relocated to `consts.rs`), and renamed/dropped pinned-checksum tests (deferred to Phase 6 vectors). No hidden polymod/HRP-mixing changes. `polymod_step` math is byte-identical.
- **HRP math (Q1 cross-check).** `'m'=0x6D=0b01101101` → high 3 = `0b011`=3, low 5 = `0b01101`=13. `'k'=0x6B=0b01101011` → high 3 = `0b011`=3, low 5 = `0b01011`=11. Result `[3, 3, 0, 13, 11]` matches `bch.rs:973`.
- **Cross-chunk hash (Q2).** `chunk.rs:67` hashes `canonical_bytecode` exactly (no extra bytes); `chunk.rs:195` recomputes over the reassembled stream excluding the trailing 4 bytes. SPEC §2.6 satisfied.
- **Header bit allocation (Q3).** `header.rs:80-98` packs `version(5) + type(5) + csid_high5(5) + csid(5) + csid(5) + csid_low5(5) + total_chunks_wire(5) + chunk_index(5)` — exactly 40 bits = 8 symbols. `chunk_set_id` is packed big-endian (high-order 5 bits at symbol 2). Endian convention is reasonable but should be explicit in the BIP — verified that the BIP `:167` simply states "20 bits" without endian, which is potentially ambiguous for cross-implementations. Minor; follow with the FOLLOWUPS entry.
- **Off-by-one (Q4).** Encoder/decoder are inverse-symmetric; the only concern is spec wording (see I-1).
- **SPEC §4 rules 11–14 mapping (Q5).**
  - Rule 11 (`ChunkSetIdMismatch`): `chunk.rs::reassemble_from_chunks:149` → tested at `chunk.rs:267` and `pipeline.rs:262`.
  - Rule 12 (`ChunkedHeaderMalformed`): `chunk.rs:131-167`, `header.rs:154-163` → tested at `chunk.rs:298`, `chunk.rs:316`, `chunk.rs:328`, `header.rs:300`.
  - Rule 13 (`CrossChunkHashMismatch`): `chunk.rs:196` → tested at `chunk.rs:285`, `pipeline.rs:272`.
  - Rule 14 (`MalformedPayloadPadding`): `pipeline.rs:130` → tested at `pipeline.rs:309`. Coverage is complete.
- **CSPRNG usage (Q6).** Closure design Q-5 (`docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md:113`): "encoders SHOULD generate it from a cryptographically secure random source." The pipeline using `getrandom::getrandom` (line 47) and panicking on failure is consistent — `getrandom` failure on a modern OS is fatal-class anyway, and no key material has been emitted. md-codec's content-derived default is a different policy choice; mk1's CSPRNG-default matches the closure lock.
- **v0.1 emit policy (Q7).** Pipeline.rs module doc lines 14-22 correctly document the de-facto behavior. The BIP draft does NOT make a "long-code only for chunks" claim — the Length envelope (lines 131-132) and chunked header table (lines 174-180) are silent on per-chunk variant choice. No conflict to fix in the BIP. The implementation plan claim is what diverged; pipeline.rs's module doc is the right place to note it. Good.
- **Bytecode boundary (Q8).** SingleString path (`pipeline.rs:73-82`) does NOT call `split_into_chunks` and does NOT include cross-chunk hash. Chunked path (`pipeline.rs:96-104`) goes through `split_into_chunks` which always appends the hash. Correct.
- **Determinism (Q9).** No `HashMap`/`HashSet`/time/RNG anywhere in the bytecode layer or in `encode_bytecode_stream` when `chunk_set_id` is supplied. `split_into_chunks` is index-based slicing. BCH checksum is a pure function. `encode_with_chunk_set_id` is byte-deterministic in `(card, chunk_set_id)`. Test `pipeline.rs:209-217` and integration test `tests/round_trip.rs:62-79` both assert byte equality.
- **Integration with prior phases (Q10).** `pipeline::decode` calls `decode_bytecode` (Phase 4) at line 142 and 151; round-trip tests at `tests/round_trip.rs` and `pipeline.rs:191-207` confirm structurally-equal `KeyCard` recovery. `KeyCard::new` constructor (`key_card.rs:62`) compiles, integrates, and is the only way to construct `KeyCard` for non-`#[non_exhaustive]`-allowed external callers — the integration tests prove this.
- **Test count.** 147 unit + 3 integration = 150 green; matches the commit message.

## Files relevant to this review

- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/bch.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/bch_decode.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/header.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/chunk.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/pipeline.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/mod.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/error.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/key_card.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/lib.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/round_trip.rs`
- `/scratch/code/shibboleth/mnemonic-key/design/SPEC_mk_v0_1.md`
- `/scratch/code/shibboleth/mnemonic-key/bip/bip-mnemonic-key.mediawiki`
- `/scratch/code/shibboleth/mnemonic-key/design/FOLLOWUPS.md`
- `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/encoding.rs` (fork source)
- `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/encoding/bch_decode.rs` (fork source)

Recommended fixup-commit scope: **I-1 only** (SPEC §2.5 and BIP "Chunked header" wording fix; close the FOLLOWUPS entry). Everything else (M-1..M-7) is appropriate to defer to FOLLOWUPS at tier `v0.1-nice-to-have` or roll into Phase 6/7 cleanups.
