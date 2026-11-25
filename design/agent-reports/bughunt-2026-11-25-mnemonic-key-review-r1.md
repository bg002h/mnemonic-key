# Whole-diff review r1: mnemonic-key `claude/project-thread-utm1aa` (mk#1-#4)

Reviewed: `git diff origin/main...claude/project-thread-utm1aa` (35c5d59, 015d106, 34c69b8), read-only. Commands were run with the debug `mk` built from this branch.

**Verdict: 0 Critical, 1 Important, 4 Minor, 3 Nit.** mk#2, mk#3 and mk#1 are sound. mk#4 is fixed for extended private keys on every route, but the key-file route still prints other secret text in full, even though the CHANGELOG says it no longer does (I1).

## Gates run

- `cargo +1.95.0 fmt --all --check`: clean.
- `cargo clippy --locked -j 2 -p mk-cli --all-targets -- -D warnings`: clean.
- `cargo test --locked -j 2 -p mk-cli` (CARGO_INCREMENTAL=0), captured once: **353 passed, 0 failed, 0 ignored**, 41 binaries. The new files `bh1125_md1_intake` (5), `bh1125_private_key_echo` (7) and `bh1125_verify_membership` (7) pass, and so do `template_id_stub` (8) and every existing slip132, keyfile and verify test.
- `cargo search md-codec`: crates.io's newest is 0.42.0. The claim that the pin cannot move yet is true; the sibling repo is at 0.48.7.

## Findings

### I1 (Important): key-file intake still prints non-extended secret text in full, in text mode and in the `--json` envelope

`crates/mk-cli/src/keyfile.rs:58-61` (`usage()`, `got {s:?}`), `:82` (`key {xpub_str:?} carries a derivation suffix`) and `:88` (`record {s:?} declares an origin but no xpub`) still quote the record. The new guard at `:55` catches only text with an xprv/tprv/SLIP-0132 prefix. A WIF key, a hex private key or a BIP-39 phrase in a key file is printed whole. Reproduced:

```
$ printf 'abandon abandon ... abandon about\n' > k2.txt; mk encode --in k2.txt --policy-id-stub aabbccdd
error: k2.txt:1: expected BIP-380 origin notation `[fingerprint/path]xpub`, got "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"   rc=64
$ printf '5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ\n' > k1.txt; mk encode --in k1.txt ...
error: k1.txt:1: expected BIP-380 origin notation ..., got "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ"
$ printf '[3442193e/84h/0h/0h]5HueCGU8...yTJ/0/*\n' > k3.txt; mk encode --in k3.txt ...
error: k3.txt:1: key "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ/0/*" carries a derivation suffix; ...
```

**Failure scenario.** An operator points `--keys` or `--in` at the wrong file, for example a plaintext seed backup or a wallet export holding WIFs. The first line of that file reaches the terminal, a CI log or a wrapper's captured stdout (`--json`). The bug report names this route as the dangerous one ("the echo is the first time it reaches the terminal"). The report's fix direction says "never echo an unparsed key argument in full". The CHANGELOG entry says "Other unparsable key text is named by its first four characters and its length ... instead of being quoted whole", which is false for this route. Only the `[fp/path]<key>` shape goes through `invalid_xpub`.

**Fix.** In `parse_key_record`, never quote the record. The caller already names the file and line; give the shape problem and at most the length. Add a test row for a WIF line, a phrase line and a `[fp/path]WIF/0/*` line.

### M1 (Minor): other flag values are still echoed whole

`crates/mk-cli/src/cmd/mod.rs:25,28` (`--policy-id-stub`), `:38` (`--origin-fingerprint`) and `:52` (`--origin-path`) quote `{s:?}`. Verified: an xprv given as the value of `--origin-path`, `--policy-id-stub` or `--origin-fingerprint` is printed in full, and also inside `--json`. One plausible way this happens is an unquoted `m/48'/0'/0'/2'` on a shell line: its apostrophes open quotes that pull the arguments after it into one value. clap's own `unexpected argument 'xprv…' found` (positional `mk encode xprv…`) also echoes it, but there the value was on argv already. Lower likelihood than the `--xpub` and key-file routes. A cheap fix is to run `contains_private_key_text` in these three parsers too.

### M2 (Minor): `--json` cannot tell a proven membership from a stub-only pass

When a `--from-md1` policy is keyless or sparse and does not hold the key, the only signal is a stderr note (`verify.rs:308-310`). The `--json` success envelope (`emit_ok`) is byte-identical to a pass where membership was proven. Verified:

```
$ mk verify --in t.txt --from-md1 md1yqpqqxzq2qwfv8urt848e --json
note: ... only its stub was compared; that does not show this card's key is one of its cosigners   (stderr)
{"chunk_set_id":{...},"chunks":2,"ok":true,"policy_id_stubs":["559e64b2"],"schema_version":1}
```

A JSON consumer (GUI or script) that ignores stderr cannot tell the two cases apart. That is the same over-claim the fix removes from the text path. Suggested fix: add a per-policy field such as `"membership":"checked"|"stub_only"`. This is an additive JSON shape change, so it would need the toolkit and GUI lockstep.

### M3 (Minor): help text and manual not updated for the new verify behaviour

The clap help (`verify.rs:52`, "Expected `policy_id_stub` derived from md1 strings") and toolkit `docs/manual/src/40-cli-reference/44-mk-cli.md:425` still describe `--from-md1` as a stub source only. `:451-453` says exit 4 is "when an expected-* flag disagrees with the decoded value", and the doc lists no `cosigner` field. The toolkit checkout has no paired change. No flag changed, so the CLAUDE.md lockstep rule is not strictly triggered. Still, the manual now under-describes what `mk verify --from-md1` refuses and does not explain the new stderr note. The private-key refusal and the md1 non-ASCII and wire-version limits are also undocumented there.

### M4 (Minor): test gaps

- No test covers verify's duplicate-key arm (`verify.rs:292-305`). I ran it by hand with the `REP` fixture from `bh1119_keyed_policy.rs`: exit 4, `{"field":"cosigner",...}`. It works but is unguarded.
- No verify rows for the `NOFP`, `ELIDED` or `SIGNET` (testnet) fixtures. By hand, `NOFP` and `ELIDED` with X0 pass at exit 0 (ELIDED shares A's stub 38bd7cec, the standard-origin fill).
- No test runs a stub-only note with `--json`.
- `PRIVATE_VERSIONS.contains(&ver)` (`slip132.rs:193`) cannot be reached for a well-formed 78-byte payload: those version bytes always base58-encode to the matching prefix, which `contains_private_key_text` refuses first. Deleting it turns no test red. It is harmless defence in depth, but say so or drop it.
- No test fails if the token-start check is turned into a substring check, which would refuse valid xpubs whose base58 body contains `xprv`.

### N1 (Nit): the four-character head on other invalid text

`invalid_xpub` (`slip132.rs:168-173`) prints the first four characters. For a pasted BIP-39 phrase that is enough to identify the first word (11 bits); for a hex private key it is 16 bits, and for a WIF about 17 bits. This is not a practical leak. To show four characters of an xprv is safe by construction: `xprv` is the version prefix, and xprv text never reaches this function anyway. Printing the head only when it is a known public prefix would remove even this.

### N2 (Nit): wording

- The mk#1 hint (`error.rs:92-100`) also appears for `mk verify --from-md1`, where "Minting with --policy-id-stub" does not apply. It also points an encode user at the unchecked route. The warning is there, but "run the policy through a newer `md`/toolkit instead" would be a better first suggestion.
- The duplicate-key message reads "expected a wallet policy whose keys are pairwise distinct (BIP 388), got the wallet policy with stub 58e73a11 holds this card's xpub at @0, @1". The grammar is broken. The exit code (4, from verify's mismatch convention) is fine, even though encode refuses the same policy with 64.

### N3 (Nit): the new stderr note fires on every ordinary single-sig template verify

This is by design (the template bundle workflow used to be silent). A wrapper that treats any stderr output as a warning will now flag every template verify.

## What held

1. **mk#4 on its own terms.** The private-key refusal covers `mk encode --xpub` (including `--xpub=`, a leading space, a quoted value and `--json`), every key-record shape (`[fp/]`, `[fp/path]`, bare, `/0/*` suffix, `[fp]` with no path, a `wpkh(...)` descriptor line) and `mk verify --xpub`. Positional xprv given to decode, verify, repair, derive, inspect and address is not echoed (checked: 0 occurrences).
   - **No false refusals.** The check is a token-start check on ASCII-alphanumeric tokens, so a base58 xpub (a single token) is refused only if it begins with a private prefix. No public form (xpub, tpub, ypub, zpub, Ypub, Zpub, upub, vpub, Upub, Vpub) can. Fingerprints (hex) and path tokens cannot spell one either. The SLIP-0132 public-acceptance tests and `a_public_key_still_encodes` pass.
   - **Version bytes.** All ten private version-byte pairs and prefixes are consistent with BIP-32 and SLIP-0132. I checked them from memory, not against the source text, but the prefix check would refuse them anyway.
   - **Debug output.** `KeyRecord` and `VerifyArgs` derive Debug, but neither is printed.
2. **mk#2.** `check_card_against_policies` mirrors encode's `check_card_against_slots` plus its membership check, case for case:
   - A key in one slot has its path and fingerprint checked; a missing fingerprint on either side is skipped.
   - A complete key list without the key gives exit 4.
   - A sparse or keyless policy without the key gets a note and never a refusal. That is the "partial list is never exhaustive" rule from CLAUDE.md.
   - A key in a filled slot of a sparse policy is checked and passes with no note.
   - **Membership is never claimed without proof.** `cosigners` is `Some` only when `pubkeys.len() == n`.
   - **Order.** The stub multiset comparison still runs first, so a stub mismatch is reported as before.
   - **Exit codes.** Exit 4 and the `ContentMismatch` envelope with `field` match verify's other mismatches.
3. **mk#3.** The guard runs after `strip_display_separators`, which strips all Unicode whitespace, so an md1 grouped with NBSP or ideographic spaces is still accepted. `--from-md1-set` skips non-`md1` lines (and their annotations) before the guard. Lines starting with `md1` that carry trailing comments were already undecodable. No valid input is newly refused.
4. **mk#1.** The hint appears for the single-string wire-v8 card and for the 8-chunk keyed `keyed_tr_liana_kofn_recovery` set (`--from-md1-set`). Older-version and other `MdCodec` errors are unchanged.
5. **The tests catch the bugs.** Each test fails without its fix:
   - Membership: the pre-fix verify exits 0 with no note, which reds `non_cosigner…`, both `contradicting_*` tests and the two `UNDECIDED` assertions.
   - Non-ASCII: the pre-fix code exits 0 on the aliased string, so all three non_ascii tests fail on `Some(1)`.
   - Private-key echo: the 12-character-window checks fail on the old full-quote messages, in both text and `--json`.
   - Wire version: the hint assertions fail without the `error.rs` arm.
   - Gaps are listed in M4.

## Recommendation

Fix I1 before this goes up for merge. It is small and confined to `keyfile.rs`. The CHANGELOG's own claim makes it a correctness issue of the fix, not just scope. M1-M4 can follow or be filed. Re-review scope after the fold: `keyfile.rs` plus its new test rows.
