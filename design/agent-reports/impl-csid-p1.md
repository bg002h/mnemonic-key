# P1 implementation report — mk-cli read-side R2 mismatch warning

Worktree: `/scratch/code/shibboleth/mk-worktrees/csid-p0`, branch
`impl/csid-p0`, on top of `58c8df4` (P0: extension vector corpus).
Implements `design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md` P1
against `design/SPEC_chunk_set_id_verification.md` contracts 2, 3, 4 and
"The comparison". **Uncommitted, unstaged** per instruction — the worktree
is left dirty for review.

## Files changed

```
 crates/mk-cli/src/cmd/address.rs |  4 +-
 crates/mk-cli/src/cmd/decode.rs  |  9 ++++-
 crates/mk-cli/src/cmd/derive.rs  |  6 ++-
 crates/mk-cli/src/cmd/inspect.rs | 24 ++++++++++--
 crates/mk-cli/src/cmd/mod.rs     | 84 +++++++++++++++++++++++++++++++++++++
 crates/mk-cli/src/cmd/verify.rs  | 44 +++++++++++++++++---
 6 files changed, 160 insertions(+), 11 deletions(-)
```

New test file (untracked): `crates/mk-cli/tests/csid_verification.rs`, 257
lines, 8 `#[test]` functions.

`mk-codec` untouched — mint-side (`mk encode --chunk-set-id` warning) and
`mk repair` are P2, out of scope here.

## Design

`crates/mk-cli/src/cmd/mod.rs` gains four functions:

- `declared_chunk_set_id(&[String]) -> Option<u32>` (private) — reads the
  FIRST string's string-layer header via `mk_codec::string_layer::decode_string`
  + `StringLayerHeader::from_5bit_symbols` (same mechanism `mk repair`
  already uses for grouping). `None` for `SingleString` headers or any
  future `#[non_exhaustive]` variant. Reading only the first chunk is
  sound because `mk_codec::decode`'s `reassemble_from_chunks` already
  proved every chunk agrees (`Error::ChunkSetIdMismatch` otherwise).
- `derived_chunk_set_id(&KeyCard) -> Option<u32>` (private) — SPEC "The
  comparison" operand: `mk_codec::derive_chunk_set_id(mk_codec::bytecode::encode_bytecode(card)?)`,
  fully qualified per the task's `md_codec` collision instruction.
- `chunk_set_id_comparison(&[String], &KeyCard) -> Option<(u32, u32)>`
  (pub) — `None` on single-string input; `Some((declared, derived))`
  otherwise, regardless of match (verify's JSON needs the pair even on a
  match).
- `warn_chunk_set_id_mismatch(Option<(u32, u32)>)` (pub) — the one-line
  stderr `eprintln!` on a genuine mismatch; no-op otherwise.
- `chunk_set_id_mismatch_warning(u32, u32) -> String` (pub) — the frozen
  R2/R6 wording, byte-identical to the corpus's `warning_text` for the
  pinned row (verified below).

Each of decode/derive/address calls
`warn_chunk_set_id_mismatch(chunk_set_id_comparison(&strings, &card))` as
**one line**, immediately after `let card = mk_codec::decode(&refs)?;` —
independently deletable per verb, which is what the mutation gate below
exercises. `inspect` and `verify` compute `chunk_set_id_comparison` once
and reuse the pair (inspect for its unconditional stamped-id print;
verify for its stdout/JSON contract-4 additions), rather than centralizing
in `read_mk1_strings` (which only reads strings and never decodes, per
plan P1 / r1 M2).

`mk inspect` (contract 3): unconditional `chunk_set_id:  {id:05x} (stamped)`
line in text mode only (`--json` untouched, per contract 3 / SPEC item 4).

`mk verify` (contract 4): `emit_ok` gained a `csid: Option<(u32,u32)>`
parameter. Text mode appends the frozen warning text on STDOUT after the
`OK:` line when mismatched. JSON mode adds an additive
`"chunk_set_id": {"declared","derived","matches"}` object (hex strings,
`{:05x}`) whenever `csid.is_some()` (i.e. chunked input, matched or not);
`schema_version` stays the integer `1`; no other envelope field changed.

## Regression scope check (before writing tests)

Audited every existing test referencing `--chunk-set-id`, `inspect`, or
`verify --json`: none decode/inspect/verify/derive/address a chunked card
minted with a **pinned, mismatching** `--chunk-set-id` (the only tests
using the flag are refusal tests on `encode` that never decode the
result). Default `mk encode` derives the id, so every existing fixture is
declared==derived — the new stderr content and inspect's new unconditional
line are additive there and do not appear in any exact-string assertion
(`cli_output_class.rs` uses substring `.contains()`, `channels.rs` compares
two runs against each other, not a fixed string). No `verify --json` test
existed previously. Confirmed empirically: full existing suite is green
below.

## RED → GREEN evidence

RED was taken by `git stash`-ing all 6 `src/cmd/*.rs` changes (keeping
only the new test file), running the suite, then `git stash pop` to
restore the implementation.

**RED (pre-P1, 8/8 fail):**

```
thread 'decode_mismatch_row_warns_clean_twin_silent' panicked at crates/mk-cli/tests/csid_verification.rs:96:5:
decode: mismatch row must warn verbatim on stderr (matches corpus warning_text); stderr="note: stdout is watch-only — public keys only, cannot spend\n"

thread 'verify_mismatch_row_warns_clean_twin_silent' panicked at crates/mk-cli/tests/csid_verification.rs:96:5:
verify: mismatch row must warn verbatim on stderr (matches corpus warning_text); stderr=""

thread 'verify_stdout_verdict_carries_the_mismatch' panicked at crates/mk-cli/tests/csid_verification.rs:174:5:
verify: mismatch must carry the pair + remedy on STDOUT too (contract 4), not only stderr; stdout="OK: mk1 string(s) decode cleanly (and any --xpub / --origin-* / --policy-id-stub / --from-md1 inputs match)\n"

Summary [0.004s] 8 tests run: 0 passed, 8 failed, 0 skipped
```

(All 8 failed; `inspect`/`derive`/`address` and the JSON/unconditional-print
tests failed the same way — no warning text, no `chunk_set_id`, no printed
stamped id.)

**GREEN (post-P1, restored):**

```
Nextest run ID 7416bccd-4dbc-4585-9a2b-04dc84691586 with nextest profile: default
    Starting 8 tests across 1 binary
        PASS [0.004s] (1/8) mk-cli::csid_verification inspect_mismatch_row_warns_clean_twin_silent
        PASS [0.004s] (2/8) mk-cli::csid_verification verify_mismatch_row_warns_clean_twin_silent
        PASS [0.004s] (3/8) mk-cli::csid_verification decode_mismatch_row_warns_clean_twin_silent
        PASS [0.005s] (4/8) mk-cli::csid_verification verify_json_chunk_set_id_object
        PASS [0.005s] (5/8) mk-cli::csid_verification address_mismatch_row_warns_clean_twin_silent
        PASS [0.005s] (6/8) mk-cli::csid_verification verify_stdout_verdict_carries_the_mismatch
        PASS [0.005s] (7/8) mk-cli::csid_verification derive_mismatch_row_warns_clean_twin_silent
        PASS [0.005s] (8/8) mk-cli::csid_verification inspect_prints_stamped_chunk_set_id_unconditionally
     Summary [0.005s] 8 tests run: 8 passed, 0 skipped
```

## `mk verify` stdout, before/after, on the pinned mismatch row (`12345`/`ef12f`)

**Before (pre-P1):**
```
OK: mk1 string(s) decode cleanly (and any --xpub / --origin-* / --policy-id-stub / --from-md1 inputs match)
```

**After (post-P1):**
```
OK: mk1 string(s) decode cleanly (and any --xpub / --origin-* / --policy-id-stub / --from-md1 inputs match)
warning: this key card's stamped chunk-set id (12345) was not derived from its content, which computes ef12f. The card decodes fine, but diagnostics that name plates by id will call it 12345. To fix it, re-mint: run mk encode again without --chunk-set-id and the id is derived from the key data automatically.
```

`verify --json` (post-P1), mismatch vs. clean twin, verified by direct
binary invocation:
```
{"chunk_set_id":{"declared":"12345","derived":"ef12f","matches":false},"chunks":2,"ok":true,"policy_id_stubs":["000c7765"],"schema_version":1}
{"chunk_set_id":{"declared":"ef12f","derived":"ef12f","matches":true},"chunks":2,"ok":true,"policy_id_stubs":["000c7765"],"schema_version":1}
```
`schema_version` confirmed integer (not string) via `.is_number()` in the
test.

## Mutation gate (decode, per plan's "e.g. decode")

Deleted the single line
`warn_chunk_set_id_mismatch(chunk_set_id_comparison(&strings, &card));`
from `decode.rs` (commented out), reran `csid_verification`, restored.

**Before restore (mutated):**
```
Starting 8 tests across 1 binary
        FAIL [0.003s] (1/8) mk-cli::csid_verification decode_mismatch_row_warns_clean_twin_silent
  stderr: decode: mismatch row must warn verbatim on stderr (matches corpus warning_text); stderr="note: stdout is watch-only — public keys only, cannot spend\n"
        PASS [0.003s] (2/8) mk-cli::csid_verification verify_json_chunk_set_id_object
        PASS [0.003s] (3/8) mk-cli::csid_verification inspect_mismatch_row_warns_clean_twin_silent
        PASS [0.003s] (4/8) mk-cli::csid_verification verify_stdout_verdict_carries_the_mismatch
        PASS [0.004s] (5/8) mk-cli::csid_verification derive_mismatch_row_warns_clean_twin_silent
        PASS [0.004s] (6/8) mk-cli::csid_verification address_mismatch_row_warns_clean_twin_silent
        PASS [0.004s] (7/8) mk-cli::csid_verification verify_mismatch_row_warns_clean_twin_silent
        PASS [0.004s] (8/8) mk-cli::csid_verification inspect_prints_stamped_chunk_set_id_unconditionally
     Summary [0.004s] 8 tests run: 7 passed, 1 failed, 0 skipped
```

Exactly `decode`'s row failed; all 4 other verbs' rows (inspect, verify
x2, derive, address) still passed — proving the comparison is per-surface
and independently deletable, and that the test actually exercises the
mutated line (not merely landing on dead code). Line restored verbatim;
confirmed 8/8 green again afterward (see below).

## Full-suite verification (post-restore)

```
cargo nextest run --locked -p mk-cli -p mk-codec
Starting 381 tests across 35 binaries
...
Summary [0.080s] 381 tests run: 381 passed, 0 skipped
```

0 failures across the whole `mk-cli` + `mk-codec` surface, including:

- The two named guarantees, byte-unchanged:
  `mk-codec::chunk_set_id_determinism::an_explicit_chunk_set_id_still_wins`
  and `mk-codec::canonical_payload::canonical_payload_is_chunk_set_id_invariant`
  — both PASS.
- Corpus pins unchanged: `mk-codec::vectors::vector_file_sha256_matches_pin`
  (v0.1.json) and `mk-codec::csid_ext_vectors::csid_ext_sha256_matches_pin`
  (csid_ext) — both PASS (P1 touched no mk-codec file).
- Pre-existing tests specifically checked for regression risk, all PASS:
  `cli_output_class::{inspect,derive,address}_emits_watch_only_advisory`,
  `cli_output_class::verify_emits_no_advisory`,
  `channels::in_on_the_reading_verbs_equals_the_positional_run`.

`cargo clippy -p mk-cli -p mk-codec --all-targets`: 0 warnings.

## Deviations from the brief, with reasons

1. **`mk address` exit-code assertion is 64, not 0, for both the mismatch
   row and its clean twin.** Both corpus seed cards
   (`SEED_pinned_12345_ef12f` / `SEED_plate_b_ef12f`) carry origin path
   `48'/0'/0'/2'` (BIP-48 multisig cosigner), which `resolve_address_type`
   refuses unconditionally — before or after this change. The warning
   still fires at `address`'s own decode call, strictly before that
   unrelated refusal (verified: it appears on stderr ahead of the
   `error:` line). The golden test asserts warning-present/absent and
   **exit-code equality between the mismatch and clean runs** (rather than
   a hardcoded "0"), which is the invariant P1 actually needs to prove:
   R2 must not change what a verb returns for a given input shape. This
   is a fixture property, not an implementation gap — no P1 corpus row is
   single-sig-shaped, so no combination of corpus rows would let `address`
   reach exit 0 while it also carries a genuine mismatch.
2. **Chose a compute-then-warn split (`chunk_set_id_comparison` /
   `warn_chunk_set_id_mismatch`) over one combined "decode-and-warn"
   function**, so `inspect` and `verify` (which need the declared/derived
   values for their own additional output) don't recompute
   `encode_bytecode` a second time. `decode`/`derive`/`address` call both
   in one line; the mutation gate confirms this still yields a single,
   independently-deletable statement per verb.
3. **Inspect's printed line wording** (`chunk_set_id:        {id:05x}
   (stamped)`) and its position (after the per-chunk BCH-variant lines) are
   not spec-frozen — contract 3 only requires an unconditional print, not
   specific wording/placement. Chosen to match the file's existing
   `label:  value` column style.

## Acceptance mapping

- Golden tests (5 verbs x mismatch/clean, contract 3 unconditional print +
  `{:05x}` leading-zero via `LZ1_derived_below_0x10000`, contract 4 both
  modes): all present, all RED→GREEN as shown above.
- Mutation gate: done for `decode` per the plan's own example; per-surface
  independence demonstrated (1 failure, not 5, on deletion).
- `cargo nextest run --locked -p mk-cli -p mk-codec`: 381/381 green.
- Named guarantees + both corpus SHA pins: unchanged, confirmed green.
- No mk-codec change; `read_mk1_strings` untouched (still a pure string
  reader); no exit code changed; P2 (mk encode / repair) untouched.
