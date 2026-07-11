# R0 convergence review — `SPEC_test_hardening_T4_mk_external_oracle.md` (round 2) — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Verified against mk HEAD `582f007`. Scoped to the I1/I2/I3/M1-M4 folds (10 BIP vectors NOT re-verified — round 1 confirmed all mechanically). **I1 verification went beyond code-trace: built `mk` at HEAD and drove the full T4-a mechanism end-to-end with the real published BIP-84 zpub + BIP-86 xpub.**

## I1 (zpub ingest via real path) — CLOSED, proven executable
**(a) `mk encode --xpub` exists + normalizes zpub — load-bearing PASS.** `cmd/encode.rs:19-20` (`--xpub`), `:94` `parse_xpub_normalized`; `cmd/mod.rs:150` `slip132::detect_and_normalize`; `slip132.rs:140` `[0x04,0xB2,0x47,0x46]=>(XPUB_MAINNET,Zpub)`, decode-swap-reencode `:149-152`, `path_matches` requires hardened `84'` (`:69`) → `m/84'/0'/0'` passes. Precedent `cli_slip132.rs:91`. `Xpub::from_str` rejects zpub (`vendor/bitcoin/src/bip32.rs:794-800`, bitcoin 0.32.8).
**(b) Encoder guard accepts both origins — CONFIRMED by execution** (`bytecode/encode.rs:38-48`; both encodes exit 0).
**(c) bc1q/bc1p rendering — CONFIRMED, addresses match VERBATIM:**
```
mk encode --xpub zpub6rFR7y4… --origin-path m/84'/0'/0' | mk address - --count 2 --chain both
  /0/0 bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu ✓  /0/1 bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g ✓  /1/0 bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el ✓
mk encode --xpub xpub6BgBgse… --origin-path m/86'/0'/0' | mk address - --count 2 --chain both
  /0/0 bc1p5cyxnux…qkedrcr ✓  /0/1 bc1p4qhjn9z… ✓  /1/0 bc1p3qkhfews… ✓
```
The mechanism is demonstrated working at HEAD, not merely plausible.

## I2 (provenance framing) — CLOSED
SPEC:25/27/39 now state the mutation DOES RED cli_address's frozen literals today + reframe T4-a's value as wrong-at-birth/shared-lineage provenance. Verified: `cli_address.rs:25-27` frozen consts; `account_84_first_address_matches_toolkit:88-92`; arm swap `derive_support.rs:106-111` / builder `render_address:129-136` → `3…`≠frozen`bc1q…` REDs today. False "not caught today" gone.

## I3 (compact-form assertion + compiling mutation) — CLOSED
(a) SPEC:32 asserts at compact-form level via `XpubCompact::from_xpub` (`pub` `xpub_compact.rs:45`, re-exported `bytecode/mod.rs:31`, `pub mod bytecode` `lib.rs:37` → reachable as `mk_codec::bytecode::XpubCompact::from_xpub`), with the coordinated-swap-cancellation rationale. (b) Mutation = `version`↔`parent_fingerprint`, both `[u8;4]` (`xpub_compact.rs:34,36`; sources in `from_xpub:45-53` both `[u8;4]`) → COMPILES + REDs (tv1 version `0488B21E`≠parent fp `00000000`). (c) Honesty caveat present (SHA-pinned corpus `V0_1_SHA256` + eager version validation `xpub_compact.rs:126`).

## M1-M4 — ALL CLOSED
M1 `slip132.rs:182` + FOLLOWUPS `:410`; M2 assert_cmd not lib (mk-cli bin-only, `assert_cmd` dev-dep, `cli_address.rs:56-64` idiom); M3 mk-codec has `sha2`+`hex` dev-deps (no bs58, zero Cargo.lock delta); M4 T4-c home + `Some(Zpub)` outcome (proven by the live run).

## Critical: none. Important: none.
## Minor (informational, non-blocking)
**M-r2-1:** the account cards chunk into **2 mk1 strings** (observed live) — the implementer must capture ALL emitted lines (the `cli_address.rs:56-64` idiom passes a `Vec<String>` via `.args(cardstrs)`; `mk address` accepts multiple positionals/stdin), not `lines().next()`.
**M-r2-2:** no `mlock.rs` in mk → `cargo fmt --all --check` safe.

## New-gap scan: none (folds left the 10 vectors + acceptance #2/#3/#4 substance untouched; T4-b input side + T4-c unaffected/strengthened).

## VERDICT: GREEN (0C / 0I)
All three round-1 Importants closed with live-source-accurate text; the load-bearing I1 mechanism executed against HEAD produced the published BIP-84/86 addresses character-for-character. Two informational Minors only. T4 implementation may begin.

---
**SHIP-READY (opus, 2026-07-10):** T4 GREEN. Implementer dispatched (mk; T4-a via `mk encode --xpub` capturing ALL chunk lines per M-r2-1; T4-b compact-form oracle; T4-c optional). Post-impl R0 → ship T4 (mk direct-FF, NO-BUMP).