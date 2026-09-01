# Implementation report: `me-cli-csid-warning-surface`

Worktree: `/scratch/code/shibboleth/me-worktrees/csid-warn` (mnemonic-engrave repo),
branch `followup/me-cli-csid-warning`, base `81383fe`. **Not committed** — tree
left dirty/unstaged per instruction, for the controller's own commit(s).

## Step 1 — the mk-codec 0.4 -> 0.5 bump, isolated and proven green first

`crates/me-cli/Cargo.toml`: `mk-codec = "0.4"` -> `mk-codec = "0.5"`.
`cargo update -p mk-codec --precise 0.5.0` moved `Cargo.lock` `mk-codec v0.4.1 ->
v0.5.0` with 34 other dependencies unchanged (`--verbose` line).

Gate run **before any warning code existed**:

- `cargo build --locked` — clean, `mk-codec v0.5.0` compiles into the tree.
- `cargo nextest run --locked` (whole `mnemonic-engrave` workspace, both crates:
  `mnemonic-engrave` + `mnemonic-io-lib`) — **580 tests run: 580 passed, 1
  skipped**, 32.2s wall (24-core parallel). Matches the recon's probe number
  (571 me-cli-crate tests; the 580 whole-package figure additionally includes
  `mnemonic-io-lib`'s own suite, e.g. its slow `fish_history_purge` tests).

The bump is confirmed non-breaking, isolated from the feature work.

## Step 2 — the warning: implementation

**New file** `crates/me-cli/src/csid_warn.rs` (registered `pub mod csid_warn;`
in `crates/me-cli/src/lib.rs`) is the single source of truth, mirroring
mk-cli's `crates/mk-cli/src/cmd/mod.rs` and md-cli's
`crates/md-cli/src/seat/input.rs` (both read to confirm the exact pattern
before writing this):

- `chunk_set_id_mismatch_warning(declared: u32, derived: u32) -> String` — the
  frozen R2/R6 `format!`.
- `declared_chunk_set_id(one_chunk: &str) -> Option<u32>` (private) — reads
  ONE chunk's `mk_codec::string_layer::StringLayerHeader`; `None` on a
  non-`Chunked` header.
- `derived_chunk_set_id(card: &KeyCard) -> Option<u32>` (private) — SPEC "The
  comparison": `derive_chunk_set_id(encode_bytecode(card))`.
- `chunk_set_id_comparison(refs: &[&str], card: &KeyCard) -> Option<(u32, u32)>`
  — public composition of the two above.
- `warn_chunk_set_id_mismatch(comparison: Option<(u32, u32)>)` — the `eprintln!`
  site; no-op on `None` or on a match.

**Exact warning text** (module doc `wording_pin_matches_the_frozen_r6_text`,
independently typed, verified byte-identical to
`mnemonic-key/crates/mk-codec/src/test_vectors/csid_ext_v0.1.json`'s
`SEED_pinned_12345_ef12f.warning_text`):

> warning: this key card's stamped chunk-set id (12345) was not derived from
> its content, which computes ef12f. The card decodes fine, but diagnostics
> that name plates by id will call it 12345. To fix it, re-mint: run mk encode
> again without --chunk-set-id and the id is derived from the key data
> automatically.

### The three surfaces

| Surface | File:line | What changed |
|---|---|---|
| `me bundle` | `crates/me-cli/src/bundle.rs:308-310` | After `let card = mk_codec::decode(&refs)?;` (line 301), one call: `warn_chunk_set_id_mismatch(chunk_set_id_comparison(&refs, &card))`. |
| `me seal` | `crates/me-cli/src/seal/record.rs:253-263`, inside `decode_public_set`'s `('k', _)` match arm | `.map(\|_\| ())` -> `.map(\|card\| { warn_chunk_set_id_mismatch(chunk_set_id_comparison(&set, &card)) })`. `Result<(), String>` shape and the whole-set-decodes-or-refuses contract are unchanged — the closure still evaluates to `()`. |
| `me sysw pack`/`show` | `crates/me-cli/src/sysw/record.rs:212-227`, inside `mdmk_unconfirmed`'s `('k', _)` match arm | `.is_ok()` -> `match mk_codec::decode(&set) { Ok(card) => { warn_..(..); true } Err(_) => false }`. `confirmed`'s boolean CONTROL-FLOW meaning (feeds `if !confirmed { out.extend(idxs) }`) is byte-for-byte unchanged; only the `Ok` arm now also warns before returning `true`. |

All three attach at the point the card is (or, for sysw, becomes) available —
none needed forcing; the task's "no card in scope" escape hatch was not
invoked.

**sysw is reached twice per binary, never twice per invocation.** `mdmk_unconfirmed`
is called from `report_unconfirmed` (`main.rs:1917`, `me sysw pack`) and
`print_mdmk_confirmation` (`main.rs:2076`, `me sysw show`) — each exactly once
per its own command's process, so no card is warned about twice within one
invocation. `testdata/sysw_vectors.json` (the golden-vector self-check`vectors.rs`
calls this from) has zero `12345`-family declared ids (`grep -c 12345` = 0), so
the addition changes no existing sysw fixture's behavior. `mdmk_unconfirmed`'s
doc comment ("returns indices and says nothing else", also cited from
`sysw/mt.rs:89` and `sysw/expect.rs:61`) is now annotated in place to say the
return-value shape is unchanged but the side effect is new — see the updated
doc block at `sysw/record.rs:169-181`.

## Step 3 — fixtures

**No fixture bytes changed.** Checked every place the recon named:

- `crates/me-cli/tests/vectors/bundle-md1-mk1.json`, consumed only by
  `bundle_emits_manifest_json_on_stdout` (`tests/cli.rs`), which asserts
  `stdout` JSON fields only (`wallet_plates`, `sets[0].chunk_set_id`) and that
  stdout excludes `"TYPE ON DEVICE"`. Never touches stderr. Unaffected —
  confirmed green, unmodified.
- `crates/me-cli/src/manifest.rs`'s test module constructs `Manifest` structs
  directly (`grep -c mk_codec::decode` = 0) — never calls the decoder, so the
  warning code path is unreachable from it. Unaffected, unmodified.
- `crates/me-cli/src/bundle.rs`'s own `#[cfg(test)] mod tests` calls
  `run_bundle`/`parse_line` in-process; none of those tests capture or assert
  on stderr (they assert on the returned `Manifest`/`Result` values only), so
  `eprintln!`'s new output is invisible to them. Unaffected, unmodified.
- `crates/me-cli/tests/golden.rs` / `cross_lang.rs` drive `mnemonic_engrave::convert()`
  (the single-string `me` converter, not `bundle`/`seal`/`sysw`) — never
  touches the three warning call sites. Unaffected.

Net: zero existing test files needed behavior changes; only new tests were
added (below).

## RED -> GREEN per surface

RED confirmed by running each new integration test **before** its surface was
wired (mutation-equivalent to "delete the feature"):

| Test | File | Pre-wiring | Post-wiring |
|---|---|---|---|
| `bundle_pinned_mk1_warns_chunk_set_id_mismatch_on_stderr` | `tests/cli.rs` | FAILED (`left: 0, right: 1`, no warning line in stderr) | PASS |
| `bundle_clean_mk1_card_is_silent_on_chunk_set_id` | `tests/cli.rs` | PASS (vacuously — no code existed yet) | PASS |
| `seal_pinned_mk1_warns_chunk_set_id_mismatch_on_stderr` | `tests/seal_cli.rs` | FAILED (`left: 0, right: 1`) | PASS |
| `seal_clean_mk1_card_is_silent_on_chunk_set_id` | `tests/seal_cli.rs` | PASS (vacuous) | PASS |
| `pack_pinned_mk1_warns_chunk_set_id_mismatch_on_stderr` | `tests/sysw_cli.rs` | FAILED (`left: 0, right: 1`) | PASS |
| `show_pinned_mk1_warns_chunk_set_id_mismatch_on_stderr` | `tests/sysw_cli.rs` | FAILED (`left: 0, right: 1`) | PASS |
| `pack_and_show_are_silent_on_a_clean_mk1_card` | `tests/sysw_cli.rs` | PASS (vacuous) | PASS |

The "vacuous PASS" rows are the clean-twin controls, which are silence
assertions true both before the feature existed and after — their value is in
staying PASS post-wiring (proving the feature doesn't over-fire), which was
verified.

Fixture pairs used, each **measured**, not assumed (a throwaway probe test
computed declared/derived via `mk_codec::string_layer::decode_string` +
`mk_codec::derive_chunk_set_id`, then was deleted before final work — not part
of the diff):

- bundle/`csid_warn.rs` unit tests: `MK1_A`/`MK1_B` (`bundle.rs`'s own
  fixture, "mk-codec v0.1.json") — declared `0x12345`, derives `0x83bb2`.
- seal/sysw: the shared `mk1qpz63tp...` pair already used in both
  `seal_cli.rs` and `sysw_cli.rs` — declared `0x16a2b`, derives `0x7a06f`.
- Clean control (all three surfaces): the P0 extension corpus's CT1 twin row
  `CT1_twin_of_V1_bip48_mainnet_1_stub_with_fp`
  (`mnemonic-key/crates/mk-codec/src/test_vectors/csid_ext_v0.1.json`) — same
  key material as `MK1_A`/`MK1_B`, minted without a pin: declared == derived
  == `0x83bb2`.
- `bundle.rs`'s OTHER existing pair, `MK1_C`/`MK1_D` ("mk-codec v0.1.json"
  V2), was ALSO measured and turned out to be a second legacy-pinned
  mismatch (declared `0x23456`, derives `0xf479a`) — not a clean card as
  might be assumed from its use as "a second complete set" in
  `multi_set_two_distinct_mk1_cards`. Not used as this cycle's clean control
  for that reason; the corpus's CT1 row was used instead, since it is
  independently pinned as clean by mk-codec's own `expect_mismatch_warning:
  false`.

## Wording-pin drift guard (R6, "same warning everywhere")

`csid_warn.rs`'s `mod tests` (8 tests, all independent of any I/O):

- `wording_pin_matches_the_frozen_r6_text` — the STRONGEST guard: an
  independently hand-typed literal (not built via the `format!` under test)
  compared byte-for-byte against `chunk_set_id_mismatch_warning(0x12345,
  0xef12f)`, matching the corpus's pinned row verbatim.
- `wording_pin_independent_fragments` — the interim guard md-cli's
  `seating_vectors.rs` uses (substrings: `starts_with("warning:")`,
  `contains("(12345)")`, `contains("computes ef12f")`, the remedy sentence),
  kept as a second, independent check.
- `wording_pin_zero_pads_below_0x10000` — `{:05x}` leading-zero rendering.
- `comparison_detects_the_pinned_mismatch`, `comparison_agrees_on_the_clean_twin`,
  `comparison_is_none_on_empty_input`, `md1_string_does_not_decode_as_mk1`,
  `warn_is_silent_on_a_match_or_none` — the comparison/no-op logic.

One deviation from the literal instruction, disclosed: `comparison_is_none_for_...`
could not be driven with a REAL single-string (unchunked) `KeyCard`, because a
compact xpub alone is 73 bytes — already larger than a regular codex32
string's usable payload (confirmed empirically: every 2-key-card fixture
touched this cycle, in this file and in `bundle.rs`'s own tests, is a 2-chunk
set; `bundle.rs`'s own `Mk1SingleString` doc says "only synthetic <=56-byte
cards hit this"). The test instead exercises the empty-input degenerate case;
the non-`Chunked` match arm is exhaustive over `#[non_exhaustive]
StringLayerHeader` (compiler-checked), so its `None` return is a type-level
guarantee rather than something needing a hand-built wire fixture.

## Mutation gate

Performed on **one** surface (`me bundle`), per instruction:

- **Before** (mutant: the 3-line `warn_chunk_set_id_mismatch(chunk_set_id_comparison(&refs, &card));`
  call at `bundle.rs:308-310` replaced with a comment, nothing else changed):
  `cargo test -p mnemonic-engrave --test cli -- bundle_pinned_mk1_warns_chunk_set_id_mismatch_on_stderr`
  -> **FAILED** (`left: 0, right: 1`, stderr shows the ordinary bundle
  checklist with no mismatch line).
- **Restored** verbatim (confirmed via `Read` against the pre-mutation
  content) and re-ran the whole suite: **595/595 passed, 1 skipped.**

## Final validation (whole `mnemonic-engrave` workspace)

- `cargo build --locked` — clean.
- `cargo nextest run --locked` — **595 tests run: 595 passed, 1 skipped**,
  32.2s (24-core). +15 over the Step-1 baseline (580): 8 `csid_warn` unit
  tests + 2 bundle + 2 seal + 3 sysw integration tests. `grep -c FAIL` on the
  captured log = 0.
- `cargo fmt --check` — clean (exit 0).
- `cargo clippy --all-targets --locked` — clean, 0 warnings (`grep -ci warning`
  on the captured log = 0).

## Files touched

```
 Cargo.lock                       |  5 +--
 crates/me-cli/Cargo.toml         |  2 +-
 crates/me-cli/src/bundle.rs      |  8 ++++
 crates/me-cli/src/lib.rs         |  1 +
 crates/me-cli/src/seal/record.rs | 11 ++++-
 crates/me-cli/src/sysw/record.rs | 29 ++++++++++++-
 crates/me-cli/tests/cli.rs       | 65 +++++++++++++++++++++++++++++
 crates/me-cli/tests/seal_cli.rs  | 76 ++++++++++++++++++++++++++++++++++
 crates/me-cli/tests/sysw_cli.rs  | 88 ++++++++++++++++++++++++++++++++++++++++
 9 files changed, 279 insertions(+), 6 deletions(-)
```

Plus one new, currently untracked file: `crates/me-cli/src/csid_warn.rs`.

**Not committed** — left dirty/unstaged in the worktree for the controller's
own commit(s), per instruction.
