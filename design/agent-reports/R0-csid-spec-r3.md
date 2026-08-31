# R0 review r3 — fold verification of `design/SPEC_chunk_set_id_verification.md` @ `e9e332b`

**Verdict: 0 Critical / 0 Important — GREEN.**

Scope per brief: exactly two questions, over the fold at `e9e332b` (contract
7's classification + the mutation-gate bullet only). r2's other 13 findings
are taken as settled, not re-derived.

---

## Q1 — is contract 7's classification now genuinely disjoint and total?

**Disposition: C3 FIXED.**

Structure as folded: first fork — does the group reassemble AND
bytecode-decode cleanly? YES → situation 4. NO → arms 1→2→3 in order, arm 3
carrying literally "no precondition." A no-precondition final arm makes
totality unconditional by construction: whatever arm 1 and arm 2 do not
claim, arm 3 claims, so no input can fall through. Evaluation is strict
if/elif/else, so exactly one arm fires per input regardless of whether raw
predicates overlap — which the spec's own Acceptance clause already treats
as by-design ("a row that satisfies two situations' raw predicates... must
land in the earlier one"), not a defect to eliminate.

**r2's counterexample now lands in arm 1, confirmed.** r2's construction: 3
chunks sharing one id, all declaring `total_chunks=2`, distinct indices
`0,1,2`. Under the folded wording: no duplicate index (0,1,2 distinct); no
total disagreement (all declare 2); but `received (3) > declared (2)` —
arm 1's newly-added "more chunks than any declared total (received >
declared...)" disjunct fires directly. Lands in arm 1, not ambiguous with
arm 2 (arm 2's own precondition is `received < declared`, which is false
here) or arm 3 (arm 1 is checked and matches first).

**Constructed counterexamples tried, per the brief's hint list — none broke
totality or disjointness:**

- *received > declared with duplicates* (e.g. two distinct strings both
  claim index 0, total 1): arm 1 fires on "duplicate chunk_index" alone; no
  ambiguity with arm 2 (received=2, not < declared=1).
- *single chunk declaring `total_chunks=1` that fails bytecode decode*: no
  duplicate (only one chunk), no disagreement (only one value), received
  (1) not > declared (1), not < declared (1) — arm 1 and arm 2 both false,
  falls to arm 3 by its "no precondition" catch, which is exactly the
  intended semantics (arm 3's exemplar text explicitly names
  "post-reassembly bytecode decode errors").
- *zero-chunk groups*: **not representable.** Groups are formed in
  `descriptor-mnemonic/crates/md-cli/src/seat/input.rs::decode_cards` via
  `groups.entry(key).or_default().push(s.clone())` inside a loop over
  actual input strings — a `GroupId` entry cannot exist in the map without
  at least one string having produced it. No zero-member group can reach
  the classification.
- *totals disagreeing AND received < min(total)*: arm 1 fires on "chunks
  disagreeing on total_chunks" before arm 2 is ever evaluated; arm 2's own
  stated precondition ("no duplicates, totals agree") is false here by
  construction, so there is no live ambiguity — order plus arm 2's
  self-limiting precondition make this non-contentious.

**Secondary, non-blocking finding (does not affect the Q1 verdict).**
Traced the actual gating mechanically: `StringLayerHeader::from_5bit_symbols`
(`crates/mk-codec/src/string_layer/header.rs:160-164`) rejects any single
chunk whose own `chunk_index >= total_chunks` **at that chunk's individual
header-parse step**, and this function is the *only* header parser used both
by `group_key_of` (`descriptor-mnemonic crates/md-cli/src/seat/
input.rs:143-158`, called during grouping) and by `pipeline::decode`
(`crates/mk-codec/src/string_layer/pipeline.rs:105-135`, called both for
grouping-peek and for reassembly). Consequence: no chunk with an
out-of-range index for its own declared total can ever survive to become
part of "a group" that reaches contract 7's classification at all — it is
rejected earlier, outside contract 7's three messages. Chasing this further:
given all-chunks-agree-on-T and no-duplicate-indices (i.e., arm 1's other two
disjuncts both false), pigeonhole forces `received <= T`, so `received >
declared` is *also* unreachable via real mk1-string input without already
tripping "duplicate index" or "disagreeing totals." `chunk.rs:248-252`'s
`idx >= total_usize` branch, which r2 cited as live evidence, is therefore
dead in the external-input pipeline — reachable only via the
`#[cfg(test)]`-only direct-field construction at `chunk.rs:390`. Net effect:
the two clauses the fold added to arm 1 are harmless and sufficient for
totality/disjointness as *stated*, but appear to do no independent
classification work against real card input — the pre-fold "duplicate index
/ totals disagree" pair already covered every reachable case in that
sub-space. Not a totality/disjointness violation (arm 3's unconditional
catch-all still guarantees total either way); recording for the record, not
gating.

---

## Q2 — does the mutation clause now cover mk-cli, md-cli, AND the Go surface?

**Disposition: M2 FIXED.**

Folded text: "deleting the recompute in mk-cli, separately in md-cli, and
perturbing the Go derivation under test, each fails that surface's rows —
with evidence the mutated line RAN, not merely landed." All three in-cycle
surfaces named in the spec's own scoping line ("In-cycle surfaces: mk-cli,
md-cli, and the Go derivation parity assertion") now appear by name, each
with an explicit mutation action (delete the recompute / perturb the
derivation) and the shared "mutated line RAN" evidence bar applying to all
three via "each." This matches contract 8's Go unit test (asserting
derivation reproduces the pinned parity rows) as the thing to perturb.
Nothing left uncovered.

---

## What was machine-checked this session

- `groups.entry(key).or_default().push(...)` in `decode_cards`
  (descriptor-mnemonic `crates/md-cli/src/seat/input.rs`) — zero-chunk
  groups confirmed unrepresentable.
- `StringLayerHeader::from_5bit_symbols` (`crates/mk-codec/src/
  string_layer/header.rs:121-172`) — confirmed as the sole per-chunk header
  parser, confirmed it rejects `chunk_index >= total_chunks` per-chunk
  before any group forms.
- `group_key_of` (descriptor-mnemonic `crates/md-cli/src/seat/
  input.rs:143-158`) and `pipeline::decode`
  (`crates/mk-codec/src/string_layer/pipeline.rs:105-135`) — both confirmed
  to call `from_5bit_symbols` before constructing any `ChunkFragment`,
  closing the reachability question for the secondary finding above.
- `chunk.rs:248-252`'s duplicate/disagreement/gap checks
  (`crates/mk-codec/src/string_layer/chunk.rs:185-280`) read in full to
  confirm the disagreement check (`total_chunks != total`) always runs
  before the `idx >= total_usize` check on the same chunk.

## Lens closure

Both questions asked have definite answers; no further construction found a
counterexample. Closing this round GREEN.
