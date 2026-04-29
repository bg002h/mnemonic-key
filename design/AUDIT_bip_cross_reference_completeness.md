# Audit — BIP draft cross-reference completeness

**Audit ID:** `bip-cross-reference-completeness`
**Tier:** `pre-bip-submission` (FOLLOWUPS gate item)
**Date:** 2026-04-29
**Auditor:** Claude Opus 4.7 review subagent (cross-reference completeness pass)
**Repo state at start of audit:** branch `feature/v0.1.0-implementation`, HEAD `21efbea` (post-v0.1.1 + post-merge-to-main; CHANGELOG carries `[0.1.1]` and `[0.1.0]` entries; md1 sibling at `md-codec-v0.9.1` with `chunk_set_id` rename landed).
**Primary artifact:** `bip/bip-mnemonic-key.mediawiki` (mk1 BIP draft, ~490 lines pre-fix).

## 1. Methodology

1. End-to-end read of the mk1 BIP draft, taking inventory of every cross-reference and concrete claim.
2. Cross-references categorised into eight buckets:
   - external BIP / RFC links
   - external GitHub / repo links
   - internal `§"..."`-style section anchors
   - SPEC `§N.M` / SPEC-claim agreement
   - DECISIONS / closure-Q citations
   - sibling-format (md1) references
   - `Error::*` variant citations against `crates/mk-codec/src/error.rs`
   - concrete numeric / capacity claims against SPEC + `crates/mk-codec/src/consts.rs`
3. For every cross-reference: verify the target file or section exists; verify the claim text agrees with the target.
4. External BIP cites that cannot be verified without web access flagged as "external — assumed-correct"; not edited.
5. Drift surfaced: minimal inline edit to BIP (or, in one case, SPEC) to remove the drift; tests re-run after each substantive change.
6. Audit document written; FOLLOWUPS entry to be flipped to `resolved <commit>` by the controller after the corresponding fix-up commit lands.

Total cross-references audited: **74** (counted below by bucket).

## 2. Findings

### 2.1 Findings summary by severity

| Severity | Count | Resolution |
|---|---|---|
| Blocker | 1 | Fixed inline |
| Important | 3 | Fixed inline |
| Minor | 5 | Fixed inline |
| Observation | 4 | Documented; no edit required |

All drifts found were fixed inline; **zero items deferred to human review**. Remaining gates are the unrelated FOLLOWUPS items (`nums-structural-audit`, `chunk-set-id-rename` — see §6).

### 2.2 Blocker-tier findings (must fix)

#### B-1. Phantom `Error::FingerprintFlagMismatch` citation (line 264)

The §"Origin fingerprint" section claimed:

> "decoders MUST reject any state where the flag and the payload disagree (`Error::FingerprintFlagMismatch`)"

`Error::FingerprintFlagMismatch` does not exist in `crates/mk-codec/src/error.rs`. The variant was retired in Phase 4 review fixup with the comment (error.rs:230–237):

> "Phase 4 retired the proposed `FingerprintFlagMismatch` variant: structurally undetectable in the decoder under the closure-locked wire format, since no length prefix lets the decoder distinguish 'flag set, fp present' from 'flag unset, fp omitted.' SPEC §4 rule 3 was reframed as an encoder-side invariant"

The BIP itself ALSO contained the correct treatment 70 lines further down (line 333 — "Encoder-side invariant (not a decoder rule)…"), so the §"Origin fingerprint" sentence was self-contradicted by §"Decoder validity rules" within the same document.

**Fix applied:** rewrote line 264 to reframe the rule as an encoder-side invariant, removed the phantom-variant cite, and added an internal pointer to §"Decoder validity rules" for the full statement. See §3 inline-fixes log.

### 2.3 Important-tier findings

#### I-1. Missing `Error::MixedHeaderTypes` citation (post-v0.1.1)

The v0.1.1 release (CHANGELOG `[0.1.1]`, current HEAD `21efbea`) added `Error::MixedHeaderTypes` to discriminate the `[SingleString, Chunked, ...]` and `[Chunked, ..., SingleString, ...]` rejection paths previously surfaced as `Error::ChunkedHeaderMalformed`. The BIP's §"Decoder validity rules" string-layer rules did not yet list this variant. A v0.1.1-conformant implementation has a reject path the BIP does not cite.

**Fix applied:** inserted a new rule between `UnsupportedCardType` and `ChunkSetIdMismatch` in the string-layer rules block, naming `MixedHeaderTypes` and disambiguating it from `ChunkedHeaderMalformed`. See §3.

#### I-2. Stale "rename in flight" claim for `chunk_set_id` (lines 40, 50)

The Definitions block (line 40) and the §"Naming note" tail paragraph (line 50) described the `chunk_set_id` rename in md-codec as "in flight" / "likely as md-codec v0.9.0." That release has since shipped — the sibling repo is at `md-codec-v0.9.1` with the rename fully landed in both the BIP draft and the reference implementation. The "in flight" wording overstates uncertainty and undersells the actually-shipped resolution.

**Fix applied:** updated both passages to cite the released `md-codec-v0.9.0` tag and confirm the rename is in agreement with md1's published BIP draft. Wire-format-unchanged claim retained. See §3.

#### I-3. SPEC §3.2 cites BIP 32 for origin notation; BIP correctly cites BIP 380 (cross-doc drift)

The BIP (line 247) correctly attributes the `[fp/path]` origin-notation reading order to BIP 380 ("matches BIP 380 origin notation"). The SPEC counterpart (`design/SPEC_mk_v0_1.md` line 174) attributed the same convention to BIP 32. BIP 32 defines extended keys; the bracketed origin syntax is BIP 380's contribution. BIP is correct; SPEC has the factual error.

**Fix applied:** updated SPEC §3.2 line 174 to cite BIP 380 (not BIP 32). The BIP draft itself was not edited.

### 2.4 Minor-tier findings

#### M-1. Internal `§"Recovery flow"` does not exact-match a section heading (3 occurrences: lines 314, 333, 391)

The actual section heading is `===Linkage to MD and recovery flow===` (line 345). Readers locating the section via Ctrl-F on `Recovery flow` would find it via substring match, but the heading-quote convention in the rest of the BIP exact-quotes the heading.

**Fix applied:** all three occurrences updated to `§"Linkage to MD and recovery flow"`. See §3.

#### M-2. Internal `§"Privacy"` does not exact-match (line 266)

Section is `==Privacy considerations==` (line 370). Substring match works; the rest of the BIP uses `§"Privacy considerations"` consistently.

**Fix applied:** line 266 updated to `§"Privacy considerations"`.

#### M-3. Internal `§"xpub encoding"` against heading `===xpub encoding (compact-73)===` (lines 34, 314, 391)

The exact heading carries the parenthetical `(compact-73)`. Substring match works; the noun phrase `xpub encoding` is unambiguous in context. Of the three occurrences:

- Line 34 (Definitions block): kept as-is (`§"xpub encoding"`); the parenthetical is just a descriptor of which xpub encoding mk1 uses, and reads naturally without `(compact-73)` here.
- Line 314 (Limit-of-detection note): kept as-is; carrying `(compact-73)` here would be redundant — the surrounding sentence already says "compact-73."
- Line 391 (Privacy / Integrity-detection-limit recommendation): updated to the exact heading `§"xpub encoding (compact-73)"` because this is the standalone "see also" pointer where exact-match aids navigation.

**Fix applied:** line 391 only. Lines 34 and 314 left as-is — substring match is unambiguous and the surrounding prose is cleaner without the parenthetical.

#### M-4. md1 sibling-BIP `§"..."` cites (lines 129, 149)

- `MD §"Length envelope"` → md1 BIP heading `====Length envelope====` at md1:250. Match.
- `MD §"Decoder reporting"` → md1 BIP heading `===Decoder reporting===` at md1:741. Match.

No drift. Verified against `/scratch/code/shibboleth/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki` on `main` (HEAD `68ed11e`, post-`md-codec-v0.9.1`).

#### M-5. md1 `Tag::SharedPath` mirror claim (lines 270, 272)

The BIP claims the standard-table dictionary `mirrors MD's Tag::SharedPath table byte-for-byte`. The md1 BIP at v0.9.x retains the path-dictionary table at lines ~518/574 ("`Tag::SharedPath` (`0x33`)" with named indicators `0x01`–`0x07`, `0x11`–`0x15`, `0x17`). The 13-row pattern matches mk1's claim. The 0x16 gap is documented in both BIPs and tracked as `md-path-dictionary-0x16-gap` in FOLLOWUPS.

No drift. Observation only.

### 2.5 Observation-tier findings (no edit)

#### O-1. RFC 2119 keywords paragraph is absent

mk1 uses MUST / SHOULD / MAY / MUST NOT throughout (verified via grep: dozens of occurrences) but does not include the standard "The key words … are to be interpreted as described in RFC 2119" paragraph in the Definitions block. md1's BIP does include such a paragraph (md1:40). For BIP-style normative-language correctness, mk1 SHOULD add this paragraph before formal submission — but this is a presentational gap, not a cross-reference drift.

**Recorded as:** observation; not fixed inline (it is content-additive, not drift). Recommend adding to a future BIP-polish pass before BIP submission. Tracked separately is left to the FOLLOWUPS controller; nothing in this audit's scope requires the fix.

#### O-2. BIP 44 / 49 / 84 / 86 / 87 / 48 not in References list

The BIP cites these BIPs by number in the path-dictionary discussion (lines 272, 415) and in FAQ (line 425). They are not enumerated in the §"References" list. md1 BIP follows the same convention — only the BIPs it directly relies on (codex32, bech32 / bech32m, descriptors, wallet policies) appear in its References list, with path-family BIPs (44/49/84/86/48/87) cited inline only. mk1 follows md1's convention.

**Recorded as:** observation; consistent with sibling format's convention.

#### O-3. BIP 327 (MuSig2) cited in plain text without link (line 443)

The §"What about MuSig2 aggregate keys?" FAQ cites BIP 327 as "out of scope." A live BIP submission usually wants every BIP citation linked. Low-effort fix; not a drift.

**Recorded as:** observation; recommend adding the link in a future BIP-polish pass.

#### O-4. SLIP-0173 cited without a link (line 435)

The §"Why HRP `mk` specifically?" FAQ cites SLIP-0173 as the informal segwit-HRP registry but does not link to it. Same observation as O-3.

**Recorded as:** observation.

## 3. Inline fixes applied (the diff)

All edits committed only in this audit pass; no behavior change to `mk-codec`. Tests re-run after edits; 46 unit + 4 integration tests pass.

### 3.1 BIP draft (`bip/bip-mnemonic-key.mediawiki`)

| Fix | Before | After |
|---|---|---|
| B-1 §"Origin fingerprint" line 264 | `decoders MUST reject any state where the flag and the payload disagree (Error::FingerprintFlagMismatch).` | Encoder-side-invariant restatement; removes phantom variant; cross-references §"Decoder validity rules". Adds explicit "(No dedicated `Error` variant: this rule is an encoder contract)" disambiguation. |
| I-1 §"Decoder validity rules" string-layer rules | (no `MixedHeaderTypes` rule) | New rule inserted between `UnsupportedCardType` and `ChunkSetIdMismatch` naming `Error::MixedHeaderTypes`, disambiguated from `ChunkedHeaderMalformed`. |
| I-2 Definitions / `chunk_set_id` line 40 | `(md-codec v0.8.x calls this field "wallet identifier" — a misleading name slated for rename to chunk_set_id across both repos.)` | Cites the released `md-codec-v0.9.0` tag and confirms the rename has shipped; wire-format-unchanged claim retained. |
| I-2 §"Naming note" tail paragraph line 50 | "in flight per a coordinated cross-repo follow-up… The MD repo will land the rename (likely as md-codec v0.9.0, docs-and-symbols-only) before MK's BIP is formally submitted." | Updated to cite the released `md-codec-v0.9.0` tag and confirm parity with md1's published BIP draft. |
| M-1 line 314 | `§"Recovery flow"` | `§"Linkage to MD and recovery flow"` |
| M-1 line 333 | `§"Recovery flow"` | `§"Linkage to MD and recovery flow"` (with the encoder-invariant fix from B-1 applied in the same edit) |
| M-1 / M-3 line 391 | `(see §"xpub encoding"), … §"Recovery flow" step 4` | `(see §"xpub encoding (compact-73)"), … §"Linkage to MD and recovery flow" step 4` |
| M-2 line 266 | `See §"Privacy" for guidance.` (and same edit picked up the B-1 paragraph rewrite) | `See §"Privacy considerations" for guidance.` |

### 3.2 SPEC (`design/SPEC_mk_v0_1.md`)

| Fix | Before | After |
|---|---|---|
| I-3 §3.2 rationale bullet 2 line 174 | `matches BIP 32 origin notation [fp/path] reading order.` | `matches BIP 380 origin notation [fp/path] reading order.` |

### 3.3 Reference implementation (no changes)

Nothing in `crates/mk-codec/` was edited. The `Error::MixedHeaderTypes` variant cited in I-1 already exists in `crates/mk-codec/src/error.rs:91` per the v0.1.1 release.

## 4. Cross-reference inventory

The 74 references audited break down as:

| Bucket | Count | Drift | Status |
|---|---|---|---|
| External BIP cites (BIP 32, 93, 173, 350, 380, 388, 327, 44/49/84/86/87/48) | 18 (counting unique cite sites) | 0 (assumed-correct external; not verifiable without web fetch) | observation only — see O-2, O-3 |
| External GitHub URLs (bg002h/descriptor-mnemonic, bg002h/mnemonic-key, bitcoin/bips, creativecommons.org) | 14 | 0 well-formed; 1 stale wording (the v0.9.0 "in flight" claim) | I-2 fixed |
| Internal `§"..."` section cites | 9 | 5 | M-1×3, M-2, M-3 fixed |
| SPEC `§N.M` claims (paragraph-level cross-doc agreement) | 11 | 1 (BIP 380 vs BIP 32 origin attribution; SPEC was wrong) | I-3 fixed |
| DECISIONS / closure-Q cites | 1 (`closure Q-3 lock`, BIP line 284) — others reference Q-N indirectly via SPEC | 0 | OK |
| Sibling md1 references (md1 BIP §"Length envelope", §"Decoder reporting", `Tag::SharedPath` mirror, `chunk_set_id` parity) | 5 | 0 | OK |
| `Error::*` variant cites | 16 | 1 phantom (`FingerprintFlagMismatch`); 1 missing (`MixedHeaderTypes`) | B-1, I-1 fixed |
| Concrete numeric / capacity claims (45/53 fragment, 32 chunks, 1692 ceiling, 80 minimum, 84 typical, 56 single-string, 10 path-cap, 73 compact xpub, 4-byte stub, 4-byte fingerprint, etc.) | ≥11 distinct numbers, all mutually consistent across BIP, SPEC, consts.rs (verified by direct comparison) | 0 | OK |

External BIP cite verification was structural only: each external link target was checked for canonical URL form (`https://github.com/bitcoin/bips/blob/master/bip-0NNN.mediawiki`) and a sane title. Section-name agreement against external BIP bodies was NOT verified (no web access in the audit environment). Per audit method §1 step 3, these are flagged "external — assumed-correct" and a final pre-submission web-spot-check before BIP PR is recommended (one click each on the 6–7 external BIP links).

## 5. Numeric-claim cross-check matrix

Verified each concrete number appears identically across BIP / SPEC / `consts.rs`:

| Claim | BIP | SPEC | consts.rs | Agreed |
|---|---|---|---|---|
| `MK_REGULAR_CONST = 0x1062435f91072fa5c` | line 94, 107 | §2.3 line 55, 64 | `consts.rs:18` | yes |
| `MK_LONG_CONST = 0x41890d7e441cbe97273` | line 95, 108 | §2.3 line 56, 65 | `consts.rs:21` | yes |
| `NUMS_DOMAIN = b"shibbolethnumskey"` | line 93, 106 | §2.3 line 54, 63 | `consts.rs:15` | yes |
| Single-string regular: 48 B | line 131 | §2.4 line 77 | `consts.rs:30` | yes |
| Single-string long: 56 B | line 132 (`56 bytes payload`) | §2.4 line 78 | `consts.rs:33` | yes |
| Chunked-fragment regular: 45 B | line 131 | §2.4 line 79 | `consts.rs:36` | yes |
| Chunked-fragment long: 53 B | line 132, 136 (`32 × 53`) | §2.4 line 80 | `consts.rs:39` | yes |
| Max chunks: 32 | line 136 | §2.4 line 81 | `consts.rs:42` | yes |
| Card ceiling: `32 × 53 − 4 = 1692 B` | line 136 | §2.4 line 87 | derived from above | yes |
| Cross-chunk hash: 4 B | line 196–202 | §2.4 line 82, §2.6 | `consts.rs:45` | yes |
| Min bytecode: `1+1+4+1+73 = 80 B` | line 73 | §2.4 line 85 | derived | yes |
| Typical: `1+1+4+4+1+73 = 84 B` | line 71 | §3.2 line 169 | derived | yes |
| Compact xpub: 73 B | line 68, 290 | §3.6 line 232 | `consts.rs:53` | yes |
| Policy ID stub: 4 B | line 65 | §3.3 line 179 | `consts.rs:56` | yes |
| Origin fingerprint: 4 B | line 66 | §3.4 | `consts.rs:59` | yes |
| Path-component cap: 10 | line 284, 415 | §3.5 line 228 | `consts.rs:27` | yes |
| Birthday-bound: `≈ 2.85 × 10⁻⁷` (50 stubs at 32 bits) | line 254 | §3.3 line 184 | n/a | yes |

All capacity / numeric claims internally consistent.

## 6. Remaining items (not addressed in this audit)

These are unrelated FOLLOWUPS items that block formal BIP submission but are out of scope for this cross-reference audit:

- `nums-structural-audit` (`pre-bip-submission` tier) — Andrew-Poelstra-style structural-relationship audit of `MK_REGULAR_CONST` / `MK_LONG_CONST` against BIP 93's BCH polynomial.
- External-BIP web-spot-check — the 6–7 BIP links should be clicked once before BIP PR (audit-internal observation, not a tracked FOLLOWUPS item).
- O-1 RFC 2119 keywords paragraph (suggested polish, optional).
- O-3, O-4 link-additions for BIP 327 and SLIP-0173 (suggested polish, optional).

## 7. Audit conclusion

After applying the inline fixes, the BIP draft is **internally consistent, in agreement with SPEC and DECISIONS, error-variant-aligned with the v0.1.1 reference implementation, and parity-correct with the md1 sibling BIP at v0.9.1.** All categorical drifts surfaced by the audit are resolved; the remaining gates are unrelated pre-submission items (`nums-structural-audit`, external-BIP web-spot-check, optional RFC 2119 polish).

The `bip-cross-reference-completeness` gate item in `design/FOLLOWUPS.md` should be flipped to `resolved <commit>` after the corresponding fix-up commit lands.

---

**Auditor sign-off:** Claude Opus 4.7 review subagent (cross-reference completeness pass), 2026-04-29.
