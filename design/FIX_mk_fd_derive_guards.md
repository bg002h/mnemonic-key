# FIX_mk_fd_derive_guards: bughunt-1119 F-D, `mk address` / `mk derive` guards (bh-mk-C1, bh-mk-I18)

Small-fix track, grade I (both keys). Class B (address output). Repo mnemonic-key, branch
`fix/mk-fd-derive-guards`, base `origin/main` 8a738ab ("release: mk-cli 0.17.0"). Toolkit manual
`docs/manual/src/40-cli-reference/44-mk-cli.md` changes in lockstep (separate toolkit change).
Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22.

Inputs (decided; not re-ruled here): coordinator `agent-reports/architect-bughunt-1119-questions.md`
row #19, T42 (I18) and P3; Brian's ruling B3 (`memory/product-rulings-sweep-bughunt-2026-11-22.md`:
"`mk address`/`mk derive` refuse child derivation, pointing at the key's own address"); grader
`agent-reports/bughunt-1119/grade-mk.md` (C1, I18, F-D); triage `triage-mk-t1.md` (C1),
`triage-mk-t4.md` (I18).

Section 8 records Brian's ruling (Option B) on how the C1 refusal points at the key's own
address; with it everything below is decided (no open STOP).

## 1. Defects (at 8a738ab, rows in section 5 run and seen)

- **bh-mk-C1.** The toolkit writes a single WIF key as an mk1 card holding a depth-0 xpub with an
  all-zero chain code (mnemonic-toolkit `crates/mnemonic-toolkit/src/cmd/bundle.rs` L781-822 at
  origin/master b0b99a16; `convert.rs` wif->xpub builds the same sentinel). `mk address` on that
  card with `--address-type` derives `m/0/i` children (address.rs L79-125) and prints receive
  addresses no wallet watches, exit 0, with only the generic depth advisory; `mk derive` derives a
  child xpub from it, exit 0 (derive.rs L49-76). SPEC_mk_derive_address.md L47 (step 5) specifies
  "proceed (permissive-with-warning)".
- **bh-mk-I18.** `mk derive --path ''` (and `m`, `m/`, which parse to the same empty
  `DerivationPath`) prints the parent xpub labelled `child_xpub`, `relative_path: m/`, exit 0
  (derive.rs `parse_relative_unhardened` L118-129 accepts an empty path).

## 2. Decided behaviour

### 2.1 Fixed-key card (C1)

- **Predicate** ("fixed-key card"): the decoded `card.xpub` has `depth == 0` and
  `chain_code.to_bytes() == [0u8; 32]`. Same definition as the sibling FIX
  (`design/FIX_mkc_keycard_validation.md` on `fix/mkc-keycard-validation`, "All-zero chain code"),
  so the two fixes name one card shape. A depth-0 card with a real chain code (a BIP-32 master
  xpub) is NOT a fixed-key card and keeps today's behaviour (derives, depth advisory, exit 0).
  A zero chain code at depth > 0 is not the toolkit card and is unchanged.
- **Commands:** both `mk address` and `mk derive` refuse (B3). `mk decode`, `mk inspect`,
  `mk verify`, `mk encode`, `mk repair` are unchanged.
- **Exit and variant:** `CliError::UsageError(<text>)`, exit 64, nothing on stdout in text mode;
  in `--json` mode the existing error envelope on stdout
  (`{"error":{"details":null,"exit_code":64,"kind":"UsageError","message":"<text>"},"schema_version":1}`).
  No new variant, flag or type.
- **Placement (both commands, Brian's ruling B, section 8):** the refusal runs right after
  `warn_chunk_set_id_mismatch` (and after `warn_if_invalid(&card)` when FIX_mkc_keycard_validation has landed: both warnings print first, then the refusal; a builder rebasing onto it keeps that order, and the exact-stderr rows stay as written because every fixture card is valid, with no origin fingerprint or the key's own) and before everything else the command checks: for `mk address`
  before address-type resolution (so before the multisig refusal and the "--address-type
  required" error), network resolution, the depth advisory and `--count`/`--range`/`--chain`; for
  `mk derive` before `--path`/`--index` are parsed. A fixed-key card is refused whatever flags
  are given (`--address-type` absent, `--network` mismatching the xpub, a hardened, empty or
  unparsable `--path`). Rationale: the card, not the flag, is the problem (same reasoning as the
  multisig refusal holding even with `--address-type`, SPEC L43). Clap's own argument errors
  (unknown flag, `--count` with `--range`) still fire first: they run before `run`.
- **Text (both commands, same wording; Brian's ruling B, fold of FIX r1 I-1/I-2).** `{XPUB}` is
  the card's xpub as base58 (`card.xpub` Display, the string `mk derive` prints as `parent_xpub`).
  The printed command has no angle brackets, `|` or other shell metacharacter, so it can be pasted
  as printed: `{TYPE}` is the `--address-type` value when `mk address` got one, else the literal
  word `TYPE`; the final ` --network {NET}` clause is appended only when the xpub is test-kind
  (`card.xpub.network == NetworkKind::Test`), and `{NET}` is the `--network` value when `mk address`
  got `testnet`, `signet` or `regtest`, else the literal word `NETWORK` (`mk derive` has neither
  flag, so it always prints `TYPE` and `NETWORK`; `--network mainnet` on a test-kind card prints
  `NETWORK`). The `--address-type` names equal the toolkit's `--script-type` names (p2pkh, p2wpkh,
  p2sh-p2wpkh, p2tr), so no mapping is needed. Run unedited, the bare words make the toolkit print
  its own value list (exit 64), so the paste itself tells the holder how to fix it. No trailing
  period.

  > `this card holds one fixed key (depth 0, all-zero chain code; the toolkit writes a WIF key this way) and has no child keys any wallet watches, so mk derives nothing from it. If this key is one of the keys in a multisig wallet, that wallet's addresses come from its md1 policy card (md address), not from this key alone. The key's own address: mnemonic convert --from xpub={XPUB} --to address --path m --script-type {TYPE}` + (test-kind only) ` --network {NET}`

  Printed for `mk address --in leafm.mk1 --address-type p2wpkh` (mainnet fixture, section 5),
  stderr, exit 64, stdout empty:

  ```
  error: this card holds one fixed key (depth 0, all-zero chain code; the toolkit writes a WIF key this way) and has no child keys any wallet watches, so mk derives nothing from it. If this key is one of the keys in a multisig wallet, that wallet's addresses come from its md1 policy card (md address), not from this key alone. The key's own address: mnemonic convert --from xpub=xpub661MyMwAqRbcEYS8w7XLSVeEsBXy79zSzH1J8vCdxAZningWLdN3zgtU6QzvJsNBNF5QPBBBg1yVF2LKrcfGdJq86PeLWDMUCYatZPzQu8R --to address --path m --script-type p2wpkh
  ```

  Printed for `mk address --in leaf.mk1 --address-type p2wpkh --network signet` (test-kind fixture):

  ```
  error: this card holds one fixed key (depth 0, all-zero chain code; the toolkit writes a WIF key this way) and has no child keys any wallet watches, so mk derives nothing from it. If this key is one of the keys in a multisig wallet, that wallet's addresses come from its md1 policy card (md address), not from this key alone. The key's own address: mnemonic convert --from xpub=tpubD6NzVbkrYhZ4WLczPJWReQycCJdd6YVWXubbVUFnJ5KgU5MDQrD998ZJLT5hqNKRA9cJzkh4D7oXwCpwFUWWWkHR9zPeBk1hnWBJKNhyYsn --to address --path m --script-type p2wpkh --network signet
  ```

  Printed for `mk derive --in leaf.mk1 ...` (and `mk address --in leaf.mk1` with neither flag):
  the same with `--script-type TYPE --network NETWORK`.

  In `--json` mode the same text is the `message` of the envelope in 2.1 (no escaping applies:
  the text has no double quote, backslash or control character).
- **The printed command runs** (toolkit 0.106.0): pre-filled forms, mainnet fixture ->
  `address: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4`; test-kind fixture with `--network signet`
  -> `address: tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx`; both exit 0. The bare-word forms exit 64
  with the toolkit's own value list.
- Consequence: mk's text names a toolkit command (precedent: the multisig refusal names
  `md address`); no address appears in the refusal itself, only from a successful command.

### 2.2 Empty relative path (I18, T42)

- `parse_relative_unhardened` refuses a path whose parsed `DerivationPath` has no components,
  checked after the parse succeeds and before the hardened check. This refuses `''`, `m`, `m/`
  and any other spelling bitcoin's parser maps to the empty path; a string the parser rejects
  (e.g. `/`) keeps today's "invalid derivation path" error.
- `CliError::UsageError`, exit 64, nothing on stdout in text mode; `--json` error envelope as
  above. `--index N` cannot produce an empty path and is unchanged.
- Text (PROPOSED, architect confirms; built from T42's "derive needs at least one step" and "the
  card's own key is `mk decode`"), where `{s:?}` is the raw `--path` value Debug-quoted as in the
  existing invalid-path error:

  > `--path {s:?} has no steps: derive needs at least one step (e.g. m/0/5); the card's own key is shown by \`mk decode\``

  Printed for `--path ''`: ``error: --path "" has no steps: derive needs at least one step (e.g. m/0/5); the card's own key is shown by `mk decode` ``

## 3. Spec delta (`design/SPEC_mk_derive_address.md`)

1. §3.1 address-type resolution, insert before step 1 (L43) a new step 0 (text of the refusal
   per section 2.1):
   > 0. **Fixed-key card refuse FIRST (bh-mk-C1, ruling B3, Option B).** A card whose xpub has depth 0 and an all-zero chain code holds one fixed key (the toolkit writes a WIF key this way) and has no child keys that any wallet watches. `mk address` refuses it right after the chunk-set-id warning and before steps 1-4 and network resolution, whatever `--address-type`, `--network`, `--count`/`--range` and `--chain` say. The refusal prints a ready-to-paste `mnemonic convert --from xpub=<XPUB> --to address --path m --script-type TYPE` command that renders the key's own address (plus `--network NETWORK` for a test-kind xpub); `TYPE` and `NETWORK` are pre-filled from `--address-type` and `--network` when those were given. If the key is one of the keys in a multisig wallet, that wallet's addresses come from its md1 policy card (`md address`), not from this key alone, and the refusal says so. `CliError::UsageError` → **exit 64**.
1b. §3.1 step 1 (L43): replace "**Multisig refuse FIRST**" with "**Multisig refuse** (first after step 0)".
2. §3.1 step 5 (L47): replace "then proceed (permissive-with-warning; matches the constellation's
   silent-default-with-stderr-notice convention)." with:
   > then proceed (permissive-with-warning; matches the constellation's silent-default-with-stderr-notice convention). A fixed-key card never reaches this step (step 0).
3. §3.2 Path bullet (L71), append:
   > A `--path` that parses to no components (`''`, `m`, `m/`) → ERROR `CliError::UsageError` ("derive needs at least one step") → **exit 64** (bh-mk-I18); the card's own key is `mk decode`'s output.
4. §3.2, new bullet after "Multisig" (L72):
   > - **Fixed-key card:** refused (bh-mk-C1, ruling B3), before the path is parsed: `CliError::UsageError` → **exit 64**; same text as §3.1 step 0 (FIX section 2.1).
5. §5 test plan: add after row 4b:
   > 4c. **Fixed-key card (C1)** — a depth-0 card with an all-zero chain code → **exit 64** with or without `--address-type` and whatever `--network` says, no address on stdout, the refusal fires before the "--address-type required" and network errors; a depth-0 BIP-32 master card (real chain code) with `--address-type` still succeeds with the depth advisory.
   and after row 11:
   > 11b. Empty relative path (I18) — `--path ''`, `m`, `m/` → **exit 64**, no `child_xpub`; `--path m/0/0` unchanged.
   > 11c. Fixed-key card (C1) — `mk derive --index 0` and `--path m/0/0` on the fixed-key card → **exit 64**, no `child_xpub`; with a hardened, empty or unparsable `--path` the refusal still fires first.

## 4. Code (anchors at 8a738ab; builder writes the code)

- `crates/mk-cli/src/cmd/derive.rs`: after L54 (`warn_chunk_set_id_mismatch`, and after `warn_if_invalid(&card)` when FIX_mkc_keycard_validation has landed: both warnings print first, then the refusal; the exact-stderr rows stay as written because every fixture card is valid), the fixed-key
  refusal; in `parse_relative_unhardened` (L118), the empty-path refusal before the hardened check.
- `crates/mk-cli/src/cmd/address.rs`: the fixed-key refusal after L84 (`warn_chunk_set_id_mismatch`, and after `warn_if_invalid(&card)` when FIX_mkc_keycard_validation has landed, same order),
  before L86 (`resolve_address_type`).
- One shared predicate helper `is_fixed_key_card(&Xpub) -> bool` in
  `crates/mk-cli/src/cmd/derive_support.rs` (both commands call it; unit-tested there with the
  LEAF and TV1 master xpubs). The refusal text (2.1) is built in one shared function there too, so
  the two commands cannot drift; `{XPUB}` is `xpub.to_string()` and the network clause keys off
  `xpub.network`. Its signature is `fixed_key_refusal(xpub, Option<AddressType>, Option<CliNetwork>)` (`mk derive` passes `None, None`).
- `--help` texts unchanged (no doc-comment change), so `gen_man` / `gui_schema` outputs and the
  GUI mirror do not move.

## 5. Rows

Committed runner: `scripts/mk-fd-derive-guards-rows.sh red|green` (builds `mk` with
`cargo build --locked -p mk-cli` on the 1.85 pin, mints four public-key cards with `mk encode`
under `$REPO/target/mk-fd-rows` (override with `WORK`), runs every row below, checks the mode's
expectation; a `=` check compares the whole stream; its header says what it does not cover).
Runs (after the Option B fold): `red` at the base code -> `ALL ROWS MATCH red`; `green` with a
scratch Option B implementation (shared predicate and text, empty-path refusal; reverted, not
committed) -> `ALL ROWS MATCH green`.

Fixtures: `leaf.mk1` = `mk encode --xpub tpubD6NzVbkrYhZ4WLczPJWReQycCJdd6YVWXubbVUFnJ5KgU5MDQrD998ZJLT5hqNKRA9cJzkh4D7oXwCpwFUWWWkHR9zPeBk1hnWBJKNhyYsn --origin-path m --privacy-preserving --policy-id-stub deadbeef`
(generator point, zero chain code, test-kind: the fixed-key shape); `leafm.mk1` = the same with
`xpub661MyMwAqRbcEYS8w7XLSVeEsBXy79zSzH1J8vCdxAZningWLdN3zgtU6QzvJsNBNF5QPBBBg1yVF2LKrcfGdJq86PeLWDMUCYatZPzQu8R`
(mainnet version bytes); `acct.mk1` = V2 BIP-84 account
`xpub6BmeGm...Cp2z6a`, `--origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a`;
`master.mk1` = BIP-32 TV1 master `xpub661MyMwA...EGMcet8`, `--origin-path m --origin-fingerprint 3442193e`.

In the table, T-TEST[type,net] and T-MAIN[type] are the two printed texts of section 2.1 (test-kind
xpub with its `--script-type` and `--network` words; mainnet xpub with its `--script-type` word);
"exact" = the whole stream equals the value plus a newline. GREEN C1 rows: exit 64, stdout empty
(text mode) and stderr exactly `error: ` + the text.

| row (test fn) | command | RED at fc1dca9 base code (seen) | GREEN |
|---|---|---|---|
| `fd_c1_address_refuses_fixed_key_card` | `mk address --in leaf.mk1 --address-type p2wpkh --network signet --count 1` | exit 0, stderr depth advisory, stdout `  0  tb1qgl57xjd595rrxmpwys85q40xh4nmqunqrmrmwp` | exit 64, stdout empty, stderr exact `error: ` T-TEST[p2wpkh,signet] |
| `fd_c1_address_refuses_fixed_key_card_json` | as above + `--json` | exit 0, `{"address_type":"p2wpkh","addresses":[{"address":"tb1qgl57...rmwp",...` | exit 64, stderr empty, stdout exact `{"error":{"details":null,"exit_code":64,"kind":"UsageError","message":"` T-TEST[p2wpkh,signet] `"},"schema_version":1}` |
| `fd_c1_address_refuses_fixed_key_card_mainnet` | `mk address --in leafm.mk1 --address-type p2wpkh --count 1` | exit 0, stderr depth advisory, stdout `  0  bc1qgl57xjd595rrxmpwys85q40xh4nmqunqfacg4j` | exit 64, stdout empty, stderr exact `error: ` T-MAIN[p2wpkh] |
| `fd_c1_derive_refuses_fixed_key_card` | `mk derive --in leaf.mk1 --index 0` and `mk derive --in leaf.mk1 --path m/0/0` | exit 0, `child_xpub: tpubDAPnywfgE8GS...hcpm8K2`, `depth: 2` | exit 64, stdout empty, stderr exact `error: ` T-TEST[TYPE,NETWORK] (both) |
| `fd_c1_derive_refuses_fixed_key_card_mainnet` | `mk derive --in leafm.mk1 --index 0` | exit 0, `child_xpub: xpub6A2AThqzWrJz...wgapAto`, `depth: 2` | exit 64, stdout empty, stderr exact `error: ` T-MAIN[TYPE] |
| `fd_c1_refusal_precedes_other_errors` (order control) | (a) `mk address --in leaf.mk1` (no address type); (b) `mk address --in leaf.mk1 --address-type p2wpkh --network mainnet` (network kind mismatch); (c) `mk derive --in leaf.mk1 --path "m/0'"` (hardened); (d) `mk derive --in leaf.mk1 --path /` (parser rejects); (e) `mk derive --in leaf.mk1 --path ''` (empty) | exit 64 with the command's own error: (a) `--address-type required: cannot infer ...`, (b) `--network mainnet disagrees with the xpub's network (testnet); ...`, (c) `--path m/0': cannot derive hardened children ...`, (d) `--path invalid derivation path "/": invalid child number format`; (e) exit 0, `child_xpub:` (the I18 defect) | all five: exit 64, stdout empty, stderr exact `error: ` + the refusal text (the refusal fires first): (a) T-TEST[TYPE,NETWORK]; (b) T-TEST[p2wpkh,NETWORK] (`--network mainnet` is not a test-kind network); (c), (d), (e) T-TEST[TYPE,NETWORK] |
| `fd_i18_derive_refuses_empty_path` | `mk derive --in acct.mk1 --path '' --json`; same with `m`, `m/`; text mode with `''` | exit 0, `"child_xpub":"xpub6BmeGm...Cp2z6a"` (= parent), `"relative_path":"m/"`, `"depth":3` | exit 64; `--json`: envelope `"kind":"UsageError"`, no `child_xpub`; text: stderr = the 2.2 text |
| `fd_c1_master_card_still_derives` (control) | `mk address --in master.mk1 --address-type p2wpkh --count 1`; `mk derive --in master.mk1 --index 0 --json` | exit 0, `bc1qp5wfcq48h6d63wyy9qz0awtpfqwwv4sma86mhz` + depth advisory; child `xpub6AvUGrnEpfvJ...EuMWwnj`, depth 2 | unchanged |
| `fd_i18_nonempty_path_still_derives` (control) | `mk derive --in acct.mk1 --path m/0/0 --json` | exit 0, child `xpub6Fec2dLaQAfW...Qv8oYaf`, depth 5 | unchanged |

The builder also adds these rows as nextest tests (test fns named exactly as the first column; the
nextest tests assert the same exact texts as the script) in a new
`crates/mk-cli/tests/bh1119_derive_guards.rs`, RED first, plus the `is_fixed_key_card` unit test.
The rows script stays as the CLI-level check.

## 6. Tests that change

None of the existing tests change: no mk-cli test runs `mk address`/`mk derive` on a depth-0
zero-chain-code card or with an empty `--path` (grep of `crates/mk-cli/tests` at 8a738ab for the
LEAF tpub, `[0u8; 32]`, `"--path", ""`: no hits). `cli_address.rs::leaf_card_requires_then_advises_depth`
and `nonstandard_requires_address_type_then_succeeds` keep passing (not fixed-key cards). The
new check order (fixed-key refusal before address-type, network and path errors) changes no
existing test: no existing mk-cli test uses a fixed-key card (same grep, plus the mk-cli sources
for `[0u8; 32]` and `ChainCode::from`: no hit). The builder confirms with the full suite.

## 7. Toolkit manual lockstep (`docs/manual/src/40-cli-reference/44-mk-cli.md`, master)

- `## mk address` intro paragraph (L486-492), after the sentence ending "...relative to the card's
  xpub)." insert (PROPOSED wording around Brian's ruling B):
  > A card that holds one fixed key (depth 0 with an all-zero chain code — how the toolkit writes a WIF key) is **refused** before anything else is checked (exit 64): it has no child keys that any wallet watches. The refusal prints the `mnemonic convert` command that renders the key's own address; a key in a multisig wallet takes its addresses from the md1 policy card with `md address`.
- `## mk derive` intro paragraph (L541-544), after "hardened components are rejected." insert
  (PROPOSED):
  > A `--path` with no steps (`''`, `m`, `m/`) is rejected (the card's own key is `mk decode`'s output), and a fixed-key card (see `mk address`) is refused.
- Flags table row `--path <REL>` (L559): "relative derivation path, unhardened only, at least one
  step (e.g. `m/0/5`)" (PROPOSED).
- No `.out` include file changes (worked examples use an account card).

## 8. Ruling (closed)

Brian 2026-11-22 chose Option B (coordinator memory `mk-c1-refusal-prints-command-2026-11-22.md`): both commands refuse a fixed-key card first (exit 64), right after the chunk-set-id warning, and print the `mnemonic convert` command for the key's own address (text in 2.1; network suffix only for a test-kind xpub).

## 9. FOLLOWUPS (listed in the author's report, not appended here)

Lines for `design/FOLLOWUPS.md` (to be direct-pushed after merge, per the coordinator rule) are in
the author's report.

## 10. Gate

`scripts/mk-fd-derive-guards-rows.sh green` -> `ALL ROWS MATCH green` (exact C1 texts, order controls and I18 rows included), then the full CI set:
`cargo nextest run --locked --workspace --no-fail-fast`, `cargo test --locked --doc --workspace`,
`cargo +1.95.0 fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`.
