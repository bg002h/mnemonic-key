# FIX_bh1119_mk_out_writes: `mk encode` output writes that fail closed and never replace an input (bh-mk-I1, bh-mk-I6; both graded Important)

Small-fix track (bughunt-1119 FIX table row #10, "F-B output writes (mk)"). Repo mnemonic-key, branch
`fix/mk-out-writes` from main `58701fb`. Crate `mk-cli` only; no mk-codec change, no wire change, no new flag, type or
command. Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24.

Rulings used, all decided (coordinator paths):
- `agent-reports/architect-bughunt-1119-questions.md` T5 (the safe write lives in mnemonic-io-lib `write_private`, and
  `same_file` answers the alias rows; mk takes both by a pin bump), T6 (failed-write contract: prefix
  `<tool>: cannot write the output:`, exit 1, for stdout and `--out`), P5 (grade of a failed overwrite: I).
- Brian 2026-11-23 file-output convention (`memory/file-output-convention-rulings-2026-11-23.md`): Q2 an output that
  names an input is refused, exit non-zero, naming both, "write to a new name"; Q3 a replace notice (deferred here, see
  section 8); R5 (convention report s3): a write failure names the path, the OS error and the state on disk.
- `agent-reports/architect-mt-output-writes-1124.md` (the sibling rulings this doc matches): point 1 the registry pin;
  point 2 the failure line `<tool>: cannot write the output: <path>: <error>` with R5's state sentence chosen by a
  second look at the path after the failure (F1); point 4 stdout is an output, so the same-file refusal covers it;
  point 5 the replace notice waits for io-lib 0.1.3.
- Batch overrides `briefs/bh1119-batch-1124-overrides.md`: every consumer pins the registry `mnemonic-io-lib = "0.1.2"`.

Final grades: **bh-mk-I1 Important** (grader I; the P5 class, Fable review OQ2 upheld I everywhere), **bh-mk-I6
Important** (grader I; data loss at exit 0, one condition).

Every new holder-visible text is PROPOSED (architect confirms) unless it is quoted from T6 or from the architect's
mt/md ruling. No point is open as a STOP; the author's choices inside the rulings are listed in section 9.

Toolchain: `export PATH=$HOME/.cargo/bin:$PATH`; cargo 1.85.0 (`rust-toolchain.toml`); `--locked`; every cargo call with
`CARGO_BUILD_JOBS=6`; nextest `--build-jobs 6 -j 6`; never `--release`. Fixtures are public: the testnet account key
`[73c5da0a/84'/1'/0']tpubDC9Go1KD...` (mk-cli fixture `V15_84_TEST`) as one key record, and the keyless md1 template
`md1yqpqqxzq2qwfv8urt848e` (fixture `PKH_BASIC_MD1`).

**crates.io at authoring:** `cargo search mnemonic-io-lib --limit 1` lists `0.1.0` only. The GREEN prototype therefore
ran against io-lib 0.1.2 through a local `[patch.crates-io]` path to `git archive e09d538d crates/mnemonic-io-lib`
(version `0.1.2`, the tree the overrides name). The builder runs `cargo search mnemonic-io-lib --limit 1` first; if it
does not list `0.1.2`, the builder does every step that needs no build, then STOPs and reports (the publish is Brian's).

## 1. The defects (RED evidence, run by the author on the base 58701fb)

Base binary `target/debug/mk` from `cargo build --locked -p mk-cli --bin mk`. "FSIZE8" = run under
`bash -c "trap '' XFSZ; exec prlimit --fsize=8 mk ..."` (file-size limit 8 bytes, the signal ignored, so the write
returns EFBIG; this stands in for a full disk or quota), with stdout and stderr through pipes.

- **bh-mk-I1 (a failed `--out` write destroys the existing file).** `mk encode --in keys --policy-id-stub deadbeef --out
  F` under FSIZE8, F holding `PRIOR-CARD\n`: exit 64, stderr `error: --out F: File too large (os error 27)`, and F is
  now the 8 bytes `mk1qpr4p`. With F absent, the same run leaves an 8-byte partial F. With `--json`, the error is a JSON
  envelope on stdout (`"kind":"UsageError"`, exit 64) and F is destroyed the same way. The cause is
  `encode.rs:470-475` calling mnemonic-io-lib **0.1.0** `write_private`, which opens, truncates and writes in place.
  The error is a `UsageError` (exit 64), which T6 makes exit 1 with the `mk: cannot write the output:` prefix.
- **bh-mk-I6 (the output replaces an input).** `mk encode --in keys --policy-id-stub deadbeef --out keys`: exit 0, and
  `keys` now holds the card instead of the key record. The same through a hard link (`--out hl`), a symlink (`--out
  sl`), `--keys keys --out ./keys`, `--from-md1-set set.md1 --out set.md1`, and `--in - --out keys <keys`; with
  `--json` the input is replaced by the JSON document. Without `--out`, stdout is the output: `mk encode --in keys ...
  >>keys` exits 0 and appends the card to the key file (3 lines), and `--json >>keys` appends the JSON; with `>keys` the
  shell empties the file first and mk reports `--in keys: no key records found ...` (exit 64), a message that hides why.
- **Also measured (io-lib, not this row's fix):** `mk encode ... --out /dev/stdout >>s.txt` replaces s.txt's earlier
  contents (`OLD BACKUP\n`) with the card, at exit 0, on the base AND on the 0.1.2 prototype: io-lib writes a
  standard-stream name in place and truncates the regular file behind it. This confirms the lead the architect left
  unverified (architect-mt-output-writes-1124 "Open questions"). Routed to io-lib in section 6; row o20 pins only
  that mk's failure line makes no false claim about it.

## 2. Spec delta (behaviour)

- **R1 (I1, T5): the registry pin.** `crates/mk-cli/Cargo.toml:68` `mnemonic-io-lib = "0.1.0"` becomes
  `mnemonic-io-lib = "0.1.2"`. `Cargo.lock` follows (`cargo update -p mnemonic-io-lib --precise 0.1.3`): its one `mnemonic-io-lib`
  entry becomes version `0.1.2`, source `registry+https://github.com/rust-lang/crates.io-index`, with crates.io's
  checksum. `vendor/` is regenerated with `cargo vendor --locked vendor/` and committed with the lock
  (`vendor/mnemonic-io-lib` becomes 0.1.2; `ci/repro/vendor-freshness.sh` must pass; no `vendor-git-sources.txt` row,
  since there is no git source). The comment at `crates/mk-cli/Cargo.toml:43-45` becomes exactly:
  `# P3's boundary table: `mk` ADOPTs items from the shared constellation IO crate --`
  `# `write::write_private`, which `mk encode --out FILE` uses to create its artifact owner-only, and (since 0.1.2,`
  `# FIX_bh1119_mk_out_writes) `write::same_file` with `write::Endpoint`, which `mk encode` uses to refuse an`
  `# output that is one of its inputs. The other ten items of the 0.1.0 surface are declined (see the plan).`
  and `:50` "`mnemonic-io-lib 0.1.0` is on crates.io." becomes "`mnemonic-io-lib` is on crates.io (0.1.2 since
  FIX_bh1119_mk_out_writes)." `encode.rs` keeps calling `mnemonic_io_lib::write::write_private` and adds no write logic
  of its own. What the holder sees, from 0.1.2's documented behaviour (`crates/mnemonic-io-lib/src/write.rs:77-98` at
  mnemonic-engrave e09d538d):
  - an existing regular FILE (or an absent one) is written to a temporary `<name>.<pid>.<n>.tmp` beside it, created
    0600, synced, and renamed over FILE; any error leaves an existing FILE byte-identical and removes the temporary;
  - a run killed mid-write can leave that temporary behind (0600);
  - FILE's directory must be writable: an existing writable FILE in a read-only directory, written in place before, is
    now refused (R3 text) and left unchanged;
  - a symlink FILE is followed and its target replaced; the link stays a link. A hard-linked FILE that is not an input
    is replaced at this name only;
  - a FIFO, device or standard-stream name (`/dev/stdout`, `/dev/fd/N`, `/proc/...`) is written directly, as today.
- **R2 (I6, Q2): `--out` must not be an input.** The very first thing `mk encode` does (`encode.rs:123`, before the
  `--privacy-preserving`/`--in`+`--keys` refusals at `:124-169` and before any file or stdin is read) is ask
  `mnemonic_io_lib::write::same_file(out, inputs)`. `out` is `Endpoint::Path(--out)` when `--out` is given (R4 covers
  the case without it). `inputs`, in this order: `--in` if given, `--keys` if given (each `Endpoint::Stdin` when its
  value is `-`, else `Endpoint::Path`), then every `--from-md1-set` value in the order given (always
  `Endpoint::Path`, since that flag reads a file even when it is named `-`). On a match (same path, a symlink or a hard
  link; io-lib never matches a tty, pipe or device), mk refuses with text X1, naming the FIRST matching input, exit
  **64** (mk's refusal code for argv it will not run), and writes nothing: the input and any `--out` file are
  byte-identical, stdout is empty. In `--json` mode the refusal is mk's usual error envelope on stdout (`kind`
  `UsageError`, `exit_code` 64, `message` = X1 without the `error: ` prefix), as every other exit-64 refusal of
  `mk encode --json` is.
- **R3 (I1, T6 + R5 + architect F1): the failure line.** Any error from `write_private` on the `--out` file (including a
  path that cannot be created and a read-only directory) prints exactly one line on stderr, chosen as in mt's W3:
  before the write mk takes `std::fs::metadata(FILE)` (symlinks followed): a regular file keeps (dev, ino, len,
  mtime) on Unix and (len, mtime) elsewhere (std has no stable file id on Windows); `NotFound` (including a dangling symlink) is "absent"; anything else is "other". On the error mk takes the
  same metadata again, then prints:
  - regular before and the same (dev, ino, len, mtime) after: `mk: cannot write the output: <FILE>: <error>. <FILE> is unchanged.`
  - absent before and still `NotFound` after: `mk: cannot write the output: <FILE>: <error>. Nothing was written.`
  - anything else: `mk: cannot write the output: <FILE>: <error>`

  `<FILE>` is `--out` as typed; `<error>` is the `std::io::Error` Display, unmodified (e.g. `File too large (os error
  27)`). Exit **1** (was exit 64 and `error: --out <FILE>: <error>`). Stdout is empty in both modes: in `--json` mode no
  envelope is printed (T6 gives the one line). Nothing else follows on stderr (no `card N of M` lines, no watch-only
  note): the run stops at the failed write. The second look keeps the sentence true for a standard-stream name (`--out /dev/stdout >>s.txt`): io-lib 0.1.2 @e09d538d truncates s.txt there before the failing write, so no sentence follows; an io-lib that appends leaves s.txt intact, and the line ends `. /dev/stdout is unchanged.` (row o20 accepts either, never a false one).
  Implementation note (builder's choice): the encode arm may print through `crate::out::stderr_write` and return
  `Ok(1)`, which `main` passes through `out::exit_code` unchanged; or add a `CliError` variant whose rendering is this
  line in both modes. The stdout sink's own line (`mk: cannot write the output: <error>`, `out.rs:80`) is unchanged.
- **R4 (I6 widened to stdout; architect point 4): stdout must not be an input.** When `--out` is not given, or `--json` is given (stdout then carries the JSON or an error envelope), the R2 check runs with `out` = `Endpoint::Stdout` and the same `inputs`, BEFORE the R2 `--out` check. Then, when `--json` and `--out` are both given, mk asks `same_file(Endpoint::Stdout, [Endpoint::Path(--out)])`; on a match it refuses with X3, exit 64, as mk's usual `--json` error envelope on stdout (stdout is FILE, not an input, so FILE then holds the envelope; the shell had already emptied it), as ms #11 R3 does. On a match mk refuses with text X2, exit **64**,
  and writes nothing to stdout. X2 is printed on **stderr in both modes**: in `--json` mode it is NOT an envelope on
  stdout, because stdout is the input file and an envelope there would be written into it. This refuses
  `mk encode --in K ... >>K`, `>K` (the shell has already emptied K; X2 replaces the misleading "no key records found"),
  `--in - <K >>K`, and the same for `--keys` and `--from-md1-set`. A pipe, tty or `/dev/null` on stdout never matches.
- **R5 (help).** The `mk encode --out` help (`encode.rs:53-54`, its first paragraph; the NOTE paragraph stays) becomes
  H1. No flag is added, removed or renamed, so the mnemonic-gui mk schema mirror is not touched.
- **R6 (T7):** `mk encode --json --out FILE` writes both: FILE receives exactly the bytes `mk encode --out FILE` writes without `--json` (the mk1 strings, one per line, grouped under `--group-size`), through `write_private` (R1, R3); stdout carries the same JSON document `mk encode --json` prints without `--out`, printed after the FILE write succeeds. If the FILE write fails, R3 applies and stdout stays empty (row o4 unchanged). The SPEC §6b comment at encode.rs:468-469 becomes `// SPEC §6b: stdout is used when --out is not given; with --out the strings go to the file, and with --json the JSON also goes to stdout (T7).`
- **Not changed:** `mk vectors --out DIR` and `mk gen-man --out DIR` (directory writers with `std::fs::write`, not
  through `write_private`; out of this row, section 8). The read verbs (`decode`, `inspect`, `verify`, `address`,
  `derive`, `repair`) with `--in FILE` and stdout appended to FILE are not refused by this row (section 6 line
  `bh-mk-read-verbs-stdout-names-input`).

## 3. Texts (exact)

- **E (T6 prefix with R5 path and state; the shape the architect ruled for mt and md, not new):** the three R3 lines.
- **X1, `--out` names an input, PROPOSED (architect confirms):**
  `error: --out <OUT> is the same file as <INPUT>, so writing the card would replace the input mk is reading. Write to a new name.`
  `<OUT>` is `--out` as typed. `<INPUT>` is one of `--in <PATH>`, `--keys <PATH>`, `--from-md1-set <PATH>` (PATH as
  typed), `standard input (--in -)`, `standard input (--keys -)`.
- **X2, stdout names an input, PROPOSED (architect confirms):**
  `error: standard output is the same file as <INPUT>, so the card would go into the file mk reads its input from. A > redirect empties that file before mk starts, so check that it still holds what you expect. Write to a new name.`
  (`<INPUT>` as in X1. The `>` sentence follows mt's ruled stdout mechanism: the shell truncates before mk runs, and mk
  cannot undo that.)
- **X3, PROPOSED (ms #11 E2 with ms -> mk):** `error: --out <OUT> and standard output are the same file (the same path, a symlink or a hard link), so what mk prints on stdout would go into the file it writes. A > redirect empties that file before mk starts, so check that it still holds what you expect. Write to a new name.`
- **H1, `mk encode --out` help, first paragraph (`encode.rs:53-54`), PROPOSED:**
  `/// Write the mk1 artifact to FILE, created 0600, instead of to stdout.`
  `/// OVERWRITES an existing file (operator ruling, 2026-08-26); a failed write`
  `/// leaves it unchanged. Refused when FILE is one of the inputs (`--in`,`
  `/// `--keys`, `--from-md1-set`). With --json, FILE gets the mk1 strings and the JSON goes to stdout. SPEC §6b.`
- **M1, toolkit manual `docs/manual/src/40-cli-reference/44-mk-cli.md:42` (the `mk encode` `--out` row, toolkit master
  e30985a5), PROPOSED, whole row:**
  ``| `--out <FILE>` | write the mk1 artifact to FILE, created `0600`, instead of stdout; the file holds exactly what stdout would without `--json` (grouped under `--group-size`); with `--json` the file still gets the mk1 string(s) and the JSON goes to stdout; overwrites an existing file (operator ruling 2026-08-26). The new contents go to a temporary `FILE.<pid>.<n>.tmp` beside FILE that then replaces it, so a failed write leaves an existing FILE unchanged, FILE's directory must be writable, and a run killed mid-write can leave that owner-only temporary behind. A symlink is followed and its target replaced. FILE may not be one of the inputs (`--in`, `--keys`, `--from-md1-set`, or standard input): that is refused (exit `64`) before anything is read or written |``
- **M2, manual `44-mk-cli.md:622` (exit code `1` row), PROPOSED, whole row:**
  ``| 1 | mk1 format violation; codec rejected the input. Maps to `Error::*` kinds except `UnsupportedVersion`. Also an I/O error reading stdin, or writing `mk encode --out` / `mk vectors --out` / `mk gen-man --out` (an unreadable `--in FILE` is `64`). |``
- **M3, manual `44-mk-cli.md:629` (the paragraph after the exit-code table), PROPOSED, two sentences appended:**
  `` A write error on `mk encode --out FILE` (a full disk, a directory that does not exist or cannot be written) prints `mk: cannot write the output: FILE: <error>`, followed by `. FILE is unchanged.` or `. Nothing was written.` when mk can see that is so, and exits 1. `mk encode` refuses (exit 64, before reading anything) an `--out` FILE or a redirected stdout that is the same file as one of its inputs, naming both.``
- **C1, `crates/mk-cli/CHANGELOG.md` `## [Unreleased]` / `### Changed`, PROPOSED (two bullets, after the existing ones):**
  - ``**A failed `mk encode --out` write no longer destroys the existing file** (bughunt-1119 bh-mk-I1). The card is written to a temporary file beside FILE and renamed over it (mnemonic-io-lib 0.1.2), so a full disk or quota leaves the old file byte-identical, and FILE's directory must now be writable. The error is now `mk: cannot write the output: FILE: <error>`, saying whether FILE is unchanged, at exit 1 (was `error: --out FILE: <error>`, exit 64).``
  - ``**`mk encode` refuses an output that is one of its inputs** (bughunt-1119 bh-mk-I6). `--out` naming the `--in`, `--keys` or `--from-md1-set` file (the same path, a symlink or a hard link), or stdout redirected into one of them, exits 64 naming both, before anything is read or written; it used to replace or extend the input at exit 0.``
  - **`mk encode --json --out FILE` writes both** (bughunt-1119 T7). The mk1 strings go to FILE and the JSON to stdout, as in md and ms; before, FILE held the JSON and stdout was empty. With `--json`, a stdout that is one of the inputs or the `--out` file is refused (exit 64).

Each of E, X1, X2, H1, M1-M3 gets a `design/WORDING_REVIEW.md` row (R5); those rows are direct-to-master records,
listed in the author's report, not appended here.

## 4. TRACE rows (RED on the base 58701fb; GREEN after the build)

The rows script `design/FIX_bh1119_mk_out_writes/rows.sh red|green` runs every row below as printed and checks it
(the builder moves it to `scripts/mk-out-writes-rows.sh`). Measured: `red` on the base binary `rows: 28 passed, 0
failed`; `green` on the base `rows: 4 passed, 24 failed` (only the four controls); `green` on the author's prototype
(io-lib 0.1.2 path patch + R2-R4) `rows: 28 passed, 0 failed`; `red` on the prototype `rows: 4 passed, 24 failed`.
The builder also writes each row as a nextest test in `crates/mk-cli/tests/bh1119_mk_out_writes.rs`, named exactly as
the "test" column `crates/mk-cli/tests/bh1119_mk_out_writes.rs` starts with `#![cfg(unix)]` (as `tests/channels.rs` gates its `--out` tests: on Windows io-lib's `same_file` detects no hard link and no redirected stdin/stdout, and the mode and read-only-directory rows are Unix semantics); the FSIZE8 rows o1, o2, o4 and o20 are also `#[cfg(target_os = "linux")]` (`prlimit`); o12 returns early, passing, when `libc::geteuid() == 0` (root ignores directory modes; ms #11 o6 does the same), and rows.sh skips o12 under root the same way. On Linux the count stays as section 5 says.. In every row: `ENC` =
`mk encode --in keys --policy-id-stub deadbeef`; `keys` holds the one key record; `F` holds `PRIOR-CARD\n`; `R` =
`so writing the card would replace the input mk is reading. Write to a new name.`; `S` = the X2 tail after
`<INPUT>, `.

| row | test | command | RED (base) | GREEN |
|---|---|---|---|---|
| o1 | `bh_mk_i1_failed_replacement_preserves_old_card` | FSIZE8 `ENC --out F` | exit 64, stderr `error: --out F: File too large (os error 27)`, F = `mk1qpr4p` | exit 1, stderr exactly `mk: cannot write the output: F: File too large (os error 27). F is unchanged.`, F byte-identical, no `*.tmp` |
| o2 | `i1_failed_write_leaves_an_absent_out_absent` | FSIZE8 `ENC --out F`, F absent | exit 64, same line, 8-byte F left | exit 1, `mk: cannot write the output: F: File too large (os error 27). Nothing was written.`, F absent, no `*.tmp` |
| o3 | `i1_missing_directory_prints_the_t6_line` | `ENC --out nodir/F` | exit 64, `error: --out nodir/F: No such file or directory (os error 2)` | exit 1, `mk: cannot write the output: nodir/F: No such file or directory (os error 2). Nothing was written.`, stdout empty |
| o4 | `i1_json_failed_write_keeps_stdout_empty` | FSIZE8 `ENC --json --out F` | exit 64, envelope on stdout, F destroyed | exit 1, stderr the o1 line, stdout empty, F byte-identical |
| o12 | `i1_read_only_directory_refused_file_unchanged` | `ENC --out ro/F`, ro mode 555, ro/F 0644 `PRIOR\n` | exit 0, ro/F = the card (written in place) | exit 1, `mk: cannot write the output: ro/F: Permission denied (os error 13). ro/F is unchanged.`, ro/F byte-identical |
| o20 | `i1_stdio_alias_onto_a_file_names_no_state` | FSIZE8 `ENC --out /dev/stdout >>s.txt`, s.txt `OLD BACKUP\n` | exit 64, `error: --out /dev/stdout: File too large (os error 27)` | exit 1, stderr starts `mk: cannot write the output: /dev/stdout: File too large (os error 27)`, and its state sentence is true of s.txt: no sentence and s.txt no longer `OLD BACKUP\n` (io-lib 0.1.2 @e09d538d truncates behind a stdio alias), or `. /dev/stdout is unchanged.` and s.txt still `OLD BACKUP\n` (an io-lib that appends); anything else fails |
| o5 | `i6_out_same_path_as_in_refuses` | `ENC --out keys` | exit 0, keys = the card | exit 64, `error: --out keys is the same file as --in keys, R`, keys byte-identical, stdout empty |
| o6 | `i6_out_aliasing_source_refuses` | `ln keys hl`; `ENC --out hl` | exit 0, keys = the card | exit 64, `error: --out hl is the same file as --in keys, R`, keys byte-identical |
| o7 | `i6_out_symlink_to_in_refuses` | `ln -s keys sl`; `ENC --out sl` | exit 0, keys = the card | exit 64, `error: --out sl is the same file as --in keys, R` |
| o8 | `i6_out_same_file_as_keys_refuses` | `mk encode --keys keys --policy-id-stub deadbeef --out ./keys` | exit 0, keys = the card | exit 64, `error: --out ./keys is the same file as --keys keys, R` |
| o9 | `i6_out_same_file_as_md1_set_refuses` | `mk encode --in keys --from-md1-set set.md1 --out set.md1` | exit 0, set.md1 = a card | exit 64, `error: --out set.md1 is the same file as --from-md1-set set.md1, R`, set.md1 byte-identical |
| o10 | `i6_out_same_file_as_stdin_refuses` | `mk encode --in - --policy-id-stub deadbeef --out keys <keys` | exit 0, keys = the card | exit 64, `error: --out keys is the same file as standard input (--in -), R` |
| o21 | `i6_json_out_refusal_is_an_envelope` | `ENC --json --out keys` | exit 0, keys = the JSON | exit 64, stderr empty, stdout envelope with `"message":"--out keys is the same file as --in keys, R"`, keys byte-identical |
| o15 | `i6_stdout_appended_to_in_refuses` | `ENC >>keys` | exit 0, keys has 3 lines | exit 64, `error: standard output is the same file as --in keys, S`, keys byte-identical |
| o16 | `i6_json_stdout_refusal_goes_to_stderr` | `ENC --json >>keys` | exit 0, JSON appended to keys | exit 64, stderr the o15 line, keys byte-identical (no envelope written into it) |
| o17 | `i6_stdout_truncating_in_refuses` | `ENC >keys` | exit 64, `error: --in keys: no key records found (blank lines and `#` comments are ignored)` | exit 64, the o15 line |
| o18 | `i6_stdout_and_stdin_one_file_refuses` | `mk encode --in - --policy-id-stub deadbeef <keys >>keys` | exit 0, keys has 3 lines | exit 64, `error: standard output is the same file as standard input (--in -), S`, keys byte-identical |
| o11 | `control_out_new_name_is_the_stdout_card` | `ENC --out G` | exit 0, stderr exactly the watch-only note, G 0600 = the stdout card, stdout empty | same |
| o13 | `control_out_symlink_replaces_its_target` | `ln -s F lk`; `ENC --out lk` | exit 0, F = the card, lk still a symlink | same |
| o14 | `control_out_overwrites_and_tightens` | F mode 0644; `ENC --out F` | exit 0, F = the card, mode 0600, no `*.tmp` | same |
| o19 | `control_stdout_pipe_is_not_an_input` | `ENC \| cat` | exit 0, the card on the pipe, keys untouched | same |
| t1 | `i1_t7_json_out_writes_both` | `ENC --json --out F` (F absent) | rc 0, F == J, stdout empty | rc 0, F == C mode 600, stdout == J |
| t2 | `i1_t7_json_out_grouped_file_unbroken_json` | `ENC --json --group-size 5 --out F` | rc 0, F == J | rc 0, F == C5, stdout == J |
| t3 | `i6_json_out_stdout_appended_to_in_refuses` | `ENC --json --out G >>keys` | rc 0, keys untouched (JSON to G) | rc 64, stderr exactly the o15 line, keys byte-identical, G absent |
| t4 | `i6_json_out_names_in_and_stdout_appends_to_it` | `ENC --json --out keys >>keys` | rc 0, keys replaced | rc 64, stderr the o15 line, keys byte-identical (no envelope appended) |
| t5 | `i6_json_usage_error_never_appends_to_in` | `mk encode --in keys --policy-id-stub zz --json --out G >>keys` | rc 64, envelope appended to keys | rc 64, stderr the o15 line, keys byte-identical |
| t6 | `i1_t7_json_stdout_is_out_refused` | `ENC --json --out F >F` | rc 0, F == J | rc 64, stderr empty, F holds the envelope with `"message":"<X3 with OUT=F, no error: prefix>"` |
| t7 | `control_json_out_piped_writes_both` | `ENC --json --out F \| cat` | rc 0, pipe empty | rc 0, F == C, pipe == J |

o20 also catches a build that skips R3's second look (on io-lib 0.1.2 @e09d538d): with a first look only, `/dev/stdout` resolves to the regular
s.txt and the line would end `. /dev/stdout is unchanged.` while s.txt was truncated (reasoned from io-lib
`write.rs:101-121,165-167` @e09d538d; that plant was not run by the author).

## 5. Tests that change

None (measured): the prototype (R1-R4, no new test) ran the workspace suite at `524 tests run: 524 passed, 0 skipped`
(`cargo nextest run --workspace --build-jobs 6 -j 6`; not `--locked` there only because the `[patch]` rewrote the
lock). The existing `--out` tests (`tests/channels.rs` `out_creates_the_artifact_owner_only`,
`out_tightens_an_existing_world_readable_target`, `out_overwrites_and_truncates`, `out_does_not_suppress_the_card`;
`tests/output_errors.rs`) pass unchanged. New: `crates/mk-cli/tests/bh1119_mk_out_writes.rs` with the 28 tests of
section 4, so the GREEN suite is **552 tests run: 552 passed** (524 + 28).

## 6. FOLLOWUPS (listed in the author's report in the format of `coordinator/docs/followups-format.md`; not appended here)

This row closes no existing mk FOLLOWUPS line (the keys were never filed there). It files three lines after merge: the
replace notice (Q3, waits for io-lib 0.1.3), the read verbs' stdout alias, and (in mnemonic-engrave) io-lib's in-place
write of a standard-stream name over a regular file.

## 7. Gate (builder, on the final head; run once each to a file, then grep)

`scripts/fix-bh1119-mk-out-writes-gate.sh` (the builder commits it; it states what it does not cover) runs, every cargo
call with `CARGO_BUILD_JOBS=6`:
- `cargo search mnemonic-io-lib --limit 1` lists `0.1.3` (else STOP, before any build);
- `cargo nextest run --locked --workspace --build-jobs 6 -j 6` expecting **552 tests run: 552 passed**;
- `cargo test --locked --workspace --doc`; `cargo +1.95.0 fmt --check --all`;
  `cargo clippy --locked --workspace --all-targets -- -D warnings`;
- `cargo nextest run --locked -p mk-cli --build-jobs 6 -j 6 -E 'binary(bh1119_mk_out_writes)'` expecting
  **28 tests run: 28 passed** (a short or empty count is a FAIL);
- `scripts/mk-out-writes-rows.sh green` expecting `rows: 28 passed, 0 failed`, and the same script `red` against a base
  binary (`MK=<base mk>`) expecting `rows: 28 passed, 0 failed`;
- the CI `fmt` job's other steps: `bash ci/docs-only.test.sh`, `sha256sum --check --strict ci/shared-ci.sha256`,
  `bash ci/code-docs-check.test.sh`, `bash ci/code-docs-check.sh`, `cd design && sha256sum -c
  display-grouping-vectors.tsv.sha256`;
- the `vendor-freshness` workflow: `sha256sum --check --strict ci/repro/shared-vendor.sha256`, then
  `bash ci/repro/vendor-freshness.sh`, then its mutation self-test `bash ci/repro/vendor-freshness.test.sh`;
- the `vectors-roundtrip` job's steps with a debug build (`target/debug/mk vectors --out <dir under /scratch/cache>`,
  then its jq diff against `crates/mk-codec/src/test_vectors/v0.1.json`), never `--release`;
- `cargo check --locked --target x86_64-unknown-freebsd -p mk-cli` if that target is installed (else say so);
  the macOS/Windows nightly legs are not reproducible locally (the R3 non-Unix branch is compiled there only);
- then `coordinator/scripts/mutation-gate.sh <worktree> 58701fb` before review (not in the script).

## 8. Overlaps and out of scope

- Q3 replace notice (one stderr line naming a replaced `--out` file): deferred to io-lib 0.1.3's `Written` outcome,
  as the architect upheld for mt and md (point 5). FOLLOWUPS line `mk-out-replace-notice`.
- `mk vectors --out DIR`, `mk gen-man --out DIR`: per-file `std::fs::write` into a directory; not this row's keys.
- The read verbs' stdout alias (`mk decode --in C >>C` and five more): the same class the architect filed for md as
  `md-out-names-input`; FOLLOWUPS line `bh-mk-read-verbs-stdout-names-input`.
- Build order against other open mk rows: F-F stale docs (`fix/mk-ff-stale-docs`) touches docs only; this row touches
  `crates/mk-cli/{Cargo.toml,CHANGELOG.md,src/cmd/encode.rs}`, `Cargo.lock`, `vendor/mnemonic-io-lib/`, a new test
  file, `scripts/`. No overlap.

## 9. Author's choices inside the rulings (flag for review)

1. Exit **64** for X1/X2 (mk's code for every argv refusal of `mk encode`; Q2 says "non-zero"; mt uses its own refusal
   code 1).
2. The R2/R4 check runs FIRST, before the existing argv refusals, so no message or envelope is ever written into a
   stdout that is an input; with several matches, the first input in the order `--in`, `--keys`, `--from-md1-set`
   is named.
3. X2 goes to stderr even in `--json` mode (stdout is the input file); X1 in `--json` mode is the usual envelope.
4. R3 in `--json` mode prints the T6 line and no envelope (T6 gives exactly one line).
5. Stdout as an output is refused on `mk encode` only (I6's verb), following the architect's mt point 4; the other
   verbs are filed, as the architect did for md (F2).
6. R3's identity off Unix is (len, mtime): std exposes no stable file id on Windows (io-lib's `same_file` documents
   the same limit), so a same-size, same-mtime rewrite would read as unchanged there.

## 10. Coverage map (rule -> rows)

- R1 (pin, atomic write, temp cleanup, read-only dir, symlink kept): o1, o2, o4, o12, o13, o14; vendor/lock: gate.
- R2 (`--out` names an input; each input kind; order; JSON envelope): o5, o6, o7, o8, o9, o10, o21; control o11.
- R3 (failure line, three states, exit 1, stdout empty, second look): o1 (unchanged), o2/o3 (nothing written), o4
  (JSON), o12 (read-only dir), o20 (no state; second look).
- R4 (stdout names an input; `>>`, `>`, stdin; JSON on stderr; pipe never matches): o15, o16, o17, o18; control o19.
- R5 (help), M1-M3, C1: text review only (no row; `mk encode --help` is checked by eye against H1).
