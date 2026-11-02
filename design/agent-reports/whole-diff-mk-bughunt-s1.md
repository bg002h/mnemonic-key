# Whole-diff review s1 (design and threat, executed): mk bug-hunt fix

Reviewer: Opus, slice s1. Worktree mk `fix/bughunt` 78f5ded5 vs c501dbe;
toolkit `fix/bughunt-mk-manual` 0637fca9 on f6355e22.
Not a re-review (first whole-diff round); 0 findings come from a prior fix commit.

## Findings (appended as found)


All probes ran `target/debug/mk` (mk 0.14.0-dev, head 78f5ded) with signet/testnet material only (tpub `V15_84_TEST`, fp 73c5da0a, `m/84'/1'/0'`, stub deadbeef). Outputs are in `/scratch/cache/bughunt/mk/s1/`.

### Executed probes: all match FIX §1/§2

Fix 01 (encode fingerprint refusal):
- Single card with neither flag: exit 64, stdout empty, stderr exactly `error: pass --origin-fingerprint <hex> or --privacy-preserving explicitly`. With `--json`: a JSON error object with `exit_code` 64 and `UsageError`. With `--out f.out`: exit 64 and no file is created.
- Following the advice literally: `--origin-fingerprint 73c5da0a` exits 0 and decodes back to fp 73c5da0a, path 84'/1'/0', stub deadbeef. `--privacy-preserving` exits 0. The advice cannot produce a wrong card.
- `--keys F`, `--in F` and `--keys -` (stdin) with `[73c5da0a/84'/1'/0']tpub…` and no flag: each exits 0. They produce the same card as the single-card `--origin-fingerprint 73c5da0a` encode, byte for byte. So no valid holder input is refused.
- Ordering: a missing stub still reports `at least one of --policy-id-stub or --from-md1 is required` first. `--keys` with either flag still gives the old mutual-exclusion texts.
- The fingerprint never comes from `--from-md1` (encode.rs:214-224), so the new arm cannot refuse an md1-sourced fingerprint.
- Cross-repo: every `mk encode` line that mr prints carries one of the two flags. The mr manual lines at 45-mr.md:482 and :4112 carry `--origin-fingerprint FP`. The mk-cli worked example at 44-mk-cli.md:61 carries `--origin-fingerprint 73c5da0a`.

Fix 05 (codex32 ASCII-only):
- An all-uppercase card, ungrouped or grouped via `--in`, decodes to output identical to the lowercase card. `mk address`, `mk repair` (a 1-error uppercase card corrects to the lowercase card), `mk verify` and `mk inspect` all accept it. Mixed case is still refused.
- U+212A in the data part at index 21, lowercase or uppercase: exit 1, `invalid character K at position 21`.
- U+212A in the HRP, `mK1…` or `MK1…`: exit 1, `invalid HRP: mK`. This is the ASCII-lowercased HRP, as S5 rules.
- An invalid ASCII `b` at data index 0 followed by `é` at index 6: `invalid character é at position 6`. The first non-ASCII character is reported, as the architect ruled.
- `mk repair` and `mk address` on the Kelvin card fail closed with the same text.

Fix 03 (address cap):
- Exact FIX texts, exit 64, for `--count 1000001`, `--count 2147483648`, `--range 0,1000000`, `--range 0,2147483647` and `--range 1147483648,2147483647`. The same holds with `--chain both` and `--json`.
- `--count 2147483649` hits the ceiling text first.
- `--range 2147483647,2147483648` and `--range 5,4` keep their old texts.
- Accepted, exit 0, exactly 1,000,000 lines each, about 31 s, run in parallel:
  - `--count 1000000` covers indices 0..999999.
  - `--range 5,1000004` covers indices 5..1000004.
  - `--range 2146483648,2147483647` covers exactly 1,000,000 indices at the top of the normal range.
- `--range 2147483647,2147483647` gives 1 line.
- The ordering `a > b` before `b - a` leaves no underflow path.

Fix 04 (`--from-md1-set` with a non-ASCII line): `s.get(..3)` cannot panic. The row is in the settled 8/8.

Manual (toolkit 0637fca9, `44-mk-cli.md`): all five changed rows match FIX §8 and the `--help` text the binary prints. The binary behaves as the rows say.

### Findings

- **M1** (Minor, docs): `crates/mk-cli/CHANGELOG.md` `[Unreleased]` has no entry for the behaviour change. A single-card `mk encode` with neither fingerprint flag used to exit 0 with no fingerprint and now exits 64. `mk address` now refuses `--count`/`--range` above 1,000,000. Both break scripts that relied on the old behaviour, and the section lists such changes as BREAKING entries. Every check passes:
  - It fails closed with exit 64 and an actionable message.
  - No printed command fails.
  - Only a third-party script that calls `mk encode` with neither flag, or asks for more than 1,000,000 addresses, is affected.
  - Fix: add two BREAKING bullets under `[Unreleased]`, naming the exit-64 refusal text and the 1,000,000 cap. File as deferred-minor.

No Critical or Important. Each fix closes its CONFIRMED bug and matches the FIX doc's exact texts and exit codes. No probe produced a wrong key, card, address or fingerprint. No valid holder input was refused: uppercase cards, `--keys`/`--in` without a flag, and exactly 1,000,000 indices at the bottom, middle and top of the range are all accepted.

0 C / 0 I / 1 M / 0 N
