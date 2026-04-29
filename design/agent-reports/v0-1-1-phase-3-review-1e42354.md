# Phase 3 review — negative-vector corpus + schema bump (commit 1e42354)

**Status:** DONE_WITH_CONCERNS
**Commit:** 1e42354 (`feat(mk-codec phase 3): negative-vector corpus + schema 2 (decoder-error-variant-parity)`)
**Reviewer / Implementer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**File(s):**
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bin/gen_mk_vectors.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/vectors.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/tests/vectors/v0.1.json`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/error.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/pipeline.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/chunk.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/header.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/bch.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/decode.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/path.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/bytecode/xpub_compact.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/string_layer/chunk.rs`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/consts.rs`
- `/scratch/code/shibboleth/mnemonic-key/design/MILESTONE_v0_1_1.md`
- `/scratch/code/shibboleth/mnemonic-key/design/agent-reports/v0-1-1-milestone-review-draft.md`
**Role:** reviewer (code)

## Summary

Phase 3 is in a strong commit-ready state. The schema-2 bump landed cleanly with `expected_error: null` consistently emitted on clean vectors (byte-determinism preserved — re-running the generator into `/tmp/mk-regen.json` produces a SHA-256-identical artifact); 22 negative vectors map every reachable `Error` variant to a triggering input that the harness asserts byte-equal against `Display`; all 156 tests pass. Two important documentation/coverage items deserve attention before tagging v0.1.1, and a handful of minor nits are recorded inline. **No critical issues.**

## Critical

(none)

## Important

### I-1. `every_error_variant_has_negative_vector` is a runtime substring gate, not the compile-time exhaustive match the plan called for

**File:** `crates/mk-codec/tests/vectors.rs:255-312`
**Plan §3.3.2:** "write the test as an in-crate exhaustive `match` over a constructed `Error` value of each variant (one arm per variant). `#[non_exhaustive]` blocks **external** exhaustive matching, but the test lives inside `mk-codec` so the compiler still warns when a new variant is added without an arm."

The implementation comment at `tests/vectors.rs:275-283` notes the reason for the deviation: rustc treats integration-test targets as external-to-crate for `#[non_exhaustive]` purposes, even when those tests live in the same package. That is correct (verifiable: `Cargo.toml`'s `[[test]]` target compiles to a separate crate that links the library). So a literal compile-time exhaustive match in `tests/vectors.rs` cannot enforce variant coverage even with the variant constructors in scope.

The runtime substring fallback (`assert_variant_covered("invalid HRP")`, etc.) gives equivalent behaviour at the *runtime* level: if a new variant lands without a corresponding `expected_error`-prefix vector, the test fails when run. But — and this is the real gap — if a new variant lands and the *test author* forgets to add a new `assert_variant_covered` line, the new variant is silently uncovered. The compile-time-match property the plan wanted (compiler warns on missing arm) is gone, replaced by an "author remembered to add a line" property.

The implementation's referenced fallback ("the unit tests in `error.rs` exhaustively construct each variant and would surface a missed-rendering case") is **not** in fact exhaustive — `parameterized_variants_render` and `static_variants_render` are hand-curated lists that have the same "author remembered" hazard. Adding a new variant without updating either test set produces a runtime drift, not a compile error.

**Recommendation (one of):**

1. **Move the exhaustiveness gate into a unit-test module under `crates/mk-codec/src/`** (e.g., `crates/mk-codec/src/error.rs::tests::every_variant_has_render_case` driven by a literal `match` over a per-variant constructed value, returning a tuple `(variant, expected_display_prefix)` for every variant — which then the integration test can iterate). The unit-test module compiles inside the crate and gets the `#[non_exhaustive]` waiver.
2. **Add the `strum` dev-dep** the plan briefly considered. `strum::EnumIter` plus a `#[derive(strum_macros::EnumDiscriminants)]` lets the test enumerate variants at runtime with no manual list to maintain. The crate-tree cost is small and isolated to dev-dependencies.
3. **Document the deviation as accepted** and move on. The current runtime gate plus the existing `parameterized_variants_render` / `static_variants_render` unit tests do catch new-variant rendering drift in practice — just not at compile time. If the team accepts this tradeoff explicitly (e.g., as a follow-up tracked in `FOLLOWUPS.md` under a `v0.2-nice-to-have` tier), this is a reasonable terminal state.

The current code already has a comment referencing option 2 as a "v0.2 nice-to-have" (line 282-283), but no FOLLOWUPS entry was added. **Either add an explicit FOLLOWUPS entry capturing the v0.2 strum-driven gate, or escalate the gate before v0.1.1 ships.**

### I-2. N17's `expected_error` mismatch with the milestone N17 row is a real documentation drift

**File:** `crates/mk-codec/src/bin/gen_mk_vectors.rs:813-837`, `crates/mk-codec/tests/vectors/v0.1.json:602-612`, `design/MILESTONE_v0_1_1.md:307`

The milestone plan's N17 row says: "explicit path with truncated LEB128" → `Error::InvalidPathComponent`. The implementation produces an input that surfaces as `Error::UnexpectedEnd` (correctly — see analysis below), with `expected_error: "unexpected end of bytecode"` pinned in the vector. The `every_error_variant_has_negative_vector` gate then exempts `InvalidPathComponent` with the comment "covered by N17 surfacing as UnexpectedEnd; reachable in principle but brittle to construct" (`tests/vectors.rs:302-306`).

**Tracing the actual decoder behaviour** (`crates/mk-codec/src/bytecode/path.rs:104-159`):

Given the N17 bytecode `[0x00, 0x01, 0xCAFEBABE, 0xFE, 0x01, 0x80, ...xpub_tail(73)]`:
- `decode_path` reads `0xFE` (explicit-path) then `count = 0x01` (passes the `count <= MAX_PATH_COMPONENTS` cap)
- `leb128_decode_u32` reads `0x80` (continuation bit set), then reads the *next* byte from cursor — which is the first byte of the xpub_tail. That byte is `0x04` (xpub_compact's version-byte high nibble). 0x04 has no continuation bit, so LEB128 finishes cleanly with `result = 0x04 << 7 = 512`. `decode_path` returns successfully with a 1-component path (component value 512).
- `decode_xpub_compact` then reads what should be 73 bytes of xpub. But one byte was consumed by LEB128, so only 72 remain → `UnexpectedEnd` from `read_array`.

So the input is *not actually triggering* an `InvalidPathComponent` path. To trigger `InvalidPathComponent`, the input would need to either:
- (a) Push `0x80` 6 times (LEB128 overflow at shift=35 → `Error::InvalidPathComponent("LEB128 overflow at shift 35")`), or
- (b) Push e.g. `0xFF, 0xFF, 0xFF, 0xFF, 0x10` (LEB128 result > u32::MAX → `Error::InvalidPathComponent("LEB128 value ... > u32::MAX")`).

Either is a one-line tweak in `n17_invalid_path_component`.

**Recommendation:** Reshape N17 to actually trigger `InvalidPathComponent` (option a is simplest: 6 × `0x80` produces a clean LEB128 overflow rejection). This eliminates the `InvalidPathComponent` exemption from the exhaustiveness gate, brings the corpus into sync with the milestone N17 row, and — importantly — adds genuine `InvalidPathComponent` coverage where today the corpus has none. The current code's claim "reachable in principle but brittle to construct" is too modest; a 6-byte-of-0x80 LEB128 overflow is a routine path-component rejection trigger with no fragility issue.

The current state is not wrong — the harness asserts byte-exact `Display` and the assertion passes — but it leaves an `Error` variant uncovered while documenting that fact awkwardly. Folding the fix in pre-tag is cheap and improves the contract.

## Minor

### m-1. Vector's `every_vector_round_trips` count assertion uses `>=` floors but pinning would catch silent drift sooner

**File:** `crates/mk-codec/tests/vectors.rs:154-155`

```rust
assert!(clean_count >= 17, "clean-vector count regressed");
assert!(negative_count >= 22, "negative-vector count regressed");
```

`>=` floors catch deletions but not additions, and the comment "Phase 4 will tighten these to floor checks if v0.1.x adds vectors" is backwards: the current `>=` *is* the floor check. If the comment intent is "Phase 4 will tighten these to exact-equals once v0.1.1 ships" — that wording would more accurately reflect an SHA-256-equivalent invariant. This is a minor wording fix in the comment.

### m-2. N17's `why` field accurately documents the surfacing as UnexpectedEnd, but the vector's `name` (`N17_invalid_path_component_truncated_leb128`) implies the variant it doesn't surface

**File:** `crates/mk-codec/src/bin/gen_mk_vectors.rs:825`

If I-2 is folded, the name becomes accurate (the LEB128-overflow form does surface `InvalidPathComponent`). If I-2 is deferred, consider renaming to e.g. `N17_truncated_leb128_surfaces_as_unexpected_end` so the vector's name doesn't promise an `Error` variant the harness's pinned `expected_error` doesn't match.

### m-3. Several `vec![baseline_valid_bytecode()[baseline_valid_bytecode().len() - 73..]]` calls trigger the encoder twice

**File:** `crates/mk-codec/src/bin/gen_mk_vectors.rs:740, 757, 776, 800, 823`

Examples (all in the bytecode-layer N* helpers):

```rust
rebuilt.extend_from_slice(&bytecode[bytecode.len() - 73..]); // OK — single bytecode local
bytecode.extend_from_slice(&baseline_valid_bytecode()[baseline_valid_bytecode().len() - 73..]);
```

The second form calls `baseline_valid_bytecode()` (non-trivial — encodes a full KeyCard) twice per slice operation. This isn't a correctness bug — the function is deterministic — but it doubles the generator-binary work for the affected negative vectors (N14/N15/N16/N17/N20). Trivial fix:

```rust
let baseline = baseline_valid_bytecode();
let xpub_tail = &baseline[baseline.len() - 73..];
bytecode.extend_from_slice(xpub_tail);
```

Pure perf nit; generator runtime is dominated by xpub derivation anyway, so the user-facing impact is negligible. Mention because the pattern is easy to grep-and-fix.

### m-4. Generator binary's `expect("...")` calls assume valid bitcoin / mk-codec helpers — fine for an internal generator but worth a one-line module comment

**File:** `crates/mk-codec/src/bin/gen_mk_vectors.rs:393, 401, 416, 423-431, 446-447`

The generator's `unwrap()` / `expect(...)` calls all sit on infallible-by-construction inputs (e.g., `DerivationPath::from_str("84'/0'/0'")` is hard-coded valid; `KeyCard::new(...)` accepts arbitrary fields). This is appropriate for a binary that's only run by maintainers as a vector-regen step, and panics give a clear stack on misuse. A one-line module-level comment ("This binary panics on construction errors by design — every `expect()` is on a hard-coded-valid input.") would document the convention for future readers.

### m-5. Schema-2 backward-compat: harness's `schema_metadata_pinned` test pins `schema == 2`, which means a third party validating against a v0.1.0 schema-1 corpus would need a separate harness, not just an SHA mismatch

**File:** `crates/mk-codec/tests/vectors.rs:108-122`

This is consistent with the milestone scope (the corpus drift between v0.1.0 and v0.1.1 *is* the migration trigger), but the in-line comment at `tests/vectors.rs:111-114` could be clarified: "schema 1 (v0.1.0) corpora are still readable by this harness" is technically true for an in-memory parse (the JSON loads), but `schema_metadata_pinned` would *fail* on a v0.1.0 corpus because the harness pins `== 2`. The forward-compat property is "schema-2 readers parse schema-1 cleanly *if* they relax the schema pin", which is a different statement.

Concrete suggestion: split the comment into two clauses — one for the in-memory parse compatibility and one for the test-time pin. Or remove the line if v0.1.1 ships as a hard schema bump that obsoletes the v0.1.0 corpus, which is the de-facto behaviour given the SHA pin already differs.

### m-6. `synthetic_singlestring` and `wrap_bytecode_in_mk1` overlap

**File:** `crates/mk-codec/src/bin/gen_mk_vectors.rs:385-417`

`synthetic_singlestring` is `wrap_bytecode_in_mk1` specialised to the `≤ SINGLE_STRING_LONG_BYTES` branch. Both are used (N6/N7/N10 use `synthetic_singlestring` directly with hand-crafted bytes; bytecode-layer Ns use `wrap_bytecode_in_mk1`). Since `wrap_bytecode_in_mk1` already auto-selects single-vs-chunked based on length, callers of `synthetic_singlestring` could just use `wrap_bytecode_in_mk1` and drop the duplicate helper.

The keep-both rationale is presumably "make N6/N7/N10's intent (we want SingleString here) explicit at the call site." That's fair. Either way is fine; flagging it because if `wrap_bytecode_in_mk1`'s signature ever changes, both functions need to be updated.

### m-7. `InvalidPathIndicator` rendering should use lowercase hex per `Display` impl, and N15 pins `0x00` (which is unambiguous) — but a future N15-style vector for, e.g., `0xAB` should pin `0xab` to match `format!("{}", err)`

**File:** `crates/mk-codec/src/error.rs:117-118`

The `Display` impl uses `{:02x}` (lowercase) — N15's `expected_error: "invalid path indicator byte: 0x00"` is correct. Reminder for future vector authors that Rust's `:02x` is lowercase. (Same applies to `InvalidXpubVersion` at `error.rs:131` — N18 correctly pins `0xdeadbeef` lowercase.)

## Observations / confirmations

### O-1. Byte-determinism verified end-to-end

I re-ran `cargo run --bin gen_mk_vectors --features gen-vectors --quiet -- --output /tmp/mk-regen.json` and `sha256sum`'d both files:

```
77e9eba529cf086734be80a3c7a02aa2cf0fcc2c2f752d667d98791cd7ed9069  crates/mk-codec/tests/vectors/v0.1.json
77e9eba529cf086734be80a3c7a02aa2cf0fcc2c2f752d667d98791cd7ed9069  /tmp/mk-regen.json
```

`cmp -s` confirms byte-identical. Pinned SHA in `tests/vectors.rs:41` matches both. The `expected_error: null` consistency on clean vectors is the load-bearing property here, and inspection of `crates/mk-codec/tests/vectors/v0.1.json:16, 40, 90, …` confirms it's emitted on every clean vector.

### O-2. All Ns trigger their pinned `expected_error` exactly

I traced each of the questioned vectors:

- **N5** (`gen_mk_vectors.rs:527-553`): perturbs chars 11..16 of chunk[0] of a 1-stub fp-omit baseline. With `mk1` (3 chars) + 8-symbol chunked header (chars 3..10), positions 11..16 land in chunk[0]'s data payload. chunk[0]'s data part is 8 (header) + 84.8 (payload + cross-chunk hash, 5-bit-symbol-encoded) = ~93 symbols → BCH(108,93,8) long code. The pinned `expected_error: "BCH uncorrectable: long code: more than 4 substitutions or pathological pattern"` matches `bch.rs:495-496` byte-for-byte (`format!("BCH uncorrectable: {}", "long code: ...")` from `Error::BchUncorrectable`'s Display).

- **N9** (`gen_mk_vectors.rs:619-651`): generator builds the 8-symbol chunked header with `total_chunks_wire = 1` (= total_chunks=2 on parse) and `chunk_index = 2`. `header.rs:154-162` parses these and the `chunk_index >= total_chunks` arm at line 159-163 fires with `format!("chunk_index = {chunk_index} >= total_chunks = {total_chunks}")` = `"chunk_index = 2 >= total_chunks = 2"`. Pinned `expected_error` matches.

- **N15** (`gen_mk_vectors.rs:769-788`): bytecode `[0x00, 0x01, 0xCA, 0xFE, 0xBA, 0xBE, 0x00, ...xpub_tail(73)]`. With header byte `0x00` (no fp), the decoder reads `header(1) → stub_count(1) → 1 stub(4) → indicator(1)` — indicator at offset 6, value `0x00`. `BytecodeHeader::parse(0x00)` succeeds (no flags); decode_path then reads `0x00`, hits the std-table reserved-range check, returns `Error::InvalidPathIndicator(0x00)` rendering as `"invalid path indicator byte: 0x00"`. Pinned `expected_error` matches.

- **N19** (`gen_mk_vectors.rs:857-872`): replaces final 33 bytes (xpub.public_key) with all-zeros. `xpub_compact.rs:96-97` calls `PublicKey::from_slice(&[0u8; 33])` which `secp256k1` rejects with `"malformed public key"`, wrapped as `Error::InvalidXpubPublicKey("malformed public key")` rendering as `"invalid xpub public key: malformed public key"`. Pinned `expected_error` matches (the harness's byte-equal assertion confirms — if secp256k1's error message ever changes, this is the canary).

- **N23** (`gen_mk_vectors.rs:909-922`): empty `input_strings`. `pipeline.rs:118-122` at `decode(&[])` returns `Err(ChunkedHeaderMalformed("empty input string list"))` rendering as `"chunked-header malformed: empty input string list"`. Pinned `expected_error` matches.

### O-3. CardPayloadTooLarge is genuinely encoder-only

Single emit site at `crates/mk-codec/src/string_layer/chunk.rs:60` inside `split_into_chunks`. The decoder side has no analogous size check: the maximum a conforming chunked input can carry is `MAX_CHUNKS=32 × CHUNKED_FRAGMENT_LONG_BYTES=53 = 1696` bytes of stream, of which the last 4 are the cross-chunk hash, leaving 1692 bytes of bytecode — exactly at the encoder's `MAX_CHUNKABLE_BYTECODE` ceiling. No way to reach 1693+ bytes via `decode`'s string input. The exhaustiveness gate's exemption (`tests/vectors.rs:248-253, 311`) is therefore correct.

### O-4. `clean_fixture!` macro is faithful to the v0.1.0 / Phase-2 hand-typed `FixtureSpec` literals

I diff'd the macro expansion (mental + by inspection of `2417401:gen_mk_vectors.rs`) against post-Phase-3 V1..V17 entries: every field maps 1:1 from the macro input to the `FixtureKind::Clean(CleanInput { ... })` shape. The byte-determinism check (O-1) is the strongest empirical proof: V1..V17 emit byte-identical content to v0.1.0 / Phase-2, so the macro can't be subtly mis-rendering field positions.

### O-5. All 22 negative vectors are structurally distinct

By inspection of `gen_mk_vectors.rs:454-922`: each `n*_*()` constructor produces a unique input byte sequence (different base bytecodes, different mutations, different positions). The `expected_error` strings are also distinct (no two negative vectors share the same pinned error rendering).

### O-6. No negative vector accidentally passes

The harness at `tests/vectors.rs:227-238` panics if `decode` returns `Ok(_)` for a negative vector. The Phase 3 commit's test-suite pass (149+3+4=156, 0 ignored beyond pre-existing) confirms no false positives.

### O-7. Vector ID sequence N1..N21, N23 confirmed (N22 intentionally absent)

`grep -oE '"name": "N[0-9]+_' v0.1.json | sort -V | uniq` returned exactly N1..N21 + N23 = 22 entries. N22 (CardPayloadTooLarge) is the documented exemption.

### O-8. SHA pin in `V0_1_SHA256` matches on-disk file

`77e9eba529cf086734be80a3c7a02aa2cf0fcc2c2f752d667d98791cd7ed9069` matches both the on-disk file and the regenerated `/tmp/mk-regen.json` (O-1).

## Verification commands run

```bash
cd /scratch/code/shibboleth/mnemonic-key
cargo test -p mk-codec --quiet
# 149 + 3 + 4 = 156 tests pass; 0 ignored

cargo run --bin gen_mk_vectors --features gen-vectors --quiet -- --output /tmp/mk-regen.json
sha256sum crates/mk-codec/tests/vectors/v0.1.json /tmp/mk-regen.json
cmp -s crates/mk-codec/tests/vectors/v0.1.json /tmp/mk-regen.json && echo "byte-deterministic OK"
# Both SHAs equal the V0_1_SHA256 pin; cmp succeeds.
```

## Recommendation

**Commit-with-fixups.** Phase 3 is in good shape; ship the fixups inline before tagging v0.1.1.

Recommended fixup commit (`style/fix(mk-codec phase 3): apply Phase 3 review fixes (commit 1e42354 review)`):

1. **I-2 (important):** Reshape `n17_invalid_path_component` to push `0x80` six times instead of one truncated `0x80` — surfaces `Error::InvalidPathComponent("LEB128 overflow at shift 35")`. Update `expected_error`, `description`, `name` (drop `_truncated_leb128` if appropriate), and remove the `InvalidPathComponent` exemption from `every_error_variant_has_negative_vector`. Re-pin SHA. Net: corpus gains real `InvalidPathComponent` coverage.

2. **I-1 (important, pick one):** Either (a) move the exhaustiveness gate to a unit-test module under `crates/mk-codec/src/error.rs` driven by an explicit per-variant constructor list to get compile-time enforcement, OR (b) add a FOLLOWUPS entry under `v0.2-nice-to-have` capturing the strum-EnumIter migration, so the deviation is tracked.

3. **m-1, m-5 (wording):** Tighten the comments at `tests/vectors.rs:153, 113-114`.

4. **m-3 (perf nit):** Hoist the `baseline_valid_bytecode()` calls in N14/N15/N16/N17/N20 into a single local. Preserves SHA (pure refactor).

m-2/m-4/m-6/m-7 are observations and can be deferred to FOLLOWUPS or skipped — none affect the corpus's correctness or the harness's enforceability.

Once those land, **proceed to Phase 4 (release plumbing)** without further design pause: the milestone scope is intact, byte-determinism holds, every reachable `Error` variant is genuinely covered (after I-2), and the harness's enforcement story is credible.
