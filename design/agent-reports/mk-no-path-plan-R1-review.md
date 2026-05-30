# R1 Review — IMPLEMENTATION_PLAN_mk_no_path_support.md

Opus architect, continuing from plan R0 (RED 1C/0I/3M). Verified the C1 fold. Persisted by controller.

## Fold verification (C1)
- **Task 0.4 Step 2 now correct.** The `xpub_strategy` instruction explains the live binding
  `let child_number = *components.last().expect(...)`, why the literal swap fails, and gives the
  full corrected block dropping `*` + inserting `.copied()`:
  `components.last().copied().unwrap_or(ChildNumber::Normal { index: 0 })`.
- **Type-check vs `tests/common/mod.rs:66-70`:** `components: Vec<ChildNumber>` → `.last()`
  `Option<&ChildNumber>` → `.copied()` `Option<ChildNumber>` → `.unwrap_or(Normal{0})` `ChildNumber`.
  Matches the prior `*…expect` result type; downstream `.prop_map` closure unaffected. `ChildNumber`
  in scope (element type of the annotated `Vec<ChildNumber>` + Xpub construction).
- **No fold-drift:** `path_strategy` empty-path arm intact (feeds the default branch), run command
  present, surrounding steps + self-review type-consistency unchanged.

## CRITICAL — None.  ## IMPORTANT — None.
## MINOR (carried from R0, cosmetic, not re-litigated)
- M1 citation off-by-one; M2 `--test '*'` run-scope note; M3 fmt nit.

## VERDICT: GREEN (0C/0I/3M) — clear to implement.
