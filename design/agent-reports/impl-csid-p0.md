# P0 implementation report — chunk_set_id extension vector corpus

**Worktree:** `/scratch/code/shibboleth/mk-worktrees/csid-p0`, branch
`impl/csid-p0`, baseline `725ccb9` (matches the plan's `mnemonic-key 7ef32f7`
lineage — worktree tip is downstream of it on the same branch history).
Working tree left **dirty, nothing staged**, per instruction — this report
was written to the main checkout as the one explicit exception to
"work only in the worktree."

Implements P0 of `design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md`:
the NEW extension vector corpus (`csid_ext_v0.1.json`) supplying the CLEAN
half plus warning content, without touching the pinned legacy `v0.1.json`.

## Files changed (all in the worktree; none committed)

| File | Change | Lines |
|---|---|---|
| `crates/mk-codec/src/bin/gen_mk_vectors.rs` | extended: csid_ext row builders + main() now writes both files | +327 / −21 (`git diff --stat`) |
| `crates/mk-codec/src/test_vectors/mod.rs` | added `pub mod csid_ext;` | +4 |
| `crates/mk-codec/src/test_vectors/csid_ext.rs` | **new** — `include_str!` bake-in, mirrors `V0_1_JSON` | 22 |
| `crates/mk-codec/src/test_vectors/csid_ext_v0.1.json` | **new**, generator output (not hand-written) | 279 |
| `crates/mk-codec/tests/csid_ext_vectors.rs` | **new** — reader test suite | 164 |

`v0.1.json` — **byte-unchanged** (confirmed both before and after a
type-alias clippy fix in the generator, via `diff` against a pre-refactor
copy and `git diff --stat` showing 0 lines touched).

## Corpus contents (21 rows, generated via
`cargo run -p mk-codec --bin gen_mk_vectors --features gen-vectors`, never
hand-written)

- **3 legacy twins** (`CT1`/`CT2`/`CT3`) — same `KeyCard` shape as v0.1.json's
  V1/V2/V3, minted via the auto-derive `encode()` path so declared==derived.
  Independent cross-check: CT1's derived id `83bb2` matches the spec's own
  measured V1 value verbatim (`SPEC_chunk_set_id_verification.md:244`: "V1
  declared 12345, derived 83bb2") — strong evidence the shape reuse is
  correct.
- **3 seed cards** — `SEED_plate_a_1b1ba` (declared=derived=`1b1ba`),
  `SEED_plate_b_ef12f` (declared=derived=`ef12f`), and
  `SEED_pinned_12345_ef12f` — the SAME card content as plate B, re-minted
  with `--chunk-set-id 0x12345` (declared=`12345`, derived=`ef12f`,
  `expect_mismatch_warning: true`), reproducing the walk's exact THE-CRUX
  scenario. Target ids were hit via a deterministic brute-force nonce search
  (`search_card`, varies `policy_id_stubs[0]` as a big-endian u32, budget
  20M tries) — both resolved in well under a second (full generator run:
  1.32s wall for all three searches plus the leading-zero search).
- **14 STANDARD_PATHS rows** (`SP01..SP14`), one per entry of
  `mk_codec::bytecode::STANDARD_PATHS` (indicators `0x01..0x07`,
  `0x11..0x17`), each clean. `tests/csid_ext_vectors.rs`'s
  `standard_paths_table_fully_covered` asserts row-count ==
  `STANDARD_PATHS.len()` live, so a future 15th entry trips that assertion
  rather than going silently uncovered (spec r4 L1-I1).
- **1 leading-zero row** (`LZ1_derived_below_0x10000`), derived id `0191c`
  (< 0x10000), exercising `{:05x}` zero-padding (r4 L1-I2). (Incidentally
  `SP09_std_path_0x12` also landed at `0012f` < 0x10000, an unplanned extra
  confirmation of that code path.)

Per-row fields: `canonical_bytecode_hex`, `strings` (the mk1 string set),
`declared_csid`, `derived_csid` (both 5-hex-digit lowercase zero-padded
strings), `expect_mismatch_warning` (bool), `warning_text` (empty for clean
rows; for the one mismatch row, the spec's contract-2 draft wording with
`12345`/`ef12f` substituted in, generated from a single `contract2_warning_text`
helper — not hand-duplicated prose).

Pinned SHA-256 (`CSID_EXT_SHA256`, tests/csid_ext_vectors.rs) — **separate**
constant from `V0_1_SHA256` (tests/vectors.rs):
`88bbe056e85dde694353475e774a78a00defe75cb8694654c4be1d2467ad68f9`.

## RED → GREEN

RED was produced with a real (non-compile-error) assertion failure: the
`csid_ext` module + placeholder `csid_ext_v0.1.json`
(`{"rows": [], ...}`) were added first, then the reader test was written and
run against that placeholder:

```
FAIL corpus_is_nonempty_and_every_row_recomputes_live
  panicked: csid_ext corpus must not be empty (P0: it must exist and carry
  rows before any downstream surface asserts against it)
FAIL csid_ext_sha256_matches_pin
  left: "32bc91094f3f236ba6ccbf97b1535d474ecffce44b4b40cbf2f9f8f90e83f3f1"
  right: "PLACEHOLDER_RED_STATE_NOT_A_REAL_HASH"
FAIL standard_paths_table_fully_covered
  left: 0
  right: 14
3 tests run: 0 passed, 3 failed
```

GREEN, after implementing the generator extension, running it, and pinning
the real SHA (`cargo nextest run --locked -p mk-codec`):

```
PASS mk-codec::csid_ext_vectors corpus_is_nonempty_and_every_row_recomputes_live
PASS mk-codec::csid_ext_vectors standard_paths_table_fully_covered
PASS mk-codec::csid_ext_vectors csid_ext_sha256_matches_pin
PASS mk-codec::canonical_payload canonical_payload_is_chunk_set_id_invariant
PASS mk-codec::chunk_set_id_determinism an_explicit_chunk_set_id_still_wins
PASS mk-codec::vectors vector_file_sha256_matches_pin
Summary [0.039s] 203 tests run: 203 passed, 0 skipped
```

Full `cargo nextest run --locked -p mk-codec` pass count: **203/203**,
0 skipped (workspace-wide `cargo build --workspace --locked` also clean,
confirming `mk-cli` still compiles against the unchanged public API).
`cargo clippy -p mk-codec --all-targets --features gen-vectors`: clean (one
`type_complexity` warning on `legacy_twin_rows`'s tuple array was fixed with
a `LegacyTwinSpec` type alias; corpus output re-verified byte-identical
before/after that refactor).

## Mutation check (plan-required, P0 is the phase whose "test can fail"
claim needs it)

Backed up `csid_ext_v0.1.json`, then corrupted
`SEED_plate_a_1b1ba`'s `derived_csid` field only (`"1b1ba"` → `"1b1bb"`,
leaving `strings`/`canonical_bytecode_hex` untouched), and reran
`cargo nextest run --locked -p mk-codec --test csid_ext_vectors`:

**Before (corrupted):**
```
FAIL corpus_is_nonempty_and_every_row_recomputes_live
  thread panicked at crates/mk-codec/tests/csid_ext_vectors.rs:101:9:
  assertion `left == right` failed: [SEED_plate_a_1b1ba] derived_csid does
  not reproduce derive_chunk_set_id(encode_bytecode(decode(strings)))
    left: "1b1ba"
    right: "1b1bb"
FAIL csid_ext_sha256_matches_pin (incidental — file bytes changed)
PASS standard_paths_table_fully_covered (unaffected — different assertion)
Summary: 3 tests run: 1 passed, 2 failed
```

The `left: "1b1ba"` is the LIVE recompute (`derive_chunk_set_id(encode_bytecode(decode(strings)))`
run against the row's real, unmutated `strings`); `right: "1b1bb"` is the
corrupted pinned value — direct proof the comparison line executed against
the mutation, not that the corpus merely failed to parse.

**After (restored via `diff` byte-check against the backup, confirmed
identical, SHA re-verified `88bbe0...ad68f9`):**
```
Summary [0.043s] 203 tests run: 203 passed, 0 skipped
```

## Deviations from the plan, with reasons

1. **Row schema wrapper field named `"rows"`, not `"vectors"`.** The plan's
   task brief calls the per-entry unit a "row" throughout; `v0.1.json` uses
   `"vectors"` for its own (differently-shaped) entries. Kept them visually
   distinct since the two files have different schemas (flat row vs.
   input/expected nesting) — a consumer should never treat them as
   interchangeable arrays. Top-level `"schema": 1` and `"family_token"`
   fields mirror `v0.1.json`'s wrapper shape for consistency.
2. **`test_vectors::csid_ext` implemented as a genuine submodule**
   (`test_vectors/csid_ext.rs` + `pub mod csid_ext;` in `mod.rs`), not a
   second flat const beside `V0_1_JSON`. The plan (r1 fold, quoted in the
   IMPLEMENTATION_PLAN P0 section) says "bake the file via `include_str!`
   into a `test_vectors::csid_ext` module" — read literally as a nested
   module path (`mk_codec::test_vectors::csid_ext::CSID_EXT_JSON`), which is
   what P1/P3 will import from.
3. **`warning_text` is the empty string for clean rows**, not a
   would-be-if-mismatched rendering. The task listed `warning_text` as "the
   normative warning content"; for a row where no warning fires there is no
   normative content to pin, and populating it anyway risked being read as
   "this text fires but the flag doesn't," which is exactly backwards.
4. **Leading-zero coverage (r4 L1-I2) got its own dedicated row** rather
   than being folded into one of the 14 STANDARD_PATHS rows, even though
   `SP09` incidentally also satisfies it. Simpler to reason about and
   verify independently; the STANDARD_PATHS search stayed a plain
   natural-derivation loop with no target-matching search inside it.
5. **`v0.1.json` byte-unchanged is NOT re-asserted inside the new test
   file** — deliberately, to avoid a second SHA-256 pin (`tests/vectors.rs`
   already owns `V0_1_SHA256`) that could silently drift out of sync with
   the real one. Verified instead by running `tests/vectors.rs::vector_file_sha256_matches_pin`
   itself (PASS, see GREEN block above) and by `git diff --stat` showing
   zero lines changed in `v0.1.json`.

No other deviations. `mk-codec`'s public API is untouched (no new exports,
no semver event) — P0 only added a generator binary extension, a new baked
corpus module, and its reader tests, exactly as scoped.

## Not done (correctly out of P0 scope, per the plan)

P1 (mk-cli read-side warnings), P2 (mk-cli write-side + repair), P3
(descriptor-mnemonic seat path), P4 (seedhammer Go parity test) — untouched.
`crates/mk-cli` was not modified; the workspace build was checked only as a
sanity confirmation that P0's additions don't break its compile.
