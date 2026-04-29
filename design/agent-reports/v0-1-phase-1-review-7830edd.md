# mk1 v0.1 Phase 1 review — commit 7830edd

**Status:** DONE — no blockers
**Reviewer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**Commit:** 7830edd
**File(s):**
- `design/SPEC_mk_v0_1.md` (under review)
- `design/DECISIONS.md` (under review)
- `docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md` (closure source-of-truth)
- `design/FOLLOWUPS.md`
- `design/IMPLEMENTATION_PLAN_mk_v0_1.md`
- `/scratch/code/shibboleth/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki` (cross-format reference)
**Role:** reviewer (spec)

## Summary

No blockers. Every closure-ripple item has concrete spec text that matches the lock; numerics verify (NUMS hex + birthday-bound); md1 cross-format alignment claim holds against the md1 BIP §"Header"/§"Length envelope". Found **2 should-address** drifts (one truly substantive: SPEC §3.5 explicit-path size formula contradicts plan + closure), **2 nits**, plus several confirmations.

## Issues

### #1 — should-address — SPEC §3.5 explicit-path encoding contradicts plan + LEB128 reality

SPEC §3.5 Case B writes the explicit-path layout as `[0xFE][component_count: 1 B][component_1 ... component_N: each LEB128-encoded u32]`. The closure design Q-6 and IMPLEMENTATION_PLAN Step 1.1.5 both annotate explicit-path size as `1 + 1 + 5N B` (i.e., **5 bytes per component**), implying fixed-width u32 LE — not LEB128. LEB128 of a hardened-bit-set u32 (high bit set ⇒ 5 bytes always) does collapse to 5 bytes per component, but only because every hardened component pessimizes; non-hardened components < 2²⁸ would be 1–4 bytes. Either the wire format is LEB128 (variable, 1-5 B per component) and the plan/closure `1 + 1 + 5N B` figure is a worst-case shorthand that should be footnoted, or the wire is fixed-width u32 LE (which is what the chunk-budget math `5N` actually implies) and §3.5 saying "LEB128" is wrong.

**Resolution:** pick one. If LEB128 is intended, §3.5 needs a note that `5N` in §3.2 / closure Q-6 is the worst-case bound (all-hardened components) and not the field's literal width. If fixed-width is intended, change §3.5 to "each component is a 4-byte little-endian u32 with the BIP 32 hardened-bit in the high bit" and update §3.2's hint accordingly. Phase 4 cannot land path.rs without this resolved — the codec impl and the SPEC §3.5 text disagree today.

### #2 — should-address — SPEC §3.2 ASCII art mis-states explicit-path size

SPEC §3.2 reads `[origin_path : 1 B (std-table indicator) OR 1 + 1 + LEB128(c_1..c_N) B (explicit)]`. The "`1 + 1 + LEB128(c_1..c_N)` B" is mixing units (the LEB128-encoded *bytes* are not a count expression). Closure Q-6 has `1 + 1 + 5N B` (a numeric upper bound). The current spec text reads as if the third term were a function call. Pick a numeric bound or use prose.

**Resolution:** replace with `1 + 1 + Σ leb128_len(c_i) B` (or `1 + 1 + (1..=5)·N B` as a worst-case bound), or with the closure's `1 + 1 + 5N B` if §3.5 actually carries fixed-width components per #1.

### #3 — nit — SPEC §2.4 capacity prose says "1–8 bytes origin path"

§2.4 reads "73-byte compact xpub + 1–8 bytes origin path". The 8-byte upper bound is wrong: with the locked path-component cap = 10 and 5-bytes-per-component worst case, an explicit path can reach `1 + 1 + 5×10 = 52 bytes`. Even at LEB128 with all-hardened components (5 B each), the bound is 52 B not 8 B.

### #4 — nit — DECISIONS.md D-12 still references mk-codec-inside-descriptor-mnemonic option

D-12 reads coherently for the actual repo layout (sibling repo) but the parenthetical history paragraph re-introduces ambiguity by calling out an option that was foreclosed. Phase 1 was supposed to drop provisional language; this paragraph is closer to provenance trivia than active decision content. Not a blocker — the prose is internally consistent — but a future reader will spend a beat figuring out which world the doc is in. Consider trimming to one sentence in a follow-up.

## Confirmations

- **Q-1 NUMS hex.** Recomputed: `SHA-256(b"shibbolethnumskey").hex() = 83121afc88397d2e4e7f2ba3502f97559a84f088f9d5b6539372f03b3494c99d`; top-65 = `0x1062435f91072fa5c`, top-75 = `0x41890d7e441cbe97273`. SPEC §2.3 matches. md1 cross-check (`shibbolethnums` → `0x815c07747a3392e7` / `0x205701dd1e8ce4b9f47`) also matches md1's published constants — derivation rule is sound.
- **Q-2 birthday-bound math.** `k(k-1)/(2·2³²)` with k=50 yields 2.8522e-07; SPEC §3.3's `≈ 2.85 × 10⁻⁷` is correct. The "even 24 bits clears it" claim (`≈ 7.3 × 10⁻⁵`) also recomputes exactly.
- **Q-3 path cap = 10.** SPEC §3.5 + §4 rule 6 + DECISIONS Q-3 row + closures table all agree.
- **Q-4 authority precedence.** SPEC §5.1 carries the closure's exact contract (mk1 origin_path authoritative; orchestrator-layer mismatch rejection; per-format decoders not cross-aware). md1 tag-byte allocation correctly punted to FOLLOWUPS `md-per-N-path-tag-allocation`.
- **Q-5 chunk types + string-layer header.** SPEC §2.5 carries 5-bit type field, 0x02..0x1F reserved exhausting the field, 8-char chunked layout (5+5+20+5+5 = 40 bits = 8 chars). Matches closure exactly.
- **Q-6 payload field order.** SPEC §3.2 byte-for-byte matches closure: header→stub_count→stubs→fp→path→xpub_compact. 84 B example arithmetic checks out (1+1+4+4+1+73=84).
- **Q-7 compact-73.** SPEC §3.6 byte breakdown sums to 73 (4+4+32+33). Reconstruction rule for depth/child_number is stated. §4 rule 8 (`XpubDepthMismatch`) correctly removed (visible in §4 closing note). Limit-of-detection note appears in both §3.6 and §6 with consistent framing.
- **Q-8 fingerprint flag.** SPEC §3.1 valid header values 0x00 / 0x04 are correct (bit 2 = 0b00000100 = 0x04). §3.4 / §3.7 / §4 rule 3 / §6 paragraph all consistent.
- **Q-9 split trigger.** SPEC §10 + DECISIONS D-13 closure note both name "both formats v1.0 with cross-validated conformance vectors."
- **Q-10 family token.** SPEC §7 carries `mk-codec X.Y` with the patch-stability clause.
- **D-14 string-layer header mirrors md1.** Verified against `bip-mnemonic-descriptor.mediawiki` lines 184-192: 2-char single, 8-char chunked, version + type + 20-bit identifier (md1: "wallet identifier"; mk1: `chunk_set_id`) + 5-bit total + 5-bit index. Bit-level layout identical. mk1's §2.4 capacity numbers (48/56/45/53) are arithmetically derivable from md1's BIP §"Length envelope" — not just claimed identical, actually identical.
- **D-15 chunk_set_id rename.** Sequencing requirement captured in DECISIONS D-15 + FOLLOWUPS `chunk-set-id-rename` (cross-repo tier) + `bip-cross-reference-completeness` (depends on the rename landing first). Hard dependency is documented.
- **No leftover provisional language.** No `0x???`, no `TBD` (except a deliberate one in DECISIONS context table — "mk BIP (TBD)" referring to BIP-number-not-yet-assigned, which is fine), no `PROVISIONAL` markers found in either file.
- **DECISIONS D-1..D-13 substance unchanged.** Spot-checked D-1, D-3 (path codec), D-7 (≥1 stub), D-10 (BCH plumbing), D-13 (split trigger). All substance preserved; only the closure-section addition + Q-rows-rewritten-as-closed are new.
- **§4 ↔ §3.x consistency.** §4 rule 1 (UnsupportedVersion) ↔ §3.1; rule 2 ↔ §3.7; rule 3 ↔ §3.1+§3.4; rule 4 ↔ §3.3; rules 5-7 ↔ §3.5; rule 8 ↔ §3.6; rules 9-10 ↔ §3.2. All resolve.
- **§3.6 limit-of-detection ↔ §6 normative.** §3.6 closes "§6 recommends an out-of-band first-address verification." §6 paragraph "Integrity detection limit (closure Q-7 → §6 normative)" says exactly that. Aligned.
- **Q-7 compact-73 ↔ §3.5 origin_path coupling.** §3.6 "depth := component_count(origin_path); child_number := last_component(origin_path)" is internally coherent with §3.5 (both std-table indicator dictionary entry and explicit-path component list expose those values). Coherent.

## Open observations

- IMPLEMENTATION_PLAN Step 3.1.2 lines 401-430 correctly stages the leading 128 bits of the 256-bit digest as u128 and shifts right by 63/53 (post the plan-review-1 fix). Phase 1 is docs-only so this is forward-only; mentioning here only to confirm the prior issue was carried.
- §10 says "Eventual extraction to a shared `mc-codex32/` workspace member is committed but deferred per closure Q-9 …". This is fine but note that "workspace member" is slightly misleading given D-12's "third sibling repo" framing — pick one term ("third sibling repo" or "workspace member") and use it consistently across §10, D-12, D-13. Minor consistency nit only.

Phase 1 is approvable on Issue #1 + #2 fix-up. #3 / #4 / observations are nice-to-have follow-ups.
