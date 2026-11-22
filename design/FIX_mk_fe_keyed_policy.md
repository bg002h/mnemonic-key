# FIX_mk_fe_keyed_policy: bughunt-1119 F-E, `mk encode` keyed-policy membership and repeated input (bh-mk-I2, bh-mk-I15, sweep-1122 row 15)

Small-fix track, grade I (all keys). Not class B (no address output; the defect is a wrong mk1
card). Repo mnemonic-key, branch `fix/mk-fe-keyed-policy`, base `origin/main` 200e8618 ("Merge pull
request #29 from bg002h/build/mkc17"). Toolkit manual
`docs/manual/src/40-cli-reference/44-mk-cli.md` changes in lockstep (separate toolkit change).
Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22.

Inputs (decided; not re-ruled here): coordinator `agent-reports/architect-bughunt-1119-questions.md`
row #18 and P2; Brian's ruling B2 (`memory/product-rulings-sweep-bughunt-2026-11-22.md`: "refuse
contradictory or repeated key records, exit 64"); sweep architect MK-Q1 + MK-Q2
(`agent-reports/architect-sweep-1122-questions.md` L101-106: refuse, exit 64, "one origin with two
xpubs, one xpub with two origins, an identical record given twice, and a repeated
`--from-md1`/`--policy-id-stub`"; mk-dup-stub rides this fix); grader
`agent-reports/bughunt-1119/grade-mk.md` (I2, I15, F-E) and `agent-reports/sweep-1122/grade-mk.md`
(row 15 notes 1 and 3); triage `triage-mk-t1.md` (I2), `triage-mk-t4.md` (I15). BIP-388 L193 (the
public keys of the key information vector "must be pairwise distinct"), checked in the coordinator's
`scripts/factcheck/sources/bip-0388.mediawiki`.

Every new printed text below is PROPOSED (architect confirms). No new flag, type or command.

## 1. Defects (at 200e8618, rows in section 5 run and seen)

- **bh-mk-I2.** `mk encode` against a KEYED wallet-policy md1 checks only that the xpub's 65-byte
  identity is one of the policy's keys (encode.rs L321-338). The origin the card declares is never
  compared with the origin the policy gives that key, so a card whose origin contradicts its own
  policy mints at exit 0: `X0` at `m/48'/0'/9'/2'` (policy: `m/48'/0'/0'/2'`), `X0` with fingerprint
  `deadbeef` (policy: `73c5da0a`), and `X1` at `[73c5da0a/48'/0'/1'/2']` (policy @1:
  `[aabbccdd/48'/0'/0'/2']`). A sparse keyed policy is not checked at all, though the slot that
  holds the key states its origin. A policy that holds the card's key in two slots (BIP-388 L193
  forbids it; md-codec 0.42 decodes it) mints too.
- **Sweep row 15 (mk-keys-contradictory-records).** `read_key_records` (keyfile.rs L137-152) parses
  each line alone; nothing compares records. One origin with two keys, one key with two origins,
  and the same record twice each mint N cards at exit 0.
- **mk-dup-stub (M, rides this fix).** A stub bound twice (`--policy-id-stub deadbeef` twice, the
  same single-string `--from-md1` twice, or `--policy-id-stub 559e64b2` with the `--from-md1` whose
  stub is 559e64b2) mints a card listing one policy twice, exit 0 (encode.rs L175-213 never
  compares stubs).
- **bh-mk-I15.** `keys_batch::membership_is_enforced_only_when_every_slot_is_keyed`
  (keys_batch.rs L659-734) needs `/scratch/code/shibboleth/descriptor-mnemonic/target/release/md`;
  without it (every CI host, this host) it prints `skipping: md binary not built` and PASSes. With
  `short_coverage_of_a_keyed_policy_is_noted_not_refused` (stub-only, never keyed) the membership
  refusal, the coverage note and the sparse rule have no test that can fail.

## 2. Decided behaviour

All refusals: exit 64 (`CliError::UsageError`), nothing on stdout in text mode, stderr exactly
`error: ` + the text + newline (no watch-only note: the run stops before minting); under `--json`
the usual envelope on stdout,
`{"error":{"details":null,"exit_code":64,"kind":"UsageError","message":"<text>"},"schema_version":1}`,
stderr empty. No card is written, `--out` is not touched.

Order in `mk encode` (existing steps unchanged, new steps in bold):
1. flag exclusivity; 2. `--policy-id-stub` parse; 3. md1 decode per card; 4. "at least one of
`--policy-id-stub` or `--from-md1` is required"; 5. **repeated stub (2.3)**; 6. single-card
fingerprint flags; 7. key records: per-line parse errors (unchanged), then **record set checks
(2.2)**, then "no key records found"; 8. depth-0 parent fingerprint and depth-0/1 origin fingerprint
checks (unchanged); 9. **per card (record order), per keyed policy (stub order): repeated key (2.1c),
origin path (2.1a), master fingerprint (2.1b)**, then the existing "is not a cosigner" refusal;
10. coverage note; mint.

So a key file whose records repeat one another reports 2.2 even when a record would also fail step 8
(row `fe_keys_one_origin_two_masters`), and a repeated stub is reported before any key file is read.

### 2.1 Card origin against its own keyed policy (bh-mk-I2)

For every `--from-md1`/`--from-md1-set` card that is a keyed wallet policy, its keyed slots are the
`md_codec::canonicalize::expand_per_at_n(&descriptor)` entries whose `xpub` is `Some` (origin path,
optional master fingerprint, key, `@N`). A slot's origin path is its `ExpandedKey::origin_path`,
except that an empty one (an elided origin: what `md encode` writes without `--path`, with or
without `--network signet`) is `md_codec::canonical_origin::canonical_origin(&descriptor.tree)`, the
path `compute_wallet_policy_id` hashes for it (identity.rs L230-235), so the stub commits to it; such
a slot is *standard-filled*. (`canonical_origin` is `None` only for a wrapper whose empty origin
`expand_per_at_n` already refused with `MissingExplicitOrigin`; if reached, keep the empty path, as
identity.rs does.) This is the expansion `compute_wallet_policy_id` already
runs on the same descriptor, so it adds no new failure (map any error with `?`, unreachable for a
card whose policy id was computed). A keyless template has no keyed slots and nothing is checked
(unchanged); `--policy-id-stub` is not checked (unchanged). The check runs for a complete AND a
sparse key list: a slot that holds the card's key states that key's origin even when other slots are
empty. Key equality is the existing 65-byte identity (`xpub_identity_65`).

For each card and each keyed policy, let S = the keyed slots holding the card's key:
- (c) |S| >= 2: refuse with T-REPKEY. `<slots>` = every `@N` in S, ascending, joined by `, `.
- |S| = 1, slot `@N`:
  - (a) card path != slot origin path (component-wise: index and hardened flag; a standard-filled
    slot compares its filled path, so an empty card path does not match it): refuse with T-PATH, or
    with T-PATH-STD when the slot is standard-filled.
  - (b) else, if BOTH the slot fingerprint and the card fingerprint are present and differ: refuse
    with T-FP. A missing fingerprint on either side is not compared (P2: "Missing fingerprints stay
    accepted"): a `--privacy-preserving` card and a policy without a fingerprint TLV both mint.
- |S| = 0: unchanged (complete key list: the existing "is not a cosigner" refusal; sparse: mints).

Path rendering in T-PATH and T-PATH-STD: `m`, then `/<index>` per component with `'` after a hardened index
(`m/48'/0'/0'/2'`; an empty path renders `m`). Fingerprints: 8 lowercase hex.

- T-PATH: `xpub <xpub> is @<N> of the wallet policy with stub <stub>, whose origin path for it is <policy path>; the card declares <card path>. A card whose origin contradicts its own policy is refused; no card was minted.`
- T-PATH-STD: `xpub <xpub> is @<N> of the wallet policy with stub <stub>, which states no origin path for it, so its policy id uses the standard origin path for its template, <policy path>; the card declares <card path>. A card whose origin contradicts its own policy is refused; no card was minted. If <card path> is this key's origin, re-encode the md1 with the keys' real origins (md encode --path, or the origin in the template) and mint against the new md1.`
- T-FP: `xpub <xpub> is @<N> of the wallet policy with stub <stub>, whose master fingerprint for it is <policy fp>; the card declares <card fp>. A card whose origin contradicts its own policy is refused; no card was minted.`
- T-REPKEY: `the wallet policy with stub <stub> holds xpub <xpub> at <slots>; BIP 388 requires the keys of a policy to be pairwise distinct, so the policy is refused and no card was minted.`

`<xpub>` is the card's xpub as the existing "is not a cosigner" text prints it (`{xpub}` Display).
T-REPKEY fires only for the card's own key; a policy that repeats some other key is not examined
here (mk certifies its own key's membership).

### 2.2 Key-record set checks (sweep row 15; `--keys` and `--in`, file or `-`)

After every line has parsed (a parse error still wins, unchanged), compare every pair of records:
for j = 2nd..last record in file order, for i = 1st..(j-1), the first pair that matches below is
refused, at record j's line. Line numbers are 1-based file lines, counting blank and comment lines,
as the parse errors already do; `<source>` is the path as given, or `<stdin>`. Same key = equal
`xpub_identity_65` (so a SLIP-132 or depth-byte re-spelling is the same key); same origin = equal
fingerprint and equal parsed path (so `48h` and `48'` are the same origin).

- same key and same origin: T-REC.
- same key, different origin: T-KEY2, `<origin>` = record i's origin.
- same origin, different key: T-ORIGIN2, `<origin>` = the shared origin.

Origin rendering in T-KEY2/T-ORIGIN2: BIP-380 notation as the file takes it, `[<fp>/<path>]` with
the path as `DerivationPath` Display (`[73c5da0a/48'/0'/0'/2']`; an empty path renders
`[3442193e/]`).

- T-REC: `<source>:<j>: this record repeats line <i>; a key is carded once, so a repeated record is refused and no card was minted.`
- T-KEY2: `<source>:<j>: this record's key is also on line <i>, with origin <origin>; one key has one origin, so the two records contradict each other and no card was minted.`
- T-ORIGIN2: `<source>:<j>: this record's origin <origin> is also on line <i>, with a different key; one origin derives one key (BIP 32), so the two records contradict each other and no card was minted.`

Records that differ in both key and origin are untouched (same master at two paths; two masters at
one path). The F-311 per-card stderr labels stay: two or more distinct cards still get them.

### 2.3 Repeated stub (mk-dup-stub)

After the stub list is assembled (step 4; order `--policy-id-stub`, `--from-md1`, `--from-md1-set`,
unchanged), refuse at the first stub equal to an earlier one, whichever channels supplied the two:

- T-STUB: `policy stub <stub> is bound twice; a card lists each policy once, so a stub repeated across --policy-id-stub, --from-md1 and --from-md1-set is refused and no card was minted.`

Unchanged: a chunked md1 set given twice is still refused by md-codec before this point (exit 1,
`md1 input rejected: chunk set incomplete: got 8 chunks, expected 4`; row
`fe_control_chunk_set_given_twice_unchanged`).

### 2.4 Help text

No clap help string changes (so `mk gui-schema` and the GUI mirror are unchanged). The refusals are
documented in the manual (section 7) and the CHANGELOG.

## 3. Spec delta

No `design/SPEC_*.md` specifies `mk encode`'s membership or key-record checks (grep of design/SPEC*
for "cosigner", "membership", "F-311": only SPEC_mk_v0_1.md L225, about the two stub forms, which
stays true). This document is the spec record. Doc-comment updates (not printed): keyfile.rs module
doc L22-30 adds that records are now compared pairwise (2.2) while a single record pairing a real
xpub with another key's same-depth origin is still accepted when no policy states that key's
origin; encode.rs L318-320 comment adds the origin check and the sparse rule; the F-311 comment at
encode.rs L456-459 and keys_batch.rs L146-150 drop "the same record twice ... accepted" (now
refused).

## 4. Code (anchors at 200e8618; the builder writes the code)

- `crates/mk-cli/src/cmd/mod.rs` L316-365: `Md1Card` gains the keyed slots of 2.1 (built in
  `decode_md1_card` from `expand_per_at_n` when `descriptor.is_wallet_policy()`); `cosigners` keeps
  its meaning (complete list or `None`).
- `crates/mk-cli/src/cmd/encode.rs`: 2.3 after L209-213; 2.1 inside the L321-338 loop before the
  `cosigners` test (the loop binds the card fingerprint, today `_fp`).
- `crates/mk-cli/src/keyfile.rs` L137-152: keep each record's 1-based line number; 2.2 after the
  loop.
- `crates/mk-cli/CHANGELOG.md` `[Unreleased]` `### Changed` (PROPOSED):
  `- \`mk encode\` refuses (exit 64) a card whose origin path or master fingerprint contradicts the slot its keyed --from-md1 policy gives that key (complete or sparse key list; a slot that states no origin gives the template's standard origin, which the policy id commits to), and a policy that holds the key in two slots.`
  `- \`mk encode --keys\`/\`--in\` refuses (exit 64) a repeated record, one key under two origins, and one origin with two keys, naming both lines.`
  `- \`mk encode\` refuses (exit 64) a policy stub bound twice across --policy-id-stub, --from-md1 and --from-md1-set.`
- `SCRIPTS.md`: row for `scripts/mk-fe-keyed-policy-rows.sh` (added on this branch; PR #30 adds its
  own row at the same place, a trivial merge).

## 5. Rows

Committed runner: `scripts/mk-fe-keyed-policy-rows.sh red|green` (builds `mk` with
`cargo build --locked -p mk-cli` on the 1.85 pin, writes the fixtures below under
`$REPO/target/mk-fe-rows` (override with `WORK`), runs every row, checks the mode's expectation; a
`=` check compares the whole stream; its header says what it does not cover). Runs: `red` at the
base code -> `ALL ROWS MATCH red` (29 rows); `green` with a scratch implementation of section 2
(reverted, not committed) -> `ALL ROWS MATCH green` (29 rows), and the full workspace suite under
that scratch: 460 run, 459 passed, 1 failed (`single_print::encode_keys_labels_each_card_on_stderr`,
section 6). The I-1 rows (architect-mk18-r1-i1) were added after those runs: the builder re-measures red, green and the suite counts.

Fixtures (public keys only; full strings in the script): X0, X1, X2 = keys_batch.rs `KEYS` xpubs;
TV1M = BIP-32 TV1 master (fp 3442193e); LEAFM = generator point, zero chain code, depth 0.
Policies `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))`, shared origin `m/48'/0'/0'/2'`:
`a.md1` = from_md1_set.rs `KEYED_POLICY_A` (stub 38bd7cec; @0 `[73c5da0a]X0`, @1 `[aabbccdd]X1`);
`sparse.md1` (stub 3a7f6520; @0 `[73c5da0a]X0`, @1 unkeyed; `md encode` 0.20.7 output);
`rep.md1` (stub 58e73a11; A with @1's key set to X0, built with md_codec 0.42 reassemble/split);
`nofp.md1` (A without the fingerprint TLV); `TPL` = `md1yqpqqxzq2qwfv8urt848e` (keyless `wpkh`,
stub 559e64b2).
`elided.md1` (stub 38bd7cec, the same policy as A with the origin elided on the wire: `md encode
'wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))' --key @0=X0 --key @1=X1 --fingerprint @0=73c5da0a
--fingerprint @1=aabbccdd`, md 0.20.7, no `--path`; `--network signet` with the tpub spellings
prints the same strings); `signet.md1` (stub 00912ee3; the same command plus
`--path "m/48'/1'/0'/2'"`, byte-identical to `md decompose --emit commands` route 1's
inline-origin encode); X0D0 = X0's chain code and key re-serialized at depth 0 (parent fp
00000000, child 0), `xpub661MyMwAqRbcGQnC8zMGwRc4EXYJCgXrxx9kXtw1RXqu4TcW26PxHqssgp6sU4N5CR5o9QcUZPG31fzPeUHEVPkFit1WpVZTmqZLzpvZG2s`.
Key files: `twice.txt` (`[73c5da0a/48'/0'/0'/2']X0` twice), `twice_h.txt` (line 1
that record, line 2 a comment, line 3 the `48h/0h/0h/2h` spelling), `twoorigins.txt` (X0 at
`48'/0'/0'/2'` then `48'/0'/1'/2'`), `twokeys.txt` (X0 line 1, blank line 2, X1 line 3, both
`[73c5da0a/48'/0'/0'/2']`), `twomasters.txt` (`[3442193e/]TV1M`, `[3442193e/]LEAFM`),
`batch9.txt` (`[73c5da0a/48'/0'/9'/2']X0`), `distinct.txt` (KEYS, three records),
`cosigners.txt` (`[73c5da0a/48'/0'/0'/2']X0`, `[aabbccdd/48'/0'/0'/2']X1`).

GREEN refusal rows: exit 64, stdout empty, stderr exactly `error: ` + the text named.

| row (test fn) | command (`mk encode ...`) | RED at 200e8618 (seen) | GREEN |
|---|---|---|---|
| `fe_i2_path_contradicts_policy` | `--xpub X0 --origin-path "m/48'/0'/9'/2'" --origin-fingerprint 73c5da0a --from-md1-set a.md1` | exit 0, `mk1qpw9gezqqsqn30tua3eutks2lcztpqyqsq...`, coverage note | T-PATH(X0, 0, 38bd7cec, `m/48'/0'/0'/2'`, `m/48'/0'/9'/2'`) |
| `fe_i2_path_contradicts_policy_json` | the row above + `--json` | exit 0, stdout JSON card (`"mk1_strings"`) | exit 64, stderr empty, stdout exactly `{"error":{"details":null,"exit_code":64,"kind":"UsageError","message":"` T-PATH(as above) `"},"schema_version":1}` |
| `fe_i2_fingerprint_contradicts_policy` | `--xpub X0 --origin-path "m/48'/0'/0'/2'" --origin-fingerprint deadbeef --from-md1-set a.md1` | exit 0, `mk1qp35nupqqsqn30tuan02m0h0q5zg3vs7...` | T-FP(X0, 0, 38bd7cec, 73c5da0a, deadbeef) |
| `fe_i2_second_slot_contradicts_policy` | `--xpub X1 --origin-path "m/48'/0'/1'/2'" --origin-fingerprint 73c5da0a --from-md1-set a.md1` | exit 0, `mk1qphtczzqqsqn30tua3eutks2lcztpqyq...` | T-PATH(X1, 1, 38bd7cec, `m/48'/0'/0'/2'`, `m/48'/0'/1'/2'`) (path is checked before fingerprint) |
| `fe_i2_sparse_slot_contradicts_policy` | `--xpub X0 --origin-path "m/48'/0'/9'/2'" --origin-fingerprint 73c5da0a --from-md1-set sparse.md1` | exit 0, `mk1qpm2syzqqsqn5lm9ypeutks2lcztpq...`, no note | T-PATH(X0, 0, 3a7f6520, `m/48'/0'/0'/2'`, `m/48'/0'/9'/2'`) |
| `fe_i2_batch_record_contradicts_policy` | `--keys batch9.txt --from-md1-set a.md1` | exit 0, mk1 strings | T-PATH(X0, 0, 38bd7cec, `m/48'/0'/0'/2'`, `m/48'/0'/9'/2'`) |
| `fe_i2_policy_repeats_key` | `--xpub X0 --origin-path "m/48'/0'/0'/2'" --origin-fingerprint 73c5da0a --from-md1-set rep.md1` | exit 0, `mk1qpv636pqqsq43ee6z9eutks2q5zg3v...`, no note | T-REPKEY(58e73a11, X0, `@0, @1`) |
| `fe_i2_elided_origin_path_contradicts_policy` | `--xpub X0 --origin-path "m/48'/0'/9'/2'" --origin-fingerprint 73c5da0a --from-md1-set elided.md1` | exit 0, `mk1qpw9gezqqsqn30tua3eutks2lcztpqyqsq...`, coverage note | T-PATH-STD(X0, 0, 38bd7cec, `m/48'/0'/0'/2'`, `m/48'/0'/9'/2'`) |
| `fe_i2_elided_origin_signet_card` | `--xpub X0 --origin-path "m/48'/1'/0'/2'" --origin-fingerprint 73c5da0a --from-md1-set elided.md1` | exit 0, `mk1qp5864pqqsqn30tua3eutks2z5zg3vs7...`, coverage note | T-PATH-STD(X0, 0, 38bd7cec, `m/48'/0'/0'/2'`, `m/48'/1'/0'/2'`) |
| `fe_i2_elided_origin_master_card` | `--xpub X0D0 --origin-path "m" --privacy-preserving --from-md1-set elided.md1` | exit 0, `mk1qp0fdvpqqqqn30tuanlqqpygkg0qqqq...`, coverage note | T-PATH-STD(X0D0, 0, 38bd7cec, `m/48'/0'/0'/2'`, `m`) (an empty card path does not match a standard-filled slot) |
| `fe_keys_identical_record_twice` | `--keys twice.txt --policy-id-stub deadbeef` | exit 0, two identical cards, `card 2 of 2` label | T-REC(twice.txt, 2, 1) |
| `fe_keys_identical_record_h_spelling` | `--keys twice_h.txt --policy-id-stub deadbeef` | exit 0, `card 2 of 2` | T-REC(twice_h.txt, 3, 1) |
| `fe_keys_one_key_two_origins` | `--keys twoorigins.txt --policy-id-stub deadbeef` | exit 0, `card 2 of 2` | T-KEY2(twoorigins.txt, 2, 1, `[73c5da0a/48'/0'/0'/2']`) |
| `fe_keys_one_origin_two_keys` | `--keys twokeys.txt --policy-id-stub deadbeef` | exit 0, `card 2 of 2` | T-ORIGIN2(twokeys.txt, 3, `[73c5da0a/48'/0'/0'/2']`, 1) |
| `fe_keys_one_origin_two_masters` (order) | `--keys twomasters.txt --policy-id-stub deadbeef` | exit 64, `origin fingerprint 3442193e does not match the xpub: ...` (step 8) | T-ORIGIN2(twomasters.txt, 2, `[3442193e/]`, 1) (2.2 precedes step 8) |
| `fe_stub_repeated_policy_id_stub` | `--xpub X0 --origin-path "m/48'/0'/0'/2'" --origin-fingerprint 73c5da0a --policy-id-stub deadbeef --policy-id-stub deadbeef` | exit 0, card | T-STUB(deadbeef) |
| `fe_stub_repeated_from_md1` | same key flags, `--from-md1 TPL --from-md1 TPL` | exit 0, card (stub list `559e64b2,559e64b2`) | T-STUB(559e64b2) |
| `fe_stub_repeated_across_channels` | same key flags, `--policy-id-stub 559e64b2 --from-md1 TPL` | exit 0, card | T-STUB(559e64b2) |
| `fe_control_true_origin_mints` | `--xpub X0 --origin-path "m/48'/0'/0'/2'" --origin-fingerprint 73c5da0a --from-md1-set a.md1` | exit 0, note `1 of them carded here, 1 not carded` | unchanged |
| `fe_control_privacy_preserving_mints` | `--xpub X0 --origin-path "m/48'/0'/0'/2'" --privacy-preserving --from-md1-set a.md1` | exit 0 | unchanged |
| `fe_control_policy_without_fingerprints_mints` | `--xpub X0 --origin-path "m/48'/0'/0'/2'" --origin-fingerprint deadbeef --from-md1-set nofp.md1` | exit 0 | unchanged |
| `fe_control_sparse_unfilled_slot_mints` | `--xpub X1 --origin-path "m/48'/0'/1'/2'" --origin-fingerprint 73c5da0a --from-md1-set sparse.md1` | exit 0 | unchanged |
| `fe_control_complete_policy_nonmember_refused` | `--xpub X2 --origin-path "m/48'/0'/2'/2'" --origin-fingerprint 73c5da0a --from-md1-set a.md1` | exit 64, `... is not a cosigner of the wallet policy with stub 38bd7cec ...` | unchanged |
| `fe_control_distinct_records_mint` | `--keys distinct.txt --policy-id-stub deadbeef` | exit 0, `card 3 of 3` | unchanged |
| `fe_control_both_cosigners_mint` | `--keys cosigners.txt --from-md1-set a.md1` | exit 0, `card 2 of 2`, no coverage note | unchanged |
| `fe_control_two_distinct_stubs_mint` | key flags, `--policy-id-stub deadbeef --from-md1 TPL` | exit 0 | unchanged |
| `fe_control_chunk_set_given_twice_unchanged` | key flags, `--from-md1-set a.md1 --from-md1-set a.md1` | exit 1, stdout empty, `chunk set incomplete: got 8 chunks, expected 4` | unchanged |
| `fe_control_md_default_elided_origin_mints` | `--xpub X0 --origin-path "m/48'/0'/0'/2'" --origin-fingerprint 73c5da0a --from-md1-set elided.md1` | exit 0, note `policy 38bd7cec has 2 cosigner(s); 1 of them carded here, 1 not carded` | unchanged |
| `fe_control_signet_card_against_path_md1_mints` | `--xpub X0 --origin-path "m/48'/1'/0'/2'" --origin-fingerprint 73c5da0a --from-md1-set signet.md1` | exit 0, note `policy 00912ee3 has 2 cosigner(s); 1 of them carded here, 1 not carded` | unchanged |

GREEN stderr of `fe_i2_elided_origin_path_contradicts_policy`, in full (with X0 written as its literal):
`error: xpub X0 is @0 of the wallet policy with stub 38bd7cec, which states no origin path for it, so its policy id uses the standard origin path for its template, m/48'/0'/0'/2'; the card declares m/48'/0'/9'/2'. A card whose origin contradicts its own policy is refused; no card was minted. If m/48'/0'/9'/2' is this key's origin, re-encode the md1 with the keys' real origins (md encode --path, or the origin in the template) and mint against the new md1.`
For the X0D0 row, `<xpub>` is the card's xpub as Display prints it. The builder pins the measured string.

The builder adds these rows as nextest tests in a new `crates/mk-cli/tests/bh1119_keyed_policy.rs`
(test fns named exactly as the first column, asserting the same exact texts, RED first), plus:
- `fe_keys_in_flag_and_stdin_same_checks`: `--in -` with `twice.txt` on stdin: T-REC with source
  `<stdin>`.
- I15, built in-process with `md_codec` (no sibling binary; fixtures `KEYED_POLICY_A` and a sparse
  copy made by `reassemble`, dropping @1's key, `split`), the three tests of `triage-mk-t4.md`
  L26-96 under these names: `fe_i15_short_coverage_note_is_printed`,
  `fe_i15_nonmember_of_complete_policy_is_refused`, `fe_i15_sparse_policy_admits_unfilled_slot_key`.
  RED for these is a plant (re-run at 200e8618 with the triage tests; each plant reverted):
  P1 `if missing > 0` -> `if false && missing > 0` (encode.rs L359) fails only the coverage test;
  P2 `if !cosigners.contains(&ident)` -> `if false && ...` (L327) fails only the non-member test;
  P3 the `.filter(|v| v.len() == descriptor.n as usize)` (mod.rs L361) -> `.filter(|_v| true)`
  fails only the sparse test; the two existing tests PASS under all three plants. The builder
  repeats the three plants on the new file and reports them.

## 6. Tests that change

- `crates/mk-cli/tests/single_print.rs::encode_keys_labels_each_card_on_stderr` (L93-122): it feeds
  the same record twice (now T-REC, exit 64). New input: `[73c5da0a/48'/0'/0'/2']X0` and
  `[73c5da0a/48'/0'/1'/2']X1` with `--policy-id-stub deadbeef`; asserts exit 0, 5 stdout lines,
  stderr exactly `card 1 of 2: lines 1-2\ncard 2 of 2: lines 3-5\n` + NOTE (measured at base). Its
  comment drops "F-311's duplicate".
- `crates/mk-cli/tests/keys_batch.rs::membership_is_enforced_only_when_every_slot_is_keyed`
  (L659-734, with its doc comment): deleted; replaced by `fe_i15_sparse_policy_admits_unfilled_slot_key` and
  `fe_control_sparse_unfilled_slot_mints`. The F-311 doc comment at L146-150 is reworded (a repeated
  record is now refused; the labels remain the card boundary for distinct cards).
- No other test changes: the full workspace suite under the scratch implementation failed only the
  single_print test above (460 run, 459 passed).

## 7. Toolkit manual lockstep (`docs/manual/src/40-cli-reference/44-mk-cli.md`, master 600c1293)

After the paragraph "At least one of `--policy-id-stub` or `--from-md1` is required (the underlying
`KeyCard.policy_id_stubs` slot must be non-empty)." (L51-52) insert (PROPOSED):

> `mk encode` refuses (exit 64, no card minted) input that contradicts itself: a key file with a
> repeated record, with one key under two origins, or with one origin holding two keys (the error
> names both lines); a policy stub bound twice across `--policy-id-stub`, `--from-md1` and
> `--from-md1-set`; and, against a keyed `--from-md1` policy, a card whose origin path or master
> fingerprint differs from the one the policy gives that key, or a policy that holds the key in two
> slots. A policy that states no origin for a key (as `md encode` writes it without `--path`, for
> any network) gives the template's standard origin, `m/48'/0'/0'/2'` for `wsh(multi(...))`; a
> card at another path needs the md1 re-encoded with the real origin. A missing fingerprint on
> either side is not compared. A key the policy does not list is
> refused only when every slot of the policy is keyed.

No flag-table row changes; no `.out` include file changes (the worked example is a single
`--policy-id-stub` card).

## 8. Rulings (closed)

B2 / P2 (refuse, exit 64, missing fingerprints accepted) and MK-Q1 + MK-Q2 (the four repeated-input
shapes, refusal over de-duplication). Decided here within those rulings, flagged for the review
round: the origin check covers sparse key lists (2.1); a policy holding the card's key in two slots
is refused (2.1c, BIP-388 L193); a stub repeated across channels counts as repeated (2.3); a chunk
set given twice keeps its existing exit-1 refusal (2.3).
An elided slot origin is standard-filled before (a), compared strictly, and refused with
T-PATH-STD (architect-mk18-r1-i1, coordinator agent-reports/architect-mk18-r1-i1.md).

## 9. FOLLOWUPS (listed in the author's report, not appended here)

## 10. Gate

`scripts/mk-fe-keyed-policy-rows.sh green` -> `ALL ROWS MATCH green`, then the full CI set:
`cargo nextest run --locked --workspace --no-fail-fast`, `cargo test --locked --doc --workspace`,
`cargo +1.95.0 fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`,
and the three I15 plants of section 5 (each fails exactly its test).
