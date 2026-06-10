# PLAN — mk1 policy_id_stub aligns to WalletPolicyId (audit I1 + I2)

**Status:** R0 GREEN at R1 (2026-06-10, `design/agent-reports/stub-formula-plan-r0-round2-review.md`) — implementation may proceed
**Source grounding:** mk-codec `main` 3882823; toolkit 59c5254; md-codec 8f5a15f (working copy md-codec 0.35.0).
**Resolves:** `design/FOLLOWUPS.md::audit-2026-06-10-backlog` items `stub-formula-divergence` (I1) + `from-md1-test-tautology` (I2).
**Design gate:** architect direction-consult (2026-06-10, persisted `design/agent-reports/stub-formula-divergence-architect-consult.md`) — GREEN decision + verified facts.

## 0. The decision (architect-blessed)

The 4-byte `policy_id_stub` linking an mk1 card to an md1 policy is computed two ways:
- **toolkit** `synthesize.rs` (6 sites): `compute_wallet_policy_id(desc).as_bytes()[..4]` — md v0.13 §5.3 CANONICAL-EXPANDED, encoder-divergence-free hash (encoding-STABLE).
- **mk-cli** `derive_stub_from_md1`: `SHA-256(encode_payload(desc))[..4]` — the md1 BYTECODE hash (= `Md1EncodingId[..4]`, encoding-SENSITIVE). Matches mk SPEC §3.3/§5 (closure Q-2, locked 2026-04-29).

**Canonical = WalletPolicyId (toolkit).** Deciding property: a card-linking stub MUST be stable under re-encoding the same logical wallet (origin/use-site elision, override-vs-baseline path placement). Only WalletPolicyId has it (pinned by md-codec `walletpolicyid_stable_across_origin_elision`/`_use_site_elision`, identity.rs:571-605); the bytecode hash fails it by construction. The mk SPEC Q-2 formula is STALE — it predates the md v0.13 WalletPolicyId feature (`d8ceb90`), which was the only md identity primitive available at the April-2026 mk closure. The toolkit is correct and changes NOTHING.

**Severity LOW:** the toolkit bundle path never calls `mk encode --from-md1` (it mints via `KeyCard::new` with the toolkit-computed stub; `self_check_bundle` recomputes via `compute_wallet_policy_id` — internally consistent). The divergence bites ONLY a user manually running `mk verify/encode --from-md1` against a toolkit md1 (spurious `ContentMismatch`; self-check-rejected cards). No shipped bundle or fielded card is wrong.

**No pin bump:** `compute_wallet_policy_id` is present at the `md-codec-v0.34.0` tag (mk-cli's pin) and byte-identical to 0.35.0 (the only `md-codec/src` delta between the two tags is `chunk.rs`, outside the hash-input path). Optional hygiene, decoupled.

**No goldens move:** mk-codec `test_vectors/v0.1.json` stubs are arbitrary literals, not derived via this formula; the toolkit byte-determinism contract never used the bytecode formula.

## 1. Spec rewrite (the substantive change)

- `SPEC_mk_v0_1.md:186` (§3.3 "Policy ID stub format", closure Q-2): replace "top 32 bits of the MD-encoded policy card's `SHA-256(canonical_bytecode)`" with "the top 4 bytes of `md_codec::compute_wallet_policy_id(descriptor)` (the 16-byte WalletPolicyId — md SPEC v0.13 §5.3 canonical-expanded, encoder-divergence-free preimage). NOT the md1 bytecode hash (`Md1EncodingId`), which is encoding-sensitive and would not survive a re-encode of the same logical wallet."
- `SPEC_mk_v0_1.md:312` (§5 step 1 recovery flow): "full 16-byte Policy ID = `SHA-256(canonical_bytecode)[0..16]`" → "= `WalletPolicyId` (`compute_wallet_policy_id`)". §5 step 4's separate Wallet Instance ID stays.
- §9 closure table Q-2 (line 385): annotate "superseded 2026-06-10 by md-codec v0.13 WalletPolicyId adoption (audit I1); see PLAN_stub_formula_walletpolicyid.md" — preserve closure history, don't silently rewrite a locked closure.
- BIP draft `bip/bip-mnemonic-key.mediawiki` (CONFIRMED present, R0-m3) — rewrite all THREE stale `SHA-256(canonical_bytecode)` sites in lockstep (SPEC+BIP agree pre-submission):
  - Glossary "Policy ID" line (~37): `the 16-byte hash <code>SHA-256(canonical_bytecode)[0..16]</code>` → `the 16-byte WalletPolicyId <code>md_codec::compute_wallet_policy_id(descriptor)</code> (md SPEC v0.13 §5.3 canonical-expanded)`.
  - "Policy ID stubs" section (~267): `the top 4 bytes (32 bits) of an MD-encoded policy card's <code>SHA-256(canonical_bytecode)</code>` → top 4 bytes of the WalletPolicyId.
  - "Linkage to MD and recovery flow" (~403): `Compute its full 16-byte Policy ID = <code>SHA-256(canonical_bytecode)[0..16]</code>` → `= the 16-byte WalletPolicyId (compute_wallet_policy_id)`.

## 2. Implementation (one function)

`crates/mk-cli/src/cmd/mod.rs::derive_stub_from_md1` (:57-63): body →
```rust
let descriptor = md_codec::decode_md1_string(md1_str)?;
let id = md_codec::compute_wallet_policy_id(&descriptor)?;
let mut stub = [0u8; 4];
stub.copy_from_slice(&id.as_bytes()[..4]); // R0-m1: mirror synthesize.rs copy_from_slice pattern
Ok(stub)
```
Error mapping is free — `compute_wallet_policy_id` returns `Result<_, md_codec::Error>` and `From<md_codec::Error> for CliError` already exists (error.rs:153-156; the line-above `decode_md1_string(...)?` already exercises it). Drop the now-unused `sha256` import (`use ... sha256` / `Hash` traits) in mod.rs IFF unused elsewhere — `encode_payload` was only called here, so its path import goes too; verify with `cargo build` warning-clean.

**Repoint ALL FOUR phantom "§3.5.1" doc cites → §3.3 (R0-I2):**
- `mod.rs:56` (the `derive_stub_from_md1` doc-comment)
- `round_trip.rs:45` (handled in §3)
- `encode.rs:3` (`//! Realizes SPEC §3.5.1 from the v0.2 plan.`)
- `encode.rs:34` (`/// ... the stub is derived per SPEC §3.5.1.`)

The two callers (`encode.rs:69`, `verify.rs:107`) call the helper and only consume the `[u8;4]` — unchanged.

## 3. De-tautologize the test (I2)

`crates/mk-cli/tests/round_trip.rs::from_md1_derivation` (:44-78): the oracle recomputes the impl's OWN chain → vacuous. Flip it to an INDEPENDENT, hardcoded golden — mirroring identity.rs:546-553's `expected_id: [u8;16]` hardcoded-golden pattern (which obtains its literal from an external `golden_vec.py`, NOT a runtime recomputation).

**R0-I1 — the load-bearing constraints (a literal that re-derives at runtime is STILL tautological):**
1. The expected value is a **`const EXPECTED_STUB: [u8; 4] = [0x..,0x..,0x..,0x..];`** — a frozen byte literal, NOT a `let` bound to `compute_wallet_policy_id(...)`.
2. **The test body MUST NOT call `compute_wallet_policy_id` (nor `encode_payload`/`sha256`) at runtime.** Any such call reintroduces the tautology the fix exists to kill. The only runtime calls are: invoke `mk encode --from-md1 PKH_BASIC_MD1`, decode the emitted mk1, assert `card.policy_id_stubs[0] == EXPECTED_STUB`.
3. The literal is obtained ONCE, externally, via a throwaway computation, and the method is documented in a comment above the const, e.g.:
   ```rust
   // EXPECTED_STUB computed once, out-of-band, via:
   //   md_codec::compute_wallet_policy_id(
   //       &md_codec::decode_md1_string(PKH_BASIC_MD1).unwrap()
   //   ).unwrap().as_bytes()[..4]
   // Frozen here as a literal so this test catches a future re-divergence
   // of derive_stub_from_md1 instead of recomputing its own chain.
   const EXPECTED_STUB: [u8; 4] = [/* 4 bytes */];
   ```
   To get the bytes for the literal: after the impl change compiles, run a one-off `dbg!` or a scratch `#[test]` that prints the value, paste it, then delete the scratch. (Equivalent to identity.rs's external golden_vec.py step.)
Repoint the §3.5.1 comment (round_trip.rs:45) → §3.3.

## 4. FOLLOWUPS + release

- Promote `stub-formula-divergence` + `from-md1-test-tautology` from the backlog index to their own `### <id>` resolved entries with the CORRECTED rationale (mk SPEC Q-2 was stale, NOT "spec says use WalletPolicyId"). One-line toolkit-side cross-repo note ("mk-cli aligned to toolkit's WalletPolicyId stub; toolkit already correct").
- Behavior change to `--from-md1` output → MINOR bump (R0-m4): `crates/mk-cli/Cargo.toml:3` `0.7.0 → 0.8.0`; add a `[0.8.0]` entry to **`crates/mk-cli/CHANGELOG.md`** (the active release log; the repo-root `CHANGELOG.md` is the codec's, untouched): `mk verify/encode --from-md1` now produces/validates stubs matching toolkit-emitted cards; a stub a user previously stamped via the OLD `--from-md1` no longer matches.
- Lockstep is INTERNAL to mnemonic-key (SPEC §3.3/§5/§9 + BIP + the function + the de-tautologized test in one commit). No cross-repo code lockstep.

## 5. Verification

mk-codec/mk-cli full suite green (the flipped test now passes with the literal golden); a manual cross-check that the emitted stub for a known md1 equals `mnemonic`-toolkit's stub for the same descriptor (if both binaries available) — or at least that the new literal golden was computed via `compute_wallet_policy_id`, not the old chain.
