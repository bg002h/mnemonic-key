# Phase 2 review — BIP draft fill-in (commit 4728230)

**Status:** DONE_WITH_CONCERNS
**Commit:** 4728230
**Reviewer / Implementer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**File(s):**
/scratch/code/shibboleth/mnemonic-key/bip/bip-mnemonic-key.mediawiki
/scratch/code/shibboleth/mnemonic-key/design/SPEC_mk_v0_1.md
/scratch/code/shibboleth/mnemonic-key/design/DECISIONS.md
/scratch/code/shibboleth/mnemonic-key/design/FOLLOWUPS.md
/scratch/code/shibboleth/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki
**Role:** reviewer (BIP)

## Summary

No blockers. One should-address consistency gap with the md1 BIP's path table (md1 omits 0x16 testnet entry), three nits around wording precision and structural BIP discipline, and several confirmed alignments. NUMS constants are independently reproduced; both forward (mk) and back (md) checks pass.

## Issues

### should-address

**S1. Testnet path indicator 0x16 — silent inconsistency with md1's published table.**
- BIP §"Origin path encoding" Case A: claims indicators `0x11` through `0x17` "name the testnet variants" and "exact dictionary mirrors MD's `Tag::SharedPath` table byte-for-byte."
- SPEC §3.5: same claim ("Exact dictionary mirrors md1's `Tag::SharedPath` table byte-for-byte").
- md1 BIP §"Path dictionary" (lines 339-349): testnet rows list 0x11, 0x12, 0x13, 0x14, 0x15, **0x17**. **0x16 is omitted** (no testnet pair for mainnet 0x06 = `m/48'/0'/0'/1'` BIP-48 nested-segwit multisig).
- Resolution options: (a) escalate to a cross-repo follow-up to add the missing 0x16 row in md1, then mk1 inherits cleanly; (b) explicitly enumerate the dictionary in mk1's BIP and call out that 0x16 is reserved with a footnote pointing to the md1 gap; (c) document the gap inline ("`0x11`–`0x15` and `0x17`; `0x16` is reserved pending md1 dictionary update"). Option (a) is cleanest and aligns with the existing `chunk-set-id-rename` cross-repo coordination model. This is should-address rather than blocker because the encoder cannot legitimately emit 0x16 today (md1 would reject), so wire-level interop is unaffected.

### nit

**N1. BIP §"Definitions and notation" `chunk_set_id` parenthetical references "md-codec ≤ v0.8.x".** The DECISIONS log D-15 says the rename will likely land as md-codec v0.9.0 docs-and-symbols-only. The phrasing is fine but consider tightening to "md-codec v0.8.x" (current shipped) since v0.9.x is hypothetical. Minor; either reading is defensible.

**N2. BIP front-matter `Status:` line.** Reads "Pre-Draft, v0.1 wire format locked, awaiting reference implementation milestone and pre-submission audit". The BIP-1 / BIP-2 conventions for the `Status:` field expect a single short token (Draft, Active, etc.). The current line is informative but not standard. Recommend the long-form text move to the Abstract or a Status note, with the field itself reading e.g. `Draft` or `Pre-Draft` only. Not blocking — many in-flight BIPs play loose with this — but reviewers at formal-submission time will flag it.

**N3. BIP §"Format overview" pre-block re-states the payload field order, then §"Payload field order" repeats it.** Tolerable for readability but consider noting "(see §"Payload field order" for normative ordering)" on the first occurrence to flag the second as authoritative.

**N4. BIP §"Backwards Compatibility" is one paragraph.** Acceptable scope for a v0.1 with no prior versions; md1's analog is similarly terse. No fix required; flagging because the section's brevity sometimes attracts reviewer pushback.

## Confirmations

- **NUMS constants verified independently.** `python3 -c 'import hashlib; n=int.from_bytes(hashlib.sha256(b"shibbolethnumskey").digest(),"big"); print(hex(n>>191), hex(n>>181))'` reproduces `0x1062435f91072fa5c` and `0x41890d7e441cbe97273`. The same procedure on `b"shibbolethnums"` reproduces md1's `0x815c07747a3392e7` and `0x205701dd1e8ce4b9f47` (md1's BIP shows the regular constant zero-padded to 17 hex digits as `0x0815c07747a3392e7`, which matches). Cross-derivation claim holds.
- **Length envelope numbers (48/56/45/53, 32 chunks, 1692 byte ceiling)** match md1 BIP §"Length envelope" lines 257-270 verbatim. The 94-95-character invalid window is also correctly flagged.
- **String-layer header mirrors md1.** mk1 BIP §"String-layer header" 2-char single / 8-char chunked breakdown matches md1 BIP §"Header" lines 175-192. Bit assignments and field semantics align (one wording difference: mk1 BIP uses "<code>chunk_set_id</code>" throughout while md1 still says "Wallet identifier" — this is the rename in flight per D-15 and is consistent with FOLLOWUPS `chunk-set-id-rename`).
- **Bytecode-header bit assignments** (bit 7-4 version / bit 3 reserved / bit 2 fingerprint flag / bit 1, 0 reserved; valid values 0x00 / 0x04) match SPEC §3.1 and DECISIONS Q-8 exactly. The "mirrors MD's bit-allocation shape" claim verified against md1 BIP lines 286-300.
- **Payload field order** (header → stub_count → stubs → fingerprint (gated) → path → xpub_compact) matches SPEC §3.2 and closure Q-6.
- **Path encoding component cap = 10**, hardened-bit-in-MSB convention, 1..=5 byte LEB128 sizing, reserved indicator ranges (`0x00`, `0x08`–`0x10`, `0x18`–`0xFD`, `0xFF`). All consistent with SPEC §3.5 and closure Q-3.
- **Compact-73 xpub byte breakdown** (4 + 4 + 32 + 33 = 73) and reconstruction rule (depth = component_count, child_number = last_component) match SPEC §3.6.
- **Validity rule list** mk1 BIP §"Decoder validity rules" enumerates 10 bytecode rules + 5 string/chunking rules. Matches SPEC §4 exactly. Each rule maps to a named `Error::*` variant — required precondition for the `decoder-error-variant-parity` audit.
- **Authority-precedence wording** (mk1 origin_path is authoritative; md1 per-`@N` is descriptive; mismatch rejected at orchestrator layer) matches SPEC §5.1 and closure Q-4.
- **Privacy recommendations** (storage discipline, no photography, stub-count tradeoff, optional fingerprint engraving, disposal of rotated cards, hand-off discipline, integrity-detection limit) are present and aligned with SPEC §6.
- **Pre-submission audit references all present in BIP body** — NUMS structural audit (§"Why new target constants?" closing paragraph), HRP `mk` SLIP-0173 collision check (§FAQ "Why HRP `mk` specifically?"), Error-variant ↔ negative-vector parity (§"Test Vectors"). All correctly framed as "required before formal BIP submission, not a v0.1 release blocker."
- **Structural BIP discipline.** Abstract / Motivation / Definitions / Format overview / Specification / Privacy / Rationale / FAQ / Backwards Compatibility / Test Vectors / Reference Implementation / Acknowledgments / References / Copyright. All present, scoped reasonably.
- **MediaWiki markup hygiene.** No malformed `[[wiki-link]]` constructs; no double-bracketed wiki-internal links (intentional — this is being submitted to bitcoin/bips, where external links via single brackets are conventional). `<code>`, `<pre>`, `<source lang=...>`, table syntax (`{| class="wikitable" ... |}`) all balanced. Section anchors (`§"..."`) are consistent prose convention, not wiki-rendered links — fine for the bitcoin/bips reading audience.

## Open observations

- **D-15 sequencing risk surfaces in BIP §"Definitions and notation".** The note "MK uses the renamed name `chunk_set_id` throughout this BIP" presupposes md1's rename will land before formal submission. FOLLOWUPS `chunk-set-id-rename` tracks this at tier `cross-repo`; the BIP correctly hedges with "before MK's BIP is formally submitted." Worth re-checking that `bip-cross-reference-completeness` in FOLLOWUPS lists the rename as a strict precondition (it does — line 85).
- **`compute_wallet_instance_id` helper reference is in SPEC §5 step 4 implementation note but not in BIP.** Probably fine — that's an md-codec implementation detail and the BIP normatively specifies the construction. Just noting the asymmetry.
- **§"Decoder validity rules" rule 7 (`Error::InvalidPathComponent`) is vague** — "BIP 32 child-number high bits set in invalid ways." This is inherited from SPEC §4 verbatim. The phrasing is sufficient for v0.1 but a future revision may want a tighter operationalization (e.g., "any LEB128 decode that yields a value > 0xFFFFFFFF").
- **Reserved-indicator-range stewardship.** mk1's reserved range `0x18`–`0xFD` is identical to md1's. If md1 ever allocates 0x16 (per S1) or any other testnet/mainnet pair, mk1 inherits the allocation by the byte-for-byte mirror clause. Worth a one-line FOLLOWUP entry to formalize the inheritance contract; otherwise a future md1 path-dictionary entry could land without an mk1 spec amendment and produce silent drift.
