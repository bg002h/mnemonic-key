# End-of-cycle R0 Review — mk-cli 0.6.0 (`mk address` + `mk derive`)

Reviewer: feature-dev:code-reviewer (opus). Reviewed the full implementation diff `main..HEAD --
crates/mk-cli/` against the GREEN SPEC + plan. Verified `ChildNumber::from_normal_idx` semantics
(bitcoin 0.32) + the mk-codec encode guard against source.

## CRITICAL

**C1 — Panic on user-controlled index ≥ 2³¹ via `--count` / `--range` / `--index` (would-ship-broken).**
`crates/mk-cli/src/cmd/address.rs:99-100`, `crates/mk-cli/src/cmd/derive.rs:49-50`.

`ChildNumber::from_normal_idx(i)` returns `Err` for `i >= 2_147_483_648` (valid normal range
`0..=2^31-1`). All three integer-input routes feed an unvalidated `u32` into
`from_normal_idx(...).unwrap()`:
- `mk address --count 4000000000` → `resolve_indices` yields `0..4000000000`; first `index >= 2^31`
  → panic at `address.rs:100`.
- `mk address --range 2147483648,2147483648` → panic at `address.rs:100`.
- `mk derive --index 2147483648` → panic at `derive.rs:50` (CONFIRMED: `thread 'main' panicked …
  InvalidChildNumber(2147483648)`).

A panic aborts with exit **101** + backtrace, violating the SPEC exit-64 contract for all error
paths and bypassing the `--json` error envelope (`main.rs::emit_error` never runs). The `--path`
route is safe (`DerivationPath::from_str` propagates the range error → `UsageError` at
`derive.rs:101`), making the integer-route panic an inconsistent, clearly-wrong outlier. Plan R0/R1
noted the `from_normal_idx → Result` range but did NOT flag the unguarded `.unwrap()` on user input.

**Fix:** map each user-index `from_normal_idx` to `UsageError` (exit 64); validate the upper bound in
`resolve_indices` before collecting (also prevents an absurd-`--count` allocation). Add regression
tests asserting exit 64 (not 101) for `--index 2147483648`, `--count 2147483649`, and
`--range 2147483648,2147483648`. `chain` (always 0/1) and the literal `0` index may keep `.unwrap()`.

## IMPORTANT
None.

## MINOR (non-blocking)
- **M1** — `--count 0` / `--range A,A` boundary is silent-empty (exit 0, empty output). Well-defined,
  not SPEC-forbidden; optional reject-or-document. (`address.rs:181`)
- **M2** — Depth advisory anchor (`xpub.depth != 3`) is sound: the codec guard `xpub.depth ==
  origin_path.len()` (encode.rs:41) keeps it consistent with the `len()==3` heuristic gate; multisig
  (depth 4) is refused before the advisory. Verified correct, no fix.
- **M3** — `render_address`/network/correctness all check out: test vectors are toolkit-computed (not
  mk-against-mk); four builders match bitcoin 0.32; resolution order (multisig-first, explicit,
  heuristic, require) correct + multisig-refuse not bypassable by `--address-type`; derive
  hardened-reject / index-sugar / ArgGroup / multisig-allowed / fingerprint-fmt conform; JSON shapes +
  integer schema_version + exit-64 routing match SPEC; no private-key surface (`verification_only`);
  release hygiene clean.

## Lockstep note (out-of-scope for this diff, required before/with tagging per SPEC §4)
manual `44-mk-cli.md` (both subcommands + flags + "Six→eight" + install-tag); `mnemonic-gui`
schema-mirror (add address+derive, backfill never-mirrored `repair`, bump pins); toolkit sibling
pins. Outside `crates/mk-cli/` so no code-verdict impact, but the cycle isn't fully shippable until
they land.

**VERDICT: RED (1C/0I)**

---

## Fold applied (controller, confirmed: `mk derive --index 2147483648` panics at derive.rs:50)
- **C1:** `address.rs::resolve_indices` now validates the BIP-32 normal ceiling (`--count` max index
  and `--range` end `< 2^31`) → `UsageError` (exit 64) before collecting; the address derive loop and
  `derive.rs --index` map `from_normal_idx` errors to `UsageError` (defense-in-depth, no `.unwrap()`
  on user input). Regression tests added (count/range/index ≥ 2^31 → exit 64, not 101).
