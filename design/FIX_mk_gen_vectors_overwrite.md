# FIX_mk_gen_vectors_overwrite: bughunt-1119 bh-mk-I16, gen_mk_vectors overwrites its own output

Small-fix track, grade I (refute upheld; coordinator ruling: a dev-only, feature-gated generator that prints a
false claim about what it wrote stays I, P8 printed text counts). Repo mnemonic-key, branch
`fix/mk-gen-vectors-overwrite`, base `origin/main` 826c2e6 ("Merge pull request #37 from
bg002h/build/mk-ff-stale-docs"). Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-24.

Inputs (decided; not re-ruled here): coordinator `agent-reports/bughunt-1119-mn-regrade/mk-grade.md` L28 and
L50-55, `mk-refute-bh-mk-I16.md` (NOT REFUTED), the dispatch brief `briefs/fix-doc-bh-mk-gen-vectors.md`.

## 0. Scope (who runs it, what it writes)

- `gen_mk_vectors` is the maintainer tool that regenerates the two committed vector corpora. It is built only
  with the `gen-vectors` feature (`crates/mk-codec/Cargo.toml` L18-28, `required-features`), run as
  `cargo run --bin gen_mk_vectors --features gen-vectors [-- --output <path>]`, and is in no release artifact
  (`.github/workflows/release.yml` L202/L208 build `-p mk-cli` only). No holder runs it.
- It writes exactly two files: the vectors corpus at `<path>` (default
  `crates/mk-codec/src/test_vectors/v0.1.json`) and the csid_ext corpus at
  `<directory of path>/csid_ext_v0.1.json`, creating the directory if it is missing. Both are truncated and
  rewritten (`File::create`).
- The fix stays proportionate: no new flag, no new dependency (no io-lib), no change to the corpora, the default
  path, argument parsing, or the existing `expect` panics on I/O errors (exit 101). It changes only what happens
  when the two output paths are the same file, and when the "wrote" lines are printed.

## 1. Defect (at 826c2e6; rows in section 4 run and seen)

`crates/mk-codec/src/bin/gen_mk_vectors.rs` `main` (L1416-1464): it writes the vectors corpus to `output_path`
(L1440) and prints `wrote 41 vectors to <output_path> (35615 bytes)` (L1441-1446). Then it computes the sibling
`output_path.parent().join("csid_ext_v0.1.json")` (L1450-1453) and writes the csid_ext corpus there (L1459).
When `output_path` is that sibling (its file name is `csid_ext_v0.1.json`), or is the same file under another
name (a hard link or symlink), the second write truncates the first. The run exits 0 and the first "wrote"
line names a file that holds 21 csid_ext rows and no vectors.

## 2. Spec delta: behaviour, texts, exit codes

The rules run in this order. `<P>` is `output_path.display()` exactly as given on the command line (or the
default); `<C>` is the sibling's `display()`; `<N>` is the number of vectors generated (`fixtures().len()`, 41
at 826c2e6); `<R>` is the csid_ext row count (21 at 826c2e6). All texts go to stderr, one line each, ending in a
newline; stdout stays empty in every case.

**Rule A (pre-write refusal).** After argument parsing and before any corpus is built or any file or directory
is created: if the final component of `<P>` (`Path::file_name`) is exactly `csid_ext_v0.1.json` (byte-equal,
case-sensitive), print T1 and exit **64**. Nothing is written and no directory is created. This catches
`--output <dir>/csid_ext_v0.1.json`, `--output csid_ext_v0.1.json` and a trailing slash
(`.../csid_ext_v0.1.json/`, whose `file_name` is the same).

- T1, PROPOSED (architect confirms):
  `gen_mk_vectors: refused: --output <P> is named csid_ext_v0.1.json, the csid_ext file this tool writes beside it; nothing was written`
- Exit 64, PROPOSED (architect confirms): EX_USAGE, the code the constellation already uses for a refused
  argument (B2).

**Rule B (post-write read-back).** Write the vectors corpus to `<P>`, then the csid_ext corpus to `<C>` (same
order and bytes as today). Then read `<P>` back in full. If its bytes differ from the vectors bytes just written,
print T2 and exit **1**. This catches every alias Rule A cannot see by name: a hard link, a symlink, and a
case-insensitive filesystem (`--output <dir>/CSID_EXT_V0.1.json` on macOS/Windows), with no platform code. The
read-back is the second look that confirms the disk state T2 states (`<P>` does not hold the vectors); T2 does
not claim which of the two causes happened.

- T2, PROPOSED (architect confirms):
  `gen_mk_vectors: error: <P> no longer holds the <N> vectors after <C> was written; it is the same file as <C> or was changed meanwhile`
- Exit 1, PROPOSED (architect confirms): a run-time failure, not an argument refusal.
- A read error on the read-back panics through `expect("read back vector output")` (exit 101), like every other
  I/O error in this tool.

**"wrote" lines.** Both existing lines keep their exact text and order:
`wrote <N> vectors to <P> (<bytes> bytes)` then `wrote <R> csid_ext rows to <C> (<bytes> bytes)`. They are
printed only after both writes have returned and the Rule B read-back has matched, so each names a file
actually written and still holding what it says. On a T1 or T2 exit neither line is printed.

Edge cases, decided:
- Default path (no `--output`): unchanged behaviour and output (row C1).
- `--output <dir>/v0.1.json` or any other name not aliasing the sibling: unchanged (row C1).
- `<P>` exists as a hard link or symlink to `<C>`: Rule B, exit 1 (rows R3, R4). After the T2 exit the shared
  file holds the csid_ext corpus, which is the correct content for `<C>`; nothing else on disk is touched.
- Case variants of the name on a case-sensitive filesystem (Linux) are different files and are written
  normally; on a case-insensitive one Rule B fires. Rule A stays exact-match on purpose.
- Argument errors (`--output` without a value, an unknown argument) keep their existing panics: out of scope.

## 3. Production changes (exact)

### 3.1 `crates/mk-codec/src/bin/gen_mk_vectors.rs`

- Add a private `const CSID_EXT_FILE_NAME: &str = "csid_ext_v0.1.json";` and use it for the sibling join
  (L1453) and in T1.
- `write_json_document` (L1402-1414) also returns the bytes it wrote (so `main` can compare the read-back); its
  write logic is unchanged.
- `main` (L1416-1464): Rule A right after `output_path` is resolved (L1431-1432); remove the `eprintln!` at
  L1441-1446 from between the writes; after the csid_ext write (L1459) do the Rule B read-back; then print the
  two "wrote" lines. `<N>` comes from the generated vectors list (no second `fixtures()` call is required).
- Replace the comment at L1448-1449 ("always a sibling ... regardless of --output overrides") with one that also
  says an `--output` named `csid_ext_v0.1.json` is refused and an aliased one is caught by the read-back.
- Module rustdoc (L10-15): after the run example add one paragraph: the tool also writes `csid_ext_v0.1.json`
  in the same directory as `--output`; `--output` may not be named `csid_ext_v0.1.json` (exit 64, nothing
  written), and an `--output` that is the same file as that sibling ends with exit 1.

### 3.2 `crates/mk-codec/CHANGELOG.md` (`[Unreleased]`, L13-38)

mk-codec's CHANGELOG already records generator changes (L46 names `gen_mk_vectors`). Insert a `### Fixed`
section between the `### Changed` list (L24-31) and `### Compatibility` (L33), with this bullet:

- **`gen_mk_vectors` (maintainer tool, feature `gen-vectors`) no longer overwrites its own output and reports it as written.** An `--output` named `csid_ext_v0.1.json` is refused before anything is written (exit 64); an `--output` that is the same file as the `csid_ext_v0.1.json` beside it (a hard or symbolic link) now ends with an error and exit 1 instead of exit 0. Both `wrote` lines print only after both files are written and the vectors file reads back as written (bh-mk-I16).

The SemVer headline line of `[Unreleased]` does not change (no library change).

### 3.3 Nothing else

No `Cargo.toml`/`Cargo.lock` change, no corpus change (C1 checks the bytes), no `docs/MK_CODEC_RUST_API.md`
change (no `pub` item changes; the binary is not API). No toolkit-manual lockstep and no GUI mirror: neither
`mk` nor `md-cli` flags or printed text change. No `.github/` change.

## 4. RED rows

All rows are in `scripts/mk-gen-vectors-rows.sh` (committed with this document; `red` and `green` modes) and,
for the builder, as nextest rows in the new `crates/mk-codec/tests/gen_vectors_output.rs` whose test functions
carry these names. Each row runs in a fresh directory.

| row | nextest name | command | base (826c2e6) | green |
|---|---|---|---|---|
| R1 | `gen_vectors_refuses_output_named_csid_ext` | `gen_mk_vectors --output <d>/csid_ext_v0.1.json`, `<d>` holding that file with `sentinel\n` | exit 0; stderr `wrote 41 vectors to <d>/csid_ext_v0.1.json (35615 bytes)` + csid line; the file holds no `"vectors"` | exit 64; stderr exactly T1 with `<P>`=`<d>/csid_ext_v0.1.json`; file still `sentinel\n`; `<d>` holds only that file |
| R2 | `gen_vectors_refuses_bare_csid_ext_name` | in an empty `<d>`: `gen_mk_vectors --output csid_ext_v0.1.json` | exit 0; `wrote 41 vectors to csid_ext_v0.1.json ...`; file holds no `"vectors"` | exit 64; stderr exactly T1 with `<P>`=`csid_ext_v0.1.json`; `<d>` stays empty |
| R3 | `gen_vectors_reports_hard_link_alias` | `<d>/csid_ext_v0.1.json` = `x\n`, hard link `<d>/v.json`; `--output <d>/v.json` | exit 0; `wrote 41 vectors to <d>/v.json ...`; `v.json` holds no `"vectors"` | exit 1; stderr exactly T2 with `<P>`=`<d>/v.json`, `<C>`=`<d>/csid_ext_v0.1.json`, `<N>`=41; no `wrote` line |
| R4 | `gen_vectors_reports_symlink_alias` (`#[cfg(unix)]`) | as R3 with a symlink `v.json -> csid_ext_v0.1.json` | as R3 | as R3 |
| C1 | `gen_vectors_writes_both_corpora` (control) | `--output <d>/v0.1.json` | exit 0; stderr exactly `wrote 41 vectors to <d>/v0.1.json (35615 bytes)\nwrote 21 csid_ext rows to <d>/csid_ext_v0.1.json (17476 bytes)\n`; both files byte-equal to `crates/mk-codec/src/test_vectors/` | the same |

The nextest rows compare against `<N>` and `<R>` parsed from `mk_codec::test_vectors::V0_1_JSON` and
`test_vectors::csid_ext::CSID_EXT_JSON` and the byte counts from their lengths, not hard-coded 41/21; their
directories live under `env!("CARGO_TARGET_TMPDIR")`; the file starts `#![cfg(feature = "gen-vectors")]` and
uses `env!("CARGO_BIN_EXE_gen_mk_vectors")`.

Seen (author, 2026-11-24, on 826c2e6):

```
$ scripts/mk-gen-vectors-rows.sh red
OK   red R1_output_named_csid_ext (defect: exit 0, claims 41 vectors, file holds none)
OK   red R2_bare_csid_ext_name (defect: exit 0, claims 41 vectors, file holds none)
OK   red R3_hard_link_alias (defect: exit 0, claims 41 vectors, file holds none)
OK   red R4_symlink_alias (defect: exit 0, claims 41 vectors, file holds none)
OK   red C1_writes_both_corpora
ALL OK (red)
$ scripts/mk-gen-vectors-rows.sh green      # on the base: exit 1
FAIL green R1_output_named_csid_ext: exit 0, want 64
FAIL green R1_sentinel: existing file was changed
FAIL green R2_bare_csid_ext_name: exit 0, want 64
FAIL green R2_nothing_written: directory is not empty
FAIL green R3_hard_link_alias: exit 0, want 1
FAIL green R4_symlink_alias: exit 0, want 1
OK   green C1_writes_both_corpora
FAIL green nextest: (the test file does not exist on the base)
```

The author also checked that section 2 is buildable and that the green rows match: a throwaway prototype of
3.1 plus the five nextest rows (never committed). The nextest rows against the base binary: 1 passed (C1),
4 failed (R1/R2 `left: Some(0) right: Some(64)`, R3/R4 `left: Some(0) right: Some(1)`). With the prototype:
`green` ALL OK, nextest 5 passed; `cargo clippy --locked -p mk-codec --features gen-vectors --all-targets
-- -D warnings` clean; `cargo +1.95.0 fmt --all -- --check` clean.

## 5. Tests that change

- New: `crates/mk-codec/tests/gen_vectors_output.rs` (R1-R4, C1 above).
- No existing test changes. `tests/vectors.rs` and `tests/csid_ext_vectors.rs` pin the corpus SHAs and are
  untouched (no corpus byte changes; C1 proves it).

## 6. Build order

1. RED: add `tests/gen_vectors_output.rs`; run it (`--features gen-vectors`) and see R1-R4 fail, C1 pass; commit.
2. GREEN: 3.1 and 3.2; run section 7; commit.

## 7. Gate

`scripts/mk-gen-vectors-rows.sh green` (exit 0, `ALL OK (green)`), plus the repo's CI jobs locally:
`cargo +1.95.0 fmt --all -- --check`; `cargo build --locked --workspace`;
`cargo nextest run --locked --workspace --no-fail-fast`; `cargo test --locked --workspace --doc`;
`cargo clippy --locked --workspace --all-targets -- -D warnings`; and, because no CI job enables the feature,
`cargo clippy --locked -p mk-codec --features gen-vectors --all-targets -- -D warnings`.

Not covered by CI: the default `cargo nextest run --workspace` does not enable `gen-vectors`, so the new test
file compiles to nothing there. A CI step for it is a `.github/` change, filed as a FOLLOWUPS line (report),
not done here.

## 8. FOLLOWUPS

bh-mk-I16 has no existing FOLLOWUPS line (grade report L50). The lines this item closes and files are listed
in the author's report for direct push after merge; none is added in this branch.

## 9. STOP (none open)

Every product point above that is new holder-visible text or an exit code (T1, T2, exit 64, exit 1) is marked
PROPOSED for the architect; no decision outside the rulings was needed.
