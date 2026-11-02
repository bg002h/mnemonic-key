**⚠ 6 open STOPs (S1-S6, FIX doc §10) must be answered before the build starts.**

# FIX doc author report: mk bug hunt

FIX doc: `design/FIX_bughunt_mk.md` (commit 5551513, branch fix/bughunt, not pushed).
It covers the spec delta (§2.1, §3.5, §6 plus one cap sentence in
SPEC_mk_derive_address.md l.51), exact production fix texts for #5, #1, #3, #4,
the RED rows, the tests that change, the 11 must-stay-green tests, build
order, gate, and manual lockstep. The commit has no production code.

## Evidence (measured)

- RED on unmodified code, `-E 'test(/(^|::)bughunt_mk_/)'`: 8 run, 3 passed,
  5 failed. The passes are expected: renamed 02 (no code change, Q2), 06
  (false positive guard) and the new keys/in row (regression guard). The
  failures are 01 (exit 0 not 64), 03 (8 GiB allocation abort), 04 (panic at
  encode.rs:480), 05 refusal (Kelvin strings accepted) and 05 variant
  (Kelvin at data index 0 accepted).
- fix-01 as drafted, planted: the new keys/in row FAILS (exit 64 with the
  fingerprint refusal), so the row guards the grade §1 regression.
- Scratch GREEN of the specified fixes (reverted): full suite 399/399; row 03
  takes ~30 s. The §3.1 codec shape alone: mk-codec 207/207.
- mnemonic-gui does not mirror the clap help (hand-paraphrased, v0.9.0 pin,
  name-only gate), so it needs no lockstep change. `mk gui-schema` and the
  `gen-man` tests carry no help prose, and there are no snapshots.
- Manual: the grade's l.501/502 are l.499/500 on toolkit origin/master f6355e22.

## STOPs

- S1: the grade's manual texts drop the chapter's backticks. Keep them verbatim or restore the backticks?
- S2: the unreleased marker for a behaviour change on an already released flag (l.16 covers new flags only), which flag name it carries, and whether l.31 (in a code block) gets one.
- S3: the exact clap help for `--origin-fingerprint` and `--privacy-preserving` (the grade says "change to match"). A candidate is given.
- S4: whether the `--count`/`--range` clap help names the cap. A candidate is given.
- S5: Q5 says `InvalidHrp(<hrp as typed>)`, but the binding `to_ascii_lowercase` shape reports the lowercased HRP for uppercase input.
- S6: in a data part holding an invalid ASCII char and a later non-ASCII char, the specified shape reports the non-ASCII one.

Counts: RED rows 8 (5 fail on unmodified code; 3 pass: 02 renamed, 06, keys/in); STOPs 6 open; tests that must stay green 11; manual lines 5; spec sentences 4; C 0, I 0.
