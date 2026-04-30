# mk-codec v0.2.0 Phases 2-4 — review of commit `fd6a407`

**Reviewer:** Senior Code Reviewer (Opus)
**Branch:** `feature/v0.2.0`
**Commit under review:** `fd6a407` — "feat(mk-codec v0.2.0): wire-additive 0x16 path indicator + bit-3 footnote + release"
**Phase 1 review predecessor:** `design/agent-reports/v0-2-0-phase-1-review-901596a.md`
**Date:** 2026-04-29
**Conclusion (top-line):** **Do not tag yet.** Two CRITICAL stale-text bugs in the BIP draft and the SPEC missed the v0.2.0 update — the path-dictionary section in both documents still says 0x16 is "reserved pending an md1 dictionary update" and that mk1 has only 13 entries. These contradict the wire change shipped in this same commit. Fix in a `style/fix(mk-codec phase 2-4): apply review fixes` follow-up before tagging. Otherwise the work is excellent: V18 hand-decode is byte-exact, all 157 tests pass, generator is byte-deterministic, family-token roll is correctly threaded through three places, and the bit-3 footnote text is sharp.

---

## 1. Summary of changes

Phases 2-4 landed together as a single feature commit (small enough scope that per-phase commits would be over-granular). The commit:

- adds `(0x16, "m/48'/1'/0'/1'")` to `STANDARD_PATHS` (path.rs:49)
- updates path.rs module rustdoc to describe the v0.2.0 addition (path.rs:7-14)
- regenerates the corpus to 18 clean + 22 negative = 40 vectors with a new V18 fixture (gen_mk_vectors.rs:345-360, vectors/v0.1.json)
- rolls `GENERATOR_FAMILY` `"mk-codec 0.1"` → `"mk-codec 0.2"` (consts.rs:50)
- updates the SHA-256 pin in `tests/vectors.rs::V0_1_SHA256` (vectors.rs:41) and family-token pin in `schema_metadata_pinned` (vectors.rs:119)
- bumps clean-vector floor from `>= 17` to `>= 18` (vectors.rs:155)
- reshapes path.rs's `rejects_reserved_indicator_0x16` test → `round_trip_indicator_0x16_added_in_v0_2` (path.rs:267)
- switches decode.rs's `rejects_invalid_path_indicator` from 0x16 → 0x18 (decode.rs:181)
- adds a "shape-shared / bits diverge from bit 3 onward" footnote to BIP §"Bytecode header" and SPEC §3.1
- bumps Cargo.toml 0.1.1 → 0.2.0
- adds CHANGELOG `[0.2.0]` section
- flips FOLLOWUPS `md-path-dictionary-0x16-gap` to `resolved by md-codec-v0.9.0 + mk-codec-v0.2.0`

All 157 tests (149 unit + 2 error_coverage + 3 round_trip + 3 vectors) pass on `feature/v0.2.0` at HEAD.

---

## 2. Findings

### Critical

#### C-1. BIP §"Origin path encoding" still claims 0x16 is reserved-pending

`bip/bip-mnemonic-key.mediawiki` lines 279, 281:

```
Indicators 0x11 through 0x15 and 0x17 name testnet variants of the mainnet
entries other than 0x06. The exact dictionary mirrors MD's Tag::SharedPath
table byte-for-byte.

Note on indicator 0x16: the mainnet entry 0x06 (BIP 48 nested-segwit
multisig, m/48'/0'/0'/1') currently has no testnet companion in md1's
published path dictionary. mk1 inherits this gap by the mirror clause;
0x16 is reserved pending an md1 dictionary update (tracked as
md-path-dictionary-0x16-gap in the cross-repo follow-up tracker). Encoders
MUST NOT emit 0x16 in v0.1; decoders MUST reject it via the same
Error::InvalidPathIndicator path as any other reserved indicator. When md1
adds the row, mk1 inherits the allocation by the mirror clause without a
wire-format change.
```

This text — left over from v0.1 — directly contradicts the wire-format change shipped in this same commit. Three problems:

1. The list of testnet indicators in the prose says `0x11 through 0x15 and 0x17` — should be `0x11 through 0x17`.
2. The "Note on indicator 0x16" paragraph still says 0x16 is reserved-pending, when in fact this commit adds it to the dictionary.
3. The closing sentence — "When md1 adds the row, mk1 inherits the allocation by the mirror clause without a wire-format change" — turned out to be wrong: md1 added the row, mk1 inherited it, AND mk1 v0.2.0 IS a (wire-additive) wire-format change. Ironic given the careful CHANGELOG framing of the asymmetry.

This is the public-facing BIP draft; shipping v0.2.0 with this text means anyone reading the BIP will see contradictory claims (Phase 3's bit-3 footnote at line 232 cites "as of mk-codec v0.2.0", but line 281 says encoders MUST NOT emit 0x16 in v0.1 and treats the gap as still-open).

**Recommended fix:**

- Update the testnet-indicator prose in §"Case A — standard-table indicator" to list 0x11 through 0x17 (no exceptions).
- Replace the "Note on indicator 0x16" paragraph with a single sentence noting that 0x16 was added to the dictionary in mk-codec v0.2.0 (after md-codec v0.9.0 closed the parallel gap on the md1 side), and that v0.1.x decoders reject v0.2.0-emitted 0x16 strings via the wire-additive asymmetry. Or just delete the note; the dictionary table speaks for itself.

(This is a doc-only fix; no code change needed and the wire-format change is correctly shipped.)

#### C-2. SPEC §3.5 has the same stale text

`design/SPEC_mk_v0_1.md` line 217 and 221:

```
| 0x11–0x15, 0x17 | Testnet variants (no 0x16 row — see note below) |
…
**Note on 0x16.** md1's published path dictionary has no testnet pair
for the mainnet 0x06 entry (BIP 48 nested-segwit multisig). mk1 inherits
the gap; 0x16 is reserved pending an md1 dictionary update (tracked as
md-path-dictionary-0x16-gap in FOLLOWUPS.md). Encoders MUST NOT emit
0x16 in v0.1; decoders MUST reject it via the same InvalidPathIndicator
path as any other reserved indicator. When md1 adds the row, mk1
inherits the allocation by the mirror clause without a wire-format change.
```

Same problem as C-1, on the SPEC side. The SPEC is the in-repo canonical document; the FOLLOWUPS entry now claims `md-path-dictionary-0x16-gap` is resolved-by-mk-codec-v0.2.0 yet the SPEC says the gap is still open.

**Recommended fix:** parallel update — add 0x16 row to the table, replace or delete the note paragraph. Cross-link to mk-codec v0.2.0 (or just to the CHANGELOG).

The Phase 3 work updated SPEC §3.1 (bit-3 footnote, lines 154-161) but missed the §3.5 path-dictionary section. Same pattern in the BIP draft — Phase 3 found the bit-3 spot but missed the parallel path-dictionary spot. A single-grep on `0x16` in both docs pre-commit would have caught both.

### Important

#### I-1. FOLLOWUPS `vector-corpus-dictionary-coverage` resolution claim is now stale

`design/FOLLOWUPS.md` line 144:

```
- **Status:** `resolved 2417401` (v0.1.1 Phase 2). Added V9..V17 covering
  9 of the 10 missing indicators; 0x16 (BIP 48 testnet nested-segwit)
  remains intentionally skipped pending the cross-repo
  md-path-dictionary-0x16-gap resolution.
```

Now stale. Rewrite the trailing clause to: "0x16 was the 10th, intentionally skipped at v0.1.1 pending the cross-repo `md-path-dictionary-0x16-gap` resolution; closed in mk-codec v0.2.0 (V18)." Or, more cleanly, append a second `Status` bullet: "`fully-resolved` mk-codec-v0.2.0: V18 closes the 0x16 gap."

This is doc-only and "important" rather than "critical" because it's an internal stewardship doc, not the public BIP. But it's the kind of stale text that erodes trust in the FOLLOWUPS tracker as a source of truth.

### Suggestions

#### S-1. CHANGELOG migration pointer

The CHANGELOG `Notes` section calls out the wire-additive asymmetry but doesn't surface the SHA-pin migration explicitly. A third-party implementation pinning the v0.1.1 corpus SHA will fail SHA verification when moving to v0.2.0. Consider adding:

```
- Cross-implementations validating against the v0.1.x corpus need to
  update their SHA-256 pin to match the regenerated v0.2.0 corpus and
  expect 18 clean + 22 negative = 40 vectors (was 17 + 22 = 39). The
  family token rolls under the Q-10 minor-bump convention; v0.1.x
  corpora remain valid for v0.1.x consumers.
```

The v0.1.1 CHANGELOG had a similar migration pointer ("Cross-implementations validating against the v0.1.0 corpus need to update their `V0_1_SHA256` pin to match the expanded v0.1.1 corpus..."). v0.2.0 should follow the same pattern. Optional but it makes life easier for the cross-implementer.

#### S-2. Filename vs. family-token mismatch

The corpus file is named `v0.1.json` but now carries `family_token: "mk-codec 0.2"`. md-codec faces the same shape (their corpus file is named after a stable filename, not the family-token version). Not a bug — the filename is intentionally stable while the family-token rolls — but a one-line note in CHANGELOG or a comment in `tests/vectors.rs` near `VECTOR_FILE` would head off "the file is named v0.1 but contains v0.2 metadata, what gives" reviewer questions. Defer to user.

#### S-3. `gen_mk_vectors.rs` module rustdoc still says "v0.1 vector corpus"

`crates/mk-codec/src/bin/gen_mk_vectors.rs:1`:

```rust
//! Generator for the canonical mk-codec v0.1 vector corpus.
```

The corpus is now generated against `GENERATOR_FAMILY = "mk-codec 0.2"`. Either change "v0.1" to "v0.x" or "v0.1.x and beyond" or just drop the version specifier — the doc-comment is accurate that it generates *the* corpus, but "v0.1" is now misleading. Defer to user.

#### S-4. Path indicator listing in §"Case A" prose (BIP) is verbose vs. table-driven

If C-1 is being fixed anyway, consider rewriting the entire Case A prose paragraph to defer to a single table (mainnet 0x01..=0x07, testnet 0x11..=0x17, byte-for-byte mirror of md1's `Tag::SharedPath`). Less prose to keep in sync as the dictionary evolves. Defer to user.

---

## 3. Verification details (per requested checklist)

### 3.1 Wire-additive correctness (✓ confirmed)

`STANDARD_PATHS` at `path.rs:34-51` now contains 14 entries; `decode_path` at `path.rs:97-106` resolves indicators against the table and returns `Err(Error::InvalidPathIndicator(0x16))` for v0.1.x decoders consuming a v0.2.0-emitted 0x16 string. The asymmetry is correctly documented in CHANGELOG `[0.2.0]` Notes.

SemVer minor bump is the right classification: a wire change that is one-way backward-compatible (new decoder reads old strings; old decoder rejects new strings with a specific Error variant rather than UB) is by SemVer convention a minor bump, not a patch. md-codec's v0.9.0 classified the parallel change identically. If anything, an argument could be made for major (since v0.1.x decoders DO observe a behavior change) but the SemVer 2.0 spec scopes "backward compatible" to "API surface", not "wire format", and the API didn't change — the same `decode()` returning `Err` is the documented contract.

### 3.2 STANDARD_PATHS table integrity (✓ confirmed)

- 14 entries (was 13). ✓
- 0x16 row sits between 0x15 (line 48) and 0x17 (line 50). ✓
- Path string is `m/48'/1'/0'/1'`. ✓ Matches md1's parallel entry.
- Module rustdoc claim "7 mainnet (`0x01`..=`0x07`) and 7 testnet (`0x11`..=`0x17`)" matches reality. ✓ (path.rs:31)

### 3.3 V18 fixture sanity (✓ hand-decoded byte-by-byte)

V18 `canonical_bytecode_hex` decodes to:

| offset | bytes | meaning |
|---|---|---|
| 0 | `04` | bytecode header — version 0, bit 2 set (fp present), bits 0/1/3 zero ✓ |
| 1 | `01` | stub_count = 1 ✓ |
| 2..6 | `4816aabb` | policy_id_stub (4 bytes) ✓ |
| 6..10 | `4816ccdd` | origin_fingerprint (4 bytes, present per header bit 2) ✓ |
| 10 | `16` | path indicator (0x16 — the new entry) ✓ |
| 11..15 | `043587cf` | xpub.version (testnet xpub) ✓ |
| 15..19 | `10203012` | xpub.parent_fingerprint ✓ |
| 19..51 | `b8b8...b8` (32 bytes) | xpub.chain_code ✓ |
| 51..84 | `036360...73f7` (33 bytes) | xpub.public_key ✓ |
| total | **84 bytes** | matches typical 1-stub-with-fp size ✓ |

Network field is `"testnet"`, xpub starts with `tpub` (`tpubDE2Qenr6qBXK1...`), origin path is `m/48'/1'/0'/1'`. Every V18 claim in the brief checks out.

### 3.4 Generator family token roll (✓ confirmed in three places)

- `consts.rs:50`: `pub const GENERATOR_FAMILY: &str = "mk-codec 0.2";` ✓
- `tests/vectors.rs:119`: `"mk-codec 0.2"` (schema_metadata_pinned) ✓
- `tests/vectors/v0.1.json` line 2: `"family_token": "mk-codec 0.2"` ✓

md-codec's v0.8 → v0.9 rolled their family token under the same Q-10 rule. ✓

### 3.5 Bit-3 footnote correctness (✓ confirmed in BIP and SPEC)

Both BIP §"Bytecode header" (lines 230-237) and SPEC §3.1 (lines 154-161) carry the new "shape-shared / bit-2 semantics shared / bits diverge from bit 3 onward" framing, with explicit citation of md-codec v0.10.0's bit-3 reclamation as the OriginPaths flag. Cross-checked against md-codec's actual v0.10.0 release: commit `fdf187e` titled "release(v0.10.0): per-@N origin paths + header bit 3 reclaim" and tag `md-codec-v0.10.0` exists. The cite link `https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.10.0` is well-formed (assumes user has published the GitHub release; if not, the link 404s but the prose still parses).

The "shape-shared" claim still holds (4-bit version + 4 flag bits, both formats), bit-2-semantics-shared still holds (optional fp), bits 0/1/3 are correctly described as independent allocations (md1's bit 3 is OriginPaths flag in v0.10.0; mk1's bit 3 is reserved-must-be-zero). ✓

### 3.6 Test reshape correctness (✓ confirmed; tests pass)

- `path.rs:267-279`: `round_trip_indicator_0x16_added_in_v0_2` — round-trips `m/48'/1'/0'/1'` through the encoder and decoder, asserts `[0x16]` encoding. Passes. ✓
- `decode.rs:181-185`: `rejects_invalid_path_indicator` now uses `wire[10] = 0x18` and asserts `Err(Error::InvalidPathIndicator(0x18))`. Passes. ✓ The comment block at decode.rs:174-180 correctly documents the rationale and cross-references `bytecode/path::round_trip_indicator_0x16_added_in_v0_2`.

The test rename is also picked up by the strum-driven exhaustiveness gate at `tests/error_coverage.rs` — `InvalidPathIndicator` continues to be exercised by N15 in the negative corpus, so removing the 0x16-specific reserved test doesn't leave the variant uncovered.

### 3.7 CHANGELOG completeness (✓ matches with one suggestion)

Cross-referenced `[0.2.0]` against the actual diff. Every commit change is surfaced:

- `Added`: 0x16 indicator ✓; V18 vector ✓; `tests/error_coverage.rs` (Phase 1 work) ✓; strum dev-dep ✓.
- `Changed`: `GENERATOR_FAMILY` roll ✓; bit-3 footnote ✓; SHA-256 pin update ✓.
- `Removed`: old `every_error_variant_has_negative_vector` runtime gate ✓.
- `Resolved`: both `error-variant-exhaustiveness-gate-strum` and `md-path-dictionary-0x16-gap` ✓.
- `Notes`: wire-additive asymmetry ✓; backward compat ✓; mirror-stewardship contract auto-extension ✓; bit-3 footnote scope ✓.

The wire-additive nature, family-token roll, SHA-pin rotation, and cross-implementation migration note are all surfaced. One suggestion (S-1) on adding an explicit "to migrate from v0.1.x" pointer; otherwise CHANGELOG is excellent.

### 3.8 FOLLOWUPS status accuracy (✓ correctly cites both repos)

`md-path-dictionary-0x16-gap` status text (FOLLOWUPS.md:115):

> `resolved by md-codec-v0.9.0 + mk-codec-v0.2.0`. md-codec v0.9.0 added the 0x16 row to md1's path-dictionary table. mk-codec v0.2.0 closed the parallel gap on the mk1 side: added `(0x16, "m/48'/1'/0'/1'")` to `STANDARD_PATHS`, regenerated the corpus with V18 exercising the indicator, rolled `GENERATOR_FAMILY` to `"mk-codec 0.2"` (Q-10: minor bumps roll the family token). Wire-additive: v0.1.x decoders reject v0.2-emitted 0x16 strings.

Correctly cites both repos with release tags. ✓

But: I-1 above — the related entry `vector-corpus-dictionary-coverage` (FOLLOWUPS.md:144) didn't get the parallel update and still says 0x16 "remains intentionally skipped." Important nit.

### 3.9 Generator-token-vs-corpus-name mismatch (S-2)

Documented above. Not a bug; the filename is intentionally stable. md-codec follows the same convention. A one-line CHANGELOG or vectors.rs comment would head off reviewer questions.

### 3.10 Migration ergonomics (S-1)

The CHANGELOG already calls out the wire-additive asymmetry but doesn't have an explicit "to migrate" pointer for a v0.1.1 → v0.2.0 cross-implementer. v0.1.1's CHANGELOG had this pattern; v0.2.0 should mirror it. Suggestion-tier, not blocking.

### 3.11 Anything missing (the "anything we haven't asked you about" sweep)

- **Stale source comments referencing v0.1.x naming:** `path.rs:49` comment (`v0.2.0+; was reserved-pending in v0.1.x`) and `path.rs:268` test comment (`0x16 was reserved-pending in v0.1.x`) are intentional provenance notes; not stale.
- **`gen_mk_vectors.rs:1` module rustdoc** says "v0.1 vector corpus" — minor staleness, S-3 above.
- **FOLLOWUPS entries that should be touched but weren't:** I-1 above (`vector-corpus-dictionary-coverage` resolution claim). The `path-dictionary-mirror-stewardship` entry (FOLLOWUPS.md:175-182) is correctly marked resolved-by-md-codec-v0.9.0 and doesn't need touching for v0.2.0; the contract is now observable in mk-codec's STANDARD_PATHS being a literal mirror of md-codec's table.
- **Cross-references with md1 BIP that need updating:** the bit-3 footnote correctly cites md-codec v0.10.0; no other md-codec post-v0.8 sections appear in mk1's BIP that I could find. The mk1 BIP `chunk_set_id` rename pointer to md-codec v0.9.0 (BIP line 50) is correct; the Wallet Instance ID and Policy ID definition cites to md-codec v0.8.0 are correct.
- **Release plumbing:** Cargo.toml at 0.2.0 ✓; Cargo.lock regenerated ✓; CHANGELOG `[0.2.0]` section dated 2026-04-30 (one day in the future from today's 2026-04-29 — minor; fine if the user plans to land the tag tomorrow, otherwise consider updating to 2026-04-29).

---

## 4. Recommendation

**Do not tag yet.** Land a `style/fix(mk-codec phase 2-4): apply review fixes` follow-up commit fixing C-1 (BIP §"Origin path encoding") and C-2 (SPEC §3.5) before tagging `mk-codec-v0.2.0`. The contradictory text would be visible to anyone reading the public BIP draft alongside the v0.2.0 release.

I-1 (FOLLOWUPS `vector-corpus-dictionary-coverage` resolution claim) should be folded into the same fixup commit. S-1, S-2, S-3, S-4 are all defer-to-user; none block the release.

After the fixup commit lands and tests still pass (the changes are doc-only so this is essentially guaranteed), proceed to:

1. Tag `mk-codec-v0.2.0` on the fixup commit.
2. Push branch + tag.
3. Open the GitHub release with the CHANGELOG `[0.2.0]` section as the body.

The work shipped in `fd6a407` is otherwise excellent: V18 hand-decode is byte-exact, the STANDARD_PATHS extension is minimal and correctly placed, the bit-3 footnote is sharp and cites the right md-codec release, the family-token roll is correctly threaded through three places, and the test reshape is clean. The only deficiency is a missed search-and-update pass on the §"Origin path encoding" / §3.5 sections; once those land, this is a clean v0.2.0 release.
