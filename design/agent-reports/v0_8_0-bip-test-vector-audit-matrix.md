# v0.8.0 BIP test vector audit matrix — mnemonic-key (mk-codec) [no-scope companion]

Built 2026-05-13 per the v0.8.0 cross-repo audit cycle.
**Predecessor (still authoritative for substantive coverage):**
[`v0_7_1-bip-test-vector-audit-matrix.md`](v0_7_1-bip-test-vector-audit-matrix.md)
(marked SUPERSEDED at v0.8.0 in lockstep with this file).

**Cycle SPEC:** `mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md`.
**Cycle plan:** `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`.
**Survey precursor:** `mnemonic-toolkit/design/agent-reports/v0_8_0-cross-repo-bip-vector-survey.md`.

## §0 Cycle disposition

**No scope for mk-codec at v0.8.0.** The cross-repo BIP-vector
adoption survey surfaced three high-ROI gaps (BIP-341 → md-codec,
BIP-93 full corpus → ms-codec, BIP-39 Trezor English fill →
mnemonic-toolkit) plus an opportunistic BIP-85 v85.3 fold; none
fall on mk-codec's surface. mk-codec continues to delegate BIP-32
derivation to `bitcoin v0.32` and carry the only relevant
coverage in the v0.7.1 matrix unchanged.

This file exists for cross-repo audit symmetry per SPEC §5.

## §1 Coverage delta vs v0.7.1

| BIP / SLIP | v0.7.1 status | v0.8.0 status | Delta |
|---|---|---|---|
| All entries in v0.7.1 matrix | (carry forward unchanged) | (carry forward unchanged) | 0 |

## §2 Sibling-repo coverage (cycle context)

For the cross-repo coverage table including this cycle's
substantive deltas in md-codec, ms-codec, and mnemonic-toolkit,
see the toolkit's v0.8.0 matrix
(`mnemonic-toolkit/design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md`).
