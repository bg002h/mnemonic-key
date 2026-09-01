# Whole-diff adversarial review — chunk_set_id recompute-and-report (P0–P4, three repos)

**Verdict: 0 Critical / 0 Important / 2 Minor / 1 Nit (already-known).**
The implementation ships. Every cross-cutting failure mode the brief named was
machine-checked and holds. The two Minors are latent drift-guards, not current
defects; the Nit is the one already logged.

Reviewed:
- mnemonic-key `725ccb9..impl/csid-p0` (P0 corpus + P1 read-side + P2 mint/repair)
- descriptor-mnemonic `044e33d4..impl/csid-p3` (P3 seat warning + R5 rewrite)
- seedhammer `5f02773..impl/csid-p4` (P4 Go derivation parity)

Method: read every diff in full; extracted and rendered the actual warning
strings from all three sources and diffed them; cross-checked the Go parity
literals against the Rust corpus; verified the corpus SHA pin, row counts, table
coverage, and the derivation algorithm parity between Rust and Go.

---

## Machine-checked and CORRECT (the load-bearing claims)

1. **R6 cross-repo warning parity — byte-identical.** Rendered
   `chunk_set_id_mismatch_warning(0x12345, 0xef12f)` from mk-cli
   (`crates/mk-cli/src/cmd/mod.rs`), from md-cli
   (`crates/md-cli/src/seat/input.rs`), and the corpus generator's
   `contract2_warning_text` (`crates/mk-codec/src/bin/gen_mk_vectors.rs`),
   resolving Rust `\`-continuation semantics, then compared to the on-disk
   `csid_ext_v0.1.json` `warning_text` for `SEED_pinned_12345_ef12f`. **All four
   are byte-equal.** The mint warning (contract 5) is a deliberately distinct
   string. No cross-repo id/content divergence exists.

2. **`{:05x}` at every id-render site.** Swept all three repos. Every
   chunk_set_id interpolation uses `:05x` or `GroupId::Display` (which is
   `write!(f, "{id:05x}")`, with explicit `0 -> "00000"` / `0xFFFFF -> "fffff"`
   unit tests at `md-cli seat/input.rs:759-761`). The only unpadded
   interpolations are chunk COUNTS (`declared_total`, `received`) and an address
   index — none is an id. Leading-zero path is exercised by `LZ1` (`0191c`) and
   `SP09_std_path_0x12` (`0012f`) in both the Rust corpus and the Go table.

3. **Exit-code / stdout contracts intact.** All R2 warnings go to stderr; the
   warning code runs strictly AFTER `mk_codec::decode(&refs)?`, so a card that
   fails to decode returns early and never warns, and single-string input yields
   `declared == None` → no-op (no behavior change for non-chunked input).
   `verify` text mode appends the mismatch line AFTER the `OK:` verdict (the
   `OK:` line still leads — asserted); `verify --json` gains only the additive
   `chunk_set_id` object with `schema_version` held at integer `1`;
   `inspect --json` gains nothing; `inspect` text adds one `chunk_set_id: …
   (stamped)` line. Seat notes render via `emit_seating_notes` →
   `eprintln!("{n}")` (stderr, no prefix), so stdout is byte-identical between a
   warned run and its clean twin (V-CSID-WARN test asserts this).

4. **`repair --json` byte-identity (D27).** The envelope emission is untouched;
   `repair_json_byte_unchanged_no_chunk_set_id_field` pins the full output
   against a captured `PRE_P2_REPAIR_JSON` constant and asserts no
   `chunk_set_id` field. The classifier's return-type change plumbs the blessed
   `KeyCard` out (no second decode) and does not touch the JSON path.

5. **mk_codec vs md_codec collision (P3).** The seat path calls fully-qualified
   `mk_codec::bytecode::encode_bytecode` + `mk_codec::derive_chunk_set_id`
   (`seat/input.rs:274-277`), with a doc comment naming the md_codec footgun.
   The seat test hardcodes the expected mk-side derived id `0x69f0e`; a
   wrong-namespace call would compute a different value and fail the assertion.

6. **Derivation parity is real.** Rust `derive_chunk_set_id`
   (`string_layer/chunk.rs:45`) = `sha256(bytecode)` then
   `(h0<<12)|(h1<<4)|(h2>>4)`. Go `top20` (`mk/encode.go:331`) hashes internally
   with the identical extraction. **All 20 Go parity literals match the Rust
   corpus `(canonical_bytecode_hex, derived_csid)` byte-for-byte, and all 20
   clean Rust rows are covered** (verified with a script diffing the two).

7. **Corpus integrity.** Committed `csid_ext_v0.1.json` SHA-256 ==
   pinned `CSID_EXT_SHA256`. 21 rows = 3 legacy twins + 3 seed + 14 SP + 1 LZ;
   exactly 1 mismatch row (`SEED_pinned_12345_ef12f`); 20 clean. SP rows (14) ==
   `STANDARD_PATHS.len()` (14, verified by reading the table).

8. **False-PASS hunt — assertions bite.** Mismatch and clean twins differ ONLY
   in the stamped id (`SEED_pinned_12345_ef12f` and `SEED_plate_b_ef12f` share
   identical `canonical_bytecode_hex`), isolating the declared-vs-derived compare.
   Silent assertions check absence of the RIGHT distinctive phrase
   ("was not derived from its content"). mk-cli's P1 warning assertion reads the
   corpus `warning_text` live and requires stderr to contain it — a real
   corpus↔mk-cli content bind. The R5 classification-order row exercises a
   supply matching two arms' predicates and asserts it lands in the earlier
   (merged) arm.

9. **R5 classifier is total and correctly ordered.** `classify` reads retained
   headers; arm 3 has no precondition (falls through to `mk_codec::decode`, then
   `terminal_refusal` on Err / clean-reassembly + possible R2 warning on Ok).
   `infos[0]` is guarded by the `!infos.is_empty()` check. `group_key_of`'s
   tuple-return change is consistently migrated (only callers: `decode_cards`
   and `group_id_of`). Retired wording ("re-mint one of them", "Two DIFFERENT
   cards pinned") asserted absent.

---

## Findings

### Minor 1 — md-cli's R6 warning content is not test-bound to the corpus (latent drift, currently correct)

**Repo/file:** descriptor-mnemonic `crates/md-cli/src/seat/input.rs:284`
(`chunk_set_id_mismatch_warning`) and its tests in
`crates/md-cli/tests/seating_vectors.rs` / the `input.rs` unit test.

**What:** md-cli carries a SECOND hand-maintained copy of the frozen R2/R6
wording. I verified it is byte-identical to mk-cli's copy and the corpus
`warning_text` **today**. But no test binds md-cli's output to the normative
corpus text or to mk-cli:
- The unit test asserts `warnings[0] == chunk_set_id_mismatch_warning(0x99999,
  0x69f0e)` — comparing md-cli's output to md-cli's OWN function (self-referential
  for wording; it does meaningfully pin the derived id `0x69f0e`).
- The integration test asserts substrings only: `(99999)`, `computes 69f0e`,
  `starts_with("warning:")` — not the remedy sentence, and not the corpus text.

The spec Acceptance says the seat warning "asserts the corpus row's
`warning_text`" (as mk-cli's P1 test does via `pinned_warning_text`). A future
edit to mk-cli's or the corpus's wording — with the P1 test updated — would NOT
flag md-cli drifting, silently violating R6 ("same warning everywhere").

**Why not Important:** R6 parity holds in fact right now (byte-verified), so
there is no current wrong result; md-cli's output does contain the pair, the
distinctive phrase, and (via its own function) the full remedy.

**Remedy (one test):** in `seating_vectors.rs`, read
`mk_codec::test_vectors::csid_ext::CSID_EXT_JSON`, pull the
`SEED_pinned_12345_ef12f` `warning_text`, and assert
`input::chunk_set_id_mismatch_warning(0x12345, 0xef12f)` equals it — mirroring
P1's `pinned_warning_text`. That makes the cross-repo parity structural.

### Minor 2 — Go parity table is not bound to the Rust corpus (already filed post-cycle; currently correct + complete)

**Repo/file:** seedhammer `mk/chunk_set_id_parity_test.go`.

**What:** the 20 clean rows are hand-carried literals; `TestChunkSetIDDerivationParity`
is self-contained (`top20(bytecodeHex) == derivedCSID`). Nothing binds these
literals to the Rust corpus, so a Rust corpus regeneration that changes a value
would not fail the Go test. I verified all 20 rows match the Rust corpus
byte-for-byte and cover all 20 clean rows, and the `csidExtCleanRowCount == 20`
length guard catches a silently-dropped row. This is the **already-spec-filed**
limitation (`go-mk-vector-corpus-ingestion`, Not-in-scope) — recorded here as
confirmation, not a new action. No change required this cycle.

### Nit 1 — repair mint-time clause lowercase after a period (already logged)

`crates/mk-cli/src/cmd/repair.rs`: the appended clause renders "…derived from
the key data automatically. this id was set when the card was minted…" —
lowercase "this" after a period. Cosmetic; this is the KNOWN nit named in the
review brief. Not materially worse than cosmetic. No re-raise.

---

## Coverage note

No build gate applies (plan §"Build gate": the plan wires two public functions
and edits strings; no extractable rust blocks). The per-surface `cargo nextest`
suites + corpus round-trip are the gate. The brief states the controller already
ran all three suites GREEN live; this review did not re-run them (they re-confirm
known-green) and instead spent budget on the cross-repo drift/parity/false-PASS
surfaces that no single-repo suite reaches — all of which check out.
