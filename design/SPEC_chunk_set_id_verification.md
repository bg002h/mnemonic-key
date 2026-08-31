# SPEC — chunk_set_id: recompute-and-report (warnings, vectors, no admission change)

**Status: DRAFT for R0 r2. Re-grounded 2026-08-31** from the operator walk
(`design/WALK_chunk_set_id_2026-08-31.md`, rulings W1–W12) and the recon
(`design/agent-reports/recon-chunk-set-id-spec-2026-08-31.md`); folded once
against `design/agent-reports/R0-csid-spec-r1.md` (4C/6I/5M/3N). Supersedes
the 2026-08-19 parked draft (git `9fbbe36`). Baselines: mnemonic-key
`9ff8922`, descriptor-mnemonic `7eca44b6`, seedhammer `5f02773c`,
mnemonic-engrave `1103d9ee`.

## The ruling (operator, 2026-08-31, walk steps 4–6)

Position A stands: **the chunk-set id remains opaque to content in
mk-codec. No decode-time rejection, ever, in this cycle.** The named
guarantee (`an_explicit_chunk_set_id_still_wins`, "the id is opaque to
content") survives untouched. Instead:

- **R2 — reassembly warning:** every surface that reassembles a chunked
  card recomputes the id from content and WARNS on stamped ≠ computed.
  Exit codes and stdout contracts are unchanged.
- **R3 — mint warning:** `mk encode --chunk-set-id` warns loudly on
  stderr. Measured today: it mints in total silence.
- **R4 — cross-language executable vectors** pin the derivation so
  Rust, Go, and the device can never drift apart silently (operator,
  verbatim endorsement). No prose formula appears in this spec.
- **R5 — grouping-refusal rewrite** in the md seat path: an ordered,
  total classification replaces the one-message-fits-all refusal.
- **R6 — one warning, everywhere** (operator, verbatim: "Same warning
  everywhere"): the same warning CONTENT on every reassembly surface.
  Rendering is per-surface; the content — the (declared, derived) pair
  and the remedy sentence — is not negotiable.
- **W12 scheduling:** the device leg, the me-cli leg (r1 C2), the Go
  corpus-ingestion mechanism (r1 I4) and mk's silent-correction
  reporting are POST-CYCLE BURNDOWN followups, enumerated in Not-in-
  scope. In-cycle surfaces: mk-cli, md-cli, and the Go derivation
  parity assertion.

## Why warnings and not refusals (measured, walk step 4)

A mismatch cannot arise from engraving damage — one flipped character
is silently BCH-corrected (`mk decode` returned the intact card;
`mk repair` named the correction), and an id wrong on only some chunks
splits the group and refuses as incomplete before any derivation could
be consulted. A clean mismatch is only ever MINTED:

- **(0) vintage, stated as fact:** mk-codec < 0.5.0 drew the id from
  the system CSPRNG (measured: 0.4.1 `fresh_chunk_set_id`); derivation
  arrived at 0.5.0 (2026-08-14). Pre-0.5 mints mismatch with
  probability ≈ 1. No such plates exist beyond test artifacts —
  operator rulings 2026-08-19 ("We don't care about old cards. None
  exist.") and 2026-08-30 ("there are no engraved plates besides test
  plates"). In-repo fixtures of this vintage DO exist (the corpus's 19
  chunked vectors, the fork's parityVectors, me-cli's golden) and are
  handled as pinned-by-design below.
- (1) `--chunk-set-id` leakage into a real mint;
- (2) encoder drift between implementations — the F-212 shape; the
  warning is a standing field tripwire;
- (3) beyond-budget miscorrection (rare; payload damage is largely
  caught by the existing 4-byte cross-chunk hash);
- (4) deliberate tamper.

None of these are the scanning operator's fault; refusal would strand
a perfectly restorable wallet. **Deliberate cross-format asymmetry,
stated as fact:** md-codec already verifies its csid unconditionally
and refuses on mismatch ("the content-id oracle; P0.2 funds-load-
bearing invariant", since 2026-04). Policy cards refuse; key cards
warn. This spec does not touch md-codec.

## The comparison (r1 I3 — the operand is pinned)

`derived = derive_chunk_set_id(encode_bytecode(decoded_card))` — the
CANONICAL RE-ENCODE of the successfully decoded card, not the raw
reassembled bytes. Chosen deliberately: a foreign encoder whose
bytecode canonicalization drifts stamps an id consistent with its own
bytes; only the re-encode route detects that drift (the F-212 channel
this warning exists for). A card whose payload does not re-encode to
the bytes on the plates is therefore IN scope for the warning. Both
functions are public mk-codec 0.5 API, so **no mk-codec change is
required this cycle** (r1 C2's publish pressure dissolves); every
in-cycle surface computes the same operand, which is what makes R6
parity testable.

## Behavior contracts, per surface

The normative warning content lives in the vector corpus's
`warning_text` field (below), not in this prose; drafts here are
R0-reviewable wording, frozen when the rows are. One rendering rule
(r1 N2): the id appears as bare lowercase hex (e.g. `12345`, `ef12f`),
matching the existing "chunk-set 12345" diagnostic surface. Two rules
from the wording walk bind every refusal and warning (W16): the HUMAN
sentence leads and the machine diagnostic follows on its own labeled
line (operator's shape: `error: <codec sentence>`; label frozen in the
vector rows) — and messages count CARDS, never plates (the tool cannot
know how many physical plates the pieces came from).

### mnemonic-key (in cycle)

1. **mk-codec: unchanged.** No new API, no error variant, no semver
   event. All surfaces use the public pair named above.
2. **All six mk1-consuming verbs** — decode, inspect, verify, repair,
   derive, address — share the intake `cmd/mod.rs::read_mk1_strings`;
   the recompute seats at that chokepoint (or equivalently at each
   verb's decode call) so "every mk surface" is structural, not an
   enumeration that decays (r1 C1). On declared ≠ derived, chunked
   input only: one stderr warning, exit unchanged. Draft:
   > `warning: this key card's stamped chunk-set id (12345) was not derived from its content, which computes ef12f. The card decodes fine, but diagnostics that name plates by id will call it 12345. To fix it, re-mint: run mk encode again without --chunk-set-id and the id is derived from the key data automatically.` (Wording per operator walk W13.)
   `mk repair`'s warning fires on its re-verified (blessed) output;
   the repair report itself is unchanged.
3. **`mk inspect` additionally prints the stamped chunk-set id
   unconditionally** (r1 M4) — matched or not — so the warning's value
   has a cross-check surface.
4. **`mk verify`:** text mode reports the mismatch in verify's own
   stdout format (content: the pair + remedy). `--json` gains an
   optional additive field
   `"chunk_set_id": {"declared": "12345", "derived": "ef12f", "matches": false}`;
   `schema_version` stays 1 (additive field, absent for single-string
   input). Still non-fatal. No `--strict` this cycle.
5. **`mk encode --chunk-set-id`:** stderr warning at mint. Draft:
   > `warning: --chunk-set-id pins 12345 in place of the content-derived id ef12f. Cards minted this way trip a mismatch warning in every conforming decoder, forever. For test fixtures only — never engrave this on a real plate. To mint a real plate, drop --chunk-set-id entirely and the id is derived from the key data automatically. Do not re-type the derived value into the flag: one mistyped character mints a mismatched plate.` (Constructive clause per W13; anti-transcription clause per W14 — the operator produced a live transposition, ef21f for ef12f, in the walk itself.)
   mk-cli computes the derived value itself via the public pair (the
   codec's pinned arm skips derivation — r1 M5; small, real work).

### descriptor-mnemonic (in cycle)

6. **After successful group reassembly** in `md descriptor` /
   `md address` seating (the only two `--from-mk1` verbs, verified
   r1): recompute per The Comparison; on mismatch add one stderr note
   with the same content. Composition, stdout, wallet id, address
   notes, exit code: unchanged.
7. **R5 — the refusal rewrite.** The seat path retains per-chunk
   headers (today `group_key_of` discards them — r1 C3). First fork:
   does the group reassemble AND bytecode-decode cleanly? If YES →
   situation 4. If NO → the first matching arm of 1–3, where arm 3
   has NO precondition, so the classification is total by
   construction (r2 C3):
   1. **duplicate chunk index, chunks disagreeing on total_chunks, or
      more chunks than any declared total (received > declared, or
      any chunk_index ≥ its declared total)** → *merged cards*. The
      message MUST carry (W15): the per-string evidence the tool
      already holds (each string's declared piece number and total —
      "two strings declare piece 1 of 2 and two declare piece 2 of 2";
      a duplicate piece number is proof of two cards), a note that
      piece order does not matter, the remedy "re-scan one card's
      pieces alone" WITHOUT asserting a plate count (W16: the tool
      counts cards from headers; a card's pieces may span plates, so
      any plate count is a guess the tool must not print), and the
      id-check named as a command ("only if two cards truly show the
      same stamped id — check each alone with mk inspect — re-engrave
      one of them"). Draft rendering is the implementer's; the four
      elements are normative and frozen by the vector rows.
   2. **received < declared** (no duplicates, totals agree) →
      *incomplete scan*: "the pieces carrying this id say there should
      be N; you supplied K — scan the missing piece(s)." (Wording
      avoids asserting a single card — r1 M3.)
   3. **terminal otherwise — every remaining failure, no
      precondition** (measured exemplar: chunk 1 of plate A + chunk 2
      of plate B, both pinned 12345 → `cross-chunk integrity hash
      mismatch`; also covers `MixedHeaderTypes` and post-reassembly
      bytecode decode errors): the neutral remedy leads — "these
      pieces carry one id but do not form one key card; re-scan one
      card's pieces alone" — and the codec error follows verbatim on
      its own labeled line.
   4. **group reassembles cleanly but derived ≠ declared** → the R2
      warning (contract 6), not a refusal.
   The retired message ("Two DIFFERENT cards pinned … re-mint one of
   them") appears nowhere. Every message glosses, at first use: "key
   card", "chunk", "chunk-set id", "re-mint", "pinned" (r1 N3).
   **Sites this rewrite touches, enumerated** (r1 I2 — expected test/
   doc churn, not forbidden churn): `seat/input.rs:310-313` assert,
   `tests/seating_vectors.rs:845-846` asserts
   (`v_collide_reaches_the_command`), module doc `seat/input.rs:1-25`
   and `:106`, doc comment `tests/seating_vectors.rs:107`.

### seedhammer fork (in cycle: derivation parity only)

8. The Go encoder already derives correctly (r1 sound-list:
   `mk/encode.go` `top20`, "NO CSPRNG"). In-cycle obligation: a Go
   unit test asserting the derivation reproduces the PINNED PARITY
   ROWS (the clean rows of the extension corpus, hand-carried like the
   existing parityVectors). Full JSON corpus ingestion is a real
   vendoring/seam mechanism the fork does not have (r1 I4) — filed
   post-cycle (Not-in-scope).

## Vectors (R4) — the derivation is pinned by rows, not prose

**The v0.1 corpus is untouched: its 19 chunked vectors all carry
pre-0.5 pinned ids (measured: 19/19 mismatch, e.g. V1 declared
`12345`, derived `83bb2`) and become the pinned-by-design MISMATCH
half.** A NEW extension corpus file (own SHA pin, own schema field
set) supplies the CLEAN half plus warning content: per row —
`canonical_bytecode_hex`, the mk1 string set, `declared_csid`,
`derived_csid`, `expect_mismatch_warning`, and `warning_text` (the
normative content — r1 I1). It is generated by
`cargo run --bin gen_mk_vectors --features gen-vectors` (extended;
`mk vectors` is a read-only printer — r1 M1), and includes at minimum:
clean twins of three legacy shapes, plus this walk's three seed cards
(plate A `1b1ba/1b1ba`, plate B `ef12f/ef12f`, pinned `12345/ef12f`).
Rust and Go conformance = reproducing every row's `derived_csid`; the
in-cycle Go assertion covers the clean rows (contract 8). The open
FOLLOWUPS V19-re-pin nit is unaffected (no v0.1 churn here).

## Acceptance

- **Per-surface golden rows** (all six mk verbs of contract 2, verify's
  two modes, mint contract 5, seat contract 6): the warning fires on
  the mismatch rows and is ABSENT on their clean twins. Warning
  content asserts the corpus row's `warning_text` — the (declared,
  derived) pair and remedy sentence must appear; surface framing may
  differ (R6 parity, testable because the operand is pinned).
- **Contract 7 rows:** each of the four situations has at least one
  vector (all four measured live this session, including the
  mixed-halves cross-chunk case); the retired message string appears
  in none; classification order is asserted by a row that satisfies
  two situations' raw predicates and must land in the earlier one.
- **Per-surface mutation gates** (r1 M2, r2 M2): deleting the
  recompute in mk-cli, separately in md-cli, and perturbing the Go
  derivation under test, each fails that surface's rows — with
  evidence the mutated line RAN, not merely landed.
- `cargo nextest run --locked` green in mnemonic-key and
  descriptor-mnemonic; fork `go test ./mk/` green including the new
  derivation-parity test. (me-cli joins when its followup lands.)
- **The two named guarantees are byte-unchanged** (r1 I2 scoping —
  this clause covers ONLY these): `an_explicit_chunk_set_id_still_wins`
  and `canonical_payload_is_chunk_set_id_invariant`. All other test
  churn is expected where enumerated (contract 7's site list, the new
  extension corpus and its consumers).

## Not in scope (all filed, all post-cycle burndown per W12)

- **me-cli leg** (`me bundle`/`seal`/`sysw` warnings): blocked on a
  semver-breaking mk-codec 0.4→0.5+ bump and an operator-gated publish
  (F-424 class) — FOLLOWUPS (mnemonic-engrave)
  `me-cli-csid-warning-surface` (r1 C1/C2).
- **Device warning surface** (R6 content parity on the engraver) —
  FOLLOWUPS `device-csid-mismatch-warning`.
- **Go JSON corpus ingestion** (vendored seam, F-425 pattern) —
  FOLLOWUPS (mnemonic-engrave) `go-mk-vector-corpus-ingestion` (r1 I4).
- **`mk decode` silent BCH auto-correction reporting** — FOLLOWUPS
  `mk-decode-silent-correction-reporting`.
- md-codec (already ships its check; asymmetry deliberate, above).
- Any `--strict`/refusal mode; resolving the mk-vs-md
  `ChunkSetIdMismatch` naming collision (recorded in the recon; a
  docs/API question for a future major).
