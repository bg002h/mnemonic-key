# Independent review: mk-cli constellation exit codes (spec rev 5, §7b)

Reviewer: independent adversarial agent (did not write the code). Date 2026-11-26.
Diff: `git diff 7212cf6^..HEAD` on `fx/mnemonic-key` (7212cf6 closed-stdout fix;
506c95f code; f3a17c8 docs). Spec: mnemonic-engrave
`design/SPEC_constellation_exit_codes.md` rev 5 (§2-§5, §7b).

**What I ran.** I built `mk` (debug, opt-level 2) and probed it by hand against the
vector corpus (`mk vectors`) and one-character corruptions of it. I ran these test
binaries: `exit_codes_spec` 6/6, `decode_verify_correction_note` 4/4,
`output_errors` 9/9 and `cli_repair` 10/10, all passing. I did not run the whole
suite.

**Verdict: NOT GREEN.** 0 Critical, 2 Important, 6 Minor, 4 Nit.

What checks out (verified, not just read):
- **R11:** the legend is identical to the spec's text. I compared it programmatically
  with the spec's `>` block.
- **R8:** stdout failures add 32 and keep the verb's own bits. I measured `decode` on a
  corrected card with `>&-` and with `>/dev/full`: 48 both times. A broken pipe (EPIPE)
  keeps the verb's code.
- **R13:** `exit_code()` is an exhaustive match.
- **Codes for each case:**
  - `address --network` mismatch: 8, and 24 on a corrected card.
  - output-is-input refusal: 8.
  - missing `--in` file: 32.
  - non-UTF-8 input file: 2.
  - unsupported version: 2.
  - xprv as `--xpub`: 2.
  - verify mismatch: 1, and 17 with a correction.
- **R2:** `derive --path x/y` on an uncorrectable card gives 64.
- **Repair:**
  - a string that is not a chunk (N6, N9) prints the string and gives 2;
  - corrected, the same strings give 18;
  - a clean complete set that does not decode (N11-N21) gives 2 with no output;
  - a miscorrected set gives 2, so the funds fix holds.

## Important

### I1. A missing input is classed 2 (INVALID); the spec says a missing argument stays 64
- **Where:** `crates/mk-cli/src/cmd/mod.rs:469-478` (`read_mk1_strings`). The test
  `tests/exit_codes_spec.rs:117-120` pins the wrong value, and `CHANGELOG.md` ("empty
  or missing input: 64 -> 2") states it as intended.
- **Spec:** decision 3 (spec:127): "A required argument that is *missing* stays 64."
  Test 1 and R2 say the same: the command line alone is wrong and nothing was read.
- **Scenario:** `mk decode`, `mk repair`, `mk address` and `mk verify` with no
  positional, no `--in` and no `-` now exit 2 ("an input is invalid"). I measured all
  four. No input was read, so a script or GUI that sends a 2 to "fix or replace the
  input" sends the user after a card that was never given. `mk decode -` with an empty
  stdin, or an empty `--in` file, is correctly 2: something was read and it was empty.
- **Fix:** in `read_mk1_strings`, return `UsageError` (64) when no channel was named
  (`args.is_empty() && in_file.is_none()`). Keep `Invalid` for a named channel that
  produced nothing. Do this before any read, so R2 holds. Flip the
  `json_envelope_carries_the_class` second half to 64 and `UsageError`, add a row for
  `mk decode -` on empty stdin giving 2, and correct the CHANGELOG line.

### I2. Repair of a single-string mk1: the code departs from the spec's worked value, and the CHANGELOG describes behaviour the code does not have
- **Where:** `crates/mk-cli/src/cmd/repair.rs:238` (a single string counts as a complete
  set of one), `:386-395` (it fails as `Err`, nothing printed), and `CHANGELOG.md:38-42`.
- **Spec:** decision 6 (spec:167): a corrected string that fails its check, "with no
  oracle, as for a single string, … gives 16 + 4 + 2 = 22", and the corrected strings
  are still printed. Decision 7: "Only a string that is not chunked at all has no oracle
  (20)."
- **What the code does:** a single string that fails decode gives 2, nothing printed,
  corrected or clean. I measured N7 (clean) and N7 with one flip: both 2 with empty
  stdout. The corrected case prints the `SetReassemblyMismatch` text, "the set does not
  reassemble (single-string chunk 0)", which is an odd message for a string that is not
  chunked.
- **What the CHANGELOG says:** "a single-string card that does not decode, is printed
  with an error line and adds **2**". That is false. It also lists "a corrected
  single-string card **20**". By the repo's own test comment (`tests/cli_repair.rs:492`:
  a single-string mk1 cannot hold a 65-byte xpub), that code cannot occur in mk, and no
  test reaches it.
- **Judgement on spec question 1:** the implemented reading is the safer one for funds.
  It never prints a "repaired" string that is no card. It also has a real argument:
  under mk's grammar a single string is a whole card, so decoding it *is* reassembly.
  But it contradicts the spec's explicit worked value, and the spec needs a ruling: "in
  mk, 22 is impossible in practice, so a single string follows the complete-set rule".
- **Fix:**
  - Get that ruling, or print and return 22.
  - Correct the CHANGELOG either way.
  - Drop "20" from the CHANGELOG and from the toolkit manual lines in `impl-mk.md` (the
    repair table there lists 0 / 16 / 20 / 2), or mark 20 unreachable for mk.
  - For the single-string group, raise an `Invalid` / UNVERIFIED-style message rather
    than "each chunk corrected individually, but the set does not reassemble".

## Minor

### M1. The R13 exhaustiveness guard does not catch a missing row
- **Where:** `crates/mk-cli/src/error.rs:394-415`.
- **Problem:** the `match` forces the new variant's *name* into the test. But `seen` is
  built only from `cases()`, and the count is the literal `12`. Someone who adds a 13th
  variant to the match but no row to `cases()` still gets 12 names, so the test passes.
  The doc comment claims otherwise. The class itself is still forced by the exhaustive
  `exit_code()` match, so this is a test-honesty problem, not a hole in the code.
- **Fix:** compare against the full list of names, for example a `const ALL: [&str; N]`
  checked against the match arms, or derive N from a `strum`-style count. Or assert
  that every arm's name appears in `seen` by iterating the arm list itself.

### M2. A card that `verify` calls invalid exits 0 from `inspect`, `decode`, `address` and `derive`
- **Where:** `crates/mk-cli/src/cmd/mod.rs:117-123` (`warn_if_invalid`).
- **Problem:** a legacy card that fails `KeyCard::validate()` makes `verify` exit 2. The
  same card in `inspect` prints its report plus "warning: this card is invalid" and
  exits 0. R5 says a report verb reports in its code, and a message must agree with its
  code.
- **Mitigating:** the mk spec (`SPEC_mk_v0_1.md:342`) makes these cards readable on
  purpose. The exit-code spec's decision 8 speaks only of strings that "would not
  decode".
- **Fix:** ask for a spec ruling: 2, or 4 ("verify before trusting"), for `inspect` at
  least. At minimum, record this as a deliberate exception.

### M3. `repair` still exits 0 on N8, a pair that `decode` rejects
- **Spec:** §7b says decision 6 "ends today's 0 on strings `mk decode` rejects"
  (spec:604).
- **Measured:** `mk repair --in N8.txt` (two chunks with different `chunk_set_id`s)
  exits 0 and prints both chunks. `mk decode` on the same input exits 2.
- **Why:** each chunk forms its own incomplete group, which decision 7 says is no
  failure.
- **Fix:** this is defensible under decision 7. Either have the spec say §7b's claim
  stops at incomplete groups, or treat two groups that declare the same
  `total_chunks` and together cover every index as a suspected `chunk_set_id`
  miscorrection.

### M4. `verify --json` on a corrected card says only `"ok": true` while the exit is 16
- **Where:** `crates/mk-cli/src/cmd/verify.rs:211-216`.
- **Problem:** the text verdict was changed to agree with the code (R5), but the JSON
  verdict carries no correction field. A JSON consumer that ignores the status learns
  nothing. R14 does not strictly apply, because this success envelope has no
  `exit_code`.
- **Fix:** add an additive field, `"corrected": true` or the per-chunk counts.

### M5. The repo's normative spec still states the old code
- **Where:** `design/SPEC_mk_v0_1.md:342`: "`mk verify` exits 1 with it".
- **Problem:** it is now 2. The implementer updated `docs/MK_CODEC_RUST_API.md:93` but
  not the format spec. This is not a dated record.
- **Fix:** change it to "exits 2 (an invalid input)".

### M6. A blanket R1 reading is applied to atomic refusals after a correction (spec question 7, extended)
- **Where:** `crates/mk-cli/src/main.rs:113-127`.
- **What it does:** it adds 16 to *every* `Err` once a corrected value was used. That
  gives 18 for a fixed-key or multisig `address`/`derive` refusal with no output, 18
  for `verify`'s `validate()` failure, and 24 for a network refusal.
- **Spec tension:** §3 and §6 explicitly allow 24 for a network refusal. R1 says "a
  verb that fails atomically reports only its failure bits".
- **Judgement:** the chosen reading follows R6 and decision 5 ("a script now learns
  that the plate is damaged") and is the more useful one.
- **Fix:** record it in the spec questions as a general rule, not only for 17, so the
  md, ms and mt implementations make the same choice. Otherwise the constellation will
  disagree on, for example, `md`'s 2 against `mk`'s 18 for the same shape of failure.

## Nit

- **N1.** `crates/mk-cli/src/main.rs:16`: `#[allow(dead_code)] mod exit;` silences the
  whole module. `STOPPED`, `Bits::contains` and others are unused today, but so would be
  any future accidental orphan. Put the allows on the individual items instead.
- **N2.** `crates/mk-cli/src/exit.rs` `the_legend_is_the_specs` checks only the start and
  end of the legend. The integration test `root_help_ends_with_the_exit_status_legend`
  covers the whole text, so this is fine, but the unit test's name claims more than it
  checks.
- **N3.** `/home/user/fx/impl-mk.md` class table: "verify content mismatch (was 1)". It
  was 4 (base `error.rs`: `ContentMismatch => 4`), and the CHANGELOG has it right. Only
  the report is wrong.
- **N4.** `crates/mk-cli/src/cmd/encode.rs:241-245`: the second "at least one of
  --policy-id-stub or --from-md1" `UsageError` is now unreachable (`read_md1_set`
  refuses an empty file, and `group_md1_cards` never returns zero groups for a
  non-empty list). If it ever became reachable, it would be a 64 raised after input was
  read (an R2 breach). Make it `unreachable!` or `Internal`.

## Spec questions: is the chosen reading closest to §3?

| # | question | judgement |
|---|---|---|
| 1 | a single string is a complete set of one: 2, nothing printed, not 22 | departs from the spec's explicit text, though safer; needs a ruling (I2) |
| 2 | a clean complete set that fails decode: 2, nothing printed | matches decision 6's last bullet |
| 3 | `address` cannot infer the type: 2 | correct: test 6 ("insufficient"), and it cannot be judged from argv (R2) |
| 4 | `derive --path` / `--index` are settings: 64, checked first | defensible. They pick what to show, like `--range`, which the spec names as a setting. Decision 2's "`--path`" example is `md compose`'s wallet-shape path, a different thing. Worth one line in the spec |
| 5 | `--chunk-set-id`, stub, fingerprint and path are inputs: 2 | correct: each is data written into or compared with the card |
| 6 | an origin fingerprint against the xpub, and a depth-0 parent fingerprint: 2 | correct. Both are self-inconsistent input that breaks the verb's correctness rules (like an xprv given as an xpub, or a BIP-388 distinctness violation, both 2), not a comparison against a reference. `verify --origin-fingerprint X` against the card stays 1, verified |
| 7 | a mismatch on a corrected card: 17 | consistent with R6 and the spec's allowance of 24; generalise it (M6) |
| 8 | JSON `kind` renames | fine. The CHANGELOG lists them, and GUI consumers must follow |

## Closed-stdout probe (7212cf6)

Correct as far as I can tell.
- **When it runs:** the `.init_array` / `__mod_init_func` constructor runs before
  `lang_start`'s `sanitize_standard_fds`.
- **What it does:** `F_GETFD` with `EBADF` detects a closed fd. It fills fds 0-2 in
  order, so `open` returns the lowest free fd and no `--out` file can take a standard
  fd. `read_only()` turns the recorded flag into EBADF at the first stdout write.
- **Measured:**
  - `decode >&-` gives 32;
  - with a correction, 48;
  - `decode <&- 2>&-` with stdout open gives 0, with the output intact;
  - the `--out` and `vectors --out` cases stay 0 (test).
- **Caveat (not a defect):** a verb that writes nothing to stdout keeps 0 with stdout
  closed. That is reasonable under R8, because nothing was lost.
- **Edge:** if `/dev/null` cannot be opened, the fd stays closed and std's own
  sanitizer handles it as before.
