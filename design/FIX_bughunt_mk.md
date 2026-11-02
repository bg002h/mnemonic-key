# FIX-mk-bughunt-2026-11: mk bug-hunt fixes #1, #3, #4, #5 (+ #2 spec/test)

Small-fix track. Repo: mnemonic-key (branch `fix/bughunt`), plus the toolkit
manual `docs/manual/src/40-cli-reference/44-mk-cli.md` in lockstep (separate
toolkit change; the mk build does not edit the toolkit).

Inputs: triage `design/agent-reports/bughunt-triage-mk.md`; grade and rulings
`design/agent-reports/grade-bughunt-mk.md` (§3 Q1, Q2, Q3, Q5 and §4 bind).
Scratch patches `/scratch/cache/bughunt/mk/fix-0N.patch` are hints only;
fix-01 as drafted is a regression and must not be applied.

**Open STOPs (§10) must be answered before the build starts.** Every other
point below is decided.

## 1. Spec delta (exact texts)

1. `design/SPEC_mk_v0_1.md` §2.1 (after l.38, the line that ends "giving the
   prefix `mk1`."), add a new paragraph:

   > An mk1 string contains only ASCII characters: the HRP, the separator and the BIP 173 bech32 alphabet, all in one case (BIP 93). Decoders MUST refuse any other character, including a non-ASCII letter whose Unicode lowercase is ASCII (e.g. U+212A KELVIN SIGN), and MUST NOT apply Unicode case folding.

2. `design/SPEC_mk_v0_1.md` §3.5 (after l.272, the line beginning "Each
   component encodes a u32 BIP 32 child number"), add a new paragraph:

   > Encoders MUST emit each component in minimal LEB128 (no trailing `0x80`-continued zero groups). Decoders accept a non-minimal component of at most 5 bytes and decode it to the same u32; only overflow is refused (§4 rule 6).

3. `design/SPEC_mk_v0_1.md` §6 l.401: replace the sentence
   "Encoders SHOULD expose this as an explicit choice rather than a default."
   with:

   > Encoders SHOULD expose this as an explicit choice rather than a default. The reference CLI makes it required: a single-card `mk encode` that passes neither `--origin-fingerprint` nor `--privacy-preserving` is refused with exit 64.

4. `design/SPEC_mk_derive_address.md` l.51 (the "**Range:**" bullet) states
   the count semantics, so append this sentence to the end of that bullet
   (after "Mutually exclusive (`conflicts_with`)."):

   > Either form is capped at 1,000,000 indices per chain, checked after the BIP-32 ceiling check: `--count N` with N > 1000000 → `CliError::UsageError` `--count N too large (max 1000000)`, `--range A,B` with B−A ≥ 1000000 → `CliError::UsageError` `--range too large (max 1000000)`, exit 64.

## 2. Behaviour, refusals, exit codes, error variants

No new flag, type, command or error variant.

| # | input | result | exit | stderr / Display |
|---|---|---|---|---|
| 1 | `mk encode` single-card (no `--keys`, no `--in`) with neither `--origin-fingerprint` nor `--privacy-preserving` | `CliError::UsageError`, no stdout | 64 | `error: pass --origin-fingerprint <hex> or --privacy-preserving explicitly` |
| 1 | `mk encode --keys F` / `--in F` with neither flag | unchanged: cards minted from records | 0 | — |
| 3 | `mk address ... --count N`, 1000000 < N ≤ 2147483648 | `CliError::UsageError` | 64 | `error: --count N too large (max 1000000)` |
| 3 | `mk address ... --range A,B`, B ≤ 2147483647, B−A ≥ 1000000 | `CliError::UsageError` | 64 | `error: --range too large (max 1000000)` |
| 3 | `--count N`, N > 2147483648 | unchanged ceiling text, fires first | 64 | `error: --count N exceeds the BIP-32 normal-index ceiling (max 2147483648)` |
| 3 | `--count 1000000`, `--range 5,1000004` | accepted, 1,000,000 lines | 0 | — |
| 4 | `--from-md1-set` file with a line whose byte 3 is inside a multi-byte char | line skipped (documented skip) | 0 | — |
| 5 | non-ASCII char in the data part (after the last `1`) | `Error::InvalidChar { ch, position }`, position = 0-based char index in the data part, first non-ASCII char; a non-ASCII char is reported first, before any alphabet error | `mk decode` 1 | `invalid character <ch> at position <i>` |
| 5 | non-ASCII char at or before the last `1` | `Error::InvalidHrp(hrp)` | `mk decode` 1 | `invalid HRP: <hrp>` (hrp ASCII-lowercased; S5) |

## 3. Production changes (exact)

### 3.1 Codec #5 — `crates/mk-codec/src/string_layer/bch.rs`, `decode_string` (l.667)

- l.673: `let s_lower = s.to_lowercase();` becomes
  `let s_lower = s.to_ascii_lowercase();`
- Immediately before `let code =` (l.685), after the `hrp != HRP` check,
  insert:

  ```rust
  if let Some((i, c)) = data_part.chars().enumerate().find(|(_, c)| !c.is_ascii()) {
      return Err(Error::InvalidChar { ch: c, position: i });
  }
  ```

- Nothing else changes (the in-loop `is_ascii` check stays; the HRP path
  needs no code: with ASCII-only lowercasing a non-ASCII HRP char can never
  equal `mk`, so `InvalidHrp` fires). Strings with no non-ASCII character
  take exactly today's path (length lookup before the alphabet check).

### 3.2 CLI #1 — `crates/mk-cli/src/cmd/encode.rs` (~l.214)

In the `match (&args.origin_fingerprint, args.privacy_preserving)`, insert
after the `(None, true) => None,` arm and before the existing
`(None, false) => None,` arm (which stays):

```rust
(None, false) if key_source.is_none() => {
    return Err(CliError::UsageError(
        "pass --origin-fingerprint <hex> or --privacy-preserving explicitly".into(),
    ));
}
```

Clap help (S3), exact:
- encode.rs l.28 becomes
  `/// 8-hex-char master fingerprint. Mutually exclusive with `--privacy-preserving`; a single-card encode needs one of the two (exit 64 otherwise); refused with `--keys`/`--in`, whose records carry their own.`
- encode.rs l.85 becomes
  `/// Encode without master fingerprint. Mutually exclusive with `--origin-fingerprint`; a single-card encode needs one of the two (exit 64 otherwise).`

### 3.3 CLI #3 — `crates/mk-cli/src/cmd/address.rs`, `resolve_indices`

- After the `--count` BIP-32 ceiling check, before `Ok((0..c).collect())`:
  `if c > 1_000_000 { return Err(CliError::UsageError(format!("--count {c} too large (max 1000000)"))); }`
- After the `--range` end ceiling check, before `Ok((a..=b).collect())`:
  `if (b - a) >= 1_000_000 { return Err(CliError::UsageError("--range too large (max 1000000)".into())); }`
- Doc comment l.196-201 becomes exactly:

  ```rust
  /// `--count N` → `0..N`; `--range A,B` → `A..=B`; neither → `0..10`.
  ///
  /// The highest generated index is validated against the BIP-32 normal-index
  /// ceiling (`< 2^31`) before any allocation (C1), then the index count is
  /// capped at 1,000,000 per chain: an out-of-range or oversized `--count` /
  /// `--range` is a `UsageError` (exit 64), never a `from_normal_idx` panic or
  /// a multi-gigabyte allocation.
  ```

Clap help l.37 becomes (S4, architect 83c0c20):

```rust
/// Number of addresses per chain, starting at index 0 (max 1000000). Conflicts with `--range`.
```

Clap help l.41 becomes:

```rust
/// Inclusive index range `A,B`, at most 1000000 indices (alternative to `--count`). Conflicts with `--count`.
```

### 3.4 CLI #4 — `crates/mk-cli/src/cmd/encode.rs` l.480

`if s.len() >= 3 && s[..3].eq_ignore_ascii_case("md1")` becomes
`if s.get(..3).is_some_and(|p| p.eq_ignore_ascii_case("md1"))`.

### 3.5 #2 — no production change (Q2). Spec sentence 1.2 and the test rename only.

## 4. RED rows

Run each by exact name:
`cargo nextest run --locked -E 'test(/(^|::)<row>$/)'`.
Measured on unmodified code (tests only, this commit):
`cargo nextest run --locked -E 'test(/(^|::)bughunt_mk_/)'` → **8 tests run:
3 passed, 5 failed, 391 skipped**.

| row (crate::file) | command / input | expected after GREEN | on unmodified code (measured) |
|---|---|---|---|
| `bughunt_mk_01_encode_without_fingerprint_flag_not_silent` (mk-cli::bughunt) | `mk encode --xpub <V1_XPUB> --origin-path "m/48'/0'/0'/2'" --policy-id-stub deadbeef` | exit 64; stderr exactly `error: pass --origin-fingerprint <hex> or --privacy-preserving explicitly\n`; stdout empty | FAIL: exit `Some(0)` vs `Some(64)` (card minted silently) |
| `bughunt_mk_01_keys_and_in_encode_need_no_fingerprint_flag` (mk-cli::bughunt) — new | file with `[73c5da0a/48'/0'/0'/2']<V1_XPUB>\n`; `mk encode --keys F --policy-id-stub deadbeef` and `mk encode --in F --policy-id-stub deadbeef` | each exit 0, ≥1 stdout line starting `mk1` | PASS (as intended: regression guard). With fix-01 as drafted planted it FAILS: `--keys: code=Some(64) ... stderr="error: pass --origin-fingerprint <hex> or --privacy-preserving explicitly\n"` |
| `bughunt_mk_03_address_huge_count_is_usage_error` (mk-cli::bughunt) | card from `KeyCard::new([deadbeef], fp 73c5da0a, m/84'/0'/0', V2_84_MAIN)`; each case run as `timeout 60 sh -c 'ulimit -v 2097152; exec "$0" "$@"' mk address <strings> <flags>` | exit 64 with stderr exactly: `--count 2147483648` → `error: --count 2147483648 too large (max 1000000)\n`; `--range 0,2147483647` → `error: --range too large (max 1000000)\n`; `--count 1000001` → `error: --count 1000001 too large (max 1000000)\n`; `--range 0,1000000` → `error: --range too large (max 1000000)\n`; `--count 2147483649` → `error: --count 2147483649 exceeds the BIP-32 normal-index ceiling (max 2147483648)\n`. Then `--count 1000000` and `--range 5,1000004` (parallel threads): exit 0, exactly 1,000,000 newlines each (~30 s) | FAIL: `--count 2147483648`: code None, stderr `memory allocation of 8589934592 bytes failed` |
| `bughunt_mk_04_from_md1_set_non_ascii_line_skipped` (mk-cli::bughunt), unchanged | `--from-md1-set` file `éé note\n` + KEYED_POLICY_A lines; `mk encode --xpub <V1_XPUB> --origin-fingerprint 73c5da0a --origin-path "m/48'/0'/0'/2'" --from-md1-set F` | exit 0 | FAIL: code 101, panic `encode.rs:480:29` "byte index 3 is not a char boundary; it is inside 'é' (bytes 2..4) of `éénote`" |
| `bughunt_mk_02_leb128_non_minimal_decodes_to_same_value` (mk-codec::bughunt) — renamed | `decode_path` on `[FE,01,80,00]` | Ok, cursor empty, path == `m/0`, `encode_path(m/0) == [FE,01,00]` | PASS (as the grade expects: no code change) |
| `bughunt_mk_05_decode_string_non_ascii_refused` (mk-codec::bughunt), unchanged | probes U+212A, U+0130, U+FF11, U+FF4D, U+00E9 at HRP, separator and data positions of the first and last strings of `encode_with_chunk_set_id(multi_chunk_card(), 0)` | every probe `Err` | FAIL: Kelvin-sign strings ACCEPTED |
| `bughunt_mk_05_non_ascii_error_variant_and_position` (mk-codec::bughunt) — new | same two strings; U+212A and U+00E9 at data index i ∈ {0, 7, data_len−1} (char 3+i); and U+212A at char 1 | data: `Err(InvalidChar { ch: p, position: i })`, Display `invalid character <p> at position <i>`; HRP: `Err(InvalidHrp("m\u{212A}"))`, Display `invalid HRP: m\u{212A}` | FAIL: `\u{212a} at data index 0: got None` (accepted) |
| `bughunt_mk_06_five_to_eight_errors_never_accepted` (mk-codec::bughunt), unchanged | — | PASS | PASS (triage FALSE POSITIVE, kept as guard) |

## 5. Tests that change

- Renamed: `bughunt_mk_02_leb128_non_minimal_refused` →
  `bughunt_mk_02_leb128_non_minimal_decodes_to_same_value`, now asserting
  same-value decode (done in this commit).
- Added: `bughunt_mk_01_keys_and_in_encode_need_no_fingerprint_flag`,
  `bughunt_mk_05_non_ascii_error_variant_and_position` (this commit).
- Changed: `bughunt_mk_01_encode_without_fingerprint_flag_not_silent` now
  asserts the exact stderr and empty stdout; `bughunt_mk_03_...` asserts the
  exact texts above, the ceiling-first ordering and the two 1,000,000-line
  accepts (this commit).
- No clap help snapshot exists (no `*.snap` in the repo; `mk gen-man` tests
  assert the page set, not prose; `mk gui-schema` emits no help text). No
  other existing test changes; the scratch GREEN ran the full suite green.

Tests that must stay green (fix-01 as drafted broke all 11):

- mk-cli::channels: `in_on_encode_equals_keys`, `in_errors_name_in_not_keys`.
- mk-cli::keys_batch: `comments_and_blanks_are_ignored`,
  `batch_matches_per_key_loop`, `empty_keyfile_is_refused`,
  `json_batch_wraps_the_single_card_object`,
  `bad_record_names_the_line_number`,
  `stdout_carries_mk1_lines_only_and_the_boundary_moves_to_the_card`,
  `origin_fingerprint_is_checked_where_the_xpub_proves_it`,
  `short_coverage_of_a_keyed_policy_is_noted_not_refused`,
  `record_order_follows_file_order`.

## 6. Build order

1. Spec delta §1 (items 1-4).
2. Codec #5 (§3.1). Run the two `bughunt_mk_05_*` rows: GREEN.
3. CLI #1 (§3.2) with its clap help (S3). Run both `bughunt_mk_01_*` rows
   and the 11 must-stay-green tests: GREEN.
4. CLI #3 (§3.3) with its clap help (S4). Row 03 GREEN.
5. CLI #4 (§3.4). Row 04 GREEN.
6. Gate (§7).
7. Toolkit manual (§8), as a separate toolkit change.

## 7. Gate

Run once, each to a file, then grep:

- `cargo nextest run --locked` — expected **399 tests run: 399 passed**
  (measured on a scratch GREEN of this exact fix set; the baseline 391 plus
  the triage's 6 rows plus the 2 new rows).
- `cargo test --locked --doc`
- `cargo fmt --check`
- `cargo clippy --locked --all-targets -- -D warnings`
- `cargo nextest run --locked -E 'test(/(^|::)bughunt_mk_/)'` — expected
  **8 tests run: 8 passed** (the 8 names in §4). A short or empty count is a
  FAIL.
- Then `coordinator/scripts/mutation-gate.sh <worktree> <base>` before any
  review.

## 8. Toolkit manual lockstep (44-mk-cli.md, origin/master f6355e22)

The grade cited l.501/502; on origin/master the `--count`/`--range` rows are
**l.499/500** (verified by `git show`). New texts below are the grade's,
verbatim except backticks restored (S1). No row carries an unreleased marker:
no marker (released flags; lint R3) (S2).

| line | old | new (grade) |
|---|---|---|
| 31 (in a `sh` block) | `mk encode --xpub <XPUB> --origin-path <PATH> [OPTIONS]` | `mk encode --xpub <XPUB> --origin-path <PATH> (--origin-fingerprint <HEX> \| --privacy-preserving) [OPTIONS]` |
| 39 | ``| `--origin-fingerprint <HEX>` | 8-hex-char master fingerprint; mutually exclusive with `--privacy-preserving` |`` | ``\| `--origin-fingerprint <HEX>` \| 8-hex-char master fingerprint; mutually exclusive with `--privacy-preserving`; a single-card encode needs one of the two (exit 64 otherwise); refused with `--keys`/`--in`, whose records carry their own \|`` |
| 47 | ``| `--privacy-preserving` | emit without master fingerprint; mutually exclusive with `--origin-fingerprint` |`` | ``\| `--privacy-preserving` \| emit without master fingerprint; mutually exclusive with `--origin-fingerprint`; a single-card encode needs one of the two (exit 64 otherwise) \|`` |
| 499 | ``| `--count <N>` | number of addresses per chain, starting at index 0 (default 10); conflicts with `--range` |`` | ``\| `--count <N>` \| number of addresses per chain, starting at index 0 (default 10, max 1000000); conflicts with `--range` \|`` |
| 500 | ``| `--range <A,B>` | inclusive index range `A..=B`; conflicts with `--count` |`` | ``\| `--range <A,B>` \| inclusive index range `A..=B`, at most 1000000 indices; conflicts with `--count` \|`` |

(`\|` above is table escaping in this document only; the manual line has a
plain `|`.)

## 9. Checks recorded

- **mnemonic-gui mirror:** `src/schema/mk.rs` holds hand-paraphrased help
  strings pinned to mk v0.9.0, not clap mirrors; its gate
  `mk_schema_flag_names_match_help_text` checks flag names only. Nothing
  changes in lockstep (the grade §5 GUI follow-up is the coordinator's).
- **`SPEC_mk_derive_address.md`:** l.51 states the count semantics; sentence
  §1 item 4 is added.
- **Evidence:** RED run above; fix-01 plant run (keys/in row fails as
  shown); scratch GREEN of §3 (all reverted): full suite 399/399, row 03
  30.1 s; the §3.1 shape alone: mk-codec 207/207.

## 10. STOP (undecided; answer before building)

- **S1 (manual backticks).** The chapter writes every flag and value in
  backticks (old lines 39, 47, 499, 500); the grade's new texts drop them.
  Transcribe the grade verbatim, or restore the chapter's backticks around
  each flag, `A..=B`, `--keys`/`--in`?
  RESOLVED (architect 83c0c20): backticks restored; §8 rows 39, 47, 499, 500 carry them (row 31 unchanged, already backticked).
- **S2 (unreleased marker).** Grade: "Each changed row carries the chapter's
  `(unreleased: mk-cli after <current release>: ...)` marker by its existing
  convention." l.16 defines the marker for flags absent from 0.13.0 only
  ("the released binary rejects it"); these flags exist in 0.13.0 and only
  their behaviour changes. Decide: (a) the marker text for a behaviour change
  on a released flag (e.g. `(unreleased: mk-cli after 0.13.0: --origin-fingerprint)`
  as-is, or a new wording plus an l.16 amendment); (b) which flag each row
  names (l.39 its own flag, l.47 `--privacy-preserving`, l.499 `--count`,
  l.500 `--range`?); (c) whether l.31, inside a `sh` code block, gets any
  marker.
  RESOLVED (architect 83c0c20): no marker (released flags; lint R3); §8 intro updated.
- **S3 (clap help, encode.rs l.28, l.85).** Grade: "change to match", no
  exact text. Today: "8-hex-char master fingerprint. Mutually exclusive with
  `--privacy-preserving`." and "Encode without master fingerprint. Mutually
  exclusive with `--origin-fingerprint`." Candidate (not chosen): append
  "; a single-card encode needs one of the two (exit 64 otherwise)" to both,
  and to `--origin-fingerprint` also "; refused with `--keys`/`--in`, whose
  records carry their own".
  RESOLVED (architect 83c0c20): full candidate; exact texts at §3.2.
- **S4 (clap help, address.rs l.37, l.41).** Does the help name the cap?
  Candidate: "Number of addresses per chain, starting at index 0 (max
  1000000). Conflicts with `--range`." and "Inclusive index range `A,B`, at
  most 1000000 indices (alternative to `--count`). Conflicts with `--count`."
  Or leave unchanged.
  RESOLVED (architect 83c0c20): the help names the cap; texts at §3.3 (S4 pointer replaced).
- **S5 (InvalidHrp payload case).** Q5 says `InvalidHrp(<hrp as typed>)`, but
  the binding shape (`to_ascii_lowercase` before the split) reports the
  ASCII-lowercased HRP, as `InvalidHrp` does today for every wrong HRP: an
  uppercase input `M\u{212A}1...` yields `invalid HRP: m\u{212A}`, not
  `M\u{212A}`. Accept the lowercased payload (no extra code; the row tests
  lowercase input only), or report the HRP as typed (extra slicing of `s`)?
  RESOLVED (architect 83c0c20): accept the ASCII-lowercased payload; no code; the optional §4 row-05 uppercase case is not folded.
- **S6 (two bad chars in one data part).** With the §3.1 shape, a data part
  that holds both an invalid ASCII char (e.g. `b`) and a later non-ASCII
  char reports the non-ASCII one (first non-ASCII), not the earlier invalid
  ASCII one. Accept (fails closed, keeps every ASCII-only path unchanged), or
  report the first invalid char of either kind when any non-ASCII is present?
  RESOLVED (architect 83c0c20): accept first non-ASCII; clause added to §2 row 5 (the data-part row, l.52).
