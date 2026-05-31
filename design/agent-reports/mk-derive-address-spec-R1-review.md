# R1 Re-Review — SPEC_mk_derive_address.md (mk-cli 0.6.0)

Reviewer: feature-dev:code-reviewer (opus). Re-review after the R0 2C/3I/6M fold
(`mk-derive-address-spec-R0-review.md`). Read the folded SPEC against real source @ `ed2596d`,
the mnemonic-gui schema-mirror targets, and the toolkit manual/reference implementations.

## Critical — None.
- **C1 RESOLVED** — all four error conditions route through `CliError::UsageError(String)` → exit 64
  (multisig-refuse §3.1 step1; address-type-required step4; network-mismatch; hardened-from-xpub
  §3.2). Confirmed vs `error.rs:85` (`UsageError(_) => 64`) / `:81-82` (codec → 3/2). Sole "exit 2"
  string is the explanatory "NOT codec-rejection exit 2."
- **C2 RESOLVED** — zero new variants (grep of the four R0-proposed names returns nothing); reuses
  `UsageError(String)`, so the four exhaustive `error.rs` matches need no edit, crate compiles.
  Matches `encode.rs:59` precedent.

## Important — None.
- **I1 RESOLVED** — §4 enumerates add address+derive, backfill `repair`, bump header+pinned_version
  "mk 0.3.1"→"mk 0.6.0", bump pinned-upstream.toml:52 v0.4.2→v0.6.0, Dropdown/Number/Range mappings.
  Every claim matches source: schema/mk.rs:1 + :312, `repair` absent from SUBCOMMANDS :267-308,
  pinned-upstream.toml:52, FlagKind::{Dropdown,Number,Range} at schema/mod.rs:121/123/127.
- **I2 RESOLVED + sound** — §3.1 step3 + §3.3 `infer_address_type` gate `Inferred` on
  `origin_path.len()==3`; step5 stderr depth-advisory; §5 test 4b covers leaf/over-deep. Traced:
  depth-3 account auto-infers (happy path intact); depth-5 leaf → exit 64 w/o flag, advisory with.
  `len()==3` correct for 44/49/84/86 accounts; multisig 48'(d4)/87'(d3) refused at step1 before the
  gate. Codec invariant `xpub.depth == component_count(origin_path)` (XpubOriginPathMismatch,
  key_card.rs:49 + mk-codec error.rs:164-180) makes `len()` and `depth` provably equal — §3.1/§3.3
  consistent.
- **I3 RESOLVED** — §3.2: `child_fingerprint = fmt_fingerprint(&child.fingerprint())`, matches
  mod.rs:73 / inspect.rs:44.

## Minor (non-blocking)
- M (citation): §4 cited the manual count at `:5`; "Six" is on line 4 (wraps to 5). Post-PR count is
  **eight**. [Folded post-R1: §4 now cites `:4` + "eight".]
- M (already-loose §3.3, not fold-introduced): `p2pkh` by-value vs `p2shwpkh` by-`&` is an impl
  detail the SPEC leaves to "mirror address_search.rs"; implementer copies exact signatures.

bitcoin-0.32 API details in §3.3 accurate vs live reference (`render_address` over
`Secp256k1<VerifyOnly>` from `verification_only()`, `to_pub()`/`to_x_only_pub()`, four `Address::p2*`,
`derive_pub`, `ChildNumber::is_hardened()`). No non-existent API, no load-bearing mis-citation, no
fold-introduced contradiction across §3.1/§3.3/§5/§6/§7.

**VERDICT: GREEN (0C/0I)**

---
Post-R1: folded the single trivial citation Minor (§4 M4 pointer `:5`→`:4`, count→eight). The
already-loose §3.3 by-value/by-ref note is left to the implementer (R0+R1 both passed it; the SPEC
directs mirroring the exact `address_search.rs` signatures). SPEC is GREEN — clear to write the
implementation plan (which gets its own R0 gate before any code).
