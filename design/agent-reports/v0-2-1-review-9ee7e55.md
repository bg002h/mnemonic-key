# mk-codec v0.2.1 review — commit `9ee7e55`

- **Reviewer:** Senior code-reviewer subagent (Opus)
- **Branch:** `feature/v0.2.1`
- **Commit reviewed:** `9ee7e55` ("docs(mk-codec v0.2.1): close v0.2.0 Phase 2-4 review deferred suggestions")
- **Scope:** Doc-only patch closing S-1..S-4 deferred from `design/agent-reports/v0-2-0-phase-2-4-review-fd6a407.md`
- **Verdict:** **APPROVED — proceed to tag + push + GitHub release.** No critical or important findings; one micro-observation noted under Suggestions; no blockers.

---

## Top-line conclusion

All four deferred Suggestions land cleanly. Wire format, corpus, and SHA pin are byte-identical to v0.2.0, as the patch advertises. Path-dictionary table in the BIP matches `STANDARD_PATHS` byte-for-byte and matches md1's `Tag::SharedPath` table byte-for-byte (the byte-for-byte-mirror contract). The CHANGELOG amendment is well-flagged with explicit retroactive-backfill provenance in both the `[0.2.0]` and `[0.2.1]` entries — defensible practice. Tests pass: 149 unit + 2 error_coverage + 3 round_trip + 3 vectors = **157**, matching the commit message claim. Generator binary rustdoc no longer pretends the corpus is "v0.1." VECTOR_FILE comment correctly documents the filename-vs-family-token convention.

The patch is publishable as-is.

---

## S-1 — CHANGELOG migration pointer (Phase 1) — **PASS**

**Verified:**

1. The backfilled paragraph (CHANGELOG.md L121–128) is correctly placed under `[0.2.0]` Notes, *not* under `[0.2.1]`. Mirrors the v0.1.1 Notes pattern (L195–198) byte-for-byte in spirit (SHA-pin update + vector-count migration + family-token note).
2. The `[0.2.1]` `Changed` subsection (L25–29) explicitly cross-references the backfill: "*backfilled the missing cross-implementation SHA-pin migration pointer that should have been in the original v0.2.0 release notes (parallel to v0.1.1's pattern)*" — readers will understand the retroactive amendment.
3. The retroactive-amendment provenance is doubly noted: the `[0.2.0]` Notes paragraph itself ends with a parenthetical pointing forward to `[0.2.1]` (L126–128: "*(Backfilled in v0.2.1; see `[0.2.1]` below for details — the original v0.2.0 release notes omitted this migration pointer.)*"), and the `[0.2.1]` Notes section reiterates the convention (L45–49). Future readers cannot mistake this for a synchronously-shipped v0.2.0 note.
4. The backfilled SHA matches the v0.2.0 corpus pin: `ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff`. Verified against `tests/vectors.rs::V0_1_SHA256` (unchanged) and against `sha256sum tests/vectors/v0.1.json` (matches).

**On the policy of editing a shipped CHANGELOG entry:** Defensible. The amendment-with-provenance approach (Keep-a-Changelog accepts this; many projects with formal release notes do as well) is preferable to the alternatives:

- Leaving the gap (cross-impl readers continue to lack the migration pointer).
- Adding the pointer *only* in `[0.2.1]` (works for new readers but doesn't help anyone who landed on `[0.2.0]` via a permalink or git-tag browse).
- Cutting a v0.2.0.1 hotfix (heavier process for what's literally a missing paragraph).

The chosen approach makes the v0.2.0 entry self-contained for future browsers while preserving the audit trail. Both the GitHub-release artifact and the git tag for v0.2.0 remain unchanged; only the CHANGELOG file content evolves under the v0.2.1 release. This is the right call.

---

## S-2 — VECTOR_FILE comment (Phase 2) — **PASS**

**Verified `crates/mk-codec/tests/vectors.rs` L43–55:**

- Comment correctly states the filename is "intentionally stable across minor-bump family-token rolls."
- Cross-reference to closure Q-10 ("minor-version bumps roll the token; patches don't") is accurate.
- Concrete examples named: v0.2+ → `family_token: "mk-codec 0.2"`, v0.1.x → `"mk-codec 0.1"`, both at the same path. Verified against the actual corpus: `grep family_token tests/vectors/v0.1.json` → `"mk-codec 0.2"`. Matches `consts::GENERATOR_FAMILY = "mk-codec 0.2"`.
- "md-codec follows the same convention for its own vector file" claim is accurate — md-codec's vector corpus also lives at a stable filename across its v0.x family-token rolls.
- Comment is rustdoc (`///`) on `const VECTOR_FILE`, so it surfaces in IDE hover and documentation generation. Good placement.

No nits.

---

## S-3 — gen_mk_vectors rustdoc (Phase 3) — **PASS**

**Verified `crates/mk-codec/src/bin/gen_mk_vectors.rs` L1–8:**

- The "v0.1 vector corpus" version specifier is gone. New L1: "Generator for the canonical mk-codec vector corpus." (no version qualifier — appropriate, since the binary generates whatever family GENERATOR_FAMILY names).
- L3: cites `crate::consts::GENERATOR_FAMILY` as the source-of-truth. Backtick-quoted `"mk-codec X.Y"` template with the Q-10 minor-vs-patch convention spelled out.
- L5–8: cross-reference to `tests/vectors.rs::VECTOR_FILE` for the filename convention. Tight cross-link; readers landing in either file find the other.

**Sweep for other stale "v0.1" / "v0.1.x" references in source/tests** (looked at 30+ matches across `src/` and `tests/`):

All remaining `v0.1` references are correct usage — they refer to the wire-format version (which IS v0.1, locked at `feature/v0.1.0-implementation`) or to historical patch versions (v0.1.0, v0.1.1) that introduced specific features. Examples checked:

- `lib.rs` L3 "v0.1 implementation in progress" — wire-format reference; correct.
- `consts.rs` L1 "Locked constants for `mk1` per design/SPEC_mk_v0_1.md v0.1" — SPEC reference; correct.
- `error.rs` L100, L104, L148 — all reference v0.1 wire-format invariants; correct.
- `bytecode/header.rs` L6–13 — bit-allocation comments referencing v0.1 wire format; correct.
- `string_layer/*.rs` — references to v0.1 emit policy / wire constraints; all correct.
- `bytecode/path.rs` L9, L12, L49, L268 — references to v0.1.x as the *implementation patch range* before 0x16 was added; all correct (and consistent with the BIP §"Origin path encoding" history paragraph).
- `gen_mk_vectors.rs` L64, L114, L239, L242, L342, L358, L433, L595, L693, L743, L747, L772 — all reference either v0.1.x patch history (V9..V17 added in v0.1.1) or v0.1 wire-format constraints. Correct.

S-3's spirit was specifically the *generator binary's module rustdoc* claiming the binary produces a "v0.1 corpus" (when it now produces a v0.2 corpus). That single claim is fixed. No collateral stale references remain to be caught.

---

## S-4 — BIP §"Origin path encoding" Case A table (Phase 4) — **PASS**

**Verified `bip/bip-mnemonic-key.mediawiki` L275–315.**

### Byte-for-byte vs `STANDARD_PATHS`

Cross-checked all 14 rows against `crates/mk-codec/src/bytecode/path.rs::STANDARD_PATHS` (L34–51):

| Indicator | STANDARD_PATHS | mk1 BIP table | Match |
|-----------|---------------------|--------------------|-------|
| `0x01` | `m/44'/0'/0'` | `m/44'/0'/0'` | ✓ |
| `0x02` | `m/49'/0'/0'` | `m/49'/0'/0'` | ✓ |
| `0x03` | `m/84'/0'/0'` | `m/84'/0'/0'` | ✓ |
| `0x04` | `m/86'/0'/0'` | `m/86'/0'/0'` | ✓ |
| `0x05` | `m/48'/0'/0'/2'` | `m/48'/0'/0'/2'` | ✓ |
| `0x06` | `m/48'/0'/0'/1'` | `m/48'/0'/0'/1'` | ✓ |
| `0x07` | `m/87'/0'/0'` | `m/87'/0'/0'` | ✓ |
| `0x11` | `m/44'/1'/0'` | `m/44'/1'/0'` | ✓ |
| `0x12` | `m/49'/1'/0'` | `m/49'/1'/0'` | ✓ |
| `0x13` | `m/84'/1'/0'` | `m/84'/1'/0'` | ✓ |
| `0x14` | `m/86'/1'/0'` | `m/86'/1'/0'` | ✓ |
| `0x15` | `m/48'/1'/0'/2'` | `m/48'/1'/0'/2'` | ✓ |
| `0x16` | `m/48'/1'/0'/1'` | `m/48'/1'/0'/1'` | ✓ |
| `0x17` | `m/87'/1'/0'` | `m/87'/1'/0'` | ✓ |

All 14 paths match.

### `0x16` row attribution

The `0x16` row carries the inline note `(added in mk-codec v0.2.0)` — correctly attributed. It's the only row with a version-history annotation, which is appropriate (the other 13 entries shipped in v0.1.0).

### Cross-check vs md1's `Tag::SharedPath` table

Verified against `descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki` L329–360 ("Path dictionary"). All 14 paths mk1 lists match md1's table byte-for-byte. md1's "Notes" column carries slightly more BIP-script-form detail (e.g., "BIP 49 mainnet (P2SH-P2WPKH)" vs mk1's "BIP 49 mainnet"); the path strings agree, which is what the byte-for-byte-mirror contract requires. The "BIP family" column wording is fine — it conveys the same family identification with less script-construction detail, which is appropriate for mk1's xpub-only context (where script construction is downstream of the recovered xpub anyway).

The mirror contract requires path equality, not Notes-column equality, so the wording divergence is allowed.

### Prose around the table

- L277 "Mirrors MD's `Tag::SharedPath` dual-mode encoding." — accurate (preserved from pre-table version).
- L279 prose-into-table cite: "mirrors MD's `Tag::SharedPath` table byte-for-byte under the path-dictionary-mirror-stewardship contract" — accurate.
- L313 (post-table summary): "Mainnet entries occupy `0x01`..`0x07`; testnet entries occupy `0x11`..`0x17` (the upper-nibble bit identifies testnet). The 14-entry table above is the canonical source of truth; future entries added to MD's `Tag::SharedPath` table are auto-inherited by mk1 under the mirror-stewardship contract." — accurate, useful summary.
- L315 history paragraph: "Note on indicator `0x16` (history): mk1 v0.1.x reserved `0x16` because md1's path dictionary at the time omitted the testnet companion to mainnet `0x06`. md-codec v0.9.0 closed the parallel gap on the md1 side; mk-codec v0.2.0 closed the gap on the mk1 side under the mirror-stewardship contract. The change is wire-additive: v0.1.x decoders reject `0x16` with `Error::InvalidPathIndicator(0x16)`; v0.2+ decoders accept and resolve to the BIP 48 testnet nested-segwit path." — accurate, preserved from pre-table version with a minor rewording (drops "adding `(0x16, m/48'/1'/0'/1')` to the standard table" since the table now states this directly). Better.

No prose contradicts the table; the table is self-consistent; the history paragraph remains accurate post-reshape.

### Reserved-indicator paragraph

L329 "Indicators `0x00`, `0x08`–`0x10`, `0x18`–`0xFD`, and `0xFF` are reserved and MUST NOT be emitted by encoders. Decoders MUST reject reserved indicator bytes (`Error::InvalidPathIndicator`)." — preserved from pre-table version, still accurate. Reserved set is the complement of the 14 + `0xFE` table.

---

## No-regression checks — **PASS**

1. `crates/mk-codec/Cargo.toml` version: `0.2.0` → `0.2.1`. Verified.
2. `Cargo.lock` mk-codec entry: `0.2.0` → `0.2.1`. Verified.
3. `tests/vectors.rs::V0_1_SHA256` = `ebd8f34d8d52896e07e1faef995f18ffa61d42e2a048fb2a8c11e67f120d78ff`. **Unchanged from v0.2.0.** Matches `sha256sum tests/vectors/v0.1.json` output.
4. `consts.rs::GENERATOR_FAMILY` = `"mk-codec 0.2"`. **Unchanged from v0.2.0.** Verified — Q-10 says patch bumps don't roll the family token; this patch correctly does not.
5. `cargo test -p mk-codec` — all 157 tests pass (149 + 2 + 3 + 3). Zero ignored, zero failed.
6. Wire format byte-identical: no source code change in `src/` outside the `gen_mk_vectors` module rustdoc (a doc-only edit that doesn't affect generator output). Cross-checked the diff: only `bin/gen_mk_vectors.rs` L1–8 module rustdoc changed in `src/`; no behavior change.

---

## CHANGELOG `[0.2.1]` completeness — **PASS**

Cross-referenced the `[0.2.1]` Added/Changed/Notes sections (CHANGELOG.md L8–49) against the actual file diffs in commit 9ee7e55:

| Diff hunk | CHANGELOG entry | Match |
|---|---|---|
| `bip/bip-mnemonic-key.mediawiki` (Case A table reshape) | Added — "BIP §'Origin path encoding' Case A — full path-dictionary table inline (14 rows mirroring md1's `Tag::SharedPath`)" | ✓ |
| CHANGELOG.md `[0.2.0]` Notes amendment | Changed — "CHANGELOG `[0.2.0]` Notes — backfilled the missing cross-implementation SHA-pin migration pointer" | ✓ |
| `crates/mk-codec/tests/vectors.rs` VECTOR_FILE comment | Changed — "`crates/mk-codec/tests/vectors.rs::VECTOR_FILE` — added a comment documenting the filename-vs-family-token convention" | ✓ |
| `crates/mk-codec/src/bin/gen_mk_vectors.rs` rustdoc | Changed — "`crates/mk-codec/src/bin/gen_mk_vectors.rs` module rustdoc — dropped the misleading 'v0.1 vector corpus' version specifier" | ✓ |
| `Cargo.toml` version bump | (implicit; version bumps don't typically warrant their own bullet — this is consistent with v0.1.1 and v0.2.0 Changelog patterns) | acceptable |
| `Cargo.lock` version bump | (auto-regenerated; mechanical) | acceptable |

All file-level diffs map to a CHANGELOG bullet (or are mechanical artifacts that conventionally don't get one). No unclaimed changes; no claims without backing diff.

---

## Cross-repo coordination

Reviewed `descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki` for any md1-side update that mk1's v0.2.1 doc patch should mirror or surface:

- md1's `Tag::SharedPath` table is unchanged from when mk1 v0.2.0 mirrored it (still 14 path entries + `0xFE` + `0xFF`). No drift.
- The `Cross-format inheritance` paragraph in md1's BIP (L362) calls out the contractual byte-for-byte sharing with mk1; that paragraph is unchanged and remains accurate after mk1's BIP table reshape (the table data is still byte-identical to md1's).
- No companion FOLLOWUPS entry on the md1 side needs flipping based on this v0.2.1 patch (mk1's S-1..S-4 were entirely mk-codec internal — CHANGELOG, comment, rustdoc, BIP §Case A — none affect md1's surface).

No cross-repo update is needed for v0.2.1.

---

## FOLLOWUPS audit

Reviewed `design/FOLLOWUPS.md` for any entries that should reference v0.2.1 closure:

- The four S-1..S-4 suggestions were inline-deferred-to-user-call inside the v0.2.0 Phase 2-4 review report; they were never promoted to standalone FOLLOWUPS entries. v0.2.1 closes them at the source (the review report); no FOLLOWUPS edits are needed.
- The single existing FOLLOWUPS reference to `fd6a407` is for the unrelated `vector-corpus-dictionary-coverage` entry (already resolved by V18 at v0.2.0); not affected by v0.2.1.
- No `pre-bip-submission` or `cross-repo` tier entries are tied to S-1..S-4.

---

## Suggestions (nice-to-have, not blocking)

**SUG-1 — `[0.2.1]` CHANGELOG entry release date.** The header reads `## [0.2.1] — 2026-04-30`, but commit 9ee7e55's author timestamp is `Wed Apr 29 20:51:53 2026 -0700`. If the release artifact (git tag, GitHub release) is created today (2026-04-29 PT), the date will mismatch by one day. Two options at tag time:

1. Tag today (2026-04-29 PT) and adjust the CHANGELOG header to `## [0.2.1] — 2026-04-29`.
2. Tag tomorrow (2026-04-30 PT) and leave the header as-is.

Either is fine — this is a tag-time housekeeping detail, not a content issue. v0.2.0's CHANGELOG entry has `2026-04-30` and that release was tagged 2026-04-29 (commit `fd6a407`), so the precedent is "use the date the entry was written, not the date the tag landed." The current `[0.2.1]` header is consistent with that precedent. No action needed unless the release window stretches.

**SUG-2 — Optional cross-link in BIP table caption.** The post-table summary paragraph (L313) could optionally cross-link to the implementation source ("see `crates/mk-codec/src/bytecode/path.rs::STANDARD_PATHS`"). Current text is fine without it; readers of the BIP draft are not expected to read the reference implementation's source. Skip unless the BIP-submission audit specifically asks for source-tree cross-references. Defer.

Neither suggestion blocks the release.

---

## Recommendation

**Tag, push, and cut the GitHub release for v0.2.1.** The patch is doc-only, behavior-preserving, well-documented, and addresses every deferred Suggestion from the v0.2.0 review with high accuracy. The CHANGELOG amendment-with-provenance approach is defensible and cleanly executed; the BIP path-dictionary table mirrors `STANDARD_PATHS` and md1's `Tag::SharedPath` byte-for-byte; the gen_mk_vectors rustdoc and VECTOR_FILE comment remove the stale version-specifier and document the filename-vs-family-token convention with sufficient cross-linking; tests pass at 157.

No critical, no important. SUG-1 (date housekeeping at tag time) is a 5-second touch-up if the tag lands on a different day than the CHANGELOG header expects; SUG-2 is purely optional cross-linking.

If any one of S-1..S-4 had landed slightly off (wrong path string in the BIP table, wrong SHA in the migration pointer, etc.), that would be an "important — fix before tag" finding. None did.

---

## Resolution notes (post-tag)

Disposition of the two non-blocking Suggestions, recorded after v0.2.1 shipped:

- **SUG-1 (CHANGELOG date housekeeping):** **wont-fix.** The 2026-04-30 date in the `[0.2.1]` entry header followed v0.2.0's precedent (entry-write-date, not tag-land-date). v0.2.1 shipped on 2026-04-29 PT / 2026-04-30 UTC; the date is internally consistent with v0.2.0's same-shape header. Closing as wont-fix per project precedent.

- **SUG-2 (BIP table cross-link):** **applied.** Added inline reference-implementation cross-link to the post-table summary paragraph in `bip/bip-mnemonic-key.mediawiki` §"Origin path encoding" — `Reference-implementation cross-link: crates/mk-codec/src/bytecode/path.rs::STANDARD_PATHS`. Lets a BIP reader follow a single hop to the source-of-truth dictionary in the reference implementation. Doc-only edit on `main`; no version bump needed (BIP draft is unversioned relative to the crate).
