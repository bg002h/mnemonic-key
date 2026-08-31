# cycle-prep recon — 2026-08-31 — SPEC_chunk_set_id_verification (parked draft 9fbbe36)

**Sync state per repo:**
- `mnemonic-key`: HEAD `2307dce9` on `main`, 2 ahead of `origin/main` (both continuity/doc commits: `2307dce` opens this cycle, `cb8f865` a push-report append — no code). Clean tree.
- `descriptor-mnemonic`: HEAD `7eca44b6` on `main`, 2 ahead of `origin/main` (both followups/report commits re: md-codec 0.43.0 publish block — no code touching csid). Clean tree.
- `seedhammer`: HEAD `5f02773c` on `main`, 0 ahead/0 behind `origin/main`. Clean tree.
- `mnemonic-engrave`: HEAD `1103d9ee` on `master`, 8 ahead of `origin/master` (F-424 publish-blocker followups, unrelated to csid). Clean tree.
- No untracked files relevant to this recon in any of the four trees.

## Per-claim verification

### 1. Derivation formula ("top 20 bits of SHA-256(canonical_bytecode), MSB-first")
- draft says (line 8): *"the top 20 bits of `SHA-256(canonical_bytecode)`, MSB-first"*
- verdict: **ACCURATE**
- evidence: `crates/mk-codec/src/string_layer/chunk.rs:45-48` — `pub fn derive_chunk_set_id(canonical_bytecode: &[u8]) -> u32 { let hash = sha256::Hash::hash(canonical_bytecode).to_byte_array(); top20(&hash) }`; `top20` at line 52-54 takes bytes `[0][1][2]` shifted MSB-first into a 20-bit value. mk-codec still 0.5.0 (`crates/mk-codec/Cargo.toml:3`).
- **FLAGGED as formula-prose restatement** — see sweep below.

### 2. "Both decoders use the csid only to check that all chunks in a set carry the same value"
- draft says (line 11): *"Both decoders use the csid only to check that all chunks in a *set* carry the same value — reassembly matching."*
- verdict: **ACCURATE**
- evidence: Rust — `crates/mk-codec/src/string_layer/chunk.rs:222-224`, `reassemble_from_chunks`: `if chunk_set_id != set_id { return Err(Error::ChunkSetIdMismatch); }` — no derivation check anywhere in the reassembly/decode path. Go — `mk/mk.go:192-193`: `if f.header.ChunkSetID != first.ChunkSetID { return nil, errChunkSetIDMismatch }` — same shape, no derivation check.

### 3. Fork test comment "the decoder does not validate the csid value"
- draft says (line 12-13): *"the decoder does not validate the csid value."*
- verdict: **ACCURATE, still present verbatim**
- evidence: `seedhammer/mk/encode_test.go:207-208`: `// chunk_set_ids, not a SHA-derived csid, so byte-identical re-emission is\n// impossible (and the decoder does not validate the csid value). For each`

### 4. "DECISIONS.md D-15 still describes the field as 'per-encoding random … used for reassembly mismatch detection, nothing more', which D-16 now contradicts"
- draft says (lines 13-15): quoted above
- verdict: **DRIFTED** (the literal quote is still present verbatim, but the draft's framing is incomplete/misleading — not because ground shifted, but because it omits something that already existed when the draft was written)
- evidence: `design/DECISIONS.md:190` still reads *"identifies all chunks belonging to one card-encoding, used for reassembly mismatch detection, nothing more"* — literal quote holds. BUT `design/DECISIONS.md:200`, immediately following D-15, already carries: *"**Amended by D-16 (2026-08-14):** the phrase 'per-encoding random tag' above describes what the field is *not* — not per-wallet — and that reading stands. The value is no longer drawn from entropy; it is derived from the payload."* This amendment note was introduced by commit `a38a908` (2026-08-14, `feat(mk-codec 0.5.0)!: derive chunk_set_id from the payload, not entropy`) — **five days before the draft was authored (2026-08-19)**. So the "contradiction" the draft names was already reconciled in-place at draft time; this is not a post-draft ground shift, it is a fact the draft's author had available and didn't cite. `commit 40efcc7` ("encode() doc caught up to 0.5.0") only touched `crates/mk-codec/src/key_card.rs`'s `encode()` doc comment (dropped the stale CSPRNG claim, added a derivation summary) — it did not touch `DECISIONS.md` at all, so it is unrelated to the D-15/D-16 question.

### 5. Named tests `an_explicit_chunk_set_id_still_wins` / `canonical_payload_is_chunk_set_id_invariant`
- draft says (lines 40-42, 46): both tests exist; the first's comment says *"the id is opaque to content."*
- verdict: **ACCURATE**
- evidence: `crates/mk-codec/tests/chunk_set_id_determinism.rs:112-127`, comment at line 121: `// Both must still decode to the same card: the id is opaque to content.` `crates/mk-codec/tests/canonical_payload.rs:155` (fn at line 155), doc comment at lines 149-152 calls it *"Cross-`chunk_set_id` determinism (**load-bearing**)"*.

### 6. "Breaks 19 tests" (blast radius of the reverted change)
- draft says (line 37): *"It also breaks **19 tests**"*
- verdict: **cannot re-verify the reverted number; current surface measured instead, and it is larger and differently shaped than 19**
- evidence/counts (commands below):
  - `mk-codec` crate: 17 distinct `#[test]` fns call `encode_with_chunk_set_id(...)` directly, plus 1 more (`every_vector_round_trips` in `tests/vectors.rs`) that calls it indirectly via a helper (`exercise_clean_vector`) once per clean corpus vector (≥18 clean vectors per `assert!(clean_count >= 18, ...)` at `tests/vectors.rs:167`) = **18 mk-codec `#[test]` fns touch this surface.**
    - command: `for f in $(grep -rl "encode_with_chunk_set_id" --include="*.rs" crates/); do awk '...' "$f"; done | sort -u | wc -l` → `17` direct + 1 indirect (`vectors.rs`), listed in full in the working transcript.
  - `mk-cli` crate: at least 9 more `#[test]` fns reference `chunk_set_id`/`chunk-set-id` across 4 files (`encode_chunk_set_id.rs`: 4, `from_md1_set.rs`: 2, `cli_mk1_repair_reverify.rs`: 2, `keys_batch.rs`: 1); `template_id_stub.rs` has none despite matching the filename grep.
  - Total measured now: **~27 `#[test]` fns across `mk-codec` + `mk-cli`** reference explicit chunk-set-id pinning in some form (not all would necessarily fail under a derivation check — several already test error paths). This is a materially different shape than "19 tests" and cannot be reconciled to it without the original reverted diff, which no longer exists in the tree.

### 7. Blast-radius table (three repos)
- draft table (lines 105-109): mnemonic-key 10 / mnemonic-engrave 6 / seedhammer 2
- verdict: **mnemonic-key row DRIFTED, mnemonic-engrave row ACCURATE (for its stated narrow scope), seedhammer row STRUCTURALLY-WRONG**
- evidence:
  - **mnemonic-key "10"**: `grep -rnE "encode_with_chunk_set_id\([^,]*, ?0x12345\)|encode_with_chunk_set_id\([^,]*, ?0xABCDE\)" --include="*.rs" crates/` → **8 call sites, 6 distinct `#[test]` fns** (`deterministic_round_trip_with_explicit_chunk_set_id`, `round_trip_typical_card_chunked`, `round_trip_explicit_path_chunked`, `deterministic_encoding_with_explicit_chunk_set_id`, `decode_rejects_chunk_set_id_mismatch`, `an_explicit_chunk_set_id_still_wins`) — not 10, by either call-site or test-fn counting.
  - **mnemonic-engrave "6"**: `grep -n "0x12345" crates/me-cli/tests/vectors/bundle-md1-mk1.json crates/me-cli/src/manifest.rs` → 3 hits in the golden JSON (lines 9, 27, 38) + 3 hits in `manifest.rs` (lines 250, 260, 272) = **exactly 6**, matching the draft's stated scope precisely. (Note: `me-cli/src/bundle.rs` alone has 13+ further references to `0x12345` in its own test module, well outside this narrow "golden fixture + manifest.rs" count — the draft's "6" is accurate only for the two files it names.)
  - **seedhammer "2"**: measured **7**, not 2. `seedhammer/mk/mk_test.go` defines `parityVectors` (lines 59-125), a table of **7** named chunked golden-string sets (`V1`..`V7`), each with an explicit (non-derived, pre-0.5.0-vintage) baked-in `chunk_set_id`, consumed by both `TestDecodeParity` and `TestEncodeGoldenRoundTrip`. `git log --oneline -- mk/mk_test.go` shows only 3 commits ever, none since 2026-08-19 (`e9a6df7`, `cb8eff9`, `c1f1fc8`, all pre-dating the draft) — this file has not changed since before the draft was written, so "2" appears to have been a miscount at draft time (possibly conflating D-16's separate mention of the *cross-implementation gate* test pinning *"two chunks engraved on steel"* — a chunk count, not a vector count — with this table), not a post-draft ground shift.

### 8. Cross-chunk hash reuse ("already computes that hash", "already verified, 4 bytes")
- draft says (line 31-32, 73-74): the chunk layer already computes the hash for cross-chunk integrity; that hash is "already verified, 4 bytes"
- verdict: **ACCURATE**
- evidence: `crates/mk-codec/src/string_layer/chunk.rs:153-156` (`split_into_chunks`): `let hash = sha256::Hash::hash(canonical_bytecode); ... stream.extend_from_slice(&hash.to_byte_array()[..CROSS_CHUNK_HASH_BYTES]);`. `chunk.rs:277-286` (`reassemble_from_chunks`): computes `sha256::Hash::hash(bytecode)` again and compares the trailing `CROSS_CHUNK_HASH_BYTES` (=4, per `crate::consts::CROSS_CHUNK_HASH_BYTES`), returning `Error::CrossChunkHashMismatch` on mismatch — this check IS already unconditionally run at decode time today, distinct from and pre-dating anything the draft proposes.

### 9. "Single-string (unchunked) encodings carry no csid"
- draft says (line 125): *"Single-string (unchunked) encodings carry no csid and are unaffected."*
- verdict: **ACCURATE**
- evidence: `crates/mk-codec/src/string_layer/pipeline.rs:61-69` — the `SingleString` branch builds a header with only `{version}`, no chunk_set_id field at all. Doc comment at `pipeline.rs:50-52` and `key_card.rs` (`encode_with_chunk_set_id`) both state: *"single-string encodings have no `chunk_set_id` field, so the value is silently ignored."*

### 10. `mk encode --chunk-set-id` purpose text ("vector regeneration and conformance fixtures")
- draft says (lines 97-99): stated purpose is pinning a value for "vector regeneration and conformance fixtures"
- verdict: **ACCURATE, verbatim**
- evidence: `crates/mk-cli/src/cmd/encode.rs:97-100`: *"Pin the 20-bit `chunk_set_id` (hex, `0x` prefix optional) instead of deriving it from the payload. Chunked output only — single-string encodings carry no such field. **For vector regeneration and conformance fixtures**; the derived default is already deterministic, so ordinary encoding never needs this."*

### 11. `Error::ChunkSetIdMismatch` semantics; no `ChunkSetIdNotDerived` yet
- draft says (lines 122-124): mismatch on the new check would be `Error::ChunkSetIdNotDerived`, distinct from `ChunkSetIdMismatch` which means chunks disagree with each other
- verdict: **ACCURATE**
- evidence: `crates/mk-codec/src/error.rs:78-80`: `/// For chunked input: chunks have inconsistent chunk_set_id values. Used at reassembly time to detect mixed-card-set inputs. #[error("chunk_set_id mismatch across chunks")] ChunkSetIdMismatch`. `grep -rn "ChunkSetIdNotDerived" .` across the whole `mnemonic-key` tree → **zero matches**, confirmed absent.

### 12. mk-codec crate version and public API
- draft implicitly assumes `derive_chunk_set_id` is reachable to write a verification check against
- verdict: **ACCURATE**
- evidence: `crates/mk-codec/Cargo.toml:3` → `version = "0.5.0"` (mk-cli, the `mk` binary, is separately at `0.13.0` — matches the brief's note that "mk 0.13.0" and "mk-codec 0.5.0" are different crates, not a contradiction). `crates/mk-codec/src/lib.rs:52`: `pub use string_layer::derive_chunk_set_id;` — public API today.

## New ground the draft does not know

### 13. Consumer map (current, all four trees)
- **`descriptor-mnemonic` converter — grouping (SPEC A3(a) step 2)**: `crates/md-cli/src/seat/input.rs:159` — `StringLayerHeader::Chunked { chunk_set_id, .. } => GroupId::Chunked(chunk_set_id)`. Groups deduped mk1 input strings by declared `chunk_set_id` BEFORE calling `mk_codec::decode`; module doc (`seat/input.rs:1-24`) states the ORDER is load-bearing: "two different cards pinned to one chunk-set id merge into one group at step 2 and then refuse at reassembly."
- **`descriptor-mnemonic` converter — r1-I2 refusal message**, `crates/md-cli/src/seat/input.rs:203-208`, quoted verbatim: *"chunk-set {set_id}: the {N} string(s) declaring this id do not reassemble into one key card: {e}. Two DIFFERENT cards pinned to one chunk-set id merge into one group here and refuse exactly like this — re-mint one of them so the set ids differ"*. `{e}` is `mk_codec::decode`'s own error (today: a generic reassembly-shape error like "received 5 chunks, header declares total_chunks = 2" — NOT a derivation mismatch, since no derivation check exists).
- **`descriptor-mnemonic` converter — `--seat '@i=<chunk-set-id>'`**: `crates/md-cli/src/seat/directive.rs:1-25` — the directive's id token IS the mk1 `chunk_set_id`, printed by the A3 refusal; the module doc notes the "ambiguous id" case is provably unreachable specifically because grouping-then-reassembly (item above) already forecloses it — this reachability argument depends only on today's intra-set consistency check, not on derivation.
- **`descriptor-mnemonic` md1 minting**: `crates/md-cli/src/cmd/descriptor.rs:371-375` — `mint_md1_cards` returns an md-codec `chunk_set_id`, printed to stderr as `chunk-set-id: 0x{csid:05x}` — this is the **md1** id (out of this draft's scope per its own "Not in scope" section), separate from the mk1 id discussed above.
- **`mnemonic-engrave` me-cli — `me bundle` grouping**: `crates/me-cli/src/bundle.rs:218-239` — BTreeMap keyed by `chunk_set_id`, separately for md1 and mk1 groups (`md1_groups.entry(chunk_set_id)...`, `mk1_groups.entry(chunk_set_id)...`), deterministic ordering by id. Golden fixture + manifest.rs usage already covered in claim 7.
- **`seedhammer` fork device/GUI**: `gui/mk1_inspect.go:59,61` and `gui/md1_gather.go:41,43` — incremental scan-progress accumulator: `g.setID = h.ChunkSetID` on first chunk seen, then `h.ChunkSetID != g.setID` gates whether a newly-scanned chunk belongs to the in-progress card as the operator scans multiple QR codes on-device. `gui/bundle.go:83,92` — classifies a scanned string as `clsChunkedMK1`/`clsChunkedMD1` keyed by `h.ChunkSetID`. `gui/md1_gather.go:154,165` — `errors.Is(err, md.ErrChunkSetIDMismatch)` is a distinct, separately-reported error case in the device's scan-progress UI (R0-C1).

### 14. Falsification check on the draft's central premises
- **(a) "Position A already delivers the stated goal"** — facts, not a ruling: the r1-I2 diagnostics-gap finding shows that today's refusal (`mk_codec::decode`'s generic reassembly error, surfaced through the converter's message above) does NOT distinguish "two different cards collided on the same 20-bit id" from any other reassembly malformation — the operator sees "received N chunks, header declares total_chunks = M" or similar, not "these two payloads hash to different derived ids." A derivation-comparison check (Position B) would produce a strictly more specific diagnostic in exactly this scenario (comparing the derived hash of each candidate payload against the declared id), which is the shape of thing the draft's own "Why it is worth closing" section (lines 26-29) already anticipated in the abstract ("chunks of different cards assembled together, even on a csid collision") before the converter cycle gave it a concrete, already-observed trigger.
- **(b) "B's marginal gain … is a narrow threat"** — facts, not a ruling: the converter's use of csid for GROUPING (item 13 above) means a wrong/colliding csid is not merely a decode-time nuisance on one card; it is now the sole key the seating/grouping step 2 uses to partition an entire multi-card batch BEFORE any content is inspected. A group-key collision merges unrelated cards' chunks into one bucket that step 3 then either successfully (and silently) reassembles into a wrong card, or fails on (per today's intra-set consistency check, not derivation). **Major additional fact, not previously known to the draft: `descriptor-mnemonic`'s own sibling format, `md-codec`, ALREADY implements Position B, unconditionally, and treats it as a funds-load-bearing invariant** — `crates/md-codec/src/chunk.rs:403-415`: comment *"Cross-chunk integrity check — UNCONDITIONAL regardless of `opts` (the content-id oracle; P0.2 funds-load-bearing invariant)"*, code `let derived_csid = derive_chunk_set_id(&md1_id); if derived_csid != expected_csid { return Err(Error::ChunkSetIdMismatch { expected: expected_csid, derived: derived_csid }); }`. This machinery is old — `git log -S"derived_csid != expected_csid" --oneline -- crates/md-codec/src/chunk.rs` finds it present at least as of the `5350f8a7` flatten commit (2026-04-30), i.e. **already existed four months before the draft was written**, not a post-draft ground shift, but a fact the draft's "Not in scope" section did not check.
- **Naming collision, newly surfaced by this recon**: `md-codec::Error::ChunkSetIdMismatch` means "derived hash != declared id" (Position-B content-tamper detection) — md-codec's "chunks disagree with each other" case instead uses a differently-named `Error::ChunkSetInconsistent` (`crates/md-codec/src/chunk.rs:375`). This is the OPPOSITE assignment from mk-codec today, where `Error::ChunkSetIdMismatch` means "chunks disagree with each other" (`crates/mk-codec/src/error.rs:78-80`) and no derivation-check error exists at all. The two sibling formats already use the identical string `ChunkSetIdMismatch` for two different failure modes. The draft's proposed new name `ChunkSetIdNotDerived` for mk's Position-B case would avoid colliding with mk's own existing `ChunkSetIdMismatch`, but would still leave the two formats disagreeing on which name maps to which of the two concepts.
- **"Not in scope: md-codec's csid" boundary** — the draft's own text (line 144): *"It has always been derived, but whether *it* verifies is a separate question and a separate cycle."* This is **not accurate as a description of present or even four-months-past reality**: md-codec does not merely "maybe verify" pending a future cycle — it already verifies, unconditionally, and its own source comment calls the check funds-load-bearing. The boundary decision itself (defer md1 to a separate cycle) may still be reasonable scoping, but the stated REASON for treating it as open/unknown is false; the two formats are not symmetrically unresolved on this question, they are already asymmetric, with md1 having shipped and load-bearing what mk1's draft is still merely proposing.

## Formula-prose restatements in the draft

Per the standing "no formula restatement in prose" rule, every line below states an internal of `derive_chunk_set_id` (or a size derived from it) in prose rather than as an executable vector:

- **Line 8-9**: *"the top 20 bits of `SHA-256(canonical_bytecode)`, MSB-first"* — full derivation formula restated in prose. Primary target.
- **Line 24**: *"Comparing `csid == derive_chunk_set_id(reassembled_payload)` is one line and catches:"* — the proposed normative check itself, written as an inline comparison expression rather than a vector (input payload, expected csid, expected pass/fail).
- **Line 28**: *"20 bits is only ~1e6, so collisions are not negligible at corpus scale"* — quantitative property (field width → collision space) derived from the formula, stated in prose.
- **Line 31-32 / 73-74**: *"the chunk layer already computes that hash for its cross-chunk integrity suffix"* / *"the cross-chunk hash (already verified, 4 bytes)"* — restates the cross-chunk hash's byte-width (4) and relationship to the derivation hash in prose.
- **Lines 119-121 (Normative change §1)**: *"`mk_codec::decode` ... computes `derive_chunk_set_id(canonical_bytecode)` over the reassembled payload and compares it to the header's `chunk_set_id`."* — this is the actual normative requirement, stated as prose describing a formula/comparison rather than as an executable test vector (input bytes → expected accept/reject + error variant). This is the single most load-bearing instance: it is the spec's operative clause, and it is exactly the shape the standing rule targets.

## Cross-cutting observations

- The draft's two BLOCKED-experiment claims that cite a **count** (line 37 "19 tests", table lines 105-109 "10/6/2") are the ones that drifted or were wrong; claims about **mechanism** (derivation formula, decoder behavior, error semantics, doc text) all held up as ACCURATE. This matches the standing lesson that records are the weaker half — the numbers need re-derivation before the next draft reuses them, the code-behavior descriptions mostly do not.
- The mnemonic-engrave "6" count is the one blast-radius row that is exactly right, and only because it was scoped to two named files; every other count in the table used an implicit scope ("Rust tests", "golden vectors") that turned out broader (mnemonic-key) or smaller/stale (seedhammer) than measured.
- The single biggest new fact for the next draft is not from the converter cycle at all — it's the pre-existing md-codec Position-B implementation the draft's own "Not in scope" section mischaracterized as an open question. The converter cycle (r1-I2, `--seat` grouping) supplies a second, independent, *observed* trigger (a diagnostics gap) for revisiting the same question; the two are complementary, not duplicative.

## Recommended re-grounding scope for the brainstorm walk

**Must change before the next draft is R0-reviewable:**
- Recount and requote the entire blast-radius section (line 37's "19 tests" and the three-row table) against current trees; do not carry forward any of the three old numbers unexamined — none of the three verified exactly except mnemonic-engrave's "6," and that only because it was already narrowly scoped.
- Rewrite the D-15/D-16 paragraph (claim 4) to reflect that D-15 already carries a reconciling amendment note predating the draft — the "docs follow-up" framing in "Not in scope" undersells what's already fixed there.
- Rewrite the "Not in scope: md-codec's csid" paragraph — it must state, as settled fact, that md-codec already implements and load-bears an equivalent-in-spirit Position-B check (`crates/md-codec/src/chunk.rs:403-415`), not defer that as an open question. Whether mk1 should follow suit is still a decision for the operator walk — but the evidence base changed, and "whether it verifies is a separate question" is no longer true to write.
- Fold in the r1-I2 diagnostics-gap fact (claim 14a) and the grouping-collision fact (claim 14b) as inputs to the A-vs-B walk — report them, do not let the next draft silently re-derive "Position A already delivers the stated goal" without addressing them.
- Replace every formula-prose line identified above with executable vectors before the next draft enters R0, per the standing rule (a prior draft's formula restatement was measured false; this is now a second draft instance of the same defect pattern).
- Note the `ChunkSetIdMismatch` cross-format naming collision as a fact for whichever position is chosen — it exists regardless of A-vs-B, and the error-naming design should account for it rather than accidentally reproducing it under a third name.

**May keep as-is (verified accurate, no rewrite needed):** the derivation-formula description of what mk-codec does (mechanism claims 1-3, 8-12 all held), the CLI doc-text quotes (claim 10), the named-test quotes (claim 5), and the `ChunkSetIdMismatch` semantics description (claim 11) — these are all still true and can be cited directly rather than re-verified from scratch.

**Not this recon's call:** whether to adopt Position B for mk1. That decision belongs to the operator walk, informed by the facts above — this report deliberately stops at "here is what changed and what it means for the evidence," not "here is what to do about it."
