# mk1 v0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans`. Steps use checkbox (`- [ ]`) syntax for tracking. Per-phase Opus reviews dispatched per the user's autonomous workflow; reports persisted to `design/agent-reports/`.

**Goal:** Ship `mk-codec` v0.1.0 — first reference implementation of the Mnemonic Key (MK) backup format. Delivers a working encode→string→decode round-trip, an initial vector corpus, the spec amended to reflect Q-1..Q-10 closures, and the BIP draft populated with concrete content where it currently has skeleton placeholders.

**Architecture:** Single coordinated release on a `feature/v0.1.0-implementation` branch. Wire format finalized per the closure design. BCH primitives forked from `md-codec` per D-13 (shared-crate extraction deferred per Q-9). New `consts.rs`, `bytecode/` (encoder + decoder), `string_layer/` (BCH + chunk header) modules in `crates/mk-codec/src/`.

**Test discipline:** Phases 1, 2, and 7 are docs-only — no test impact. Phases 3, 4, 5 are TDD: each Error variant gets a negative test before its code path lands; each public type gets a property-shape test before its impl lands. Within each impl task, **test substeps land before impl substeps**; tests start `#[ignore]`-marked or build-failing and become passing as the impl substeps complete. `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --all -- --check` run as part of every commit; CI gates on both. Phase 6 is end-to-end test corpus. Phase 8 is reconciliation.

**Spec references:**

- Closure design (locks for Q-1..Q-10): [`docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`](../docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md)
- Wire-format spec (gets amended in Phase 1): [`design/SPEC_mk_v0_1.md`](./SPEC_mk_v0_1.md)
- Decisions log (gets D-14, D-15 added in Phase 1): [`design/DECISIONS.md`](./DECISIONS.md)
- BIP draft (gets filled in Phase 2): [`bip/bip-mnemonic-key.mediawiki`](../bip/bip-mnemonic-key.mediawiki)
- FOLLOWUPS source-of-truth: [`design/FOLLOWUPS.md`](./FOLLOWUPS.md)
- Sibling md1 BIP (parallel structural reference): `/scratch/code/shibboleth/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki`

---

## File structure

| File | Responsibility | Phase |
|---|---|---|
| `design/SPEC_mk_v0_1.md` | Apply closure ripple §2.3, §2.4, §2.5, §3.1–§3.6, §4, §5, §6, §7, §9 | 1 |
| `design/DECISIONS.md` | Close Q-1..Q-10; add D-14, D-15 | 1 |
| `bip/bip-mnemonic-key.mediawiki` | Fill in skeleton sections per closure locks | 2 |
| `crates/mk-codec/src/consts.rs` (NEW) | `MK_REGULAR_CONST`, `MK_LONG_CONST`, `MAX_PATH_COMPONENTS`, capacity numbers | 3 |
| `crates/mk-codec/src/error.rs` | Add/remove/rename Error variants per closure | 3 |
| `crates/mk-codec/src/bytecode/header.rs` (NEW) | `BytecodeHeader` with bit-2 fingerprint flag | 4 |
| `crates/mk-codec/src/bytecode/path.rs` (NEW) | Path dictionary + explicit-path codec | 4 |
| `crates/mk-codec/src/bytecode/xpub_compact.rs` (NEW) | Compact-73 codec with depth/child_number reconstruction | 4 |
| `crates/mk-codec/src/bytecode/encode.rs` (NEW) | `KeyCard → Vec<u8>` bytecode encoder | 4 |
| `crates/mk-codec/src/bytecode/decode.rs` (NEW) | `Vec<u8> → KeyCard` bytecode decoder | 4 |
| `crates/mk-codec/src/string_layer/bch.rs` (NEW, forked from md-codec) | BCH polymod + target-residue verification | 5 |
| `crates/mk-codec/src/string_layer/header.rs` (NEW) | String-layer header (single-string 2-char + chunked 8-char) | 5 |
| `crates/mk-codec/src/string_layer/chunk.rs` (NEW) | Chunk split/merge + cross-chunk integrity hash | 5 |
| `crates/mk-codec/src/string_layer/mod.rs` (NEW) | Layer-3 encode/decode entry points | 5 |
| `crates/mk-codec/src/key_card.rs` | Update `KeyCard` for Optional<Fingerprint>; rewrite `encode`/`decode` to call into the new layers | 4, 5 |
| `crates/mk-codec/src/lib.rs` | Re-exports for new pub items | 3, 4, 5 |
| `crates/mk-codec/tests/round_trip.rs` | End-to-end round-trip tests | 6 |
| `crates/mk-codec/tests/vectors/v0.1.json` (NEW) | Initial vector corpus | 6 |
| `crates/mk-codec/Cargo.toml` | Version 0.0.0 → 0.1.0 | 7 |
| `CHANGELOG.md` (NEW at repo root) | `[0.1.0]` section | 7 |
| `README.md` | Drop "design-stage skeleton" framing | 7 |

---

## Phase 0 — Branch + workspace prep

**Goal:** Set up the work branch and verify the toolchain before any real work begins.

### Task 0.1 — Branch

- [ ] **Step 0.1.1**: `git checkout -b feature/v0.1.0-implementation` from `main`. All Phase 1–7 commits land on this branch.

### Task 0.2 — Toolchain sanity

- [ ] **Step 0.2.1**: Confirm `cargo build`, `cargo test`, `cargo clippy --all-targets`, `cargo fmt --check` all pass on the existing scaffold.

---

## Phase 1 — Spec amendments (docs-only)

**Goal:** Apply the closure design's spec ripple to `design/SPEC_mk_v0_1.md` and `design/DECISIONS.md`. No code, no BIP-draft work yet.

**Strategy:** Walk §-by-§ in the closure design's "Spec ripple" section. Each amendment is local; verify after each edit that the spec stays internally consistent (cross-references between sections still resolve).

**Per-phase agent report:** `design/agent-reports/v0-1-phase-1-review-<commit>.md`

### Task 1.1 — Apply SPEC_mk_v0_1.md amendments

- [ ] **Step 1.1.1: §2.3 — concrete NUMS constants**

Replace `0x???????????????` placeholders with the locked values:

```
MK_REGULAR_CONST = 0x1062435f91072fa5c    (65 bits)
MK_LONG_CONST    = 0x41890d7e441cbe97273  (75 bits)
```

Add the Python reproducer block per the closure design Q-1.

- [ ] **Step 1.1.2: §2.4 — concretize chunked-fragment capacity**

Add to the existing length-envelope discussion (and pin matching identifiers in `crates/mk-codec/src/consts.rs` per Phase 3 — names listed here for cross-doc consistency):

```
single-string regular code:    48 bytes payload    (SINGLE_STRING_REGULAR_BYTES)
single-string long code:       56 bytes payload    (SINGLE_STRING_LONG_BYTES) ← already present
chunked-fragment regular code: 45 bytes per fragment  (CHUNKED_FRAGMENT_REGULAR_BYTES) ← NEW
chunked-fragment long code:    53 bytes per fragment  (CHUNKED_FRAGMENT_LONG_BYTES)    ← NEW
max chunks per card:           32                  (MAX_CHUNKS) ← NEW
cross-chunk integrity hash:    SHA-256(canonical_bytecode)[0..4]   (4 bytes)   ← NEW
```

Note: with up to 32 long-code chunks, capacity is `32 × 53 − 4 = 1692` bytes — vastly above any plausible mk1 payload. Spec text MUST use the constant identifiers from `consts.rs` parenthetically so naming stays in lockstep across spec, BIP, and code.

- [ ] **Step 1.1.3: §2.5 — expand chunk-type table to full string-layer header**

Replace the current chunk-type table with the full string-layer header layout from closure Q-5:

- 2-char single-string header (version + type)
- 8-char chunked header (version + type + chunk_set_id + total_chunks + chunk_index)
- Use `chunk_set_id` naming (md-codec v0.9.0 closed the legacy "wallet identifier" naming as a cross-repo prerequisite)
- Reserved chunk-type range `0x02..0x1F` exhausts the 5-bit field; note this for future authors

- [ ] **Step 1.1.4: §3.1 — bit-2 fingerprint flag**

Bytecode header bit allocation update per closure Q-8:

- bits 7-4: version (0x0)
- bit 3:    reserved (MUST be 0)
- bit 2:    fingerprint flag ← NEW (was reserved)
- bit 1:    reserved (MUST be 0)
- bit 0:    reserved (MUST be 0)

Add the cross-format alignment prose: "mk1's bytecode header mirrors md1's bit-allocation shape — 4-bit version + 4 flag/reserved bits — and shares bit-2 semantics ('optional fingerprint-related block follows'). Block contents differ; bit-level convention is shared."

- [ ] **Step 1.1.5: §3.2 — payload field order with conditional fingerprint**

```
[bytecode_header   : 1 B]
[stub_count        : 1 B]
[policy_id_stubs   : 4 × N B]
[origin_fingerprint: 4 B]   ← present iff bytecode_header bit 2 set
[origin_path       : 1 B (std-table) OR 1 + 1 + 5N B (explicit path)]
[xpub_compact      : 73 B]  ← was 78
```

Total for typical 1-stub mainnet card with std-table indicator + fingerprint present: 84 B.

- [ ] **Step 1.1.6: §3.3 — tighten Q-2 rationale**

Drop the loose "matches md1 chunk-header convention" claim. Replace with the explicit birthday-bound math from the closure: `P(collision) ≈ k(k−1)/(2·2³²) ≈ 2.85×10⁻⁷` for `k = 50` wallets. Note: even 24 bits clears the threat model (`≈ 7.3×10⁻⁵`).

- [ ] **Step 1.1.7: §3.4 — origin_fingerprint conditional**

Add: "Present only if bytecode-header bit 2 is set; otherwise omitted from the payload."

- [ ] **Step 1.1.8: §3.5 — path-component cap 32 → 10**

Update the explicit-path encoding subsection: `component_count` MUST be in range `1..=10`. `PathTooDeep` error fires at 11 or above. Update the table accordingly.

- [ ] **Step 1.1.9: §3.6 — xpub encoding compact-73**

Reframe from "full 78-byte serialization" to compact-73 form. Document:

```
Compact-73 byte breakdown:
  [xpub.version          : 4 B]
  [xpub.parent_fingerprint: 4 B]
  [xpub.chain_code       : 32 B]
  [xpub.public_key       : 33 B]
                           ────
                           73 B
```

Decoder reconstruction rule:

```
depth        := component_count(origin_path)
child_number := last_component(origin_path) including hardened-bit encoding
```

Add the limit-of-detection subsection (operator-pick-wrong-indicator caught at §5 step 4 via Wallet Instance ID, not at the per-card level).

- [ ] **Step 1.1.10: §4 — validity rules update**

- Remove rule 8 (`XpubDepthMismatch`) — impossible by construction under compact-73.
- Add: "Encoders MUST set the fingerprint flag iff `origin_fingerprint` is present in the payload, and decoders MUST reject any state where the flag and the payload disagree."
- Update path-cap rule from `> 32` to `> 10`.
- Add: "Decoders MUST reject any chunked input where the cross-chunk integrity hash does not match `SHA-256(reassembled_canonical_bytecode)[0..4]`."
- Add: "Decoders MUST reject any payload whose 5-bit symbols, after BCH verification, do not byte-align (analog to md1's `MalformedPayloadPadding`)."

- [ ] **Step 1.1.11: §5 — authority precedence subsection**

Add the closure Q-4 lock: when both mk1 and md1 (with per-`@N` paths) participate in recovery, mk1's `origin_path` is authoritative; mismatch MUST cause the recovery orchestrator to reject. Per-format decoders are not required to be cross-aware.

- [ ] **Step 1.1.12: §6 — privacy framing amendments**

Add the four normative-recommendation paragraphs from closure Q-8:

1. Optional-fingerprint privacy mode (encoder SHOULD expose explicit choice).
2. Disposal of rotated cards.
3. Hand-off discipline.
4. Integrity detection limit (verify first derived address against external anchor before moving funds when no Wallet Instance ID is available).

- [ ] **Step 1.1.13: §7 — confirm `mk-codec X.Y` family token**

Already provisional in §7; promote to locked. No structural change.

- [ ] **Step 1.1.14: §9 — replace open-questions table with closure pointer**

Empty out the "Open questions explicitly punted" table; replace with a "Closures (2026-04-29)" subsection pointing to the closure design doc.

### Task 1.2 — Apply DECISIONS.md amendments

- [ ] **Step 1.2.1: Close Q-1 through Q-10**

Convert each open-question row to a closed-decision entry with a one-sentence summary of the lock and a pointer to the closure design doc. Format mirrors the existing D-1..D-13 entries.

- [ ] **Step 1.2.2: Add D-14 — cross-format header parsing alignment with md1**

> mk1's string-layer header structure mirrors md1's exactly (2-char single, 8-char chunked, identical bit allocation). mk1's bytecode header shares bit-allocation shape with md1, including bit-2 semantics ("optional fingerprint-related block follows") at the cross-format pattern level. This enables a common header-parsing helper when D-13's `mc-codex32` extraction happens (Q-9 trigger). Block contents inside the optional bit-2 block differ between formats; the convention is at the bit-level pattern.

- [ ] **Step 1.2.3: Add D-15 — `chunk_set_id` rename across both repos**

> The 20-bit per-encoding random tag in the chunked string-layer header is named `chunk_set_id` in mk1 from day 1. md1 v0.8.x originally called the same field "wallet identifier"; that name conflicted with `Policy ID` and `Wallet Instance ID` and was misleading. The rename in md1 was a sequencing prerequisite for mk1's BIP submission per the closure design and shipped in [md-codec v0.9.0](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0); the cross-repo coordination is now resolved.

### Task 1.3 — Build + commit + review

- [ ] **Step 1.3.1: Verify spec consistency**

Read SPEC_mk_v0_1.md end-to-end; confirm internal cross-references resolve, no leftover `0x???` or `TBD` markers, no contradictions between updated sections.

- [ ] **Step 1.3.2: Commit**

```
docs(spec): apply mk1 v0.1 closure ripple

Translates Q-1..Q-10 closures from
docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md
into concrete amendments to design/SPEC_mk_v0_1.md and
design/DECISIONS.md. Adds D-14 (cross-format header alignment)
and D-15 (chunk_set_id rename).

No code changes; no BIP-draft changes (Phase 2).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

- [ ] **Step 1.3.3: Phase 1 review**

Dispatch Opus reviewer.

- Files: `design/SPEC_mk_v0_1.md`, `design/DECISIONS.md`.
- Verify: every closure ripple item in the design doc maps to a concrete edit; no contradictions introduced; D-14 and D-15 read coherently with D-1..D-13.
- Output: `design/agent-reports/v0-1-phase-1-review-<commit>.md`.

Apply critical/important findings inline; minor items append to FOLLOWUPS.

---

## Phase 2 — BIP draft fill-in (docs-only)

**Goal:** Replace skeleton sections in `bip/bip-mnemonic-key.mediawiki` with concrete content per the locked design. The BIP draft becomes coherent enough to submit for first human review (per D-11's coordination note for Andrew Poelstra).

**Strategy:** Mirror md1's BIP structure section-by-section. Where md1 has analogous content (e.g., "Why new target constants?", "Length envelope"), adapt md1's wording with mk1-specific replacements rather than rewrite from scratch. Keep mk1 narrower: no descriptor-tree language, no policy-template framing.

**Per-phase agent report:** `design/agent-reports/v0-1-phase-2-review-<commit>.md`

### Task 2.1 — Fill encoding-layer sections

- [ ] **Step 2.1.1: §"Target residue constants"**

Adapt md1's "Why new target constants?" section verbatim with mk1 replacements:
- domain string → `b"shibbolethnumskey"`
- `T_REGULAR` → `MK_REGULAR_CONST = 0x1062435f91072fa5c`
- `T_LONG` → `MK_LONG_CONST = 0x41890d7e441cbe97273`
- Python reproducer block → mk1 version

- [ ] **Step 2.1.2: §"Length envelope"**

Per closure Q-5: capacities 48/56/45/53; max chunks 32; cross-chunk hash 4 bytes. Note 32×53−4 = 1692 max bytes per card.

- [ ] **Step 2.1.3: §"Header" (string-layer)**

Single-string 2-char (version + type); chunked 8-char (version + type + chunk_set_id 20 bits + total_chunks + chunk_index). Mirror md1's wording (md-codec v0.9.0 already uses `chunk_set_id`).

- [ ] **Step 2.1.4: §"Cross-chunk integrity hash"**

`cross_chunk_hash = SHA-256(canonical_bytecode)[0..4]`; appended to the canonical bytecode before chunk-splitting; verified at reassembly. Adapt md1's wording.

### Task 2.2 — Fill bytecode-layer sections

- [ ] **Step 2.2.1: §"Bytecode header"**

Per closure Q-8: bit allocation table; bit 2 = fingerprint flag; valid v0.1 header values are 0x00 (no fingerprint) and 0x04 (fingerprint present). Reserved bits 0, 1, 3 MUST be 0.

- [ ] **Step 2.2.2: §"Payload field order"**

Per closure Q-6: ordered field list with sizes and conditionals.

- [ ] **Step 2.2.3: §"Path encoding"**

Per closure Q-3 + Q-7's coupling. Standard-table dictionary (mirror md1's table including testnet variants) + 0xFE explicit-path escape. Component cap = 10. LEB128-encoded u32 components with hardened bit in high position.

- [ ] **Step 2.2.4: §"xpub encoding"**

Per closure Q-7: compact-73 form. Byte breakdown. Reconstruction rule for depth + child_number from origin_path. Limit-of-detection note carrying into §"Privacy considerations".

### Task 2.3 — Fill linkage + privacy sections

- [ ] **Step 2.3.1: §"Linkage to MD"**

Authority-precedence subsection per closure Q-4: mk1's origin_path authoritative; recovery-orchestrator-layer mismatch check; precise error reporting.

- [ ] **Step 2.3.2: §"Privacy considerations"**

Per closure Q-8 §6 amendments: optional-fingerprint mode, disposal, hand-off, integrity detection limit. Compare-and-contrast with md1 privacy footprint.

### Task 2.4 — Fill validity rules + decoder reporting

- [ ] **Step 2.4.1: §"Decoder validity rules"**

Concrete enumerated list per closure §4 ripple. Each rule cites its named Error variant (forward-reference to Phase 3's `error.rs`).

- [ ] **Step 2.4.2: §"Decoder reporting"**

Adapt md1's "clean / N substitutions corrected / N erasures corrected / structure-aided / failed" framing. mk1 inherits this conformance pattern via the BCH-layer fork from md-codec.

### Task 2.5 — Build + commit + review

- [ ] **Step 2.5.1: Read end-to-end**

Read the BIP draft top to bottom; confirm coherent narrative, no skeleton-leftover language, no contradictions with SPEC_mk_v0_1.md.

- [ ] **Step 2.5.2: Commit**

```
docs(bip): fill mk1 BIP draft per v0.1 closure

Replaces skeleton sections in bip/bip-mnemonic-key.mediawiki with
concrete content from the locked closure design. Adapts md1's
parallel sections (target constants, header, length envelope,
cross-chunk hash, decoder reporting) with mk1 substitutions.

The draft is now coherent for first human review per D-11.
Pre-BIP-submission audit items (FOLLOWUPS pre-bip-submission tier)
remain to be cleared before formal submission.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

- [ ] **Step 2.5.3: Phase 2 review**

Dispatch Opus reviewer.

- Files: `bip/bip-mnemonic-key.mediawiki`.
- Cross-reference against: `design/SPEC_mk_v0_1.md` (post-Phase-1), `design/DECISIONS.md`, md1's BIP for parallel-section accuracy.
- Verify: no spec-vs-BIP drift; mk1's narrower scope is preserved (no md1-style descriptor-tree language); cross-format alignment claims hold.
- Output: `design/agent-reports/v0-1-phase-2-review-<commit>.md`.

---

## Phase 3 — Constants module + Error overhaul (TDD)

**Goal:** Set up the supporting types — constants, Error variants — that downstream phases consume. Update the existing `error.rs` to reflect closure-mandated changes.

**Strategy:** Write/update the Error enum first (each new variant gets a `#[test] fn rejects_<case>()` placeholder that's `#[ignore]`-marked until its implementing phase lands). Add a `consts.rs` module. Wire up `lib.rs` re-exports.

**Per-phase agent report:** `design/agent-reports/v0-1-phase-3-review-<commit>.md`

### Task 3.1 — `consts.rs`

- [ ] **Step 3.1.1: Create `crates/mk-codec/src/consts.rs`**

```rust
//! Locked constants for `mk1` per `design/SPEC_mk_v0_1.md` v0.1.
//!
//! Reproducers for the NUMS-derived target constants are documented in
//! the BIP draft's "Why new target constants?" section.

/// HRP for `mk1` strings. BIP 173 separator follows.
pub const HRP: &str = "mk";

/// Domain string for NUMS-derived target constants.
pub const NUMS_DOMAIN: &[u8] = b"shibbolethnumskey";

/// Top 65 bits of `SHA-256(NUMS_DOMAIN)`. Regular-code target residue.
pub const MK_REGULAR_CONST: u128 = 0x1062435f91072fa5c;

/// Top 75 bits of `SHA-256(NUMS_DOMAIN)`. Long-code target residue.
pub const MK_LONG_CONST: u128 = 0x41890d7e441cbe97273;

/// Maximum components in an explicit-path encoding (per closure Q-3).
pub const MAX_PATH_COMPONENTS: u8 = 10;

/// Single-string regular-code payload bytes.
pub const SINGLE_STRING_REGULAR_BYTES: usize = 48;

/// Single-string long-code payload bytes.
pub const SINGLE_STRING_LONG_BYTES: usize = 56;

/// Chunked-fragment regular-code payload bytes per chunk.
pub const CHUNKED_FRAGMENT_REGULAR_BYTES: usize = 45;

/// Chunked-fragment long-code payload bytes per chunk.
pub const CHUNKED_FRAGMENT_LONG_BYTES: usize = 53;

/// Maximum chunks per card.
pub const MAX_CHUNKS: u8 = 32;

/// Family-stable generator string for vector-corpus SHA anchoring.
/// Replace `X.Y` at release-tag time. Patch-version bumps don't roll the token.
pub const GENERATOR_FAMILY: &str = "mk-codec 0.1";
```

- [ ] **Step 3.1.2: Sanity test**

The top 65 bits and top 75 bits of `SHA-256(NUMS_DOMAIN)` both fit in `u128` (since 65 ≤ 128 and 75 ≤ 128), so the staging is a single right-shift on the leading 128 bits of the big-endian 256-bit digest:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn nums_constants_reproduce_from_domain() {
        let digest = Sha256::digest(NUMS_DOMAIN);
        // Stage the leading 128 bits as a big-endian u128.
        let hi: u128 = u128::from_be_bytes(digest[0..16].try_into().unwrap());

        // Top 65 bits: shift the leading 128 bits right by (128 - 65) = 63.
        let derived_regular = hi >> 63;
        assert_eq!(derived_regular, MK_REGULAR_CONST,
            "MK_REGULAR_CONST drift from SHA-256(NUMS_DOMAIN) top-65-bits");

        // Top 75 bits: shift right by (128 - 75) = 53.
        let derived_long = hi >> 53;
        assert_eq!(derived_long, MK_LONG_CONST,
            "MK_LONG_CONST drift from SHA-256(NUMS_DOMAIN) top-75-bits");
    }
}
```

This test catches accidental drift between the documented derivation rule (closure Q-1) and the locked hex constants — if anyone changes either side without updating the other, the test fails immediately.

### Task 3.2 — `error.rs` overhaul

- [ ] **Step 3.2.1: Remove deprecated variant**

Remove `Error::XpubDepthMismatch` (impossible by construction under compact-73 per Q-7). Update the rustdoc on the enum to point at SPEC §4 post-amendment.

- [ ] **Step 3.2.2: Update path cap message**

`Error::PathTooDeep`: change message from "max 32" to "max 10". Bound now matches `MAX_PATH_COMPONENTS`.

- [ ] **Step 3.2.3: Add new variants per closure ripple**

```rust
/// Cross-chunk integrity hash mismatch — reassembled bytecode does
/// not match the trailing 4-byte SHA-256 prefix.
#[error("cross-chunk integrity hash mismatch")]
CrossChunkHashMismatch,

/// Bytecode-header fingerprint flag and payload presence disagree:
/// either bit-2 is set but `origin_fingerprint` is absent, or
/// bit-2 is unset but `origin_fingerprint` was emitted.
#[error("fingerprint flag does not match payload presence")]
FingerprintFlagMismatch,

/// 5-bit payload symbols, after BCH verification, do not byte-align.
/// Parallels md1's MalformedPayloadPadding rejection.
#[error("malformed payload padding (5-bit symbols don't byte-align)")]
MalformedPayloadPadding,

/// Chunked input has chunks from different `chunk_set_id`s.
#[error("chunk_set_id mismatch across chunks")]
ChunkSetIdMismatch,

/// Chunked input has duplicate or out-of-range `chunk_index` values
/// or a `total_chunks` value out of range 1..=32.
#[error("chunked-header malformed: {0}")]
ChunkedHeaderMalformed(String),

/// Decoder rejects an unknown card-type byte (reserved 0x02..0x1F range).
#[error("unsupported card type byte: 0x{0:02x}")]
UnsupportedCardType(u8),
```

(`UnsupportedCardType` already exists; widen its rustdoc to mention the reserved-range exhaustiveness.)

- [ ] **Step 3.2.4: TDD sad-path test scaffolds**

For each new variant, add an `#[ignore]`-marked test in the relevant module that documents the expected reject case. The `#[ignore]` is removed in the phase that lands the code path.

### Task 3.3 — `lib.rs` re-exports

- [ ] **Step 3.3.1: Add `consts` module re-export**

```rust
pub mod consts;
pub use consts::{
    HRP,
    NUMS_DOMAIN,
    MK_REGULAR_CONST,
    MK_LONG_CONST,
    MAX_PATH_COMPONENTS,
    SINGLE_STRING_REGULAR_BYTES,
    SINGLE_STRING_LONG_BYTES,
    CHUNKED_FRAGMENT_REGULAR_BYTES,
    CHUNKED_FRAGMENT_LONG_BYTES,
    MAX_CHUNKS,
    CROSS_CHUNK_HASH_BYTES,
    XPUB_COMPACT_BYTES,
    POLICY_ID_STUB_BYTES,
    ORIGIN_FINGERPRINT_BYTES,
    GENERATOR_FAMILY,
};
```

(Phases 4 and 5 add re-exports for `BytecodeHeader`, `XpubCompact`, `StringLayerHeader`. `KeyCard`, `Error`, `encode`, `decode` are already re-exported by the existing scaffold.)

### Task 3.4 — Build + commit + review

- [ ] **Step 3.4.1: Build**

```bash
cargo build -p mk-codec
cargo test -p mk-codec --lib  # ignored sad-path tests stay ignored
```

Expected: clean build; `cargo test` reports the existing `types_compile` test plus the new sanity-check passing, with sad-path tests `#[ignore]`-marked.

- [ ] **Step 3.4.2: Commit**

```
feat(mk-codec phase 3): constants + error overhaul

Adds crates/mk-codec/src/consts.rs with locked NUMS constants,
capacity numbers, MAX_PATH_COMPONENTS=10, and GENERATOR_FAMILY.

Updates error.rs per the closure ripple:
- Remove XpubDepthMismatch (impossible by construction post-Q-7)
- Update PathTooDeep cap from 32 to 10
- Add CrossChunkHashMismatch, FingerprintFlagMismatch,
  MalformedPayloadPadding, ChunkSetIdMismatch, ChunkedHeaderMalformed

Each new variant has an #[ignore]-marked sad-path test scaffold;
the #[ignore] is removed in the phase that lands the code path.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

- [ ] **Step 3.4.3: Phase 3 review**

Dispatch Opus reviewer.

- Files: `consts.rs`, `error.rs`, `lib.rs`, scaffold tests.
- Verify: every Error variant maps to a SPEC §4 rule (or to a layer-3 string-layer reject case); no orphaned variants; constant values match closure-locked hex; path cap = 10 everywhere consistent.
- Output: `design/agent-reports/v0-1-phase-3-review-<commit>.md`.

---

## Phase 4 — Bytecode layer (TDD)

**Goal:** Implement encode/decode at the bytecode level — `KeyCard` ↔ `Vec<u8>` (canonical bytecode, pre-chunking). Phase 5 layers BCH/string-encoding on top.

**Strategy:** Bottom-up TDD per submodule. Path codec → xpub-compact codec → policy_id_stubs codec → header codec → top-level encode/decode entry. Each gets a property-shape test (round-trip) and the relevant sad-path tests (Error variant per closure §4) before the impl lands.

**Per-phase agent report:** `design/agent-reports/v0-1-phase-4-review-<commit>.md`

### Task 4.1 — `BytecodeHeader` (TDD)

- [ ] **Step 4.1.1: Tests first** (write before any impl)

In `crates/mk-codec/src/bytecode/header.rs` `#[cfg(test)] mod tests`:

- happy-path round-trip: `to_byte(parse(0x00)) == 0x00`; `to_byte(parse(0x04)) == 0x04`.
- happy-path semantics: `parse(0x00).fingerprint_flag == false`; `parse(0x04).fingerprint_flag == true`.
- sad path: any reserved bit (bit 0, 1, or 3) set → `Error::ReservedBitsSet`. Test all three positions.
- sad path: version > 0 (e.g., 0x10) → `Error::UnsupportedVersion(0x1)`.

Tests fail to compile until 4.1.2 lands the type. Use `#[ignore]` markers if the implementer prefers compile-clean checkpoints; remove markers in 4.1.2.

- [ ] **Step 4.1.2: Type + parse + emit**

```rust
// crates/mk-codec/src/bytecode/header.rs
pub struct BytecodeHeader {
    pub version: u8,        // 0..=15
    pub fingerprint_flag: bool,  // bit 2
}

impl BytecodeHeader {
    pub fn parse(byte: u8) -> Result<Self, Error>;
    pub fn to_byte(self) -> u8;
}
```

Add `pub use bytecode::header::BytecodeHeader;` to `lib.rs`.

- [ ] **Step 4.1.3: Verify**

Run `cargo test -p mk-codec --lib bytecode::header::`. All Task-4.1 tests pass; no `#[ignore]` markers remain.

### Task 4.2 — Path codec (TDD)

- [ ] **Step 4.2.0: Verify md1 dictionary contents byte-for-byte before forking**

The closure design Q-3 framing commits mk1 to "mirror md1's `Tag::SharedPath` precedent." Before writing the dictionary, confirm md1's table values exactly. Read `/scratch/code/shibboleth/descriptor-mnemonic/bip/bip-mnemonic-descriptor.mediawiki` §"Path dictionary" and md1's source `crates/md-codec/src/bytecode/path/`. Capture the indicator → path mapping verbatim.

- [ ] **Step 4.2.1: Tests first**

- Round-trip every entry in the standard-table dictionary (14 entries: 7 mainnet `0x01..=0x07`, 7 testnet `0x11..=0x17`).
- Round-trip ~10 explicit paths covering: depth=1 (just `m/0`), depth=10 (cap), depth=11 (rejected → `Error::PathTooDeep`), hardened/non-hardened mixes, edge u32 values (0, 0x7FFFFFFF, 0x80000000, 0xFFFFFFFF).
- Sad paths: indicator `0x00` and `0xFF` (reserved) → `Error::InvalidPathIndicator(_)`; truncated LEB128 → `Error::UnexpectedEnd`; out-of-BIP-32-range component → `Error::InvalidPathComponent(_)`.

- [ ] **Step 4.2.2: Standard-table dictionary**

`pub const STANDARD_PATHS: &[(u8, &str)]` — exactly the 14 entries verified in 4.2.0 (mainnet `0x01..=0x07`, testnet `0x11..=0x17`). No additions, no omissions; mk1's table is byte-identical to md1's `Tag::SharedPath` table.

Helpers: `lookup_indicator(indicator: u8) -> Option<DerivationPath>`; reverse `lookup_path(path: &DerivationPath) -> Option<u8>`.

- [ ] **Step 4.2.3: Explicit-path encode/decode**

`encode_explicit_path(path: &DerivationPath) -> Vec<u8>`: emits `[0xFE, count, leb128(c1), ..., leb128(cN)]`.

`decode_explicit_path(bytes: &mut &[u8]) -> Result<DerivationPath, Error>`: rejects `count > MAX_PATH_COMPONENTS` (= `Error::PathTooDeep`), rejects malformed LEB128 (= `Error::InvalidPathComponent`), rejects unexpected EOF (= `Error::UnexpectedEnd`).

- [ ] **Step 4.2.4: Path codec entry + lib.rs re-export**

`encode_path(path) → Vec<u8>` chooses standard-table indicator if `lookup_path` matches; falls through to explicit otherwise.

`decode_path(bytes) → Result<DerivationPath>` reads indicator byte; dispatches to standard-table lookup or explicit-path decode; rejects reserved indicators.

Add `pub use bytecode::path::{encode_path, decode_path, STANDARD_PATHS, MAX_PATH_COMPONENTS};` to `lib.rs`.

- [ ] **Step 4.2.5: Verify**

Run `cargo test -p mk-codec --lib bytecode::path::`. All tests from 4.2.1 now pass.

### Task 4.3 — Xpub compact-73 codec (TDD)

- [ ] **Step 4.3.1: Tests first**

- Round-trip: `Xpub` → `XpubCompact::from(&xpub)` → `encode_xpub_compact` → `decode_xpub_compact` → `reconstruct_xpub(compact, &path)` → byte-equal to the original `Xpub`. Test for paths at depth 1, 4 (BIP 48 multisig), 10 (cap), and 0 (master xpub).
- Sad path: bytes with version prefix not in {`xpub`, `tpub`} → `Error::InvalidXpubVersion(_)`.
- Sad path: truncated input (< 73 bytes) → `Error::UnexpectedEnd`.
- Inspection: `XpubCompact` exposes `version`, `parent_fingerprint`, `chain_code`, `public_key` as pub fields so debug tooling and the future `decoder-error-variant-parity` audit can read the on-wire form before reconstruction.

- [ ] **Step 4.3.2: Type (pub for inspection)**

```rust
// crates/mk-codec/src/bytecode/xpub_compact.rs
pub struct XpubCompact {
    pub version: [u8; 4],
    pub parent_fingerprint: [u8; 4],
    pub chain_code: [u8; 32],
    pub public_key: [u8; 33],
}                                  // total 73 B
```

`impl From<&Xpub> for XpubCompact`: extracts the four preserved fields.

`pub fn reconstruct_xpub(compact: &XpubCompact, path: &DerivationPath) -> Xpub`: rebuilds the full 78-byte form by computing `depth = path.len() as u8` and `child_number = path.last_or_zero()` including the hardened-bit encoding.

- [ ] **Step 4.3.3: Wire codec**

`pub fn encode_xpub_compact(xpub: &XpubCompact, out: &mut Vec<u8>)`: writes 73 bytes.

`pub fn decode_xpub_compact(bytes: &mut &[u8]) -> Result<XpubCompact>`: reads 73 bytes; rejects `Error::InvalidXpubVersion(_)` if version doesn't match a known mainnet/testnet xpub prefix.

Add `pub use bytecode::xpub_compact::{XpubCompact, encode_xpub_compact, decode_xpub_compact, reconstruct_xpub};` to `lib.rs`.

- [ ] **Step 4.3.4: Verify**

Run `cargo test -p mk-codec --lib bytecode::xpub_compact::`. All tests pass.

### Task 4.4 — Top-level bytecode encode/decode (TDD)

- [ ] **Step 4.4.1: Tests first**

Build the test fixtures and assertions before any encode/decode impl:

- Fixtures: `KeyCard` with 1-stub mainnet (BIP 48 std-table) + fp present; 3-stub mainnet + fp present; 1-stub testnet; 1-stub mainnet + `origin_fingerprint = None` (fp omitted); 1-stub mainnet with explicit-path (forces non-std-table).
- Round-trip: for each fixture, `decode_bytecode(encode_bytecode(card)) == card`.
- Canonicality: encode the same card twice and assert byte-equal output.
- Sad paths (one test per SPEC §4 rule): unsupported version, reserved bits set, stub_count == 0, invalid path indicator, path-too-deep, invalid path component, invalid xpub version, fingerprint flag/payload mismatch, unexpected end, trailing bytes.

Mark `#[ignore]` until 4.4.2/4.4.3/4.4.4 land; remove markers in 4.4.5.

- [ ] **Step 4.4.2: Update `KeyCard`** (breaking change to scaffold; called out in commit message)

`origin_fingerprint: Fingerprint` → `origin_fingerprint: Option<Fingerprint>` (per closure Q-8). Existing `tests/round_trip.rs` `types_compile` test stays green; the type is `#[non_exhaustive]` so external constructors aren't broken.

- [ ] **Step 4.4.3: Encoder**

```rust
// crates/mk-codec/src/bytecode/encode.rs
pub fn encode_bytecode(card: &KeyCard) -> Result<Vec<u8>>;
```

Layout per closure Q-6:
- bytecode_header (1) — bit 2 set iff `card.origin_fingerprint.is_some()`
- stub_count (1) — MUST be ≥ 1 (return `Error::InvalidPolicyIdStubCount` if `card.policy_id_stubs.is_empty()`)
- stubs (4N)
- origin_fingerprint (4 if `Some`, omitted otherwise)
- origin_path (variable; via `bytecode::path::encode_path`)
- xpub_compact (73; via `bytecode::xpub_compact::encode_xpub_compact`)

- [ ] **Step 4.4.4: Decoder**

```rust
pub fn decode_bytecode(bytes: &[u8]) -> Result<KeyCard>;
```

Reverses encoder; applies all SPEC §4 validity rules including the new `Error::FingerprintFlagMismatch` (encoder/decoder disagree on whether `origin_fingerprint` is in payload). Returns `Error::TrailingBytes` if any bytes remain after the xpub.

- [ ] **Step 4.4.5: Verify and remove `#[ignore]`**

Run `cargo test -p mk-codec --lib bytecode::`. Remove `#[ignore]` markers from 4.4.1's tests; full bytecode-layer suite passes.

### Task 4.5 — Build + commit + review

- [ ] **Step 4.5.1: Build + test**

```bash
cargo test -p mk-codec --lib bytecode::
```

Expected: all bytecode-layer tests passing; previously `#[ignore]`-marked sad-path tests un-ignored where Phase 4 lands their code paths.

- [ ] **Step 4.5.2: Commit**

```
feat(mk-codec phase 4): bytecode-layer encoder + decoder

Implements the closure-locked wire format at the bytecode level
(KeyCard <-> Vec<u8>; pre-chunking). Submodules:

- bytecode/header.rs   — BytecodeHeader with bit-2 fingerprint flag
- bytecode/path.rs     — Standard-table dictionary (14 entries; byte-identical to md1's Tag::SharedPath) + 0xFE explicit codec
- bytecode/xpub_compact.rs — 73-byte form with depth/child_number reconstruction from origin_path
- bytecode/encode.rs   — top-level encoder
- bytecode/decode.rs   — top-level decoder

BREAKING CHANGE TO SCAFFOLD: KeyCard.origin_fingerprint is now
Option<Fingerprint> per closure Q-8 (bit-2 fingerprint-flag).
KeyCard remains #[non_exhaustive], so external consumers are
not affected by the field-type change.

Round-trip and sad-path tests cover every SPEC §4 validity rule.
String-layer (BCH + chunking) is Phase 5.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

- [ ] **Step 4.5.3: Phase 4 review**

Dispatch Opus reviewer.

- Files: every file under `crates/mk-codec/src/bytecode/`; updated `key_card.rs`.
- Verify: every closure ripple item is reflected in code; canonicality (deterministic encoding); every Error variant from SPEC §4 has a triggering test; xpub depth/child_number reconstruction correct; field order matches closure Q-6 byte-for-byte.
- Output: `design/agent-reports/v0-1-phase-4-review-<commit>.md`.

---

## Phase 5 — String layer (BCH + chunking, forked from md-codec) (TDD)

**Goal:** Layer the BCH error-correction and chunking on top of the bytecode codec, completing the public `encode(card) -> Vec<String>` / `decode(strings) -> KeyCard` API.

**Strategy:** Fork `md-codec`'s BCH + HRP-mixing modules verbatim per D-13. Adjust target residues to mk1's locked constants. Implement string-layer header (single-string vs chunked variants) and the chunking layer with cross-chunk integrity hash. End-to-end TDD against the synthetic fixtures from Phase 4.

**Per-phase agent report:** `design/agent-reports/v0-1-phase-5-review-<commit>.md`

### Task 5.1 — Fork BCH primitives from md-codec

- [ ] **Step 5.1.1: Identify md-codec's BCH module**

In `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/`, locate the BCH polymod implementation, HRP-expansion helpers, syndrome computation, and erasure/substitution correction.

- [ ] **Step 5.1.2: Copy + adapt**

Copy into `crates/mk-codec/src/string_layer/bch.rs`. Adapt:
- HRP literal: `"md"` → `"mk"`.
- Target residues: `T_REGULAR` / `T_LONG` constants → `MK_REGULAR_CONST` / `MK_LONG_CONST`.
- File header comment notes the fork date and the eventual `mc-codex32` extraction (D-13).

- [ ] **Step 5.1.3: Sanity tests**

Confirm `polymod` of HRP-expanded `"mk"` + a known-good payload + checksum yields `MK_LONG_CONST` (or `MK_REGULAR_CONST`). Hand-construct one or two test vectors against the fork.

### Task 5.2 — String-layer header

- [ ] **Step 5.2.1: Type**

```rust
// crates/mk-codec/src/string_layer/header.rs
pub enum StringLayerHeader {
    SingleString { version: u8 },  // 2 chars: version (5 bits) + type=0 (5 bits)
    Chunked {
        version: u8,
        chunk_set_id: u32,    // 20 bits
        total_chunks: u8,     // 5 bits, range 1..=32
        chunk_index: u8,      // 5 bits, range 0..total_chunks
    },
}
```

- [ ] **Step 5.2.2: Bech32 encode/decode**

Read/write the 2-char or 8-char header using bech32-alphabet symbol values. Validate against Error variants:
- `UnsupportedCardType(_)` for type byte not in {0x00, 0x01}
- `UnsupportedVersion(_)` for version != 0 in v0.1
- `ChunkedHeaderMalformed(_)` for total_chunks=0 or chunk_index >= total_chunks

### Task 5.3 — Chunking + cross-chunk integrity hash

- [ ] **Step 5.3.1: Encoder split**

```rust
pub fn split_into_chunks(canonical_bytecode: &[u8], chunk_set_id: u32)
    -> Vec<(StringLayerHeader, Vec<u8>)>;
```

Computes `cross_chunk_hash = SHA-256(canonical_bytecode)[0..4]`, appends, splits the resulting stream into chunks of approximately equal long-code-fragment size (53 bytes each), assigns sequential chunk_index, returns one (header, fragment) pair per chunk.

- [ ] **Step 5.3.2: Decoder reassemble**

```rust
pub fn reassemble_from_chunks(chunks: Vec<(StringLayerHeader, Vec<u8>)>)
    -> Result<Vec<u8>>;
```

Validates: all chunks share `chunk_set_id` (else `ChunkSetIdMismatch`); chunk_index values cover `0..total_chunks` exactly once (else `ChunkedHeaderMalformed`); concatenates fragments in index order; verifies `cross_chunk_hash` (else `CrossChunkHashMismatch`); returns the bytecode less the trailing 4-byte hash.

### Task 5.4 — Public encode/decode entry (TDD)

**Encoding policy decisions (pinned for v0.1):**

1. **Code-variant selection.** *Single-string mode:* use the regular code (13-char checksum) iff total data ≤ 93 chars (= payload ≤ `SINGLE_STRING_REGULAR_BYTES = 48`); else use the long code (15-char checksum). *Chunked mode:* v0.1 emits **long code (15-char checksum) for all chunked fragments**, matching md1's published convention (BIP §"Length envelope": "Encoders that produce a header + payload exceeding 93 characters MUST use the long code"). Mixed-code-per-chunk is wire-permitted but not v0.1 emit policy.
2. **`chunk_set_id` RNG.** Use the `getrandom` crate (CSPRNG) to produce a `u32`, masked to 20 bits (`x & 0x000FFFFF`). The `encode` API also accepts an optional explicit `chunk_set_id` parameter for deterministic encoding (vector regen, conformance tests). Default-randomized; explicit-override available.

- [ ] **Step 5.4.1: Tests first**

End-to-end fixtures (extending Phase 4's set):

- E1–E4: every KeyCard fixture from Phase 4, fed through the public `encode`/`decode` API with deterministic `chunk_set_id = 0x12345`. Round-trip byte-equal.
- E5: multi-chunk case (KeyCard whose bytecode exceeds 56 bytes). Confirm: returned `Vec<String>` has `total_chunks` entries; each starts with `mk1`; `decode` reassembles to the same KeyCard.
- E6: regular-vs-long single-string boundary (KeyCard with payload at 48 bytes uses regular; at 49 bytes uses long).
- E7: explicit `chunk_set_id = None` produces a string set with a 20-bit `chunk_set_id` value (mask check; not byte-stability).
- Sad: `decode` rejects a chunked input with one chunk's `chunk_set_id` flipped → `Error::ChunkSetIdMismatch`.
- Sad: `decode` rejects a chunked input with cross-chunk hash perturbed → `Error::CrossChunkHashMismatch`.
- Sad: `decode` rejects a SingleString input whose 5-bit padding bits are non-zero → `Error::MalformedPayloadPadding`.

`#[ignore]`-marked until 5.4.2/5.4.3 land; un-ignored in 5.4.4.

- [ ] **Step 5.4.2: Top-level encoder**

```rust
// crates/mk-codec/src/key_card.rs (rewritten)
pub fn encode(card: &KeyCard) -> Result<Vec<String>>;

/// Like `encode`, with an explicit `chunk_set_id` override
/// (deterministic encoding for vector regeneration / tests).
pub fn encode_with_chunk_set_id(card: &KeyCard, chunk_set_id: u32) -> Result<Vec<String>>;
```

Pipeline:
1. `bytecode = encode_bytecode(card)`.
2. If `bytecode.len() <= SINGLE_STRING_LONG_BYTES`, take SingleString path: choose regular code (≤ 48 bytes) vs long code (49–56 bytes). One-element `Vec<String>`.
3. Else multi-chunk: derive `chunk_set_id` (CSPRNG-masked-to-20-bits unless caller-supplied); call `split_into_chunks`. v0.1 always uses long-code per chunked fragment.
4. For each (header, fragment): encode header to 2/8 bech32 chars; convertbits 8→5 the fragment; compute BCH checksum (regular or long per pinned policy above); assemble final string `mk1<header><payload><checksum>`.

- [ ] **Step 5.4.3: Top-level decoder**

```rust
pub fn decode(strings: &[&str]) -> Result<KeyCard>;
```

Pipeline:
1. For each string: validate HRP `"mk"`; verify BCH (with correction) — emit decoder report. Mixed regular/long code per-chunk is decoder-permitted (forward compat).
2. Strip checksum; convertbits 5→8 with byte-align validation (= `Error::MalformedPayloadPadding` if non-zero pad bits).
3. Parse string-layer header: SingleString or Chunked.
4. If multi-chunk: collect all (header, fragment) pairs, call `reassemble_from_chunks`.
5. Pass reassembled bytecode to `decode_bytecode`.

- [ ] **Step 5.4.4: Verify and remove `#[ignore]`**

`cargo test -p mk-codec`. All E1–E7 tests pass; sad-path Error variants fire as expected.

### Task 5.5 — Build + commit + review

- [ ] **Step 5.5.1: Build + full test**

```bash
cargo test -p mk-codec
```

Expected: all tests pass including round-trips.

- [ ] **Step 5.5.2: Commit**

```
feat(mk-codec phase 5): string-layer (BCH + chunking)

Forks md-codec's BCH primitives and adapts to mk1's HRP and
target residues. Implements:

- string_layer/bch.rs    — polymod, syndromes, correction (forked)
- string_layer/header.rs — single-string + chunked headers
- string_layer/chunk.rs  — split/reassemble + cross-chunk integrity hash
- key_card.rs (rewritten encode/decode)

Public API:
  pub fn encode(card: &KeyCard) -> Result<Vec<String>>
  pub fn decode(strings: &[&str]) -> Result<KeyCard>

End-to-end round-trip tests cover SingleString and multi-chunk paths.
Per-error-variant sad paths complete; every SPEC §4 validity rule
has a triggering test.

Per D-13, BCH primitives are forked-not-shared. The
mc-codex32 extraction trigger (Q-9) is unchanged.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

- [ ] **Step 5.5.3: Phase 5 review**

Dispatch Opus reviewer.

- Files: every file under `crates/mk-codec/src/string_layer/`; rewritten `key_card.rs`.
- Verify: BCH polymod correctness (cross-check against md-codec); HRP-mixing emits the documented residue; chunking math matches closure §2.4 capacity numbers; cross-chunk hash byte-exact match against `SHA-256(bytecode)[0..4]`; chunk_set_id uses 20 bits not 32 or 16.
- Output: `design/agent-reports/v0-1-phase-5-review-<commit>.md`.

---

## Phase 6 — Initial vector corpus

**Goal:** Produce `crates/mk-codec/tests/vectors/v0.1.json` — the canonical test corpus future implementations will validate against. Anchor with `mk-codec 0.1` family token + SHA pin.

**Strategy:** Build 5–8 hand-curated KeyCard fixtures covering the closure's wire-format diagonal: typical 1-stub mainnet, 1-stub testnet, multi-stub, fingerprint-omitted, explicit-path. Encode each; capture (input KeyCard, expected mk1 string(s), expected canonical bytecode hex, decoder-correction report on clean input). Pin SHA-256 of the JSON file in `tests/vectors_schema.rs`.

**Per-phase agent report:** `design/agent-reports/v0-1-phase-6-review-<commit>.md`

### Task 6.1 — Vector fixtures

- [ ] **Step 6.1.1: Define fixture set**

Minimum coverage:
- V1: 1-stub mainnet, BIP 48 multisig (`m/48'/0'/0'/2'`), fingerprint present, single-string-fitting.
- V2: 1-stub mainnet, BIP 84 single-sig (`m/84'/0'/0'`), fingerprint present, single-string-fitting.
- V3: 1-stub testnet, BIP 48 testnet (`m/48'/1'/0'/2'`), fingerprint present.
- V4: 1-stub mainnet, **fingerprint omitted** (bit 2 unset), single-string-fitting.
- V5: 1-stub mainnet, **explicit path** `m/9999'/1234'/56'/7'`, fingerprint present (forces multi-chunk via path length).
- V6: 3-stub mainnet, fingerprint present (multi-chunk from stub count).
- V7: maximum-component-count explicit path (`component_count = 10`), fingerprint omitted.
- V8: dictionary boundary case — BIP 87 multisig (`m/87'/0'/0'`).

- [ ] **Step 6.1.2: Generator binary + JSON canonicality**

`cargo run --bin gen_mk_vectors -- --output crates/mk-codec/tests/vectors/v0.1.json`

**Canonicality discipline (pin BEFORE the SHA pin, otherwise patch-version JSON-tooling drift will roll the SHA):**

- Keys sorted alphabetically at every nesting level.
- Hex literals lowercase (`a`–`f`, not `A`–`F`).
- Byte-array fields rendered as continuous hex strings (no `0x` prefix, no separators).
- Indentation: 2 spaces; trailing newline at EOF; LF line endings; no trailing whitespace.
- The generator binary writes via `serde_json::to_writer_pretty` with a custom serializer that enforces sorted keys and lowercase hex; alternatively, a post-write `jq -S` pass to canonicalize. Document the chosen approach in a header comment in the generator source.
- Deterministic encoding: every vector specifies an explicit `chunk_set_id` (multi-chunk vectors) so re-running the generator produces byte-identical output.

Schema (mirrors md1's vector schema):

```json
{
  "schema": 1,
  "family_token": "mk-codec 0.1",
  "vectors": [
    {
      "name": "V1_bip48_mainnet_1_stub",
      "input": {
        "policy_id_stubs": ["..."],
        "origin_fingerprint": "d34db33f",
        "origin_path": "m/48'/0'/0'/2'",
        "xpub": "xpub6...",
        "chunk_set_id": 305419896
      },
      "expected_canonical_bytecode_hex": "...",
      "expected_strings": ["mk1..."],
      "expected_decoder_correction": "clean"
    },
    ...
  ]
}
```

The `chunk_set_id` field is required for multi-chunk vectors (deterministic) and ignored for single-string vectors.

### Task 6.2 — Vector test harness

- [ ] **Step 6.2.1: `tests/vectors.rs`**

For each vector entry:
- Decode `expected_canonical_bytecode_hex` and verify it matches `encode_bytecode(input)` byte-for-byte.
- Verify `expected_strings` matches `encode(input)` element-by-element.
- Verify `decode(expected_strings)` returns a KeyCard equal to `input`.

- [ ] **Step 6.2.2: SHA pin**

`crates/mk-codec/tests/vectors_schema.rs`:

```rust
pub const V0_1_SHA256: &str = "<hex>";
```

Pinned SHA-256 of the JSON file. CI test verifies file matches pinned SHA.

### Task 6.3 — Build + commit + review

- [ ] **Step 6.3.1: Generate, sha-pin, test**

```bash
cargo run --bin gen_mk_vectors -- --output crates/mk-codec/tests/vectors/v0.1.json
sha256sum crates/mk-codec/tests/vectors/v0.1.json
# Update tests/vectors_schema.rs::V0_1_SHA256 with the new value.
cargo test -p mk-codec
```

- [ ] **Step 6.3.2: Commit**

```
feat(mk-codec phase 6): initial vector corpus

8 hand-curated test vectors covering wire-format diagonal:
  V1 BIP 48 mainnet 1-stub, fingerprint present
  V2 BIP 84 single-sig mainnet 1-stub
  V3 BIP 48 testnet
  V4 fingerprint omitted (bit 2 unset)
  V5 explicit path forcing multi-chunk
  V6 3-stub multi-chunk
  V7 max-cap explicit path (component_count = 10)
  V8 BIP 87 multisig boundary case

Family token "mk-codec 0.1". SHA-256 pinned in
tests/vectors_schema.rs. Cross-implementations validate by
matching this file's SHA + every vector's round-trip.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

- [ ] **Step 6.3.3: Phase 6 review**

Dispatch Opus reviewer.

- Files: vector JSON; `tests/vectors.rs`; `tests/vectors_schema.rs`.
- Verify: 8 vectors cover the diagonal; SHA pin matches file; every vector's expected_canonical_bytecode_hex regenerates byte-exact via the encoder; every vector decodes round-trip equal.
- Output: `design/agent-reports/v0-1-phase-6-review-<commit>.md`.

---

## Phase 7 — Release plumbing

**Goal:** Cargo bump 0.0.0 → 0.1.0; CHANGELOG; README update; tag `mk-codec-v0.1.0`.

**Strategy:** Mechanical. The plumbing here mirrors md1's release process.

### Task 7.1 — Cargo bump

- [ ] **Step 7.1.1**: `crates/mk-codec/Cargo.toml`: `0.0.0` → `0.1.0`. Update description: drop "design-stage skeleton" wording.

### Task 7.2 — CHANGELOG (NEW at repo root)

- [ ] **Step 7.2.1**: Create `CHANGELOG.md`:

```markdown
# Changelog

All notable changes to mk-codec will be documented in this file.

## [0.1.0] — 2026-04-XX

First reference implementation of the Mnemonic Key (MK) backup format.

### Added
- Working encode/decode round-trip for mk1-prefixed strings.
- BCH error-correction layer forked from md-codec per design D-13.
- Compact-73 xpub form (depth + child_number reconstructed from origin_path).
- Optional origin_fingerprint via bytecode-header bit 2.
- Standard-table path dictionary (BIP 44/49/84/86/48-segwit/48-nested/87 +
  testnet variants) + 0xFE explicit-path escape.
- Initial vector corpus (8 vectors) anchored under family token "mk-codec 0.1".

### Notes
- Wire format finalized per the v0.1 closure design (Q-1..Q-10 closed).
- Pre-BIP-submission audit items remain (see `design/FOLLOWUPS.md`).
- Eventual `mc-codex32` extraction (Q-9) deferred until both md-codec and
  mk-codec reach v1.0 with cross-validated conformance.
```

### Task 7.3 — README update

- [ ] **Step 7.3.1**: `README.md`: drop "design-stage skeleton, no implementation" framing in the status block; replace with "v0.1 reference implementation shipped; pre-BIP-submission audit items pending — see design/FOLLOWUPS.md."

### Task 7.4 — Tag

- [ ] **Step 7.4.1**: After Phase 7 commit lands and CI passes:

```bash
git tag -a mk-codec-v0.1.0 -m "mk-codec v0.1.0 — first reference implementation"
git push origin mk-codec-v0.1.0
```

### Task 7.5 — Optional Phase 7 review

(Optional: Phase 7 is mechanical; skip if time-constrained. If run, save report to `design/agent-reports/v0-1-phase-7-review-<commit>.md`.)

---

## Phase 8 — Final reconciliation

### Task 8.1 — Reconcile agent reports vs FOLLOWUPS

- [ ] **Step 8.1.1**: List all `design/agent-reports/v0-1-*` reports; for each minor item, verify a `design/FOLLOWUPS.md` entry exists. The pre-BIP-submission tier items already pre-exist (closure design seeded them); per-phase reviews may have added more.

### Task 8.2 — Update PR + memory

- [ ] **Step 8.2.1**: PR description: completed checkbox states across all phases.
- [ ] **Step 8.2.2**: Add v0.1.0 entry to project memory.

### Task 8.3 — Cross-repo coordination follow-up

- [x] **Step 8.3.1**: ~~After mk-codec v0.1.0 ships, file an issue / PR / message to the descriptor-mnemonic repo for the `chunk-set-id-rename` and `md-per-N-path-tag-allocation` follow-ups (per `design/FOLLOWUPS.md`). These are sequencing prerequisites for mk1's eventual BIP submission, not for v0.1 release itself.~~ Resolved upstream: `chunk-set-id-rename` shipped in [md-codec v0.9.0](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0); `md-per-N-path-tag-allocation` shipped in [md-codec v0.10.0](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.10.0) as `Tag::OriginPaths = 0x36`. Both BIP-submission gates cleared on the md1 side without any explicit issue-filing from mk1.

---

(End of mk1 v0.1.0 implementation plan.)
