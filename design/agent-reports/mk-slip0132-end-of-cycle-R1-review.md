# mk SLIP-0132 (A2) — End-of-Cycle R1 Review

**Cycle:** mk SLIP-0132 acceptance (A2)
**Branch:** `mk-slip0132-acceptance` (toolkit repo)
**HEAD at review time:** `e572888`
**Reviewer:** Claude Sonnet 4.6
**Date:** 2026-06-01
**Scope:** C1 fold verification only — R0 was GREEN except C1 + 2 non-gating Minors

---

## Verdict: ✅ GREEN — C1 resolved, no drift, cycle ready for ship-authorization

---

## C1 Verification: stale `mk-cli-v0.6.0` prose pin in `44-mk-cli.md`

### Check 1 — Line 12 now reads v0.7.0

```
docs/manual/src/40-cli-reference/44-mk-cli.md:12:
`cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.7.0 --bin mk`.
```

CONFIRMED. `grep -rn 'mk-cli-v0.6' docs/manual/src/` → empty. No stale v0.6 pin anywhere in manual src.

### Check 2 — v0.7.0 consistent with all authoritative pin sites

All four canonical sites consistently pin `mk-cli-v0.7.0`:

- `scripts/install.sh:41` → `mk-cli-v0.7.0` ✓
- `.github/workflows/manual.yml:77` → `mk-cli-v0.7.0` ✓
- `.github/workflows/quickstart.yml:71` → `mk-cli-v0.7.0` ✓
- `docs/manual/src/40-cli-reference/44-mk-cli.md:12` → `mk-cli-v0.7.0` ✓ (the fixed line)

mk-cli `Cargo.toml` version in mnemonic-key repo: `version = "0.7.0"` ✓

md/ms prose pins: `grep -rnE '(descriptor-mnemonic-md-cli|ms-cli)-v[0-9]' docs/manual/src/` → only the single mk result above; no md or ms prose install commands exist in manual src (R0 noted "only the mk line existed" — confirmed).

### Check 3 — Companion FOLLOWUP filed

`design/FOLLOWUPS.md` entry `sibling-pin-check-skips-manual-prose-install-commands` present at line 3418:

- Status: `open`
- Tier: `ci-hardening`
- Surfaced: 2026-06-01, A2 end-of-cycle review (C1)
- Documents the gate gap accurately: `sibling-pin-check.yml` scans only `.github/workflows/*.yml`, blind to `docs/manual/src/**` prose install commands; drifted across TWO prior cycles undetected.
- Options for remediation documented (extend workflow scan OR add manual lint).

CONFIRMED.

### Check 4 — No drift in e572888

`git show e572888 --stat` touches exactly:

```
design/FOLLOWUPS.md            | 11 +++++++++++
docs/manual/src/40-cli-reference/44-mk-cli.md |  2 +-
2 files changed, 12 insertions(+), 1 deletion(-)
```

Exactly the expected scope: 1 prose line change + FOLLOWUP entry. No code, no version bumps, no other prose.

The B1 commit (`36b08a4`) touches: `manual.yml`, `quickstart.yml`, `CHANGELOG.md`, `Cargo.lock`, `README.md`, `Cargo.toml` (version bump), `crates/mnemonic-toolkit/README.md`, `44-mk-cli.md` (SLIP-0132 section additions), `scripts/install.sh`. That is the expected A2 implementation scope — the C1 fold (`e572888`) adds nothing beyond the two files above.

### Check 5 — Lint neutrality

The change is a single version string substitution (`v0.6.0` → `v0.7.0`) in prose. The prior `make -C docs/manual lint` run was exit 0. Version strings are lint-neutral (not a heading, anchor, or link). No re-run required.

---

## R0 Minors — Disposition

### M1: `slip0132.rs:22` comment mislabel ("BIP-49 multisig" for YpubMultisig / 0295B43F, should be BIP-48)

Pre-existing toolkit code — confirmed NOT introduced by the A2 branch. Verification:

- `git log 4d5ef56..e572888 -- crates/mnemonic-toolkit/src/slip0132.rs` → empty (zero A2-branch commits touch `slip0132.rs`)
- `git show 4d5ef56:crates/mnemonic-toolkit/src/slip0132.rs` at A2 branch divergence point already has the identical mislabel at line 22 and line 13 doc comment

This is a pre-existing cosmetic inaccuracy in a doc comment (the const value `0295B43F` is correct). Out of this cycle's scope — confirmed.

### M2: mk FOLLOWUP paraphrase (cosmetic)

Non-gating cosmetic wording concern in a FOLLOWUP entry. No action required for ship authorization.

---

## Summary

All six R1 checks pass:

1. ✅ `44-mk-cli.md:12` reads `mk-cli-v0.7.0`
2. ✅ `grep -rn 'mk-cli-v0.6' docs/manual/src/` → empty; all 4 pin sites at v0.7.0; mk-cli Cargo.toml = 0.7.0
3. ✅ FOLLOWUP `sibling-pin-check-skips-manual-prose-install-commands` filed, status open, ci-hardening
4. ✅ `e572888` touches only `44-mk-cli.md` (1 line) + `FOLLOWUPS.md` (11 lines) — zero drift
5. ✅ Lint neutral (version string in prose; prior exit 0 holds)
6. ✅ `slip0132.rs:22` mislabel is pre-existing (zero A2-branch commits modify the file); M2 cosmetic

**Cycle is GREEN. Ship-authorized.**
