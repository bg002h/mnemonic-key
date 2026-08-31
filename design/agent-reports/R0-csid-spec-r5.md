# R0 r5 — fold-check of the r4 lens findings against `fcf3971`

**Scope: exactly as briefed.** Two questions only — (1) does `fcf3971`
discharge each of the 10 r4 findings (5I/4M/1N), and (2) did the fold
introduce a new defect or internal contradiction. This is **not** a fresh
audit: the two r4 lenses were not re-run, and settled rulings (warnings-not-
refusals, opaque-id guarantee, vectors-only formulas, post-cycle legs, R0
rounds r1–r3) are taken as given. Code citations already measured in r4
(`seat/input.rs:59`, `directive.rs:68`, `repair.rs:410`, the `fd6a407`
STANDARD_PATHS growth, the 0.20s grind) are trusted as prior measurement, not
re-run here.

**Verdict: 0 Critical / 0 Important / 0 Minor / 0 Nit open.** All 10 r4
findings are FIXED. No new defect or internal contradiction found in the
fold. **Lens closes.**

Diff inspected: `git diff c4900a7..fcf3971 -- design/SPEC_chunk_set_id_verification.md`
(89 insertions / 22 deletions). Spec read whole at `fcf3971` (310 lines).

---

## Per-finding disposition

| id | sev | disposition | evidence (spec line @ `fcf3971`) |
| --- | --- | --- | --- |
| L1-I1 | Important | **FIXED** | L94–106: *"The operand is FROZEN for id purposes from mk-codec 0.5... any future change to `encode_bytecode`'s output — including adding a `STANDARD_PATHS` entry — is a mismatch-generating WIRE-COMPAT event, not a minor additive change... The extension corpus MUST carry a row pinning the current 14-entry table."* Corpus obligation echoed at L254–255. |
| L1-I2 | Important | **FIXED** | L112–122: *"the id appears as **exactly five lowercase hex digits, zero-padded (`{:05x}`)** — the token `GroupId::Display` prints (`seat/input.rs:59`) and `md --seat @i=` ACCEPTS (it refuses any token whose digit count ≠ 5, `directive.rs:68`); never `{:x}`."* Corpus row obligation at L256–257: *"at least one row whose derived id is `< 0x10000` so the `{:05x}` leading-zero rendering is exercised."* |
| L2-I1 | Important | **FIXED** | L142–150 (coverage) + L268–276 (Acceptance). Coverage: *"repair decodes a card ONLY on the blessed re-verify... An already-valid supply (exit 0) and a partial/single-plate Candidate supply (exit 5, UNVERIFIED advisory) do NOT decode and do NOT warn."* Acceptance no longer says "all six mk verbs"; repair gets its own scoped clause: *"its golden row supplies a DAMAGED pinned card... asserts the warning + mint-time clause fire on exit 5, AND asserts an undamaged pinned supply (exit 0) and a single-chunk Candidate supply are SILENT — repair's warning is not asserted on the plain mismatch rows, which it never decodes."* |
| L2-I2 | Important | **FIXED** | L150–154: *"On the blessed path the warning MUST carry a mint-time clause... e.g. append 'this id was set when the card was minted; the repair did not change it.'"* — matches the required "did not change it" shape exactly. |
| L2-I3 | Important | **FIXED** | L166–173: *"All OTHER JSON envelopes are UNCHANGED this cycle: `mk repair --json` is a byte-match cross-CLI contract with `mnemonic repair --json` (D27; its `schema_version` is the STRING `"1"`, unlike verify's integer) and this cycle does NOT renegotiate it... `mk inspect`'s unconditional stamped-id print (contract 3) is TEXT MODE only; its `--json` envelope gains nothing this cycle."* |
| L1-M1 | Minor | **FIXED** | L63–69: *"the warning is NOT a tamper/authenticity control and must never be read as one... it is a consistency check on how the id was CHOSEN, never on who chose the content. A re-minted substitute card always passes... the 20-bit id can be ground onto any target id in ~2^20 work — measured 0.20 s."* Non-goal sentence present as required. |
| L1-M2 | Minor | **FIXED** | L85–88: *"A card whose STAMPED ID disagrees with the re-encode is in scope; a non-conforming plate whose id was itself computed from the canonical re-encode is NOT, and is out of the warning's reach by design."* Near-verbatim match to the prescribed fix text. |
| L2-M1 | Minor | **FIXED** | L58–62: *"beyond-budget miscorrection (rare; payload damage is largely caught by the existing 4-byte cross-chunk hash — measured, r4 L2-M1: in practice every miscorrection is absorbed by the cross-chunk hash or the group split BEFORE the warning could fire, so the warning is not itself a miscorrection tripwire)"* — the "absorbed before" shape required by the finding. |
| L2-M2 | Minor | **FIXED** | L138–140: *"one warning per mismatching group, in the surface's existing group order (r4 L2-M2 — repair and md seat are batch-capable; the other verbs decode one card)."* — exact match to the prescribed fix wording. |
| L2-N1 | Nit | **FIXED** | L148–150: *"the blessed decode plumbs its `Ok(card)` out rather than decoding twice (r4 L2-N1)."* |

---

## New-defect / internal-contradiction check

**1. Does the L2-I1 Acceptance fold contradict R6 ("same warning
everywhere", L28–31) or contract 2's "all six verbs" claim?**

No contradiction, and the two candidate tensions are each resolved in the
fold text itself:

- Contract 2's opening sentence (L133–137, unchanged by this fold) says all
  six verbs *"share the intake... the recompute seats at that chokepoint
  (or equivalently at each verb's decode call)"* — that parenthetical
  qualifier predates r4 and already anticipates that the recompute rides on
  each verb's own decode call rather than a single shared point. Repair's
  decode call is reached only on the blessed path, so "the recompute seats
  at each verb's decode call" is structurally consistent with L2-I1's
  narrower coverage, not contradicted by it.
- The Acceptance section itself no longer claims uniform repair coverage:
  the diff **removed** "all six mk verbs" from the golden-rows bullet and
  replaced it with an explicit five-verb list (*"decode/inspect/verify/
  derive/address"*) plus a separate, differently-scoped repair clause
  (L265–276). The "all six" framing that L2-I1 flagged as false is gone
  from Acceptance; it survives only in contract 2's structural claim about
  intake-sharing, which is a different (and true) claim.
- R6 tension is addressed explicitly, not left implicit: L155 states *"On
  the blessed path the warning MUST carry a mint-time clause... Acceptance
  already permits per-surface framing, so R6 is not violated."* This
  reasoning is itself grounded in the pre-existing Acceptance clause at
  L274–276 (*"surface framing may differ (R6 parity, testable because the
  operand is pinned)"*), which the fold did not need to touch since it
  already covered this case.

**2. Does the `{:05x}` rule conflict with any `warning_text` example
showing a 5-hex id without padding?**

No. Checked every hex-id-shaped token in the fold and surrounding draft
text (`12345`, `ef12f`, `ef21f`, `91ff6`, `94b47`, `1b1ba`) — all are
already full 5-hex-digit values, so `{:05x}` and unpadded rendering are
indistinguishable for them; none is a 4-digit value printed without a
leading zero. The one place a padded value is shown explicitly (L117–119:
*"V7 `03994`, V9 `04cf9`, V16 `01789`"*) prints it correctly zero-padded,
matching r4's measured values (`grep` confirms this is the only occurrence
of those digit strings in the file). No silent violation.

**3. Do the two new corpus-row obligations (table-pin, `<0x10000`)
contradict "legacy 19 untouched" / "zero changes to existing tests"
clauses?**

No such contradiction. `grep -n -i "untouched\|zero change\|legacy 19\|byte-unchanged\|unaffected"` against the fold's file shows the only
matching clauses are:

- L242 (unchanged by this fold): *"The v0.1 corpus is untouched: its 19
  chunked vectors all carry pre-0.5 pinned ids... and become the
  pinned-by-design MISMATCH half."*
- L289 (unchanged by this fold): *"The two named guarantees are
  byte-unchanged"* — scoped explicitly to `an_explicit_chunk_set_id_still_wins`
  and `canonical_payload_is_chunk_set_id_invariant`, unrelated to corpus
  rows.

Both r4-required new rows (table-pin, `<0x10000`) are added to the **NEW
extension corpus file** (L245: *"A NEW extension corpus file (own SHA pin,
own schema field set) supplies the CLEAN half"*), explicitly distinct from
the v0.1 corpus. L260's *"unaffected (no v0.1 churn here)"* clause is about
the FOLLOWUPS V19-re-pin nit specifically, and the two new rows do not touch
v0.1 either. No conflict.

---

## Conclusion

All 10 r4 findings are FIXED with a spec-line citation each; no unfixed or
partially-fixed items remain. The three targeted new-defect checks in the
brief each came back clean, with the specific mechanism recorded above
(pre-existing qualifier, exact-token grep, corpus-file separation) rather
than asserted. **0 Critical / 0 Important — this fold closes the r4 lens
round.**
