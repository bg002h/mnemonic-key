# mk1 v0.1 Phase 6 review — initial vector corpus

**Status:** DONE_WITH_CONCERNS
**Commit:** 053a54c
**Reviewer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**File(s):**
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bin/gen_mk_vectors.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/vectors/v0.1.json
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/vectors.rs
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/Cargo.toml
- /scratch/code/shibboleth/mnemonic-key/design/SPEC_mk_v0_1.md (§2.4 amendment)
- /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/header.rs (cross-check)
- /scratch/code/shibboleth/mnemonic-key/bip/bip-mnemonic-key.mediawiki (cross-check)

**Role:** reviewer (code)

## Summary

Phase 6 lands an 8-vector corpus that hits every plan-§6.1.1 dimension correctly, with a clean, byte-deterministic generator and a thorough test harness. Hand-verification of V1 and V5 confirms the bytecode hex is structurally exact (header byte, stub count, fingerprint presence, path indicator, compact-73 layout). JSON canonicality (sorted keys, lowercase hex, 2-space indent, LF, trailing newline) holds; SHA-256 of a re-run matches the pinned constant byte-for-byte. The work is solid release material. Findings are mostly minor: one **important** doc-coherence drift between SPEC §2.4's strengthened claim and the BIP draft's softer wording, plus a stale code comment that contradicts the new SPEC §2.4 text.

## Verifications performed

1. **Structural decode of V1 bytecode** (`canonical_bytecode_hex` for V1_bip48_mainnet_1_stub_with_fp): byte-by-byte hand-decode confirmed `[0]=0x04 (fp flag)`, `[1]=0x01 (stub_count)`, `[2..6]=11223344 (stub)`, `[6..10]=aabbccdd (fp)`, `[10]=0x05 (std-table indicator for m/48'/0'/0'/2')`, `[11..15]=0488b21e (xpub mainnet version)`, `[15..19]=10203001 (synthetic parent_fp matching seed_byte=0x01 generator recipe)`, `[19..51]=0xab×32 (chain_code = seed_byte^0xAA = 0xAB)`, `[51..84]=33-byte compressed pubkey`. Total 84 bytes — matches §3.2 worked example. tests/vectors.rs:140 confirms via encoder round-trip.
2. **Structural decode of V5** (explicit-path m/9999'/1234'/56'/7'): `[10]=0xFE (explicit indicator)`, `[11]=0x04 (count)`, `[12..32]=4 hardened components × 5-byte LEB128 each` (verified each LEB128 byte against `leb128_encode(0x80002710|...)`); `[32..105]=73-byte xpub_compact`. Total 105 bytes. Matches the encoder.
3. **Path indicator audit across all 8 vectors**: V1=0x05, V2=0x03, V3=0x15, V4=0x03, V5=0xFE, V6=0x05, V7=0xFE, V8=0x07 — all match the prompt's expected values per SPEC §3.5.
4. **bytecode_header byte audit**: V1/V2/V3/V5/V6/V8 all 0x04 (fp present), V4/V7 both 0x00 (fp omitted). Matches `origin_fingerprint` field state per fixture.
5. **chunk_set_id wire-encoding round-trip**: manually decoded the 8-symbol chunked header from V1's chunk 0 (`mk1qpzg69pqq...`); recovered `version=0`, `type=1 (Chunked)`, `chunk_set_id=0x12345=74565` (matches JSON), `total_chunks=2` (off-by-one wire value 1 + 1, consistent with the SPEC §2.5 closure-locked encoding), `chunk_index=0`. Confirms FOLLOWUPS resolution of the off-by-one wire encoding from Phase 5.
6. **Generator determinism**: `cargo run --bin gen_mk_vectors --features gen-vectors -- --output /tmp/regen-vectors.json` produces a file whose SHA-256 equals the on-disk pinned `f17506e7...`. Identical byte-for-byte. Passes the byte-deterministic-regeneration property the BIP submission depends on.
7. **JSON canonicality scan** (Python parse + key-sort + hex-case audit): keys alphabetically sorted at every nesting level (top, per-vector, per-input, per-expected); all hex strings lowercase, no `0x` prefix, no separators; indentation strictly 2-space-multiples; no tabs; no CRLF; single trailing LF.
8. **Test suite**: `cargo test -p mk-codec --features gen-vectors` passes 153 tests — 147 unit + 3 round_trip + 3 vectors. No ignored tests in Phase 6's harness.
9. **Network-vs-xpub consistency check** (tests/vectors.rs:88-91): confirmed via test execution; mainnet/testnet declarations match xpub.network for all 8 vectors.

## Critical issues

None.

## Important issues

### I-1. SPEC §2.4 vs BIP draft drift on chunked-vs-SingleString reachability

- **Where:** design/SPEC_mk_v0_1.md:85 vs bip/bip-mnemonic-key.mediawiki:73.
- **What:** SPEC §2.4 (amended in this commit) now states the strong claim: *"Every conforming v0.1 mk1 KeyCard therefore encodes as a chunked card... [SingleString] is wire-defined for forward compatibility but unreachable for v0.1 encoders."* The corresponding BIP §"Length envelope" still says only the softer *"Multi-stub cards and cards with explicit-path encoding overrun this; multi-chunk MK is the norm."* This understates the v0.1 invariant.
- **Why it matters:** A BIP reviewer who reads the BIP alone will believe SingleString is a reachable path; in practice, a conforming v0.1 encoder cannot emit one. The mismatch will surface during the pre-BIP-submission cross-reference audit (FOLLOWUPS `bip-cross-reference-completeness`) but is cheaper to fix now while the §2.4 amendment is fresh.
- **Recommendation:** Mirror the SPEC §2.4 paragraph into the BIP §"Length envelope" — note the smallest-bytecode arithmetic (1+1+4+1+73=80, > 56), state that v0.1 is always chunked, keep SingleString described for forward compat. Defer if the parent agent prefers to bundle this with the BIP cross-ref audit, but record it.

### I-2. Stale `SingleString` comment in string_layer/header.rs

- **Where:** crates/mk-codec/src/string_layer/header.rs:284.
- **What:** Comment in the `wire_total_chunks_zero_decodes_to_one` test reads *"`SingleString` is the canonical encoding for one-chunk cards, but a `Chunked` with 1 chunk is still well-formed at the header layer."* Per the new SPEC §2.4 amendment, SingleString is *not* the canonical encoding for any v0.1 card — it's unreachable. The comment is now factually misleading.
- **Recommendation:** Tweak to *"`SingleString` is wire-defined for forward compatibility (SPEC §2.4) but unreachable for v0.1 encoders; a `Chunked(total=1)` is a defined-but-rare shape produced only by hand-constructed test inputs."* Trivial one-line fix.

## Minor issues

### M-1. Missing BIP 44/49/86 dictionary coverage in vectors

- **Where:** tests/vectors/v0.1.json — fixture set V1..V8.
- **What:** The plan §6.1.1 implies dictionary breadth via "BIP 44/48/84/86/87 dictionary entries" (CHANGELOG bullet). Actual coverage: BIP 48 (V1, V3, V6 — indicator 0x05 ×2 and 0x15 ×1), BIP 84 (V2, V4 — indicator 0x03 ×2), BIP 87 (V8 — indicator 0x07). **Missing: BIP 44 (0x01), BIP 49 (0x02), BIP 86 (0x04), BIP 48-nested (0x06), and explicit testnet std-table coverage beyond 0x15** — no 0x11, 0x12, 0x13, 0x14, 0x17. Of the 13 std-table entries, only 4 are exercised.
- **Why it's tolerable for v0.1:** The encoder unit tests in `bytecode/path.rs::round_trip_all_standard_paths` already cycle every entry. The vector corpus is for cross-implementation conformance, not internal encoder coverage; missing dictionary entries means a third-party encoder could pass all 8 vectors while having a bug in BIP 44/49 mainnet handling. The 8-vector budget is tight; expanding to ~14 (one per dictionary entry) is straightforward.
- **Recommendation:** Defer to FOLLOWUPS as `vector-corpus-dictionary-coverage` at tier `pre-bip-submission` (or a v0.2 corpus-expansion bucket). Don't block on it for v0.1.

### M-2. No corruption-recovery vectors

- **Where:** tests/vectors/v0.1.json + tests/vectors.rs:191-194.
- **What:** All 8 vectors have `decoder_correction: "clean"`; no negative or BCH-correctable vectors. The plan §6.1.1 doesn't require these for v0.1, but the closure design `decoder-error-variant-parity` audit gate (FOLLOWUPS `pre-bip-submission`) calls for one per Error variant. The harness already enforces `clean` everywhere; this is a budget choice consistent with the plan, but worth tracking.
- **Recommendation:** Already tracked in FOLLOWUPS. No new action needed unless the parent agent wants to surface it more explicitly in CHANGELOG.

### M-3. No third-party-implementation schema documentation

- **Where:** tests/vectors/v0.1.json schema.
- **What:** A Python or Go reference would need to know: (a) the family-token format, (b) the off-by-one `total_chunks` wire encoding, (c) the chunk_set_id symbol order (big-endian 5-bit), (d) the BCH code-variant per chunk (auto-selected; mixed allowed), (e) the cross-chunk hash construction. None of these are in the JSON itself; they live in SPEC + BIP. The JSON is structural-only (input + expected hex + expected strings). This is consistent with md-codec's vector corpus, but the prompt asks whether anything is missing for cross-implementation — the answer is "the spec is the schema, the JSON is the I/O" and that's intentional.
- **Recommendation:** No code change. Consider adding a `README.md` next to `tests/vectors/v0.1.json` pointing to the SPEC sections that define each field's wire semantics. Defer; this is a v0.2 polish item.

### M-4. `total_chunks` field validated but identity not guaranteed

- **Where:** tests/vectors.rs:171-176.
- **What:** The harness asserts `expected.total_chunks == actual_strings.len()`, which catches drift between the metadata and the emitted-string count. It does NOT cross-check `total_chunks` against the `chunked_header.total_chunks` value embedded in any chunk's first 8 symbols. A future generator bug that mis-encodes the chunked-header `total_chunks` field would slip through (until the decoder catches it via `ChunkedHeaderMalformed` — which it already does, see `string_layer/chunk.rs:131`). The decode round-trip on line 180-185 covers this in practice. So the assertion is redundant-by-design.
- **Recommendation:** No action; the existing decode-round-trip provides the structural guarantee.

### M-5. Generator panics on unrecognised CLI argument

- **Where:** crates/mk-codec/src/bin/gen_mk_vectors.rs:277.
- **What:** `panic!("unrecognised argument: {other}")` is the right behaviour for a developer-only binary, but produces an ugly stack trace rather than a usage message. Trivial polish item.
- **Recommendation:** Defer; the binary is run by the maintainer, not a user.

## Observations (no action recommended)

- **Generator code structure is clean.** `FixtureSpec` is well-shaped for adding V9, V10 by appending one struct literal. The `synthetic_xpub` helper makes parent_fingerprint and chain_code deterministic-from-seed, so a hand-debugger can verify any byte. Only one panic via `expect` per fixture call (path parse, encode_bytecode), each with a distinct message — diagnosable.
- **`lowercase_hex` reimplementation justified.** The `bitcoin::hashes::hex` and `hex` crates are both available, but the maintained inline 4-line implementation is correct, bypasses any `hex-conservative` quirks, and removes a dependency-discipline concern. Comment at line 207-208 documents the rationale.
- **Feature-gating discipline is exemplary.** The `gen-vectors` feature pulls `serde_json` into the library only on the binary path; `dev-dependencies` keeps the test harness self-contained without bleeding into the main lib. Cargo.toml comments make this explicit.
- **`build_card_from_input` cross-check (tests/vectors.rs:88-91)** catches a class of fixture-inconsistency bug: if a future contributor declares `network: testnet` but uses an `xpub` (mainnet prefix), the test fails with a clear message rather than producing wrong bytecode.
- **Synthetic xpubs are real secp256k1 points.** `secret_bytes = [seed_byte; 32]` for seed_byte in {0x01..0x08} are all valid (non-zero, non-overflow) secret keys. `SecretKey::from_slice` would reject them otherwise; the `expect` is safe by construction.
- **Per-fixture `chunk_set_id` choice (memorable hex digits 0x12345..0x89012)** is a thoughtful detail — when a vector fails, the maintainer can read the failing chunk's chunk_set_id and immediately know which fixture it was without cross-referencing.
- **Fingerprint omission via `serde_json::Value::Null`** rather than missing-field is the right choice for cross-implementation parity; a Go or Python decoder reading the JSON gets a typed null, not a `KeyError`.
- **FOLLOWUPS item `encode-with-chunk-set-id-singlestring-silent-ignore`** is now moot in practice — Phase 6's amendment to SPEC §2.4 confirms no v0.1 encoder ever hits the SingleString path. Worth noting in FOLLOWUPS but no code action.

## Plan deviations

None observed. The implementation matches plan §6.1.1, §6.1.2, §6.2.1, §6.2.2, §6.3.1, and §6.3.2 task-for-task. Two minor positive deviations:
- The plan suggested a separate `tests/vectors_schema.rs` for the SHA pin; the implementation consolidates into `tests/vectors.rs` (one integration target). This is cleaner.
- The plan listed the "expected" object fields in a specific order; the implementation uses alphabetically-sorted keys, which is the canonicality pin. The plan's order was illustrative.

## Recommended follow-ups

1. **I-2** (stale comment) — fix inline; one-line edit.
2. **I-1** (BIP doc-coherence) — fix inline OR record in FOLLOWUPS as a `pre-bip-submission` item that bundles with `bip-cross-reference-completeness`.
3. **M-1** (dictionary coverage) — add to FOLLOWUPS as `vector-corpus-dictionary-coverage` at `pre-bip-submission` tier.

The work is ready to proceed to Phase 7 (release plumbing) once I-2 is fixed (or recorded). I-1 can wait if the parent agent is bundling pre-BIP-submission audit work.
