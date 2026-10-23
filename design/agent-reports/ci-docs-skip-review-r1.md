r1: 0C/0I/0M/0N

# L3 delta review: mnemonic-key PR #14 (`feat/ci-docs-skip`), round r1

Head reviewed: 6404fcc91ffbb4f0f03484938a60a3ef6ce13fe9 (diff against merged base 945b97cb).
Scope: the four deltas in the brief only (cursory, per Brian 2026-10-22). Shared
files, the gate, byte identity, L1 and L2 were settled and not re-checked.

## 1. `ci/docs-only-paths.txt` additions and `ci/code-docs-exempt.txt`

- `ci/code-docs-exempt.txt` holds only its 4-line header comment: no exempt
  patterns, so no exempt file can be run by a guarded step. Matches SPEC §3 table
  ("exempt files: none").
- Added `- ^docs/verify-reproducibility\.md$`: named only in an error message in
  `ci/repro/vendor-freshness.sh:90`, which runs from `vendor-freshness.yml`
  (unguarded). Protecting it is conservative, not wrong; matches SPEC §3.
- Added `- ^CHANGELOG\.md$`: `crates/mk-cli/tests/version_is_honest.rs:17`
  `include_str!("../CHANGELOG.md")` reads `crates/mk-cli/CHANGELOG.md`, which
  `^[^/]+\.md$` never matched; the root file is protected for the name collision
  only. Conservative, not wrong; matches SPEC §3.
- Spot-check for an unprotected docs read: `grep` of `crates/`/`fuzz/` for
  `design/`, `docs/`, `bip/`, README reads finds only the two already-protected
  `design/` files (`SPEC_mk_v0_1.md`, `display-grouping-vectors.tsv`). The crates'
  `readme = "README.md"` are crate-local, not root, and only used by `cargo package`.

## 2. New step insertion sites

- `ci.yml`: inserted in job `fmt` (`name: fmt (pinned 1.95.0)`) immediately after
  `Self-test the docs-only guard` and before `Docs-only change?`, no `if:` (the
  self-test has none). Matches the §2.3 row `mnemonic-key | ci.yml | fmt | none`.
- `release.yml`: inserted in job `test` immediately after the self-test, with
  `if: runner.os == 'Linux' && github.event.inputs.tag == ''` as its second line,
  identical to the self-test's. Matches the §2.3 row. Step body is byte-for-byte
  the §2.3 block.

## 3. `release.yml` `test` job

- Checkout `ref:` is `${{ github.event.inputs.tag || (startsWith(github.ref, 'refs/tags/') && github.ref) || github.sha }}`; `name:` is `release test (${{ matrix.os }})${{ (github.event.inputs.tag || startsWith(github.ref, 'refs/tags/')) && ' (tag)' || '' }}`; guard `CHECK_NAME: release test (${{ matrix.os }})`. All three exactly as SPEC §2.1.
- Old name `test (<os>)`: no remaining reference in `.github/`, `ci/` or `CLAUDE.md`
  (grep). `needs: [test, build]` (release.yml:241) uses the unchanged job id.
  The only required check on `main` (live `gh api .../protection/required_status_checks`)
  is `build (stable on ubuntu-latest)`, from `ci.yml`; `CLAUDE.md`'s staging recipe
  names that same context. No merge tool or guard reads the old name.

## 4. PR description

Says what changes (A′, the code-docs check, the release.yml checkout and rename,
with the ` (tag)` suffix), states that it changes `.github/workflows/` and `ci/`,
cites the GREEN spec and byte identity, and ends with the Claude Code footer.
PR is still draft at 6404fcc9. Its omission of the repo's own paths/exempt lines is
PR 2 L3 N1, already filed; not re-raised.

## Findings

None.
