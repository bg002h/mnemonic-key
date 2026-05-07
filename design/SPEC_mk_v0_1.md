# `mk1` v0.1 Design Spec — Mnemonic Key card

**Status:** v0.1 wire format locked. All Q-1..Q-10 closures landed 2026-04-29; see [closure design](../docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md) for fresh-eyes rationale. Implementation work tracked in [`IMPLEMENTATION_PLAN_mk_v0_1.md`](./IMPLEMENTATION_PLAN_mk_v0_1.md).
**Companion documents:**
- Closure design (locks for Q-1..Q-10): [`docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`](../docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md)
- Decisions log: [`DECISIONS.md`](./DECISIONS.md)
- Implementation plan: [`IMPLEMENTATION_PLAN_mk_v0_1.md`](./IMPLEMENTATION_PLAN_mk_v0_1.md)
- Follow-ups: [`FOLLOWUPS.md`](./FOLLOWUPS.md)
- MD BIP draft (sibling repo): [`bip-mnemonic-descriptor.mediawiki`](https://github.com/bg002h/descriptor-mnemonic/blob/main/bip/bip-mnemonic-descriptor.mediawiki)
- MD design docs (sibling repo): [`design/`](https://github.com/bg002h/descriptor-mnemonic/tree/main/design)

This document specifies the wire format for `mk1`-prefixed strings. All ten v0.1 open questions are closed; pre-BIP-submission audit items (NUMS structural audit, HRP `mk` collision check, BIP cross-reference completeness, decoder Error-variant ↔ negative-vector parity) are tracked in `FOLLOWUPS.md` at tier `pre-bip-submission`.

---

## §1. Scope

`mk1` encodes one extended public key (xpub) plus its origin metadata, in a codex32-derived BCH-checksummed string designed to engrave alongside MD-encoded policy cards. The intended use case is foreign-xpub multisig recovery: one cosigner backs up their xpub on its own card, separate from the policy card and from other cosigners' xpubs.

In scope:

- BIP 32 extended public keys (no extended private keys).
- BIP 380 origin notation (fingerprint + derivation path).
- One or more Policy ID linkage stubs identifying the MD-encoded policy card(s) this xpub serves.
- Single-string and chunked encodings (mirroring `md1`'s chunk-type taxonomy).

Out of scope for v0.1:

- Extended private keys (xprv). Backing up secret material is BIP 93's job.
- BIP 327 MuSig2 aggregate keys. Future milestone.
- Embedded wallet-policy fragments. Wallet-policy belongs on the MD card; mk1 is key-only.
- Watch-only descriptor reconstruction. mk1 is a key backup; the descriptor reassembly happens at recovery time using policy card + N key cards.

## §2. String Layer

### 2.1 HRP

`mk` (lowercase, 2 characters). Separator is `1` per BIP 173, giving the prefix `mk1`.

### 2.2 BCH plumbing

mk1 reuses BIP 93 (codex32) BCH generator polynomials verbatim. Per **D-10** in `DECISIONS.md`, this preserves codex32's error-correction guarantees without requiring an independent polynomial search.

Domain separation between mk1, md1, and ms1 (codex32) is provided by:

1. **HRP-mixing** (BIP 173-style HRP expansion folded into the polymod's initial state).
2. **Per-format target residue constants**, NUMS-derived from a per-format domain string.

### 2.3 NUMS-derived target constants

Locked per closure Q-1:

```text
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

Cross-check: applying the same procedure to `b"shibbolethnums"` reproduces md1's published `T_REGULAR = 0x0815c07747a3392e7` and `T_LONG = 0x205701dd1e8ce4b9f47`, confirming derivation correctness.

The string itself is the audit trail (NUMS — "Nothing Up My Sleeve"): any reader can recompute the SHA-256 and verify the constants follow from it.

### 2.4 Length envelope

Capacity numbers (mirror md1; concrete identifiers from `crates/mk-codec/src/consts.rs`):

```text
single-string regular code:    48 bytes payload   (SINGLE_STRING_REGULAR_BYTES)
single-string long code:       56 bytes payload   (SINGLE_STRING_LONG_BYTES)
chunked-fragment regular code: 45 bytes per fragment   (CHUNKED_FRAGMENT_REGULAR_BYTES)
chunked-fragment long code:    53 bytes per fragment   (CHUNKED_FRAGMENT_LONG_BYTES)
max chunks per card:           32                  (MAX_CHUNKS)
cross-chunk integrity hash:    SHA-256(canonical_bytecode)[0..4]   (4 bytes)
```

mk1's smallest valid bytecode (1-byte header + 1-byte stub_count + 4-byte single stub + std-table path indicator + 73-byte compact xpub, fingerprint omitted) is `1 + 1 + 4 + 1 + 73 = 80 bytes` — already above `SINGLE_STRING_LONG_BYTES = 56`. **Every conforming v0.1 mk1 KeyCard therefore encodes as a chunked card** (typical 1-stub-with-fingerprint case is 84 bytes, lands in 2 long-code chunks). The `SingleString` chunk-type variant is reserved for future format extensions whose bytecode could shrink below 56 bytes (e.g., the Compact-65 mode discussed in §3.6); it is wire-defined for forward compatibility but unreachable for v0.1 encoders.

With up to 32 long-code chunks, an mk1 card can encode up to `32 × 53 − 4 = 1692` bytes of canonical bytecode — vastly above any plausible mk1 payload.

### 2.5 String-layer header

mk1's string-layer header structure mirrors md1's exactly (per D-14). Two variants:

**Single-string header** (2 chars):

```text
char 0:  version       (5 bits)   — 0x0 in v0.1; decoders MUST reject unknown versions
char 1:  type = 0x00   (5 bits)   — SingleString
```

**Chunked header** (8 chars):

```text
char 0:    version          (5 bits)
char 1:    type = 0x01      (5 bits)   — Chunked
chars 2-5: chunk_set_id     (20 bits, random per-encoding)
char 6:    total_chunks     (5 bits, semantic range 1..=32, encoded as count − 1)
char 7:    chunk_index      (5 bits, range 0..total_chunks)
```

**Wire encoding for `total_chunks`:** the 5-bit field carries `count − 1` so the semantic range `1..=32` maps onto the wire range `0..=31`. Encoders MUST emit `count − 1`; decoders MUST add 1 after reading the 5-bit field. (A literal "1..=32" wire encoding is impossible because a 5-bit field holds 32 distinct values whose minimum is 0, not 1.)

**Wire encoding for `chunk_set_id`:** the 20-bit value is packed into chars 2–5 in big-endian 5-bit-symbol order — bits 19..15 in char 2, 14..10 in char 3, 9..5 in char 4, 4..0 in char 5. Decoders reconstruct the value via `(char2 << 15) | (char3 << 10) | (char4 << 5) | char5`.

`chunk_set_id` is opaque to the format; its only purpose is mismatch detection during reassembly. Encoders SHOULD generate it from a cryptographically secure random source at first encoding and reuse the same value for all subsequent re-encodings of the same card.

Chunk-type byte assignments:

| Type byte | Variant | Use |
|---|---|---|
| `0x00` | `SingleString` | Payload + headers fit in one long-code string |
| `0x01` | `Chunked` | Default for typical mk1 cards |
| `0x02..0x1F` | reserved | Decoders MUST reject; range exhausts the 5-bit field |

The `type` field is exactly 5 bits wide; the reserved range `0x02..0x1F` covers all unallocated values without spillover space. Future format extensions cannot widen the field without bumping the version byte.

No additional types required for v0.1 (D-5 — one xpub per card, atomic).

### 2.6 Cross-chunk integrity hash

For chunked cards, the canonical bytecode (from §3) is followed by a 4-byte `cross_chunk_hash` before chunk-splitting:

```text
cross_chunk_hash = SHA-256(canonical_bytecode)[0..4]
```

The hash is appended to the canonical bytecode, and the resulting byte stream is split into chunk fragments. Decoders compute the hash over the reassembled bytecode (excluding the trailing 4 hash bytes) and verify it matches the trailing bytes. Mismatch MUST cause the decode to fail with an error specifically identifying the cross-chunk integrity check (`Error::CrossChunkHashMismatch`).

## §3. Bytecode Layer

### 3.1 Bytecode header

The first byte of the bytecode payload (after chunk-header reassembly) is the **mk1 bytecode header**:

```text
bit 7-4: version         (4 bits)   — 0x0 in v0.1
bit 3:   reserved        (1 bit)    — MUST be 0 in v0.1
bit 2:   fingerprint flag (1 bit)   — 1 if origin_fingerprint is present, 0 if omitted
bit 1:   reserved        (1 bit)    — MUST be 0 in v0.1
bit 0:   reserved        (1 bit)    — MUST be 0 in v0.1
```

Valid v0.1 header values: `0x00` (no fingerprint) and `0x04` (fingerprint present). Other values MUST be rejected.

mk1's bytecode header mirrors md1's bit-allocation **shape** — 4-bit version field plus 4 flag/reserved bits — and shares **bit-2 semantics** ("optional fingerprint-related block follows"). Each format's specific block contents differ (md1: 5N-byte fingerprints block, one fp per master; mk1: 4-byte `origin_fingerprint`); the bit-level convention is shared to enable a common header-parsing helper when D-13's `mc-codex32` extraction happens (Q-9 trigger).

The two formats' **specific bit allocations diverge from bit 3 onward**. As of mk-codec v0.2.0 / md-codec v0.10.0:

- **mk1** (this format) reserves bits 0, 1, 3 (encoders MUST emit zero; decoders MUST reject non-zero). Bit 2 is the optional-fingerprint flag.
- **md1** reserves bits 0, 1 (zero), uses bit 2 as the optional-fingerprint flag (shape-shared with mk1), and reclaimed bit 3 as the OriginPaths flag in [md-codec v0.10.0](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.10.0) (1 = per-`@N` origin-path block via `Tag::OriginPaths = 0x36`; 0 = shared-path declaration via `Tag::SharedPath = 0x34`).

The shape-shared property survives the divergence: a future `mc-codex32` shared parser can extract the 4-bit version generically and pass the 4 flag bits back to each format's higher-level layer for format-specific interpretation. The bit-2 fingerprint-flag convention remains shared between mk1 and md1; bits 0, 1, 3 are independent allocations.

### 3.2 Payload field order

After the bytecode header, the payload encodes the following fields in order (per closure Q-6):

```text
[bytecode_header   : 1 B]
[stub_count        : 1 B; MUST be ≥ 1]
[policy_id_stubs   : 4 × N B]
[origin_fingerprint: 4 B]   ← present iff bytecode_header bit 2 set
[origin_path       : 1 B (std-table indicator) OR 3..=52 B (explicit: 0xFE + count + 1..=10 LEB128 components, ≤5 B each; see §3.5)]
[xpub_compact      : 73 B]
```

Total bytes for a typical 1-stub mainnet card with std-table indicator and fingerprint present: `1 + 1 + 4 + 4 + 1 + 73 = 84 B`.

**Rationale for ordering** (closure-locked):

- Stubs first → a recovery tool scanning many cards can fast-filter by Policy ID before parsing the rest.
- Origin fingerprint + path next → matches BIP 380 origin notation `[fp/path]` reading order.
- Xpub last → fixed-size, end-of-payload, simplest streaming-parser shape; also positions the largest field at a known offset.

### 3.3 Policy ID stub format

Each stub is **4 bytes** = the top 32 bits of the MD-encoded policy card's `SHA-256(canonical_bytecode)` (closure Q-2 lock).

Why 4 bytes:

- The stub is a **human-indexing aid**, not a cryptographic primitive. The cryptographic check happens at recovery time when the xpub is plugged into the policy and the `Wallet Instance ID` is recomputed from the assembled descriptor (see §5 step 4).
- Birthday-bound collision probability among `k = 50` stubs at 32 bits: `P(collision) ≈ k(k−1) / (2 · 2³²) ≈ 2.85 × 10⁻⁷` ≈ 0.00003% — effectively zero for the 50-wallet ceiling per D-7. Even 24 bits clears the threat model (`≈ 7.3 × 10⁻⁵`); 32 bits provides a conservative margin.
- Each stub costs 4 bytes on the wire. Round-byte alignment is friendlier to implementers than sub-byte packing.

`stub_count` is encoded as a single byte, so 1–255 stubs are allowed. Practical limit is set by chunk-payload capacity, not by the count field.

### 3.4 Origin fingerprint

The 4-byte BIP 32 master fingerprint, identifying the seed from which this xpub was derived. Verbatim from the BIP 380 origin-notation `[fp/...]` prefix.

**Present only if bytecode-header bit 2 is set; otherwise omitted from the payload entirely** (closure Q-8).

### 3.5 Origin path encoding

mk1-internal indicator-byte path dictionary plus a `0xFE` explicit-path escape hatch. The path encodes as a 1-byte indicator with two cases:

> **Path dictionary divergence note (v0.2.2).** Earlier mk-codec releases (≤ v0.2.1) described this dictionary as mirroring md1's `Tag::SharedPath` table byte-for-byte. md-codec v0.10.0 carried a compatible `Tag::OriginPaths` table at the same byte-for-byte tip, then md-codec v0.11 dropped path dictionaries from md1 entirely as part of the v0.11 wire-format cleanup (per [`descriptor-mnemonic/design/SPEC_v0_11_wire_format.md` §1.4](https://github.com/bg002h/descriptor-mnemonic/blob/main/design/SPEC_v0_11_wire_format.md)). As of mk-codec v0.2.2, mk1's standard-table dictionary is **standalone** — it is mk1-internal and no longer a sibling mirror of an md1 table. The 14-entry table below is the canonical mk1 source of truth; future entries are an mk1-side decision and have no md1 counterpart.

**Case A — standard-table indicator** (1 byte total):

| Indicator | Path |
|---|---|
| `0x01` | `m/44'/0'/0'` (BIP 44 mainnet) |
| `0x02` | `m/49'/0'/0'` (BIP 49 mainnet) |
| `0x03` | `m/84'/0'/0'` (BIP 84 mainnet) |
| `0x04` | `m/86'/0'/0'` (BIP 86 mainnet) |
| `0x05` | `m/48'/0'/0'/2'` (BIP 48 segwit-v0 multisig mainnet) |
| `0x06` | `m/48'/0'/0'/1'` (BIP 48 nested-segwit multisig mainnet) |
| `0x07` | `m/87'/0'/0'` (BIP 87 multisig mainnet) |
| `0x11`–`0x17` | Testnet variants — full 7-entry coverage as of v0.2.0 |

(The 14-entry table is mk1-internal as of mk-codec v0.2.2; see the divergence note at the start of §3.5.)

**Note on `0x16` (history).** mk1 v0.1.x reserved `0x16`, awaiting alignment with md1's then-corresponding dictionary entry. md-codec v0.9.0 (and v0.10.x) carried a compatible table; mk-codec v0.2.0 added `(0x16, "m/48'/1'/0'/1'")` to `STANDARD_PATHS` at that time. The change is wire-additive: v0.1.x decoders reject `0x16` with `Error::InvalidPathIndicator(0x16)`; v0.2+ decoders accept and resolve to the BIP 48 testnet nested-segwit path. md-codec v0.11+ has since dropped path dictionaries entirely; the historical mirror invariant is retired (see the divergence note above and `design/FOLLOWUPS.md::path-dictionary-mirror-stewardship`).

**Case B — explicit-path escape hatch**, marked by indicator `0xFE`:

```text
[0xFE]
[component_count: 1 byte; MUST be in 1..=10]
[component_1 ... component_N: each LEB128-encoded u32]
```

Each component encodes a u32 BIP 32 child number (hardened bit set in the high bit per BIP 32 convention; the LEB128 carries the full 32 bits including hardened-marker).

**Component-byte sizing.** LEB128 of a u32 takes 1–5 bytes depending on the encoded value: any u32 with bit 28 or higher set (which includes every hardened component, since the hardened-marker is bit 31) requires the full 5 bytes; non-hardened components < 2²⁸ take 1–4 bytes. The closure design's and implementation plan's worst-case bound `1 + 1 + 5N B` is the upper limit for an N-component explicit path; for typical BIP-style paths (which are mostly hardened), this bound is also the typical case. Concretely: `m/48'/0'/0'/2'` (4 components, all hardened) explicit-encodes to `1 + 1 + 4×5 = 22 bytes` — though such a path would normally use the 1-byte standard-table indicator `0x05` instead. The `5N` shorthand in §3.2 below uses this worst-case bound.

**Path-component cap = 10** (closure Q-3). BIP 32 itself allows depth ≤ 255; real wallets use ≤ 6. The 10-component cap bounds chunk-size attacks without rejecting any plausibly real path; tighter caps don't lock out real users while bounding worst-case malicious-path inflation more aggressively. Decoders MUST reject `component_count > 10` with `Error::PathTooDeep`.

Indicators `0x00`, `0x08`–`0x10`, `0x18`–`0xFD`, and `0xFF` are **reserved** and MUST NOT be emitted by encoders. Decoders MUST reject reserved indicator bytes with `Error::InvalidPathIndicator`.

### 3.6 xpub encoding (compact-73)

mk1 v0.1 encodes xpubs in a **73-byte compact form** (closure Q-7):

```text
[xpub.version          : 4 B]   — network-specific (mainnet xpub = 0x0488B21E)
[xpub.parent_fingerprint: 4 B]
[xpub.chain_code       : 32 B]
[xpub.public_key       : 33 B] — compressed secp256k1
                         ────
                         73 B
```

The fields `xpub.depth` and `xpub.child_number` are absent from the wire and reconstructed at decode time from `origin_path`:

```
depth        := component_count(origin_path)
child_number := last_component(origin_path) including hardened-bit encoding
```

For a standard-table indicator, both come from the dictionary entry. For the explicit-path escape hatch, both come from the on-wire components.

**Why compact-73:** the dropped fields (`depth`, `child_number`) are redundant with `origin_path`; carrying them on-wire risks the drift detected by md1-style drift-rule logic. Compact-73 is *lossless* — both fields are reconstructible from the path — and saves 5 bytes per card (~one row of typical hand-engraving). The drift class is impossible by construction.

**Limit-of-detection note (carries into §6 normative recommendations):** under compact-73, an operator who picks the wrong standard-table indicator while engraving the right xpub bytes produces a card that decodes without error and reconstructs an xpub claiming the wrong derivation path. The card's chain_code and public_key are correct (addresses derive correctly), but the path embedded in the reconstructed BIP 32 serialization is wrong. Detection of this class of error happens at the §5 step 4 Wallet Instance ID check — only when the user has an externally-anchored expected wallet identity to compare against. Single-wallet recoveries without an external anchor will reconstruct the wrong-path xpub silently. §6 recommends an out-of-band first-address verification before moving funds.

### 3.7 Bytecode header reserved bits

Reserved bits in the bytecode header (bits 0, 1, 3) MUST be 0 in v0.1. Decoders MUST reject any reserved bit set with `Error::ReservedBitsSet`. v0.X may allocate them for:

- Network indicator override (if mk1 is used cross-network for some reason).
- Optional human-readable wallet name field.
- Compact-65 mode (drop `xpub.version` + `xpub.parent_fingerprint`; lossy).

(Bit 2 was originally reserved; closure Q-8 allocated it as the fingerprint flag.)

## §4. Bytecode-Validity Rules

A decoder MUST reject mk1 bytecode that:

1. Has a bytecode header with version ≠ 0 in v0.1 (`Error::UnsupportedVersion`).
2. Has any reserved bit set in v0.1 (`Error::ReservedBitsSet`).
3. Has `stub_count == 0` (`Error::InvalidPolicyIdStubCount`).
4. Has an origin path indicator outside the defined table (`Error::InvalidPathIndicator`).
5. Has an explicit path with `component_count > 10` (or `== 0`) (`Error::PathTooDeep`).
6. Has any path component with the BIP 32 child-number high bits set in invalid ways, or LEB128 overflow (`Error::InvalidPathComponent`).
7. Has xpub version bytes that don't match a known network's xpub prefix (`Error::InvalidXpubVersion`).
8. Has xpub `public_key` bytes that do not parse as a valid compressed secp256k1 point (`Error::InvalidXpubPublicKey`). Realistically unreachable for inputs that pass BCH verification, but required for hand-constructed inputs fed directly to the bytecode decoder.
9. Truncates anywhere mid-field (`Error::UnexpectedEnd`).
10. Has trailing bytes after the xpub (`Error::TrailingBytes`).

Encoder-side invariant (not a decoder rule): encoders MUST set the bytecode-header fingerprint flag (bit 2) if and only if `origin_fingerprint` is present in the payload. The invariant is structurally undetectable at decode time (no length prefix distinguishes "flag set, fp present" from "flag unset, fp omitted"); a decoder follows the flag verbatim. A hand-crafted bytecode that violates the invariant decodes to a wrong-but-internally-consistent `KeyCard`; detection happens at the higher Wallet Instance ID check (§5 step 4).

Additional decoder rules at the string/chunking layer:

11. For chunked input: chunks share a common `chunk_set_id`; mismatched `chunk_set_id` MUST be rejected (`Error::ChunkSetIdMismatch`).
12. For chunked input: `chunk_index` values cover `0..total_chunks` exactly once; gaps, duplicates, or out-of-range values MUST be rejected (`Error::ChunkedHeaderMalformed`).
13. For chunked input: reassembled bytecode's trailing `cross_chunk_hash` MUST equal `SHA-256(canonical_bytecode)[0..4]` (`Error::CrossChunkHashMismatch`).
14. The 5-bit payload symbols, after BCH verification, MUST byte-align (i.e., the trailing pad bits of the final 5-bit symbol are zero). Non-zero pad bits MUST be rejected with `Error::MalformedPayloadPadding` (parallels md1's analog).

Note: the v0 spec sketch's `XpubDepthMismatch` rule is removed under compact-73 — `xpub.depth` is no longer carried on-wire, so drift between the depth field and the path is impossible by construction.

## §5. Linkage to MD

A key card with Policy ID stubs `[stub_1, ..., stub_N]` declares: "this xpub is intended to serve any MD-encoded policy whose canonical-bytecode SHA-256 prefix matches one of these stubs."

**Recovery flow:**

1. Decode the policy card. Compute its full 16-byte Policy ID = `SHA-256(canonical_bytecode)[0..16]`. Take the top 4 bytes as `policy_stub`.
2. For each candidate key card:
   a. Decode and extract its Policy ID stubs.
   b. Reject the card unless `policy_stub` matches one of its stubs.
3. For each accepted key card, plug its xpub into the corresponding `@N` slot in the policy template (matched by origin path or by user assignment).
4. Compute the **Wallet Instance ID** for the assembled wallet:

   ```text
   wallet_instance_id = SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]
   ```

   where `canonical_xpub_serialization` is the concatenation of each `@N`-resolved xpub's full 78-byte BIP 32 serialization, in placeholder-index order. Compare against the wallet identity the user expected to recover (e.g., a separately-anchored Wallet Instance ID, or against a digital backup record). Reject on mismatch.
5. Accept and proceed to address derivation.

Step 2 is the indexing aid (fast filter, template-level). Step 4 is the cryptographic per-instance check. Both are required for safe recovery.

**Implementation note:** the `compute_wallet_instance_id(canonical_bytecode, xpubs)` helper is provided by md-codec v0.8.0+. mk1 implementations integrating with md-codec ≥ 0.8.0 SHOULD use that helper directly rather than reimplementing the SHA-256 construction.

### 5.1 Authority precedence (mk1 ↔ md1 path information)

When both an mk1 card and an md1 card with per-`@N` paths participate in recovery for the same wallet, mk1's `origin_path` is **authoritative** for the xpub's derivation. md1's per-`@N` path is the policy's **expected** path (descriptive, sanity-check role).

Mismatch MUST cause **the recovery orchestrator** to reject the assembly. Per-format decoders are not required to be aware of cross-format context; the cross-format consistency check belongs to the orchestrator layer that sits above both decoders.

Implementations MUST surface a precise error identifying both the policy-side expected path and the key-side actual path so the user can diagnose which card's path information is wrong.

(The md1-side wire-format allocation — `Tag::OriginPaths = 0x36`, header bit 3 reclaimed as the OriginPaths flag — shipped in [md-codec v0.10.0](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.10.0); see md1's BIP §"Per-`@N` path declaration".)

## §6. Privacy

**An mk1 card alone is a strictly larger privacy footprint than the corresponding md1 policy card.**

A lost or photographed mk1 card reveals:

- The xpub → full transaction history (every receive/change address) for that account is reconstructable by anyone with the bitcoin chain.
- The origin fingerprint (when present) → identifies the master seed (4-byte fingerprint is weak but sufficient as a hint). Optional per closure Q-8.
- The origin path → identifies the BIP-style account family.
- One or more Policy ID stubs → identifies which MD-encoded wallets this xpub serves.

By contrast, an md1 policy card alone reveals only the **policy structure** (template) and optionally fingerprints (with stderr warning); it does not enable transaction-history reconstruction.

**Recommendations** (informational, not normative):

- Engrave mk1 cards on durable physical media but store with the same physical security as a seed backup. Safety-deposit boxes, fire safes, etc.
- Never photograph mk1 cards. The xpub's full transaction history is one OCR away.
- Cosigners who participate in multiple wallets have a privacy choice in the stub count: stamping multiple Policy IDs on one key card means recovery for any of those wallets reveals (to the recoverer) that the cosigner is in all of them. Cosigners who care about cross-wallet privacy should stick to one stub per card and engrave additional cards.
- **Optional fingerprint engraving (closure Q-8).** Cosigners who do not need their mk1 card to carry master-seed identification (e.g., when the policy card already carries fingerprints, or when `@N` assignment is performed out-of-band) MAY engrave with the fingerprint flag unset, omitting `origin_fingerprint`. Encoders SHOULD expose this as an explicit choice rather than a default.
- **Disposal of rotated cards.** When an xpub is rotated to a new master seed, the corresponding mk1 card SHOULD be physically destroyed. Engraved-on-steel cards that are no longer cryptographically active still carry the same privacy footprint as active cards (full transaction history reconstructable).
- **Hand-off discipline.** An mk1 card transferred to a wallet creator at provisioning time carries the same per-card privacy footprint as one read at recovery time. Cosigners SHOULD treat the act of hand-off — including any photograph, scan, or transcription that occurs during it — with the same operational discipline as they would the card itself in long-term storage.
- **Integrity detection limit (closure Q-7 → §6 normative).** Under the locked compact-73 xpub form, an operator who selects the wrong standard-table path indicator while engraving the correct xpub bytes produces a card that decodes without error but reconstructs an xpub claiming the wrong derivation path. Detection of this class of error happens at the Wallet Instance ID check (§5 step 4), which requires the user to hold an externally-anchored expected wallet identity. Cosigners performing single-wallet recoveries without an external anchor SHOULD verify the first address derived from the reconstructed xpub against an independently-recorded expected address before proceeding with funds movement.

## §7. Family-stable generator string

Locked per closure Q-10: `mk-codec X.Y` where `X.Y` is the major.minor version of the reference implementation's published crate. Mirrors `md-codec`'s convention.

Vector files use this token for family-stable SHA-256 anchoring. Patch-version bumps don't roll the token.

## §8. Out-of-scope items deferred

| Item | Rationale | Future version |
|---|---|---|
| xprv (extended private key) | Secret material is BIP 93's job; out of scope here | (no plan) |
| MuSig2 aggregate keys | Different primitive; needs its own design | mk v0.X |
| Compact-65 xpub (drop version + parent_fingerprint) | Lossy; parent_fingerprint loss not justified at v0.1's threat model. Reserved bits available if future requires. | mk v0.2+ |
| Xpub network override | Network is implied by the policy card today; cross-network use cases not yet driving | mk v0.X |

## §9. Closures (2026-04-29)

All ten v0.1 open questions Q-1..Q-10 are closed. See [`docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`](../docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md) for fresh-eyes rationale and the locks.

| ID | Locked answer | Section |
|---|---|---|
| Q-1 | Domain `b"shibbolethnumskey"`; constants 0x1062435f91072fa5c (regular), 0x41890d7e441cbe97273 (long) | §2.3 |
| Q-2 | 4-byte Policy ID stub | §3.3 |
| Q-3 | Path-component cap = 10 | §3.5 |
| Q-4 | mk1 declares authority precedence; md1 tag-byte allocation shipped as `Tag::OriginPaths = 0x36` in [md-codec v0.10.0](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.10.0) | §5.1 |
| Q-5 | Chunk types 0x00=SingleString, 0x01=Chunked; full string-layer header structure pinned | §2.5 |
| Q-6 | Payload field order: header → stubs → fp → path → xpub_compact | §3.2 |
| Q-7 | Compact-73 xpub form (drop redundant depth + child_number) | §3.6 |
| Q-8 | Bytecode header bit 2 = optional fingerprint flag | §3.1 + §6 |
| Q-9 | Split trigger: both formats v1.0 with cross-validated conformance vectors | (DECISIONS.md / FOLLOWUPS) |
| Q-10 | Family token `mk-codec X.Y` | §7 |

Pre-BIP-submission audit items (NUMS structural-relationship audit, HRP `mk` collision check, BIP cross-reference completeness, decoder Error-variant ↔ negative-vector parity) tracked in `FOLLOWUPS.md` at tier `pre-bip-submission`.

## §10. Reference implementation

`crates/mk-codec/` — reference implementation. v0.1 forks BCH primitives from `md-codec` per D-13. Eventual extraction to a shared `mc-codex32` crate (likely a third sibling repo per D-12) is committed but deferred per closure Q-9 (trigger: both md-codec and mk-codec at v1.0 with cross-validated conformance vectors).

## §11. BIP draft

`bip/bip-mnemonic-key.mediawiki` — BIP draft. Mirrors the structure of `bip/bip-mnemonic-descriptor.mediawiki` adapted for mk1's narrower scope. Pre-submission audit items must clear before formal BIP submission per D-11.

---

## Appendix A — provenance

This v0.1 spec was originally written 2026-04-29 as a follow-on artifact to `DECISIONS.md`. The Q-1..Q-10 closures landed the same day via a re-litigate-from-scratch design pass with the project author; the closure design is at `docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md` and was reviewed by an opus reviewer (8 should-address fixes applied inline before commit). The implementation plan is at [`IMPLEMENTATION_PLAN_mk_v0_1.md`](./IMPLEMENTATION_PLAN_mk_v0_1.md); per-phase opus reviews are persisted to `design/agent-reports/`.
