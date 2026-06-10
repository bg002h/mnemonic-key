# R0 Architecture Review — PLAN_stub_formula_walletpolicyid.md — Round 2 (R1)

**Reviewer:** Fable 5 (feature-dev:code-architect)
**Date:** 2026-06-10
**Plan doc:** `design/PLAN_stub_formula_walletpolicyid.md` (R0-r1 fold)
**Prior review:** `design/agent-reports/stub-formula-plan-r0-round1-review.md`

---

## VERDICT: GREEN — 0 Critical, 0 Important, 0 Minor

---

## Round-1 finding resolutions

### I1 — De-tautologized test still tautological — **RESOLVED**

Plan §3 now contains all three load-bearing constraints:
(a) frozen `const EXPECTED_STUB: [u8; 4] = [...]` literal, NOT a `let` bound to `compute_wallet_policy_id(...)`.
(b) "The test body MUST NOT call `compute_wallet_policy_id` (nor `encode_payload`/`sha256`) at runtime. Any such call reintroduces the tautology the fix exists to kill."
(c) documented external-computation method + "run a one-off `dbg!` or a scratch `#[test]` that prints the value, paste it, then delete the scratch." The prohibition is no longer vague.

### I2 — Missing encode.rs §3.5.1 cites — **RESOLVED**

Plan §2 lists all four: `mod.rs:56`, `round_trip.rs:45`, `encode.rs:3`, `encode.rs:34` → §3.3.

### m1 — copy_from_slice — **RESOLVED**

§2 snippet uses `stub.copy_from_slice(&id.as_bytes()[..4]);` mirroring synthesize.rs.

### m3 — BIP sites hedge — **RESOLVED**

§1 states "(CONFIRMED present, R0-m3)" and enumerates the three stale sites; "if present" hedge removed.

### m4 — CHANGELOG path + version — **RESOLVED**

§4 names `crates/mk-cli/CHANGELOG.md` (root is the codec's, untouched) and the `0.7.0 → 0.8.0` bump.

---

## Fold-introduced drift check

- **Parameter name:** impl snippet uses `md1_str`; source `crates/mk-cli/src/cmd/mod.rs:57` confirms `pub fn derive_stub_from_md1(md1_str: &str) -> Result<[u8; 4]>`. Matches.
- **Version consistency:** header grounding mk-codec `main` 3882823; §4 says 0.7.0 → 0.8.0; Cargo.toml:3 is 0.7.0. No contradiction.
- **Compilability:** `decode_md1_string(md1_str)?` + `compute_wallet_policy_id(&descriptor)?` both present/re-exported at pinned md-codec-v0.34.0; `?` uses confirmed `From<md_codec::Error> for CliError` (error.rs:153-156); `copy_from_slice` on `&[u8;16][..4]` well-typed/infallible. Plan calls out dropping unused `sha256`/`encode_payload` imports + `cargo build` warning-clean step.
- **No section contradicts another.** §3 test constraints consistent with §2 impl; §2 four-site repoint consistent with §3 handling round_trip.rs:45; §4 CHANGELOG consistent with SemVer MINOR.

---

## Summary

All 2 Important + 4 Minor from round 1 resolved. No new Critical/Important introduced. Plan internally consistent; impl snippet syntactically plausible with confirmed parameter names; test de-tautologization constraints unambiguous. **Implementation may proceed.**
