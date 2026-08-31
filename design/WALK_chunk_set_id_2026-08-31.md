# WALK — chunk_set_id verification brainstorm (2026-08-31, in progress)

Step 1: the seating-refusal moment, reproduced minimally (policy card
`md1yppqqxppsg2vlumagltz27le`, two 2-chunk plates pinned to 0x12345;
and separately one ordinary plate half-scanned). Commands + verbatim
output in the session transcript; artifacts in scratchpad.

## Findings so far (operator confusion is data)

- W1: the refusal's vocabulary is not held by the operator at the
  moment of failure — "key card", "chunk-set id", "re-mint", "pinned"
  all undefined at point of use. Operator, verbatim: "I don't
  understand what it means. I don't understand the suggested remedy.
  What is a keycard?"
- W2: one sentence fires for at least three different situations with
  three different correct remedies (missing chunk / two cards merged /
  clean reassembly with non-derived id). Measured: a 1-of-2 half-scan
  is told to "re-mint one of them". Only the third case touches the
  derivation; the first two are distinguishable today from header
  forensics alone (received-vs-declared counts, duplicate chunk
  indexes, mixed total_chunks).
- W3: the journey itself was not legible from a prose presentation —
  the walk needed the full command transcript before step 1 could be
  judged. Grounding artifact for the spec: the 5-command minimal
  reproduction.

Classification of divergences: pending, with the operator.

## Step 2 — re-anchored at the operator's real starting point

- W4 (journey divergence, operator verbatim: "Step 1, I didn't do
  that. I would start with the md1 string."): the restore journey
  begins with engraved strings in hand. Mint commands are a different
  actor on a different day; presenting them as journey steps confused
  the walk. Spec journeys must be restore-first.
- W5 (message direction SETTLED, operator verbatim: "Your verbiage is
  way better than the error messages and would help me a lot!"): the
  refusal rewrite direction is plain-language state+remedy — say what
  the tool measured (received vs declared, duplicate piece numbers,
  mixed totals), name the concrete next action, define terms at point
  of use. Candidate shapes endorsed: half-scan -> "this card says it
  has 2 pieces and you've only given me 1 — scan the missing piece";
  merged -> "pieces of two different cards mixed together; separate
  them, and only if both truly carry the same id, re-engrave one".
- Measured (both first moves from the md1 string alone):
  `md descriptor <md1>` exits 2 with an ACTIONABLE refusal naming
  --from-mk1 and `md decode` (the converter cycle's fix, working);
  `md decode <md1>` prints the keyless template plus origin notes.
  Residue: the refusal's front matter is jargon ("wallet-policy mode
  (Pubkeys TLV)") before the actionable part.

## Step 3 — the rerun with --from-mk1, end to end (measured)

- W6 (operator ground truth, verbatim: "I would probably know an
  address from wallet or a wallet id"): the operator's restore
  verification instinct is address/wallet-id comparison. MEASURED:
  `md descriptor` already serves it — stderr prints "composed wallet
  id 72739df2 · policy shape id a235ee75" and "address 0 ... compare
  against your wallet software before trusting." The instinct and the
  tool already meet.
- W7 (vocabulary hazard, operator verbatim: "What's an engrave id?
  How is that different from a chunk set id?"): the walk author's own
  loose phrase ("engraved id") spawned a phantom concept. Any id
  prose does this. Spec language must fix ONE term: the chunk-set id
  as stamped-on-steel vs recomputed-from-content.
- Journey correction: hand-minting the policy without inline template
  origins produced a seating refusal (slots [deadbeef/] vs cards
  [deadbeef/48'/0'/0'] — exact matching, no path inference).
  `md decompose <concrete descriptor>` prescribes the exact mint and
  prevents this class. That refusal message is the house's best:
  names every slot, card (by csid), stub, and origin.
- THE CRUX, measured: a plate pinned to 0x12345 (content derives
  ef12f) seats SILENTLY on the happy path — identical descriptor,
  identical wallet id, identical address 0, exit 0. Only trace: the
  card's display name in the SHAPE-CONFIRMED note reads 12345 instead
  of ef12f. The operator's own checks (wallet id, address) can NEVER
  catch a non-derived id — it does not enter wallet math. What it
  does poison: (a) grouping/collision refusals (step-1 class),
  (b) every diagnostic that NAMES plates by csid (re-mint "12345" and
  the replacement mints as ef12f — name drift), (c) on-device scan
  grouping (fork gui accumulator keyed by ChunkSetID).
- Address-integrity control: `md address --template --key
  @i=[fp/path]xpub` from the ORIGINAL keys = bc1qgx6...qpts2c,
  byte-identical to the composed-from-plates derivation; mk decode
  returns the exact original xpub. (Composed stdout re-serializes
  xpubs at depth 0 — cosmetic, addresses equal, not chased.)

Pending: operator classification of "stamped id != content on the
happy path" — refusal (codec, Position B) / warning (tools recompute
and note) / documentation only. Constraint from the walk rule: a
change is earned only if silence is worse.

## Step 4 — operator leans WARNING; channel analysis (measured)

- W8 (operator, verbatim: "I'd lean towards warning but I fail to see
  how we would get a mismatch to begin with…maybe user hand engraving
  error, could that do it?"). Measured answer: hand-engraving error
  essentially CANNOT produce stamped!=derived:
  - one flipped character (header region, B3 chunk 1, 'p'->'x'):
    `mk decode` SILENTLY AUTO-CORRECTS it and returns the intact card;
    `mk repair` names the same correction ("1 correction at position
    6: 'x' -> 'p'"). The slip never reaches the id comparison.
  - an id altered on ONE chunk only (simulated: pinned chunk 1 +
    ordinary chunk 2 of the same plate): chunks land in different
    groups -> two incomplete groups -> the "received 1, declares
    total_chunks = 2 … re-mint one of them" refusal. Never a clean
    mismatch, and the misleading message fires AGAIN on a third
    distinct situation.
  - to survive to a clean mismatch, ALL chunks must consistently carry
    the same non-derived id with intact payload — i.e. minted that way.
- Realistic channels, in order: (1) `mk encode --chunk-set-id` leaking
  into a real mint (stale runbook / copied fixture command — the flag
  ships today); (2) cross-implementation derivation drift, the F-212
  shape: if the Go port's formula drifts, every plate minted on one
  side reads mis-stamped to the other, and TODAY nothing would notice
  — a recompute-and-report warning is a continuous field tripwire for
  encoder conformance; (3) beyond-budget BCH miscorrection converging
  on a wrong valid string (rare; payload changes are mostly caught by
  the 4-byte cross-chunk hash; id-field residue tiny); (4) deliberate
  tamper — the id is the only field no content check covers.
- Discovered en route, recorded: `mk decode` corrects engraving errors
  SILENTLY (no note, no count) — same class as the mt "error-budget
  consumed silently" lesson; a plate near its correction budget passes
  as pristine. Candidate followup for mk verify/decode correction
  reporting; NOT this cycle's scope.

## Step 5 — rulings (operator, this session)

- W9 (operator, verbatim: "--chunk-set-id flag should probably warn
  loudly when used …"): mint-time loud warning on the pin flag.
  MEASURED today: pinning mints in total silence (stderr carries only
  the generic watch-only note) — the change is silence -> loud.
- W10 (operator, verbatim: "And earn per 2 above", read as "and warn
  per [channel] 2 above"): the reassembly-time recompute-and-compare
  warning is ruled in — the cross-implementation drift tripwire
  (F-212 shape) is its primary justification.

## Converged rule set (pending only the device sub-call)

- R1: mk-codec keeps the tested guarantee — the id stays OPAQUE to
  content; NO decode-time rejection. The 19-test design fork never
  happens; `an_explicit_chunk_set_id_still_wins` survives.
- R2: recompute-and-report WARNING at reassembly in the tools,
  plain-language state+remedy per W5 (stamped vs computed, "restores
  fine, id not minted normally, id-named diagnostics may mislead").
- R3: `mk encode --chunk-set-id` warns loudly at mint time (W9).
- R4: the derivation is pinned by CROSS-LANGUAGE EXECUTABLE VECTORS
  (Rust mk-codec leads, Go port converges) — no prose formulas in the
  spec; vector rows from the first draft.
- R5 (companion, already settled by W2/W5): the grouping refusal
  messages are rewritten to distinguish missing-chunks / merged-cards
  / (with R2) non-derived-id, naming the concrete remedy for each.

Open: does the ENGRAVER (on-device scan grouping keyed by ChunkSetID)
adopt the same warning shape, and in what UI form? (fork leg,
three-repo lockstep scheduling).

## Step 6 — closing rulings

- W11 (operator, verbatim: "Same warning everywhere"): one warning
  content on every reassembly surface — mk decode/inspect/verify, md
  seating, me bundle, the engraver's scan flow.
- W12 (operator, verbatim: "Yes, on device same warning…let's treat
  it as part of post-cycle burndown along with the 'One genuine
  side-discovery from the probe, logged as a follow-up candidate'"):
  the DEVICE leg of W11 and the mk silent-correction discovery are
  scheduled as post-cycle burndown followups; they do not gate the
  cycle. me bundle stays in-cycle (unmoved by the ruling).
Walk CLOSED. Spec re-grounded on W1-W12; next gate: R0.

## Step 7 — post-GREEN wording walk (spec at e9e332b)

- W13 (moment 1, the R2 warning on mk decode, operator cold read):
  threat level read correctly ("I presume it is fine"), cause
  self-attributed to mint time ("must have made a mistake at encode
  time by giving it as test name"), action correct ("carry on").
  Remedy clause ruled inadequate — operator, verbatim: "it would be
  better to say something like recreating mk1 string without the
  —chunkset-id flag set will automatically encode a chunk set id set
  by the key data only." Fold: the clause becomes constructive
  how-to-fix ("re-mint: run mk encode again without --chunk-set-id
  and the id is derived from the key data automatically"). Gloss
  calibration: message carried without glosses for this operator
  (who has walked the vocabulary); gloss rule stays for fresh users.
