# Review: mk FU1 bundle-stamped chunk-set id (PR #26), whole diff 89192ac7..65f0470c

Reviewer: Claude Sonnet 5.5, model id claude-sonnet-5-5. First review (not a re-review).

Verdict: no Critical, no Important. All five call sites, verify's stdout repeat and repair's blessed loop route
through `chunk_set_id_warns`; `bundle_stamped` in `verify --json` is right.

## Lens 1: tests-can-fail (plants in `cmd/mod.rs` unless named; each run `cargo nextest run --locked -p mk-cli -j 8`; tree restored, clean)

| Plant | Result |
|---|---|
| limit 32 -> 33 | RED: `a_pin_one_past_the_last_slot_still_warns`, `verify_json_names_a_bundle_stamped_id` |
| limit 32 -> 31 | RED: `bundle_cards_at_slots_1_16_31_are_silent` |
| `is_bundle_stamped` -> `true` | RED: `a_pin_one_past_the_last_slot_still_warns` |
| `is_bundle_stamped` -> `false` | RED: 4 tests (silent-verbs, repair, json, ...) |
| `s[0] << 12` -> `<< 11` | RED (same group; run saw 4 failures) |
| no-stub branch `return false` -> `true` | **GREEN 208/208** (see N1: unreachable) |
| `.first()` -> `.last()` | **GREEN 208/208** (M1) |
| verify.rs stdout repeat reverted to `declared != derived` | RED: `bundle_card_is_silent_on_every_read_verb` |
| repair.rs blessed loop reverted | RED: `repair_of_a_bundle_card_does_not_warn` |
| json `bundle_stamped` hard-coded false / true | RED both: `verify_json_names_a_bundle_stamped_id` |

## Lens 2: slot window vs toolkit (read code, /scratch/code/shibboleth/mnemonic-toolkit)

`synthesize.rs:77-95`: `derive_mk1_chunk_set_id` = `(s0<<12)|(s1<<4)|(s2>>4)`, slot id = that `^ slot`: identical to
`is_bundle_stamped`'s base. Every multi-cosigner path stamps `vec![stub; n]` (`synthesize.rs:527,715,1123`), so the first
stub is the base for every slot. Slots reach 31 only on the descriptor path (`synthesize.rs:618,873` cap the other paths at
16; md1 caps keys at 32); the test's SLOT31 card is real `mnemonic bundle` output. 32 excluded (pin test), 31 included
(RED under limit 31). A card with no stubs cannot exist: `mk-codec` `bytecode/encode.rs:25` rejects empty `policy_id_stubs`.

## Lens 3: texts

BIP note, SPEC contracts 2 and 4, CHANGELOG, FOLLOWUPS are true to the code (bip :327 is the Note line; "repair" and
"verify --json" claims match). Advice lens: no new advice text; the unchanged warning/remedy still fires for every non-bundle
mismatch, and "re-minting is not needed" is correct for a bundle card (a re-minted template card would break verify-bundle).

## Findings

- **M1** `crates/mk-cli/src/cmd/mod.rs:149` ("only the FIRST stub counts") is untested: `.first()` -> `.last()` passes 208/208.
  Fault is spurious warning (or rare spurious silence) on a multi-stub card. Fix: add a test with a 2-stub card
  (`mk encode --policy-id-stub A --policy-id-stub B --chunk-set-id <A-top20 ^ 3>`, multi-chunk) asserting silent, and one stamped
  from B's top 20 asserting the warning.
- **N1** `mod.rs:146-148` the `let Some .. else { return false }` branch is unreachable from any decodable card (empty stubs
  rejected at encode); the "no stubs warns" behaviour cannot be tested end to end. Keep as defensive; no action.
- **N2** SPEC contract 2 (`design/SPEC_chunk_set_id_verification.md` ~:138): the inserted exception clause now sits before the
  bold "one warning per mismatching group" and reads as if the bold text were the silent case. Suggest: end the exception with a
  full stop and start a new sentence "On any other mismatch: **one warning per mismatching group, ...**".
- **N3** BIP note: "SHOULD NOT warn" is acceptable (the BIP already uses 9 SHOULDs, no BCP14 boilerplate to conflict with) but it
  governs optional diagnostics only; "MAY omit the warning" would match the document's otherwise-opaque-field stance. Not required.
- (Not filed) `bundle_stamped` can be true together with `matches: true` by coincidence; harmless and documented.

## Mutation gate

`mutation-gate 65f0470c vs 89192ac7: caught 31, MISSED 2, timeout 0, unviable 0 (pass 1 exit 2)`; 33 mutants.
Both survivors, graded with survivor.sh: `mod.rs:153:61` and `mod.rs:153:38`, `|` -> `^` in the `base` expression: **equivalent**
(the three shifted u8 terms occupy disjoint bit ranges 12-19, 4-11, 0-3, so OR equals XOR).

C / I / M / N: 0 / 0 / 1 / 3
