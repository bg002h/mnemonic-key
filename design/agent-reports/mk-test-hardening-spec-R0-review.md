# R0 ARCHITECT REVIEW — `SPEC_mk_codec_test_hardening.md`

Opus feature-dev:code-reviewer. Mandatory pre-impl gate. Reviewed against live source at `mnemonic-key` `crates/mk-codec` (origin/main `d9d2ed9`) + the toolkit consumer. Persisted (folds applied immediately after; see end-of-file fold log).

## Verification summary (claims that CHECK OUT)
- `KeyCard` derives `PartialEq, Eq` (`key_card.rs:23`); `Xpub` derives PartialEq/Eq (bitcoin 0.32) — P1 `==` sound.
- API exact: `lib.rs:51` `decode(&[&str]) -> Result<KeyCard>`, `encode_with_chunk_set_id(&KeyCard,u32)`, `encode`. No `decode_string`/`reassemble` at crate root (`reassemble_from_chunks` takes `Vec<ChunkFragment>`, `chunk.rs:109`).
- Stub cap 1..=255 (`encode.rs:22,25`); path cap 1..=10 (`path.rs:114`,`consts.rs:27`); 14-entry dict (`path.rs:38-55`).
- Both codes t=4: `bch.rs:376,451`; `bch_decode.rs:566` rejects deg>4; `k=L-1-d` `:587`. No surviving >4 expectation.
- 4-error correction reachable via public `decode()`. Cross-chunk hash 4 bytes (`consts.rs:45`,`chunk.rs:67-70,189-201`) → 2⁻³² residual; T2c `!= Ok(original)` justified.
- getrandom panic path (`pipeline.rs:45-49`); csid 20-bit (`header.rs:27`). std crate; no clippy disallowed-methods.
- Fail-closed decode primitives (`bch.rs:78-101`, `header.rs:120-176`) → P2 panic-freedom + T3a `is_err()` reachable.
- T3b: `bch_code_for_length` (`bch.rs:117-124`) None for 94..=95 → `InvalidStringLength` (`bch.rs:669`). Deterministic.
- T4 255-stub ≈1100 bytes < `MAX_CHUNKABLE_BYTECODE=1692` (`chunk.rs:21`) → ~21 chunks. 256 → `InvalidPolicyIdStubCount`.
- `error.rs:56` "(4 for regular, 8 for long)" exists + is misleading as §4 T2-doc states.

## CRITICAL
None. No specified test is unbuildable, intrinsically flaky, or false-failing; no load-bearing source claim wrong.

## IMPORTANT
**I1 — §3 strategy mis-describes `synthetic_xpub`; conflates `Xpriv::new_master` derivation with the direct-construction precedent.** `test_helpers.rs:22-40` builds `Xpub` by DIRECT struct construction (fixed `SecretKey::from_slice(&[1u8;32])`, fixed parent_fp/chain_code, `network: Main`, depth/child from path) — NOT `new_master`/derive. Spec says "seed → Xpriv::new_master → derive ... mirrors synthetic_xpub" — two different constructions presented as one; an implementer can't satisfy both. Not Critical (both round-trip), but wrong about the precedent + ambiguous. Fix: pick one — recommended direct construction with depth/child from path, optionally strategy-varied pubkey/chaincode (`SecretKey::from_slice` + prop_filter for the ~2⁻¹²⁸ invalid case). Drop the false "mirrors synthetic_xpub"/new_master framing.

**I2 — Consumer citation path wrong: `mnemonic-toolkit/src/repair.rs` → `crates/mnemonic-toolkit/src/repair.rs`.** §1/§5/§9/§10 drop the `crates/` prefix (`src/repair.rs` doesn't exist; there's a decoy `cmd/repair.rs`). `Mk1IndelOracle` confirmed at `:1001`; comment spans `:997-1000` (spec said 997-998). Navigation defect in a module-doc cross-cite that ships into the test file. Fix all four cites + comment span.

## MINOR
**M1** — §10 cites `pipeline.rs:288` as "the guard"; line 288 is test-comment prose. Actual guard `chunk.rs:189-201`; 288 is inside the existing 5-burst test (`:271-348`). Relabel.
**M2** — §7 local gate omits `-- -D warnings`; CI runs `clippy --workspace --all-targets -- -D warnings` (`.github/workflows/ci.yml:58`). A proptest harness can pass locally yet fail CI. Add `-D warnings` to §7.
**M3** — State the T3a/T3b (deterministic→is_err/variant-pin) vs T2c (randomized→`!= Ok(original)`) distinction explicitly in §5 so a maintainer doesn't randomize T3a into a 2⁻³² flake.
**M4** — Typical card chunks are BOTH regular-code (14..=93). To exercise long-code `BCH(108,93,8)` (the "both code variants" T2a/T2b goal), the fixture must reach a 96..=108 data-part band — size deliberately (reuse the 255-stub card). Flag for Phase 1.
**M5** — `.gitignore`: proptest writes nested `crates/mk-codec/tests/proptest-regressions/`; a root `proptest-regressions/` line won't match. Use `**/proptest-regressions/` (mirrors the repo's `**/target/`).

## VERDICT: RED (0C / 2I / 5M)
Substantively sound — every t=4 / API / band / hash / assertion claim verified true. RED solely on I1 (strategy construction ambiguity) + I2 (consumer-citation path). Fold I1+I2, sweep the 5 Minors (esp. M4 — affects whether T2a/T2b meet "both code variants" — and M5 — affects whether the gitignore line works), re-dispatch for R1.

---
## FOLD LOG (post-R0, this cycle)
- I1 → §3 rewritten to precedent-faithful direct `Xpub` construction (depth/child from path; strategy-varied pubkey/chaincode w/ prop_filter; network from the network axis, not hardcoded Main). new_master/"mirrors synthetic_xpub" contradiction removed.
- I2 → all cites corrected to `crates/mnemonic-toolkit/src/repair.rs:1001` + comment `:997-1000` (§1, §5, §10).
- M1 → §10 relabels `pipeline.rs:288` as the 5-burst test comment; guard = `chunk.rs:189-201`.
- M2 → §7 gate adds `-- -D warnings` (+ CI cite).
- M3 → §5 adds the deterministic-vs-randomized assertion-rationale paragraph.
- M4 → §4 T2a adds the long-band (96..=108) fixture-sizing requirement (reuse the 255-stub card); run T2a/T2b against both a regular- and a long-band chunk.
- M5 → §3 gitignore line → `**/proptest-regressions/`.
