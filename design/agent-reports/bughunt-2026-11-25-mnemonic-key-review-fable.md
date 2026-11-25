# Final review (Fable): mnemonic-key `claude/project-thread-utm1aa` (mk#1-mk#4)

Reviewed `git diff 113d8b6..6c6d83b` (merge-base with `origin/main`), read-only. Probes used the debug `mk` built from HEAD. No tracked file was changed.

**Verdict: 0 Critical, 0 Important, 1 Minor, 4 Nit.** All four findings are fixed as the PR body describes. The branch is fit to be offered as a PR.

## Gates run (HEAD 6c6d83b)

- `CARGO_INCREMENTAL=0 cargo test --locked -j 2 -p mk-cli`: 354 passed, 0 failed, 0 ignored.
- `CARGO_INCREMENTAL=0 cargo test --locked -j 2 --workspace`: 580 passed, 0 failed, 0 ignored (the PR body's number).
- `cargo +1.95.0 fmt --all -- --check`: clean.
- `cargo clippy --locked -j 2 --workspace --all-targets -- -D warnings`: clean.
- Logs: `fable-mk-test.log`, `fable-mk-ws.log`, `fable-mk-clippy.log` in the scratchpad.

## Answers to the review questions

**Is each finding fixed?** Yes.
- **mk#2:** the report's reproduction (a non-cosigner card minted with a bare stub, verified against the complete keyed policy) now exits 4 in text and `--json`. A real cosigner at its policy origin exits 0. `check_card_against_policies` (`crates/mk-cli/src/cmd/verify.rs:222-312`) matches encode's `check_card_against_slots` plus its membership test case for case. Membership is refused only when `cosigners` is `Some`, which requires a key for every placeholder, so a partial key list never refuses or claims membership.
- **mk#4:** no body of the xprv appears in output for `--xpub` with the bare key, a leading space, a trailing or leading extra character, a descriptor wrapper, a dropped first character, an upper-cased prefix or a space after the prefix, with `--json`. Key-file and `mk verify --xpub` routes are covered by tests. The ten version-byte rows match BIP-32 and SLIP-0132.
- **mk#3:** every md1 group passes through `decode_md1_card`, whose ASCII guard runs before md-codec. The only other md-codec call (`md1_chunk_set_id`, grouping) cannot accept a card. The error text is identical to md-codec 0.48.5's (`descriptor-mnemonic/crates/md-codec/src/codex32.rs:167`).
- **mk#1:** the hint appears; the exit code (1) and the old prefix are unchanged.

**Wire format or existing cards?** No change. The diff touches only `crates/mk-cli` and `design/`: no mk-codec, `vendor/`, `Cargo.toml` or `Cargo.lock` change. Encoding, stubs and decode are untouched. The only behaviour change for an existing card is that `mk verify --from-md1` can now exit 4 where it exited 0 (see F-M1).

**Regressions or new secret leaks?** None found. Comments are stripped before a key record is parsed, the caller adds only file and line, and the token-start test cannot match a public key, a fingerprint or a path component.

**Do the tests test the behaviour?** Yes; they drive the binary and assert exit codes and output in both modes. The known gaps are already filed (r1 M4, r2 M2).

**PR body accuracy.** Checked and correct: test counts (7, 8, 5), the 580 total, fmt and clippy, "no flags changed", the exit codes, the R2-M1 exception, and #42 (open, the report-only PR). See F-N1 and F-N4 for two small additions.

## Findings

### F-M1 (Minor): verify now fails a card against an elided-origin policy, without encode's remedy

`crates/mk-cli/src/cmd/verify.rs:262-279`.

**Scenario.** A keyed md1 that states no origin for a key gets the template's standard origin (`m/48'/0'/0'/2'`). A card for that key at its real origin, say account 1, was minted with a bare stub. Before this branch `mk verify --from-md1` said OK. Now:

```
error: verify mismatch on origin_path: expected m/48'/0'/0'/2' (the origin of @0 of the wallet policy with stub 38bd7cec; the policy states no origin for it, so this is the standard origin its policy id uses), got m/48'/0'/1'/2'    rc=4
```

This is consistent with `mk encode`, which refuses the same pair, so I do not grade it higher. But the key is the right key and the plate may be the correct one; the md1 is what lacks the origin. Encode's message says so ("re-encode the md1 with the keys' real origins ... and mint against the new md1"); verify's does not, so an operator may conclude the plate is wrong and re-engrave it. `verify.rs`'s own comment at the stub comparison calls that the expensive direction.

**Fix.** Append encode's remedy sentence to the `standard_filled` note, and add one row with the `ELIDED` fixture (also covers part of r1 M4). Mention this case in the CHANGELOG entry.

### F-N1 (Nit): FOLLOWUPS and CHANGELOG cite a file this branch does not hold

`design/FOLLOWUPS.md` (the ten new lines, `src=`/`refs=BugHunt-2026-11-25-fable.md#...`). That file exists only in open PR #42. If this PR merges first, the references dangle. **Fix:** merge #42 first, or say so in the PR body.

### F-N2 (Nit): a non-ASCII character in one chunk of a set is reported as a missing chunk

`crates/mk-cli/src/cmd/verify.rs:139-152` (grouping runs before the guard in `cmd/mod.rs:354-360`). When the character does not alias the right symbol, `md1_chunk_set_id` fails for that chunk, it forms its own group, and the first group reports:

```
error: md1 input rejected: chunk set incomplete: got 3 chunks, expected 4    rc=1
```

All four chunks were supplied. It fails closed, and any mistyped chunk already behaved this way, so this is diagnostics only. **Fix:** run the ASCII check over all `--from-md1` values before `group_md1_cards`; add a chunked row.

### F-N3 (Nit): the stub-derivation notes are now a detached `//` block

`crates/mk-cli/src/cmd/mod.rs:275-298`. Removing `derive_stub_from_md1_card` left its doc as a plain comment between two items, so it no longer appears in rustdoc and reads as attached to `xpub_identity_65`. **Fix:** move it into `decode_md1_card`'s doc comment.

### F-N4 (Nit): the PR body does not name mnemonic-gui

No flag changed, so the GUI's mk schema mirror needs no update. Say that in "Follow-ups in other repos", and note that the GUI will see the new stderr note and the new `cosigner` mismatch field if it runs `mk verify --from-md1`. When r1 M3 (help text) is done, the toolkit manual mirrors `--help` verbatim, so the help and manual change must land together.

## Already filed, grades confirmed

r1 M1-M4, N1-N3 and r2 M1, M2, N1 are in `design/FOLLOWUPS.md` as written; I would not raise any of them. The saved r2 report is byte-identical to the scratchpad copy.

## Not checked

- I did not run the toolkit or GUI against this `mk`.
- The duplicate-key arm of verify was read, not run (r1 ran it by hand).
