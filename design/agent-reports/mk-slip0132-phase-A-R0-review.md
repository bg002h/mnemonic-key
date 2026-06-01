# Phase A (mk-cli) per-phase R0 review — SLIP-0132 acceptance (A2)

**Reviewer:** opus architect (adversarial, read-only build/test/clippy).
**Target:** repo `mnemonic-key`, branch `mk-slip0132-acceptance`, diff `fc2341b..ac76f2d`
(A1 `95118e8`, A2 `1772fca`, A3 `24ba2c7`, release `ac76f2d`).
**Toolkit primitive checked against:** `mnemonic-toolkit/crates/mnemonic-toolkit/src/slip0132.rs` (live).
**Date:** 2026-06-01.

## Verdict: GREEN (0C/0I)

3 Minors, none gating. The phase is correct, in-scope, and ships-clean. Byte-parity with the
CI-tested toolkit table verified arm-by-arm; normalization is sound; the predicate is correct
(incl. hardened + short-path); encode/verify wiring is right; exit codes are correct (64 vs 4);
the SLIP-0132 note + watch-only advisory coexist with the right ordering; 73/73 mk-cli tests
green; clippy `-D warnings` clean; 248/248 whole-workspace tests green; mk-codec untouched.

## Critical

None.

## Important

None.

## Minor (finding — file:line — why — fix)

- **M1 — drift-guard test is self-referential, not anchored to an external literal.**
  `crates/mk-cli/src/slip132.rs:118-149` (`slip132_version_bytes_match_slip0132`). The test
  declares its own 8-entry `[u8;4]` array, builds a SLIP-0132 string from each, and asserts
  `detect_and_normalize` returns the matching variant. Because the test array and the production
  `match` arms are independent copies, the test catches a *one-sided* edit or a *transposition* of
  production bytes, but a *coordinated* edit (same wrong byte in both the test array and the
  production arm) passes green while silently mis-normalizing a key. The plan (A2 Step 1(a)) framed
  this as "assert each match-arm byte-array equals the SLIP-0132 literal"; the implementation
  asserts the round-trip instead. Mitigating facts: (a) I byte-verified all 8 arms against the
  toolkit's CI-tested, spec-vector-anchored table below — they are correct *now*; (b) mk-cli is
  upstream of the toolkit and cannot dep it, so a compile-time shared constant isn't available.
  Not gating because correctness is established at ship and the realistic future-drift mode
  (editing one side) IS caught. **Fix (optional, future):** add an independent assertion that
  pins each production arm against a hard-coded SLIP-0132 spec literal that is NOT reused to build
  the probe string (e.g. assert the `swap`/`variant` produced for a string built from a separately
  declared literal), or pin one published spec vector (BIP-49 ypub / BIP-84 zpub) round-trip the
  way the toolkit does in `slip0132_spec_bitcoin_test_vector_*`.

- **M2 — shared note text is encode-centric ("the engraved card's script type…") but also fires on
  `verify`.** `crates/mk-cli/src/cmd/mod.rs:127`. `verify` does not engrave; the phrase reads
  slightly off in that context. Defensible (the card under verification does have a script type
  derived from its origin path) and harmless, but a generic phrasing ("the card's script type
  derives from the origin path") would read correctly for both callers. **Fix (optional):** drop
  "engraved".

- **M3 — testnet single-sig mismatch help suggests a mainnet prefix.**
  `crates/mk-cli/src/slip132.rs:76-77`. `Vpub`/`Upub` reuse the mainnet `Zpub`/`Ypub` advice
  strings ("supply the ypub for a 49' path", "supply the zpub/xpub…"), so a testnet `vpub`
  mismatch points at a mainnet `ypub`. The `label()` already names "testnet", and this is a rare
  path (testnet + script-type/path disagreement), so it is cosmetic. **Fix (optional):** branch the
  testnet arms to name `upub`/`vpub`.

## Verification

### Table byte-parity (all 8 arms vs `mnemonic-toolkit/src/slip0132.rs:82-90`) — PASS
| Variant | mk-cli `slip132.rs` ver bytes | swap target | toolkit ver bytes | toolkit neutral | match |
|---|---|---|---|---|---|
| Ypub (ypub) | `04 9D 7C B2` | `04 88 B2 1E` xpub | `04 9D 7C B2` | xpub | ✓ |
| Zpub (zpub) | `04 B2 47 46` | `04 88 B2 1E` xpub | `04 B2 47 46` | xpub | ✓ |
| YpubMultisig (Ypub) | `02 95 B4 3F` | `04 88 B2 1E` xpub | `02 95 B4 3F` | xpub | ✓ |
| ZpubMultisig (Zpub) | `02 AA 7E D3` | `04 88 B2 1E` xpub | `02 AA 7E D3` | xpub | ✓ |
| Upub (upub) | `04 4A 52 62` | `04 35 87 CF` tpub | `04 4A 52 62` | tpub | ✓ |
| Vpub (vpub) | `04 5F 1C F6` | `04 35 87 CF` tpub | `04 5F 1C F6` | tpub | ✓ |
| UpubMultisig (Upub) | `02 42 89 EF` | `04 35 87 CF` tpub | `02 42 89 EF` | tpub | ✓ |
| VpubMultisig (Vpub) | `02 57 54 83` | `04 35 87 CF` tpub | `02 57 54 83` | tpub | ✓ |

`XPUB_MAINNET = 04 88 B2 1E` and `TPUB_TESTNET = 04 35 87 CF` (slip132.rs:14-15) equal the
toolkit's `SWAP_TO_XPUB_MAINNET` / `SWAP_TO_TPUB_TESTNET` (slip0132.rs:54-56). Network mapping
(4 mainnet→xpub, 4 testnet→tpub) is correct. No byte mis-normalizes a key.

### Normalization soundness — PASS
`detect_and_normalize` (slip132.rs:90-115): `base58::decode_check` → `len() < 4` guard precedes
`try_into().unwrap()` (4 bytes guaranteed; no panic) → match → `copy_from_slice(&swap)` →
`encode_check` → `Xpub::from_str`. Canonical xpub/tpub and unknown versions fall through to
`from_str` returning `(xpub, None)` or the preserved `UsageError`. Garbage/short base58 → `Err`
branch → `from_str(s)` (preserves existing error). No panic on any input. `normalize_zpub_yields_same_key`
proves `public_key`/`chain_code`/`depth`/`child_number`/`parent_fingerprint` are preserved.

### Predicate (`path_matches`) — PASS
slip132.rs:55-64: hardened-only via `matches!(.., ChildNumber::Hardened{index} if *index==idx)`.
49'/84' single-sig (component 0); 48' + component-3 index 1'/2' for multisig. `c.get(3)` returns
`None` on short paths (no panic, no match) — confirmed by the `m/48'/0'/0'` cell. Unhardened
`m/84/0/0` does NOT satisfy `Zpub` (confirmed cell `Zpub must NOT match unhardened m/84/0/0`).
Canonical xpub → `variant=None` → predicate never consulted → never refused.

### Encode + verify wiring — PASS
- **encode** (encode.rs:84-85): `parse_xpub_normalized(&args.xpub, Some(&path))` — path always
  supplied, so mismatch is always checkable. Note + watch-only advisory both emit on the non-JSON
  and JSON paths (advisory at encode.rs:97-100 is unconditional after the if/else).
- **verify** (verify.rs:52-69): `want_path` parsed once via `.as_deref().map(...).transpose()?`
  and reused by both the xpub-normalization check (`want_path.as_ref()`) and the origin-path
  matcher (verify.rs:92). No double-parse. Bare `verify --xpub <zpub>` (no `--origin-path`) →
  `origin_path=None` → normalize + note, no refusal (cell `verify_zpub_without_path_ok` exit 0).
- **mismatch refusal = exit 64** (UsageError; error.rs:85) — distinct from value
  **ContentMismatch = exit 4** (error.rs:84). Both asserted by cells.
- Existing matchers (`origin_fingerprint`, `origin_path`, `policy_id_stub`, `from_md1`) preserved
  verbatim. `parse_xpub` fully deleted; grep shows only `parse_xpub_normalized`. No orphaned
  imports (`Xpub`/`DerivationPath`/`FromStr` all still used in `mod.rs`).

### Coexistence with Phase-2 advisory — PASS
`encode_emits_both_slip132_note_and_watchonly_advisory`: both lines present, `slip_offset <
watch_offset` asserted. SLIP-0132 fires at parse (encode.rs:85) before stdout emit + watch-only
(encode.rs:97). `verify` emits no watch-only advisory (inert) — no conflict.

### Tests + clippy — PASS
`cargo test -p mk-cli`: **73 passed / 0 failed** across all binaries (lib-equivalent bin unit
tests 10 incl. all 5 slip132 units; cli_slip132 8; cli_address 15; gui_schema 8; cli_repair 10;
round_trip 4 incl. `verify_content_mismatch_exits_4`; version_help 3; + others). `cargo test`
(whole workspace): **248 passed / 0 failed** — no mk-codec/cross-crate regression.
`cargo clippy -p mk-cli --all-targets -- -D warnings`: clean (EXIT=0, forced recompile).
Key-material equality uses decode-and-compare-Xpub-fields (sound: `mk encode` is
non-deterministic via random `chunk_set_id`, so raw mk1-string comparison would be wrong; the
test compares decoded `card.xpub`). Stderr-ordering cell uses the em-dash `\u{2014}` matching the
Phase-2 advisory literal.

### Scope — PASS
9 files: `slip132.rs` (new), `main.rs` (mod decl), `cmd/mod.rs` (helper + `parse_xpub`
deletion), `cmd/encode.rs` + `cmd/verify.rs` (wiring), `tests/cli_slip132.rs` (new),
`Cargo.toml` (0.7.0), `Cargo.lock`, `design/FOLLOWUPS.md`. mk-codec UNTOUCHED. No GUI/manual
files (correct — Phase A is mk-cli only; lockstep is Phase B). No flag/subcommand surface change
→ no gui-schema-mirror lockstep required (gui_schema tests green).

### SemVer — PASS
0.6.1 → 0.7.0 MINOR. Additive: inputs previously rejected (`Xpub::from_str` fails on SLIP-0132
prefixes) are now accepted/normalized; no removal or breaking change to existing behavior.
FOLLOWUP `mk-slip0132-prefix-acceptance` filed `resolved 24ba2c7`; commits `95118e8 + 1772fca +
24ba2c7` match the log; format consistent with neighboring entries; mk-only (no sibling companion).

## Notes

- The base58 layer re-checksums on `encode_check`, so a normalized string is independently valid;
  `Xpub::from_str` re-validates structure. The swap touches only the 4 version bytes.
- Phase B (toolkit re-pin `mk-cli-v0.6.1 → v0.7.0` at 3 sibling-pin sites + manual prose +
  toolkit PATCH) is out of scope for this per-phase gate and remains gated on the `mk-cli-v0.7.0`
  tag existing on the remote (plan §Phase B note). Nothing in Phase A blocks it.
- The three Minors are quality polish only; none affects correctness, exit contracts, or
  byte-fidelity. Recommend filing M1 (strengthen the drift guard with an externally-anchored
  literal or a published-vector round-trip) as a low-priority test-hardening FOLLOWUP if not
  folded now; M2/M3 are cosmetic.
