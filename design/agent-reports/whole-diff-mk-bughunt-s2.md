# Whole-diff review s2 (Sonnet): tests-can-fail, mk bug-hunt fix, head 78f5ded5

No Critical or Important. Previous-round fix commits: this is the first whole-diff round (0 findings from a prior fold).

## Method
Target dir `/scratch/cache/bughunt/mk/s2-target`. Ran all 8 `bughunt_mk_` rows by exact-name regex filter on the head: 8 run, 8 passed (row 03 30.1 s). Then 12 one-hunk scratch plants, each reverted with `git checkout -- <file>` (git status clean afterwards apart from reports/untracked logs).

## Plants (result)
| # | plant | caught by |
|---|---|---|
| P1 | bch.rs `to_ascii_lowercase` -> `to_lowercase` only | both `bughunt_mk_05_*` FAIL |
| P2 | drop the data-part non-ASCII check only | `05_non_ascii_error_variant_and_position` FAIL (refused-test still passes via InvalidStringLength; expected) |
| P3 | position = byte index (`data_part.find(c)`) | NOT caught (equivalent mutant, see N1) |
| P4 | position `i + 1` | `05_..._position` FAIL |
| P5 | move non-ASCII check after length lookup | `05_..._position` FAIL |
| P6 | guard without `key_source.is_none()` (fix-01 as drafted) | `01_keys_and_in_...` FAIL |
| P7 | remove guard arm | `01_encode_without_fingerprint_flag_not_silent` FAIL |
| P8 | revert `s.get(..3)` hunk | `04_...` FAIL |
| P9 | count cap `>=` | `03` FAIL |
| P10 | range cap `>` | `03` FAIL (30 s: the 0,1000000 row accepted) |
| P11 | count cap before ceiling check | `03` FAIL (ceiling-first ordering pinned) |
| P12 | drop range cap | `03` FAIL |
| P13 | `s_lower = s.to_string()` (uppercase no longer lowercased) | whole suite 398/399: only `mk-cli::cli_repair repair_uppercase_input_emits_redecodeable_lowercase` fails; no mk-codec test and no bughunt row (see N2) |

`bughunt_mk_02` and `_06` pass by design on the fix (no production change); 02 asserts same-value decode, cursor empty and minimal re-encode (spec 1.2 both halves pinned). `bughunt_mk_01`/`03` assert exact stderr and exit codes from the FIX doc. Row 04 asserts exit 0 only (not that the md1 lines were still used); acceptable.

## Spec sentences vs pins
1.1 (ASCII-only, no Unicode folding): pinned by 05 x2 (Kelvin at HRP and data). 1.2: pinned by 02. 1.3: pinned by 01 (both halves). 1.4: pinned by 03 (all five error texts, boundary accepts). All pinned.

## Minors / Nits
- N1: P3 is an equivalent mutant. With exactly one non-ASCII char the first-non-ASCII byte offset equals its char index, so "char index, not byte index" (FIX §2 row 5; test doc comment) cannot be falsified by the test. A probe with a non-ASCII char EARLIER than the reported one cannot exist (first is reported), so no test could pin it; leave as is.
- N2: no mk-codec-level test pins that `decode_string` accepts an all-uppercase valid card (P13); the only guard is the cli_repair test, which reaches it through repair. Fix text (optional): in `crates/mk-codec/tests/bughunt.rs` add `assert!(decode_string(&strings[0].to_uppercase()).is_ok());` to the position test. Not run as a plant-fix; trivially mechanical. Fails closed, one unusual condition (a plant), so Nit.
- N3: replacing the separator char (index 2) with a non-ASCII char gives `InvalidHrp("mk<c>q...")` carrying the whole string (probed: `InvalidHrp("mkKqpqqq...")`),. Row 05a asserts only `Err`, and no spec text fixes this payload; fails closed. Not pinned, not wrong.
- N4: row 04 and 01 use `Some(0)`/success only; adequate.

0 C / 0 I / 0 M / 4 N
