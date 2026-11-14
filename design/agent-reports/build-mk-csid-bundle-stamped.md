# Build report: mk FU1, bundle-stamped chunk-set id stays silent (Option A)

- Model: Claude Sonnet 5.5, claude-sonnet-5-5
- Base: 89192ac706e0dc6d5a4c68f83a56ccc4ecf5d904 (origin/main, unmoved at start; `git apply --check` of the diff passed)
- Head: see the final summary (the report is committed last)
- PR: see the final summary

## RED (commit e94833f, unmodified src)
    FAIL verify_json_names_a_bundle_stamped_id
    FAIL bundle_card_is_silent_on_every_read_verb
    FAIL repair_of_a_bundle_card_does_not_warn
    FAIL bundle_cards_at_slots_1_16_31_are_silent
    PASS bundle_card_inspect_still_prints_the_stamped_id   (guard)
    PASS a_pin_one_past_the_last_slot_still_warns          (guard)
    Summary 6 tests run: 2 passed, 4 failed   (log: /scratch/cache/mk-csid-logs/red.log)

## GREEN (commit 7a6d698)
    Summary 6 tests run: 6 passed, 0 skipped   (log: /scratch/cache/mk-csid-logs/green.log)
`cargo fmt` (pinned 1.85.0) changed nothing.

## Tests that changed
None edited; one file added (`crates/mk-cli/tests/csid_bundle_stamped.rs`).

## Text deltas
SPEC contract 2 insert (silent) and contract 4 `bundle_stamped`; BIP :327 append; `crates/mk-cli/CHANGELOG.md` [Unreleased] ### Changed (the brief's "CHANGELOG.md" for mk-cli); FOLLOWUPS Q4 entry (status `fixed in this PR`).
Note: the grade gives the contract 4 delta only as "add the `bundle_stamped` field"; I extended the JSON example with `"bundle_stamped": false` and one clause saying it is true for a contract-2 id and that `matches` keeps its literal meaning. Wording is mine, from the grade's Q2 ruling; see STOPs.

## STOPs
- Contract 4 wording (above): the grade did not give exact sentence text. Minor; coordinator to confirm or reword.

## Gate
