# P3 implementation report — descriptor-mnemonic seat path (warning + R5 refusal rewrite)

Worktree: `/scratch/code/shibboleth/dm-worktrees/csid-p3`, branch
`impl/csid-p3`, baseline `044e33d4`. Implements
`design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md` P3 against
`design/SPEC_chunk_set_id_verification.md` contracts 6 and 7 (and the W15/W16
wording rulings in `design/WALK_chunk_set_id_2026-08-31.md`). **Uncommitted,
unstaged** — `git status --short` in the worktree shows only the modified/new
files below; nothing staged, nothing committed.

## Files changed

```
crates/md-cli/src/seat/input.rs          | +451/-36  (763 lines total)
crates/md-cli/src/seat/mod.rs            | +8/-1     (219 lines total)
crates/md-cli/tests/seating_vectors.rs   | +82/-3    (1096 lines total)
crates/md-cli/tests/fixtures/seating/v-csid-warn.txt   NEW, 20 lines
```

7 new `#[test]` functions (5 in `seat/input.rs`, 2 in `tests/seating_vectors.rs`);
2 existing tests updated (`v_collide_two_cards_pinned_to_one_chunk_set_id_refuse_at_reassembly`
in `input.rs`, `v_collide_reaches_the_command` in `seating_vectors.rs`) plus 2
doc comments reworded (`seat/input.rs:104-114`, `tests/seating_vectors.rs:98-112`).

## Design

- **`ChunkInfo { chunk_index, total_chunks }`** (new struct) is what
  `group_key_of` now retains per string — pre-P3 it read the chunked header
  only to extract `chunk_set_id` and discarded the rest. `group_key_of`'s
  return type changed from `Result<GroupId, CliError>` to
  `Result<(GroupId, Option<ChunkInfo>), CliError>`; its only other caller
  (`group_id_of`, used by `disposition.rs`) is a one-line adapter
  (`.map(|(g, _)| g)`) so its own external signature is unchanged.
- **`classify(infos: &[ChunkInfo]) -> Option<Failure>`** answers arms 1/2
  purely from retained headers, called from `decode_cards` BEFORE
  `mk_codec::decode` for any non-empty (i.e. genuinely chunked) group:
  `Merged` on duplicate `chunk_index`, disagreeing `total_chunks`, or an
  index/count exceeding its own declared total; `Incomplete { declared_total }`
  on fewer strings than the (agreed) declared total; `None` otherwise, in
  which case `decode_cards` falls through to `mk_codec::decode` and wraps
  whatever it returns as arm 3 (`terminal_refusal`). Arm 3 carries no enum
  variant — it has no precondition of its own (SPEC r2 C3), so it is simply
  "neither of the above."
- **`derived_chunk_set_id(&KeyCard) -> Option<u32>`** is
  `mk_codec::derive_chunk_set_id(mk_codec::bytecode::encode_bytecode(card)?)`,
  fully qualified. **Both segments matter**: `derive_chunk_set_id` is
  re-exported at the mk-codec crate root (`pub use string_layer::derive_chunk_set_id`,
  `mk-codec-0.5.0/src/lib.rs:52`) but `encode_bytecode` is **not** — it is
  public only as `mk_codec::bytecode::encode_bytecode`
  (`mk-codec-0.5.0/src/bytecode/mod.rs:28`, def `bytecode/encode.rs:24`; no
  root re-export). I verified this by compiling a throwaway binary against
  the same `mk-codec = "0.5"` registry crate md-cli depends on — a bare
  `mk_codec::encode_bytecode(&card)` is `error[E0425]: cannot find function`.
  The task brief's "fully qualify `mk_codec::`" is correct as the *crate*
  qualifier; `mk_codec::bytecode::encode_bytecode` is the concrete path. This
  crate's own `cmd/encode.rs:8` imports `md_codec::chunk::derive_chunk_set_id`
  under the same bare name — confirming the footgun the brief named — so
  `seat/input.rs` never imports either function unqualified.
- **`chunk_set_id_mismatch_warning(declared, derived) -> String`** — R2/R6
  frozen wording, kept **byte-identical** to mk-cli's own function of the
  same name (P1, `/scratch/code/shibboleth/mk-worktrees/csid-p0/crates/mk-cli/src/cmd/mod.rs:133-140`,
  branch `impl/csid-p0`) and to the extension corpus's pinned `warning_text`
  field for the `SEED_pinned_12345_ef12f` row
  (`crates/mk-codec/src/test_vectors/csid_ext_v0.1.json` in that same
  worktree) — confirmed by diffing the three copies character-for-character.
  R6 requires the SAME content on every reassembly surface; md-cli and
  mk-cli share no runtime code (independent binaries), so each computes its
  own operand and carries its own copy of this string.
- **`seat_chunk_set_id_warnings(cards) -> Vec<String>`** (contract 6) —
  called from `seat::run` immediately after `decode_cards` succeeds, one
  note per mismatching `Chunked` group in `cards`'s own order (already
  ascending set-id per `decode_cards`'s contract); `Single` groups have no
  declared id and are skipped. Wired into `Seating.notes` ahead of the B1/B2
  disposition notes (`seat/mod.rs`): composition, stdout, wallet id, address
  notes and exit code are all untouched — the change is additive stderr only.

## RED → GREEN

**Genuine RED**, not a compile-error stand-in: `git stash push -- crates/md-cli/src/seat/input.rs crates/md-cli/src/seat/mod.rs`
(leaving the new/updated test files and fixture in the working tree) reverts
the binary to baseline behavior while the E2E test *source* stays new; ran,
observed failures, then `git stash pop` and reran.

```
RED  (baseline binary, new/updated E2E tests):
  FAIL v_collide_reaches_the_command
       md: ...chunk-set 12345: the 5 string(s) declaring this id do not
       reassemble into one key card: chunked-header malformed: received 5
       chunks, header declares total_chunks = 2. Two DIFFERENT cards pinned
       to one chunk-set id merge into one group here and refuse exactly
       like this — re-mint one of them so the set ids differ
  FAIL v_csid_seat_warning_fires_on_pinned_mismatch_and_composes_identically
       assertion `left == right` failed: exactly one mismatch note ...
         left: 0
        right: 1
  PASS v_csid_seat_no_warning_on_a_clean_card_set   (control row: trivially
       true against baseline too, since baseline never warns — expected for
       a negative row, not evidence either way)

GREEN (P3 binary, same tests): 3/3 pass.
```

Unit-level classifier/warning tests live in the same file as their own
implementation (`seat/input.rs`), so a source-stash can't isolate them the
same way; the mutation gates below are their RED↔GREEN evidence instead —
the stronger form for co-located test/impl per this project's own standing
practice.

## Classification-order test evidence (plan r1 I2)

`r5_classification_order_prefers_merged_over_incomplete`: two DIFFERENT
3-chunk cards (KEY 2 / KEY 3, `crates/md-cli/tests/fixtures/pathological/keys.txt`),
both pinned `--chunk-set-id 0x44444`, only each card's chunk 0 supplied.
Both raw predicates hold simultaneously: received (2) < declared (3) is
arm 2's predicate; both chunks declare `chunk_index=0` (duplicate) is arm
1's predicate. Asserted outcome: the message contains `"2 strings declare
piece 1 of 3"` (arm 1's evidence) and does **not** contain `"scan the
missing piece"` (arm 2's phrase) — i.e. it lands in the earlier arm. Passes.
Headers verified independently before pinning the fixture, via a throwaway
`mk_codec = "0.5"` reader binary against the live-minted mk1 strings:

```
...cld706hn9svfgll7zvw5qnkxgea7nkj6jsf2avy9zwj: chunk_set_id=44444 total_chunks=3 chunk_index=0
...ej0n5eghh0620cpg9jly68gp3qxjnv0ty9cpzm2edu5: chunk_set_id=44444 total_chunks=3 chunk_index=0
```

## The three refusal situations — exact messages

All fixtures were minted live with `mk encode --chunk-set-id` (`mk` 0.13.0
on PATH) from `crates/md-cli/tests/fixtures/pathological/keys.txt` KEY
material, with every chunk header re-verified via the throwaway reader
before being pinned into the test source as literals.

**1. Merged cards** — two DIFFERENT 2-chunk cards (KEY 1, KEY 5), both
pinned `0x11111`; all 4 chunks supplied (duplicate `chunk_index` 0 and 1):

> `chunk-set 11111: 2 strings declare piece 1 of 2 and 2 strings declare piece 2 of 2. A duplicated piece number is proof this chunk-set id is pinned (stamped as a fixed value rather than derived from content) to two DIFFERENT key cards, not one — each key card's mk1 strings are its chunks (pieces), and piece order does not matter. Re-scan one card's pieces alone, not mixed with any other card's pieces. Only if two cards truly show the same stamped id is a re-mint (re-encoding without --chunk-set-id) needed — check each card alone first with \`mk inspect\`.`

Also exercised on the original `v-collide.txt` fixture (card A 2-chunk /
card B 3-chunk, pinned `0x12345` — disagreeing `total_chunks` is arm 1's
second predicate), both at the unit level
(`v_collide_two_cards_pinned_to_one_chunk_set_id_refuse_at_reassembly`) and
end-to-end (`v_collide_reaches_the_command`).

**2. Incomplete scan** — one 2-chunk card (KEY 1), pinned `0x33333`, only
chunk 0 supplied:

> `chunk-set 33333: the pieces (chunks) carrying this id say there should be 2; you supplied 1 — scan the missing piece(s).`

**3. Terminal** — chunk 0 of card T1 (KEY 1) + chunk 1 of card T2 (KEY 5),
both pinned `0x22222`, both declaring `total_chunks=2` (arms 1/2's
predicates all false — this is what makes it reach arm 3, not merely that
it's the SPEC's exemplar shape):

> `chunk-set 22222: these pieces (chunks) carry one id but do not form one key card; re-scan one card's pieces alone.`
> `error: cross-chunk integrity hash mismatch`

Confirmed live (before writing the classifier) that `mk_codec::decode` on
exactly this pair returns `Error::CrossChunkHashMismatch` ("cross-chunk
integrity hash mismatch") via the same throwaway reader — matching the
spec's named exemplar verbatim.

**Contract 6 warning**, on a pinned-mismatch card that seats cleanly (KEY 1
re-minted `--chunk-set-id 0x99999`, content still derives `69f0e`; KEY 5
unchanged/natural, declared == derived == `decb1`):

> `warning: this key card's stamped chunk-set id (99999) was not derived from its content, which computes 69f0e. The card decodes fine, but diagnostics that name plates by id will call it 99999. To fix it, re-mint: run mk encode again without --chunk-set-id and the id is derived from the key data automatically.`

E2E test (`v_csid_seat_warning_fires_on_pinned_mismatch_and_composes_identically`)
additionally asserts `out_of(&warned) == out_of(&clean)` — composed
descriptor byte-identical to the unmodified V-USP twin — and that exactly
one stderr line matches, for the one mismatching group.

## Retired string — proof of removal

```
$ grep -rn "re-mint one of them\|Two DIFFERENT cards pinned" crates/md-cli/src crates/md-cli/tests
crates/md-cli/src/seat/input.rs:556:            !msg.contains("re-mint one of them"),
crates/md-cli/src/seat/input.rs:560:            !msg.contains("Two DIFFERENT cards pinned"),
crates/md-cli/tests/seating_vectors.rs:850:        !e.contains("re-mint one of them") && !e.contains("Two DIFFERENT cards pinned"),
```

**0 occurrences outside negative-assertion strings** (`!msg.contains(...)` /
`!e.contains(...)`, i.e. the grep-assertable proof the brief asked for) in
both src and tests — no production `format!`/`eprintln!` emits it, and no
test expects to see it. The two module-doc comments that historically
quoted the retired message (`seat/input.rs:104-114`,
`tests/seating_vectors.rs:98-112`) were reworded to describe the pre-P3
behavior without reproducing either substring verbatim (first draft still
had `"re-mint one of them"` inside the paraphrase; caught by re-running
this grep and fixed before considering P3 done).

## Mutation gates

**1. Contract 6 recompute** (as the brief names it — "delete the seat
recompute"): `seat_chunk_set_id_warnings` mutated to `return Vec::new();`
as its first line.

```
BEFORE (mutated):
  FAIL seat::input::tests::csid_warning_fires_on_a_pinned_mismatch_and_is_silent_on_the_clean_twin
       assertion `left == right` failed: exactly one mismatching group: []
         left: 0
        right: 1
  FAIL seating_vectors::v_csid_seat_warning_fires_on_pinned_mismatch_and_composes_identically
       (same shape: 0 matching stderr lines where 1 was expected)
AFTER (restored): both PASS; full md-cli suite 697/697.
```

**2. R5 classifier** (bonus — not explicitly required by the brief, run for
the same reason: co-located test/impl can't be RED-checked by source-stash):
`classify` mutated to `return None;` as its first line (equivalent to
"delete the classifier," falling every group through to `mk_codec::decode`
and wrapping its raw error as arm 3).

```
BEFORE (mutated), filtered to the affected rows:
  FAIL r5_merged_two_cards_pinned_to_one_id_classify_as_merged
       ...chunk-set 11111: these pieces (chunks) carry one id but do not
       form one key card; re-scan one card's pieces alone.
       error: chunked-header malformed: received 4 chunks, header declares
       total_chunks = 2
  FAIL r5_incomplete_one_of_two_chunks_classifies_as_incomplete
  FAIL r5_classification_order_prefers_merged_over_incomplete
  FAIL v_collide_two_cards_pinned_to_one_chunk_set_id_refuse_at_reassembly
  FAIL v_collide_reaches_the_command
  (5 of 6 targeted rows fail; r5_terminal_cross_chunk_hash_mismatch... is
   the one row whose EXPECTED classification already IS arm 3, so removing
   the classifier changes nothing for it — correctly unaffected, not a gap)
AFTER (restored): full md-cli suite 697/697; whole workspace 1164/1164 (2
  pre-existing, unrelated skips).
```

Both mutations restored from a pre-mutation backup and independently
re-diffed against `git diff` (clean — no residual `MUTATION-GATE` marker in
either file) before the suite was accepted as GREEN.

## Verification

- `cargo build -p md-cli --tests`: clean.
- `cargo clippy -p md-cli --all-targets`: clean (0 warnings), both before and
  after the mutation-gate restores.
- `cargo nextest run --locked -p md-cli`: **697/697 passed, 0 skipped.**
- `cargo nextest run --locked` (whole descriptor-mnemonic workspace):
  **1164/1164 passed, 2 skipped** (pre-existing `md-codec` skips, unrelated
  to this phase — same 2 both before and after this diff).
- Worktree left dirty and unstaged, as instructed: `git status --short`
  shows 3 modified files + 1 new fixture file, nothing staged.

## Deviations from the brief, with reasons

1. **V-USP/V-CSID-WARN needed `--seat '@0=<id>'`.** V-USP is deliberately
   the same fixture as V-AMB (`v_amb_the_ambiguity_refusal_reaches_the_operator_with_exit_1`,
   `tests/seating_vectors.rs:379`): both cards declare the same origin path
   at both slots, so the matching engine reports 2 candidate assignments
   and refuses without a `--seat` pin — discovered when my first draft of
   the contract-6 E2E test failed with that ambiguity refusal instead of
   seating. Fixed by pinning `--seat @0=<declared id>` identically on both
   the pinned-mismatch side (`@0=99999`) and the clean-twin comparison side
   (`@0=69f0e`), which is orthogonal to contract 6 and does not change what
   the byte-identical-output comparison proves.
2. **`Failure::Terminal` was drafted, then removed.** My first classifier
   draft declared a `Terminal` enum variant for symmetry with arms 1/2, but
   `classify` never constructs it — arm 3 has no precondition (SPEC r2 C3),
   so it is reached by `decode_cards` falling through to `mk_codec::decode`
   directly, not by a classifier return value. Kept as a 2-variant enum
   (`Merged`, `Incomplete`) plus a doc comment explaining arm 3's absence,
   to avoid a never-constructed variant.
3. **Doc-comment churn beyond the enumerated line numbers.** The brief's
   citations (`seat/input.rs:203-211`, `:1-25`, `:106`;
   `tests/seating_vectors.rs:845-846`, `:107`) were baseline line numbers;
   my P0-recon read confirmed content rather than trusting numbers, and one
   passage (`seat/input.rs` lines ~12-16, the module-top doc's "refuse at
   reassembly" claim) needed updating for a reason beyond quoting the
   retired string: it asserted collisions are caught "at reassembly," which
   is no longer precise now that arms 1/2 refuse *before* `mk_codec::decode`
   is ever called. Updated to describe the classifier-first behavior.

## Observation (not this phase's to fix, recorded for the record)

P1's persisted report (`design/agent-reports/impl-csid-p1.md`, committed to
mnemonic-key `main` at `d7a6426`) states the worktree was "Uncommitted,
unstaged per instruction." The actual worktree
(`/scratch/code/shibboleth/mk-worktrees/csid-p0`) has 2 real commits on
`impl/csid-p0` (`37a9524` "feat(mk-cli): P1", `1711228` "feat(mk-cli): P2")
and a clean `git status`. I relied on this worktree only as a **read-only**
source for the frozen R2/R6 wording (byte-diffed against the corpus's own
pinned `warning_text`, not taken on the report's word), so it does not
affect P3's own correctness — flagging the discrepancy since it's outside
what I was asked to fix.

## Acceptance mapping (SPEC contracts 6 + 7, plan P3)

- Contract 6 (seat warning): DONE — `seat_chunk_set_id_warnings`, wired
  into `seat::run`, golden E2E row + unit row, mutation-gated.
- Contract 7 (R5 refusal rewrite): DONE — 3-arm classifier, total by
  construction over chunked-group failures; all 4 situations (3 refusal
  arms + the contract-6 success arm) covered by at least one vector;
  classification-order row proves arm precedence; mutation-gated.
- Retired string: 0 production/test occurrences (grep-verified above).
- `mk_codec::`-qualified derivation: verified against the live registry
  crate, not assumed from the task brief.
- No mk-codec or md-codec changes; no exit-code, stdout, wallet-id or
  address-note changes (asserted directly in the E2E test).
