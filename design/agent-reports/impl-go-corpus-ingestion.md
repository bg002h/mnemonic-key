# impl: go-mk-vector-corpus-ingestion -- vendored corpus + SHA gate + go:embed reader

Worktree: `/scratch/code/shibboleth/sh-worktrees/corpus-ingest`
Branch: `followup/go-corpus-ingestion`, base `195df90` (seedhammer fork)
Status: **worktree left dirty/unstaged, no commit made** (per instructions).

## What changed

Replaced the hand-transcribed `csidParityVectors` literal table in
`mk/chunk_set_id_parity_test.go` with a vendored copy of the Rust corpus +
a `go:embed` reader + a SHA-256 gate, so Rust<->Go drift becomes a red
test instead of a silent manual-copy gap.

### Files

- **Added** `mk/testdata/csid_ext_v0.1.json` -- byte-for-byte vendored copy
  of `mnemonic-key/crates/mk-codec/src/test_vectors/csid_ext_v0.1.json`
  (verified via `diff`: identical). 17,476 bytes, untracked (new file).
- **Changed** `mk/chunk_set_id_parity_test.go` -- rewritten. 166 -> 123
  lines (`git diff --stat`: 93 insertions, 136 deletions against the P4
  version). Adds:
  - `//go:embed testdata/csid_ext_v0.1.json` into `csidCorpusJSON []byte`.
  - `csidCorpusSHA256` const pin (SHA gate).
  - `csidCorpusRow` / `csidCorpus` structs (name, canonical_bytecode_hex,
    derived_csid, expect_mismatch_warning -- json.Unmarshal ignores the
    unread fields: declared_csid, description, warning_text, strings).
  - `TestVendoredCorpusSHA256` -- asserts `sha256(csidCorpusJSON)` equals
    the pin.
  - `TestChunkSetIDDerivationParity` -- parses the embedded corpus,
    filters clean rows (`expect_mismatch_warning == false`), asserts the
    clean-row count == `csidExtCleanRowCount` (20), asserts
    `top20(hexDecode(canonical_bytecode_hex))` == `derived_csid` per row
    (same assertion as P4), and asserts at least one clean row has
    `derived_csid < 0x10000` (leading-zero coverage, r4 L1-I2).
- **Untouched**: `mk/encode.go` (`top20` and all derivation logic).

### SHA pin

```
csidCorpusSHA256 = 88bbe056e85dde694353475e774a78a00defe75cb8694654c4be1d2467ad68f9
```
(`sha256sum mk/testdata/csid_ext_v0.1.json`, matches the pinned constant.)

## Clean rows read from the embed

**20 of 21** rows parsed with `expect_mismatch_warning == false` --
matches the pinned `csidExtCleanRowCount` and the count guard held (a
`t.Fatalf` on mismatch, not exercised in the green run since the count
matched). All 20 clean-row subtests passed, including the two rows with
`derived_csid < 0x10000` (`SP09_std_path_0x12` = `0012f`,
`LZ1_derived_below_0x10000` = `0191c`) -- the leading-zero assertion
observed `sawLeadingZero == true`.

## RED/GREEN demonstrations

### 1. Corrupted `derived_csid` (CT1 row: `83bb2` -> `83bb3`)

Before (baseline): `go test ./mk/` -- `ok seedhammer.com/mk 0.049s`.

After corrupting the vendored file's `"derived_csid": "83bb2"` (CT1's
only occurrence) to `"83bb3"`:

```
--- FAIL: TestVendoredCorpusSHA256 (0.00s)
    chunk_set_id_parity_test.go:72: sha256(testdata/csid_ext_v0.1.json) = 66050bc21306d8b5c887dea2b73e12efd7d13255348f6142acc5031eb2c83443, want 88bbe056e85dde694353475e774a78a00defe75cb8694654c4be1d2467ad68f9 (pinned) ...
--- FAIL: TestChunkSetIDDerivationParity (0.00s)
    --- FAIL: TestChunkSetIDDerivationParity/CT1_twin_of_V1_bip48_mainnet_1_stub_with_fp (0.00s)
    --- PASS: TestChunkSetIDDerivationParity/CT2_... (and all other rows PASS)
FAIL
```

Both gates fired, as expected for a content edit that changes both the
assertion input and the file's bytes.

### 2. Isolated SHA-gate-only demonstration

Changed one byte in a field the parity test never reads
(`"family_token": "mk-codec 0.5"` -> `"mk-codec 0.6"`) to isolate the SHA
gate from the parity assertion:

```
--- FAIL: TestVendoredCorpusSHA256 (0.00s)
--- PASS: TestChunkSetIDDerivationParity (0.00s)
    (all 20 subtests PASS, including CT1, SP09, LZ1)
FAIL
```

This proves the SHA gate is a genuine additional check: it caught a
vendored-file edit that the parity assertion, driven only by fields it
reads, would have silently accepted.

### Restore

`cp` back from a pre-corruption backup; verified `diff` against
`mnemonic-key/crates/mk-codec/src/test_vectors/csid_ext_v0.1.json` ==
identical, and `sha256sum` == the pinned value again.

## Final verification (post-restore)

```
$ go test ./mk/
ok  	seedhammer.com/mk	0.017s

$ go test ./mk/ -v   (full package)
PASS
ok  	seedhammer.com/mk	0.028s
# 13 top-level "--- PASS", 0 "--- FAIL" (grep -c on captured log)

$ gofmt -l mk/
(empty)

$ go vet ./mk/
(empty, exit 0)

$ git diff --stat mk/encode.go
(empty -- top20/encode.go untouched)

$ git status --porcelain
 M mk/chunk_set_id_parity_test.go
?? mk/testdata/
```

## Deviations from the brief

None. `top20` and all Go derivation/encoder logic in `mk/encode.go` are
untouched (confirmed by empty `git diff --stat`). No cross-language drift
was found -- all 20 clean rows matched on the first real run, so there is
no Critical to report. `gui` suite was not run (out of scope per brief).
Worktree left dirty/unstaged; no commit created.
