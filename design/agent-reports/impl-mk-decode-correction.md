# Implementation report: `mk-decode-silent-correction-reporting`

**Worktree:** `/scratch/code/shibboleth/mk-worktrees/decode-corr`
**Branch:** `followup/mk-decode-correction` (base `ad0accc`)
**Status:** implemented, TDD, all gates green. Worktree left DIRTY/UNSTAGED per
instruction — nothing committed by this agent.

## What changed

Non-risk-set diagnostics addition: `mk decode` and `mk verify` now emit a
non-fatal stderr note naming per-chunk BCH correction counts when correction
fired during decode. Exit codes and stdout are byte-for-byte unchanged;
`--json` was left untouched (see "Deviations" below).

### Files + lines

- `crates/mk-cli/src/cmd/mod.rs` — new shared helpers, added after
  `warn_chunk_set_id_mismatch` (lines 155–206):
  - `BCH_CORRECTION_CEILING: usize = 4` (line 160)
  - `pub fn correction_counts(strings: &[String]) -> Vec<(usize, usize)>`
    (line 177) — re-runs `mk_codec::string_layer::decode_string` per raw
    input string, returns `(chunk_index, corrections_applied)` for every
    chunk with `corrections_applied > 0`, in input order.
  - `pub fn warn_corrections_applied(counts: &[(usize, usize)])` (line 193)
    — no-op on empty input; otherwise prints the note below to stderr.
- `crates/mk-cli/src/cmd/decode.rs` — import extended (lines 14–17), call
  site added at line 49, directly after the existing R2
  `warn_chunk_set_id_mismatch` call (line 45), same placement pattern.
- `crates/mk-cli/src/cmd/verify.rs` — import extended (lines 16–20), call
  site added at line 75, directly after the existing R2 call (line 71).
- `crates/mk-cli/tests/decode_verify_correction_note.rs` — new file, 2
  integration tests (RED→GREEN below).

### Exact note text (format string, `mod.rs` lines 200–205)

```
note: BCH error-correction repaired this card while decoding -- {breakdown}. The card \
 is intact, but a plate consuming its correction budget (max 4 per chunk) is \
 degrading; run `mk repair` for the per-position detail.
```

where `{breakdown}` is a comma-joined `chunk {i}: {n} correction(s)` list,
e.g. for the test fixture:

```
note: BCH error-correction repaired this card while decoding -- chunk 0: 2 correction(s), chunk 1: 1 correction(s). The card is intact, but a plate consuming its correction budget (max 4 per chunk) is degrading; run `mk repair` for the per-position detail.
```

## TDD evidence

Fixture (`decode_verify_correction_note.rs`): the canonical V1 KeyCard
(same fixture as `cli_repair.rs`, minted via `mk_codec::encode` directly,
2 chunks — chunk 0 long code, chunk 1 regular code). `flip_at`/`flip_many`
apply single-symbol substitutions: chunk 0 gets 2 flips (positions 20, 50),
chunk 1 gets 1 flip (position 20) — all well within `t <= 4` per chunk, so
the damaged set still decodes and reassembles to the identical card.

**RED** (test file written, source unchanged — `cargo nextest run --locked
-p mk-cli --test decode_verify_correction_note`):

```
FAIL decode_damaged_input_warns_with_per_chunk_counts_same_card
  stderr must name chunk 0's count; stderr="note: stdout is watch-only — public keys only, cannot spend\n"
FAIL verify_damaged_input_warns_with_per_chunk_counts_exit_unchanged
  stderr=""
2 tests run: 0 passed, 2 failed
```

This also confirmed the fixture itself was sound pre-fix: both runs
exited 0 (in-budget correction succeeds), just with no note — exactly the
silent-correction defect the followup names.

**GREEN** (after the `mod.rs`/`decode.rs`/`verify.rs` edits):

```
PASS verify_damaged_input_warns_with_per_chunk_counts_exit_unchanged
PASS decode_damaged_input_warns_with_per_chunk_counts_same_card
2 tests run: 2 passed, 0 skipped
```

Both tests assert: exit 0 unchanged, damaged-vs-clean stdout byte-identical
(same decoded card / same verify verdict), the note contains
`chunk 0: 2 correction(s)`, `chunk 1: 1 correction(s)`, `max 4 per chunk`,
and `mk repair`; and the CLEAN (undamaged) twin's stderr never contains
`"BCH error-correction repaired"` — silent on clean input, both verbs.

## Mutation gate

Mutated `correction_counts` in `crates/mk-cli/src/cmd/mod.rs` to:

```rust
pub fn correction_counts(_strings: &[String]) -> Vec<(usize, usize)> {
    Vec::new() // MUTATION: always report no corrections
}
```

**Before restore (mutated):**

```
FAIL decode_damaged_input_warns_with_per_chunk_counts_same_card
  stderr must name chunk 0's count; stderr="note: stdout is watch-only — public keys only, cannot spend\n"
FAIL verify_damaged_input_warns_with_per_chunk_counts_exit_unchanged
  stderr=""
2 tests run: 0 passed, 2 failed
```

**After restore** (`cp` of the pre-mutation file back into place):

```
PASS verify_damaged_input_warns_with_per_chunk_counts_exit_unchanged
PASS decode_damaged_input_warns_with_per_chunk_counts_same_card
2 tests run: 2 passed, 0 skipped
```

Confirms the note-presence assertions actually depend on
`correction_counts` computing real values, not just on some other
incidental stderr content.

## Full validation surface

- `cargo nextest run --locked -p mk-cli -p mk-codec`: **389 tests run, 389
  passed, 0 skipped, 0 failed** (37 binaries). Includes all `csid_ext`/P1
  chunk-set-id tests (`csid_verification.rs`,
  `encode_repair_chunk_set_id_p2.rs`, etc. — 14 references to those files
  in the run log, no failures) — the two named guarantees (R2 warning
  behavior, `repair --json` byte-match) are unaffected.
- `cargo fmt --check`: initially found one diff in the new test file
  (a collapsible `assert!` block); ran `cargo fmt`, re-ran `--check` —
  clean, exit 0, no output.
- `cargo clippy --all-targets -p mk-cli -p mk-codec`: clean build, 0
  warnings (`grep -ic warning` on the log returns `0`).

## Deviations from the brief

- **`mk verify --json` was left unchanged** (no `corrections` field
  added). The brief made this optional ("if it's clean to do so... if
  unsure, keep the note stderr-only and leave `--json` alone"). Given the
  existing D27/`csid_verification.rs` byte-match regression tests pinning
  the `--json` envelope shape (`verify_json_chunk_set_id_object`,
  `repair_json_byte_unchanged_no_chunk_set_id_field` and siblings), stderr-only
  was the conservative, unambiguously-safe choice and satisfies the
  followup's stated goal ("surface correction counts... so budget
  consumption is visible"). No `--json` schema change was made or tested.
- No `mk-codec` changes were made (per constraint).
- Nothing was committed; the worktree remains dirty with 3 modified files
  and 1 new untracked test file, as instructed.
