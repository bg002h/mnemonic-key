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

### `mk-cli-repair-flag` — `mk repair` subcommand mirroring toolkit's `mnemonic repair`

- **Surfaced:** 2026-05-17, mnemonic-toolkit v0.22.0 brainstorm.
- **Where:** `crates/mk-cli/src/cmd/` (NEW subcommand).
- **What:** Add `mk repair <mk1>...` for mk1 BCH error-correction (regular + long codes via the already-public `mk_codec::string_layer::bch_decode::{decode_regular_errors, decode_long_errors}` per v0.3.1). Mirrors toolkit's `mnemonic repair --mk1`. Note: `mk_codec::decode` already does internal BCH correction within the same t=4 capacity, so this is primarily a UX-parity feature (explicit-fix-with-report vs silent-fix-during-decode) rather than a new capability.
- **Status:** `resolved 0ecbf1a` — `mk-cli-v0.4.0` (mnemonic-toolkit v0.22.x follow-ups cycle Phase A.3'; plan `/home/bcg/.claude/plans/nifty-wiggling-gosling.md` §2.A.2). New `cmd/repair.rs` consumes `mk_codec::string_layer::decode_string` (already-public BCH primitive) and surfaces `DecodedString` (`code`/`corrections_applied`/`corrected_positions`/`corrected_char_at`); exit 5 = REPAIR_APPLIED per D26; JSON envelope byte-matches toolkit's `RepairJson` schema per D27. 7 new integration cells in `tests/cli_repair.rs`. D25 handler-signature cascade (6 handlers `Result<()> → Result<u8>`) ships in the same release commit.
- **Resolution:** `0ecbf1a` (this repo) + cross-repo lockstep on `bg002h/mnemonic-toolkit` master (install.sh pin bump + manual chapter `## mk repair` section + companion FOLLOWUP closure) shipping concurrently.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` FOLLOWUPS.md `mk-cli-repair-flag` — closed lockstep at the cross-repo `docs(release)` commit on toolkit master.

### `from-md1-derivation-wire-version-skew` — `mk-cli/tests/round_trip.rs:45` `from_md1_derivation` fails `WireVersionMismatch { got: 0 }`

- **Filed:** 2026-05-17 (during v0.22.x follow-ups cycle Phase A.1' execution)
- **Status:** RESOLVED in mk-cli-v0.4.1 — md-codec dep bumped 0.32.1→0.34.0 and fixture refreshed against md-codec v0.34.0's canonical `pkh_basic.phrase.txt` (`md1yqpqqxzq2qwfv8urt848e`, replacing pre-v0.18-wire-format `md1qqpqqxyepwspuepy268e`). `from_md1_derivation` now passes.
- **Severity:** low (pre-existing failure on pristine main; predates this cycle)
- **Body:** the `from_md1_derivation` integration cell at `crates/mk-cli/tests/round_trip.rs:45` decodes an md1 fixture and fails with `WireVersionMismatch { got: 0 }` — the md1 fixture appears to have been authored against a pre-`mk-codec`/`md-codec` wire-version bump and was not refreshed. Cell has been failing silently for at least 3 release cycles. Fix: regenerate the md1 fixture against current md-codec, OR `#[ignore]`-gate the cell until a fresh fixture is available, OR delete the cell if md1-derivation is not in mk-cli's scope.
- **Surfaced by:** mnemonic-toolkit v0.22.x follow-ups cycle Phase A.1' (D25 handler-signature cascade). The cell was untouched by D25 but its failure became visible against the freshly-bumped Cargo.toml.

### `mk-cli-vector-corpus-inlined` — `crates/mk-cli/src/cmd/v0.1.json` is a working copy of `crates/mk-codec/tests/vectors/v0.1.json`

- **Surfaced:** v0.3 `mnemonic-gui` cycle crates.io publish round 2026-05-15. `mk-cli` `cargo publish --dry-run` rejected `include_str!("../../../mk-codec/tests/vectors/v0.1.json")` per cargo's "published package is self-contained" rule (the .crate tarball cannot reach outside the package dir).
- **Where:** `crates/mk-cli/src/cmd/vectors.rs:15` (`include_str!("v0.1.json")`) + `crates/mk-cli/src/cmd/v0.1.json` (34KB; working copy of the canonical mk-codec test fixture).
- **What:** The SHA-pinned v0.1 `mk-codec` test-vector corpus (SPEC §3.5.5) is duplicated in two locations: the canonical at `crates/mk-codec/tests/vectors/v0.1.json` (mk-codec's own tests consume this) and the working copy at `crates/mk-cli/src/cmd/v0.1.json` (mk-cli's `mk vectors` subcommand `include_str!`s this at compile time). When the corpus changes, both copies must sync manually; no compile-time or CI gate verifies byte-equality today.
- **Why deferred:** Quick crates.io publishability fix at commit `135651f`. Proper fixes require coordinated re-publish: (a) factor into `mk-codec` public API as a feature-gated `pub const VECTORS_V0_1_JSON: &str = ...`, then bump mk-codec + mk-cli; OR (b) feature-gate the `mk vectors` subcommand off by default in published `mk-cli` builds (matching the existing `gen-vectors` feature precedent in `mk-codec/Cargo.toml` for `gen_mk_vectors`).
- **Status:** `resolved 33f2ca2` — mk-codec 0.3.0 promotes the corpus to `pub mod test_vectors::V0_1_JSON` (canonical file moved to `crates/mk-codec/src/test_vectors/v0.1.json`); mk-cli 0.3.2 re-imports the const, dropping the 34KB working copy. Both crates published to crates.io 2026-05-15.
- **Tier:** `v1+`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` entry `md-cli-vectors-manifest-inlined` — same pattern in the md-cli side; resolved at descriptor-mnemonic commit `8a52bed` (md-codec 0.33.0 + md-cli 0.5.1).

### `bip-vector-adoption-v0_8` — cross-repo cycle: BIP-vector adoption v0.8.0 (no-scope companion)

- **Surfaced:** 2026-05-13. Cycle SPEC at `mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md`. Plan at `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`. R1 review at `mnemonic-toolkit/design/agent-reports/v0_8_0-phase-0-spec-plan-r1.md`.
- **Where:** No `mk-codec` / `mk-cli` source change. mk-codec's v0.7.1 audit matrix at `design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md` covered the BIP coverage relevant to this crate (BIP-32 xpub derivation delegated to `bitcoin v0.32`); no new gap was surfaced for this cycle by the cross-repo BIP-vector survey at `mnemonic-toolkit/design/agent-reports/v0_8_0-cross-repo-bip-vector-survey.md`.
- **What:** This entry exists for cross-repo audit symmetry per SPEC §5 ("`mnemonic-key` is OUT-OF-SCOPE for v0.8.0 — no new gap; mk-codec's v0.7.1 matrix carries the only relevant coverage and continues to delegate xpub-format / BIP-32 derivation to `bitcoin v0.32`. The `bip-vector-adoption-v0_8` entry in mnemonic-key reads: *'no scope for this cycle; included for cross-repo audit symmetry.'*"). Closes when the cycle's audit-matrix successor doc lands at `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` (Phase 4; even with no coverage delta, the no-op v0.8.0 matrix file replicates the v0.7.1 content with a SUPERSEDED header).
- **Status:** `resolved 6d43115` — PR #10 merged (docs-only no-scope companion; mk-codec carried no source change so no tag). Companion sibling-repo tags: ms-codec-v0.1.2 (mnemonic-secret 527c9c7), md-codec-v0.32.1 (descriptor-mnemonic ef00e07), mnemonic-toolkit-v0.9.1 (f036737).
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md`, `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md` — same `bip-vector-adoption-v0_8` short-id in each.

### `md-mk-private-key-surface-watch` — reopen md/mk Cycle A participation if this repo grows a private-key surface

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 0 R3 architect-review I-R3-4 fold (drop md/mk symmetry-stubs); opened as a standalone tracker entry per Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-md-mk` class. Primary tracker entry in `mnemonic-toolkit/design/FOLLOWUPS.md`.
- **Where:** This repo (`mk-codec` + `mk-cli`). Currently holds xpub / wallet-key material only — no private-key buffer.
- **What:** v0.9.0 Cycle A's secret-memory hygiene work (toolkit + ms repos; tags `mnemonic-toolkit-v0.9.2`, `ms-codec-v0.1.3`, `ms-cli-v0.2.2`, shipped 2026-05-13) dropped the no-scope-symmetry matrix stubs originally planned for md/mk repos because they have no secret material to audit. If this repo later gains a private-key surface (e.g., a future mk-codec xprv passthrough), this FOLLOWUP fires and Cycle A's hygiene discipline (Zeroizing + SAFETY anchors + matrix delta) reopens for mk.
- **Why deferred:** No secret material to audit today.
- **Status:** `open` (monitoring)
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md` (primary tracker), `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md` — same `md-mk-private-key-surface-watch` short-id.

### `manual-cli-surface-mirror` — `mk-codec` public-API changes must mirror to the toolkit-side user manual

- **Surfaced:** 2026-05-07, m-format-star user manual v0.1 release in `bg002h/mnemonic-toolkit` (`manual-v0.1.0` tag; toolkit PR #1).
- **Where:** Cross-repo coordination only; no `mk-codec` source change required at filing time. Future public-API additions (new `pub` items, removed re-exports, signature changes, `#[non_exhaustive]` field additions to `KeyCard`) must touch the manual side in lockstep — for v0.2+ this means both `mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md` (flag surface) and the archived Rust API reference at `mnemonic-key/docs/MK_CODEC_RUST_API.md` (with the toolkit-side `44-mk-codec-rust.md` deleted in toolkit PR 2 of the v0.2 mk-cli cycle).
- **What:** v0.1 of the m-format-star user manual mirrored the *Rust API* surface only because `mk-codec` was library-only. With v0.2 the manual mirrors the `mk-cli` flag surface; the Rust API reference is archived in this repo at `docs/MK_CODEC_RUST_API.md`. Both surfaces are bound by the same lockstep-mirror invariant. The manual's `tests/lint.sh flag-coverage` step grep-gates `mk` flags from v0.2 onward (toolkit PR 2 commit 2a). **Companion:** primary entry `manual-cli-surface-mirror` in `mnemonic-toolkit/design/FOLLOWUPS.md`; sibling companions in `descriptor-mnemonic/design/FOLLOWUPS.md` and `mnemonic-secret/design/FOLLOWUPS.md`.
- **Why filed:** the manual is a separate artifact (its own `manual-v*` versioning); without an explicit mirror invariant, sibling-side API changes would silently drift the manual.
- **Status:** `open` (mirror invariant active for the lifetime of `mnemonic-toolkit/docs/manual/`)
- **Tier:** `cross-repo`

### `mk-cli-v0_2-toolkit-docs-mirror` — toolkit-side docs mirror for mk-cli v0.2 (companion)

- **Surfaced:** 2026-05-08, mk-cli v0.2 cycle (this repo's branch `mk-cli/v0_2`, plan `concurrent-cooking-scone`).
- **Where:** `mnemonic-toolkit` repo branch `mk-cli/docs-v0_2`. Toolkit PR 2 lands the manual chapter `docs/manual/src/40-cli-reference/44-mk-cli.md` (~600 lines), deletes `44-mk-codec-rust.md` (archived in this repo at `docs/MK_CODEC_RUST_API.md` in commit `1c74c70`), extends `tests/lint.sh` + `Makefile` for the 4-CLI shape, and rebuilds the manual / quickstart / ultraquickstart PDFs. Toolkit PR 2 → manual-v0.1.7 + quickstart-v0.1.4 + ultraquickstart-v0.1.2 patch-tag releases.
- **What:** This repo's PR 1 (commit `77bdb2f` adds the binary; commit `1c74c70` archives the Rust API doc; commit on this entry pins the v0.2 manual-mirror language in `CLAUDE.md`). The companion toolkit PR 2 lands the user-facing chapter and the lint-gate update so the four-CLI parity invariant holds end-to-end. **Companion:** primary entry `mk-cli` resolution in `mnemonic-toolkit/design/FOLLOWUPS.md` (moves from "Open" → "Resolved/Closed" with citation `Resolved by mk-cli-v0.2.0`).
- **Why filed:** Same lockstep-pattern as ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility. Filing here makes it discoverable from this repo's tracker; closes when toolkit PR 2 merges.
- **Status:** `open` (closes when toolkit PR 2 lands)
- **Tier:** `cross-repo`

### `ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility` — ms1 v0.1 wire-format plan needs revision (BIP-93 codex32 length-bracket conflict with locked `0x00` prefix byte)

- **Surfaced:** 2026-05-03 pre-SPEC spike in `mnemonic-secret` repo (in conversation; before ms1's SPEC drafted). Companion: primary entry of same id in `mnemonic-secret/design/FOLLOWUPS.md`; mirror in `descriptor-mnemonic/design/FOLLOWUPS.md`.
- **Where:** Cross-repo coordination only; no md1/mk1 wire-format change required. Affects: ms1 v0.1 SPEC (not yet drafted) and downstream `mnemonic-toolkit` (when it lands) — both will need to know which payload kinds ms1 v0.1 actually emits (currently locked as {seed, entr, xprv}, likely to narrow).
- **What:** ms1 v0.1's `0x00` reserved-prefix byte (designed to make the v0.2 share-encoding migration non-breaking for v0.1 strings) pushes 64-B BIP-32 master seeds to 65-B payloads — one byte past BIP-93 codex32's long-code max (rust-codex32 v0.1.0 rejects with `InvalidLength(128)`). `xprv` (78 B) was never inside any BIP-93 bracket. Likely remediation: narrow ms1 v0.1 to `entr`-only payloads; defer `seed`/`xprv` to v0.2+ with their own framing. Awaiting user direction in the ms1 session.
- **Why deferred:** ms1-internal SPEC decision; no mk1 source change. Logged here so future sessions in this repo don't re-litigate the four-format-star payload assumptions when toolkit work begins.
- **Status:** `resolved 2026-05-03 — ms1 v0.1 shipped with Option A remediation: v0.1 narrowed to entr-only; seed/xprv deferred to v0.2+ with own framing. ms-codec v0.1.0 release commit ab374ed in mnemonic-secret; primary FOLLOWUPS entry there records full mechanics. mk1 source unchanged — this entry was a coordination flag only. Four-format-star payload assumptions for downstream toolkit work: ms1 emits entr only in v0.1.`
- **Tier:** `cross-repo`

### `mc-codex32-extraction-retired-2026-05-03` — original shared-crate plan retired in favor of ms1 adopting `rust-codex32` directly

- **Surfaced:** 2026-05-03, ms1 plan-mode brainstorm in the `descriptor-mnemonic` repo (plan file: `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md`). Companion: same-id entry in `descriptor-mnemonic/design/FOLLOWUPS.md`.
- **Where:** Cross-repo design / process. Affects `mnemonic-key/CLAUDE.md` (line 38 retirement language already updated 2026-05-03), `descriptor-mnemonic/CLAUDE.md` (mirrored), `descriptor-mnemonic/design/DECISIONS.md` D-13 (still records "fork-now-refactor-later" — historically accurate, no change needed), and the future cross-repo `PATTERNS.md` doc that will replace the shared-crate plan.
- **What:** Closure Q-9 originally specified that md1 and mk1 would extract their shared BIP-93 BCH plumbing into a third sibling crate `mc-codex32` once both formats hit v1.0 with cross-validated conformance vectors. With the addition of a third sibling format ms1 (HRP `ms`, repo `bg002h/mnemonic-secret`) that adopts BIP-93 codex32 *directly* via Andrew Poelstra's `rust-codex32` crate, the calculus changed: md1 and mk1 use HRP-mixed BCH with per-format target residues that are NOT upstreamable to `rust-codex32`'s vanilla BIP-93 implementation, and ms1 doesn't need them either. There is no longer shared code worth extracting — only a shared *pattern* (HRP-mixed BCH with per-format target residue) that is better captured as documentation. md1↔mk1 BCH plumbing stays forked indefinitely; the pattern will be documented in a future cross-repo `PATTERNS.md`.
- **Why deferred:** Decision was locked during ms1 plan-mode r1..r5 review convergence on 2026-05-03; CLAUDE.md updates landed in lockstep. The `PATTERNS.md` doc itself is non-blocking and can be drafted opportunistically when the next BCH-plumbing concern surfaces in either repo.
- **Status:** `wont-do — superseded by ms1 adopting rust-codex32 directly (2026-05-03 cross-repo decision)`. CLAUDE.md retirement language landed same day.
- **Tier:** `cross-repo`

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
- **Status:** `resolved by md-codec-v0.9.0 + mk-codec-v0.2.0`. md-codec v0.9.0 ([release](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0)) added the 0x16 row to md1's path-dictionary table. mk-codec v0.2.0 closed the parallel gap on the mk1 side: added `(0x16, "m/48'/1'/0'/1'")` to `STANDARD_PATHS`, regenerated the corpus with V18 exercising the indicator, rolled `GENERATOR_FAMILY` to `"mk-codec 0.2"` (Q-10: minor bumps roll the family token). Wire-additive: v0.1.x decoders reject v0.2-emitted 0x16 strings.
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
- **Status:** `resolved 901596a` (v0.2.0 Phase 1). Took option 1 with a small adaptation: instead of deriving `EnumIter` directly on `mk_codec::Error` (parameterized variants make construction-via-strum awkward), used a hand-written mirror enum `ErrorVariantName` at `crates/mk-codec/tests/error_coverage.rs` matching md-codec's exact precedent. Two tests gate the corpus: `every_error_variant_is_exercised_or_explicitly_exempt` (forward direction) and `every_negative_vector_maps_to_a_known_variant` (reverse direction; catches typos / stale vectors).
- **Tier:** `v0.2-nice-to-have`

### `vector-corpus-dictionary-coverage` — v0.1 corpus exercises only 4 of 13 path-dictionary entries

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 6 review (M-1, commit 053a54c).
- **Where:** `crates/mk-codec/tests/vectors/v0.1.json` (V1..V8 fixture set).
- **What:** The v0.1 vector corpus exercises std-table indicators 0x03 (BIP 84), 0x05 (BIP 48 segwit-v0 mainnet), 0x07 (BIP 87), and 0x15 (BIP 48 testnet) plus the 0xFE explicit-path codec. Missing: 0x01 (BIP 44), 0x02 (BIP 49), 0x04 (BIP 86), 0x06 (BIP 48 nested-segwit mainnet), and the testnet entries 0x11, 0x12, 0x13, 0x14, 0x17. A third-party encoder could pass all 8 v0.1 vectors while still mishandling BIP 44/49/86 mainnet inputs.
- **Why deferred:** The internal encoder unit test `bytecode/path::round_trip_all_standard_paths` already cycles every dictionary entry; the gap is in the cross-implementation conformance corpus, not in encoder correctness. Closing the gap is straightforward (one fixture per missing indicator) but expands the corpus from 8 to ~14 vectors; defer to the pre-bip-submission corpus expansion.
- **Status:** `resolved 2417401 + fd6a407` (v0.1.1 Phase 2 + v0.2.0 Phase 2). v0.1.1 added V9..V17 covering 9 of the 10 missing indicators; v0.2.0 added V18 for 0x16 after md-codec v0.9.0 / mk-codec v0.2.0 closed the wire-additive parallel gap. Corpus now exercises every closure-locked path-dictionary entry (14 std-table + 0xFE explicit).
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

### `path-dictionary-mirror-stewardship` — formalize mk1↔md1 path-dictionary inheritance contract (retired in mk-codec v0.2.2)

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 2 BIP review open observation (commit 4728230).
- **Where:** mk1 SPEC §3.5; mk1 BIP §"Origin path encoding"; md1 BIP §"Path dictionary".
- **What:** mk1's path dictionary was originally contractually identical to md1's `Tag::SharedPath` table. If md1 allocated new dictionary entries (e.g., closing the 0x16 gap, or adding new BIP-style accounts in future md1 revisions), mk1 was to inherit the allocation by the byte-for-byte mirror clause. The contract was formalized in md-codec v0.9-p3 (commit `abbec54`).
- **Why deferred:** Originally process / stewardship concern, not a v0.1 release blocker.
- **Status:** `resolved 6509f8e` (mk-codec v0.2.2 docs-only patch). md-codec v0.11 (per [`descriptor-mnemonic/design/SPEC_v0_11_wire_format.md`](https://github.com/bg002h/descriptor-mnemonic/blob/main/design/SPEC_v0_11_wire_format.md) §1.4 — "Wire-layer dictionaries (path, use-site-path, shape). Considered and rejected for architectural cleanliness") dropped the md1 path dictionary entirely; md1 now encodes paths explicitly via `OriginPath`. The mirror invariant therefore has no md1-side anchor: there is nothing left to mirror. mk-codec v0.2.2 retired the mirror clause across the mk-codec source doc-comments (`crates/mk-codec/src/bytecode/path.rs::STANDARD_PATHS` rustdoc + module docs), `design/SPEC_mk_v0_1.md` §3.5 (added "Path dictionary divergence note (v0.2.2)"), and `bip/bip-mnemonic-key.mediawiki` §"Origin path encoding". mk1's standard-table dictionary is now documented as **mk1-internal** (standalone). md-codec v0.11+ does not carry a sibling table; the `descriptor-mnemonic/CLAUDE.md` cross-repo coordination block notes the retirement under "Recently retired".
- **Tier:** `cross-repo`

### `mnemonic-gui-schema-mirror` — companion to `bg002h/mnemonic-gui` schema gate

- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`; CI gate at `.github/workflows/schema-mirror.yml`.
- **Where:** This CLI's clap-derive `Args` blocks for every subcommand the GUI surfaces (v0.1: `mk inspect`; v0.2+: encode/decode/verify/test-vectors).
- **What:** The `mnemonic-gui` GUI mirrors this CLI's clap-derive flag surface at pinned tag `mk-cli-v0.2.0`. Any flag add / remove / rename / `conflicts_with` / `required_unless_present_any` change in this repo's CLI surface must land in lockstep with a companion `mnemonic-gui` PR that bumps the schema + the `pinned-upstream.toml` tag for this CLI. The `mnemonic-gui` CI gate runs `cargo install --locked --git <this-repo> --tag <pin>` + `cargo test --test schema_mirror`, so drift surfaces as a CI failure.
- **Status:** `open` (mirror-invariant; tracking only — every flag-surface PR carries this lockstep work).
- **Tier:** `v1 / mirror-invariant`

### `error-bchuncorrectable-doc-says-8-for-long` — `Error::BchUncorrectable` doc reads "8 for long" but both codes are t=4

- **Surfaced:** 2026-05-29, mk-codec test-hardening cycle (theme 2 T2-doc).
- **Where:** `crates/mk-codec/src/error.rs:56` — `/// substitution capacity (4 for regular, 8 for long).`
- **What:** The parenthetical reads as a correction count, but the long code `BCH(108,93,8)` has `t = 4` (the `8` is the designed minimum distance / syndrome count). Both regular and long correct up to 4 substitutions (`string_layer/bch.rs:376,451`; `bch_decode.rs:566` rejects `deg > 4`). Reword to "(t = 4 for both; the trailing 8 in BCH(•,•,8) is the min-distance, not the correction count)".
- **Why deferred:** doc-only; no behavior impact. Fold into any error-surface touch.
- **Status:** `open`
- **Tier:** `docs`

### `mk1-depth-child-lossless-by-construction-unenforced` — encoder drops xpub.depth/child_number and reconstructs from path WITHOUT validating agreement

- **Surfaced:** 2026-05-29, mk-codec test-hardening cycle (theme-1 strategy design; `design/SPEC_mk_codec_test_hardening.md` §1.1).
- **Where:** `crates/mk-codec/src/bytecode/xpub_compact.rs:4` (drops depth/child), `:85-106` (`reconstruct_xpub` rebuilds them from `origin_path`), `bytecode/encode.rs` (`XpubCompact::from_xpub` silently drops). SPEC `design/SPEC_mk_v0_1.md:263,301` claims "lossless by construction" + removes `XpubDepthMismatch`.
- **What:** The "lossless by construction" claim holds ONLY when the caller passes `xpub.depth == origin_path.len()` and `xpub.child_number == origin_path.last()`. Nothing enforces this; a depth-4 xpub + 3-component path silently round-trips to a DIFFERENT xpub. Decide: (a) re-introduce encode-time `XpubDepthMismatch` (genuinely lossless), OR (b) document the lossy contract + pin it with a test. The toolkit compensates downstream (`mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs:494-503` depth check) — option (a) would let it drop that.
- **Why deferred:** behavior/contract decision (likely MINOR bump + toolkit coordination), out of the test-only test-hardening cycle's scope. The cycle's theme-1 strategy sidesteps it by building the xpub from the path (depth/child consistent by construction).
- **Status:** `open`
- **Tier:** `v0.4`
- **Companion:** `mnemonic-toolkit` FOLLOWUP `mk1-depth-child-compensating-check-watch`.
