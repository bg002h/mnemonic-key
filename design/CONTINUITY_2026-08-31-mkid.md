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
