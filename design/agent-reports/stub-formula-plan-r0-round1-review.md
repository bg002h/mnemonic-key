# R0 Architecture Review — PLAN_stub_formula_walletpolicyid.md — Round 1

**Reviewer:** Fable 5 (feature-dev:code-architect)
**Date:** 2026-06-10
**Plan doc:** `design/PLAN_stub_formula_walletpolicyid.md`
**Architect consult (prior):** `design/agent-reports/stub-formula-divergence-architect-consult.md`

---

## VERDICT: YELLOW — 0 Critical, 2 Important, 4 Minor

---

## Critical

None.

---

## Important

### I1 — The de-tautologized test is still tautological as written; the plan's "hardcode a literal" instruction is correct in intent but incomplete in specification

**Claim (plan §3):** "compute the expected stub ONCE via `compute_wallet_policy_id(...).as_bytes()[..4]`, hardcode it as a 4-byte literal `EXPECTED_STUB`."

**Problem:** The plan does not specify HOW the implementer is supposed to obtain the literal value to hardcode, and the only path the plan suggests — "decode PKH_BASIC_MD1, compute the expected stub via `compute_wallet_policy_id`" — is a computation the implementer will perform ONCE to get the literal. If the implementer writes an ephemeral helper that calls `compute_wallet_policy_id` to obtain the bytes and then hard-codes them, the literal is correctly independent. But the plan never says "run this code ephemerally to get the bytes, then paste them; do NOT call `compute_wallet_policy_id` at test runtime." An implementer following the plan literally could instead write `EXPECTED_STUB = compute_wallet_policy_id(decode(PKH_BASIC_MD1)).as_bytes()[..4].try_into().unwrap()` as a `const` or `lazy_static`, which would re-tautologize the test at runtime.

The existing test (round_trip.rs:44-78) is tautological because `expected_stub` is computed at test runtime via the same chain as the implementation (mod.rs:59-61 === round_trip.rs:47-49 — literally identical three lines). The fix the plan proposes is correct in principle: a hardcoded `[u8; 4]` literal that was computed once externally. But the plan should explicitly prohibit any runtime call to `compute_wallet_policy_id` inside the test body. The plan currently says "hardcode it as a 4-byte literal `EXPECTED_STUB` (mirroring identity.rs:546-553's hardcoded-WalletPolicyId-golden pattern)" — the identity.rs:546-553 pattern is actually the right model (it hardcodes `expected_id: [u8; 16]` computed by an external `/tmp/golden_vec.py` script), but the plan does not tell the implementer to use an equivalent external computation step. Without explicit wording, an implementer might call `compute_wallet_policy_id` inline in the test and believe they have followed the plan.

**Fix:** Plan §3 must add: (a) the hardcoded literal must be a `const [u8; 4]` with a comment documenting how it was computed (e.g., "computed externally via a one-off call to `md_codec::compute_wallet_policy_id(md_codec::decode_md1_string(PKH_BASIC_MD1).unwrap()).unwrap().as_bytes()[..4]`"); (b) the test body MUST NOT call `compute_wallet_policy_id` at runtime — any call to that function in the test body reintroduces tautology. The plan must make this explicit.

---

### I2 — encode.rs has TWO additional phantom `§3.5.1` citations the plan does not list

**Claim (plan §2 + architect consult):** "The phantom §3.5.1 cite appears only in mod.rs:56 + round_trip.rs:45 doc-comments; repoint both to §3.3."

**Evidence against:** The grep confirms TWO additional `§3.5.1` occurrences the plan misses:

- `crates/mk-cli/src/cmd/encode.rs:3` — module-level doc comment: `//! Realizes SPEC §3.5.1 from the v0.2 plan.`
- `crates/mk-cli/src/cmd/encode.rs:34` — field doc comment on `--from-md1` arg: `/// Repeatable. Each value is an md1 string; the stub is derived per SPEC §3.5.1.`

Both cite the phantom heading. The plan lists only `mod.rs:56` and `round_trip.rs:45`. These two missed encode.rs sites will ship stale citations if the plan's repoint scope is followed literally.

**Fix:** Plan §2 must add `encode.rs:3` and `encode.rs:34` to the repoint list (→ §3.3). There are now four `§3.5.1` sites to repoint, not two.

---

## Minor

### m1 — `as_bytes()[..4].try_into()` vs `copy_from_slice` — style divergence from existing toolkit pattern (no semantic consequence)

**Plan §2 writes:** `Ok(id.as_bytes()[..4].try_into().expect("4-byte slice"))`

The toolkit pattern at synthesize.rs:158-159, :191, :244, :424, :596 consistently uses:
```rust
let mut stub = [0u8; 4];
stub.copy_from_slice(&policy_id.as_bytes()[..4]);
```

The plan's `try_into().expect(...)` is semantically correct (both extract the same 4 bytes; the `try_into` from `&[u8]` to `[u8;4]` is infallible when length is 4, which it is — `as_bytes()` returns `&[u8; 16]`, so `[..4]` is definitely length 4). This is minor polish, not a correctness issue.

**Suggestion:** Adopt `copy_from_slice` for pattern consistency, or at least note in the plan that either form is acceptable and why.

---

### m2 — §5 step 1 SPEC rewrite does not correct the Wallet Instance ID formula in step 4 (unchanged, but worth noting it is also stale)

**Plan §1 rewrites:** SPEC_mk_v0_1.md:312 (§5 step 1) — the Policy ID formula.

**Observation:** SPEC §5 step 4 (around line 318) reads:
```
wallet_instance_id = SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]
```
This also uses `canonical_bytecode` phrasing that implicitly references the encoding-sensitive hash, not the WalletPolicyId. However, the Wallet Instance ID is a SEPARATE concept (per md-codec v0.8.0 naming note in the BIP glossary) and the plan correctly treats step 4 as "stays unchanged." This is correct scope — step 4's formula is the Wallet Instance ID (not the Policy ID stub), computed at recovery time differently. No change needed; just noting that the plan's "§5 step 4's separate Wallet Instance ID stays" is correct.

No action required — this is a confirmatory note.

---

### m3 — BIP draft `bip/bip-mnemonic-key.mediawiki` exists and DOES contain the stale formula; the plan's conditional "if present" language understates the certainty

**Evidence:** The file exists at `bip/bip-mnemonic-key.mediawiki`. It contains:

- Glossary line ~37: `'''Policy ID''': the 16-byte hash <code>SHA-256(canonical_bytecode)[0..16]</code>` — uses the encoding-sensitive `canonical_bytecode` framing.
- "Policy ID stubs" section (~line 267): `Each stub is the top 4 bytes (32 bits) of an MD-encoded policy card's <code>SHA-256(canonical_bytecode)</code>.`
- "Linkage to MD and recovery flow" section (~line 403): `Decode the MD policy card. Compute its full 16-byte Policy ID = <code>SHA-256(canonical_bytecode)[0..16]</code>.`

All three sites use `SHA-256(canonical_bytecode)` which is the `Md1EncodingId` (encoding-sensitive) formula, not `WalletPolicyId`. The plan §1 says "BIP draft (`bip/bip-mnemonic-key.mediawiki` if present) ... same rewrite in lockstep." The conditional "if present" language obscures that this file exists, is present, and has three stale sites. The plan should state this explicitly.

**Suggestion:** Remove the "if present" hedge and enumerate the three BIP sites directly in plan §1, matching the same specificity as the SPEC rewrite bullets.

---

### m4 — CHANGELOG location: two CHANGELOG files exist; plan is silent on which to update

**Evidence:** Both `CHANGELOG.md` and `crates/mk-cli/CHANGELOG.md` exist. The crates/mk-cli one is the active release log (most recent entry [0.6.0] — 2026-05-30). The plan §4 says "CHANGELOG note" without specifying the path. The implementer must update `crates/mk-cli/CHANGELOG.md` with a new `[0.8.0]` entry (following `[0.7.0]` current version per Cargo.toml:3).

**Suggestion:** Plan §4 should say "add `[0.8.0]` entry to `crates/mk-cli/CHANGELOG.md`" (current version is 0.7.0; MINOR bump → 0.8.0).

---

## Confirmed-correct claims (explicit sign-off)

1. **`derive_stub_from_md1` current body (mod.rs:57-63):** Confirmed. Lines 57-63 are the bytecode-hash path exactly as the plan describes: `decode_md1_string` → `encode_payload` → `sha256::Hash::hash` → `[..4].try_into()`.
2. **`compute_wallet_policy_id` exists and is `pub`:** Confirmed. `identity.rs:172` declares `pub fn compute_wallet_policy_id(d: &Descriptor) -> Result<WalletPolicyId, Error>`. Re-exported at `md-codec/src/lib.rs:51`.
3. **Return type and `.as_bytes()` method:** Confirmed. `WalletPolicyId` at identity.rs:117-132 implements `pub fn as_bytes(&self) -> &[u8; 16]`. The plan's `id.as_bytes()[..4]` is well-typed; the `[..4]` slice on a `&[u8; 16]` is a valid `&[u8]` of length 4 → `.try_into().expect(...)` cannot panic.
4. **`compute_wallet_policy_id` returns `Result<WalletPolicyId, md_codec::Error>`:** Confirmed (identity.rs:172). The `?` operator invokes `From<md_codec::Error> for CliError` confirmed at error.rs:153-156. Plan's "Map any md_codec::Error through the existing CliError path" via `?` is correct and compiles.
5. **Slice consistency — toolkit uses identical `[..4]` on same `WalletPolicyId::as_bytes()`:** Confirmed at synthesize.rs lines 159, 191, 244, 424, 596: all use `stub.copy_from_slice(&policy_id.as_bytes()[..4])`. Byte-parity exact.
6. **Two callers (`encode.rs:69`, `verify.rs:107`) only consume `[u8; 4]`:** Confirmed. Both push the result into a `Vec<[u8;4]>` / compare against `card.policy_id_stubs`. Neither depends on bytecode-hash semantics. Unchanged callers correct.
7. **No other mk-cli source replicates the bytecode-hash formula:** Confirmed. Only `encode_payload`/`sha256` usage in `crates/mk-cli/src/` is the three lines in `cmd/mod.rs`.
8. **Phantom `§3.5.1` in SPEC:** Confirmed absent in SPEC_mk_v0_1.md. Sections are §3.3 (line 184/186), §5 step 1 (line 312), §9 Q-2 (line 385).
9. **SPEC lines say what the plan claims:** Confirmed. :186 top-32-bits-of-SHA-256(canonical_bytecode); :312 full 16-byte Policy ID = SHA-256(canonical_bytecode)[0..16]; :385 Q-2 row.
10. **md-codec pinned at 0.34.0 in mk-cli Cargo.toml:** Confirmed `crates/mk-cli/Cargo.toml:24`: `md-codec = "0.34.0"`. Function reachable at that pin.
11. **SemVer MINOR:** Confirmed. Current version 0.7.0 (Cargo.toml:3). Behavior change to `--from-md1` output → MINOR (0.7.0 → 0.8.0).
12. **No goldens move:** `test_vectors/v0.1.json` stubs are arbitrary literals, not formula-derived. Scope to `round_trip.rs::from_md1_derivation` only is correct.

---

## Summary

Two Important: I1 (test must explicitly forbid runtime `compute_wallet_policy_id` call + obtain literal externally) and I2 (encode.rs:3 + encode.rs:34 also cite §3.5.1 → 4 repoint sites, not 2). Both fixable by amending plan text — no architectural change. Fold I1+I2 (and the minors), then re-dispatch for R1.
