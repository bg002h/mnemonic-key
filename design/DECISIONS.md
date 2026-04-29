# `mk1` design — decisions log

Captures the design decisions made about extending the MD ecosystem with a sibling format for engraving extended public keys (xpubs). Started 2026-04-29, in conversation with the project author.

This is a **rolling decisions log**, not a spec. It exists so that the rationale for each decision is traceable when later questions arise. Each decision below was reached through interactive design discussion in a Claude Code session; no agent-review checkpoints have been run yet.

The companion artifacts when they exist will be:

- `design/mk/SPEC_mk_v0_1.md` — the actual wire-format spec
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
- Exceed an MD-specific max-component cap. (Exact cap: TBD in the spec; suggested ~32 to bound chunk-size attacks without rejecting any plausibly real path.)

**Reasoning.**

- Correctness gate, not a UX opinion. Rejecting these protects implementations from malformed inputs without any tradeoff.

### D-5. mk1: one xpub per card (atomic, no bundling)

**Decision.** Each `mk1`-prefixed string encodes exactly one xpub plus its metadata. No "bundle of N xpubs" card variant.

**Reasoning.**

- Atomic per-cosigner cards: lose one card = lose one cosigner's xpub. With a bundle card, lose one card = lose every cosigner's xpub on that bundle.
- One-card-per-cosigner makes the recovery story easier to reason about (`@0` from card A, `@1` from card B, ...).
- Cosigners can hand their own key card to the wallet creator without revealing other cosigners' xpubs — atomic cards align with the trust boundary.

### D-6. Linkage via wallet ID

**Decision.** Each key card carries one or more wallet-ID stubs identifying which MD-encoded policy card(s) it serves.

**Reasoning.**

- The cryptographic recovery check doesn't strictly need the wallet ID on the key card — once xpubs are inserted into `@N` slots, the recomputed policy hash either matches the policy card's wallet ID or it doesn't. So the wallet ID on the key card is an **indexing aid** ("which storage-box drawer does this card go in"), not a security primitive.
- Without indexing, recovery flow is "try every key card in every `@N` slot until the wallet ID matches." Workable but ugly. With indexing, recovery is "match key cards to policy by stamped wallet ID, then validate."

### D-7. Wallet IDs per key card: ≥1 required (default = 1)

**Decision.** Each key card carries at least one wallet-ID stub. The default is exactly one (the wallet ID at the time of engraving). Cosigners may stamp additional wallet-ID stubs if their xpub serves multiple wallets they participate in.

**Rejected alternatives.**

- *Exactly one* wallet ID: simplest, but cosigners participating in multiple wallets need multiple key cards for the same xpub. Multiplies engraving work for no cryptographic benefit, since xpubs are reusable across wallets in practice (a hardware-wallet account xpub serving family + business multisig is common).
- *Zero allowed* (anonymous key cards): best privacy, but recovery has no automated cross-check against mis-filed cards. The privacy gain is small (the cosigner can stick to one wallet ID per card and engrave more) and the UX cost is real.

**Caveat.** Stamping multiple wallet IDs on one key card means recovery for *any* of those wallets reveals (to the recoverer) that the cosigner is in *all* of them. If the cosigner cares about cross-wallet privacy, they should stick to one wallet ID per card and engrave more cards.

### D-8. Separate BIP for `mk1`

**Decision.** mk1 is its own BIP draft, not a sub-format or extension of MD's BIP. The two specs cross-reference each other (mk references MD for wallet-ID linkage protocol; MD optionally references mk for the foreign-xpub recovery pattern), but neither is a structural dependency of the other.

**Reasoning.**

- An xpub backup format is useful even outside MD: watch-only wallet provisioning, master-public-key archival, key-rotation continuity. Coupling mk to MD's release cadence is artificial.
- Different audiences and timelines: MD's "wallet descriptor" framing has a specific multisig/foreign-xpub problem statement; key backup is a more general concern.
- Cleaner evolution: mk's wire-format decisions (compact xpub representation, origin-path embedding) are governed by their own concerns. Tying them to md's lifecycle means every md revision risks pulling in mk-irrelevant constraints, and vice versa.
- Existing precedent: BIP 93 stayed scoped to seed shares; it didn't try to be "BIP 32 + bech32 + recovery." mk staying scoped to xpub backup is consistent.

### D-9. HRP `mk`

**Decision.** mk1 uses HRP `mk` — alphabetical extension of the `ms`/`md` codex32-family namespace.

**Reasoning.** Two-letter HRP, mnemonic ("mnemonic key"), no known collisions with bech32/Lightning/codex32 HRPs to date. Anyone reaching for `mk` in the codex32-derived namespace is most plausibly thinking exactly what we are.

**Verification gate before formal registration.** Search SLIP-0173 (informal segwit-HRP registry) and recent bitcoin-dev mailing-list archives + BIPs PR history for any soft `mk` claim. None expected, but should be confirmed before publishing a draft BIP.

**Alternatives considered (if collision found later):** `mx` (mnemonic xpub, unambiguous via the `x`), `mkc` (mnemonic key card), `mpk` (mnemonic public key). Pick if `mk` is contested.

### D-10. BCH plumbing: reuse BIP 93's polynomial; new NUMS-derived target constants

**Decision.** mk1 reuses BIP 93's BCH generator polynomials verbatim (same as md1). Domain separation between mk1, md1, and codex32 is provided by:

1. Different per-format target residue constants (`MK_REGULAR_CONST`, `MK_LONG_CONST`), NUMS-derived from a fresh domain-separation string (e.g. `b"shibbolethnumskey"`; exact string TBD).
2. HRP-mixing in the polymod (BIP 173-style HRP expansion).

**Reasoning.**

- BCH polynomials are not cryptographic secrets; they're chosen for minimum-distance and weight-distribution properties. Sharing them across formats does not weaken either format's error-correction guarantees.
- Computing a fresh polynomial from scratch is CPU-hours of search and would require independent analysis to claim equivalent guarantees. No actual security gain.
- Precedent: bech32 itself uses one polynomial across `bc`/`tb`/`bcrt`, with HRP-mixing providing the domain separation. BIP 350 explicitly designed for this pattern.
- md-codec already does exactly this: reuses BIP 93's generator polynomials, defines `MD_REGULAR_CONST` and `MD_LONG_CONST` as NUMS-derived constants from `SHA-256(b"shibbolethnums")`.

**Caveat.** mk1's NUMS string MUST be independent from md1's `"shibbolethnums"`. Random-looking 65-bit/75-bit residues from independent SHA-256 inputs almost never have meaningful structural relationships, so this is a low-risk gate. But the discipline of "independent domain string per format" is the rule.

### D-11. Defensive registration strategy

**Decision.** Soft claim `mk1` HRP in this repo's design docs immediately. Defer formal BIP submission until both md and mk are mature and implementation-tested.

**Reasoning.**

- MD's own BIP is still pre-Draft, awaiting human review. Stacking a second BIP draft before the first is reviewed risks spreading reviewer attention thin.
- mk1's design is still in flux. A formal claim that later changes shape erodes credibility more than the defensive value gained.
- The realistic risk isn't "someone races us to `mk`" — it's "someone doesn't notice we're using it and picks `mk` for an orthogonal purpose." Soft-claiming via a public GitHub repo with `bip/bip-mnemonic-key.mediawiki` and `design/mk/` is enough lead time for that risk profile.

**Coordination note.** Andrew Poelstra (codex32 author) is the natural reviewer for mk1's BCH-plumbing reuse story. Loop him in before formal BIP submission to avoid "you missed a structural concern" rework.

### D-12. Repo layout: same repo as MD; sibling crate; sibling design folder

**Decision.** mk-codec lives as a sibling crate inside `descriptor-mnemonic/`, alongside md-codec and md-signer-compat. The mk BIP draft lives in the existing `bip/` folder. mk's design discussion lives in `design/mk/`.

**Reasoning.** Pre-1.0, design-stage, dense cross-references between md and mk specs, shared CI infrastructure. Splitting to a separate repo is mechanical when warranted and not warranted yet.

### D-13. Plumbing-reuse strategy: fork now, refactor later

**Decision.** mk-codec initially **forks** the BCH primitives from md-codec (option 3 from the architecture-question discussion). Once both formats stabilize, refactor the shared codex32-derived plumbing into a third workspace member (e.g. `crates/mc-codex32/`).

**Reasoning.** Premature shared-crate extraction is its own footgun. The plumbing is small enough (~200 lines) that fork-then-merge is cheaper than design-the-shared-API-up-front. Both formats need to be implementation-validated before the shared API can be designed correctly.

**Eventual split commitment.** The split *will* happen — agreed in this session — but is deliberately deferred for efficiency during the design phase.

## Open questions

These are flagged as still-under-discussion or to-be-decided-when-spec-drafting:

| ID | Question | Notes |
|---|---|---|
| Q-1 | Exact NUMS domain string for mk1's target constants | Suggested `b"shibbolethnumskey"`; not locked. Must be independent from md's `"shibbolethnums"`. |
| Q-2 | Wallet-ID stub format on mk1 cards: 4-byte chunk-header stub or 16-byte full wallet ID? | md1 uses 4 bytes in chunk header for chunking sanity; full 16-byte ID is a separate optional anchor. Decide which form mk1 carries. |
| Q-3 | Path-component cap exact number | Suggested 32 (no real wallet uses >6). Lock when the wire-format spec is drafted. |
| Q-4 | Per-`@N` path tag byte allocation in MD bytecode | New tag in the unallocated 0x36+ range, or backfill 0x24–0x32. Has implications for the v0.X version bump (wire-format change in MD). |
| Q-5 | mk1 chunk-type byte allocation | Independent of md1's allocation. Probably `SingleString = 0x00`, `Chunked = 0x01`, mirroring md1. |
| Q-6 | mk1 payload layout: byte order of `[fingerprint, origin_path, xpub_bytes, wallet_id_stubs]` | Order doesn't affect correctness but affects parser ergonomics. Decide when wire-format spec is drafted. |
| Q-7 | mk1 xpub encoding: full 78-byte Base58Check decoded, or compact ~65-byte (chain code + 33-byte pubkey, dropping serialization framing)? | Smaller is better for engraving but requires reconstructing the BIP 32 serialization metadata at decode time. |
| Q-8 | Privacy framing: how does mk1's xpub-revelation compare to MD's existing fingerprints-leak warning? | mk1 reveals more than just fingerprints. Spec needs an analogous privacy section. |
| Q-9 | When the eventual md/mk split into a shared `mc-codex32` crate happens | Triggered by what milestone? Probably "when both formats are implementation-validated and we can identify a stable shared API." |
| Q-10 | mk1 version-string anchor (analogous to md's `GENERATOR_FAMILY = "md-codec X.Y"`) | Pin the family-stable token convention for mk's vector files. |

## Conversation provenance

Decisions D-1 through D-13 were reached interactively in a Claude Code session on 2026-04-29. The author drove the decisions; the assistant surfaced tradeoffs, recommended defaults, and captured the conclusions. No agent-review checkpoints were run during this design phase — those land when the SPEC and BIP draft are written.
