# mk baseline GAP-B hardening report

Model: Claude Sonnet 5.5 (claude-sonnet-5-5). Base: main 95d70a5a83e115530a2de6877d50e021f9fe2620 (tip unchanged after fetch).
Tests only; no production code changed. Mutation loop (scripted sed-equivalent, not committed) ran each mutant against the one named test, reverting after each.
Note: the missed list at the base holds 14 GAP-B mutants (path.rs:165:15 has both `>`→`==` and `>`→`>=`; the grader counted 1). Both are killed.

| survivor | status | test | base | mutant |
|---|---|---|---|---|
| path.rs:165:15 `>`→`==` | killed | mk-codec bytecode::path::tests::leb128_u32_range_boundary | 1 passed | FAIL (1 failed) |
| path.rs:165:15 `>`→`>=` | killed | same (u32::MAX accepted) | 1 passed | FAIL |
| inspect.rs:74 → String::new() | killed | baseline_gapb inspect_prints_xpub_fingerprint_and_stub | 1 passed | FAIL |
| inspect.rs:74 → "xyzzy" | killed | same | 1 passed | FAIL |
| mod.rs:355 fmt_stub → String::new() | killed | baseline_gapb inspect_prints_xpub_fingerprint_and_stub, verify_json_prints_policy_stub, decode_prints_policy_stub | 5 passed | 3 failed |
| mod.rs:355 fmt_stub → "xyzzy" | killed | same | 5 passed | 3 failed |
| mod.rs:347:29 `==`→`!=` | killed | from_md1_set encode_from_complete_keyed_md1_refuses_non_member_xpub (exit 64 "is not a cosigner", 2-key fixture, BIP-32 vector-1 xpub) | 1 passed | FAIL |
| repair.rs:309:9 delete SingleString arm | killed | cli_repair repair_corrected_single_string_that_fails_decode_exits_2 | 1 passed | FAIL |
| repair.rs:309:71 `==`→`!=` | killed | same | 1 passed | FAIL |
| verify.rs:99:26 guard→true | killed | baseline_gapb verify_origin_fingerprint_right_wrong_omitted (0 / 4 / 4) | 1 passed | FAIL |
| verify.rs:99:26 guard→false | killed | same | 1 passed | FAIL |
| verify.rs:99:30 `==`→`!=` | killed | same | 1 passed | FAIL |
| verify.rs:118:18 `!=`→`==` | killed | baseline_gapb verify_origin_path_right_wrong (0 / 4) | 1 passed | FAIL |
| slip132.rs:70:61 `&&`→`||` | killed | slip132::tests::path_predicate_truth_table (two added negatives) | 1 passed | FAIL |

Deviation from the grader's suggestion (item 5): a valid single-string mk1 cannot carry an xpub (65 bytes > the 56-byte single-string cap), so the Blessed outcome is unreachable for a single string. The test uses the reachable Reject path (BCH-valid single string with undecodable bytecode, 1 char corrupted, expect exit 2 and empty stdout); both repair.rs:309 mutants turn it into an exit-5 candidate.

Open point from the grader (mod.rs:318 chunked md1 via decode_md1_string) not in the 13; untouched.

## Gates (logs in /scratch/cache/mk-harden-gapb-logs/)
- cargo nextest run --locked -j 8 --workspace --no-fail-fast: 423 passed, 0 failed
- cargo test --locked --workspace --doc: ok (0 failures)
- cargo clippy --locked --workspace --all-targets -- -D warnings: clean
- cargo +1.95.0 fmt --check --all: clean; design vector checksum pin: OK
- Not run: vectors-roundtrip (release build, tests-only change), freebsd compile-gate and musl jobs (cross targets, not installed/needed for tests-only), docs-only guard scripts.
- Not run by instruction: mutation-gate.sh, cargo-mutants.

KILLED 14 / EQUIV 0 / STOP 0 / BASE-FAIL 0
