# FIX_mk_fa_line_readers: bughunt-1119 F-A, mk-cli line readers (bh-mk-I3, bh-mk-I4)

Small-fix track, grade I (both keys). Repo mnemonic-key, branch `fix/mk-fa-line-readers`, base `origin/main`
af12fea ("FOLLOWUPS: security sweep step 3 grades"). Toolkit manual
`docs/manual/src/40-cli-reference/44-mk-cli.md` changes in lockstep (separate toolkit change, section 7).
Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-23.

Inputs (decided; not re-ruled here): coordinator `agent-reports/architect-bughunt-1119-questions.md` row #42 and
P1; Brian's ruling B1 (`memory/product-rulings-sweep-bughunt-2026-11-22.md`: "near-canonical input: normalise BOM
+ CRLF only, refuse every other variant"); grader `agent-reports/bughunt-1119/grade-mk.md` (I3, I4, F-A); triage
`triage-mk-t1.md` (I3, I4 RED tests and scratch patch); consultant report
`design/agent-reports/bug-hunt-2026-11-19.md` I3 (L101-111), I4 (L113-125).

## 1. Defects (at af12fea; rows in section 5 run and seen)

- **I3** `read_key_records` (`crates/mk-cli/src/keyfile.rs:108-160`, used by `mk encode --keys` and `mk encode
  --in`, file or stdin). The R2 2026-08-21 guard at :132 refuses a CR only when the input holds no LF at all. In a
  file that holds one LF anywhere, a bare CR (a CR not followed by LF) passes; `lines()` does not split on it, and
  the comment strip at :145 (`split('#')`) then discards everything after a `#`, including a whole record that sat
  after the CR. `R0 # first<CR>R1<LF>` mints ONE card at exit 0 (a short bundle).
- **I4** `read_md1_set` (`crates/mk-cli/src/cmd/encode.rs:595-613`, `mk encode --from-md1-set`). No BOM strip,
  and no CR policy. The reader skips every line that does not start with `md1` after
  `strip_display_separators` (which does not strip U+FEFF), so a BOM-led first line, a first line glued to a
  `# note<CR>` comment, or a U+FEFF at the start of a later line (two files joined with `cat`) each silently drop
  that md1 string; one surviving md1 line suppresses the empty-file refusal. The card mints at exit 0 with a
  different stub array (and, for a chunked policy, a broken set).

Root cause shared by both: no single policy for the two input-encoding variants a hand-edited or cross-platform
file carries. B1 gives it: normalise a leading BOM and CRLF; refuse every other CR and every other U+FEFF.

Out of scope (measured, not changed): `read_mk1_strings` / `push_stripped_lines` (`crates/mk-cli/src/cmd/mod.rs:438-485`,
the mk1 `--in` / `-` reader of decode, inspect, verify, address, derive, repair). There, every non-empty line becomes
one string and goes to the codec, so a BOM or bare CR yields a loud codec refusal, never a silent drop (no I3/I4
class). Proposed as a FOLLOWUPS line in the author's report (B1 says a leading BOM should be normalised there too).
The key-record grammar's own whitespace and `#`-comment handling, and `--from-md1-set`'s skipping of non-md1 lines,
are the documented formats and do not change.

## 2. Decided behaviour

One rule, applied by both readers (`read_key_records` and `read_md1_set`) to the whole input text before any line
is parsed or skipped (comment and skipped lines included):

1. **A UTF-8 BOM (U+FEFF) as the first character of the input is removed** (normalised). Unchanged for the key
   reader; new for `--from-md1-set`.
2. **CRLF is a line end** (normalised): a CR immediately followed by LF. Unchanged in effect for both.
3. **Key reader only, unchanged:** if the input (after rule 1) holds a CR and no LF at all, the existing CR-only
   refusal fires first, with its text unchanged:
   `{flag} {source}: the file uses CR-only line endings (classic Mac), so every record after the first would be read as part of line 1. Convert it to LF or CRLF.`
4. **Otherwise the input is scanned from the start for the first "bare CR" (a CR not immediately followed by LF;
   a CR as the last character of the input counts) or "stray BOM" (a U+FEFF at any position after rule 1). The
   first one found, by position, is refused** at exit 64 (`UsageError`), naming the 1-based line it is on: 1 plus
   the number of LF characters before it, counted in the input after rule 1 (the same numbering the key reader's
   record errors already use). Nothing is minted; stdout is empty (text mode) or holds only the `--json` error
   envelope.
5. For `--from-md1-set`, a CR-only file (no LF) is refused by rule 4 (the first CR, line 1); there is no separate
   CR-only text for this reader.

`{flag}` is the flag the caller gave (`--keys` or `--in`), `{source}` is the path as given or `<stdin>` (the key
reader's existing `source`), `{path}` the `--from-md1-set` value as given, `{n}` the line. Repeated
`--from-md1-set` files are each checked when read, in flag order (the existing read order); the first file with a
violation is the one named.

### 2.1 Texts (all PROPOSED, architect confirms; printed as `error: <text>` on stderr, or as the `message` of the
`{"error":{"details":null,"exit_code":64,"kind":"UsageError","message":"<text>"},"schema_version":1}` envelope on
stdout under `--json`)

- T-KCR (key reader, bare CR):
  `{flag} {source}:{n}: this line holds a carriage return (CR) with no line feed (LF) after it, so the text after the CR would be read as part of line {n} and a key record after it could be lost. Convert the file to LF or CRLF line endings.`
- T-SCR (`--from-md1-set`, bare CR, including a CR-only file):
  `--from-md1-set {path}:{n}: this line holds a carriage return (CR) with no line feed (LF) after it, so the text after the CR would be read as part of line {n} and an md1 string after it could be skipped. Convert the file to LF or CRLF line endings.`
- T-KBOM (key reader, stray BOM):
  `{flag} {source}:{n}: this line holds a byte-order mark (U+FEFF), which is accepted only as the first character of the file. Remove it (joining two files can leave the second file's mark inside the result).`
- T-SBOM (`--from-md1-set`, stray BOM):
  `--from-md1-set {path}:{n}: this line holds a byte-order mark (U+FEFF), which is accepted only as the first character of the file. Remove it (joining two files can leave the second file's mark inside the result).`

### 2.2 Edge cases (decided here by B1; listed so the reviewer can check each against the ruling)

- A bare CR inside a `#` comment of a key file, or on a line `--from-md1-set` would skip: refused (rule 4 runs
  before comment stripping and skipping; this is I3's exact shape).
- A lone CR as the last character of the input (`R0<LF>R1<CR>`): refused, line 2. At base this case is harmless
  (exit 0, two cards; `trim()` removes the CR). B1 normalises CRLF "only", so a lone CR is not normalised; the
  behaviour change is deliberate and has its own row.
- A U+FEFF that is the first character of the input is removed once; a second U+FEFF right after it is a stray
  BOM on line 1.
- A key file with a stray BOM: at base already refused (exit 64), but as bad BIP-380 notation, which never names
  the cause; it now gets T-KBOM (one policy for both readers).
- CRLF, LF and a leading-BOM file mint exactly what the plain LF file mints (controls).

## 3. Spec delta

mk-cli has no reader spec of its own; the binding statement is the rustdoc on each reader plus the manual rows in
section 7. The builder:

- `keyfile.rs:97-107` rustdoc of `read_key_records`: add "Input encoding (B1, FIX_mk_fa_line_readers): a leading
  UTF-8 BOM is removed and CRLF is a line end; any other CR, and any U+FEFF after the first character, is refused
  naming its line, before comments are stripped." Replace the :120-130 comment block's claim "CRLF needs no special
  case" with a pointer to the shared rule.
- `encode.rs:580-594` rustdoc of `read_md1_set`: add the same sentence ("... before any line is skipped").
- PROPOSED (architect confirms where it lands): the grader suggested one sentence in mnemonic-engrave
  `design/SPEC_constellation_cli_uniformity.md` §6b (channels). Proposed sentence, if the architect wants the rule
  constellation-wide: "A tool reading a line-oriented input file removes a leading UTF-8 BOM and reads CRLF as a
  line end; any other CR, and any U+FEFF after the first character, is refused naming its line (B1, 2026-11-22)."
  This FIX does not edit mnemonic-engrave.

## 4. Code (anchors at af12fea; builder writes the code)

Source files this FIX changes:
- `crates/mk-cli/src/keyfile.rs` (`read_key_records` :108-160; one shared checker used by both readers, placed
  here or in `crates/mk-cli/src/format.rs`, builder's choice; it takes the reader-specific text as input so the
  four texts stay exactly as in 2.1)
- `crates/mk-cli/src/cmd/encode.rs` (`read_md1_set` :595-613: strip one leading BOM, then the shared check, then
  the existing `lines()` loop unchanged)
- `crates/mk-cli/tests/bh1119_line_readers.rs` (new, section 6)
- `scripts/mk-fa-line-readers-rows.sh` and its `SCRIPTS.md` row (committed with this document)
- `CHANGELOG.md` (mk-cli entry under the next unreleased version: "`mk encode --keys/--in` and `--from-md1-set`
  refuse a bare carriage return or a byte-order mark after the first character, naming the line;
  `--from-md1-set` now accepts a leading byte-order mark (bh-mk-I3, bh-mk-I4)." PROPOSED wording.)

The scratch patch in `triage-mk-t1.md` is NOT the fix: its I4 half splits on bare CR (normalises), which B1
refuses.

## 5. Rows

Fixture bytes (public keys only): `R0` = `[73c5da0a/48'/0'/0'/2']` + X0, `R1` = `[73c5da0a/48'/0'/1'/2']` + X1
(mk-cli test corpus); `A`, `B` the template-md1 pair; `KA` the four-chunk
`KEYED_POLICY_A` of `from_md1_set.rs`; `<CR>` `<LF>` `<BOM>` the bytes 0d, 0a, ef bb bf. Base commands:
`KEYS` = `mk encode --keys F --policy-id-stub deadbeef [--json]`; `SET` = `mk encode --xpub X84 --origin-path
"m/84'/0'/0'" --origin-fingerprint 73c5da0a --from-md1-set F`, compared byte-for-byte against the same command on
`A<LF>B<LF>` (`want_set`); `CHUNK` = `mk encode --xpub X0 --origin-fingerprint 73c5da0a --origin-path
"m/48'/0'/0'/2'" --from-md1-set F`, compared against the plain four-line file (`want_chunk`).
All rows are in `scripts/mk-fa-line-readers-rows.sh red|green` (exact commands and bytes); RED column = run at
af12fea (section 5.1 holds the run).

| row | input | RED (af12fea) | GREEN |
| --- | --- | --- | --- |
| `fa_i3_keys_file_bare_cr_after_comment_refused` | KEYS on `R0 # first<CR>R1<LF>` | exit 0, `"card_count":1` | exit 64, stdout empty, stderr exactly `error: ` T-KCR[--keys, k_cr_comment.txt, 1] |
| `fa_i3_keys_file_bare_cr_after_comment_refused_json` | as above + `--json` | (RED row above) | exit 64, stderr empty, stdout exactly the envelope with T-KCR[--keys, k_cr_comment.txt, 1] |
| `fa_i3_keys_stdin_bare_cr_after_comment_refused` | same bytes on stdin, `--keys -` | exit 0, `"card_count":1` | exit 64, `error: ` T-KCR[--keys, <stdin>, 1] |
| `fa_i3_in_file_bare_cr_after_comment_refused` | same file via `--in` | exit 0, `"card_count":1` | exit 64, `error: ` T-KCR[--in, k_cr_comment.txt, 1] |
| `fa_i3_keys_trailing_bare_cr_refused` | KEYS on `R0<LF>R1<CR>` | exit 0, `"card_count":2` (behaviour change, 2.2) | exit 64, `error: ` T-KCR[--keys, k_trailing_cr.txt, 2] |
| `fa_i3_keys_bom_after_first_char_refused` | KEYS on `R0<LF><BOM>R1<LF>` | exit 64, `error: k_bom_mid.txt:2: expected BIP-380 origin notation ... got "\u{feff}[73c5da0a/...` (loud, cause not named) | exit 64, `error: ` T-KBOM[--keys, k_bom_mid.txt, 2] |
| `fa_i4_md1_set_leading_bom_normalised` | SET on `<BOM>A<LF>B<LF>` | exit 0, stdout differs from want_set (A dropped) | exit 0, stdout identical to want_set |
| `fa_i4_md1_set_leading_bom_chunked_normalised` | CHUNK on `<BOM>KA[0]<LF>..KA[3]<LF>` | exit 1, `error: md1 input rejected: chunk set incomplete: got 3 chunks, expected 4` (first chunk dropped; a valid file refused) | exit 0, stdout identical to want_chunk |
| `fa_i4_md1_set_bare_cr_after_comment_refused` | SET on `# note<CR>A<LF>B<LF>` | exit 0, stdout differs from want_set | exit 64, `error: ` T-SCR[s_cr_comment.txt, 1] |
| `fa_i4_md1_set_bom_after_first_char_refused` | SET on `A<LF><BOM>B<LF>` | exit 0, stdout differs from want_set (B dropped) | exit 64, `error: ` T-SBOM[s_bom_mid.txt, 2] |
| `fa_i4_md1_set_cr_only_refused` | SET on `A<CR>B<CR>` | exit 1, `error: md1 input rejected: codex32 decode error: character '1' not in codex32 alphabet` (A and B glued; cause not named) | exit 64, `error: ` T-SCR[s_cr_only.txt, 1] |
| `fa_i4_md1_set_trailing_bare_cr_refused` | SET on `A<LF>B<CR>` | exit 0, stdout identical to want_set (behaviour change, 2.2) | exit 64, `error: ` T-SCR[s_trailing_cr.txt, 2] |
| `fa_control_keys_lf_mints_two` | KEYS `--json` on `R0 # first<LF>R1<LF>` | exit 0, `"card_count":2` | same |
| `fa_control_keys_crlf_mints_two` | KEYS `--json` on `R0<CR><LF>R1<CR><LF>` | exit 0, `"card_count":2` | same |
| `fa_control_keys_leading_bom_mints_two` | KEYS `--json` on `<BOM>R0<LF>R1<LF>` | exit 0, `"card_count":2` | same |
| `fa_control_keys_cr_only_text_unchanged` | KEYS on `R0<CR>R1<CR>` | exit 64, `error: ` + the unchanged CR-only text (rule 3) | same |
| `fa_control_md1_set_crlf_equals_lf` | SET on `A<CR><LF>B<CR><LF>` | exit 0, identical to want_set | same |

### 5.1 RED run at af12fea

`CARGO_BUILD_JOBS=2 scripts/mk-fa-line-readers-rows.sh red` at af12fea, cargo 1.85.0, script sha256
c2cb3d7dbd2ba920de2fdc643dd013f8ef74d9dfd546a98c709ed99d56badcbe:

```
OK   red fa_i3_keys_file_bare_cr_after_comment_refused rc=0
OK   red fa_i3_keys_stdin_bare_cr_after_comment_refused rc=0
OK   red fa_i3_in_file_bare_cr_after_comment_refused rc=0
OK   red fa_i3_keys_trailing_bare_cr_refused rc=0
OK   red fa_i3_keys_bom_after_first_char_refused rc=64
OK   red fa_i4_md1_set_leading_bom_normalised rc=0
OK   red fa_i4_md1_set_leading_bom_chunked_normalised rc=1
OK   red fa_i4_md1_set_bare_cr_after_comment_refused rc=0
OK   red fa_i4_md1_set_bom_after_first_char_refused rc=0
OK   red fa_i4_md1_set_cr_only_refused rc=1
OK   red fa_i4_md1_set_trailing_bare_cr_refused rc=0
OK   red fa_control_keys_lf_mints_two rc=0
OK   red fa_control_keys_crlf_mints_two rc=0
OK   red fa_control_keys_leading_bom_mints_two rc=0
OK   red fa_control_keys_cr_only_text_unchanged rc=64
OK   red fa_control_md1_set_crlf_equals_lf rc=0
ALL ROWS MATCH red
```

GREEN feasibility (author check, not the fix): a throwaway patch of the rule in section 2 (about 20 lines in the two
readers) made `scripts/mk-fa-line-readers-rows.sh green` print `ALL ROWS MATCH green` and the full mk-cli suite
`279 tests run: 279 passed`; the patch was reverted and is not on this branch.

## 6. Tests that change

- New `crates/mk-cli/tests/bh1119_line_readers.rs`: one `#[test]` per section 5 row, named exactly as the row
  (rows with `[...]` none), driving the real `mk` binary with the same bytes, asserting the GREEN column (exact
  texts from 2.1). Seen RED before the fix (all `fa_i3_*`/`fa_i4_*` fail; `fa_control_*` pass), then GREEN.
- No existing test changes: no test at af12fea feeds a CR or a U+FEFF to either reader (`grep -rn '\\r\|feff'
  crates/mk-cli/tests` is empty; the `keyfile.rs` unit tests parse single records). `keys_batch.rs` and
  `from_md1_set.rs` stay green unchanged.

## 7. Toolkit manual lockstep (`docs/manual/src/40-cli-reference/44-mk-cli.md`, master 68ea05e)

No flag changes (no GUI schema change). Two table rows gain a clause (PROPOSED wording):

- L39 `--from-md1-set <FILE>`: after "... are skipped" insert "; a leading byte-order mark is removed and CRLF
  line ends are read as LF; any other carriage return, or a byte-order mark after the first character, is refused
  (exit 64) naming the line".
- L40 `--keys <FILE>`: after "blanks and `#` comments ignored" append "; a leading byte-order mark is removed and
  CRLF line ends are read as LF; a CR-only file, any other carriage return, or a byte-order mark after the first
  character, is refused (exit 64) naming the line". (L41 `--in` says "the same reader `--keys` uses" and needs no
  change.)

## 8. Ruling (closed)

B1 (Brian 2026-11-22). The triage's two opposite scratch policies (I3 refuse, I4 normalise) are resolved by it:
refuse, except the leading BOM and CRLF.

## 9. FOLLOWUPS (listed in the author's report, not appended here)

No open FOLLOWUPS line names bh-mk-I3 or bh-mk-I4 at af12fea, so this FIX closes none. The author's report
proposes three lines: `mk-fa-line-readers` (owed-to-build, closed by the GREEN commit),
`mk-mk1-reader-encoding` (the out-of-scope mk1 reader, section 1) and `mk-md1-set-invisible-prefix` (measured at
af12fea: a U+200B before the first md1 string of a `--from-md1-set` file drops that policy at exit 0, the I4 shape
with a character B1 does not name; needs its own rule, not this FIX).

## 10. Gate

- RED/GREEN rows: `scripts/mk-fa-line-readers-rows.sh red` at af12fea (must print `ALL ROWS MATCH red`), `green`
  on the build head (`ALL ROWS MATCH green`).
- `cargo nextest run --locked -p mk-cli --test bh1119_line_readers` RED then GREEN; then the repo's full CI
  (fmt with 1.95.0, clippy, the full nextest suite, `ci/code-docs-check.sh`) on the build head.
