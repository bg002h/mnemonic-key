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

**Open STOP (section 8) must be answered before the build starts.** It decides only how the C1
refusal points at the key's own address (and with it the C1 check order and text). Everything
else below is decided: the I18 half in full, and the C1 predicate, exit code, error variant,
derive-side placement, spec delta skeleton, rows and controls.

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
- **`mk derive` placement:** the refusal runs right after `warn_chunk_set_id_mismatch` and before
  `--path`/`--index` are parsed, so a fixed-key card is refused whatever the path (including an
  empty or hardened one). Rationale: the card, not the flag, is the problem (same reasoning as the
  multisig refusal holding even with `--address-type`, SPEC L43).
- **`mk address` placement and text:** OPEN, section 8 (the two candidate shapes fix different
  orders).
- **`mk derive` text:** OPEN, section 8.

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
   per section 8):
   > 0. **Fixed-key card refuse (bh-mk-C1, ruling B3).** A card whose xpub has depth 0 and an all-zero chain code holds one fixed key (the toolkit writes a WIF key this way) and has no child keys that any wallet watches. `mk address` refuses it, pointing at the key's own address: `CliError::UsageError` → **exit 64**. [order relative to steps 1-4 and network resolution: per section 8.]
2. §3.1 step 5 (L47): replace "then proceed (permissive-with-warning; matches the constellation's
   silent-default-with-stderr-notice convention)." with:
   > then proceed (permissive-with-warning; matches the constellation's silent-default-with-stderr-notice convention). A fixed-key card never reaches this step (step 0).
3. §3.2 Path bullet (L71), append:
   > A `--path` that parses to no components (`''`, `m`, `m/`) → ERROR `CliError::UsageError` ("derive needs at least one step") → **exit 64** (bh-mk-I18); the card's own key is `mk decode`'s output.
4. §3.2, new bullet after "Multisig" (L72):
   > - **Fixed-key card:** refused (bh-mk-C1, ruling B3), before the path is parsed: `CliError::UsageError` → **exit 64**; text per §3.1 step 0's counterpart [section 8].
5. §5 test plan: add after row 4b:
   > 4c. **Fixed-key card (C1)** — a depth-0 card with an all-zero chain code → **exit 64** with or without `--address-type`, no address on stdout; a depth-0 BIP-32 master card (real chain code) with `--address-type` still succeeds with the depth advisory.
   and after row 11:
   > 11b. Empty relative path (I18) — `--path ''`, `m`, `m/` → **exit 64**, no `child_xpub`; `--path m/0/0` unchanged.
   > 11c. Fixed-key card (C1) — `mk derive --index 0` and `--path m/0/0` on the fixed-key card → **exit 64**, no `child_xpub`.

## 4. Code (anchors at 8a738ab; builder writes the code)

- `crates/mk-cli/src/cmd/derive.rs`: after L54 (`warn_chunk_set_id_mismatch`), the fixed-key
  refusal; in `parse_relative_unhardened` (L118), the empty-path refusal before the hardened check.
- `crates/mk-cli/src/cmd/address.rs`: the fixed-key refusal at the position section 8 fixes
  (Option A: after `resolve_network`, L87; Option B: after L84).
- One shared predicate helper `is_fixed_key_card(&Xpub) -> bool` in
  `crates/mk-cli/src/cmd/derive_support.rs` (both commands call it; unit-tested there with the
  LEAF and TV1 master xpubs).
- `--help` texts unchanged (no doc-comment change), so `gen_man` / `gui_schema` outputs and the
  GUI mirror do not move.

## 5. Rows

Committed runner: `scripts/mk-fd-derive-guards-rows.sh red|green` (builds `mk` with
`cargo build --locked -p mk-cli` on the 1.85 pin, mints three public-key cards with `mk encode`
under `/scratch/cache/mk-fd-derive-guards-rows`, runs every row below, checks the mode's
expectation; its header says what it does not cover). Run at 8a738ab by the author:
`red` -> `ALL ROWS MATCH red`; `green` -> `ROWS DIFFER FROM green` (8 FAIL, 3 controls OK). With an
option-neutral scratch fix (both refusals exit 64; reverted) `green` -> `ALL ROWS MATCH green`.

Fixtures: `leaf.mk1` = `mk encode --xpub tpubD6NzVbkrYhZ4WLczPJWReQycCJdd6YVWXubbVUFnJ5KgU5MDQrD998ZJLT5hqNKRA9cJzkh4D7oXwCpwFUWWWkHR9zPeBk1hnWBJKNhyYsn --origin-path m --privacy-preserving --policy-id-stub deadbeef`
(generator point, zero chain code: the fixed-key shape); `acct.mk1` = V2 BIP-84 account
`xpub6BmeGm...Cp2z6a`, `--origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a`;
`master.mk1` = BIP-32 TV1 master `xpub661MyMwA...EGMcet8`, `--origin-path m --origin-fingerprint 3442193e`.

| row (test fn) | command | RED at 8a738ab (seen) | GREEN |
|---|---|---|---|
| `fd_c1_address_refuses_fixed_key_card` | `mk address --in leaf.mk1 --address-type p2wpkh --network signet --count 1` | exit 0, stderr depth advisory, stdout `  0  tb1qgl57xjd595rrxmpwys85q40xh4nmqunqrmrmwp` | exit 64, stdout empty, stderr = section 8 text |
| same, `--json` | as above + `--json` | exit 0, `{"address_type":"p2wpkh","addresses":[{"address":"tb1qgl57...rmwp",...` | exit 64, stdout envelope `"kind":"UsageError"`, no `tb1qgl57` |
| `fd_c1_derive_refuses_fixed_key_card` | `mk derive --in leaf.mk1 --index 0` (and `--path m/0/0`) | exit 0, `child_xpub: tpubDAPnywfgE8GS...hcpm8K2`, `depth: 2` | exit 64, stdout empty |
| `fd_i18_derive_refuses_empty_path` | `mk derive --in acct.mk1 --path '' --json`; same with `m`, `m/`; text mode with `''` | exit 0, `"child_xpub":"xpub6BmeGm...Cp2z6a"` (= parent), `"relative_path":"m/"`, `"depth":3` | exit 64; `--json`: envelope `"kind":"UsageError"`, no `child_xpub`; text: stderr = the 2.2 text |
| `fd_c1_master_card_still_derives` (control) | `mk address --in master.mk1 --address-type p2wpkh --count 1`; `mk derive --in master.mk1 --index 0 --json` | exit 0, `bc1qp5wfcq48h6d63wyy9qz0awtpfqwwv4sma86mhz` + depth advisory; child `xpub6AvUGrnEpfvJ...EuMWwnj`, depth 2 | unchanged |
| `fd_i18_nonempty_path_still_derives` (control) | `mk derive --in acct.mk1 --path m/0/0 --json` | exit 0, child `xpub6Fec2dLaQAfW...Qv8oYaf`, depth 5 | unchanged |

The builder also adds these rows as nextest tests (test fns named exactly as the first column) in a
new `crates/mk-cli/tests/bh1119_derive_guards.rs`, RED first, plus the `is_fixed_key_card` unit test.
The rows script stays as the CLI-level check; green asserts only exit/stdout for C1 until section
8 fixes the text, then the builder adds the text check to both the script and the tests.

## 6. Tests that change

None of the existing tests change: no mk-cli test runs `mk address`/`mk derive` on a depth-0
zero-chain-code card or with an empty `--path` (grep of `crates/mk-cli/tests` at 8a738ab for the
LEAF tpub, `[0u8; 32]`, `"--path", ""`: no hits). `cli_address.rs::leaf_card_requires_then_advises_depth`
and `nonstandard_requires_address_type_then_succeeds` keep passing (not fixed-key cards). The
builder confirms with the full suite.

## 7. Toolkit manual lockstep (`docs/manual/src/40-cli-reference/44-mk-cli.md`, master)

- `## mk address` intro paragraph (L486-492), after the sentence ending "...relative to the card's
  xpub)." insert (PROPOSED; the last clause follows section 8):
  > A card that holds one fixed key (depth 0 with an all-zero chain code — how the toolkit writes a WIF key) is **refused**: it has no child keys that any wallet watches. [Option A: The refusal prints the key's own address for the `--address-type` and network given.] [Option B: The refusal prints the `mnemonic convert` command that renders the key's own address.]
- `## mk derive` intro paragraph (L541-544), after "hardened components are rejected." insert
  (PROPOSED):
  > A `--path` with no steps (`''`, `m`, `m/`) is rejected (the card's own key is `mk decode`'s output), and a fixed-key card (see `mk address`) is refused.
- Flags table row `--path <REL>` (L559): "relative derivation path, unhardened only, at least one
  step (e.g. `m/0/5`)" (PROPOSED).
- No `.out` include file changes (worked examples use an account card).

## 8. STOP: how the C1 refusal points at the key's own address

B3 decides refuse and "pointing at the key's own address" but not the mechanism. The two
readings fix different check orders and texts; I do not choose.

- **Option A: print the address in the refusal.** `mk address` resolves address type (the empty
  origin requires `--address-type`, so without it the existing "--address-type required" exit 64
  fires first) and network as today, then refuses with the key's own address (the card's pubkey
  rendered by `render_address` for that type and network; for p2tr the BIP-86 key-path tweak with
  no script tree, as `render_address` already does). `mk derive` has no address type, so it points
  at `mk address`. PROPOSED texts:
  - address: `this card holds one fixed key (depth 0, all-zero chain code; the toolkit writes a WIF key this way) and has no child keys any wallet watches, so mk derives no addresses from it. The key's own {type} address on {network} is {address}`
  - derive: `this card holds one fixed key (depth 0, all-zero chain code; the toolkit writes a WIF key this way) and has no child keys any wallet watches, so mk derives no child xpub from it. To see the key's own address, run \`mk address\` on the card with \`--address-type\``
  Consequence: an address appears on stderr at exit 64; `--count`/`--range`/`--chain` are
  ignored; `--json` carries the address only inside the error message.
- **Option B: name a command that prints it.** Both commands refuse right after the chunk-set-id
  warning (before address type, network and path), and print the toolkit command that renders
  the key's own address from public data. Verified at toolkit 0.106.0:
  `mnemonic convert --from xpub=tpubD6NzVbkrYhZ4WLczPJWReQycCJdd6YVWXubbVUFnJ5KgU5MDQrD998ZJLT5hqNKRA9cJzkh4D7oXwCpwFUWWWkHR9zPeBk1hnWBJKNhyYsn --to address --path m --script-type p2wpkh --network signet`
  -> `address: tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx`, exit 0. PROPOSED text (both commands):
  - `this card holds one fixed key (depth 0, all-zero chain code; the toolkit writes a WIF key this way) and has no child keys any wallet watches, so mk derives nothing from it. The key's own address: mnemonic convert --from xpub={xpub} --to address --path m --script-type <TYPE>` followed, for a test-kind xpub, by ` --network <testnet|signet|regtest>`.
  Consequence: mk's text names a toolkit command (precedent: the multisig refusal names
  `md address`); the holder picks the script type.

Question for the architect/Brian: A or B (or another shape)? The answer fixes §3.1 step 0's
order, the section 2.1/8 texts and the manual sentence; no other part of this document moves.

## 9. FOLLOWUPS (listed in the author's report, not appended here)

Lines for `design/FOLLOWUPS.md` (to be direct-pushed after merge, per the coordinator rule) are in
the author's report.

## 10. Gate

`scripts/mk-fd-derive-guards-rows.sh green` -> `ALL ROWS MATCH green`, then the full CI set:
`cargo nextest run --locked --workspace --no-fail-fast`, `cargo test --locked --doc --workspace`,
`cargo +1.95.0 fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`.
