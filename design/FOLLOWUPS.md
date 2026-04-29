# mk1 follow-up tracker

Single source of truth for items surfaced during a review or implementation pass that were not fixed in the same commit. Mirrors the convention in the `descriptor-mnemonic` (md1) repo.

## How to use this file

**Format for each entry:**

```markdown
### `<short-id>` — <one-line title>

- **Surfaced:** <Phase X.Y review of commit SHA>, or <inline TODO at file:line>, or <design discussion 2026-MM-DD>
- **Where:** <file:line> or <design — section name>
- **What:** 1–3 sentences describing the gap or improvement
- **Why deferred:** the reason it didn't ship in the original commit
- **Status:** `open` | `resolved <COMMIT>` | `wont-fix — <one-line reason>`
- **Tier:** `v0.1-blocker` | `v0.1-nice-to-have` | `v0.2` | `pre-bip-submission` | `cross-repo` | `v1+` | `external`
```

The `<short-id>` is a stable handle (e.g., `chunk-set-id-rename`, `nums-structural-audit`). Reference this id from commit messages when closing: `closes FOLLOWUPS.md chunk-set-id-rename`.

## Conventions for adding items

**During a review subagent run:** the reviewer should append to this file (one entry per minor item) and reference it in their report. Reviewers in parallel batches must not write to this file simultaneously — the controller appends afterwards from the consolidated reports.

**During an implementer subagent run:** if the implementer notices a side concern they explicitly chose not to fix in their commit, they append an entry here in the same commit.

**During controller (main-thread) work:** when wrapping a task, the controller verifies all minor items from that task's reviews are either resolved or recorded here.

## Tiers

- **`v0.1-blocker`**: must fix before tagging `mk-codec-v0.1.0`. Failing to fix = ship blocked.
- **`v0.1-nice-to-have`**: should fix before v0.1 if time permits, but won't block release. Document the deferral in v0.1's CHANGELOG/README.
- **`v0.2`**: explicitly deferred to v0.2.
- **`pre-bip-submission`**: not blocking v0.1 release, but MUST be resolved before formal BIP submission per D-11. Examples: NUMS structural audit, HRP collision check.
- **`cross-repo`**: depends on action in `descriptor-mnemonic` repo.
- **`v1+`**: deferred indefinitely.
- **`external`**: depends on work outside both repos.

---

## Open items

### `chunk-set-id-rename` — rename "wallet identifier" to `chunk_set_id` in md1

- **Surfaced:** 2026-04-29 mk1 closure-design pass (Q-5(d)).
- **Where:** `descriptor-mnemonic` repo — BIP draft `bip/bip-mnemonic-descriptor.mediawiki` line ~188; `md-codec` reference implementation symbols carrying "wallet identifier" naming; mk1's own SPEC §2.5 already uses `chunk_set_id` per closure lock.
- **What:** md1 v0.8.0 shipped with the 20-bit chunked-header random tag named "wallet identifier" — a name that conflicts with `Policy ID` and `Wallet Instance ID` and means neither. Closure design Q-5 locks the rename to `chunk_set_id` across both repos. Wire format unchanged; this is purely a documentation and code-symbol rename.
- **Why deferred:** Lives in the descriptor-mnemonic repo, not this one. mk1's spec already uses the new name.
- **Sequencing requirement:** the rename MUST land in md-codec (likely a docs-and-symbols-only release, e.g. md-codec v0.9.0) **before** mk1's BIP draft is submitted. mk1's BIP cites md1 by field name; mk1 cannot publish referencing a name md1 itself does not use.
- **Status:** `open`
- **Tier:** `cross-repo`

### `md-per-N-path-tag-allocation` — md1's per-`@N` path bytecode tag allocation (Q-4)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (Q-4).
- **Where:** `descriptor-mnemonic` repo — md1 bytecode tag table; new tag in unallocated `0x36+` range, or backfill `0x24-0x32`.
- **What:** mk1 declares the authority-precedence semantics (mk1's `origin_path` is authoritative; md1's per-`@N` path is descriptive). The wire-format question of which tag byte md1 uses is an md-repo decision. mk1 cannot answer it.
- **Why deferred:** Lives in the descriptor-mnemonic repo's next phase.
- **Status:** `open`
- **Tier:** `cross-repo`

### `nums-structural-audit` — structural-relationship audit of `MK_REGULAR_CONST` / `MK_LONG_CONST`

- **Surfaced:** 2026-04-29 mk1 closure-design pass (Q-1, captured as pre-BIP-submission audit item (1)).
- **Where:** design / cryptography review.
- **What:** Verify there are no accidental structural relationships between the locked target constants and the BIP 93 BCH polynomial. Required: weight-distribution analysis under the new target, intersection of mk1 codeword space with md1 and codex32 codeword spaces, confirmation that error-correction guarantees (8-character detection, 4-substitution correction) hold under the new constants.
- **Why deferred:** Not a v0.1 implementation gate; gates formal BIP submission. Andrew Poelstra is the natural reviewer per D-11.
- **Status:** `open`
- **Tier:** `pre-bip-submission`

### `hrp-mk-collision-check` — formal HRP `mk` collision verification

- **Surfaced:** 2026-04-29 mk1 closure-design pass (D-9 / pre-BIP-submission audit item (2)).
- **Where:** SLIP-0173 (informal segwit-HRP registry); recent bitcoin-dev mailing-list archives; BIPs PR history.
- **What:** Search for any soft `mk` claim before formal SLIP-0173 registration. None expected, but confirmation is the registration gate. Alternatives `mx`, `mkc`, `mpk` documented in D-9 if collision is found.
- **Why deferred:** Not a v0.1 gate; gates formal HRP registration.
- **Status:** `open`
- **Tier:** `pre-bip-submission`

### `bip-cross-reference-completeness` — BIP draft cross-reference audit

- **Surfaced:** 2026-04-29 mk1 closure-design pass (pre-BIP-submission audit item (3)).
- **Where:** `bip/bip-mnemonic-key.mediawiki` — final cross-reference pass before submission.
- **What:** mk1's BIP draft must cross-reference: BIP 93 (codex32 plumbing reuse), BIP 32 (xpub serialization), BIP 380 (origin notation), BIP 388 (wallet policy / Policy ID semantics), and the published md1 BIP (linkage protocol, shared-parser conventions, `chunk_set_id` field). Any post-rename of "wallet identifier" → `chunk_set_id` in md1 (see `chunk-set-id-rename` above) MUST land before mk1's draft is finalized.
- **Why deferred:** Final pre-submission audit step; depends on `chunk-set-id-rename` landing first.
- **Status:** `open`
- **Tier:** `pre-bip-submission`

### `decoder-error-variant-parity` — Error-variant ↔ negative-vector parity

- **Surfaced:** 2026-04-29 mk1 closure-design opus review pass (pre-BIP-submission audit item (4)).
- **Where:** `crates/mk-codec/src/error.rs` (variants), test vectors negative-cases corpus (TBD).
- **What:** Every reject case in SPEC §4 validity rules MUST map to a uniquely-named `Error` variant in the reference crate, and every variant MUST have at least one planned negative test vector. Mirrors md-codec's 30-negative-vectors-one-per-Error-variant conformance contract.
- **Why deferred:** v0.1 implementation will define the Error variants; the *parity gate* (every variant has a vector, no orphaned variants, no variantless reject paths) is checked just before BIP submission and v1.0 release.
- **Status:** `open`
- **Tier:** `pre-bip-submission`

### `md-path-dictionary-0x16-gap` — md1 path dictionary missing testnet 0x16 entry

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 2 BIP review (commit 4728230).
- **Where:** `descriptor-mnemonic` repo — md1 BIP `bip-mnemonic-descriptor.mediawiki` §"Path dictionary" lines ~339-349. Testnet rows list 0x11, 0x12, 0x13, 0x14, 0x15, 0x17 — **0x16 omitted** (no testnet pair for mainnet 0x06 = `m/48'/1'/0'/1'`, BIP 48 nested-segwit multisig testnet).
- **What:** Mainnet has 0x06 (`m/48'/0'/0'/1'`, BIP 48 nested-segwit multisig) but the testnet companion 0x16 (`m/48'/1'/0'/1'`) is absent from md1's published BIP table. mk1's spec and BIP both claim "exact dictionary mirrors md1's `Tag::SharedPath` table byte-for-byte"; mk1 inherits the gap. mk1 v0.1 BIP §"Origin path encoding" footnotes this — `0x16` is reserved-pending-md1-update — but the cleanest fix is to add the missing 0x16 row in md1.
- **Why deferred:** Lives in the descriptor-mnemonic repo. Not blocking mk1 v0.1 wire-level interop because no encoder can legitimately emit 0x16 today (md1 would reject).
- **Status:** `open`
- **Tier:** `cross-repo`

### `chunked-header-total-chunks-wire-encoding-clarification` — SPEC §2.5 wording on `total_chunks` field

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 5 string-layer implementation.
- **Where:** mk1 SPEC §2.5 ("String-layer header" / chunked variant); mk1 BIP §"String-layer header" / "Chunked".
- **What:** The chunked-header `total_chunks` field was documented as "5 bits, range 1..=32," but 32 distinct values 1..=32 do not fit in 5 bits (which hold 0..=31). The mk-codec v0.1 reference implementation resolves the mismatch by encoding `count - 1` on the wire (wire 0..=31 → semantic 1..=32). The same gap applied to `chunk_set_id` endian convention — "20 bits" was silent on packing order.
- **Resolution (2026-04-29, Phase 5 review fixup):** added explicit "Wire encoding for `total_chunks`" (`count − 1`) and "Wire encoding for `chunk_set_id`" (big-endian 5-bit-symbol order) paragraphs to both `design/SPEC_mk_v0_1.md` §2.5 and `bip/bip-mnemonic-key.mediawiki` §"Chunked header". The reference implementation already encoded both correctly; this is purely a documentation tightening.
- **Status:** `closed`
- **Tier:** `pre-bip-submission`

### `error-variant-exhaustiveness-gate-strum` — replace runtime substring gate with a compile-time variant-iteration check

- **Surfaced:** 2026-04-29 v0.1.1 Phase 3 review (I-1, commit 1e42354).
- **Where:** `crates/mk-codec/tests/vectors.rs::every_error_variant_has_negative_vector`.
- **What:** The milestone v0.1.1 plan §3.3.2 specified an in-crate exhaustive `match` over `Error` variants for compile-time enforcement of negative-vector coverage. The implementation reverted to a runtime substring gate (`assert_variant_covered("...")`) because `#[non_exhaustive]` blocks integration-test exhaustive matching even for in-crate test targets — rustc treats integration tests as separate crates. The runtime gate fails when a vector is missing for a known variant, but it doesn't fire when a *new* variant is added without a corresponding `assert_variant_covered` call. The same gap applies to `error.rs::tests::parameterized_variants_render` and `static_variants_render` — both are hand-curated lists.
- **Why deferred:** Two viable resolutions, both v0.2-grade:
  1. Add `strum = { version = "0.26", features = ["derive"] }` as a dev-dep and `#[derive(strum_macros::EnumIter)]` on `Error`. The test iterates `Error::iter()` and asserts coverage for every variant. This is the path md-codec uses for its `error_coverage` test.
  2. Move the gate into `crates/mk-codec/src/error.rs::tests` (a unit-test module inside the crate), where exhaustive matching IS compile-time-checked even with `#[non_exhaustive]`. Pair with a dynamic JSON-loading helper so the unit test reads the vector corpus.
- **Status:** `open`
- **Tier:** `v0.2-nice-to-have`

### `vector-corpus-dictionary-coverage` — v0.1 corpus exercises only 4 of 13 path-dictionary entries

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 6 review (M-1, commit 053a54c).
- **Where:** `crates/mk-codec/tests/vectors/v0.1.json` (V1..V8 fixture set).
- **What:** The v0.1 vector corpus exercises std-table indicators 0x03 (BIP 84), 0x05 (BIP 48 segwit-v0 mainnet), 0x07 (BIP 87), and 0x15 (BIP 48 testnet) plus the 0xFE explicit-path codec. Missing: 0x01 (BIP 44), 0x02 (BIP 49), 0x04 (BIP 86), 0x06 (BIP 48 nested-segwit mainnet), and the testnet entries 0x11, 0x12, 0x13, 0x14, 0x17. A third-party encoder could pass all 8 v0.1 vectors while still mishandling BIP 44/49/86 mainnet inputs.
- **Why deferred:** The internal encoder unit test `bytecode/path::round_trip_all_standard_paths` already cycles every dictionary entry; the gap is in the cross-implementation conformance corpus, not in encoder correctness. Closing the gap is straightforward (one fixture per missing indicator) but expands the corpus from 8 to ~14 vectors; defer to the pre-bip-submission corpus expansion.
- **Status:** `open`
- **Tier:** `pre-bip-submission`

### `cross-chunk-hash-test-fixture-stability` — Phase 5 perturbation test fixture brittleness

- **Surfaced:** 2026-04-29 Phase 5 review (M-3, commit 12c54f8).
- **Where:** `crates/mk-codec/src/string_layer/pipeline.rs` test `decode_rejects_perturbed_cross_chunk_hash`.
- **What:** The test perturbs the last byte of the last chunk's fragment and re-encodes, asserting `CrossChunkHashMismatch`. Under the current fixture this works, but the test depends on the perturbation not landing somewhere the BCH t=4 correction silently un-perturbs into a CRC-valid bytecode. A future fixture change could mask the test. Cleanest fix: perturb in 5-bit-symbol space *after* re-encoding, or pin a perturbation pattern at BCH-distance > 4 from any valid codeword in the chunk's data part.
- **Why deferred:** Test is currently green; the brittleness is potential, not actual. v0.1-nice-to-have.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `pipeline-decode-mixed-header-error-naming` — `ChunkedHeaderMalformed` variant overloaded

- **Surfaced:** 2026-04-29 Phase 5 review (M-5, commit 12c54f8).
- **Where:** `crates/mk-codec/src/string_layer/pipeline.rs::decode` — the `[SingleString, Chunked, ...]` and `[Chunked, SingleString, ...]` rejection paths surface as `Error::ChunkedHeaderMalformed("…")`. The variant name suggests a chunked-set issue; the actual condition is "header types disagree across the supplied strings." Consider adding a dedicated `MixedHeaderTypes` Error variant (or a more specific `String`-parameterised variant) when the v0.2 wire format admits more chunk types and the discrimination matters.
- **Why deferred:** Reachable only through user error; current message text is clear. Variant proliferation has its own cost. v0.1-nice-to-have.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `encode-with-chunk-set-id-singlestring-silent-ignore` — explicit `chunk_set_id` is silently dropped

- **Surfaced:** 2026-04-29 Phase 5 review (M-6, commit 12c54f8).
- **Where:** `crates/mk-codec/src/string_layer/pipeline.rs::encode_with_chunk_set_id`.
- **What:** When the bytecode lands in single-string territory, the `chunk_set_id` parameter is silently ignored. This is friendly but masks a Phase-6-vector-regenerator failure mode: if the SingleString-vs-Chunked cutoff drifts, vectors pinned with explicit chunk_set_id may stop testing what they intended. Consider returning `Err(Error::ChunkedHeaderMalformed("chunk_set_id supplied but encoding is SingleString"))` when the override is supplied and the bytecode fits in a single string. Alternative: document that the test harness should assert the `chunked vs single` plan before pinning.
- **Why deferred:** The Phase-6 vector corpus generator (next phase) will surface this if it happens; better to defer the API decision until the regenerator is concrete.
- **Status:** `wont-fix — moot per SPEC §2.4 (SingleString unreachable for v0.1 conforming KeyCards).`
- **Closure note:** Closed during v0.1.1 Phase 1 Task 1.3 (`design/MILESTONE_v0_1_1.md`). The smallest valid v0.1 bytecode is 80 bytes (1+1+4+1+73), already above SINGLE_STRING_LONG_BYTES = 56; the SingleString branch in `encode_with_chunk_set_id` is dead code under the v0.1 wire format, and the `chunk_set_id` argument is therefore never silently dropped under any conforming input.
- **Sequencing requirement:** if a future format extension lands a smaller bytecode (e.g., the Compact-65 mode discussed in SPEC §3.6, which would drop `xpub.version` + `xpub.parent_fingerprint` and bring some bytecodes below 56 bytes), this item MUST be re-opened **before the format extension ships**. The silent-drop semantics is friendly today but masks an encoder-side determinism bug under any wire format that makes SingleString reachable. Any future smaller-bytecode design pass (or a Compact-65-shaped FOLLOWUPS entry) MUST cite this requirement and re-open the issue.
- **Tier:** `v0.1-nice-to-have`

### `path-dictionary-mirror-stewardship` — formalize mk1↔md1 path-dictionary inheritance contract

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 2 BIP review open observation (commit 4728230).
- **Where:** mk1 SPEC §3.5; mk1 BIP §"Origin path encoding"; md1 BIP §"Path dictionary".
- **What:** mk1's path dictionary is contractually identical to md1's `Tag::SharedPath` table. If md1 allocates new dictionary entries (e.g., closing the 0x16 gap, or adding new BIP-style accounts in future md1 revisions), mk1 inherits the allocation by the byte-for-byte mirror clause — but the contract is currently a prose statement, not a tracked invariant. A future md1 path-dictionary entry could land without an mk1 spec amendment and produce silent drift.
- **Why deferred:** Process / stewardship concern, not a v0.1 release blocker.
- **Status:** `open`
- **Tier:** `cross-repo`
