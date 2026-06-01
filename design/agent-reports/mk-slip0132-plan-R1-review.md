# A2 PLAN R1 review — mk SLIP-0132 acceptance (fold verification)

**PLAN:** `design/IMPLEMENTATION_PLAN_mk_slip0132_acceptance.md` (post-R0-fold)
**R0 review:** `design/agent-reports/mk-slip0132-plan-R0-review.md`
**Reviewer:** sonnet verification agent (R1 — fold-only scope; core verified GREEN at R0)
**Source SHA reviewed:** mk `main` `fc2341b`; `output_advisory.rs` line 31 empirically re-grepped.

## Verdict: GREEN (0C / 0I)

All five R0 findings (C1, I1a, I1b, M-a, M-c) are correctly folded. No drift introduced. The plan is execution-ready.

---

## Fold verification

### C1 — `parse_xpub` dead-code / no non-card caller

**Status: RESOLVED.**

- **A1 Step 5** (plan :172) now reads: "`parse_xpub` has ONLY the encode + verify callers — after both are rewired (encode here, verify in A3) it becomes dead code that FAILS `-D warnings` in this bin-only crate, so A3 deletes it. (Do not claim a 'non-card caller'; there is none.)" The original false rationale ("keep it for any non-card caller") is gone.
- **A3 Step 3b** (plan :254) is an explicit new step: "Delete the now-orphaned `parse_xpub` from `crates/mk-cli/src/cmd/mod.rs` (R0 C1 — both callers are now rewired; leaving it = `dead_code` → `-D warnings` failure). Remove the `pub fn parse_xpub` (`:57-59`) and confirm no remaining references (`grep -rn parse_xpub crates/mk-cli/src` → only `parse_xpub_normalized`)."
- **A3 Step 6 `git add`** (plan :262) includes `crates/mk-cli/src/cmd/mod.rs` explicitly.
- **Timeline soundness confirmed:** after A1 (encode rewired), `verify.rs:53` still calls `parse_xpub` — the function has one remaining caller, so A1 clippy passes. After A3 Step 3 (verify rewired) + Step 3b (function deleted), the A3 clippy passes with no orphan. There is no window where `parse_xpub` is orphaned but not yet deleted.

### I1a — dead_code gate (not missing_docs) + pub-item reachability

**Status: RESOLVED.**

A1 Step 7 (plan :202) now states explicitly: "**CRITICAL (R0 I1c):** the gate here is `dead_code`, not `missing_docs` (the latter is already crate-allowed). Every `slip132` `pub` item — the `Slip132Variant` enum + `label`/`canonical_label`/`path_matches`/`mismatch_help`/`detect_and_normalize` — must be reachable from NON-test code via the encode→`parse_xpub_normalized` path (a `#[cfg(test)]`-only use does NOT keep bin-target items live). `mismatch_help` is reached on the refuse branch (a real use)."

The reachability chain is correct: `encode` → `parse_xpub_normalized` → `detect_and_normalize` (always) → `Slip132Variant` constructed → on `Some(v)`: `label()` + `canonical_label()` in the `eprintln!`; → if `path_matches()` fails → `mismatch_help()`. All six pub items reachable from non-test code at the A1 commit. The A1 Step 8 `git add` co-locates `slip132.rs` + `cmd/mod.rs` + `encode.rs` in one commit, so there is no intermediate state where `slip132.rs` lands without `parse_xpub_normalized` using it.

### I1b — em-dash U+2014 in A3 Step 4

**Status: RESOLVED.**

A3 Step 4 (plan :256) specifies the constant verbatim: `const WATCH_ONLY: &str = "note: stdout is watch-only \u{2014} public keys only, cannot spend";` with explicit instruction "(R0 I1b — assert `\u{2014}`, NOT a hyphen, or the `contains` is vacuous)."

Live source check: `output_advisory.rs:31` reads `"note: stdout is watch-only \u{2014} public keys only, cannot spend"` — byte-for-byte match with the plan's constant. The plan's literal is correct and will not produce a vacuous `contains`.

### M-a — `want_path` reuse in verify `:84-93` block

**Status: RESOLVED.**

A3 Step 3 (plan :252) now explicitly states: "Reuse `want_path` for the existing origin_path content-match block (`:84-93`): that block currently re-parses `args.origin_path` via `parse_derivation_path` — replace its re-parse with the already-computed `want_path` (e.g. `if let Some(want) = &want_path { if *want != card.origin_path { … } }`) so the path is parsed once." The rewritten block form is shown; the double-parse and move/borrow hazard are both resolved by the pattern.

### M-c — `missing_docs` vs `dead_code` framing (folded into I1a)

**Status: RESOLVED** (absorbed into I1a fold above — A1 Step 7 now names `dead_code` as the real gate; `missing_docs` is noted as crate-allowed and not the risk).

---

## Drift check

No drift introduced by the folds:

- `parse_xpub_normalized` is the helper name consistently throughout A1 Step 5, A1 Step 6, A3 Step 3, and the Architecture summary (plan :7). No variant spellings.
- A1/A2/A3 task structure is coherent: A1 lands slip132 + encode wiring; A2 adds unit tests + encode edge cells; A3 wires verify + deletes parse_xpub + adds verify cells + stderr-ordering cell. The sequence is sound and no step references a prior step that no longer exists.
- No new placeholders (e.g. "TBD", "TODO", "see later") introduced.
- The A3 Step 6 `git add` (`verify.rs` + `cmd/mod.rs` + test file) is consistent with the work described in Steps 3, 3b, and 4.
- The A1 Step 8 `git add` continues to stage `cmd/mod.rs` (for adding `parse_xpub_normalized`) — correct, `parse_xpub` is NOT deleted at A1 (that is A3 Step 3b).
- M-b (parse_derivation_path ordering change noted as benign) was Minor/non-blocking at R0 and carries forward unmodified — the plan's note that `want_path` is hoisted before `:50`'s decode is consistent with the M-b observation; no assertion of old ordering exists in tests, so no action required.

---

## Summary

All R0 findings folded correctly and completely. The plan is execution-ready.
