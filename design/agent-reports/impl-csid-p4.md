# P4 implementation report — seedhammer fork Go derivation-parity test

**Scope:** IMPLEMENTATION_PLAN_chunk_set_id_verification.md P4 only — the
cross-language derivation-parity test (spec contract 8 + "Vectors (R4)").
No device UI, no JSON ingestion, no encoder/derivation change.

**Worktree:** `/scratch/code/shibboleth/sh-worktrees/csid-p4`, branch
`impl/csid-p4`, baseline `5f02773c`. Not committed — working tree left
dirty/unstaged per instructions.

## What was added

`mk/chunk_set_id_parity_test.go` — **166 lines**. New file, untracked
(`git status --porcelain mk/` → `?? mk/chunk_set_id_parity_test.go`;
`mk/encode.go` has zero diff, confirmed via `git diff --stat mk/encode.go`
after the mutation gate below). Contains:

- `csidParityVectors`: a `[]struct{name, bytecodeHex, derivedCSID string}`
  hand-carrying every row, matching the existing `parityVectors` pattern in
  `mk/mk_test.go`. No JSON reader added (out of scope per plan).
- `csidExtCleanRowCount = 20`, and `TestChunkSetIDDerivationParity` asserts
  `len(csidParityVectors) == csidExtCleanRowCount` before running any row,
  so a silently-dropped transcription row fails the test rather than
  passing with reduced coverage.
- Per row: `hex.DecodeString(bytecodeHex)` then
  `fmt.Sprintf("%05x", top20(bytecode))` compared against `derivedCSID`.

## Rows transcribed

Source: `/scratch/code/shibboleth/mk-worktrees/csid-p0/crates/mk-codec/src/test_vectors/csid_ext_v0.1.json`
(read-only; not modified). Measured via `jq`:

- Total rows: **21**
- Clean rows (`expect_mismatch_warning == false`): **20**
- Mismatch rows: **1** (`SEED_pinned_12345_ef12f`, declared `12345` /
  derived `ef12f` — deliberately excluded, this is the one row the corpus
  pins as a mismatch)

All **20/20** clean rows transcribed as literals (`canonical_bytecode_hex`
→ `bytecodeHex`, `derived_csid` → `derivedCSID`), including
`SP09_std_path_0x12` (csid `0012f`, the leading-zero row proving `{:05x}`
zero-padded rendering) and `LZ1_derived_below_0x10000` (csid `0191c`).
`csidExtCleanRowCount` pins the 20 count against the corpus's own clean
count, not a hardcoded guess.

## TDD: RED → GREEN

1. Wrote the test with all 20 correct rows first; ran it — GREEN (20/20
   pass), `go vet ./mk/` clean.
2. **RED:** edited `SP09_std_path_0x12`'s `derivedCSID` from `0012f` to a
   deliberately wrong `abcde`. `go test ./mk/ -run TestChunkSetIDDerivationParity -v`:
   `--- FAIL: TestChunkSetIDDerivationParity` / `--- FAIL: .../SP09_std_path_0x12`,
   all other 19 subtests still PASS (proves the assertion is row-scoped,
   not a blanket pass).
3. Restored `derivedCSID` to `0012f`. Re-ran with `-count=1` (bypassing the
   test cache): GREEN, 20/20 PASS.

## Mutation gate: `top20`

Before: `mk/encode.go:331-334`
```go
func top20(bytecode []byte) uint32 {
	h := sha256.Sum256(bytecode)
	return uint32(h[0])<<12 | uint32(h[1])<<4 | uint32(h[2])>>4
}
```

Mutated (temporary, in the worktree only): shifted by different amounts —
`<<13 | <<5 | >>3` in place of `<<12 | <<4 | >>4`.

`go test ./mk/ -run TestChunkSetIDDerivationParity -count=1 -v`: **20/20
subtests FAIL**, each with a distinct wrong computed value proving the
mutated line ran, e.g.:
```
top20(bytecode) = 4993a, want 24c9d (corpus derived_csid)
top20(bytecode) = 1a4ce0, want d2670 (corpus derived_csid)
top20(bytecode) = 0025f, want 0012f (corpus derived_csid)
```
(full 20-line list captured; every row's got-value differs from its
want-value and differs across rows — not a stuck/constant output).

Restored `top20` to the original three-line body verbatim;
`git diff --stat mk/encode.go` shows no diff (confirms exact restoration).

## Final verification

- `go vet ./mk/` → clean (no output, exit 0).
- `go test ./mk/ -v -count=1` (whole `mk/` package, not just the new
  test): **all tests PASS** — `TestEncodeGoldenRoundTrip` (7 subtests),
  `TestParseHeader`, `TestFiveBitToBytes`, `TestDecodeParity` (7
  subtests), `TestDecodeReassemblyOrderIndependent`, `TestDecodeNegative`
  (9 subtests), and the new `TestChunkSetIDDerivationParity` (20
  subtests) — `ok seedhammer.com/mk 0.016s`.
- `go` toolchain used: `/home/bcg/.local/go/bin/go` (`go1.26.4
  linux/amd64`) — not on default `$PATH` in this shell, added explicitly.

## Cross-language drift finding

**None.** All 20 clean rows' `top20(bytecode)` reproduced the Rust
corpus's `derived_csid` exactly on the first (unmutated) run — no row
failed to match. No Go derivation/encoder code was changed as part of
delivering this test (the mutation to `top20` was temporary and fully
reverted, verified via `git diff --stat`).

## Constraints honored

- Edited the canonical fork checkout's `mk/` package at the specified
  worktree/branch/baseline, not any other scratch copy.
- No `mk_codec`/derivation logic change left in place.
- Not committed; worktree left dirty (`?? mk/chunk_set_id_parity_test.go`
  only).
