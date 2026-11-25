# Fold review r2: mnemonic-key `claude/project-thread-utm1aa` (I1 fold)

Reviewed: `git diff dd187de~2..dd187de` (3156c3b, dd187de) at HEAD `dd187de`, read-only. Probes used the debug `mk` built from HEAD. I made no change to tracked files. A throwaway worktree, since removed, was used for the revert test.

**Verdict: 0 Critical, 0 Important, 2 Minor, 1 Nit.** I1 is fixed on every route that quoted the whole record. One route still prints the record's first four characters (the N1 head, through `invalid_xpub`), and the CHANGELOG says no part of a record is printed (R2-M1). The two new map_err arms have no test (R2-M2).

## Gates (HEAD dd187de)

- `CARGO_INCREMENTAL=0 cargo test --locked -j 2 -p mk-cli`, captured once: **354 passed, 0 failed, 0 ignored**, 41 binaries (r1 had 353; the one added test is `key_file_record_is_never_quoted`, which passes).
- `cargo +1.95.0 fmt --all --check`: clean.
- `cargo clippy --locked -j 2 -p mk-cli --all-targets -- -D warnings`: clean.

## Q1: is I1 fully fixed?

Each case ran `mk encode --in kN.txt --policy-id-stub aabbccdd`, with and without `--json`. Every case exited 64.

| Record | Output (text; `--json` `message` is the same) |
|---|---|
| WIF line | `k1.txt:1: expected BIP-380 origin notation ...; this record (51 characters) does not have that shape (the record is not shown)` |
| BIP-39 phrase line | same, 93 characters |
| 5000-char line, WIF padded with whitespace | same, 5000 / 51 characters |
| `[fp/path]<WIF>/0/*` | `the record's key carries a derivation suffix; ...` |
| `[<WIF>/84h]xpub` | `the record's origin fingerprint is not 8 hex characters (the record is not shown)` |
| `[fp/<WIF>]xpub` | `the record's origin derivation path does not parse (the record is not shown)` |
| `[fp]<WIF>`, `[<WIF>]xpub` | `the record's origin has no derivation path ... (e.g. [3442193e/...])`; it uses the placeholder `73c5da0a` when the origin is not 8 hex characters |
| `[fp/path]xprv` with a bad checksum; a bare xprv with a bad checksum | fixed private-key refusal, nothing quoted |
| `[fp/path]<WIF>` (also with CRLF line ends, also on line 2) | `invalid xpub (begins "5Hue", 51 characters): base58 payload` |
| `[fp/path]<WIF>` with the last character changed (bad checksum) | `invalid xpub (begins "5Hue", 51 characters): base58 encoding error` |
| `[fp/path]<phrase>` | `invalid xpub (begins "aban", 93 characters): ...` |
| `[fp/path]<64-hex private key>` | `invalid xpub (begins "e8f3", 64 characters): ...` |
| `[fp/path] <WIF>` (space after `]`) | `invalid xpub (begins " 5Hu", 52 characters): ...` |
| `[fp/path]xpub <WIF>` (a secret after whitespace) | `invalid xpub (begins "xpub", 163 characters): ...`, so the secret is not shown |

The bitcoin 0.32.8 errors that `invalid_xpub` appends ("base58 payload", "base58 encoding error") carry no data, and no checksum or character value appears. Other routes:

- Only `mk encode` reads key files (`--keys`/`--in`, including stdin `-`, through `read_key_records`). No other command reads one.
- `CliError` is never printed with Debug outside tests.
- `check_record_set` prints only origins of records that already parsed, so they are public.
- The SLIP-0132 `mismatch_help` prints only the parsed path.

### R2-M1 (Minor): `[fp/path]<secret>` records still print the secret's first four characters, and the CHANGELOG says they don't

`crates/mk-cli/src/keyfile.rs:116` passes the key text to `parse_xpub_normalized`. For a non-xpub, that function reaches `invalid_xpub` (`crates/mk-cli/src/slip132.rs:168-173`), which prints the text's first four characters (see the table: `"5Hue"`, `"aban"`, `"e8f3"`, `" 5Hu"`).

This is the r1 N1 leak (about 11-17 bits), and r1 graded it Nit as a leak. Two statements in the fold contradict it:

- The new CHANGELOG text (`crates/mk-cli/CHANGELOG.md:39-40`) says "a `--keys`/`--in` key-file record is never quoted back at all ... its errors name the file, the line and the shape of the problem only".
- The code comment at `keyfile.rs:53-57` says "no refusal here quotes the record".

r1 raised I1 partly because a CHANGELOG claim was false, so this needs fixing, not only filing. The brief's bar is "nothing beyond length, line number, or a valid fingerprint", and this fails it.

**Fix.** Either option works:
- At `keyfile.rs:116`, map the `invalid_xpub` failure to a record message ("the record's key is not a valid xpub (N characters; the record is not shown)"), but let the SLIP-0132 path-mismatch error through unchanged.
- Or implement N1: print the head only when it is a known public prefix.

Otherwise, change the CHANGELOG to say "the first four characters".

### R2-M2 (Minor): the two new map_err arms and the `[fp/path]<secret>` shape have no test

`crates/mk-cli/tests/bh1125_private_key_echo.rs:221-261`. The five rows are WIF, phrase, `[fp/path]WIF/0/*`, `[WIF]xpub` and `[fp/WIF]`.

- `[WIF]xpub` takes the no-path branch.
- `[fp/WIF]` takes the empty-xpub branch, before the fingerprint or path is parsed.

So no row reaches `parse_fingerprint` (`keyfile.rs:105-110`) or `parse_derivation_path` (`:111-115`). Before the fold, both parsers quoted their input (`cmd/mod.rs:38,52`, `{s:?}`), so removing either new `map_err` would bring the leak back with the suite still green.

There is also no `[fp/path]<WIF>` row (R2-M1's shape). Even if one were added, `assert_not_echoed` skips the first four characters and checks 12-character windows, so it would not catch the head.

**Fix.** Add rows `[<WIF>/84h]<xpub>` and `[3442193e/<WIF>]<xpub>`. Add a `[fp/path]<WIF>` row once R2-M1 is decided.

### R2-N1 (Nit): a path typo loses its reason

`keyfile.rs:111-115` drops the bip32 error. A record with `48'/0'/0'/2''`, or a child index of 2^31 or more, now reads only "does not parse". Before the fold the message also gave the reason, for example "invalid child number format", under a misleading `--origin-path:` prefix.

The line number is enough to find the record. bip32's own `Display` text is data-free apart from the child-number value, so appending `{e}` would restore the reason. The fingerprint message ("is not 8 hex characters") covers both of the old fingerprint cases and is fine.

## Q2: legitimate xpub typos

The messages are still enough to fix a public record:

- An xpub with one character changed: `k19.txt:1: invalid xpub (begins "xpub", 111 characters): base58 encoding error`.
- A truncated xpub: same message, 100 characters, so the length shows the truncation.
- An extra leading character: `begins "5xpu", 112 characters`.
- A fingerprint or path typo: the field is named, plus the line (only the path reason is lost, R2-N1).
- `[fp]xpub` with no path: the user's own fingerprint is reused in the example.
- A suffix: the shape is named.

The user has the file and line, and the record is their own text. Dropping the wrong `--origin-fingerprint:`/`--origin-path:` flag prefixes is an improvement on a key-file route.

## Q3: does the test fail without the fix, and does it cover `--json`?

Yes on both counts. In a throwaway worktree at dd187de, I restored `keyfile.rs` to its pre-fold content (34c69b8) and kept the new test:

- `key_file_record_is_never_quoted` FAILED at `bh1125_private_key_echo.rs:49` (WIF row, text mode).
- The other 7 tests passed.

By reading the code, every row fails on its own under the old messages:
- WIF and phrase hit `usage`, which quoted `{s:?}`.
- The suffix row quoted `{xpub_str:?}`.
- `[WIF]xpub` quoted `{origin:?}`.
- `[fp/WIF]` hit `record {s:?}`.

Each row runs with and without `--json`, and the check reads both stdout and stderr, so the `--json` envelope is covered. The gaps are in R2-M2.

## Q4: are the FOLLOWUPS faithful, and did anything else change?

- `design/FOLLOWUPS.md` adds seven lines: M1-M4 at `grade=M` and N1-N3 at `grade=N`, the same grades as r1. Each `src=` points to the saved report's anchor.
  - The substance matches. M4's five sub-points are condensed but all present.
  - The N1 line does not mention that the four-character head also reaches the key-file `[fp/path]<secret>` route (see R2-M1). Update it if R2-M1 is resolved by filing rather than fixing.
- `design/agent-reports/bughunt-2026-11-25-mnemonic-key-review-r1.md` is byte-identical to the scratchpad r1 report (`diff` printed nothing).
- dd187de only changes `format!(...)` with no arguments to `"...".into()` at `keyfile.rs:92-96`, and the behaviour is the same.
- The diff touches no other file (5 files: CHANGELOG, keyfile.rs, the test, FOLLOWUPS and the saved review).

## Recommendation

R2-M1 is small and in the same function. Fix it (map the invalid-xpub error at `keyfile.rs:116`), or at least correct the CHANGELOG sentence, before the PR is offered. File R2-M2 and R2-N1, or fold them with R2-M1 (two test rows and one `{e}`). Re-review scope after that is `keyfile.rs:104-116` and its test rows only.
