# R0 Review — SPEC_mk_no_path_support.md

Opus architect (feature-dev:code-reviewer), mandatory pre-impl R0 gate. Branch
`mk-no-path-support`, base `main` `5c2bc8c`. Verified against live source + bitcoin
0.32. Persisted by controller (review agent had no Write tool).

## Headline confirmations (file:line)

- **Decode-side bug confirmed.** `path.rs:114` is exactly `if count == 0 || count > MAX_PATH_COMPONENTS { return Err(Error::PathTooDeep(count)); }` (verified against local tree AND `origin/main`). `MAX_PATH_COMPONENTS = 10` (`consts.rs:27`). The loop `for _ in 0..count` (`path.rs:118`) runs zero times for `count==0`, and `Ok(DerivationPath::from(components))` with an empty vec yields the empty path. §3.1 (drop the `count == 0` disjunct) is sufficient. ✓
- **reconstruct_xpub `.expect()` confirmed.** `xpub_compact.rs:92-95` `.expect("origin_path must be non-empty per SPEC §3.5")` panics on empty today. §3.2's `.unwrap_or(ChildNumber::Normal { index: 0 })` is correct; `depth = components.len() as u8` (`:88`) already yields 0. `ChildNumber` already imported (`xpub_compact.rs:18`). ✓
- **Guard confirmed + is the exact inverse of reconstruct.** `encode.rs:33-42` current guard uses `Some(card.xpub.child_number) != path_child`. §3.3's `expected_child = path_child.unwrap_or(Normal{0})` then `card.xpub.child_number != expected_child` is the precise inverse of the post-§3.2 `reconstruct_xpub`. All 5 truth-table rows verified, incl. "depth-3 xpub + empty path": `path_depth=0`, `xpub.depth=3` → `3 != 0` → reject via depth clause. ✓
- **Production-scope `ChildNumber` import needed-not-duplicate.** `encode.rs:14-18` production imports lack `ChildNumber`; only `#[cfg(test)]` mod imports it (`:68`). §3.3 correctly adds it to production scope. ✓
- **TDD inversions confirmed.** `rejects_path_count_zero` (`path.rs:248-258`) asserts `Err(PathTooDeep(0))`; `rejects_empty_origin_path` (`encode.rs:155-170`) asserts `Err(XpubOriginPathMismatch{ path_child: None, .. })`. Both correctly become accept-cases. T4 pre-change panic confirmed. ✓
- **Mirror claim confirmed.** No new `Error` variant. `error_coverage.rs` `ErrorVariantName::XpubOriginPathMismatch` (`:77`), `display_prefix` (`:108`), `is_exempt` (`:124-129`) all present; `is_exempt` rationale remains accurate (guard stays encoder-side). `mk-cli/src/error.rs:133` kind map already has the arm. No mirror edits needed. ✓
- **Versions/pins confirmed.** `mk-codec/Cargo.toml:3` = `0.3.2`; `mk-cli/Cargo.toml:3` = `0.4.3`, pin `mk-codec … version="0.3.2"`. ✓
- **No existing vector has an empty path** — corpus paths all ≥3 comps or negative cases; `gen_mk_vectors.rs:966-989` synthetic builder uses `.unwrap_or(Normal{0})`. ✓
- **§4 SPEC_mk_v0_1.md citations all confirmed:** `:172`, `:229`, `:237`, `:263`, `:285`, `:294`, `:303`. Before-texts match.

## CRITICAL — None

The three code changes are correct, the guard is the exact inverse of reconstruction, imports are right, the no-path round-trip (T8) will pass.

## IMPORTANT

**I1 — Stale companion design doc `SPEC_mk_depth_child_enforcement.md` directly contradicts this cycle and is NOT in the §4 edit list.** `error.rs:170-171` (the `XpubOriginPathMismatch` rustdoc) cites `design/SPEC_mk_depth_child_enforcement.md` as a normative reference. That doc makes three claims this cycle reverses, with no proposed edit:
- `:30`: "An empty `origin_path` (`None`) is a mismatch → reject (it is encode-unreachable for a valid card — `encode_path` + the decoder require `1..=10` components ...)."
- `:57`: "**depth-0 master xpub:** mk1 cannot represent one (paths are `1..=10`; `encode_path`→decoder rejects `count==0` as `PathTooDeep(0)`). No spurious reject."
- `:14`: frames `depth := component_count(origin_path)` as "the invariant" with no depth-0 carve-out.

A future reader following the error.rs doc-pointer lands on a doc asserting the opposite of the new behavior. Add `SPEC_mk_depth_child_enforcement.md` to §4 with a v0.4.0 superseding note on `:30`/`:57` (depth-0 carve-out: empty path is now a valid consistent card). Fold into Phase 1.

## MINOR

**M1 — §2/§3 miss two production doc-comment sites asserting the old `1..=10`/non-empty invariant.** §3.2 updates only `xpub_compact.rs`. Still stale: `path.rs:18-20` module rustdoc ("1-byte component count **(1..=10)**"); `key_card.rs:46-51` (the `xpub` field reconstruction-rule block, no empty-path/`Normal{0}` note). Add both to a Phase-0 doc-touch (same files in scope).

**M2 — Property-test `tests/common/mod.rs` not inventoried; bijection won't cover depth-0.** `path_strategy` (`:39-56`) generates only standard-table OR `1..=10`-comp explicit; `xpub_strategy` (`:68-70`) hard-`.expect("path is non-empty …")`. Nothing breaks (never produces empty), but depth-0 gains no property coverage — only T8. Optional: add an empty-path arm + relax `.expect` to `.unwrap_or(Normal{0})`. Flag so green proptest isn't read as depth-0 proof.

**M3 — §4 E4 line-span loose.** The two-line reconstruction block is `257-258` (inside fence `256-259`); `254` is intro, `261` the "standard-table indicator" sentence to KEEP. Tighten to "block at `:257-258`, keeping `:261`" at Phase-1 write time.

**M4 — Spec §1 cites toolkit-side line numbers** (`verify_bundle.rs:1225`, `inspect.rs:178`, pin "0.3.1") out-of-repo/unverifiable here. Not load-bearing (motivate, don't gate); Phase 3 is a separate toolkit cycle with its own R0. No action this cycle.

## Test-coverage assessment

Strong. T1-T8 cover every changed surface; three reject-inversions correctly FAIL pre-change (TDD red); T8 is the high-value end-to-end. Gaps non-blocking: depth-0 absent from proptest bijection (M2); optional corpus vector deferrable. Exhaustiveness gate + kind-map need no touch (no new variant), correctly identified.

## VERDICT: RED (0C/1I/4M)

One IMPORTANT (I1: the normatively-cited companion doc `SPEC_mk_depth_child_enforcement.md` still asserts "mk1 cannot represent a depth-0 xpub"/"empty path → always reject," absent from §4). Fold I1 into §4/Phase 1; address MINORs at discretion (M1 cheap same-file doc touch; M2/M3 flags; M4 no-action). Code design itself correct and ready — the gate is the documentation-consistency miss.
