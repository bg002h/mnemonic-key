# Mnemonic Key (MK)

A specification for backing up bitcoin extended public keys (xpubs) on durable media (paper, steel) in a form that is compact, hand-transcribable, and strongly error-correcting. Designed to engrave alongside a [Mnemonic Descriptor (MD)](https://github.com/bg002h/descriptor-mnemonic) policy card for foreign-xpub multisig recovery.

> **Status: v0.1 reference implementation shipped.**
> The wire format is locked per the closure design at
> [`docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`](docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md);
> [`crates/mk-codec`](crates/mk-codec/) provides a working
> `KeyCard` ↔ `Vec<String>` round-trip with BCH error correction and
> a canonical 39-vector conformance corpus (17 clean + 22 negative,
> one negative per `Error` variant). Pre-BIP-submission audit
> items remain — see [`design/FOLLOWUPS.md`](design/FOLLOWUPS.md).

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
│   └── bip-mnemonic-key.mediawiki     ← BIP draft (content-complete; pre-submission audit)
├── crates/
│   └── mk-codec/                       ← Rust reference implementation (v0.1)
├── design/
│   ├── DECISIONS.md                    ← rolling design-decisions log (D-1..D-15, Q-1..Q-10 closed)
│   ├── FOLLOWUPS.md                    ← deferred items + pre-bip-submission audit gates
│   └── SPEC_mk_v0_1.md                 ← wire-format spec (post-closure)
├── CHANGELOG.md
├── LICENSE
└── README.md
```

## Where to start reading

- **For format users / implementers**: [`bip/bip-mnemonic-key.mediawiki`](bip/bip-mnemonic-key.mediawiki) is the canonical (draft) spec.
- **For the reference implementation**: [`crates/mk-codec/`](crates/mk-codec/) — Rust crate, v0.1.1 working encode/decode round-trip + 39-vector conformance corpus (17 clean + 22 negative) at [`crates/mk-codec/src/test_vectors/v0.1.json`](crates/mk-codec/src/test_vectors/v0.1.json).
- **For why the design is the way it is**: [`design/DECISIONS.md`](design/DECISIONS.md) walks through D-1..D-15 (the 2026-04-29 design discussion plus closures of Q-1..Q-10). [`design/SPEC_mk_v0_1.md`](design/SPEC_mk_v0_1.md) is the post-closure wire-format spec.
- **For deferred work**: [`design/FOLLOWUPS.md`](design/FOLLOWUPS.md) — pre-BIP-submission audit gates and cross-repo coordination items.

## Man pages

`mk` ships man pages generated from its own clap definition — the same source as `--help` — so they cannot drift from the binary. Three ways to install them:

1. **Automatic (default).** The [constellation installer](https://github.com/bg002h/mnemonic-toolkit) installs them alongside the binary into `~/.local/share/man/man1` — no sudo, no system files:

   ```sh
   sh -c "$(curl -fsSL https://raw.githubusercontent.com/bg002h/mnemonic-toolkit/master/scripts/install.sh)"
   ```

   Then `man mk` works (and `man mk-<subcommand>` for each subcommand). Pass `--no-man` to skip, or `--man-dir <dir>` to relocate.

2. **By hand.** If you installed the binary directly (`cargo install`), emit them yourself:

   ```sh
   mk gen-man --out ~/.local/share/man/man1
   ```

3. **Download.** Each release attaches a `mk-man.tar.gz` asset — extract it into your manpath.

If `man mk` can't find them (older `man-db`, or macOS/BSD `man` that doesn't auto-read `~/.local/share/man`): `man -M ~/.local/share/man mk`.

## What's covered in v0.1

- **Foreign-xpub multisig recovery**: each cosigner backs up their xpub on its own MK card. Recovery: assemble policy card (MD) + cosigner key cards (MK) → reconstruct full descriptor → verify Wallet Instance ID matches.
- **Per-card metadata**: optional BIP 32 origin fingerprint (closure Q-8 privacy-preserving mode), derivation path (standard-table indicator OR explicit-path escape hatch with a 10-component cap), 73-byte compact xpub (closure Q-7), ≥1 Policy ID stubs identifying which MD-encoded wallets the xpub serves.
- **Privacy framing**: an MK card alone enables full transaction-history reconstruction; physical security parity with seed backups is recommended (MUST NOT be photographed).

## What's NOT in scope

- **Extended private keys (xprv)**: backing up secret material is BIP 93's job.
- **MuSig2 aggregate keys (BIP 327)**: future milestone, possibly its own format.
- **Embedded wallet-policy fragments**: the policy lives on the MD card; MK is key-only.

## License

The specification text in this repository and the reference implementation in `crates/mk-codec/` are dual-licensed, at your option, under either the [MIT License](LICENSE) or the [Unlicense](UNLICENSE) public-domain dedication — SPDX `MIT OR Unlicense`.

## Contact

bg002h · `bcg@pm.me`

## Related work

- [Mnemonic Descriptor (MD)](https://github.com/bg002h/descriptor-mnemonic) — sibling repo for the wallet-policy-template format.
- [BIP 93 — codex32](https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki) — Andrew Poelstra. Provides the BCH plumbing MK reuses.
- [BIP 32 — Hierarchical Deterministic Wallets](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) — defines the xpub serialization MK encodes.
- [BIP 380 — Output Descriptors](https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki) — origin notation `[fp/path]xpub`.
- [BIP 388 — Wallet Policies](https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki) — the placeholder-template framing MD encodes.
