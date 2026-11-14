# Build report: mk-remedies, the `mk address` multisig pointer (item R)

- Base: origin/main cea6c70bc940616aa302a33efff253932113ace4 (did not move before the build).
- Branch: fix/mk-remedies. Head: see the PR and the final commit (this report is the last commit).
- PR: filled in below after the push.

## RED
Row `multisig_advisory_prints_the_md_address_route` (crates/mk-cli/tests/cli_address.rs), run on the unmodified base:
`cargo nextest run --locked -j 8 -p mk-cli --test cli_address multisig_advisory_prints_the_md_address_route`
Result: `Summary [   0.008s] 1 test run: 0 passed, 1 failed, 15 skipped` (left: "mnemonic verify-bundle", right: "md address <MD1>... --from-mk1-file FILE").

## GREEN
Same file after applying mk-fix.diff and the SPEC copy: `Summary [   0.023s] 16 tests run: 16 passed, 0 skipped`.

## Executed evidence (pointer row, grader's md binary)
`bash /scratch/cache/grade-md-mk-remedies/rows/mk_pointer_row.sh <worktree>/target/debug/mk`:
```
mk address rc=64
printed: md address <MD1>... --from-mk1-file FILE
ran: md address md1ypfds3cf... rc=0 bc1q3x9sy9cfx7y90rcscpqhad37fwzwk4ernn802cs9737ngmq224wspuk4z8 note: composed wallet id c147b29e · policy shape id aad0e0e0
PASS
```

## Tests that changed
None existing pinned the old text (grep for `verify-bundle` / `descriptor tooling` in tests: none). One row added.

## Other files
CHANGELOG (crates/mk-cli, Unreleased, Changed), design/SPEC_mk_derive_address.md:43, design/FOLLOWUPS.md (new `policy-id-stub-256-refused` deferred-minor; p2tr measurement appended to F3 entry `crash-sweep-mk-address-json-memory`; new `mk-mutants-audit-owed`).
The SPEC copy no longer mentions "(M6)" and the `mnemonic convert --to address` hatch: that is the grade's text verbatim.

## STOPs
None.
