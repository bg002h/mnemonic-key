# SPEC — chunk_set_id: recompute-and-report (warnings, vectors, no admission change)

**Status: DRAFT for R0. Re-grounded 2026-08-31** from the operator walk
(`design/WALK_chunk_set_id_2026-08-31.md`, rulings W1–W10) and the recon
(`design/agent-reports/recon-chunk-set-id-spec-2026-08-31.md`). Supersedes
the 2026-08-19 parked draft (git `9fbbe36`) whose central question — reject
non-derived ids at decode time? — the operator has now RULED. Baselines:
mnemonic-key `9ff8922`, descriptor-mnemonic `7eca44b6`, seedhammer
`5f02773c`, mnemonic-engrave `1103d9ee`.

## The ruling (operator, 2026-08-31, walk steps 4–5)

Position A stands: **the chunk-set id remains opaque to content in
mk-codec. No decode-time rejection, ever, in this cycle.** The named
guarantee (`an_explicit_chunk_set_id_still_wins`, "the id is opaque to
content") survives untouched, and the 19-test design fork in the old
draft never happens. Instead:

- **R2 — reassembly warning:** tools that reassemble a chunked card
  recompute the id from content and WARN on stamped ≠ computed. Exit
  codes and stdout are unchanged.
- **R3 — mint warning:** `mk encode --chunk-set-id` warns loudly on
  stderr. Measured today: it mints in total silence.
- **R4 — cross-language executable vectors** pin the derivation so
  Rust, Go, and the device can never drift apart silently (operator,
  verbatim endorsement). No prose formula appears in this spec.
- **R5 — grouping-refusal rewrite** in the md seat path: three
  distinguishable situations get three messages with three remedies.
- **R6 — one warning, everywhere** (operator, verbatim: "Same warning
  everywhere"): the same warning content renders on every surface that
  reassembles a card — mk, md seating, me bundle, and the engraver's
  on-device scan flow.

Why warnings and not refusals (measured, walk step 4): a mismatch
cannot arise from engraving damage — one flipped character is silently
BCH-corrected (`mk decode` returned the intact card; `mk repair` named
the correction), and an id wrong on only some chunks splits the group
and refuses as incomplete before any derivation could be consulted. A
clean mismatch is only ever MINTED: (1) `--chunk-set-id` leakage,
(2) encoder drift between implementations (the F-212 shape — the
warning is a standing field tripwire), (3) beyond-budget miscorrection
(rare; payload damage is caught by the existing 4-byte cross-chunk
hash), (4) deliberate tamper. None of these are the scanning
operator's fault; refusal would strand a perfectly restorable wallet.

**Deliberate cross-format asymmetry, stated as fact:** md-codec
already verifies its csid unconditionally and refuses on mismatch
(`descriptor-mnemonic crates/md-codec/src/chunk.rs` — "the content-id
oracle; P0.2 funds-load-bearing invariant", present since 2026-04).
Policy cards refuse; key cards warn. This spec does not touch
md-codec.

## Behavior contracts, per surface

Warning text is normative-by-vector: the exact strings live in the
acceptance rows below, not in prose. Drafts here are R0-reviewable
wording, frozen only when the rows are.

### mnemonic-key (Rust leads)

1. **mk-codec:** expose the comparison non-breakingly — decode of a
   chunked set also reports (declared_csid, derived_csid) to callers.
   No error variant is added to the decode path; admission semantics
   and every existing test are unchanged. (Mechanism — new API vs.
   enriched return — is the implementer's choice at plan time.)
2. **`mk decode` / `mk inspect` (chunked input only):** on
   declared ≠ derived, stderr gains one warning; exit stays 0:
   > `warning: this card's stamped chunk-set id (0x12345) does not match the id computed from its content (0xef12f). The card decodes fine, but it was not minted normally, and tools that name plates by id will call it 12345 — expect confusing diagnostics until it is re-minted.`
3. **`mk verify`:** same detection, reported in verify's own output
   format. Still non-fatal. No `--strict` mode this cycle.
4. **`mk encode --chunk-set-id`:** stderr warning at mint:
   > `warning: --chunk-set-id pins 0x12345 in place of the content-derived id (0xef12f). Cards minted this way trip a mismatch warning in every conforming decoder, forever. For test fixtures only — never engrave this on a real plate.`
   (Requires computing the derived id at mint; it is already computed
   on the unpinned path.)

### descriptor-mnemonic (seat path, `crates/md-cli/src/seat/`)

5. **After successful group reassembly** in `md descriptor` /
   `md address` seating: recompute; on mismatch add one stderr note in
   the R2 shape. Composition, stdout, wallet id, address notes, exit
   code: unchanged (measured baseline: today this seats silently).
6. **R5 — the refusal rewrite.** The current message ("Two DIFFERENT
   cards pinned to one chunk-set id … re-mint one of them") was
   measured firing on three distinct situations; it is replaced by
   header forensics the tool already possesses:
   - received < declared, no duplicate chunk indexes → *incomplete
     scan*: name the missing piece(s) ("this card says it has 2
     pieces; you supplied 1 — scan the missing piece").
   - duplicate chunk indexes, or chunks disagreeing on total_chunks →
     *merged cards*: "these strings are pieces of two different cards
     that share one stamped id — separate the scans; only if both
     plates truly carry the same id, re-engrave one."
   - group reassembles but derived ≠ declared → the R2 warning (not a
     refusal).
   Terms are defined at point of use (W1): "card", "chunk", "stamped
   id" appear with one-clause glosses the first time each message uses
   them.

### seedhammer fork (Go, strictly downstream)

7. **R4 convergence (in cycle):** the Go `mk/` port consumes the
   same vector corpus and must reproduce every derived id. **R6 on
   the device — post-cycle burndown (operator scheduling ruling):**
   the scan flow (grouping keyed by `ChunkSetID`) will surface the
   same warning content when a completed, reassembled set has
   declared ≠ derived — content parity with contract 2, fork-native
   UI form. Filed as FOLLOWUPS `device-csid-mismatch-warning`; does
   not gate this cycle (R1 means no normative codec change
   locksteps, so the vectors are the in-cycle fork surface).

### mnemonic-engrave (me-cli)

8. `me bundle` emits the same warning (R6) when a bundled chunked
   group's declared id differs from the recomputed one — engrave leg,
   this cycle.

## Vectors (R4) — the derivation is pinned by rows, not prose

The corpus extends mk-codec's SHA-pinned vector corpus (`mk vectors`)
with, per chunked vector: the canonical payload (hex), the mk1 string
set, `derived_csid`, `declared_csid`, and `expect_mismatch_warning`.
Conformance = reproducing every row; both the Rust suite and the fork
Go suite consume the same JSON. Seed rows, measured live this session
(policy `md15p8dsssfdsssj5qqcyxppgtcfh4dhmh72lmcq638s7y9fyu5u6s`,
fixture keys per the walk record):

| card | declared | derived | warn |
| --- | --- | --- | --- |
| plate A (deadbeef, m/48'/0'/0') | 1b1ba | 1b1ba | no |
| plate B (cafef00d, m/48'/0'/0'/2') | ef12f | ef12f | no |
| plate B pinned `--chunk-set-id 0x12345` | 12345 | ef12f | **yes** |

Corpus regeneration happens at implementation time via `mk vectors`;
the three rows above are the executable anchor this spec is checked
against.

## Acceptance

- Golden-stderr rows for contracts 2–6: each warning fires byte-exact
  on its vector and is ABSENT on the unpinned twins. A warning that
  cannot fire is the false-PASS class: mutation gate — removing the
  recompute comparison must fail the pinned rows.
- Contract 6's three refusal situations each have a vector row (we
  measured all three shapes live: 4-strings-one-id, 1-of-2, mixed-id
  chunks); the retired message string appears in none of them.
- `cargo nextest run --locked` green in mnemonic-key and
  descriptor-mnemonic with zero changes to existing csid tests; fork
  `mk/` Go tests consume the extended corpus and pass.
- R6 parity: `me bundle`'s warning carries the same content as
  contract 2's — rendering may differ; the (declared, derived) pair
  and the remedy sentence may not. Firing row + absent-on-clean row.
  The device leg asserts the same parity when its post-cycle
  followup lands.
- Existing guarantees re-asserted, not weakened:
  `an_explicit_chunk_set_id_still_wins` and
  `canonical_payload_is_chunk_set_id_invariant` unchanged.

## Not in scope

- md-codec (already ships its check; asymmetry is deliberate, above).
- Device warning surface: ruled same-content (R6), scheduled
  post-cycle burndown — FOLLOWUPS `device-csid-mismatch-warning`.
- `mk decode` silent BCH auto-correction reporting: walk discovery,
  post-cycle burndown — FOLLOWUPS
  `mk-decode-silent-correction-reporting`.
- Any `--strict`/refusal mode; any change to `Error::ChunkSetIdMismatch`
  naming (the mk-vs-md naming collision is recorded in the recon;
  resolving it is a docs/API question for a future major).
