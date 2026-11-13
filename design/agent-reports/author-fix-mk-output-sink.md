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
