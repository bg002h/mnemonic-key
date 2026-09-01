# Implementation report: `md-cli-seat-warning-corpus-binding` (interim same-repo guard)

**Worktree:** `/scratch/code/shibboleth/dm-worktrees/warn-bind`
**Branch:** `followup/md-warning-binding` (base `d41950fe`, descriptor-mnemonic)
**Scope:** interim same-repo guard only (NOT the cross-repo `go-mk-vector-corpus-ingestion` vendoring)
**Status:** implemented, dirty/unstaged as instructed — no commit made

## Files + lines changed

Only `crates/md-cli/src/seat/input.rs` — 48 insertions, 0 deletions, 0 files
touched besides this one (`git diff --stat` confirms). No other seat logic,
exit codes, or messages changed.

1. **Doc comment addition**, `chunk_set_id_mismatch_warning` (lines ~382–399
   after edit; function body/format string itself unchanged): appended a
   paragraph stating this function IS md-cli's single source of truth for the
   wording, pointing at `design/FOLLOWUPS.md`'s
   `md-cli-seat-warning-corpus-binding` (this guard) and
   `go-mk-vector-corpus-ingestion` (the cross-repo successor).
2. **New test**, `crates/md-cli/src/seat/input.rs::tests`, function
   `csid_warning_wording_is_pinned_against_literals_not_the_function_itself`
   (inserted after the existing `csid_warning_fires_on_a_pinned_mismatch_...`
   test, ~line 740 post-edit).

## Was the text already a const, or hoisted?

**Already a single named source of truth — no hoist performed.**
`chunk_set_id_mismatch_warning(declared: u32, derived: u32) -> String` at
`crates/md-cli/src/seat/input.rs:387` (pre-edit numbering) was already a
`pub fn` (not a `const`, but explicitly permitted by the brief as "a small
format helper"). Verified by grep:

- `chunk_set_id_mismatch_warning` occurs in `crates/md-cli/` at exactly 4
  lines: its own doc comment, its own definition, its ONE production call
  site (`seat_chunk_set_id_warnings`, `input.rs:411`), and the pre-existing
  test at `input.rs:720` (which calls the function itself, not a literal
  copy).
- Grepped for the load-bearing phrases (`was not derived`, `re-mint`,
  `chunk-set id`) across all of `crates/md-cli/`: no second hand-copy of this
  warning's prose exists anywhere else in md-cli (other `re-mint` /
  `chunk-set id` hits are the unrelated R5 policy-card message in
  `seat/matching.rs:217` and CLI-flag-parsing messages in
  `seat/directive.rs`, neither of which share this wording).

So the followup's structural premise ("hand-maintained copy" duplicated
somewhere) doesn't apply within md-cli itself — the real gap, matching the
followup's own text, was that **nothing tested the wording's actual content**:
the one existing test (`csid_warning_fires_on_a_pinned_mismatch_...`,
`input.rs:718-722`) builds its expected value by calling
`chunk_set_id_mismatch_warning` itself, so it is tautological and cannot
catch a drift in the function's own template.

## The pinning test

```rust
#[test]
fn csid_warning_wording_is_pinned_against_literals_not_the_function_itself() {
    let w = chunk_set_id_mismatch_warning(0x99999, 0x69f0e);
    assert!(w.starts_with("warning:"), "{w}");
    assert!(
        w.contains(
            "this key card's stamped chunk-set id (99999) was not derived from its content"
        ),
        "{w}"
    );
    assert!(w.contains("which computes 69f0e"), "{w}");
    assert!(
        w.contains("diagnostics that name plates by id will call it 99999"),
        "{w}"
    );
    assert!(
        w.contains(
            "To fix it, re-mint: run mk encode again without --chunk-set-id and the id is \
             derived from the key data automatically."
        ),
        "{w}"
    );
}
```

Every literal in this test is independent of the source's own template
string — it asserts the (declared, derived) pair AND the full remedy
sentence P3 actually shipped (read from source, not guessed).

## TDD: mutation before/after

1. **Baseline green (unmutated source):** ran the new test alone —
   `cargo nextest run --locked -p md-cli csid_warning_wording_is_pinned` →
   `1 test run: 1 passed, 697 skipped`.
2. **Mutation:** temporarily replaced the remedy clause in
   `chunk_set_id_mismatch_warning`'s `format!` template with `"To fix it,
   just try again."` (dropping "re-mint: run mk encode again without
   --chunk-set-id and the id is derived from the key data automatically.").
3. **Red:** re-ran the same filtered test →
   `FAIL ... 0 passed; 1 failed`, panic message showing the mutated
   ("just try again.") output, proving the assertion on the remedy phrase is
   load-bearing.
4. **Restore:** reverted the template to the original literal; `diff`
   against a pre-mutation backup of the file confirmed byte-identical
   source (mutation cleanly reverted, no residue).
5. **Green again:** re-ran the filtered test → `1 passed, 697 skipped`.

## Existing seat tests — byte-identical output confirmed

Full `md-cli` suite before any change: 697/697 passed (baseline capture).
After the doc-comment addition + new test + `cargo fmt` pass: **698/698
passed** (697 pre-existing + 1 new), including both pre-existing seat-warning
tests unchanged:

- `seat::input::tests::csid_warning_fires_on_a_pinned_mismatch_and_is_silent_on_the_clean_twin`
- `crates/md-cli/tests/seating_vectors.rs::v_csid_seat_warning_fires_on_pinned_mismatch_and_composes_identically`
- `crates/md-cli/tests/seating_vectors.rs::v_csid_seat_no_warning_on_a_clean_card_set`

No production code (the `format!` template, `seat_chunk_set_id_warnings`,
`derived_chunk_set_id`) was touched — only a doc comment was added above the
function and a new `#[cfg(test)]` test was appended, so byte-identical output
is structural, not just measured.

## fmt + clippy

- `cargo fmt --check` initially flagged the new test's formatting (one
  `assert!` line exceeded width); ran `cargo fmt -p md-cli` to apply the
  project's standard formatting, then `cargo fmt --check` → clean (exit 0).
  Re-ran the full suite after reformatting: still 698/698.
- `cargo clippy -p md-cli --all-targets` → clean, no warnings.

## nextest count

`cargo nextest run --locked -p md-cli` → **698 tests run: 698 passed, 0
skipped** (697 P3-era baseline + 1 new pinning test).

## Deviations from the brief

None. Did not touch mk-codec/md-codec, did not change any exit code or
other seat message, did not commit (worktree left dirty/unstaged per
instruction — `git status --short` shows exactly
`M crates/md-cli/src/seat/input.rs`). Chose not to introduce a separate
`const SEAT_CSID_MISMATCH_WARNING` because the brief's own alternative ("or a
small format helper") was already satisfied by the pre-existing
`chunk_set_id_mismatch_warning` function, and no second hand-copy of the
wording existed anywhere in md-cli to consolidate — introducing a new const
that the function would then need to route through would have been a
no-op refactor with no reduction in duplication, so the fix instead focused
on the actual gap the followup named: an assertion that never checks the
wording's own content.
