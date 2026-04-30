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
- **`v0.2-nice-to-have`**: should land before v0.2 release if time permits but won't block; document deferral in v0.2's CHANGELOG.
- **`v0.2`**: explicitly deferred to v0.2.
- **`pre-bip-submission`**: not blocking v0.1 release, but MUST be resolved before formal BIP submission per D-11. Examples: NUMS structural audit, HRP collision check.
- **`cross-repo`**: depends on action in `descriptor-mnemonic` repo.
- **`v1+`**: deferred indefinitely.
- **`external`**: depends on work outside both repos.

---

## Open items

### `chunk-set-id-rename` — rename "wallet identifier" to `chunk_set_id` in md1 (resolved)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (Q-5(d)).
- **Where:** `descriptor-mnemonic` repo — BIP draft `bip/bip-mnemonic-descriptor.mediawiki` line ~188; `md-codec` reference implementation symbols carrying "wallet identifier" naming; mk1's own SPEC §2.5 already uses `chunk_set_id` per closure lock.
- **What:** md1 v0.8.0 shipped with the 20-bit chunked-header random tag named "wallet identifier" — a name that conflicts with `Policy ID` and `Wallet Instance ID` and means neither. Closure design Q-5 locks the rename to `chunk_set_id` across both repos. Wire format unchanged; this is purely a documentation and code-symbol rename.
- **Why deferred:** Lives in the descriptor-mnemonic repo, not this one. mk1's spec already uses the new name.
- **Sequencing requirement:** the rename MUST land in md-codec (likely a docs-and-symbols-only release, e.g. md-codec v0.9.0) **before** mk1's BIP draft is submitted. mk1's BIP cites md1 by field name; mk1 cannot publish referencing a name md1 itself does not use.
- **Status:** `resolved by md-codec-v0.9.0` ([release](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0), merge commit `9eeb9ab` in `bg002h/descriptor-mnemonic`). The rename landed across ~85 sites / ~150 references in md-codec docs + symbols. mk1's BIP-submission gate is cleared. Cross-update pass on the mk1 side: BIP §"Naming and identifiers" updated past-tense; DECISIONS D-15 sequencing-requirement updated past-tense.
- **Tier:** `cross-repo`

### `md-per-N-path-tag-allocation` — md1's per-`@N` path bytecode tag allocation (Q-4) (resolved)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (Q-4).
- **Where:** `descriptor-mnemonic` repo — md1 bytecode tag table; new tag in unallocated `0x36+` range, or backfill `0x24-0x32`.
- **What:** mk1 declares the authority-precedence semantics (mk1's `origin_path` is authoritative; md1's per-`@N` path is descriptive). The wire-format question of which tag byte md1 uses is an md-repo decision. mk1 cannot answer it.
- **Why deferred:** Lived in the descriptor-mnemonic repo's next phase. md1's parallel entry (`md-per-at-N-path-tag-allocation` in `descriptor-mnemonic/design/FOLLOWUPS.md`) was scheduled whenever per-`@N` paths became a planned md release feature.
- **Status:** `resolved by md-codec-v0.10.0` ([release](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.10.0), merge commit `172830a` in `bg002h/descriptor-mnemonic`). md1 allocated `Tag::OriginPaths = 0x36` and reclaimed header bit 3 as the OriginPaths flag; per-`@N` divergent origin paths are now first-class on the policy card. mk1's BIP §"Authority precedence (MK ↔ MD path information)" pins the cross-format precedence semantics; no mk1-side wire-format change was required. mk1 cross-update pass on 2026-04-29 (post md-codec v0.10.0 ship): BIP §"Authority precedence" updated past-tense; SPEC §5.1 updated past-tense; DECISIONS Q-4 / closure-design §Q-4 + §3 item (2) updated past-tense.
- **Tier:** `cross-repo`

### `nums-structural-audit` — structural-relationship audit of `MK_REGULAR_CONST` / `MK_LONG_CONST` (resolved at md1's bar)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (Q-1, captured as pre-BIP-submission audit item (1)).
- **Where:** design / cryptography review.
- **What:** Verify there are no accidental structural relationships between the locked target constants and the BIP 93 BCH polynomial. Required: weight-distribution analysis under the new target, intersection of mk1 codeword space with md1 and codex32 codeword spaces, confirmation that error-correction guarantees (8-character detection, 4-substitution correction) hold under the new constants.
- **Why deferred:** Not a v0.1 implementation gate; gates formal BIP submission. Andrew Poelstra is the natural reviewer per D-11.
- **Status:** `resolved at md1's bar` (2026-04-29 cross-update pass). md1 / md-codec ship with the same NUMS construction (truncate top-N bits of `SHA-256(domain_string)`) and chose to document the construction in the BIP itself with a Python reproducer rather than commission a separate structural audit; `md`'s BIP §"Why new target constants?" is the audit trail. mk1 already meets that bar: BIP §"Why new target constants?" carries the equivalent reproducer for `b"shibbolethnumskey"`; SPEC §2.3 carries the same; and `consts.rs::tests::nums_constants_reproduce_from_domain` reproduces the construction at runtime (`cargo test`-enforced). The original FOLLOWUPS entry called for an external Poelstra structural review — a higher bar than md1 chose. Per the project's "don't adopt a higher bar than md1" principle, the entry is closed at the audit-trail-in-BIP level. If a future reviewer (Poelstra or other) volunteers a structural pass, it can land as a strengthening note in the BIP without re-opening this gate.
- **Tier:** `pre-bip-submission`

### `slip-0173-register-mk-hrp` — file SLIP-0173 PR registering `mk` HRP (resolved)

- **Surfaced:** 2026-04-29 cross-update pass after closing `hrp-mk-collision-check`. md1 filed a parallel PR (#2011 at satoshilabs/slips) registering `md` as a defensive measure; mk1 follows the same pattern.
- **Where:** [satoshilabs/slips](https://github.com/satoshilabs/slips) PR adding one row to `slip-0173.md`. Draft PR text + diff at `design/SLIP_0173_PR_DRAFT.md`.
- **What:** Defensive registration of the `mk` HRP in SLIP-0173 to close off future collision risk from independent Bitcoin-family projects. The registration is a docs-level act in the SatoshiLabs registry; no code change in mk-codec, no wire-format implications, no binding consequence beyond the registry record.
- **Why deferred:** Single user-action item (file the PR under the maintainer's GitHub account). The `hrp-mk-collision-check` audit at `design/AUDIT_hrp_mk_collision.md` cleared the technical gate; this entry tracks the actual PR filing.
- **Status:** `resolved 2026-04-29 — PR filed at https://github.com/satoshilabs/slips/pull/2012`. The requested action (FILE the PR) is complete; merge state is now tracked externally on SatoshiLabs review cadence and is no longer an mk1-side deferral. Parallel to md1's `slip-0173-register-md-hrp` (PR #2011 at the same repo, also still in external-review state). If #2011 merges first, #2012 will need a one-line rebase to insert `mk` after `md` rather than after `Lightning Network`; otherwise the two PRs are mergeable in either order.
- **Tier:** `pre-bip-submission` (closed; awaiting upstream merge tracked separately)

### `hrp-mk-collision-check` — formal HRP `mk` collision verification (resolved)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (D-9 / pre-BIP-submission audit item (2)).
- **Where:** SLIP-0173 (informal segwit-HRP registry); recent bitcoin-dev mailing-list archives; BIPs PR history.
- **What:** Search for any soft `mk` claim before formal SLIP-0173 registration. None expected, but confirmation is the registration gate. Alternatives `mx`, `mkc`, `mpk` documented in D-9 if collision is found.
- **Why deferred:** Not a v0.1 gate; gates formal HRP registration.
- **Status:** `resolved` — see [`design/AUDIT_hrp_mk_collision.md`](AUDIT_hrp_mk_collision.md). SLIP-0173 has no `mk` registration; closest neighbours (`ms` BIP 93, `md` Mnemonic Descriptor, `mm` Miden, `my` Myriad) are at Hamming distance 1 but BIP 173 HRP-mixing prevents cross-HRP false-positive validation (≈ 2⁻⁶⁵ collision probability), and mk1's NUMS-derived target residues are independent from md1's and codex32's. Formal SLIP-0173 registration of `mk` is folded into the BIP-submission workflow.
- **Tier:** `pre-bip-submission`

### `bip-cross-reference-completeness` — BIP draft cross-reference audit (resolved)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (pre-BIP-submission audit item (3)).
- **Where:** `bip/bip-mnemonic-key.mediawiki` — final cross-reference pass before submission.
- **What:** mk1's BIP draft must cross-reference: BIP 93 (codex32 plumbing reuse), BIP 32 (xpub serialization), BIP 380 (origin notation), BIP 388 (wallet policy / Policy ID semantics), and the published md1 BIP (linkage protocol, shared-parser conventions, `chunk_set_id` field). Any post-rename of "wallet identifier" → `chunk_set_id` in md1 (see `chunk-set-id-rename` above) MUST land before mk1's draft is finalized.
- **Why deferred:** Final pre-submission audit step; depends on `chunk-set-id-rename` landing first.
- **Status:** `resolved` — see [`design/AUDIT_bip_cross_reference_completeness.md`](AUDIT_bip_cross_reference_completeness.md). 74 cross-references audited across 8 categories; 9 drifts found (1 blocker, 3 important, 5 minor) and all 9 fixed inline. Notable fixes: removed phantom `Error::FingerprintFlagMismatch` cite (retired in v0.1.0 Phase 4); added `Error::MixedHeaderTypes` to §"Decoder validity rules" (added in v0.1.1 Phase 1); refreshed the stale "rename in flight" claim for `chunk_set_id` (md-codec v0.9.0/v0.9.1 has shipped); fixed BIP 380 attribution in SPEC §3.2; corrected several internal heading-quote mismatches. `chunk-set-id-rename` cross-repo dependency is now noted as resolved-on-md1-side; mk1's BIP draft is internally consistent and parity-correct with md1 v0.9.1.
- **Tier:** `pre-bip-submission`

### `decoder-error-variant-parity` — Error-variant ↔ negative-vector parity

- **Surfaced:** 2026-04-29 mk1 closure-design opus review pass (pre-BIP-submission audit item (4)).
- **Where:** `crates/mk-codec/src/error.rs` (variants), `crates/mk-codec/tests/vectors/v0.1.json` (corpus).
- **What:** Every reject case in SPEC §4 validity rules MUST map to a uniquely-named `Error` variant in the reference crate, and every variant MUST have at least one planned negative test vector. Mirrors md-codec's 30-negative-vectors-one-per-Error-variant conformance contract.
- **Why deferred:** v0.1 implementation will define the Error variants; the *parity gate* (every variant has a vector, no orphaned variants, no variantless reject paths) is checked just before BIP submission and v1.0 release.
- **Status:** `resolved 1e42354 + 59878ca` (v0.1.1 Phase 3 + Phase 3 review fixup). 22 negative vectors N1..N21, N23 cover every `Error` variant reachable from `decode`'s string-input path; `every_error_variant_has_negative_vector` integration test enforces variant coverage. `Error::CardPayloadTooLarge` is documented exempt (encoder-only — no decoder path can trigger it). The Phase 3 fixup commit `59878ca` reshaped N17 to actually trigger `InvalidPathComponent` (LEB128 overflow at 6 × 0x80) — the original 1e42354 form surfaced as `UnexpectedEnd` and left `InvalidPathComponent` exempt. Compile-time exhaustiveness via strum is recorded as `error-variant-exhaustiveness-gate-strum` for v0.2.
- **Tier:** `pre-bip-submission`

### `md-path-dictionary-0x16-gap` — md1 path dictionary missing testnet 0x16 entry (resolved)

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 2 BIP review (commit 4728230).
- **Where:** `descriptor-mnemonic` repo — md1 BIP `bip-mnemonic-descriptor.mediawiki` §"Path dictionary" lines ~339-349. Testnet rows list 0x11, 0x12, 0x13, 0x14, 0x15, 0x17 — **0x16 omitted** (no testnet pair for mainnet 0x06 = `m/48'/1'/0'/1'`, BIP 48 nested-segwit multisig testnet).
- **What:** Mainnet has 0x06 (`m/48'/0'/0'/1'`, BIP 48 nested-segwit multisig) but the testnet companion 0x16 (`m/48'/1'/0'/1'`) is absent from md1's published BIP table. mk1's spec and BIP both claim "exact dictionary mirrors md1's `Tag::SharedPath` table byte-for-byte"; mk1 inherits the gap. mk1 v0.1 BIP §"Origin path encoding" footnotes this — `0x16` is reserved-pending-md1-update — but the cleanest fix is to add the missing 0x16 row in md1.
- **Why deferred:** Lives in the descriptor-mnemonic repo. Not blocking mk1 v0.1 wire-level interop because no encoder can legitimately emit 0x16 today (md1 would reject).
- **Status:** `resolved by md-codec-v0.9.0` ([release](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0)). The 0x16 row was added to md1's path-dictionary table in v0.9.0 (`m/48'/1'/0'/1'`). mk1 v0.1 vector corpus still skips 0x16 (no fixture covers it); a v0.1.2 or v0.2 corpus expansion can add the missing vector now that md1 publishes the indicator. Tracked as a follow-on: not new FOLLOWUPS until someone wants to add the vector.
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
- **Status:** `resolved 2417401` (v0.1.1 Phase 2). Added V9..V17 covering 9 of the 10 missing indicators; 0x16 (BIP 48 testnet nested-segwit) remains intentionally skipped pending the cross-repo `md-path-dictionary-0x16-gap` resolution.
- **Tier:** `pre-bip-submission`

### `cross-chunk-hash-test-fixture-stability` — Phase 5 perturbation test fixture brittleness

- **Surfaced:** 2026-04-29 Phase 5 review (M-3, commit 12c54f8).
- **Where:** `crates/mk-codec/src/string_layer/pipeline.rs` test `decode_rejects_perturbed_cross_chunk_hash`.
- **What:** The test perturbs the last byte of the last chunk's fragment and re-encodes, asserting `CrossChunkHashMismatch`. Under the current fixture this works, but the test depends on the perturbation not landing somewhere the BCH t=4 correction silently un-perturbs into a CRC-valid bytecode. A future fixture change could mask the test. Cleanest fix: perturb in 5-bit-symbol space *after* re-encoding, or pin a perturbation pattern at BCH-distance > 4 from any valid codeword in the chunk's data part.
- **Why deferred:** Test is currently green; the brittleness is potential, not actual. v0.1-nice-to-have.
- **Status:** `resolved 8685608 + 8df9910` (v0.1.1 Phase 1 Task 1.1 + Phase 1 review fixup). Replaced with `decode_rejects_5_symbol_burst_in_last_chunk_data_part` which perturbs at the 5-bit-symbol layer **past the chunked header** (chars 11..16); a 5-symbol burst always exceeds BCH `t = 4` correction radius. Accept set widened to `{CrossChunkHashMismatch, BchUncorrectable}`. The Phase 1 fixup commit `8df9910` moved the perturbation from chars 3..8 (inside the chunked header) to chars 11..16 (post-header) so the test's accept set stays tight against actual code paths.
- **Tier:** `v0.1-nice-to-have`

### `pipeline-decode-mixed-header-error-naming` — `ChunkedHeaderMalformed` variant overloaded

- **Surfaced:** 2026-04-29 Phase 5 review (M-5, commit 12c54f8).
- **Where:** `crates/mk-codec/src/string_layer/pipeline.rs::decode` — the `[SingleString, Chunked, ...]` and `[Chunked, SingleString, ...]` rejection paths surface as `Error::ChunkedHeaderMalformed("…")`. The variant name suggests a chunked-set issue; the actual condition is "header types disagree across the supplied strings." Consider adding a dedicated `MixedHeaderTypes` Error variant (or a more specific `String`-parameterised variant) when the v0.2 wire format admits more chunk types and the discrimination matters.
- **Why deferred:** Reachable only through user error; current message text is clear. Variant proliferation has its own cost. v0.1-nice-to-have.
- **Status:** `resolved 8685608` (v0.1.1 Phase 1 Task 1.2). Added `Error::MixedHeaderTypes`; migrated `pipeline.rs:137` (forward direction) and `chunk.rs:171` (reverse direction); preserved `chunk.rs:124` defense-in-depth as `ChunkedHeaderMalformed`. CHANGELOG calls out the message-text change for downstream consumers.
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

### `path-dictionary-mirror-stewardship` — formalize mk1↔md1 path-dictionary inheritance contract (resolved)

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 2 BIP review open observation (commit 4728230).
- **Where:** mk1 SPEC §3.5; mk1 BIP §"Origin path encoding"; md1 BIP §"Path dictionary".
- **What:** mk1's path dictionary is contractually identical to md1's `Tag::SharedPath` table. If md1 allocates new dictionary entries (e.g., closing the 0x16 gap, or adding new BIP-style accounts in future md1 revisions), mk1 inherits the allocation by the byte-for-byte mirror clause — but the contract is currently a prose statement, not a tracked invariant. A future md1 path-dictionary entry could land without an mk1 spec amendment and produce silent drift.
- **Why deferred:** Process / stewardship concern, not a v0.1 release blocker.
- **Status:** `resolved by md-codec-v0.9.0` ([release](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0)). The mirror-stewardship contract was formalized in md-codec v0.9-p3 (commit `abbec54`): md1's BIP §"Path dictionary" gained an explicit "Stewardship contract" subsection naming mk1 as the mirror-inheritor and committing both repos to the byte-for-byte mirror invariant. mk1's own SPEC §3.5 already cites this contract; no mk1-side text change required. Future md1 dictionary additions automatically extend mk1's coverage by the contract, with the Path dictionary table at `bytecode/path::STD_PATHS` as the single source of truth.
- **Tier:** `cross-repo`
