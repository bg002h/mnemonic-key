# CONTINUITY — chunk_set_id-verification cycle (opened 2026-08-31)

**Mission:** pick up the PARKED chunk_set_id-verification spec draft
(committed at `9fbbe36`, authored 2026-08-19, R0 never started) and
run it through the full gate chain — **starting with a
/cycle-prep-style RECON, not R0**: the draft predates both the
converter cycle (which rebuilt the id-grouping pipeline in
descriptor-mnemonic) and the stub-semantics rulings, so every
citation and premise must be verified against the CURRENT trees
before any reviewer is engaged. Find the draft file via
`git show 9fbbe36 --stat`.

## The assessment that opened this cycle (controller, 2026-08-31)

The IDEA is sound and near-overdue; the DRAFT is stale. Three
grounds for the capability:
1. **Measured diagnostics gap:** converter r1-I2 — one card scanned
   twice in different case merged into an id group and refused with
   "two DIFFERENT cards pinned to one chunk-set id … re-mint one of
   them", prescribing re-engraving a good plate.
   Recompute-and-compare distinguishes same-card-twice / genuine
   collision / pinned id.
2. **Cross-language hazard, already-paid-for lesson:** F-212 —
   Go and Rust computed DIFFERENT WalletPolicyIds while 887/887
   tests passed. The chunk-set id is derived
   (mk-codec `derive_chunk_set_id`), load-bearing (seating names,
   `--seat` directives, converter grouping), and pinnable
   (`mk encode --chunk-set-id`) — the formula wants CROSS-LANGUAGE
   EXECUTABLE VECTORS before the fork/device ever trusts it.
3. Cheap: derivation is deterministic; verification is
   recompute-and-report.

Cautions binding the spec: NO formula restatement in prose (a prior
draft restated it and was measured false — see the corrected stub
entry, commit `f5a18a2`: three-repo lockstep + canonical-origin
obligation M4); vectors-first from the first draft
(classifier-precision lesson). Companion candidate for the same
mini-cycle: FOLLOWUPS `stub-keyed-wallet-binding-at-mint`
(commit `bcd8505`, operator-ruled pre-v1.0 compat-free window).

## Process

Risk set (normative codec behavior, three-repo lockstep):
recon → re-grounded brainstorm/spec (walk WITH the operator) → R0 to
0C/0I → plan → R0 → one implementer per phase → whole-diff review.
Reviewer tiers sonnet/opus, fable never. Agents persist reports to
design/agent-reports/. Rust-primary: mk-codec leads, Go follows.
Push per this repo's conventions (check CLAUDE.md; staging ritual
class). Date convention: document dates are working-day labels, git
is the clock (operator ruling B, 2026-08-31).

## Same-day context (2026-08-31, for grounding only)

md-cli 0.14.0 released; mdcli-mini cycle shipped (its N1 taxonomy +
seating diagnostics are the freshest adjacent ground); md-codec
publish BLOCKED (derive needs fork-only miniscript APIs) — F-424
twice-blocked, remedy needs re-deriving; mk installed at v0.13.0.

## Status update (2026-08-31, later same day)

Recon DONE (report committed a2b8850, all headline claims re-verified).
Walk DONE with the operator, W1-W12 (design/WALK_chunk_set_id_2026-08-31.md,
committed 9ff8922): warnings not refusals; loud mint warning; vectors;
same warning content everywhere; device + me-cli + Go-ingestion +
silent-correction legs = post-cycle burndown followups (filed, both
repos). SPEC re-grounded and GREEN 0C/0I at e9e332b after three R0
rounds (r1 opus 4C/6I/5M/3N, r2 sonnet C3 partial, r3 sonnet clean);
reports + folds committed pairwise per process. LENSES STILL UNRUN on
the spec (r1 closure note): adversarial defeat-the-warning, repair
exit-5 bless interaction, and the live operator walk of the NEW message
wordings (W5 lens). NEXT: operator decides walk-of-wordings now vs
straight to IMPLEMENTATION_PLAN (then plan R0, then one implementer;
staleness rule applies -- re-validate plan against tree before
dispatch).

## Status update 2 (2026-08-31, spec question-exhausted)

SPEC done and CLOSED across four lenses at fcf3971:
- contract-completeness: r1 opus 4C/6I/5M/3N -> r2 sonnet (C3 partial)
  -> r3 sonnet clean 0C/0I.
- wording walk WITH operator: W13-W16, six message shapes, folds
  b694005/208efee/7e0d1b3/c4900a7.
- adversarial + failure-states: r4 opus 0C/5I/4M/1N (operand version-
  freeze, {:05x} rendering, mk repair blessed-path coverage, JSON
  scoping, tamper non-goal) -> r5 sonnet clean 0C/0I/0M/0N (fcf3971).
Reports r1-r5 + lenses-r4 persisted in design/agent-reports/.
Post-cycle burndown followups filed (device, me-cli leg, go corpus
ingestion, mk silent-correction, seat auto-partition candidate).

NEXT: IMPLEMENTATION_PLAN_chunk_set_id_verification.md. Risk set applies
(normative-adjacent, three-repo). Plan gets its own R0 to 0C/0I, then
one implementer per phase, staleness re-check before dispatch. Record
the plan's baseline rev. In-cycle surfaces only: mk-cli (6 verbs +
mint + inspect print + verify json), md-cli (seat warning + R5 refusal
rewrite + retain per-chunk headers), fork Go derivation-parity test,
extension vector corpus (incl. table-pin row + <0x10000 row).

## Status update 3 (2026-08-31, plan GREEN — implementation begins)

PLAN done and GREEN 0C/0I at 1cc4c66:
- r1 opus 0C/2I/3M/2N (I1 verify text-mode verdict uncovered; I2
  classification-order row missing) -> fold bdd2349 -> r2 sonnet 0C/0I
  (one fold-introduced Minor: M1 Go/include_str contradiction) -> fixed
  inline 1cc4c66. Reports plan-r1/r2 in agent-reports/.
Baselines re-validated at dispatch: STANDARD_PATHS=14, gen_mk_vectors
feature-gated, V0_1_SHA256 pin enforced, both named guarantees present.

IMPLEMENTATION (UC off, one implementer/phase, TDD, worktree):
P0 extension corpus -> P1 mk-cli read -> P2 mk-cli write+repair ->
P3 md-cli seat -> P4 fork parity. NOW DISPATCHING P0.

## Status update 4 (2026-08-31, implementation COMPLETE — whole-diff review pending)

All 5 phases implemented, TDD, controller-verified live at each gate:
- P0 mk-codec extension corpus (58c8df4, main) -- 21 rows, recomputed by controller.
- P1 mk-cli read warnings 5 verbs + inspect print + verify text/json (37a9524).
- P2 mk-cli mint + repair blessed warnings (1711228) -- boundaries verified live.
  Branch impl/csid-p0 (mnemonic-key), base 725ccb9, 12 src files 875/41.
- P3 md-cli seat warning + R5 refusal rewrite (d30b44f9, descriptor-mnemonic
  impl/csid-p3) -- 4 messages + classification order verified live; retired
  wording gone from production.
- P4 fork Go derivation-parity, all 20 clean rows, no drift (a63f1fb,
  seedhammer impl/csid-p4) -- reran live PASS.
Reports impl-csid-p0..p4 persisted. Suites: mk 387/387, dm 697/697
(workspace 1164/1164), fork ./mk green.

PARKED NIT (fix in whole-diff cleanup): repair blessed-path mint-time clause
starts lowercase after a period ("...automatically. this id was set...").

NEXT: mandatory whole-diff independent review across all 3 repos (risk-set,
non-deferrable) -> fold -> integrate (staging push ritual per repo) ->
burndown reconcile.

## Status update 5 (2026-08-31, INTEGRATED locally — origin push PENDING operator go-ahead)

Whole-diff review CLOSED 0C/0I (cc77688): R6 warning parity byte-identical
across mk-cli/md-cli/corpus, {:05x} everywhere, stdout/exit contracts intact,
repair --json byte-pinned, all 20 Go rows match. Nit 1 (lowercase clause)
folded (ea9437f), full suite reverified 387/387. 2 Minors filed post-cycle
(md-cli-seat-warning-corpus-binding; Go binding = existing
go-mk-vector-corpus-ingestion).

LOCAL INTEGRATION done, all validated on merged trees:
- mnemonic-key main @ ab58726 (merge of impl/csid-p0) -- 387/387
- descriptor-mnemonic main @ 77be9fc8 (merge of impl/csid-p3) -- 697/697
- seedhammer main @ 195df90 (merge of impl/csid-p4) -- ./mk ok
Worktrees removed. Ahead of origin: mk 39, dm 6, sh 2.

NEXT (needs operator go-ahead — 3 outward-facing origin pushes):
push each repo per its ritual. mnemonic-key/descriptor-mnemonic use the
ci/staging ref dance (push master:refs/heads/ci/staging, watch test
(rust+go), push master, delete staging; FREEZE tip for the window).
Confirm seedhammer's branch/protection (fork bg002h/seedhammer; DCO on P4).
Then reconcile burndown (5 post-cycle followups filed).
