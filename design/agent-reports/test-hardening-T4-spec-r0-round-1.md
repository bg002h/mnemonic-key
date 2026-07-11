# R0 review — `SPEC_test_hardening_T4_mk_external_oracle.md` (round 1) — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Verified against mk `582f007a` (=HEAD) + authoritative BIP text fetched 2026-07-10 from `raw.githubusercontent.com/bitcoin/bips/master/bip-00{84,86,32}.mediawiki`.

## Probe 1 (load-bearing): BIP-vector re-verification — PASS
All ten pinned strings verified MECHANICALLY (fixed-string grep of each SPEC constant against the fetched BIP files; each matched exactly once): BIP-84 account zpub (bip-0084:75), /0/0 `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` (:80), /0/1 (:85), /1/0 (:90); BIP-86 account xpub (bip-0086:92), /0/0 `bc1p5cyxnux…` (:100), /0/1 (:108), /1/0 (:116); BIP-32 tv1 seed (:220) + master xpub (:222). Oracle constants correct; path labels + mnemonic match. **No wrong-oracle risk.**

## Critical: none.

## Important
**I1 — T4-a's parse mechanism fails on the BIP-84 vector.** BIP-84 publishes the account key only as a **zpub**; `bitcoin::bip32::Xpub::from_str` (rust-bitcoin 0.32.8, `Cargo.lock:140-142`) rejects it (`Error::UnknownVersion`, `vendor/bitcoin/src/bip32.rs:794-800` accepts only xpub/tpub). Also mk-cli is BIN-ONLY (`[[bin]] mk`, no `[lib]`) → the test can't import `slip132::detect_and_normalize`. Fix: (a) inline version-byte swap via `bitcoin::base58::decode_check`/`encode_check` (the `to_slip132` idiom, `slip132.rs:168-172`), fail-closed; or (b) **build through the real ingest path `mk encode --xpub <published zpub>`** (SLIP-132 normalization wired `cmd/mod.rs:150`, exercised by `cli_slip132.rs`) — (b) end-to-end stronger + makes T4-c nearly free. BIP-86 needs nothing (canonical xpub).

**I2 — T4-a's "NOT independently caught" differential claim is false as an executable protocol.** All three named mutations ARE caught today by `cli_address.rs`'s frozen literals (they don't re-derive at test time): 84'↔49' arm swap → `m/84'/0'/0'` infers P2shP2wpkh → `3…` ≠ frozen `bc1q…` → `account_84_first_address_matches_toolkit` (:88-92) REDs; builder/86' swaps likewise. The genuine gap is **wrong-at-birth shared-lineage provenance** + the "re-sync drifted literals from sibling" repair failure. Reword acceptance #1 to that provenance framing (recon §3's nuance stated it; the SPEC dropped it).

**I3 — T4-b's named RED mutation does not compile + the assertion target is blind.** (a) `chain_code=[u8;32]`, `public_key=[u8;33]` (`xpub_compact.rs:38-40`) → swapping their sources in `from_xpub`(:49-51) is E0308; the mirrored `reconstruct_xpub` swap fails too. (b) Asserting the RECONSTRUCTED `{…}` (SPEC:32) is blind to a *coordinated* swap — the two mutations cancel, reconstruction = identity → stays GREEN against exactly the target class. Fix: assert at the **compact-form level** (`XpubCompact::from_xpub` fields, public + re-exported `bytecode/mod.rs:18-31`) + name a **compiling** mutation (`version`↔`parent_fingerprint`, both `[u8;4]`). Carry the honesty caveat: the SHA-pinned corpus (`tests/vectors.rs`, `V0_1_SHA256`) + decode's eager version validation (`xpub_compact.rs:126`) already catch most wire-visible variants; T4-b's value = spec-external provenance + unit-level locality.

## Minor
**M1** — `slip132_version_bytes_match_slip0132` at `slip132.rs:182` (not ~145); FOLLOWUPS entry `:410`.
**M2** — SPEC:26 "(or the library render path)" infeasible (no lib target) — drive `mk` via `assert_cmd`.
**M3** — mk-codec already has `sha2`+`hex` dev-deps → the double-SHA256 needs no new dep; `bs58` unnecessary → literally zero Cargo.lock delta.
**M4** — T4-c home = `slip132.rs` `#[cfg(test)]` (a src-file test edit, still NO-BUMP) or `tests/cli_slip132.rs`; assert `detect_and_normalize(<published zpub>)` = `Some(Zpub)` + field-equality vs the version-swapped parse.

## Confirmed sound
T4-a construction (`KeyCard::new` + `mk_codec::encode` needs no seed; encoder guard `XpubOriginPathMismatch` satisfied — both account keys depth-3/child-0' match `m/84'/0'/0'`/`m/86'/0'/0'`); CLI surface (`--count 2 --chain both`; no `--network` — mainnet inferred; depth-3 avoids depth advisory; stderr advisory doesn't pollute stdout); T4-b depth-0 (tv1 master exercises the no-path branch, accepted since v0.4.0); NO-BUMP (test-only, all deps present, no clap-surface change, mk no branch protection; T2 #8 committed `582f007`).

## VERDICT: OPEN (0C / 3I)
Load-bearing deliverable (vector provenance) clean — every constant correct. The three Importants are mechanism/acceptance defects (zpub parse; two mis-stated RED differentials, one naming a non-compiling mutation + tautology-prone target). All fixable with targeted edits; no scope restructuring. Fold + re-dispatch.

---
**FOLD STATUS (opus, 2026-07-10):** I1 folded (T4-a ingests via `mk encode --xpub <zpub>`, real SLIP-132 path; assert_cmd, bin-only). I2 folded (provenance framing — wrong-at-birth; acceptance #1 reworded; the mutation does RED cli_address today, reported as such). I3 folded (assert at compact-form level via `XpubCompact::from_xpub`/`bytecode/mod.rs:18-31`; RED mutation = `version`↔`parent_fingerprint` [u8;4] swap, compiles; honesty caveat added). M1-M4 folded (slip132:182; assert_cmd not lib; sha2+hex no-bs58; T4-c home). Vectors already verified — untouched. Re-dispatch T4 R0.