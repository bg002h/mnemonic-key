# `mk1` design — decisions log

Captures the design decisions made about extending the MD ecosystem with a sibling format for engraving extended public keys (xpubs). Started 2026-04-29.

This is a **rolling decisions log**, not a spec. It exists so that the rationale for each decision is traceable when later questions arise.

The companion artifacts:

- `design/SPEC_mk_v0_1.md` — the wire-format spec (now post-closure)
- `design/IMPLEMENTATION_PLAN_mk_v0_1.md` — the v0.1 implementation plan
- `design/FOLLOWUPS.md` — deferred items including pre-BIP-submission audit gates
- `bip/bip-mnemonic-key.mediawiki` — the BIP draft
- `crates/mk-codec/` — the reference implementation

## Context

MD (HRP `md1`) currently encodes only the BIP 388 *template* (the `@N`-placeholder shape); the xpubs that map `@N` → key never make it onto the engraved card. Recovery works only if the user holds every seed referenced by the policy.

For wallets where the user holds every seed, MD's omission is fine — the seeds reconstruct the xpubs, the descriptor is reconstructable from policy template + seeds. For **foreign-xpub multisig** (the user is one of N cosigners and doesn't hold all seeds), MD alone is insufficient: the missing xpubs have to come from somewhere at recovery time.

`mk1` is the proposed companion format: a codex32-derived backup format for individual xpubs, with HRP `mk`, designed to engrave separately from the policy card. The conceptual triad is:

| Format | HRP | Purpose | Reference |
|---|---|---|---|
| Mnemonic Seed share | `ms1` | Seed shares (codex32) | BIP 93 (Andrew Poelstra) |
| Mnemonic Descriptor | `md1` | Wallet-policy template | This project, MD BIP |
| Mnemonic Key | `mk1` | Extended public keys | This project, mk BIP (TBD) |

## Decisions

### D-1. Separate "key card" architecture, not embedded xpubs in policy card

**Decision.** Each cosigner's xpub is engraved on its own card, separate from the policy template card. Key cards are produced and stored independently from the policy card.

**Reasoning.**

- Real wallets where the user holds all seeds don't need xpub backup — the seeds reconstruct everything. Only foreign-xpub multisig genuinely needs xpub backup.
- Embedding all xpubs in the policy card balloons single-card payload (each xpub ≈ 78 bytes). A 7-cosigner wallet would always need multi-chunk encoding.
- Separate cards let cosigners back up their own xpubs once and re-use them across multiple wallets they participate in.
- Coldcard's `.txt` multisig config and Sparrow's `.json` both pack xpubs with the policy. mk's separate-card approach diverges deliberately to optimize for the engraved-on-steel use case where atomic per-cosigner cards are easier to manage.

### D-2. Wire-format extension on the **policy card**: per-`@N` derivation paths

**Decision.** Extend MD's bytecode to optionally encode a per-placeholder origin-path indicator. Today all `@N` placeholders share one path (via `Tag::SharedPath`). The extension lets each `@N` carry its own origin path.

**Reasoning.**

- BIP 388 wallet policies routinely have different origin paths per cosigner (e.g. `[fp1/48'/0'/0'/2']xpub_A` and `[fp2/48'/0'/0'/100']xpub_B` in the same multisig). Without per-`@N` paths, MD cannot losslessly encode such policies.
- Real foreign-multisig setups don't follow a single shared origin path even when all cosigners use the same wallet vendor.

### D-3. Path encoding: dual mode (standard indicator + explicit-path escape hatch)

**Decision.** When encoding a path (whether the shared one or a per-`@N` override), support two encodings:

- **Standard-table 1-byte indicator**, mirroring `Tag::SharedPath`'s existing dictionary (BIP 44 / 49 / 84 / 86 / 48 / 48-nested / 87, plus testnet variants).
- **Explicit-path escape hatch** for arbitrary paths, marked by a reserved indicator byte (`0xFE` is the existing precedent), followed by LEB128-encoded path components.

**Reasoning.**

- Refusing arbitrary paths means real, valid BIP 388 wallet policies have no MD backup. That's a feature gap, not discipline.
- Coldcard, Sparrow, Liana, BIP 388, BIP 380 all admit arbitrary paths. MD being narrower caps interop.
- Lock-out failure mode is bad: a user with a non-standard setup discovers at recovery time that their backup format can't represent their wallet.
- The bytes cost itself is the discouragement: a standard path = 1 byte, a 4-component arbitrary path = ~10–15 bytes. Long backups → more chunks → natural nudge toward standard accounts without locking out the legitimate weirdo.
- The "you should use standard paths" opinion belongs in wallet UX (red text, "this will produce a longer backup"), not in the format itself.

### D-4. Hard limits on encoded paths

**Decision.** Reject paths that:

- Are not valid BIP 32 derivation paths (component out of range, malformed hardened-bit encoding, etc.).
- Exceed BIP 32's depth ceiling (~10 components in practice; spec literally allows 255 but no real wallet uses more than ~6).
- Exceed mk1's max-component cap (closure Q-3 locked at **10**; see SPEC §3.5).

**Reasoning.**

- Correctness gate, not a UX opinion. Rejecting these protects implementations from malformed inputs without any tradeoff.

### D-5. mk1: one xpub per card (atomic, no bundling)

**Decision.** Each `mk1`-prefixed string encodes exactly one xpub plus its metadata. No "bundle of N xpubs" card variant.

**Reasoning.**

- Atomic per-cosigner cards: lose one card = lose one cosigner's xpub. With a bundle card, lose one card = lose every cosigner's xpub on that bundle.
- One-card-per-cosigner makes the recovery story easier to reason about (`@0` from card A, `@1` from card B, ...).
- Cosigners can hand their own key card to the wallet creator without revealing other cosigners' xpubs — atomic cards align with the trust boundary.

### D-6. Linkage via Policy ID

**Decision.** Each key card carries one or more Policy ID stubs identifying which MD-encoded policy card(s) it serves.

(Naming note: the value formerly called "wallet ID" was renamed to "Policy ID" in md-codec v0.8.0, since it hashes the BIP 388 template only and two wallets sharing a template share the value. mk1 adopts the renamed nomenclature throughout.)

**Reasoning.**

- The cryptographic recovery check doesn't strictly need the Policy ID on the key card — once xpubs are inserted into `@N` slots, the recomputed `Wallet Instance ID` either matches the user's expected wallet identity or it doesn't. So the Policy ID on the key card is an **indexing aid** ("which storage-box drawer does this card go in"), not a security primitive.
- Without indexing, recovery flow is "try every key card in every `@N` slot until the wallet ID matches." Workable but ugly. With indexing, recovery is "match key cards to policy by stamped Policy ID, then validate."

### D-7. Policy IDs per key card: ≥1 required (default = 1)

**Decision.** Each key card carries at least one Policy ID stub. The default is exactly one (the Policy ID at the time of engraving). Cosigners may stamp additional Policy ID stubs if their xpub serves multiple wallets they participate in.

**Rejected alternatives.**

- *Exactly one* Policy ID: simplest, but cosigners participating in multiple wallets need multiple key cards for the same xpub. Multiplies engraving work for no cryptographic benefit, since xpubs are reusable across wallets in practice.
- *Zero allowed* (anonymous key cards): best privacy, but recovery has no automated cross-check against mis-filed cards. The privacy gain is small (the cosigner can stick to one Policy ID per card and engrave more) and the UX cost is real.

**Caveat.** Stamping multiple Policy IDs on one key card means recovery for *any* of those wallets reveals (to the recoverer) that the cosigner is in *all* of them. If the cosigner cares about cross-wallet privacy, they should stick to one Policy ID per card and engrave more cards.

### D-8. Separate BIP for `mk1`

**Decision.** mk1 is its own BIP draft, not a sub-format or extension of MD's BIP. The two specs cross-reference each other (mk references MD for Policy ID linkage protocol; MD optionally references mk for the foreign-xpub recovery pattern), but neither is a structural dependency of the other.

**Reasoning.**

- An xpub backup format is useful even outside MD: watch-only wallet provisioning, master-public-key archival, key-rotation continuity. Coupling mk to MD's release cadence is artificial.
- Different audiences and timelines: MD's "wallet descriptor" framing has a specific multisig/foreign-xpub problem statement; key backup is a more general concern.
- Cleaner evolution: mk's wire-format decisions are governed by their own concerns. Tying them to md's lifecycle means every md revision risks pulling in mk-irrelevant constraints, and vice versa.
- Existing precedent: BIP 93 stayed scoped to seed shares; it didn't try to be "BIP 32 + bech32 + recovery." mk staying scoped to xpub backup is consistent.

### D-9. HRP `mk`

**Decision.** mk1 uses HRP `mk` — alphabetical extension of the `ms`/`md` codex32-family namespace.

**Reasoning.** Two-letter HRP, mnemonic ("mnemonic key"), no known collisions with bech32/Lightning/codex32 HRPs to date. Anyone reaching for `mk` in the codex32-derived namespace is most plausibly thinking exactly what we are.

**Verification gate before formal registration.** Search SLIP-0173 (informal segwit-HRP registry) and recent bitcoin-dev mailing-list archives + BIPs PR history for any soft `mk` claim. None expected, but should be confirmed before publishing a draft BIP. Tracked as `hrp-mk-collision-check` in `FOLLOWUPS.md` at tier `pre-bip-submission`.

**Alternatives considered (if collision found later):** `mx` (mnemonic xpub, unambiguous via the `x`), `mkc` (mnemonic key card), `mpk` (mnemonic public key). Pick if `mk` is contested.

### D-10. BCH plumbing: reuse BIP 93's polynomial; new NUMS-derived target constants

**Decision.** mk1 reuses BIP 93's BCH generator polynomials verbatim (same as md1). Domain separation between mk1, md1, and codex32 is provided by:

1. Different per-format target residue constants (`MK_REGULAR_CONST`, `MK_LONG_CONST`), NUMS-derived from a fresh domain-separation string (closure Q-1: `b"shibbolethnumskey"`).
2. HRP-mixing in the polymod (BIP 173-style HRP expansion).

**Reasoning.**

- BCH polynomials are not cryptographic secrets; they're chosen for minimum-distance and weight-distribution properties. Sharing them across formats does not weaken either format's error-correction guarantees.
- Computing a fresh polynomial from scratch is CPU-hours of search and would require independent analysis to claim equivalent guarantees. No actual security gain.
- Precedent: bech32 itself uses one polynomial across `bc`/`tb`/`bcrt`, with HRP-mixing providing the domain separation. BIP 350 explicitly designed for this pattern.
- md-codec already does exactly this: reuses BIP 93's generator polynomials, defines `MD_REGULAR_CONST` and `MD_LONG_CONST` as NUMS-derived constants from `SHA-256(b"shibbolethnums")`.

**Caveat.** mk1's NUMS string is independent from md1's `"shibbolethnums"`. Random-looking 65-bit/75-bit residues from independent SHA-256 inputs almost never have meaningful structural relationships, so this is a low-risk gate. A formal structural-relationship audit is tracked as `nums-structural-audit` in `FOLLOWUPS.md` at tier `pre-bip-submission`.

### D-11. Defensive registration strategy

**Decision.** Soft claim `mk1` HRP in this repo's design docs immediately. Defer formal BIP submission until both md and mk are mature and implementation-tested.

**Reasoning.**

- MD's own BIP is still pre-Draft, awaiting human review. Stacking a second BIP draft before the first is reviewed risks spreading reviewer attention thin.
- mk1's design needs to be implementation-validated before formal submission. Pre-BIP-submission audit items (FOLLOWUPS pre-bip-submission tier) gate the formal-submission step.
- The realistic risk isn't "someone races us to `mk`" — it's "someone doesn't notice we're using it and picks `mk` for an orthogonal purpose." Soft-claiming via a public GitHub repo with `bip/bip-mnemonic-key.mediawiki` and `design/` is enough lead time for that risk profile.

**Coordination note.** Andrew Poelstra (codex32 author) is the natural reviewer for mk1's BCH-plumbing reuse story and for the NUMS structural-relationship audit. Loop him in before formal BIP submission to avoid "you missed a structural concern" rework.

### D-12. Repo layout: sibling repo to `descriptor-mnemonic`

**Decision.** `mnemonic-key` is its own sibling repo to `descriptor-mnemonic`. mk-codec lives at `crates/mk-codec/`. mk's BIP draft lives in `bip/`. mk's design discussion lives in `design/`. The eventual `mc-codex32` shared crate per D-13 will live in a third sibling repo.

**Reasoning.** Pre-1.0, design-stage, dense cross-references between md and mk specs, but separate release cadences and reviewer audiences. Sibling-repo with cross-references is the right shape; bundling mk-codec inside `descriptor-mnemonic`'s workspace was considered earlier and foreclosed for release-cadence independence.

### D-13. Plumbing-reuse strategy: fork now, refactor later

**Decision.** mk-codec initially **forks** the BCH primitives from md-codec (option 3 from the architecture-question discussion). Once both formats stabilize, refactor the shared codex32-derived plumbing into a third workspace member (e.g. `mc-codex32`).

**Reasoning.** Premature shared-crate extraction is its own footgun. The plumbing is small enough (~200 lines) that fork-then-merge is cheaper than design-the-shared-API-up-front. Both formats need to be implementation-validated before the shared API can be designed correctly.

**Eventual split commitment (closure Q-9 lock).** The split *will* happen — agreed in this session. Trigger condition: **both md-codec and mk-codec at v1.0 with cross-validated conformance vectors and stable public APIs.** Until then, fork-from-md-codec; both implementations carry their own BCH-primitives copy.

### D-14. Cross-format header parsing alignment with md1

**Decision.** mk1's string-layer header structure mirrors md1's exactly (2-char single, 8-char chunked, identical bit allocation). mk1's bytecode header shares bit-allocation shape with md1, including bit-2 semantics ("optional fingerprint-related block follows") at the cross-format pattern level.

**Reasoning.**

- Enables a common header-parsing helper when D-13's `mc-codex32` extraction happens (Q-9 trigger).
- Implementers writing mk-codec can lift md-codec's header-parsing primitives wholesale.
- Block contents inside the optional bit-2 block differ between formats (md1: 5N-byte fingerprints block; mk1: 4-byte `origin_fingerprint`); the convention is at the bit-level pattern, not the block contents.
- The headers stay independent — versioning, reserved-bit allocation, format-specific flags evolve separately. The convergence is structural, not semantic-locked.

**Surfaced:** 2026-04-29 closure-design pass, motivated by the user's "header parser reuse" pressure question.

### D-15. `chunk_set_id` rename across both repos

**Decision.** The 20-bit per-encoding random tag in the chunked string-layer header is named `chunk_set_id` in mk1 from day 1. md1 v0.8.x originally called the same field "wallet identifier"; that name conflicted with `Policy ID` and `Wallet Instance ID` and was misleading.

**Reasoning.**

- "Wallet identifier" reads as the wallet's ID, but the field is per-encoding random not per-wallet.
- "chunk_set_id" accurately captures the role: identifies all chunks belonging to one card-encoding, used for reassembly mismatch detection, nothing more.
- Wire format is unchanged; this is purely a documentation/code-symbol rename.

**Sequencing requirement (resolved).** The rename was a sequencing prerequisite for mk1's BIP submission — mk1's BIP cites md1 by field name; mk1 could not publish referencing a name md1 itself did not use. **The rename shipped in md-codec v0.9.0 ([release](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0), merge commit `9eeb9ab`); mk1's BIP-submission gate is now cleared.** Originally tracked as `chunk-set-id-rename` in `FOLLOWUPS.md` at tier `cross-repo`; that companion entry is now resolved.

**Surfaced:** 2026-04-29 closure-design pass during cross-format wire-format review.

## Closures (2026-04-29)

All ten v0.1 open questions are closed. See [`docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`](../docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md) for fresh-eyes rationale and the locks.

| ID | Locked answer | Spec section |
|---|---|---|
| Q-1 | Domain `b"shibbolethnumskey"`; `MK_REGULAR_CONST = 0x1062435f91072fa5c`, `MK_LONG_CONST = 0x41890d7e441cbe97273` | SPEC §2.3 |
| Q-2 | 4-byte Policy ID stub | SPEC §3.3 |
| Q-3 | Path-component cap = 10 | SPEC §3.5 |
| Q-4 | mk1 declares authority precedence; md1 tag-byte allocation shipped as `Tag::OriginPaths = 0x36` in [md-codec v0.10.0](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.10.0) | SPEC §5.1 |
| Q-5 | Chunk types `0x00=SingleString`, `0x01=Chunked`; full string-layer header structure pinned | SPEC §2.5 |
| Q-6 | Payload field order: header → stubs → fp → path → xpub_compact | SPEC §3.2 |
| Q-7 | Compact-73 xpub (drop redundant depth + child_number) | SPEC §3.6 |
| Q-8 | Bytecode header bit 2 = optional fingerprint flag | SPEC §3.1 + §6 |
| Q-9 | Split trigger: both formats v1.0 with cross-validated conformance vectors | D-13 above |
| Q-10 | Family token `mk-codec X.Y` | SPEC §7 |

Pre-BIP-submission audit items tracked in `FOLLOWUPS.md` at tier `pre-bip-submission`:

- `nums-structural-audit` — structural-relationship audit of `MK_REGULAR_CONST` / `MK_LONG_CONST`
- `hrp-mk-collision-check` — formal HRP collision verification per D-9
- `bip-cross-reference-completeness` — BIP draft cross-reference audit
- `decoder-error-variant-parity` — Error-variant ↔ negative-vector parity

## Conversation provenance

Decisions D-1 through D-13 were reached interactively in a Claude Code session on 2026-04-29. The author drove the decisions; the assistant surfaced tradeoffs, recommended defaults, and captured the conclusions.

D-14 and D-15 were added 2026-04-29 during the closure-design pass that re-litigated Q-1..Q-10 from scratch. The closure design at `docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md` captures the fresh-eyes rationale; an opus reviewer caught 8 should-address items that were applied inline before the closure design landed on `main`.

The implementation plan at `design/IMPLEMENTATION_PLAN_mk_v0_1.md` sequences the v0.1 work; per-phase opus reviews are persisted to `design/agent-reports/`.
