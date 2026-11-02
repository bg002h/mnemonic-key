architect mk-fix-stops: 6 answered; 0 prior rulings changed

Scope: FIX_bughunt_mk.md §10 S1-S6 on fix/bughunt @7eca58f. Grade rulings
(grade-bughunt-mk.md §3 Q1/Q2/Q3/Q5) bind; none has a C-class defect. One
reading is stated under S2: the grade's marker clause, taken by its own
"by its existing convention" qualifier, yields no marker. If the coordinator
reads it literally instead, count it as 1 ruling changed: the literal reading
fails the manual build gate (measured below).

Evidence (scratch only, /scratch/cache/bughunt/mk/architect/):
- tk/ = toolkit origin/master f6355e22 (detached). The baseline lint is green:
  lint-base.log ends "[lint] OK".
- variant.py rewrites manual l.31/39/47/499/500 in a copy of src/. Its
  variants are: A = grade text verbatim (no backticks); Ax = A with
  `--origin-fingerprint` renamed `--origin-fingerprintx`; B = backticks plus
  markers; C = backticks, no markers; Cx = C with the same rename. Logs are
  lint-{A,Ax,B,C,Cx}.log. The pin is mk-cli 0.13.0
  (/scratch/cache/ci-tools/bin/mk).
- In every variant, cspell ("Files checked: 0") and lychee (relative pdf
  links) fail only because the copy lives outside docs/manual. The baseline,
  run in place, passes both. Ignore those two steps; the rows below are the
  markdownlint and flag-coverage steps.
- mk/ = mk @7eca58f with the §3.1 shape applied, plus a probe in
  crates/mk-codec/tests/bughunt.rs (S5/S6 outputs quoted below).

## S1: restore the chapter's backticks (a correctness fix, not a style one)

Answer: restore them. Every flag, `<VALUE>`, `A..=B` and `--keys`/`--in`
goes in backticks, as on the old rows. Only the quoting changes; the words
are the grade's.

Why: the grade wrapped each row in one code span (grade l.39, 40, 63, 64),
so it could not carry inner backticks. The missing backticks are a quoting
artifact, not a ruling. Transcribing them verbatim breaks two things:
(1) markdownlint fails. Variant A gives "Summary: 2 error(s)":
`44-mk-cli.md:39:24 MD033/no-inline-html Inline HTML [Element: HEX]` and
`:499:11 ... [Element: N]`. That is, `<HEX>` and `<N>` would render as
invisible HTML tags.
(2) The reverse flag lint scans only rows whose first cell starts with a
backtick (lint.sh `^\|[[:space:]]*\``). So the four rows would drop out of
flag coverage without any message. Variant Ax renames a flag and the log
has no line for `--origin-fingerprintx`. With the backticks restored (Cx),
the lint catches it: "flag --origin-fingerprintx is documented for `mk
encode` ... but the pinned binary does not define it". Variant C reports
markdownlint "Summary: 0 error(s)" and no flag-lint failure.

Change: replace FIX §8's manual table texts (FIX l.191-206) with these
exact lines.
- l.31 (inside the `sh` block):
  `mk encode --xpub <XPUB> --origin-path <PATH> (--origin-fingerprint <HEX> | --privacy-preserving) [OPTIONS]`
- l.39:
  ``| `--origin-fingerprint <HEX>` | 8-hex-char master fingerprint; mutually exclusive with `--privacy-preserving`; a single-card encode needs one of the two (exit 64 otherwise); refused with `--keys`/`--in`, whose records carry their own |``
- l.47:
  ``| `--privacy-preserving` | emit without master fingerprint; mutually exclusive with `--origin-fingerprint`; a single-card encode needs one of the two (exit 64 otherwise) |``
- l.499:
  ``| `--count <N>` | number of addresses per chain, starting at index 0 (default 10, max 1000000); conflicts with `--range` |``
- l.500:
  ``| `--range <A,B>` | inclusive index range `A..=B`, at most 1000000 indices; conflicts with `--count` |``

(The outer double backticks above are this report's quoting. Each manual
line starts at its `|`, or at `mk` for l.31.)
Fold: yes (transcription, FIX §8).

## S2: no marker on any row; no l.16 amendment

Answer: (a) no marker text; (b) no row names a flag; (c) l.31 gets none.

Why: the grade attaches the marker "by its existing convention" (grade
l.41). That convention, manual l.16-18 and lint.sh l.102-111, covers only a
flag "on mnemonic-key's `main` but not in the released mk-cli 0.13.0 ...;
the released binary rejects it". All four flags exist in 0.13.0, so the
convention does not apply. Applying the marker anyway fails the gate.
Variant B produced four errors, e.g.
"[lint] FAIL: stale marker in 44-mk-cli.md (mk encode): --origin-fingerprint
is defined by the pinned mk-cli 0.13.0; remove it from the marker". This is
the toolkit's mutation R3 in ci/doc-flag-lint.test.sh. l.31 sits in a code
block, which the lint does not read. Until the release, the gap is
harmless:
- The new rows are stricter than 0.13.0. A holder who passes a fingerprint
  or uses a count of at most 1000000 succeeds on both binaries. No printed
  command fails as printed.
- A new marker kind plus an l.16 amendment would be more printed text and a
  lint change, for no holder gain. On the tie rule, the smaller text wins.

Change: FIX §8 drops the marker sentence. FIX §10 S2 closes with "no marker
(released flags; lint R3)".
Fold: yes (transcription).

## S3: clap help for encode, use the full candidate

Answer:
- encode.rs l.28 becomes
  `/// 8-hex-char master fingerprint. Mutually exclusive with `--privacy-preserving`; a single-card encode needs one of the two (exit 64 otherwise); refused with `--keys`/`--in`, whose records carry their own.`
- encode.rs l.85 becomes
  `/// Encode without master fingerprint. Mutually exclusive with `--origin-fingerprint`; a single-card encode needs one of the two (exit 64 otherwise).`

Why: Q1 says the help strings "change to match" the manual (grade l.42).
The manual rows above are the match, so the help carries the same clauses,
ending with a period as both help lines do today. Keeping the
`--keys`/`--in` clause also saves a holder one failed run, because the
refusal is a runtime check (encode.rs l.145-168), not a clap conflict
shown in usage. That outranks the smaller-text tie-break, so this was not a
tie. No `--help` snapshot exists, so no test text changes.
Fold: yes (FIX §3 or §8, a new "help" row with these two lines).

## S4: clap help for address names the cap

Answer:
- address.rs l.37 becomes
  `/// Number of addresses per chain, starting at index 0 (max 1000000). Conflicts with `--range`.`
- address.rs l.41 becomes
  `/// Inclusive index range `A,B`, at most 1000000 indices (alternative to `--count`). Conflicts with `--count`.`

Why: on fewest holder steps, a holder reading `--help` should see the limit
before an exit-64 refusal. The manual rows (S1, l.499/500) already state
it, and Q3's "help matches" logic applies the same way. The numbers match
the §3.3 refusal texts (`max 1000000`).
Fold: yes (transcription).

## S5: accept the ASCII-lowercased InvalidHrp payload

Answer: accept it. Do not add code. Optionally, extend FIX §4 row 05 with
an uppercase case that expects the same `InvalidHrp("m\u{212A}")`.

Why: measured on the §3.1 shape, uppercase `M\u{212A}1...` gives
`InvalidHrp("mK")` (the K is U+212A, preserved), and an uppercase wrong
ASCII HRP gives `InvalidHrp("mq")` / "invalid HRP: mq". Every wrong HRP is
reported that way today. Q5's "<hrp as typed>" (grade l.71) exists to
forbid the Unicode fold that turned K into k. The shape keeps the Kelvin
sign, so the ruling is met under that reading. Reporting the original case
would need extra slicing for a payload that only shows on screen, and both
forms fail closed with exit 1 (mk-cli error.rs l.125-129). No ruling
changed.
Fold: optional (one extra row 05 case).

## S6: accept "first non-ASCII char wins"

Answer: accept it.

Why: measured on the §3.1 shape, a data part with `b` at 2 and `é` at 9
gives `InvalidChar { ch: 'é', position: 9 }`, and `b` at 2 alone gives
`InvalidChar { ch: 'b', position: 2 }`. Both refuse; neither decodes.
`position` is used only for display (toolkit friendly.rs:173). No repair or
erasure path in mk reads it. ASCII-only inputs keep their current error.
The alternative needs a second scan for no holder gain.

Change: in FIX §2 row 5, add "a non-ASCII char is reported first, before
any alphabet error".
Fold: yes (transcription, one clause).

## Side check

FIX §2's exit 1 for `mk decode` on InvalidHrp is correct: mk-cli error.rs
maps `Codec(UnsupportedVersion)` to 3, any other Codec error to 1 and
UsageError to 64. The toolkit's exit 2 for mk InvalidHrp (toolkit error.rs
484-491) is a separate binary and outside this FIX.

Brian's alone: none.

Counts: 6 answered; 0 prior rulings changed (1 if S2's marker clause is
read literally); 0 Critical, 0 Important.
