# HRP `mk` collision audit

**Status:** complete
**Audit type:** pre-BIP-submission gate, FOLLOWUPS item `hrp-mk-collision-check`
**Date:** 2026-04-29
**Auditor:** v0.1.x worktree maintenance pass

## Goal

Verify the `mk` HRP chosen for the Mnemonic Key format does not collide with any registered Bitcoin-family bech32 HRP, and that the BIP 173 HRP-mixing design provides adequate cross-HRP-misvalidation protection in the realistic threat model.

## Method

1. Fetched the canonical Bitcoin-family HRP registry, [SLIP-0173](https://github.com/satoshilabs/slips/blob/master/slip-0173.md), at the maintainer copy. Enumerated every registered HRP across all listed chains.
2. Filtered the registry for any HRP equal to `mk` (case-sensitive — bech32 mandates lowercase HRPs).
3. Filtered for 2-character HRPs starting with `m`, since these are visually closest to `mk` and thus the highest-risk neighbours under hand-transcription.
4. Cross-checked against codex32 (BIP 93) and Mnemonic Descriptor (`md`) — the two sibling formats sharing `mk1`'s codex32-derived plumbing.
5. Considered the BIP 173 HRP-mixing argument for cross-HRP-typed-as-other false-positive validation.

## Findings

### F-1. `mk` is not registered in SLIP-0173

`mk` does not appear in the registry as of the audit date. The only 2-character HRPs starting with `m` are:

| HRP | Chain | Variant |
|-----|-------|---------|
| `my` | Myriad | mainnet |
| `tm` | Myriad | testnet |
| `mm` | Miden | mainnet |

None equal `mk`. The HRP space is sparse enough that adding `mk` does not crowd any existing allocation.

### F-2. Visually-similar registered HRPs

Within the codex32-and-friends family, the closest HRPs to `mk` are:

| HRP | Format | Hamming distance from `mk` |
|-----|--------|----------------------------|
| `ms` | BIP 93 codex32 master seed | 1 (k → s) |
| `md` | Mnemonic Descriptor (sibling format) | 1 (k → d) |
| `mm` | Miden | 1 (k → m) |
| `my` | Myriad | 1 (k → y) |

All four are within Hamming distance 1 of `mk`. The relevant question is whether a one-character HRP-typing mistake produces a string that validates against the wrong format's checksum.

### F-3. BIP 173 HRP-mixing prevents cross-HRP false-positive validation

[BIP 173 §3](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki) defines HRP-expansion as:

```text
hrp_expand(hrp) := [c >> 5 for c in hrp] + [0] + [c & 31 for c in hrp]
```

The polymod runs over `hrp_expand(hrp) || data || checksum`, so the HRP bytes contribute deterministically to the residue. A different HRP produces a different polymod input and thus a different valid-codeword set.

For the `mk` ↔ `ms` neighbour pair (the highest-risk pair, since both are codex32-derived):

```text
hrp_expand("mk") = [3, 3, 0, 13, 11]
hrp_expand("ms") = [3, 3, 0, 13, 19]
```

The two preludes differ in their final symbol (11 vs 19). For a string typed under HRP `mk` but parsed as HRP `ms`, the polymod input differs at position 5 (the last HRP byte's low 5 bits). This is a single-symbol shift in the polymod input. The probability that two random valid-codewords agree under both HRPs at random data is approximately `2^-65` for the regular code (since the BCH code has 65-bit residue space and HRP-mixing applies before residue computation) — vanishingly small.

Concretely: if a user transcribes `mk1xyz...` but a verifier validates against HRP `ms`, the polymod produces a residue different from `MS32_CONST` (BIP 93's target) with probability ≈ 1 - 2^-65. Verification fails. The string is rejected, not silently misinterpreted. The same argument holds for `mk` ↔ `md`, `mk` ↔ `mm`, and `mk` ↔ `my`.

### F-4. Cross-format target-residue separation

`mk` further protects itself by using NUMS-derived target residues (`MK_REGULAR_CONST = 0x1062435f91072fa5c`, `MK_LONG_CONST = 0x41890d7e441cbe97273`) drawn from `SHA-256(b"shibbolethnumskey")`, distinct from md1's `MD_REGULAR_CONST` / `MD_LONG_CONST` (drawn from `SHA-256(b"shibbolethnums")`) and BIP 93's `MS32_CONST`. Even an HRP-collision-prone construction would still need to also collide on target residue to silently misvalidate; the NUMS construction makes this combinatorially implausible.

### F-5. Visual / hand-transcription disambiguation

Bech32's alphabet exclusion of `b` and `i` (BIP 173 §1) is intended to remove the most-confusable letters. `m` and `k` are both in the alphabet and are not visually adjacent (`m` resembles `n`, `k` resembles `h` or `x` in some fonts). The HRP `mk` is well-formed under bech32's visual-distinguishability discipline. None of the 1-Hamming-neighbour HRPs (`ms`, `md`, `mm`, `my`) introduce a plausible visual-confusion pair.

## Conclusion

`mk` is a safe HRP choice for the Mnemonic Key format:

1. **Unregistered:** no SLIP-0173 collision.
2. **Cross-HRP false-positive validation:** prevented by BIP 173 HRP-mixing, with theoretical false-positive probability ≈ 2^-65 for a `mk` ↔ neighbour mistype.
3. **Cross-format target-residue separation:** NUMS-derived constants are independent across `mk` / `md` / codex32, so even an HRP-collision-prone construction would still need a constant collision to silently misvalidate.
4. **Bech32 visual discipline:** the HRP is well-formed; the Hamming-distance-1 neighbours are not visually confusable.

## SLIP-0173 registration (filed)

Following md1's defensive-filing pattern (`md` registered via [PR #2011](https://github.com/satoshilabs/slips/pull/2011) on 2026-04-28), the `mk` HRP was filed for SLIP-0173 registration as **[PR #2012](https://github.com/satoshilabs/slips/pull/2012)** on 2026-04-29. The PR adds one row to `slip-0173.md` between `Lightning Network` and `Zcash`. Merge state is tracked externally on SatoshiLabs's review cadence; the mk1-side gate is cleared.

If md1's PR #2011 merges first, mk1's PR #2012 will need a one-line rebase to insert `mk` after `md` rather than after `Lightning Network`. Otherwise the two PRs are mergeable in either order.

## Audit closure

This document closes FOLLOWUPS item `hrp-mk-collision-check` (pre-bip-submission tier). The SLIP-0173 registration follow-on is closed at FOLLOWUPS item `slip-0173-register-mk-hrp` with the PR URL pinned.
