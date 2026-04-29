# `mk1` v0.1 Design Spec — Mnemonic Key card

**Status:** Draft, design-stage. Pre-spec sketch — not yet implementation-tested.
**Companion documents:**
- Decisions log: [`DECISIONS.md`](./DECISIONS.md)
- MD BIP draft (sibling repo): [`bip-mnemonic-descriptor.mediawiki`](https://github.com/bg002h/descriptor-mnemonic/blob/main/bip/bip-mnemonic-descriptor.mediawiki)
- MD design docs (sibling repo): [`design/`](https://github.com/bg002h/descriptor-mnemonic/tree/main/design)

This document sketches the wire format for `mk1`-prefixed strings. It is the first cut at concrete encoding decisions; many fields are marked **PROVISIONAL** and locked only when the BIP draft is written. Open questions from `DECISIONS.md` (Q-1 through Q-10) are pinned here as defaults that can change up to the BIP-submission gate.

---

## §1. Scope

`mk1` encodes one extended public key (xpub) plus its origin metadata, in a codex32-derived BCH-checksummed string designed to engrave alongside MD-encoded policy cards. The intended use case is foreign-xpub multisig recovery: one cosigner backs up their xpub on its own card, separate from the policy card and from other cosigners' xpubs.

In scope:

- BIP 32 extended public keys (no extended private keys).
- BIP 380 origin notation (fingerprint + derivation path).
- One or more wallet-ID linkage stubs identifying the MD-encoded policy card(s) this xpub serves.
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

**PROVISIONAL** (pin at BIP-draft time):

```text
domain_string = b"shibbolethnumskey"
h = SHA-256(domain_string)
MK_REGULAR_CONST = top_65_bits(h)   # 0x???????????????
MK_LONG_CONST    = top_75_bits(h)   # 0x?????????????????
```

The exact hex values are computed as part of writing the BIP draft. The domain string `"shibbolethnumskey"` is provisional; any independent string suffices, but it MUST differ from md1's `"shibbolethnums"`.

### 2.4 Length envelope

Single-string capacity for codex32 long-code is 56 bytes payload (after subtracting headers and the 15-character checksum). mk1's typical payload (78-byte xpub + ~8 bytes origin path + 4-byte fingerprint + 4-byte wallet-ID stub + headers) overruns this; **single-string mk1 is uncommon, multi-chunk is the norm**.

### 2.5 Chunk-type taxonomy

Mirrors `md1`. The chunk header's type nibble distinguishes:

| Type byte | Variant | Use |
|---|---|---|
| `0x00` | `SingleString` | Rare; only when payload + headers fit in one long-code string |
| `0x01` | `Chunked` | Default for typical mk1 cards |

No additional types required for v0.1 (D-5 — one xpub per card, atomic).

## §3. Bytecode Layer

### 3.1 Bytecode header

The first byte of the bytecode payload (after chunk-header reassembly) is the **mk1 bytecode header**. It mirrors `md1`'s header in structure but is interpreted in mk1's namespace.

```text
bit 7-4: version (4 bits)   — 0x0 in v0.1
bit 3-1: reserved (3 bits)  — MUST be 0 in v0.1
bit 0:   reserved flag      — MUST be 0 in v0.1
```

So the header byte is `0x00` in v0.1. Reserved bits/flags are gated for future use (e.g., a "compact xpub" bit if Q-7 is revisited).

### 3.2 Payload field order

**PROVISIONAL** (Q-6 in DECISIONS.md). After the bytecode header, the payload encodes the following fields in order:

```text
[bytecode header: 1 byte]
[wallet_id_stub_count: 1 byte; MUST be ≥ 1]
[wallet_id_stub_1 ... wallet_id_stub_N: 4 × N bytes]
[origin_fingerprint: 4 bytes]
[origin_path: variable]
[xpub_bytes: 78 bytes]
```

**Rationale for ordering** (recovery-friendly):
- Wallet-ID stubs first → a recovery tool scanning many cards can filter by wallet ID before parsing the rest.
- Origin fingerprint + path next → matches BIP 32 origin notation `[fp/path]`.
- Xpub bytes last → fixed-size, end-of-payload, simplest streaming-parser shape.

### 3.3 Wallet-ID stub format

**Naming note**: as of [md-codec v0.8.0](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.8.0) the 16-byte template-only hash that md1 originally called a "wallet ID" is renamed to **Policy ID** (it hashes the BIP 388 template only, not the assembled descriptor; two wallets sharing a template share a Policy ID). The mk1 stub adopts the renamed nomenclature throughout this spec; the wire-format and byte-level semantics are unchanged.

**PROVISIONAL** (Q-2 in DECISIONS.md): each stub is **4 bytes** = the top 32 bits of the MD-encoded policy card's `SHA-256(canonical_bytecode)`.

Why 4 bytes and not the full 16-byte wallet ID:
- The stub is a **human-indexing aid**, not a cryptographic primitive. The cryptographic check happens at recovery time when the xpub is plugged into the policy and the wallet ID is recomputed from the assembled descriptor.
- 4 bytes = ~4 billion distinct values. A user plausibly in 1–50 wallets has effectively zero collision probability.
- Each stub costs 4 bytes on the wire. A key card serving 3 wallets carries 12 bytes of stubs vs 48 bytes if full 16-byte IDs were used.
- Matches the existing chunk-header wallet-ID stub convention from md1.

`wallet_id_stub_count` is encoded as a single byte, so 1–255 stubs are allowed. Practical limit is set by chunk-payload capacity, not by the count field.

### 3.4 Origin fingerprint

The 4-byte BIP 32 master fingerprint, identifying the seed from which this xpub was derived. Verbatim from the BIP 380 origin-notation `[fp/...]` prefix.

### 3.5 Origin path encoding

**Mirrors `md1`'s `Tag::SharedPath` precedent** (D-3 in DECISIONS.md). The path encodes as a 1-byte indicator with two cases:

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
| `0x11`–`0x17` | Testnet variants of the above |

(Exact dictionary mirrors md1's `Tag::SharedPath` table.)

**Case B — explicit-path escape hatch**, marked by indicator `0xFE`:

```text
[0xFE]
[component_count: 1 byte]
[component_1 ... component_N: each LEB128-encoded u32]
```

Each component encodes a u32 BIP 32 child number (hardened bit set in the high bit per BIP 32 convention; the LEB128 carries the full 32 bits including hardened-marker).

**PROVISIONAL** (Q-3): `component_count ≤ 32`. BIP 32 itself allows depth ≤ 255; real wallets use ≤ 6. The 32-component cap bounds chunk-size attacks without rejecting any plausibly real path.

Indicators `0x00`, `0x08`–`0x10`, `0x18`–`0xFD`, and `0xFF` are **reserved** and MUST NOT be emitted by encoders. Decoders MUST reject reserved indicator bytes.

### 3.6 xpub encoding

**PROVISIONAL** (Q-7 in DECISIONS.md): full **78-byte BIP 32 serialization**, byte-identical to the binary form of a Base58Check-decoded xpub.

```text
[version: 4 bytes]              — network-specific (mainnet xpub = 0x0488B21E)
[depth: 1 byte]
[parent_fingerprint: 4 bytes]
[child_number: 4 bytes]
[chain_code: 32 bytes]
[public_key: 33 bytes]          — compressed secp256k1
```

Why the full 78 bytes (rather than a "compact" form dropping version/depth/parent-fingerprint):
- BIP 32 round-trip requires the full serialization. Recovery tools that hand the xpub back to a wallet need a valid xpub string (Base58Check), which needs the full bytes.
- Compact forms save 5–13 bytes per card. The savings don't change the chunking math (mk1 is multi-chunk regardless).
- Full 78 bytes has zero edge cases in implementation. Compact forms have several (network indicator coupling, depth-from-path-component-count, parent-fingerprint reconstruction).
- Future format extensions can add a "compact" bit in the bytecode header's reserved flags if compaction proves valuable.

### 3.7 Bytecode header reserved bits

Reserved bits in the bytecode header (bits 0–3) MUST be 0 in v0.1. v0.X may allocate them for:
- Compact xpub mode (drop version + depth → save 5 bytes).
- Network indicator override (if mk1 is used cross-network for some reason).
- Optional human-readable wallet name field.

## §4. Bytecode-Validity Rules

A decoder MUST reject mk1 bytecode that:

1. Has a bytecode header with version != 0 in v0.1 (`UnsupportedVersion`).
2. Has any reserved bit set in v0.1 (`ReservedBitsSet`).
3. Has `wallet_id_stub_count == 0` (`InvalidWalletIdStubCount`).
4. Has an origin path indicator outside the defined table (`InvalidPathIndicator`).
5. Has an explicit path with `component_count > 32` (`PathTooDeep`).
6. Has any path component with the BIP 32 child-number high bits set in invalid ways (`InvalidPathComponent`).
7. Has xpub version bytes that don't match a known network's xpub prefix (`InvalidXpubVersion`).
8. Has xpub `depth` field inconsistent with the encoded origin path's component count (`XpubDepthMismatch`) — this catches xpub-vs-path drift, an important integrity check.
9. Truncates anywhere mid-field (`UnexpectedEnd`).
10. Has trailing bytes after the xpub (`TrailingBytes`).

## §5. Linkage to MD

A key card with Policy ID stubs `[stub_1, ..., stub_N]` declares: "this xpub is intended to serve any MD-encoded policy whose canonical-bytecode SHA-256 prefix matches one of these stubs."

**Recovery flow:**

(The value formerly named "wallet ID" was renamed to **Policy ID** in [md-codec v0.8.0](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.8.0). Two wallets sharing an identical policy template share this value; the cryptographic per-instance verification happens at step 4 below via `WalletInstanceId`, not at the stub match in step 2.)

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

**Implementation note:** the `compute_wallet_instance_id(canonical_bytecode, xpubs)` helper is provided by md-codec v0.8.0+. mk1 implementations integrating with md-codec ≥0.8.0 SHOULD use that helper directly rather than reimplementing the SHA-256 construction.

## §6. Privacy

**An mk1 card alone is a strictly larger privacy footprint than the corresponding md1 policy card.**

A lost or photographed mk1 card reveals:

- The xpub → full transaction history (every receive/change address) for that account is reconstructable by anyone with the bitcoin chain.
- The origin fingerprint → identifies the master seed (4-byte fingerprint is weak but sufficient as a hint).
- The origin path → identifies the BIP-style account family.
- One or more wallet IDs → identifies which MD-encoded wallets this xpub serves.

By contrast, an md1 policy card alone reveals only the **policy structure** (template) and optionally fingerprints (with stderr warning); it does not enable transaction-history reconstruction.

**Recommendations** (informational, not normative):

- Engrave mk1 cards on durable physical media but store with the same physical security as a seed backup. Safety-deposit boxes, fire safes, etc.
- Never photograph mk1 cards. The xpub's full transaction history is one OCR away.
- Cosigners who participate in multiple wallets have a privacy choice in the wallet-ID stub count: stamping multiple wallet IDs on one key card means recovery for any of those wallets reveals (to the recoverer) that the cosigner is in all of them. Cosigners who care about cross-wallet privacy should stick to one wallet ID per card and engrave additional cards.

## §7. Family-stable generator string

**PROVISIONAL** (Q-10 in DECISIONS.md): `mk-codec X.Y` where `X.Y` is the major.minor version of the reference implementation's published crate. Mirrors `md-codec`'s convention.

Vector files (when written) use this token for family-stable SHA-256 anchoring. Patch-version bumps don't roll the token.

## §8. Out-of-scope items deferred

| Item | Rationale | Future version |
|---|---|---|
| xprv (extended private key) | Secret material is BIP 93's job; out of scope here | (no plan) |
| MuSig2 aggregate keys | Different primitive; needs its own design | mk v0.X |
| Compact xpub form | Bytecode-header reserved bit available; revisit if engraving cost matters | mk v0.2+ |
| Xpub network override | Network is implied by the policy card today; cross-network use cases not yet driving | mk v0.X |

## §9. Open questions explicitly punted

These are pinned with provisional answers above but are explicitly open for change up to the BIP-draft submission gate. Cross-referenced to `DECISIONS.md`:

| ID | Provisional answer | Lock at |
|---|---|---|
| Q-1 | NUMS string `b"shibbolethnumskey"` | BIP-draft submission |
| Q-2 | 4-byte wallet-ID stub | First implementation milestone |
| Q-3 | Path-component cap = 32 | First implementation milestone |
| Q-5 | Chunk types `SingleString=0x00`, `Chunked=0x01` (mirrors md1) | First implementation milestone |
| Q-6 | Payload field order: header → stubs → fingerprint → path → xpub | First implementation milestone |
| Q-7 | Full 78-byte xpub | First implementation milestone |
| Q-10 | Family token `"mk-codec X.Y"` | First implementation milestone |

## §10. Reference implementation

`crates/mk-codec/` (TBD). Initial scaffold forks BCH primitives from `md-codec` per D-13 in `DECISIONS.md`; eventual extraction to a shared `mc-codex32/` workspace member is committed but deferred for efficiency during the design phase.

## §11. BIP draft

`bip/bip-mnemonic-key.mediawiki` (TBD). Mirrors the structure of `bip/bip-mnemonic-descriptor.mediawiki` adapted for mk1's narrower scope.

---

## Appendix A — provenance

This v0.1 draft was written 2026-04-29 as a follow-on artifact to `design/mk/DECISIONS.md` (see that file's `## Conversation provenance` section). It captures the design decisions made in the same Claude Code session and pins provisional answers to open questions where the spec needs concrete values to be useful. None of the provisional values have been validated against an implementation yet; expect drift up until the first round-trip test succeeds.
