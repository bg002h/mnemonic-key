# v0.2.2 BIP test vector audit matrix — mnemonic-key (mk-codec)

Built 2026-05-07 per the v0.7.1 audit cycle plan
(`/home/bcg/.claude/plans/let-s-work-on-the-soft-waterfall.md`).

Scope: mk-codec is the reference impl of the mk1 wire format —
HRP `mk`, BIP-93-derived BCH **forked** (HRP-mixing + per-format NUMS
target residues drawn from `SHA-256(b"shibbolethnumskey")`), xpub-only
backup with optional origin fingerprint + path-dictionary indicator.
mk-codec consumes `bitcoin v0.32` for BIP-32 primitives.

Status legend: same as toolkit matrix.

---

## mk1 custom corpus

Source: `crates/mk-codec/tests/vectors/v0.1.json` — 40 vectors total
(18 clean V1..V18 + 22 negative N1..N23 with N22 reserved). Round-trip +
negative-rejection asserted via `tests/vectors.rs::every_vector_round_trips`
+ `tests/error_coverage.rs::every_negative_vector_maps_to_a_known_variant`.

### Clean vectors (V1..V18)

| # | Description | Path / dict | FP | Status | Notes |
|---|---|---|---|---|---|
| V1 | BIP-48 mainnet segwit-v0 multisig | m/48'/0'/0'/2' | yes | COVERED | `tests/vectors.rs::every_vector_round_trips` |
| V2 | BIP-84 mainnet single-sig | m/84'/0'/0' | yes | COVERED | same |
| V3 | BIP-48 testnet multisig | m/48'/1'/0'/2' | yes | COVERED | same |
| V4 | BIP-84 mainnet | m/84'/0'/0' | no | COVERED | privacy-preserving mode |
| V5 | explicit-path 4-component | non-dict | yes | COVERED | non-dictionary path encoding |
| V6 | 3-stub mainnet | dict | yes | COVERED | multi-stub policy_id |
| V7 | max path components (10) | explicit | no | COVERED | path-depth ceiling |
| V8 | BIP-87 mainnet | m/87'/0'/0' | yes | COVERED | path indicator 0x05 |
| V9 | BIP-44 mainnet | m/44'/0'/0' | yes | COVERED | path indicator 0x01 |
| V10 | BIP-49 mainnet | m/49'/0'/0' | yes | COVERED | path indicator 0x02 |
| V11 | BIP-86 mainnet | m/86'/0'/0' | yes | COVERED | path indicator 0x04 |
| V12 | BIP-48 nested-segwit mainnet | m/48'/0'/0'/1' | no | COVERED | path indicator 0x06 |
| V13 | BIP-44 testnet | m/44'/1'/0' | yes | COVERED | path indicator 0x11 |
| V14 | BIP-49 testnet | m/49'/1'/0' | yes | COVERED | path indicator 0x12 |
| V15 | BIP-84 testnet | m/84'/1'/0' | yes | COVERED | path indicator 0x13 |
| V16 | BIP-86 testnet | m/86'/1'/0' | no | COVERED | path indicator 0x14 |
| V17 | BIP-87 testnet | m/87'/1'/0' | no | COVERED | path indicator 0x15 |
| V18 | BIP-48 nested-segwit testnet | m/48'/1'/0'/1' | yes | COVERED | path indicator 0x16 (gap closed in v0.1.1 / v0.2.0) |

Path-dictionary coverage matrix (cross-mirror with md-codec):

| Indicator | Path | Coverage |
|---|---|---|
| 0x01 | m/44'/0'/0' | V9 |
| 0x02 | m/49'/0'/0' | V10 |
| 0x03 | m/84'/0'/0' | V2, V4 |
| 0x04 | m/86'/0'/0' | V11 |
| 0x05 | m/87'/0'/0' | V8 |
| 0x06 | m/48'/0'/0'/1' | V12 |
| 0x07 | m/48'/0'/0'/2' | V1, V6 |
| 0x11 | m/44'/1'/0' | V13 |
| 0x12 | m/49'/1'/0' | V14 |
| 0x13 | m/84'/1'/0' | V15 |
| 0x14 | m/86'/1'/0' | V16 |
| 0x15 | m/87'/1'/0' | V17 |
| 0x16 | m/48'/1'/0'/1' | V18 |
| 0x17 | m/48'/1'/0'/2' | V3 |

Phase 12 deliverable: ~~cross-verify byte-identity of path-dict entries
against md-codec's mirror~~ **RETIRED in mk-codec v0.2.2.** md-codec v0.11
dropped path dictionaries from md1 entirely (per
`descriptor-mnemonic/design/SPEC_v0_11_wire_format.md` §1.4 — "Wire-layer
dictionaries (path, use-site-path, shape). Considered and rejected for
architectural cleanliness"); md1 now encodes paths explicitly via
`OriginPath` and there is no sibling table to mirror. mk-codec v0.2.2
documents mk1's path dictionary as standalone (mk1-internal). The
lockstep invariant per the older `descriptor-mnemonic/CLAUDE.md` cross-
repo coordination block is retired; the v0.2.2 patch updates the SPEC,
BIP, doc-comments, and FOLLOWUPS in lockstep, plus the descriptor-
mnemonic CLAUDE.md cross-repo coordination block.

### Negative vectors (N1..N23)

22 negative vectors (N1..N21, N23; N22 reserved) covering each
`mk_codec::Error` variant. `tests/error_coverage.rs` enforces variant
exhaustiveness via `strum::EnumIter`.

| # | Variant exercised | Status |
|---|---|---|
| N1 | `InvalidHrp` (HRP `bt`) | COVERED |
| N2 | `MixedCase` | COVERED |
| N3 | `InvalidStringLength` | COVERED |
| N4 | `InvalidChar` (`b`) | COVERED |
| N5 | `BchUncorrectable` (5 substitutions) | COVERED |
| N6 | `UnsupportedCardType` (0x02) | COVERED |
| N7 | `MalformedPayloadPadding` | COVERED |
| N8 | `ChunkSetIdMismatch` | COVERED |
| N9 | `ChunkIndexOutOfRange` | COVERED |
| N10 | `MixedHeaderTypes` | COVERED |
| N11 | `CrossChunkHashMismatch` | COVERED |
| N12 | `UnsupportedVersion` (v1) | COVERED |
| N13 | `ReservedBitsSet` (bit 3) | COVERED |
| N14 | `InvalidPolicyIdStubCount` (0) | COVERED |
| N15 | `InvalidPathIndicator` (0x00) | COVERED |
| N16 | `PathTooDeep` (11 components) | COVERED |
| N17 | `InvalidPathComponent` (LEB128 overflow) | COVERED |
| N18 | `InvalidXpubVersion` (0xdeadbeef) | COVERED |
| N19 | `InvalidXpubPublicKey` (all zeros) | COVERED |
| N20 | `UnexpectedEnd` (truncated xpub) | COVERED |
| N21 | `TrailingBytes` (one extra) | COVERED |
| N23 | `EmptyInput` | COVERED |

---

## BIP-93 — codex32 (forked BCH)

Source: <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki>.

**Forked**, not delegated. mk-codec implements its own BCH layer at
`crates/mk-codec/src/string_layer/bch_decode.rs` over GF(32) with
HRP-mixing using NUMS target residues
(`MK_REGULAR_CONST = 0x1062435f91072fa5c`,
 `MK_LONG_CONST = 0x41890d7e441cbe97273`).

Conformance posture:
- mk1 BCH polynomial matches BIP-93 §"Generation of valid checksum" up to
  the target-residue constant.
- BIP-93 valid vectors are NOT bit-identical to mk1 vectors — different
  target residue.
- Structural rejections (wrong HRP, mixed case, invalid chars b/i/o/1)
  ARE shared and exercised in N1, N2, N4.

| # | BIP-93 vector | Applicability | Status |
|---|---|---|---|
| 93.1–93.5 | upstream codex32 valid | NOT BIT-IDENTICAL; different target residue | OUT-OF-SCOPE-PER-SPEC |
| 93.invalid (42) | structural overlap only | | partially OUT-OF-SCOPE-PER-SPEC; structural rejections COVERED via N1/N2/N4/N5 |

Phase 12 deliverable: `design/AUDIT_bip_cross_reference_completeness.md`
already establishes mk1 BCH is BIP-93-derived but not bit-identical.
Confirm CHANGELOG mentions this for v0.2.2.

---

## BIP-32 — HD wallets

Source: <https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki>.

mk-codec consumes `bitcoin::bip32::Xpub` from `bitcoin v0.32`. Direct
vector-pinning is OUT-OF-SCOPE-PER-SPEC at the mk-codec level — `bitcoin v0.32`
carries its own BIP-32 vector tests.

mk-codec's BIP-32-relevant surface is:
- xpub serialization (78 bytes from bitcoin's `Xpub::encode`).
- mainnet/testnet network determination from xpub version bytes.
- 0-hardened derivation depth (mk1 is descriptor-of-an-account-xpub, not master).

| # | Surface | Status | Notes |
|---|---|---|---|
| BIP32-mk.1 | xpub 78-byte version-byte parsing | COVERED | exercised in V1..V18 + N18 (invalid version) |
| BIP32-mk.2 | mainnet xpub `0488B21E` | COVERED | V1, V2, V4, V6, V8..V12 |
| BIP32-mk.3 | testnet tpub `043587CF` | COVERED | V3, V13..V18 |
| BIP32-mk.4 | non-account-level xpub (depth != path-indicator-implied) | COVERED-NEGATIVE | `tests/vectors.rs::exercise_clean_vector` does not validate xpub-depth-vs-path-indicator coherence (mk1 is xpub-as-bytes; depth in xpub may differ — this is by design per SPEC §3.5) |

Phase 12: NO new BIP-32 tests at this layer. mk-codec is a
xpub-by-the-bytes carrier, not a derivation tester.

---

## BIP-39 — mnemonic seed

mk-codec has zero BIP-39 surface. xpub is the input; mnemonics live in
`bip39 = 2` (toolkit) + `ms-codec`. OUT-OF-SCOPE-PER-SPEC at mk-codec.

---

## BIP-44 / 48 / 49 / 84 / 86 / 87 — derivation path conventions

mk-codec's path-dictionary table is now mk1-internal (per the v0.2.2
divergence note in SPEC §3.5). Per-indicator coverage is in the V1..V18
table above.

**Cross-repo invariant (RETIRED in mk-codec v0.2.2):** mk-codec's
path-dict table was previously contractually mirrored byte-for-byte
against md-codec's `Tag::SharedPath` / `Tag::OriginPaths`. md-codec v0.11
dropped path dictionaries entirely (per
`descriptor-mnemonic/design/SPEC_v0_11_wire_format.md` §1.4); the mirror
invariant is therefore retired. The 14-entry table at
`crates/mk-codec/src/bytecode/path.rs::STANDARD_PATHS` is now the sole
source of truth; future entries are an mk1-side decision.

| # | BIP / path | Indicator | mk1 vector | Status (post-v0.2.2: mirror RETIRED — md1 has no path dict) |
|---|---|---|---|---|
| 0x01 | BIP-44 mainnet | 0x01 | V9 | mirrored in md-codec; COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x02 | BIP-49 mainnet | 0x02 | V10 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x03 | BIP-84 mainnet | 0x03 | V2, V4 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x04 | BIP-86 mainnet | 0x04 | V11 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x05 | BIP-87 mainnet | 0x05 | V8 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x06 | BIP-48 nested mainnet | 0x06 | V12 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x07 | BIP-48 segwit mainnet | 0x07 | V1, V6 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x11 | BIP-44 testnet | 0x11 | V13 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x12 | BIP-49 testnet | 0x12 | V14 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x13 | BIP-84 testnet | 0x13 | V15 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x14 | BIP-86 testnet | 0x14 | V16 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x15 | BIP-87 testnet | 0x15 | V17 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x16 | BIP-48 nested testnet | 0x16 | V18 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |
| 0x17 | BIP-48 segwit testnet | 0x17 | V3 | COVERED (mk1-internal, mirror RETIRED v0.2.2) |

Phase 12 deliverable: ~~1 new test~~
~~`tests/path_dict_md_mirror.rs::path_dict_byte_identical_to_md_codec`~~
**RETIRED.** md-codec v0.11 dropped path dictionaries entirely; there
is no md-codec table to mirror against. Phase 12 closed as docs-only
(SPEC + BIP + FOLLOWUPS + cross-repo CLAUDE.md mirror clause removal);
no new test landed.

---

## BIP-380 — descriptor expressions

mk-codec has zero BIP-380 surface. xpub-as-bytes carrier; no descriptor
emit/parse. OUT-OF-SCOPE-PER-SPEC.

---

## BIP-388 — wallet policies

mk-codec has zero BIP-388 surface. xpub backup, not policy backup.
md-codec owns BIP-388. OUT-OF-SCOPE-PER-SPEC.

---

## SLIP-0132 — registered HD version bytes

mk-codec parses xpub version bytes per BIP-32 (`0488B21E` mainnet,
`043587CF` testnet). It does NOT support SLIP-0132 prefixes (ypub, zpub,
Ypub, Zpub, etc.) — those normalize through the toolkit's
`src/slip0132.rs` *before* hitting mk-codec. mk-codec's xpub input MUST
be BIP-32 neutral.

| # | Prefix | Status | Notes |
|---|---|---|---|
| 132.* | ypub/zpub/Ypub/Zpub/... | OUT-OF-SCOPE-PER-SPEC | normalized in toolkit upstream of mk-codec |

---

## BIP-173 — bech32

Source: <https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki>.

mk-codec implements bech32 character set + HRP-expansion (forked from
BIP-93's codex32 base). Negative vector N4 (`InvalidChar` for `b`)
exercises the bech32 alphabet exclusion. N2 (mixed case) covers
BIP-173's case discipline.

| # | BIP-173 invariant | Status | Notes |
|---|---|---|---|
| 173.1 | alphabet excludes b/i/o/1 | COVERED | N4 |
| 173.2 | mixed-case rejection | COVERED | N2 |
| 173.3 | HRP-expansion (low-bits + 0 + high-bits) | COVERED-INTERNAL | exercised in BCH compute path; not isolated test |
| 173.4 | `1` separator handling | COVERED-IMPLICIT | every V* vector has the right separator |

---

## Summary

| Category | Total vectors | Covered | Missing (in-scope) | Out-of-scope-per-user | Out-of-scope-per-spec |
|---|---|---|---|---|---|
| mk1 clean corpus | 18 | 18 | 0 | 0 | 0 |
| mk1 negative corpus | 22 | 22 | 0 | 0 | 0 |
| BIP-93 valid (5) | 5 | 0 | 0 | 0 | 5 (forked BCH) |
| BIP-93 invalid (42) | 42 | structural subset | 0 | 0 | ~38 (delegated) |
| BIP-32 | 18 | 4 layer-only | 0 | 0 | 18 (delegated to bitcoin v0.32) |
| BIP-39 | n/a | — | 0 | 0 | n/a (no surface) |
| Path-dict (14 entries; mk1-internal as of v0.2.2) | 14 | 14 | 0 (byte-identity test RETIRED; mirror invariant retired) | 0 | 0 |
| BIP-380 | n/a | — | 0 | 0 | n/a |
| BIP-388 | n/a | — | 0 | 0 | n/a |
| SLIP-0132 | n/a | — | 0 | 0 | n/a (toolkit-normalized) |
| BIP-173 | 4 | 4 | 0 | 0 | 0 |
| **TOTAL** | **>140** | **~62** | **~1** | **0** | **~61** |

Phase 12 target: ~~1 net-new test (path-dict byte-identity vs md-codec)~~ **0 net-new tests; docs-only patch.** md-codec v0.11 dropped path dictionaries entirely (per `descriptor-mnemonic/design/SPEC_v0_11_wire_format.md` §1.4); the mirror invariant has no md1-side anchor and is retired. v0.2.2 reclassifies mk1's dictionary as standalone (mk1-internal) across SPEC, BIP, doc-comments, FOLLOWUPS, and the descriptor-mnemonic CLAUDE.md cross-repo coordination block.

---

## Discoveries (require architect review before pinning)

1. **No bug-shaped findings.** mk-codec's vector posture is the strongest
   of the 4 sibling repos by design — BCH-correct/canonical-xpub
   decisions had to be locked early because the BIP draft was filed
   pre-v0.1.1 (per `design/MILESTONE_v0_1_1.md` notes). The clean
   18-vector + negative 22-vector corpus exhaustively covers mk-codec's
   surface; the only audit-cycle add is the cross-mirror byte-identity
   pin.

2. **RESOLVED — mirror invariant retired in mk-codec v0.2.2 (docs-only).**
   The Phase 0 audit flagged the path-dict mirror as human-mirrored
   rather than programmatically-mirrored, with Phase 12 originally
   planned to add a byte-identity test. During Phase 12 implementation,
   re-reading `descriptor-mnemonic/design/SPEC_v0_11_wire_format.md`
   §1.4 surfaced that md-codec v0.11+ has **dropped path dictionaries
   entirely** (architectural-cleanliness decision; md1 now encodes
   paths explicitly via `OriginPath`). There is no md-codec table to
   mirror against. mk-codec v0.2.2 reclassifies mk1's dictionary as
   standalone (mk1-internal) across SPEC §3.5, the BIP draft, source
   doc-comments, FOLLOWUPS, and the descriptor-mnemonic CLAUDE.md
   cross-repo coordination block. The audit-cycle deliverable is
   purely documentation; no code or test change. Mirror invariant
   RETIRED.

3. **OUT-OF-SCOPE-PER-SPEC dominance is by design.** mk-codec is a
   minimal-surface xpub container; ~80% of BIPs in the broader
   audit (BIP-32 derivation, BIP-39 mnemonics, BIP-380 descriptors,
   BIP-388 policies) are not its responsibility. The OOSPS counts
   above are not coverage gaps — they're appropriate delegation
   boundaries.
