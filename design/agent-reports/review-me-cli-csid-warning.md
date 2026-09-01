# Review — me-cli chunk_set_id mismatch warning + mk-codec 0.4→0.5 bump

**Branch:** `followup/me-cli-csid-warning` (worktree `/scratch/code/shibboleth/me-worktrees/csid-warn`), base `81383fe`.
**Question:** Is this change correct and safe to ship — does the mk-codec 0.4→0.5 bump alter any decode/funds behavior, do the three warning surfaces behave, is there a false-PASS test?
**Verdict: 0 Critical / 0 Important / 0 Minor / 2 Nit. SHIPS.**

All machine-checks run against the real tree, not described:
- `cargo nextest run --locked -p mnemonic-engrave`: **586 passed, 1 skipped** (the skip is `sysw/vectors.rs:132` `#[ignore = "regenerates the fixture"]`, unrelated). All 15 new csid tests ran and passed (verified by name in the captured log).
- `cargo clippy --locked -p mnemonic-engrave --all-targets`: clean, no diagnostics.
- mk-codec 0.4.1 vs 0.5.0 source diffed file-by-file from the cargo registry cache.

---

## 1. Dependency bump blast radius — CLEAN (no decode/funds change)

Diffed every changed `.rs` in mk-codec 0.4.1→0.5.0. me-cli only ever **decodes** mk1 (its one encode call is a test helper, `encode_with_chunk_set_id`, unchanged). Findings:

- **`string_layer/bch.rs` and `string_layer/header.rs`: byte-identical** (no non-comment change). BCH error-correction acceptance (which `validate.rs` relies on) and `StringLayerHeader` fields (`Chunked { chunk_set_id, .. }`, `SingleString`) are unchanged.
- **`error.rs`: doc-comment only.** No variant added/removed/renamed. me-cli's only matched variant, `mk_codec::Error::InvalidHrp` (`validate.rs:40`, `bundle.rs:420`), is intact.
- **`pipeline.rs`: the only decode-adjacent behavioral change is mint-side** — `None => fresh_chunk_set_id()` (CSPRNG) became `None => derive_chunk_set_id(bytecode)`. me-cli never mints, so this does not touch any me-cli path. `getrandom` dropped as a consequence.
- **`bytecode/encode.rs`: added a `PathTooDeep` guard** for paths > `MAX_PATH_COMPONENTS`. This only affects the new warning's re-encode (`derived_chunk_set_id`), and only degrades to *silent* (`.ok()?` → `None` → no warning). It is moreover unreachable for any card me-cli holds: `decode_explicit_path` has *always* refused over-deep paths, so a card that `mk_codec::decode` returned `Ok` for cannot trip the encode-side cap. No legitimate mismatch is silently swallowed.
- **`GENERATOR_FAMILY` "mk-codec 0.2"→"0.5"** (`consts.rs`) does **NOT** poison the derived id. Grep confirms it is used only in the vector-gen binary (`bin/gen_mk_vectors.rs`) and the lib re-export — it is **not** in `encode_bytecode`. So `derive_chunk_set_id(encode_bytecode(card))` is a pure function of card content (stubs, fingerprint, xpub, path), version-stable. This is the load-bearing safety point: the clean-twin control cards derive the same `0x83bb2` under 0.5.0 as when minted, so no clean card spuriously warns.
- **`Cargo.lock`: clean.** Only the `mk-codec` block changed (version + checksum + removed `getrandom` edge). No unexpected transitive bumps. `bitcoin 0.32.101` unchanged.
- New API used (`derive_chunk_set_id`, `bytecode::encode_bytecode`, `string_layer::decode_string`, `StringLayerHeader::from_5bit_symbols`) all resolve and are exercised by passing tests.

## 2. sysw `.is_ok()` change — control-flow IDENTICAL (funds-safe)

`sysw/record.rs:231` `('k', _) => match mk_codec::decode(&set) { Ok(card) => { warn(...); true } Err(_) => false }`. `confirmed` is `true` iff decode succeeded — bit-identical to the old `.is_ok()`. `confirmed` only drives `if !confirmed { out.extend(idxs) }`, so the returned `Vec<usize>` of unconfirmed indices, and therefore what `sysw pack` packs and whether it succeeds, are unchanged. Warning is a pure side effect on the Ok path.

## 3. seal round-trip — Result type/value unchanged

`seal/record.rs:258` `('k', _) => mk_codec::decode(&set).map(|card| { warn(...) }).map_err(...)`. `warn_chunk_set_id_mismatch` returns `()`, so the closure yields `Result<(), _>` exactly as the old `.map(|_| ())` did. Nothing sealed changes; the seal/unseal round-trip is untouched (`seal::tests::every_encrypted_vector_round_trips` still green). Warning is purely additive on Ok.

## 4. False-PASS hunt — tests genuinely assert

- **"warns" tests are not tautologies.** Each pins the real operands: e.g. bundle checks `stderr.contains(chunk_set_id_mismatch_warning(0x12345, 0x83bb2))`. The stderr is produced from the *real* computed `(declared, derived)`; the deterministic warning embeds both values, so the substring matches **only if** the real pair equals the hard-coded pair. `0x83bb2`/`0x7a06f` are thus pinned to the real derivation, not echoed.
- **Wording is independently pinned.** `wording_pin_matches_the_frozen_r6_text` asserts the `format!` output equals a hand-typed frozen literal (`12345`/`ef12f`); `wording_pin_independent_fragments` checks the three segments as substrings. A format drift is caught even though the integration tests use the function on both sides.
- **"silent on clean" checks the RIGHT string** (`!contains("was not derived from its content")`) **for the right reason**: `comparison_agrees_on_the_clean_twin` proves the clean cards decode AND `declared==derived==0x83bb2`, so silence is a genuine match, not a decode failure. (For bundle/seal a non-decoding card would also fail `.success()`, which the clean tests assert.)
- **Single-string deviation is acceptable.** The `SingleString`/non-`Chunked` arm returns `None` (no warning), which is the *correct* R6 behavior (a single-string card has no chunk set to compare). The match is exhaustive over `#[non_exhaustive] StringLayerHeader` (compiler-checked), and the same `SingleString => None` pattern already exists at `seal/record.rs:306`. Driving it via the empty-input `None` rather than real single-string wire bytes leaves no correctness gap.
- No panic paths: every fallible mk-codec call uses `.ok()?`; derivation is pure hashing.

## 5. R6 content parity — byte-identical three ways

Corpus row `SEED_pinned_12345_ef12f.warning_text` (`mnemonic-key/crates/mk-codec/src/test_vectors/csid_ext_v0.1.json:80`) == me-cli's frozen literal (`csid_warn.rs` `wording_pin` test) == md-cli's `format!` string (`descriptor-mnemonic/crates/md-cli/src/seat/input.rs:389`). Same `{:05x}` rendering, same remedy sentence. No drift.

## 6. Exit codes / stdout — unchanged

Warning is emitted via `eprintln!` → stderr only. Tests assert exit 0 (`.success()`) on all three surfaces and unchanged stdout (`bundle` manifest is valid JSON with `wallet_plates == 4`). Each surface warns exactly once (`filter(... ).count() == 1`), including the two independent sysw call sites (`pack`→`report_unconfirmed`, `show`→`print_mdmk_confirmation`). No stdout leak, no exit-code change.

---

## Nits (non-blocking, no owning gate)

- **N1 — fixture name collision across test files.** `MK1_A`/`MK1_B` denote a `0x12345`/`0x83bb2` card in `tests/cli.rs` but a `0x16a2b`/`0x7a06f` card in `tests/sysw_cli.rs` and (`MK1_PINNED_*`) `tests/seal_cli.rs`. Each file resolves correctly; the collision is a readability hazard only, and is documented in the comments. Remedy: rename per file if ever touched.
- **N2 — self-disclosed sibling doc staleness.** `sysw/record.rs`'s new doc notes that `sysw/mt.rs` / `sysw/expect.rs` comments ("returns indices and says nothing else") are now stale about side effects. The return value is genuinely unchanged; updating those two comments is cosmetic.

## Positive notes (not findings)
- The re-encode route for derivation (`derive_chunk_set_id(encode_bytecode(card))`, not raw reassembled bytes) is correct: it detects a foreign encoder whose bytecode canonicalization drifts.
- Because decode already caps path depth, the encode-side `PathTooDeep` cannot silence a real mismatch for any decodable card — the warning is total over decoded chunked mk1 sets.
