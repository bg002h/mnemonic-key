# SLIP-0173 PR draft — register HRP `mk` for Mnemonic Key

This document holds the draft text for the SLIP-0173 PR registering the
`mk` HRP. Mirrors [md-codec's filing](https://github.com/satoshilabs/slips/pull/2011)
(SatoshiLabs PR #2011, opened 2026-04-28) which registers `md` for the
sibling Mnemonic Descriptor format.

## Filing instructions (for the maintainer)

1. Fork [satoshilabs/slips](https://github.com/satoshilabs/slips) under your account.
2. Apply the diff below to `slip-0173.md`. The new row goes immediately
   after the `Mnemonic Descriptor` row that md1's PR adds (alphabetical
   order — "Mnemonic Descriptor" < "Mnemonic Key").
   - If md1's PR (#2011) has merged by the time you file: insert after the
     existing `md` row.
   - If md1's PR hasn't merged yet: rebase your PR onto md1's branch (or
     wait for md1 to merge first to keep the PRs sequential and reviewable).
3. Commit message: `slip-0173: register HRP mk for Mnemonic Key (BIP 32 xpub backup)`.
4. Open the PR against `satoshilabs/slips:master` with the body text below.
5. Once filed, update `design/FOLLOWUPS.md::slip-0173-register-mk-hrp`
   with the PR number; close as `resolved <date> — PR filed at <url>;
   merge state tracked externally`.

## PR title

```
slip-0173: register HRP mk for Mnemonic Key (BIP 32 xpub backup)
```

## PR body

```markdown
Registers the HRP `mk` for the Mnemonic Key (MK) format — a steel-engravable backup format for individual BIP 32 extended public keys (xpubs). Spec + reference implementation at https://github.com/bg002h/mnemonic-key.

**Format scope**: not a cryptocurrency network. MK encodes individual xpubs into Bech32-style strings prefixed `mk1...` for engraving on steel backup plates, designed to engrave alongside the sibling Mnemonic Descriptor (MD, HRP `md`) policy card for foreign-xpub multisig recovery. The engraved card carries one xpub plus declarative metadata (Policy ID stubs, optional master fingerprint, derivation path); keys remain in BIP 39 seed words.

**HRP collision vet performed prior to selection** (2026-04-29, recorded in [`design/AUDIT_hrp_mk_collision.md`](https://github.com/bg002h/mnemonic-key/blob/main/design/AUDIT_hrp_mk_collision.md)):
- SLIP-0173 main coin table — clean (closest 2-char HRPs are `mm` Miden, `my` Myriad; closest 1-Hamming-distance neighbours are `ms` codex32 BIP 93, `md` Mnemonic Descriptor, `mm`, `my`)
- Codex32 BIP 93 (`ms`) — distinct from `mk`; cross-HRP false-positive validation prevented by BIP 173 HRP-mixing (≈ 2⁻⁶⁵ collision probability per cross-HRP mistype)
- Mnemonic Descriptor (`md`) — sibling format from the same project; deliberate 1-Hamming-distance pair to surface the family relationship without sharing the HRP
- Lightning Network HRPs (`lnbc`, `lntb`, `lnbcrt`, `lnsb`, `lno`, `lni`, `lnr`) — distinct
- Liquid sidechain (`ex`, `lq`, `el`, `tlq`, `ert`) — distinct
- Nostr NIP-19 (`npub`, `nsec`, `note`, etc.) — distinct
- Cosmos chain HRPs — distinct

**Cross-format separation** beyond HRP-mixing: `mk` and `md` use independent NUMS-derived target residues drawn from different domain strings (`shibbolethnumskey` and `shibbolethnums` respectively), so even an HRP-collision-prone construction would still need a constant collision to silently misvalidate.

**Status**: BIP draft is currently *Pre-Draft, AI + reference implementation, awaiting human review*. Latest release: [`mk-codec-v0.1.1`](https://github.com/bg002h/mnemonic-key/releases/tag/mk-codec-v0.1.1).

**Companion filing**: PR #2011 registers `md` for the sibling Mnemonic Descriptor format. Both filings share the same author, project, and design lineage.

Filing this defensive registration to close off future collision risk from independent projects.
```

## Diff for `slip-0173.md`

Insert one row after md1's `Mnemonic Descriptor` row (or, if md1's PR
hasn't merged, after `Lightning Network` directly, leaving md1's PR to
add its row independently in any merge order):

```diff
 | Lightning Network | `ln[currency prefix + amount]` |
 | Mnemonic Descriptor | `md`                         |                            |                               |
+| Mnemonic Key      | `mk`                           |                            |                               |
 | Zcash             | `zs`                           | `ztestsapling`             | `zregtestsapling`             |
```

(Column alignment matches md1's PR — single mainnet HRP, no testnet or
regtest columns, since mk1's testnet handling lives in the bytecode
xpub.version field and not in the HRP. Same convention md1 uses.)

## Sequencing note

The HRP collision audit is at
[`design/AUDIT_hrp_mk_collision.md`](AUDIT_hrp_mk_collision.md). The
audit's "Pending: SLIP-0173 registration" section directly motivates
this PR. Once the PR is filed, update both that audit document (replace
"Pending" with the PR URL) and the `slip-0173-register-mk-hrp`
FOLLOWUPS entry to reflect the filing.
