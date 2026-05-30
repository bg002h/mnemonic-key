# R1 Review — SPEC_mk_no_path_support.md

Opus architect (feature-dev:code-reviewer), continuing from R0 (RED 0C/1I/4M). Verified
the fold against live source @ branch `mk-no-path-support`. Persisted by controller.

## Fold verification

**I1 (E10 — superseding note for `SPEC_mk_depth_child_enforcement.md`) — RESOLVED.**
E10 added (§4). Names `:30`/`:57`/`:14`; cites the normative pointer `error.rs:170-171`
(confirmed it references both docs). All three before-texts match verbatim
(`:14` invariant framing, `:30` "empty origin_path → reject … require 1..=10",
`:57` "depth-0 master xpub: mk1 cannot represent one"). Head-blockquote + inline-note
approach sound; the 0.3.2 guard description elsewhere stays accurate.

**M1 (§3.3a doc-comment touches) — RESOLVED.** `path.rs:18-20` reads "1-byte component
count (1..=10)" exactly; `key_card.rs:46-51` is the ```text `depth :=`/`child_number :=`
block. Both touches land on the right text.

**M2 (T9 — empty-path arm + `.expect` relax) — RESOLVED, snippet compiles.**
`.expect("path is non-empty …")` confirmed at `tests/common/mod.rs:68-70`. The
`prop_oneof![standard, explicit, Just(DerivationPath::from_str("m").unwrap())]` snippet
COMPILES: the existing `prop_oneof![standard, explicit]` already unifies two different
concrete `Map` strategy types via proptest's `TupleUnion`; a third bare arm with common
`Value = DerivationPath` needs no weight-tuple / per-arm `.boxed()`. Nit: live code ends
`.boxed()`, T9 quote drops it — harmless.

**M3 (E4 tightened to `:257-258`) — RESOLVED.** `:254` intro, `:256-259` fence,
`:257`=`depth :=`, `:258`=`child_number :=`, `:261`=standard-table sentence. E4's
"block at :257-258, keep :254 + :261" is exactly right.

**M4 — correctly no-action.**

## Fold-drift scan
§2 inventory new sites (`error.rs:170-171`, `path.rs:18-20`, `key_card.rs:46-51`,
`tests/common/mod.rs:39-72`) all verified accurate. Phase 0.4 carries §3.3a + T9;
Phase 1 covers E1-E10. §3.3 truth table consistent with E7/E8 + guard code. No contradiction.

## CRITICAL — None.  ## IMPORTANT — None.
## MINOR
- M-r1-1 (cosmetic): T9 cites the `path.as_ref()` comment at `:66-67`; spans `:64-66`. Off-by-one.
- M-r1-2 (cosmetic): E4 replacement-text paraphrases "including hardened-bit encoding" as "(with …)". Reword of replacement text, not a citation error.

## VERDICT: GREEN (0C/0I/2M)
Both Minors cosmetic/optional; the I1 fold + all four M-folds landed correctly; the one
fold-introduced code snippet (T9 `prop_oneof!`) compiles. Gate cleared — proceed.
