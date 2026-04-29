# Mnemonic Key (MK)

A specification for backing up bitcoin extended public keys (xpubs) on durable media (paper, steel) in a form that is compact, hand-transcribable, and strongly error-correcting. Designed to engrave alongside a [Mnemonic Descriptor (MD)](https://github.com/bg002h/descriptor-mnemonic) policy card for foreign-xpub multisig recovery.

> **Status: design-stage skeleton, no implementation.**
> The wire-format spec, BIP draft, and reference-implementation crate
> scaffold are committed. Encoding/decoding logic is not yet written;
> all public functions in the reference crate panic with `todo!()`.
> Expect the API surface and wire format to change up to the first
> implementation milestone.

MK is the third format in a triad of codex32-derived bitcoin backup formats:

| Format | HRP | Purpose | Spec |
|---|---|---|---|
| Mnemonic Seed share | `ms1` | Seed shares | [BIP 93](https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki) (Andrew Poelstra) |
| Mnemonic Descriptor | `md1` | Wallet-policy template | [`bg002h/descriptor-mnemonic`](https://github.com/bg002h/descriptor-mnemonic) |
| Mnemonic Key | `mk1` | Extended public keys | this repo |

All three reuse BIP 93's BCH plumbing (same generator polynomials, same alphabet) with HRP-mixing and per-format target residues providing domain separation.

## Why MK?

[MD](https://github.com/bg002h/descriptor-mnemonic) encodes the BIP 388 wallet-policy template (the `@N`-placeholder shape), but not the xpubs that map placeholders to keys. For wallets where the user holds every seed, MD's omission is fine — the seeds reconstruct the xpubs. For **foreign-xpub multisig** (the user is one of N cosigners and doesn't hold all seeds), MD alone is insufficient: the missing xpubs must come from somewhere at recovery time.

MK fills that gap with separate per-cosigner cards. Each cosigner backs up their own xpub on its own card, atomically, and may stamp one or more "Policy ID" stubs declaring which MD-encoded wallets the xpub serves. Cosigners can hand their MK card to a wallet creator without revealing other cosigners' xpubs.

## Repository contents

```
.
├── bip/
│   └── bip-mnemonic-key.mediawiki     ← BIP draft skeleton
├── crates/
│   └── mk-codec/                       ← Rust reference-implementation skeleton
├── design/
│   ├── DECISIONS.md                    ← rolling design-decisions log
│   └── SPEC_mk_v0_1.md                 ← wire-format sketch (provisional)
├── LICENSE
└── README.md
```

## Where to start reading

- **For format users / implementers**: [`bip/bip-mnemonic-key.mediawiki`](bip/bip-mnemonic-key.mediawiki) is the canonical (draft) spec.
- **For the reference implementation**: [`crates/mk-codec/`](crates/mk-codec/) — Rust crate, currently scaffolded only.
- **For why the design is the way it is**: [`design/DECISIONS.md`](design/DECISIONS.md) walks through 13 design decisions reached during the 2026-04-29 design discussion. [`design/SPEC_mk_v0_1.md`](design/SPEC_mk_v0_1.md) sketches the wire format with provisional answers to 10 still-open questions.

## What's covered in v0.1

- **Foreign-xpub multisig recovery**: each cosigner backs up their xpub on its own MK card. Recovery: assemble policy card (MD) + cosigner key cards (MK) → reconstruct full descriptor → verify Wallet Instance ID matches.
- **Per-card metadata**: BIP 32 origin fingerprint, derivation path (standard-table indicator OR explicit-path escape hatch), full 78-byte xpub, ≥1 Policy ID stubs identifying which MD-encoded wallets the xpub serves.
- **Privacy framing**: an MK card alone enables full transaction-history reconstruction; physical security parity with seed backups is recommended (MUST NOT be photographed).

## What's NOT in scope

- **Extended private keys (xprv)**: backing up secret material is BIP 93's job.
- **MuSig2 aggregate keys (BIP 327)**: future milestone, possibly its own format.
- **Embedded wallet-policy fragments**: the policy lives on the MD card; MK is key-only.

## License

The specification text in this repository is dedicated to the public domain under [CC0-1.0](LICENSE). The reference implementation in `crates/mk-codec/` is released under the same CC0-1.0 license.

## Contact

bg002h · `bcg@pm.me`

## Related work

- [Mnemonic Descriptor (MD)](https://github.com/bg002h/descriptor-mnemonic) — sibling repo for the wallet-policy-template format.
- [BIP 93 — codex32](https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki) — Andrew Poelstra. Provides the BCH plumbing MK reuses.
- [BIP 32 — Hierarchical Deterministic Wallets](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) — defines the xpub serialization MK encodes.
- [BIP 380 — Output Descriptors](https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki) — origin notation `[fp/path]xpub`.
- [BIP 388 — Wallet Policies](https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki) — the placeholder-template framing MD encodes.
