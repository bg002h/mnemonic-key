# mk1 v0.1 — Open-Questions Closure Design

**Date:** 2026-04-29
**Scope:** Close the ten open questions (Q-1 through Q-10) in `design/DECISIONS.md` and `design/SPEC_mk_v0_1.md`, applying fresh-eyes re-litigation rather than rubber-stamping the provisional answers. Capture the resulting wire-format and spec amendments, plus cross-repo coordination items and pre-BIP-submission audit tasks.

**Sibling-repo dependency:** `bg002h/descriptor-mnemonic` (md1) is at md-codec v0.8.0, with the BIP draft pre-formal-submission. Several mk1 closures align mk1 with md1 to enable shared-parser logic per D-13.

**Status:** This document is the brainstorming/closure-design artifact. The implementation plan (subsequent step) translates these closures into edits to `design/SPEC_mk_v0_1.md`, `design/DECISIONS.md`, the BIP draft skeleton, and the `mk-codec` reference-implementation scaffold.

---

## 1. Closures

Each entry: (a) provisional answer from `SPEC_mk_v0_1.md` §9; (b) fresh-eyes finding; (c) locked answer; (d) rationale and downstream implications.

### Q-1. NUMS domain string + target residue constants

**Provisional:** `b"shibbolethnumskey"`; constants computed at BIP-draft time.

**Fresh-eyes finding:** The provisional string follows md1's `b"shibbolethnums"` convention with a descriptive suffix (`key` for "Mnemonic Key"). Future format extensions naturally take the form `shibbolethnums<suffix>` (e.g., `shibbolethnumsmusig`). The `shibbolethnums` prefix-of-`shibbolethnumskey` relationship is not a concern: SHA-256 outputs of independent inputs are effectively independent, and we are deriving constants, not authenticating anything.

**Lock:**

```
domain_string         = b"shibbolethnumskey"   (17 bytes ASCII)
MK_REGULAR_CONST      = 0x1062435f91072fa5c    (65 bits)
MK_LONG_CONST         = 0x41890d7e441cbe97273  (75 bits)
```

Reproducer (anyone can run independently to verify):

```python
import hashlib
h = hashlib.sha256(b"shibbolethnumskey").digest()
MK_REGULAR_CONST = int.from_bytes(h, 'big') >> (256 - 65)  # = 0x1062435f91072fa5c
MK_LONG_CONST    = int.from_bytes(h, 'big') >> (256 - 75)  # = 0x41890d7e441cbe97273
```

md1 sanity check: applying the same procedure to `b"shibbolethnums"` reproduces md1's published `T_REGULAR = 0x0815c07747a3392e7` and `T_LONG = 0x205701dd1e8ce4b9f47`, confirming derivation correctness.

**Rationale:** Independence from md1 is the only formal requirement (per D-10). The descriptive-suffix pattern enables clean future-format extensions. The "string itself is the audit trail" property carries over from md1.

**Downstream:** SPEC §2.3 hex placeholders replaced with the concrete values above. BIP draft "Why new target constants?" section can be written as a near-verbatim adaptation of md1's analog.

---

### Q-2. Policy ID stub size

**Provisional:** 4 bytes = top 32 bits of `SHA-256(canonical_bytecode)`.

**Fresh-eyes finding:** The §3.3 rationale's claim "matches the existing chunk-header wallet-ID stub convention from md1" is loose — md1's chunk-header `walletID` is 4 bech32 chars × 5 bits = 20 bits, not 32 bits. The two truncations are at different bit lengths and live at different layers (string header vs. bytecode body); the parallelism is shape-level, not byte-level. Threat-model permits even 24 bits (birthday-bound collision probability among 50 entries: `50·49 / (2·2²⁴) ≈ 7.3×10⁻⁵` ≈ 0.0073%). At the locked 32 bits the same bound is `50·49 / (2·2³²) ≈ 2.85×10⁻⁷` ≈ 0.00003% — effectively zero for the 50-wallet ceiling per D-7. Byte-savings argument for 3-byte stubs is weak in the typical 1-stub case (no chunk-boundary impact) and the 7-stub case is explicitly not being optimized.

**Lock:** 4 bytes. Conservative collision margin; round-byte alignment is friendlier to implementers than sub-byte packing; future-proof if md1 ever adds a "32-bit Policy ID stub" mode.

**Downstream:** SPEC §3.3 rationale tightened: drop the "matches md1's chunk-header convention" claim (or rephrase as "shape-level parallel"); add the explicit collision-vs-50-wallets math.

---

### Q-3. Path-component cap

**Provisional:** 32.

**Fresh-eyes finding:** This is a sanity-gate, not a UX decision. Real BIP-style derivations top out at 6 (BIP 48 multisig is 4). Tighter caps are strictly better — they don't lock out real users and they bound the worst-case malicious-path inflation. The cap value is a decoder-side rejection threshold; it does not appear on the wire as a field, so cap value has no wire-format impact.

**Lock:** 10. Margin is still 1.5–2× over real-world. Malicious explicit-path inflation drops from 162 bytes (cap 32) to 52 bytes (cap 10).

**Downstream:** SPEC §3.5 explicit-path rule: `component_count` MUST be in range `1..=10`. `PathTooDeep` error fires at 11+.

---

### Q-4. Per-`@N` path tag byte allocation in MD bytecode

**Provisional (in DECISIONS.md):** New tag in unallocated `0x36+` range, or backfill `0x24-0x32`.

**Fresh-eyes finding:** This is an md-repo concern; mk1's spec cannot answer it. What mk1 *can* declare is the **authority precedence** between mk1's per-card `origin_path` and md1's per-`@N` paths once both formats support them.

**Lock (mk1's side):** mk1 declares the contract:

> When both an mk1 card and an md1 card with per-`@N` paths participate in recovery for the same wallet, mk1's `origin_path` is **authoritative** for the xpub's derivation. md1's per-`@N` path is the policy's **expected** path. Mismatch MUST cause **the recovery orchestrator** to reject the assembly. Per-format decoders are not required to be aware of cross-format context; the cross-format check belongs to the orchestrator layer that sits above both decoders. Implementations MUST surface a precise error identifying both the policy-side expected path and the key-side actual path.

**Downstream:** SPEC §5 (Linkage to MD) gains an "Authority precedence" subsection. The actual md1 wire-format change (tag-byte allocation) is deferred to the descriptor-mnemonic repo. Captured in §3 below as a cross-repo coordination item.

---

### Q-5. Chunk-type byte allocation + string-layer header structure

**Provisional:** `SingleString = 0x00`, `Chunked = 0x01`, mirroring md1. mk1's string-layer header structure was under-specified ("mirrors md1" without pinning bit-level details).

**Fresh-eyes finding:** No reason to diverge from md1 at the chunk-type level (HRP already disambiguates formats). The under-specified pieces of mk1's string-layer header are a real gap — pinning them resolves the chunked-fragment-payload-per-chunk number that drives chunk-count math, plus the `chunk_set_id` width that drives reassembly mismatch detection.

**Lock:**

**(a) Chunk type values:**

```
type = 0x00  →  SingleString
type = 0x01  →  Chunked
type = 0x02..0x1F  →  reserved; decoders MUST reject
```

The `type` field is 5 bits wide (per the locked string-layer header structure). The reserved range `0x02..0x1F` therefore *exhausts* the unallocated space; future format extensions cannot widen the field without bumping the version byte. The SPEC ripple should make this exhaustiveness explicit so a future decoder author does not assume sub-byte spillover space exists.

**(b) String-layer header layout (mirrors md1):**

```
single-string header (2 chars):
  char 0:  version (5 bits)
  char 1:  type = 0x00 (5 bits)

chunked header (8 chars):
  char 0:    version (5 bits)
  char 1:    type = 0x01 (5 bits)
  chars 2-5: chunk_set_id (20 bits, random per-encoding)
  char 6:    total_chunks (5 bits, range 1-32)
  char 7:    chunk_index  (5 bits, 0-indexed)
```

**(c) Capacity (mirrors md1):**

```
single-string regular code:    48 bytes payload
single-string long code:       56 bytes payload
chunked-fragment regular code: 45 bytes per fragment
chunked-fragment long code:    53 bytes per fragment
max chunks per card:           32
cross-chunk integrity hash:    SHA-256(canonical_bytecode)[0..4]   (4 bytes)
```

With up to 32 chunks of long-code chunked encoding, an mk1 card can encode up to `32 × 53 − 4 = 1692` bytes of canonical bytecode — vastly more than any plausible mk1 payload needs.

**(d) Naming alignment:** Rename the 20-bit per-encoding random tag from md1's "wallet identifier" to **`chunk_set_id`** across both repos. The current name conflicts with `Policy ID` and `Wallet Instance ID` and means neither of those things; it identifies "all chunks belonging to this card-encoding," nothing more. Wire format unchanged; documentation/code rename only. Captured as cross-repo coordination item.

**Downstream:** SPEC §2.5 expands from the chunk-type table into a full string-layer header subsection. SPEC §2.4 capacity numbers concretized.

---

### Q-6. Payload field order

**Provisional:** `header → stubs → fp → path → xpub` (xpub at 78 bytes pre-Q-7).

**Fresh-eyes finding:** The provisional ordering preserves three desirable properties: (i) recovery fast-filter via stubs at offset 2, (ii) BIP 380 origin block `[fp/path]xpub` reads in natural left-to-right order, (iii) structural mirror with md1's `header → optional metadata → path → main payload` shape.

**Lock:**

```
[bytecode_header   : 1 B]
[stub_count        : 1 B]
[policy_id_stubs   : 4 × N B]
[origin_fingerprint: 4 B]   ← present iff bytecode_header bit 2 set (see Q-8)
[origin_path       : 1 B (std-table indicator) OR 1 + 1 + 5N B (explicit path)]
[xpub_compact      : 73 B]  ← per Q-7
```

Total bytes for typical 1-stub mainnet card with std-table indicator and fingerprint present: `1 + 1 + 4 + 4 + 1 + 73 = 84 B`.

**Downstream:** SPEC §3.2 updated; xpub size figure changes from 78 to 73 (per Q-7).

---

### Q-7. xpub encoding

**Provisional:** Full 78-byte BIP 32 serialization.

**Fresh-eyes finding:** The provisional rationale's "5–13 bytes saved by compaction" range conflated two different things. Concrete decomposition of what's recoverable from `origin_path`:

| Field | Bytes | Recoverable from `origin_path`? |
|---|---|---|
| `xpub.version` | 4 | partial — could be derived from a network indicator (option (c)); kept on-wire in option (b) |
| `xpub.depth` | 1 | yes — `component_count(origin_path)` |
| `xpub.parent_fingerprint` | 4 | no — needs the actual parent xpub at derivation time |
| `xpub.child_number` | 4 | yes — last component of `origin_path` (with hardened bit) |
| `xpub.chain_code` | 32 | — |
| `xpub.public_key` | 33 | — |

Three real options:

| Option | Bytes | Lossless? | Tradeoff |
|---|---|---|---|
| (a) full-78 | 78 | yes | redundancy with `origin_path`; allows §4 rule 8 drift detection |
| (b) compact-73 | 73 | yes | drops only redundant fields; structurally eliminates the drift class |
| (c) compact-65 | 65 | no | also drops `parent_fingerprint` (irrecoverable) and `version` (needs network bit) |

(b) wins on bytes saved + structural-integrity (drift impossible by construction) at the cost of one narrow detection-loss case: an operator who picks the wrong standard-table indicator while engraving the right xpub bytes is detected at the per-card level under (a) (`XpubChildNumberMismatch`) but only at the wallet-assembly level under (b) (`Wallet Instance ID` mismatch in §5 step 4). That class of error is real but narrow, and the §5 step 4 check still catches it — one layer further out.

(c) was considered as a chunk-boundary saver for the 7-stub corner case, but the user explicitly chose not to optimize for that case.

**Lock:** (b) — compact-73. Drop `xpub.depth` and `xpub.child_number` from the on-wire serialization; reconstruct both from `origin_path` at decode time.

**Decoder rule:**

```
depth        := component_count(origin_path)
child_number := last_component(origin_path) including hardened-bit encoding
```

For a standard-table indicator, both come from the dictionary entry. For the explicit-path escape hatch, both come from the on-wire components.

**Compact-73 byte breakdown (what's preserved on-wire):**

```
[xpub.version          : 4 B]
[xpub.parent_fingerprint: 4 B]
[xpub.chain_code       : 32 B]
[xpub.public_key       : 33 B]
                         ────
                         73 B
```

`xpub.depth` and `xpub.child_number` are absent from the wire and reconstructed from `origin_path` per the decoder rule above. `xpub.version` is preserved on the wire; the locked compact form does not move version off-card.

**Limit-of-detection note (carries into §6 amendments under Q-8 below):**

Under compact-73, the only on-card consistency cross-check between `origin_path` and `xpub_compact` is a tautology in the standard-table-indicator case (the indicator *defines* depth/child_number; there is no second copy to compare against). An operator who picks the wrong standard-table indicator while engraving the right xpub bytes produces a card that decodes without error and reconstructs an xpub claiming the wrong derivation path. Detection of this class of error happens at the §5 step 4 Wallet Instance ID check — i.e., only when the user has an externally-anchored expected wallet identity to compare against. A single-wallet recovery without an external anchor will reconstruct the wrong-path xpub silently and may produce wrong addresses on first derivation. This is the cost of compact-73 over option (a) and is acknowledged here so it can be documented in SPEC §6.

**Downstream:**
- SPEC §3.6 reframed from "full 78-byte serialization" to "73-byte compact form (depth and child_number reconstructed from origin_path at decode time)".
- SPEC §3.2 `xpub_bytes` size changes from 78 to 73.
- SPEC §4 rule 8 (`XpubDepthMismatch`) is removed — drift is impossible by construction. Any pre-existing language about "depth field consistency" is redundant.
- The tradeoff that operator-picks-wrong-indicator errors are caught one layer later (at §5 step 4) is documented in §6 (privacy/operational notes) or in a new §3.6 subsection on the integrity story.
- This commits to the path-on-mk1 architecture (the 73-byte compact form depends on `origin_path` being present and authoritative) — a structural decision now visible at the wire level.

---

### Q-8. Privacy framing

**Provisional:** §6 of `SPEC_mk_v0_1.md` already addresses the threat surface, comparison vs md1, and operational recommendations.

**Fresh-eyes finding:** §6 enumerates leakage but doesn't expose the privacy-preserving choice that md1's bit-2 fingerprint flag offers. With the bytecode-header bit 2 still reserved in mk1's provisional spec, there is room to make `origin_fingerprint` optional, mirroring md1's bit-2 semantics at the cross-format pattern level.

**Lock:**

**(a) Bytecode-header bit 2 = fingerprint flag.**

```
mk1 bytecode header (v0.1, final):
  bits 7-4: version (0x0)
  bit 3:    reserved (MUST be 0)
  bit 2:    fingerprint flag    ← was reserved; now defined
  bit 1:    reserved (MUST be 0)
  bit 0:    reserved (MUST be 0)
```

When bit 2 is set: `origin_fingerprint` (4 bytes) is present in the payload at the position defined in §3.2.
When bit 2 is unset: `origin_fingerprint` is omitted; the payload moves directly from `policy_id_stubs` to `origin_path`.

**(b) §6 amendments:**

- New paragraph: "Cosigners who do not need their mk1 card to carry master-seed identification (e.g., when the policy card already carries fingerprints, or when `@N` assignment is performed out-of-band) MAY engrave with the fingerprint flag unset, omitting `origin_fingerprint`. Encoders SHOULD expose this as an explicit choice rather than a default."
- New paragraph on disposal: "When an xpub is rotated to a new master seed, the corresponding mk1 card SHOULD be physically destroyed. Engraved-on-steel cards that are no longer cryptographically active still carry the same privacy footprint as active cards (full transaction history reconstructable)."
- New paragraph on hand-off: "An mk1 card transferred to a wallet creator at provisioning time carries the same per-card privacy footprint as one read at recovery time. Cosigners SHOULD treat the act of hand-off — including any photograph, scan, or transcription that occurs during it — with the same operational discipline as they would the card itself in long-term storage."
- New paragraph on integrity detection limit (carries the Q-7 limit-of-detection note into normative recommendations): "Under the locked compact-73 xpub form, an operator who selects the wrong standard-table path indicator while engraving the correct xpub bytes produces a card that decodes without error but reconstructs an xpub claiming the wrong derivation path. Detection of this class of error happens at the Wallet Instance ID check (§5 step 4), which requires the user to hold an externally-anchored expected wallet identity. Cosigners performing single-wallet recoveries without an external anchor SHOULD verify the first address derived from the reconstructed xpub against an independently-recorded expected address before proceeding with funds movement."

**Privacy-mode interactions:**

| md1 fp block | mk1 origin_fp | recovery `@N` assignment |
|---|---|---|
| present | present | doubly-linked (most rigorous, least private) |
| present | absent | md1 owns the linkage; mk1 stays clean |
| absent | present | mk1 owns the linkage per-card |
| absent | absent | manual assignment at recovery (max privacy) |

A privacy-conscious cosigner can choose "absent on mk1" while the policy card may or may not carry their fingerprint. Cosigners participating in multiple wallets across different privacy regimes get per-card flexibility.

**Wire cost:** zero (bit was already reserved). 4 bytes saved per card when fingerprint omitted. Defaults are encoder-CLI choice, not spec-level.

**Downstream:**
- SPEC §3.1 bytecode-header bit 2 newly defined.
- SPEC §3.2 `origin_fingerprint` annotated as conditional on bit 2.
- SPEC §3.4 (origin fingerprint) section adds: "Present only if bytecode-header bit 2 is set; otherwise omitted."
- SPEC §4 (validity rules) adds: "Encoders MUST set the fingerprint flag iff `origin_fingerprint` is present in the payload, and decoders MUST reject any state where the flag and the payload disagree."
- SPEC §6 amendments per (b) above.

---

### Q-9. Md/mk split into shared `mc-codex32` crate — trigger condition

**Provisional:** "When both formats are implementation-validated and we can identify a stable shared API."

**Fresh-eyes finding:** D-13 already commits to the split; Q-9 is just the trigger. Premature extraction designs the shared API against md alone (forcing mk to retrofit). Late extraction is just code duplication, not a correctness risk.

**Lock:** Split trigger = **both md-codec and mk-codec at v1.0 with cross-validated conformance vectors and stable public APIs.**

Concretely:
- md-codec at v0.8.0; v1.0 gates on BIP-draft submission + round-trip test corpus + decoder error-path conformance.
- mk-codec at v0; v1.0 gates on first round-trip working + initial vector corpus + BIP-draft alignment.

Until then: fork-from-md-codec per D-13. Both reference implementations carry their own copy of the BCH primitives.

**Downstream:** SPEC §10 acknowledges deferred extraction. DECISIONS.md Q-9 row updated with the trigger.

---

### Q-10. Family-stable generator string

**Provisional:** `mk-codec X.Y` where `X.Y` is the major.minor of the published reference crate. Mirrors md1's `md-codec X.Y`.

**Fresh-eyes finding:** No live counterargument. Patch-version stability preserves vector validity across bug-fix releases; HRP-letter prefix gives natural namespacing across the codex32 family (`ms-codec`, `md-codec`, `mk-codec`); when D-13's split happens, `mc-codex32 X.Y` becomes the third token.

**Lock:** `mk-codec X.Y`.

**Downstream:** SPEC §7 confirms the convention. Vector files (when written) will use this token for family-stable SHA-256 anchoring.

---

## 2. Spec ripple — concrete amendments

The locks above translate to the following concrete edits across the spec, decisions log, BIP draft, and reference-crate scaffold. The implementation plan (subsequent step) sequences these.

### `design/SPEC_mk_v0_1.md`

- **§2.3** — Replace `MK_REGULAR_CONST = 0x???????????????` and `MK_LONG_CONST = 0x?????????????????` placeholders with the concrete locked values. Add the Python reproducer block.
- **§2.4** — Concretize chunked-fragment-per-chunk capacity (45 regular / 53 long), max chunks (32), cross-chunk integrity hash (4 bytes).
- **§2.5** — Expand from chunk-type table to full string-layer header layout: 2-char single (version + type), 8-char chunked (version + type + chunk_set_id + total_chunks + chunk_index). Use the new `chunk_set_id` naming.
- **§3.1** — Bytecode header: define bit 2 as fingerprint flag; remaining bits 0, 1, 3 still reserved MUST-be-0. Make the cross-format alignment with md1 explicit in the prose: "mk1's bytecode header mirrors md1's bit-allocation shape — 4-bit version field plus 4 flag/reserved bits — and shares bit-2 semantics ('optional fingerprint-related block follows'). Each format's specific block contents differ (md1: 5N-byte fingerprints block; mk1: 4-byte `origin_fingerprint`); the bit-level convention is shared to enable a common header-parsing helper."
- **§3.2** — Update payload field order to reflect `origin_fingerprint` as conditional on bit 2; `xpub_bytes` size changes from 78 to 73.
- **§3.3** — Tighten Q-2 rationale: drop the "matches md1 chunk-header convention" claim; replace with the explicit birthday-bound math `P(collision) ≈ k(k−1)/(2·2³²) ≈ 2.85×10⁻⁷` for `k = 50` wallets, and note that even 24 bits (`≈ 7.3×10⁻⁵`) clears the threat model.
- **§3.4** — Add: "Present only if bytecode-header bit 2 is set."
- **§3.5** — Update path-component cap from 32 to 10.
- **§3.6** — Reframe from "full 78-byte serialization" to compact-73 form; document depth/child_number reconstruction rules; remove or rephrase the "5–13 bytes saved" comparison since the actual lossless saving is exactly 5; add the "drift impossible by construction; operator-pick-wrong-indicator caught at §5 step 4 instead" subsection.
- **§4** — Remove rule 8 (`XpubDepthMismatch`) — impossible by construction under compact-73. Add: encoder/decoder MUST agree on fingerprint flag presence vs `origin_fingerprint` payload presence.
- **§5** — Add "Authority precedence" subsection: when md1 supplies per-`@N` paths and mk1 supplies `origin_path`, mk1's path is authoritative; mismatch MUST fail recovery with a precise error.
- **§6** — Three new paragraphs: optional-fingerprint-engraving privacy mode; disposal of rotated cards; hand-off footprint.
- **§7** — Confirm `mk-codec X.Y` convention.
- **§9** — Empty out the open-questions table; replace with a "Closures (2026-04-29)" pointer to this design doc.

### `design/DECISIONS.md`

- Convert Q-1 through Q-10 from open questions to closed decisions, each with a one-sentence summary of the locked answer and a pointer to this design doc for full rationale.
- Add D-14: cross-format header parsing alignment with md1 (string-layer header structure mirrored, bytecode-header version field shared, bit-2 semantics aligned at the "optional fingerprint block follows" pattern level).
- Add D-15: `chunk_set_id` rename across both repos (cross-repo coordination).

### `bip/bip-mnemonic-key.mediawiki`

The BIP draft is currently a 327-line skeleton. The closure design supplies the concrete content for:

- Encoding-layer §"Target residue constants": fill in the locked hex values + Python reproducer.
- Encoding-layer §"Length envelope": fill in the locked capacity numbers.
- Encoding-layer §"Header" subsection: fill in the locked chunked-header structure.
- Bytecode-layer §"Bytecode header": fill in bit 2 = fingerprint flag.
- Bytecode-layer §"Payload field order": the locked field order.
- Bytecode-layer §"Path encoding": fill in the locked component cap (10) and explicit encoding rules.
- Bytecode-layer §"xpub encoding": the locked compact-73 form with reconstruction rules.
- §"Linkage to MD": authority-precedence subsection.
- §"Privacy considerations": the §6 amendments.

### `crates/mk-codec/`

- Constants module: define `MK_REGULAR_CONST`, `MK_LONG_CONST` with the locked values.
- `BytecodeHeader` parsing: handle bit 2 fingerprint flag.
- `KeyCard` struct: `origin_fingerprint: Option<[u8; 4]>` to reflect the optional fingerprint.
- `XpubCompact` representation: 73-byte form with a method to reconstruct the full 78-byte BIP 32 serialization on demand.
- Path-component cap constant: `MAX_PATH_COMPONENTS = 10`.

---

## 3. Cross-repo coordination

These items require action in the `descriptor-mnemonic` repo, not this one. Captured here so the handoff is explicit.

**(1) `chunk_set_id` rename — sequencing pin.** md-codec's BIP and reference implementation currently use "wallet identifier" for the 20-bit per-encoding random tag in the chunked-header. Rename to `chunk_set_id` (or another agreed neutral term) to disambiguate from `Policy ID` and `Wallet Instance ID`. Wire format unchanged; this is purely a documentation and code-symbol rename. Cheap because md-codec has no external users yet.

**Sequencing requirement:** the rename MUST land in md-codec (likely as a docs-and-symbols-only release such as md-codec v0.9.0) **before** mk1's BIP draft is submitted. mk1's BIP cites md1 by field name; mk1 cannot be published referencing a name md1 itself does not use. The descriptor-mnemonic repo owns the trigger to release the renamed md-codec; mk1's BIP-draft completion gate inherits a hard dependency on that release.

**(2) Per-`@N` path tag-byte allocation in MD bytecode (Q-4 in DECISIONS.md).** mk1's closure declares the authority-precedence semantics: when both formats supply path information, mk1 is authoritative. The wire-format question — which tag byte md1 allocates for per-`@N` paths, whether to extend the unallocated `0x36+` range or backfill `0x24-0x32` — belongs to descriptor-mnemonic's next phase. mk1's spec only needs to know that md1's per-`@N` paths are *descriptive* (sanity-check role) when mk1 cards are present.

**(3) Header-parsing primitives extraction readiness.** Both formats' bytecode headers now share structure (4-bit version field + format-specific flag bits, with bit 2 commonly meaning "optional fingerprint-related block follows"). When D-13's `mc-codex32` extraction happens (Q-9 trigger), the shared parser can extract version generically and pass format-specific flag interpretation back to each format's higher-level layer. Both repos' implementations should converge to a common header-parsing helper signature in anticipation, even before extraction.

---

## 4. Pre-BIP-submission audit items

Closures above are sufficient for first-implementation-milestone work and BIP-draft writing. The following items are *not blockers for closure* but MUST be resolved before formal BIP submission per D-11.

**(1) Structural-relationship audit of `MK_REGULAR_CONST` / `MK_LONG_CONST` against the BIP 93 BCH polynomial.** Verify there are no accidental structural relationships: weight-distribution analysis under the new target, intersection of mk1 codeword space with md1 and codex32 codeword spaces, confirmation that error-correction guarantees (8-character detection, 4-substitution correction, etc.) hold under the new constants. Andrew Poelstra is the natural reviewer per D-11's coordination note.

**(2) HRP `mk` collision verification per D-9.** Search SLIP-0173 (informal segwit-HRP registry), recent bitcoin-dev mailing-list archives, and BIPs PR history for any soft `mk` claim. None expected, but confirmation is the gate before formal SLIP-0173 registration. Alternatives `mx`, `mkc`, `mpk` documented in D-9 if collision is found.

**(3) BIP-draft cross-reference completeness.** mk1's BIP draft must cross-reference: BIP 93 (codex32 plumbing reuse), BIP 32 (xpub serialization), BIP 380 (origin notation), BIP 388 (wallet policy and template framing for Policy ID semantics), and the published md1 BIP (linkage protocol, shared-parser conventions, `chunk_set_id` field). Any post-rename of "wallet identifier" → `chunk_set_id` in md1 must land before mk1's draft is finalized (see §3 item (1) above for the sequencing pin).

**(4) Decoder Error-variant enumeration ↔ negative-vector parity.** The locked SPEC §4 validity rules (after this closure: removal of rule 8, addition of fingerprint-flag-payload-consistency rule, path-cap update from 32 to 10) define the rejection cases. Pre-submission, every reject case in §4 MUST map to a uniquely-named `Error` variant in the `mk-codec` reference crate, and every variant MUST have at least one planned negative test vector. This mirrors md-codec's 30-negative-vectors-one-per-Error-variant conformance contract (see md1 BIP §"Test vectors" and the `descriptor-mnemonic` repo's negative-vector taxonomy). Cross-implementation conformance depends on this parity; landing it pre-submission avoids a thrash where independent implementations surface different error variants for the same input.

---

## 5. What this closure does NOT decide

To be explicit about scope:

- **md1 bytecode-header re-litigation** — the user surfaced this option mid-session. Out of scope here; mk1's locks do not require any md1-side wire-format change. If md1-specific concerns surface during the descriptor-mnemonic repo's Q-4 work, they belong to that repo's process.
- **MuSig2 / aggregate-key support** — out of scope per SPEC §1; future format milestone.
- **Compact xpub option (c)** — explicitly considered and rejected in this closure (parent_fingerprint loss not justified given the user's stated indifference to the 7-stub corner case).
- **Different BCH polynomial** — explicitly considered and rejected in this closure (D-10's reuse-with-domain-separation calculus reaffirmed; bytecode-layer compaction via Q-7 (b) achieves comparable savings at a fraction of the cost).

---

## Appendix — Provenance

This closure design was produced 2026-04-29 in a Claude Code session, applying the "re-litigate from scratch" framing (the costliest of the four proposed approaches at session start) to all ten open questions. The user drove the decisions; the assistant surfaced fresh tradeoffs (notably correcting the chunking math under chunk-header overhead, surfacing the path-with-xpub-vs-with-descriptor architectural question, and the parser-reuse pressure that motivated cross-format alignment of string-layer headers and bit-2 semantics). No agent-review checkpoints have been run on this design yet — those land at the implementation-plan stage and during BIP-draft review.
