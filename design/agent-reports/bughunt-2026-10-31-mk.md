# Bug report: bg002h/mnemonic-key @ bdb1df7

## Summary
- **Scope:** 37 Rust source files with about 11,500 lines, in the `mk-codec` and `mk-cli` crates. The fuzz targets and test helpers were not read.
- **Agents:** 9 slices, each read in full by a reader and again by a second reviewer. All 18 finished.
- **Result:** 4 suspicions were rated 3 or higher. All 4 were **demonstrated** with failing tests, and none were rejected.
- **How they were tested:** the pinned Rust 1.85 toolchain isn't installed in the sandbox, so the crates couldn't be built. Each test is a standalone `rustc` copy of the function in question, with its fix checked on the copy. Tests are under `/tmp/huntmk/prover-N/` in the sandbox.

Overall, the codec core looks solid. Both reviewers hand-checked the BCH/GF arithmetic, Berlekamp–Massey, Chien, Forney, chunking and reassembly, and the NUMS constants (checked in Python), and found no defects. The bugs below are at the edges: the CLI flags and the strictness of path decoding.

## Confirmed bugs (worst first)

### Medium

**1. `mk encode` silently mints a card without a fingerprint when no fingerprint flag is given** (demonstrated)
- Where: `crates/mk-cli/src/cmd/encode.rs` `run`, around lines 214-219. The `(None, false) => None` arm gives the same result as `(None, true)`, which is `--privacy-preserving`.
- Input: `mk encode --xpub X --origin-path m/48'/0'/0'/2' --policy-id-stub deadbeef`, with no `--origin-fingerprint` and no `--privacy-preserving`.
- Should: refuse with a usage error (exit 64). The spec, `SPEC_mk_v0_1.md` §6, says leaving out the fingerprint should be an explicit choice, not a default.
- Does: the card is minted without its fingerprint and nothing warns the user.
- Test: `prover-4/t.rs`.
- Fix: `(None, false) => return Err(CliError::UsageError("one of --origin-fingerprint or --privacy-preserving is required".into()))`.

### Low

**2. Path decoding accepts non-minimal LEB128, so one card has several valid encodings** (demonstrated)
- Where: `crates/mk-codec/src/bytecode/path.rs` `leb128_decode_u32`, line 154.
- Input: explicit path bytes `FE 01 80 00`, or `FE 01 81 80 80 00`.
- Should: reject them. The canonical forms are `FE 01 00` and `FE 01 01`, and SPEC §2.5 asks that re-encoding give back the same bytes.
- Does: decodes them to `m/0` and `m/1`, so decoding and re-encoding doesn't give back the original bytes.
- Test: `prover-2/sa.rs`.
- Fix: inside the loop, `if byte == 0 && shift > 0 { return Err(non-minimal) }`.

**3. `mk address --count` / `--range` can ask for 8 GiB of memory** (demonstrated)
- Where: `crates/mk-cli/src/cmd/address.rs` `resolve_indices`, line 209. Only the BIP-32 range is checked, not how many indices are requested.
- Input: `--count 2147483648` or `--range 0,2147483647`.
- Should: a usage error. The function's own doc comment promises it will "never" make a multi-gigabyte allocation.
- Does: collects 2^31 u32s (8 GiB), then tries to derive up to 2^32 addresses.
- Test: `prover-3/t.rs`.
- Fix: cap the count (e.g. 1,000,000) before calling `collect()`.

**4. `--from-md1-set` panics on a line that starts with certain non-ASCII characters** (demonstrated)
- Where: `crates/mk-cli/src/cmd/encode.rs` `read_md1_set`, line 480. The `s[..3]` slice cuts through the middle of a character.
- Input: a file line such as `éé note`.
- Should: skip the line, as the docstring promises for any line that isn't md1.
- Does: panics with "byte index 3 is not a char boundary".
- Test: `prover-1/t.rs`.
- Fix: `s.get(..3).is_some_and(|p| p.eq_ignore_ascii_case("md1"))`.

## Rejected
None.

## Not checked
These 22 suspicions were rated 2 or lower and were not sent to provers. The most notable:
- `bch_decode.rs:565`: the error count comes from the degree of Λ, not from the Berlekamp–Massey length `l`. In theory this could accept a wrong correction of a word with more than 4 errors. **This is the one most worth a targeted test**, since a wrong correction would change the card without an error.
- Two encodings for the same card: a standard-table path written as an explicit `0xFE` path, and a single string written as a "chunked 1-of-1".
- The chunk version field is ignored during reassembly.
- Possible panics in the public functions `to_5bit_symbols` (when `total_chunks` is 0) and `bch.rs:564` (an `ALPHABET[v]` lookup with no bound check).
- `encode_path` writes the component count without checking `MAX_PATH_COMPONENTS`.
- `inspect` doesn't warn when BCH corrections were applied.
- The coin type is never checked against the xpub network.
- Smaller items: the `repair` report shows wrong `was` values for uppercase input; a stale exit-code comment; `vectors` overwrites a file when two vectors share a name; the length labels in `cmd/mod.rs`; and two test-vector generator nits.

## What was read
Every non-test line of all 37 files was read by both a reader and a reviewer. The only exception is `mk-codec/src/error.rs`: the reader read all 307 lines, the reviewer about 90.
