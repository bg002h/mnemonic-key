# P2 implementation report — chunk_set_id verification (mint warning + repair blessed-path warning)

Worktree: `/scratch/code/shibboleth/mk-worktrees/csid-p0`, branch `impl/csid-p0`
(P0 `58c8df4` + P1 `37a9524` already committed there; P2 left UNCOMMITTED per
instruction). Plan: `design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md`
P2. Spec: `design/SPEC_chunk_set_id_verification.md` contracts 2 (repair
coverage, r4 L2-I1/I2) and 5 (mint).

## Files touched

- `crates/mk-cli/src/cmd/encode.rs` — +44/-2 (import, mint-loop insertion,
  `warn_pinned_chunk_set_id_mismatch` helper).
- `crates/mk-cli/src/cmd/repair.rs` — +52/-9 (`classify_mk1_set` return type
  gains blessed `(GroupKey, KeyCard)` pairs; `run()` plumbs them and emits the
  warning).
- `crates/mk-cli/tests/encode_repair_chunk_set_id_p2.rs` — new file, 6 tests,
  325 lines.

No changes to `mk-codec`, no `--json` schema changes, no exit-code changes.

## Design notes (where this deviates from the literal plan text, and why)

- **Mint (a):** rather than calling `mk_codec::derive_chunk_set_id(mk_codec::
  encode_bytecode(&card))` directly, the mint site reuses P1's public
  `crate::cmd::chunk_set_id_comparison(&strings, &card)` on the JUST-MINTED
  strings. This is strictly more correct: when the bytecode is short enough
  for single-string output, `encode_with_chunk_set_id` silently ignores the
  pin (its own doc comment: "only consulted on the chunked path") — reading
  the comparison back off the actual minted strings makes that case `None`
  (no warning) for free, instead of a naive direct comparison that would
  warn about a pin the codec never wrote anywhere. Confirmed by
  `chunk_set_id_comparison`'s existing semantics (`declared_chunk_set_id`
  returns `None` for a `SingleString` header).
- **Repair (b):** `classify_mk1_set`'s Bless arm now pushes `(*key, card)`
  into a `blessed: Vec<(GroupKey, KeyCard)>` returned alongside `SetVerify`.
  `run()` iterates it after the report/advisory are printed, uses
  `GroupKey::Chunked(declared)` directly (no re-decode of any string — the
  wire-header id was already parsed into the `GroupKey` during
  classification) and `crate::cmd::derived_chunk_set_id(card)` — a P1
  helper that is `fn` (private to `cmd`), not `pub`. Rust's privacy rule
  (visible in the defining module **and its descendants**) makes it callable
  from `cmd::repair` and `cmd::encode` alike; confirmed by a clean build, not
  assumed. `GroupKey::SingleString` groups are skipped (`let GroupKey::
  Chunked(declared) = key else { continue }`) — matches contract 2's "None
  for single-string input, nothing to compare."
- Mint-time clause is appended to P1's frozen `chunk_set_id_mismatch_warning`
  text (not a separate wording), per plan r4 L2-I2.

## RED → GREEN

Pre-impl (RED), `cargo nextest run -p mk-cli --test
encode_repair_chunk_set_id_p2`: **3/6 failed** — `mint_pinned_mismatch_...`,
`repair_blessed_damaged_pinned_card_...`, and `repair_json_byte_unchanged_...`
(its last assertion, the mint-time clause). The 3 "must stay silent"
guarantees passed immediately (nothing to break pre-cycle).

Post-impl (GREEN): **6/6 passed.**

### Mint — mismatched pin (RED→GREEN evidence)

`mk encode --xpub <V1_XPUB> ... --chunk-set-id 0x12345` (fixture's real
derived id, measured live: `83bb2`). Exit 0, mk1 strings still on stdout.
Exact stderr:

```
warning: --chunk-set-id pins 12345 in place of the content-derived id 83bb2. Cards minted this way trip a mismatch warning in every conforming decoder, forever. For test fixtures only — never engrave this on a real plate. To mint a real plate, drop --chunk-set-id entirely and the id is derived from the key data automatically. Do not re-type the derived value into the flag: one mistyped character mints a mismatched plate.
```

### Mint — pin EQUAL to derived id (silent, evidence)

`mk encode ... --chunk-set-id 83bb2` (== the fixture's real derived id):
exit 0, stderr contains no `--chunk-set-id pins` line at all — confirmed both
by the automated test and by live capture (`diff` of stdout against the
unpinned mint: byte-identical).

### Repair blessed — damaged pinned card (RED→GREEN evidence)

Row `SEED_pinned_12345_ef12f` (declared `12345`/derived `ef12f`, 2 chunks),
chunk 1 damaged by one bech32-symbol substitution at data-part position 20
(`g`→`f`, well past the 8-symbol header, single substitution trivially within
t≤4). `mk repair <chunk0> <damaged_chunk1>`:

- exit **5**
- stdout: `# Repair report` + the correction line + both chunks, chunk 1
  reproduces the original undamaged string exactly.
- stderr:
  ```
  warning: this key card's stamped chunk-set id (12345) was not derived from its content, which computes ef12f. The card decodes fine, but diagnostics that name plates by id will call it 12345. To fix it, re-mint: run mk encode again without --chunk-set-id and the id is derived from the key data automatically. this id was set when the card was minted; the repair did not change it.
  ```

### Repair silent — evidence

- **Undamaged pinned card** (both chunks unmodified): exit **0**, stderr
  carries neither "was not derived from its content" nor "chunk-set id".
- **Single damaged chunk supplied alone** (Candidate): exit **5**, stderr
  carries `UNVERIFIED` (the pre-existing advisory, unaffected) but neither
  "was not derived from its content" nor "chunk-set id" — the warning never
  fires because the group never reaches `mk_codec::decode` (incomplete).

### `repair --json` byte-unchanged (proof)

Golden captured LIVE against the **pre-P2** binary this session for
`mk repair --json <chunk0> <damaged_chunk1>`:

```json
{"schema_version":"1","kind":"mk1","corrected_chunks":["mk1qpzg69pqqsqsqrrhvket9v4jq5zg3vs7zqsrq9dlh7lml0alh7lml0alh7lml0alh7lml0alh7lml0alh7lml0alhupawhtfl552clzu3rgv","mk1qpzg69ppjd334aa2pecfgwwagl7qqxkdpvrjwectvecw5552eq7tqynlfth397uhnqu3pd7wy4mw3"],"repairs":[{"chunk_index":1,"original_chunk":"mk1qpzg69ppjd334aa2pecffwwagl7qqxkdpvrjwectvecw5552eq7tqynlfth397uhnqu3pd7wy4mw3","corrected_chunk":"mk1qpzg69ppjd334aa2pecfgwwagl7qqxkdpvrjwectvecw5552eq7tqynlfth397uhnqu3pd7wy4mw3","corrected_positions":[{"position":20,"was":"f","now":"g"}]}]}
```

Post-P2 stdout for the identical invocation is asserted byte-equal to this
string (`assert_eq!`) in `repair_json_byte_unchanged_no_chunk_set_id_field`,
which PASSES — the warning appears on stderr only (also asserted present
there in the same test); the JSON envelope carries no `chunk_set_id` field.

## Mutation gates

**Mint recompute deleted** (call site replaced with `let _ =
chunk_set_id_comparison(...)`, discarding the result — compiler emitted a
`dead_code` warning on the now-uncalled `warn_pinned_chunk_set_id_mismatch`,
proving the line was actually removed from the call path):
`mint_pinned_mismatch_warns_stderr_exit0_strings_still_mint` **FAILED**
(only that test; other 5 stayed green). Restored; `diff` against pre-mutation
copy: identical.

**Repair blessed recompute deleted** (`derived_chunk_set_id(card)` call
replaced with `None::<u32>`): `repair_blessed_damaged_pinned_card_warns_
with_mint_time_clause` **and** `repair_json_byte_unchanged_no_chunk_set_id_
field` (its stderr assertion) **FAILED**; `repair_undamaged_pinned_card_
exit0_silent` and `repair_single_chunk_candidate_of_pinned_card_silent`
stayed **green**, confirming the mutation is isolated to the blessed path.
Restored; `diff` against pre-mutation copy: identical.

## Whole-surface gate

- `cargo nextest run --locked -p mk-cli -p mk-codec`: **387/387 passed**
  (381 pre-P2 + 6 new P2 tests; 0 failed, 0 skipped).
- `cargo clippy --all-targets --workspace`: clean, no warnings.
- Named guarantees, individually re-run: `an_explicit_chunk_set_id_still_wins`
  and `canonical_payload_is_chunk_set_id_invariant` both PASS, unchanged.
- Corpus pins unchanged (both already asserted by passing tests
  `vector_file_sha256_matches_pin` / `csid_ext_sha256_matches_pin`, and
  independently confirmed via `sha256sum`):
  `v0.1.json` = `c3a13b67...b4f1123`,
  `csid_ext_v0.1.json` = `88bbe056...2467ad68f9` (both files untouched by
  this phase — no write path in P2 touches either).
- `git status --short`: only `crates/mk-cli/src/cmd/encode.rs` (M),
  `crates/mk-cli/src/cmd/repair.rs` (M), `crates/mk-cli/tests/
  encode_repair_chunk_set_id_p2.rs` (new, untracked). Nothing staged, nothing
  committed, per instruction. P3 (`descriptor-mnemonic`) and P4 (fork) not
  touched.

## Deviations from the literal task text (both justified above, none blocking)

1. Mint comparison computed via `chunk_set_id_comparison(&strings, &card)`
   (reusing the P1 read-side helper on the freshly-minted strings) rather
   than a direct `mk_codec::derive_chunk_set_id(mk_codec::encode_bytecode(&card))`
   call in encode.rs — more correct on the single-string-ignored edge case,
   same underlying computation.
2. Repair's declared id is read from the already-parsed `GroupKey::Chunked`
   rather than re-derived via `chunk_set_id_comparison` on the group's raw
   strings — avoids a redundant string re-decode; the value is identical
   (both ultimately come from the same post-correction wire header).
