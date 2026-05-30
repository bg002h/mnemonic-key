# SPEC — mk-codec encode-time xpub ⇄ origin_path agreement enforcement

**Status:** design → mandatory opus R0 gate (0C/0I before any implementation).
**Repo:** `mnemonic-key` (default branch **`main`**). **Source SHA:** `998f3c9` (origin/main @ recon).
**Resolves:** FOLLOWUP `mk1-depth-child-lossless-by-construction-unenforced` (`design/FOLLOWUPS.md:284`); Theme-C of the constellation feature survey (recon: `mnemonic-toolkit/cycle-prep-recon-theme-c-footguns.md` item 2).
**SemVer:** mk-codec **0.3.1 → 0.3.2 (PATCH)** — correctness bug fix; only newly-rejects inputs that previously produced silently-corrupt cards; the new error variant is additive under `#[non_exhaustive]`. No GUI schema-mirror, no manual lockstep (no clap-flag change).

---

## §1 — The bug (verified)

mk1's compact-73 xpub wire form drops `xpub.depth` and `xpub.child_number` (`crates/mk-codec/src/bytecode/xpub_compact.rs:45` `XpubCompact::from_xpub`, infallible; preserves only version/parent_fingerprint/chain_code/public_key). On decode, `reconstruct_xpub` (`xpub_compact.rs:85`) rebuilds them from `origin_path`: `depth := component_count(origin_path)`, `child_number := last_component(origin_path)` — **with NO validation that the supplied xpub agreed**. The encode entry `encode_bytecode` (`bytecode/encode.rs:21`, guard-relevant lines `:43` `encode_path` then `:44` `from_xpub`) silently drops.

**Consequence:** a caller passing an xpub whose `depth`/`child_number` don't match `origin_path` gets a card that decodes to a **different-metadata xpub**. `chain_code` + `public_key` are preserved, so *addresses still derive correctly* — the corruption is confined to the reconstructed BIP-32 serialization's `depth`/`child_number` fields, detectable only at the §5 step-4 Wallet-Instance-ID check (the SPEC's own "Limit-of-detection note", `SPEC_mk_v0_1.md:265`, documents this as an *accepted* silent error). `SPEC_mk_v0_1.md:263` calls compact-73 "lossless by construction"; `:301` says the `XpubDepthMismatch` rule was "removed … impossible by construction." Both are FALSE: losslessness holds ONLY if the caller pre-aligns. The format's own reconstruction rule `depth := component_count(origin_path)` (`SPEC:257`) IS the invariant — currently unenforced.

## §2 — The fix: an encoder-side agreement guard

**This is ENCODER-ONLY enforcement.** The decoder genuinely *cannot* detect the mismatch — the wire form carries no depth/child to compare against — so this rule is structurally parallel to the existing fingerprint-flag encoder invariant (`SPEC:292`; the retired `FingerprintFlagMismatch` variant at `error.rs:231-236` is the direct in-codebase precedent for an encoder-side invariant), NOT a decoder-rejection rule.

### §2.1 — Guard location (sole chokepoint)
`encode_bytecode` (`bytecode/encode.rs`) is the **single** xpub-serialization site. Architect-traced: both public encode entries (`string_layer/pipeline.rs:56,67`) call `encode_bytecode(card)` first, then `encode_bytecode_stream` operates on opaque bytes — the single-string, chunked, long-code, and vector-gen paths all flow through `encode_bytecode`, and none re-serializes the xpub. A guard here covers **100%** of wire emission. (NOT `KeyCard::new` — `KeyCard` fields are `pub`, so a constructor check is bypassable. NOT `from_xpub` — it lacks the path.)

Insert the guard in `encode_bytecode` immediately before `XpubCompact::from_xpub(&card.xpub)` (current `encode.rs:44`), after `encode_path` (`:43`). `encode_bytecode` already returns `Result<Vec<u8>>`.

### §2.2 — Check semantics
Reject if EITHER:
- `card.xpub.depth as usize != card.origin_path.into_iter().count()`, OR
- `card.xpub.child_number != <last component of origin_path>`.

Extract the last component as `card.origin_path.into_iter().last().copied()` → `Option<ChildNumber>`. Compare the raw `ChildNumber` with `==` (bitcoin `0.32` `ChildNumber` is an enum `Normal{index}`/`Hardened{index}` carrying the hardened bit structurally — direct `==` is the exact inverse of reconstruction; do NOT `u32`-normalize). An empty `origin_path` (`None`) is a mismatch → reject (it is encode-unreachable for a valid card — `encode_path` + the decoder require `1..=10` components, `SPEC:237,285` — but is hand-buildable via the `pub` fields, so the guard must not `unwrap`).

### §2.3 — Error variant
mk-codec `error.rs` enum is `#[non_exhaustive]` (`:18`) → additive. **It is NOT alphabetized** (it is grouped string-layer then bytecode-layer with section banners, `:99` "Bytecode-layer errors"). Place the new variant in the **bytecode-layer group** (near `TrailingBytes`/`UnexpectedEnd`, `:141-146`). **Do NOT alphabetize** — that is the *toolkit's* `ToolkitError` convention, not mk-codec's; flag this in the plan so a reviewer doesn't "correct" it.

```rust
/// Encoder-side: the supplied `xpub`'s BIP-32 `depth`/`child_number`
/// disagree with `origin_path` (depth ≠ component count, or child_number
/// ≠ last component). Compact-73 reconstructs both from the path, so an
/// emitted card would decode to a different-metadata xpub. Rejected at
/// encode to keep compact-73 genuinely lossless.
#[error(
    "xpub origin-path mismatch: xpub depth {xpub_depth} / child {xpub_child} \
     vs origin_path depth {path_depth} / last {path_child:?}"
)]
XpubOriginPathMismatch {
    xpub_depth: u8,
    path_depth: u8,
    xpub_child: ChildNumber,
    path_child: Option<ChildNumber>,
},
```
Name is `XpubOriginPathMismatch` (NOT the historical `XpubDepthMismatch` — that undersells the child-number coverage; reconcile the historical name in the SPEC/FOLLOWUP prose). `path_child` is `Option` for the empty-path case (renders via `{path_child:?}`). **`error.rs` must add `use bitcoin::bip32::ChildNumber;`** (re-grep at impl time — it is not currently imported there).

### §2.4 — Edge cases (architect-verified: NO false positives against any valid card)
- **Standard-table indicator** path: the dictionary always dereferences to the FULL path (`bytecode/path.rs:38-55`); a correctly-derived xpub-at-that-path matches the full deref depth/child. The only divergence is the bug itself.
- **Elided / partial origin** (the make-or-break risk): mk1 has **NONE** — `path.rs` encodes the full path in both standard-table (table `:38-55`, deref `lookup_indicator` `:60-65`) and explicit (`encode_path` `:85-98`, LEB128 every component) modes; no md1-style "last N components" facility (md1's elided origin lives in a different codec/layer). `depth == len` enforces the format's existing contract (`SPEC:257`), not a new constraint.
- **depth-0 master xpub:** mk1 cannot represent one (paths are `1..=10`; `encode_path`→decoder rejects `count==0` as `PathTooDeep(0)`). No spurious reject.

## §3 — SPEC_mk_v0_1.md edits (re-grep line numbers at impl time)
The decoder-cannot-detect framing MUST be preserved — keep these in the encoder-side-invariant bucket, do not move into the numbered decoder rules.
1. **§3.6:263** — reword "Compact-73 is *lossless* … impossible by construction": lossless *because the encoder enforces agreement* (both fields reconstructible from the path AND the encoder rejects any xpub whose depth/child_number disagree with `origin_path`, `Error::XpubOriginPathMismatch`).
2. **§3.6:265** (Limit-of-detection note) — REFRAME, do not delete: the encoder now closes the EMIT side (you can no longer *produce* such a card through `encode`). Residual limit-of-detection applies only to hand-constructed bytecode fed directly to the decoder (which still reconstructs from path with no on-wire depth to cross-check). Keep the §6 out-of-band first-address recommendation.
3. **§4:301** — replace "rule is removed … impossible by construction" with: re-instated as an **encoder-side invariant** `Error::XpubOriginPathMismatch` (covers depth AND terminal child-number).
4. **§4 (near :292)** — add a sibling "Encoder-side invariant (not a decoder rule)" paragraph mirroring the fingerprint-flag one: encoders MUST reject `xpub.depth ≠ component_count(origin_path)` OR `xpub.child_number ≠ last_component(origin_path)` with `Error::XpubOriginPathMismatch`; structurally undetectable at decode (no on-wire depth).

## §4 — FOLLOWUP updates (same PR, no toolkit code)
- Flip `mk1-depth-child-lossless-by-construction-unenforced` (`mnemonic-key/design/FOLLOWUPS.md:284`) → `Status: resolved <sha>` with the resolution (encoder-side `XpubOriginPathMismatch` guard).
- The toolkit companion `mk1-depth-child-compensating-check-watch` (`mnemonic-toolkit/design/FOLLOWUPS.md:3335`) + the compensating check `synthesize.rs:494-503` stay as **defense-in-depth** this cycle (keep scope to one repo). Per the lockstep convention, annotate the toolkit watch entry that the upstream is resolved and the toolkit check is now reviewable-for-removal but NOT removed here. (Annotating the toolkit FOLLOWUP is a docs-only toolkit commit; optional — may be deferred to when the toolkit relaxation is actually done. Decide at plan time.)

## §5 — Tests (`crates/mk-codec/`, per-phase TDD)
1. **Reject on depth mismatch:** build a `KeyCard` with `xpub.depth` ≠ `origin_path.len()` → `encode_bytecode` returns `Err(XpubOriginPathMismatch{..})`.
2. **Reject on child mismatch (same depth):** `origin_path.len() == xpub.depth` but `origin_path.last() != xpub.child_number` → `Err(XpubOriginPathMismatch{..})`. (The previously-silent same-depth case.)
3. **Empty-path reject:** hand-built `KeyCard` with empty `origin_path` → `Err` (`path_child: None`), no panic.
4. **Aligned card still encodes + round-trips:** a correctly-aligned card (incl. the existing `xpub_compact.rs:144` `round_trip_full_xpub_depth_4` fixture) encodes successfully and `reconstruct_xpub` yields the identical xpub — genuine losslessness.
5. **Standard-table-indicator aligned card** encodes (no false positive on a real dictionary-path card).
6. *(optional, plan-author discretion — R0 suggestion)* **Standard-table child-mismatch:** a card using a dictionary indicator (e.g. `0x05` = `m/48'/0'/0'/2'`) but with `xpub.child_number` set to a different terminal (e.g. `1'`) → `Err(XpubOriginPathMismatch{..})`. Strengthens dictionary-path child coverage symmetrically to cell 5.

## §6 — Cross-cutting
- **mk-cli same-repo lockstep (added in end-of-cycle R0 fold):** the new variant is reachable from `mk encode` (mis-aligned `--xpub`/`--origin-path`), so mk-cli re-pins `mk-codec 0.3.1 → 0.3.2` + PATCH-bumps `0.4.2 → 0.4.3` + adds the `XpubOriginPathMismatch` arm to its `mk_codec_error_kind` JSON-`kind` map (`mk-cli/src/error.rs`). Adding ANY `mk_codec::Error` variant requires updating BOTH mirrors: `mk-codec/tests/error_coverage.rs` (`ErrorVariantName` + `display_prefix` + `is_exempt`) AND `mk-cli/src/error.rs` (`mk_codec_error_kind`) — the end-of-cycle R0 caught their omission (the SPEC/plan R0s reviewed the design, not the sibling-mirror surface). No GUI schema-mirror / manual change (no clap-flag change).
- **Fix-the-class (architect-verified clean):** no 2nd instance in mk-codec (parent_fingerprint/version/chain_code/public_key all carried verbatim; depth/child are the only reconstructed fields → one variant, one chokepoint). No analog in md-codec (md1 hardcodes depth/child, never reconstructs — `descriptor-mnemonic/.../md-codec/src/derive.rs:44-60`). No share path (v0.1 has none).
- **CHANGELOG:** mk-codec has no `CHANGELOG.md` found at `crates/mk-codec/CHANGELOG.md` (re-verify at impl; if absent, skip or add a minimal one — decide at plan time, don't block).
- **Bug-handling:** none expected (this is the fix; existing tests must stay green — the aligned round-trip cells already pass).

## §7 — Phasing (for the plan-doc)
- **Phase 0:** error variant + `ChildNumber` import + the `encode_bytecode` guard. Tests 1-5 (TDD: reject tests before the guard).
- **Phase 1:** SPEC_mk_v0_1.md edits (§3.6:263/265, §4:292/301) + FOLLOWUP flip.
- **Phase 2:** version 0.3.1→0.3.2 (+ CHANGELOG if present) + end-of-cycle R0 + ship to `main`.
