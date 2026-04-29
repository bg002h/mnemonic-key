# Review — Phase 2 (vector corpus dictionary expansion), commit `2417401`

**Status:** DONE
**Commit:** `2417401` (`feat(mk-codec phase 2): expand vector corpus to 17 entries`)
**Reviewer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**Files:**
- `crates/mk-codec/src/bin/gen_mk_vectors.rs`
- `crates/mk-codec/tests/vectors/v0.1.json`
- `crates/mk-codec/tests/vectors.rs`
- `crates/mk-codec/src/bytecode/path.rs` (read for cross-check; not modified)
- `design/MILESTONE_v0_1_1.md` (read for plan-vs-impl alignment)
**Role:** reviewer (code)

## Summary

Phase 2 of v0.1.1 lands cleanly. All 9 new fixtures (V9..V17) are well-formed; every path indicator
matches the milestone-prescribed value; mainnet/testnet tagging is consistent across `network` /
`xpub.version` / xpub-prefix; fingerprint-bit alternation matches the plan; canonicality discipline
(alphabetical keys, lowercase hex, 2-space indent, LF EOF, byte-determinism) holds; SHA pin matches
the on-disk corpus byte-for-byte; the full test suite (149 unit + 3 round_trip + 3 vectors = 155)
passes; and `0x16` is correctly absent. Coverage tally is exactly the claimed 13-of-14 std-table
entries plus 0xFE explicit-path. **No critical or important findings.** Three minor / observation
items below.

## Verification done

1. **Path-indicator hand-decode (V9..V17).** Decoded each `canonical_bytecode_hex` byte-by-byte
   following layout `[flags(1)][stub_count(1)][stubs(4n)][fp?(4)][indicator(1)][xpub(73)]`. All 9
   indicators match the milestone table at `MILESTONE_v0_1_1.md:185-194`:

   | Vector | bytecode prefix (first 11 bytes hex)            | offset of indicator | indicator | expected |
   |--------|-------------------------------------------------|---------------------|-----------|----------|
   | V9     | `04 01 44 44 44 44 c0 01 ca fe 01`              | 10 (fp=Y, 1 stub)   | `0x01`    | `0x01` ✓ |
   | V10    | `04 01 49 49 49 49 fe ed be ef 02`              | 10                  | `0x02`    | `0x02` ✓ |
   | V11    | `04 01 86 86 86 86 86 40 70 05 04`              | 10                  | `0x04`    | `0x04` ✓ |
   | V12    | `00 01 48 48 00 01 06`                          | 6 (fp=N, 1 stub)    | `0x06`    | `0x06` ✓ |
   | V13    | `04 01 44 11 00 00 44 11 aa bb 11`              | 10                  | `0x11`    | `0x11` ✓ |
   | V14    | `04 01 49 12 00 00 49 12 cc dd 12`              | 10                  | `0x12`    | `0x12` ✓ |
   | V15    | `04 01 84 13 00 00 84 13 ee ff 13`              | 10                  | `0x13`    | `0x13` ✓ |
   | V16    | `00 01 86 14 00 00 14`                          | 6                   | `0x14`    | `0x14` ✓ |
   | V17    | `00 01 87 17 00 00 17`                          | 6                   | `0x17`    | `0x17` ✓ |

2. **Network / xpub-version / xpub-prefix consistency.** V9..V12 mainnet → xpub strings begin
   `xpub` and bytecode encodes `0488b21e` immediately after the path indicator; V13..V17 testnet
   → `tpub` + `043587cf`. Per BIP 32 these prefixes correspond to `0x0488B21E` (mainnet) and
   `0x043587CF` (testnet) — confirmed. The harness's structural `xpub.network` vs declared
   `network` cross-check (`tests/vectors.rs:88-91`) further enforces this at runtime.

3. **Fingerprint-bit alternation matches plan §2.1.1.** Header byte `0x04` (bit 2 = fp present)
   on V9, V10, V11, V13, V14, V15; header byte `0x00` (bit 2 cleared = fp omitted) on V12, V16,
   V17. Identical to the milestone's recommended split.

4. **chunk_set_id discipline.** Every value ≤ `0xFFFFF` (20-bit fit confirmed numerically); all 17
   values distinct; no collision with V1..V8. The new values are V9 `0x9A012`, V10 `0xAB123`,
   V11 `0xBC234`, V12 `0xCD345`, V13 `0xDE456`, V14 `0xEF567`, V15 `0xF0678`, V16 `0x01789`,
   V17 `0x1289A`.

5. **Seed-byte uniqueness.** V9..V17 use 0x09..0x11 (decimal 9..17), exactly continuing V1..V8's
   0x01..0x08 sequence. None collide with prior values. All values ≪ secp256k1 group order, so
   `SecretKey::from_slice(&[seed_byte; 32])` succeeds — confirmed by the `expect()` round-tripping
   under test.

6. **Generator determinism.** `cargo run -p mk-codec --features gen-vectors --bin gen_mk_vectors -- --output /tmp/mk-vectors-regen.json`
   produces a 19381-byte file; `cmp -s` against the on-disk corpus reports byte-identity. The
   generator preserves alphabetical key ordering at every nesting depth (verified via Python walk;
   no out-of-order keys), all hex strings lowercase (verified via `.lower()` round-trip), 2-space
   indent (line 3 = `  "schema": 1,`), LF-only line endings, trailing LF at EOF.

7. **SHA pin match.** `sha256sum tests/vectors/v0.1.json` → `6a2667c21e80060844e69de8114652810d883b3a017b232524fe749af30d1106`,
   exactly the constant pinned at `tests/vectors.rs:41`.

8. **Test suite green.** `cargo test -p mk-codec` runs 149 unit + 3 round_trip + 3 vectors = 155
   tests, 0 failures, 0 ignored beyond the existing un-counted ones (`#[ignore]` scaffolds were
   already retired). `every_vector_round_trips` iterates 17 entries; spot-checked V9 and V12
   bytecode regenerates byte-exact via `encode_bytecode`, and the corresponding mk1 string set
   regenerates byte-exact via `encode_with_chunk_set_id`.

9. **Coverage gap closure.** Std-table reference at `crates/mk-codec/src/bytecode/path.rs:29-45`
   lists 13 entries: 7 mainnet (`0x01`..=`0x07`) + 6 testnet (`0x11`..=`0x15`, `0x17`). After
   Phase 2 every entry is exercised exactly once, plus 0xFE (V5, V7) is exercised twice. **0x16
   absent** as deferred. Tally = 13/14 std-table entries — matches the commit-message claim.

10. **0x16 deliberate exclusion.** Confirmed: no vector encodes 0x16. The
    `gen_mk_vectors.rs:168-175` block-comment cites the cross-repo `md-path-dictionary-0x16-gap`
    deferral; commit message also calls it out explicitly. `path.rs:9-10` already pins the gap at
    the source-of-truth and an existing unit test (`rejects_reserved_indicator_0x16` at
    `path.rs:261-269`) gates the rejection path. Decoder consistency holds.

11. **Schema unchanged.** `tests/vectors/v0.1.json:2` still emits `"schema": 1`; the
    `schema_metadata_pinned` test continues to gate against `1u64`. No new fields appear in V9..V17
    entries (each entry has the same `{name, description, input{...}, expected{...}}` shape as
    V1..V8). Phase 3 will be the schema bump.

12. **Toolchain gates.** `cargo clippy` and `cargo fmt --check` are blocked by the known nightly-
    toolchain shim issue (CLAUDE.md notes this is acceptable to skip; rustup's nightly default
    can't find clippy/rustfmt and the system clippy at `/usr/bin` isn't picked up by the shim).
    Source code visually matches the existing crate's idiomatic style (no obvious format drift in
    the diff).

## Critical

_None._

## Important

_None._

## Minor

### m-1. V16 `chunk_set_id = 0x01789` reads as 5-digit hex but its high nibble is 0

**Where:** `gen_mk_vectors.rs:264` — V16 fixture, `chunk_set_id: 0x01789`.

The plan's Step 2.1.1 prescribes "distinct memorable hex digits"; V9..V15 and V17 use a
top-nibble-non-zero value (e.g., V9 `0x9A012`, V13 `0xDE456`). V16 is an outlier at `0x01789` —
five hex digits when written but functionally a 4-digit value. Not wrong (still ≤ 20 bits, still
distinct), just stylistically inconsistent with the surrounding fixture set. Possibly a typo for
`0x01789` → `0x10789` (5-digit, mnemonic-friendly). Cosmetic only; does not affect determinism,
tests, or wire output. **No action required**; flagging for awareness if a v0.1.2 corpus refresh
ever wants the values to read identically wide.

### m-2. V17 `chunk_set_id = 0x1289A` collides numerically with V8 `0x89012` byte-rotated

**Where:** `gen_mk_vectors.rs:277` and `:166`.

V8 = `0x89012`, V17 = `0x1289A`. They are not equal (no actual collision; uniqueness verified
above), but they share the same 5 hex digits in different orders. If a future debugging session
hand-confuses the two, the harness will surface a clear mismatch via the `[name]`-prefixed
assertion message in `tests/vectors.rs`, so the practical risk is near-zero. **No action required.**

### m-3. `IMPLEMENTATION_PLAN_mk_v0_1.md:1043,1078` still references "8 vectors"

**Where:** lines `1043` ("Verify: 8 vectors cover the diagonal") and `1078` ("Initial vector
corpus (8 vectors) anchored under family token mk-codec 0.1") inside the v0.1.0 implementation
plan.

These describe what shipped at v0.1.0 (the file is the v0.1.0 closed-phase plan, not a live spec),
so the count is historically accurate. **No action required**; flagging only because the prompt
asked me to look for stale "8 vectors" references and these turned up in the grep. The v0.1.1
milestone plan (`MILESTONE_v0_1_1.md`) correctly references "expanded from 8 to 17."

## Observations

### O-1. Description text matches encoded BIP for every V9..V17 vector

Each fixture's `description` cites the BIP family + path string + indicator value; I cross-checked
all 9 and the (BIP, path, indicator) triple matches the encoded bytecode exactly. No description
drift. Particularly checked V11 (`BIP 86 taproot`, `m/86'/0'/0'`, `0x04`) and V14 (`BIP 49
nested-segwit`, `m/49'/1'/0'`, `0x12`) which the prompt flagged as potential mis-tag risks —
both are correct.

### O-2. Generator's V9..V17 block-comment doc is good

`gen_mk_vectors.rs:168-175` documents the rationale for the new fixtures (close coverage,
0x16 deferral, fp alternation per plan) inline at the insertion point. This is exactly the kind
of forward-readable comment that makes the corpus self-describing for a future maintainer who
asks "why these 9 specifically?" without having to dig up the milestone document.

### O-3. SPEC §3.5 dictionary is internally consistent with the corpus

The path table at `path.rs:29-45` lists all 14 std-table entries in canonical order; the corpus
exercises 13 of them, with the missing one (`0x16`) being the deferred entry. Cross-implementations
that pin against this corpus get full path-dictionary coverage minus the deliberately-deferred
testnet nested-segwit gap. This is the right semantics for v0.1.1 — encoders can't legitimately
emit 0x16 anyway (per `path.rs:9-10` it's reserved), so the corpus accurately reflects the wire
constraint.

### O-4. `every_vector_round_trips` test scales with fixture count

The harness at `tests/vectors.rs:122-197` is parameterized purely over the JSON's `vectors[]`
array; no constants embed "8" or "17" anywhere. Phase 2's expansion is genuinely additive — no
harness modification was needed beyond updating the SHA pin. This is a clean separation of
"corpus content" from "corpus harness logic" and bodes well for Phase 3's negative-vector
schema extension (which will need additional harness branches but won't have to thread a count).

### O-5. Synthetic xpub field-population strategy unchanged from v0.1.0

`synthetic_xpub` at `gen_mk_vectors.rs:290-313` produces xpubs whose `parent_fingerprint` and
`chain_code` are derived deterministically from `seed_byte` (specifically `[0x10,0x20,0x30,seed_byte]`
and `[seed_byte ^ 0xAA; 32]`). For V9..V17 with seed_byte `0x09..0x11`, this gives 9 distinct
synthetic xpubs (verified implicitly by the differing bytecode-suffix-after-indicator bytes
across the new fixtures). No real-world chain-of-trust is implied; this is correct per the
function's existing rustdoc.

## Recommended action

**Proceed to Phase 3.** Phase 2 is structurally sound, byte-deterministic, plan-aligned, and fully
test-covered. None of the minor findings warrant a fix-up commit; they are stylistic / historical
notes that don't affect the corpus's contract with cross-implementations or the SHA pin's
stability. The 17-vector v0.1.1 corpus is ready to anchor Phase 3's schema-2 extension.

(End of review.)
