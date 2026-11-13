# Author report: FIX_mk_output_sink

Branch `fix/mk-output-sink` from 2fc367c560013667180ee19572bbfad544d8cd88. Deliverables: `design/FIX_mk_output_sink.md` (ten sections), `scripts/mk-output-sink-fix-gate.sh`. No production code on the branch. Head before this report: 599f0b9.

## Gate (last lines, run on the final commits)

```
cmp: blocks identical to the grade's files
RED 1..5 ok (rc=100)  (each failed with its stated substring)
RED whole file: 6 tests run: 1 passed, 5 failed
self-test ok: the checker rejects a broken row
GREEN rows: 6 tests run: 6 passed
GREEN suite: 405 tests run: 405 passed
GREEN doc, fmt, clippy: ok
GATE PASS
```

A copy of the document with one `## expect:` text broken made the gate die at RED row 1 ("did not fail for its stated reason"). The blocks cmp-equal the grade's `f1-fix.diff` and `f1-red-rows.diff`; the document also states their sha256 and the gate checks it without the grade files. Clippy and fmt ran on the repo pin (1.85.0), not the grade's 1.95.0 fmt.

## Case table
In section 2 of the document: closed reader (0, repair 5), other stdout error (0, 5 become 1), failing code kept (64), `--help`/`--version` both ways, stderr write error ignored, clap parse error unchanged.

## STOPs
- S1 (document section 10): which toolkit PR carries the manual sentence, whether it may merge before the toolkit pins an mk that has this fix (pin 0.13.0 now, 0.14.0 after F1, fix is 0.15.0-dev), and whether it carries an `(unreleased: ...)` marker. The grade says only "its lockstep PR" and that it may land before the F1 re-pin of the table rows.

## Proposals (marked P-n)
None; no holder-visible text was invented.

## Small choices to confirm (not holder-visible)
- This fix's FOLLOWUPS entry `crash-sweep-mk-output-sink` is my own wording, `Status: resolved -- shipped by this PR` (the file's existing precedent line).
- The three FOLLOWUPS entries are appended at the end of the file, as a `diff followups` block that `git apply`s.
- Section 7 uses `binary(output_errors)` as the nextest filter for the whole file.

## Not run
The nine plants (listed from the grade's `plants.txt`, not re-run); Windows (first), macOS, musl, release-mode and FreeBSD CI legs; the toolkit manual edit; `mutation-gate.sh`; any review. The mk scripts have no catalog row to update (no `scripts/` directory existed in the repo).

## S1 fold (2026-11-13)
Fold commit `72c46b99be0b2e19fbcc9abe7bdda0e7f2894a6a` (base `15a1825`), files `design/FIX_mk_output_sink.md` and `scripts/mk-output-sink-fix-gate.sh`.
- Section 10: the S1 item is replaced by the ruling's text, verbatim (coordinator `agent-reports/grade-ms-md-output-errors.md` section 4). No STOP is open.
- Section 8: the ruled sentence replaces the STOP sentence; the toolkit companion entry `mk-manual-when-stdout-fails` is added in the ruling's form with mk's values.
- The FOLLOWUPS entry `crash-sweep-mk-output-sink` is renamed `mk-output-sink` everywhere and gains its Companion line; the gate script's FOLLOWUPS grep follows the rename (its one changed line).
- Gate: `GATE PASS` (RED rows 1-5 fail as expected, whole file 1 passed and 5 failed; checker self-test ok; GREEN rows 6 of 6; suite 405 of 405; doc, fmt, clippy ok). The fold agent's first run was killed (exit 137) and its second passed; the controller reran it on the fold head and it passed again.
- Written by the controller from the fold agent's return and its own checks: the agent ended without this section, and its commit message was rewritten before any push to carry the gate output and the two trailer lines.

## R1 fold (2026-11-13)

Folded into `design/FIX_mk_output_sink.md` and `scripts/mk-output-sink-fix-gate.sh` (commit 952b8b2):
- The architect's read-only-stdout ruling: section 3 = `fix.diff` (sha256 62286bfc...), section 4 = `rows-combined-i3.diff` (sha256 89331217..., review I3 combined), all 17 `old:`/`new:` pairs of `texts.md`, counts 7 rows / 406 tests / 6 RED commands. Checked first: the head document plus the architect's blocks and pairs equals `FIX_mk_output_sink.folded.md` byte for byte.
- Review I1 (SIGXFSZ paragraph and the section 9 evidence clause), I2 (step 6 and the section 9 CI item list), I3 (P10 "only 64 kept", plants renumbered P11 and P12, table cell, heading), I4 (step 3a). The combined lines of `texts.md` L241-273 were used where they overlap.
- Gate: compares against `/scratch/cache/architect-output-sink-ro/mk` (`fix.diff`, `rows-combined-i3.diff`); expects 6 RED commands, 7/7 rows, 406/406 suite.
- FOLLOWUPS: 5 Minors and 5 Nits filed (commit 2) before `vendor-freshness-shared`, so section 3.7's hunk still applies (`git apply --check` passes on the head).
- Gate result: `GATE PASS`.

Count changes beyond the inputs' text, for the coordinator to confirm: I2's "the six rows" now reads "the seven rows" (the review predates the seventh row); the gate-coverage line in section 9 now says "the plants (P1 to P12)" instead of "the nine plants". Section 4's wording "byte for byte `f1-red-rows.diff`" is the architect's line and stays, though the rows now differ from that file.

STOP: none.
