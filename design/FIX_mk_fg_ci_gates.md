# FIX_mk_fg_ci_gates: bughunt-1119 F-G, CI and release gates (bh-mk-I11, bh-mk-I12, bh-mk-I13)

Small-fix track, grade I (all three keys). Not class B (no card, key or address output changes).
Repo mnemonic-key, branch `fix/mk-fg-ci-gates`, base `origin/main` 9b0b365 ("Merge pull request #30
from bg002h/build/mk19"). The fix edits `.github/workflows/` inside its normally reviewed PR; the
PR description says so (coordinator RULES "Builds and tests"). No toolkit manual change (no mk flag
or printed CLI text changes).
Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22.

Inputs (decided; not re-ruled here): coordinator `agent-reports/architect-bughunt-1119-questions.md`
row #44 and T45 ("stable lanes block; a beta lane may stay, labelled non-blocking. A gate that cannot
fail was the defect; a labelled informational lane is not a gate"); grader
`agent-reports/bughunt-1119/grade-mk.md` (I11, I12, I13, F-G, Q8); triage `triage-mk-t3.md` (RED
scripts `ci/bh_mk_i1{1,2,3}.sh`, note 2 on new clippy, note 3 on the two adjacent `|| true`);
consultant report `design/agent-reports/bug-hunt-2026-11-19.md` I11-I13.

## 1. Defects (at 9b0b365; rows in section 5 run and seen)

- **bh-mk-I11.** `rust-toolchain.toml` L2 pins `channel = "1.85.0"` (added in e3c3984 for the
  reproducible musl build). rustup ranks that file above the `rustup default <label>` that
  `dtolnay/rust-toolchain` sets (action.yml L111-112 at df2a88e9), so every unqualified
  `cargo`/`rustc` in a job runs 1.85.0 whatever the job installed: the build matrix lanes `stable`
  and `beta` (ci.yml L66-128), the `fmt` job "pinned 1.95.0" (L139-177; its own comment L134-135
  says "A committed rust-toolchain.toml would override this pin too ... keep the repo free of
  one"), and `vectors-roundtrip` "on stable" (L197-198). The `1.85` lane runs 1.85.0 rather than
  the 1.85.x the action resolves (no practical difference). A stable or beta lane that passes says
  nothing about its labelled compiler.
- **bh-mk-I12.** ci.yml `release-on-tag`, step "Create GitHub release if missing" (L390-401):
  `gh release create "$TAG" --generate-notes || true` (L400). On an `mk-codec-v*` tag nothing after
  it touches the release, so the job is green with no release. The same `|| true` ends the
  ensure-release commands of `musl-binaries.yml` L350 and `release.yml` L458 (triage note 3); there
  a later `gh release upload` fails loud, so they are not a green-without-release gate, but the
  error then names the upload instead of the missing release. All three are fixed the same way
  (one idiom; the release.yml header L13-16 says the three workflows share it).
- **bh-mk-I13.** `docs/verify-reproducibility.md` section 9.3 (L304-321), the printed aarch64 recipe:
  (a) `SOURCE_DATE_EPOCH=$(...)` is a prefix assignment of the `cross` command only, so the later
  `tar --mtime="@$SOURCE_DATE_EPOCH"` expands an unset (or stale) caller variable: tar prints
  `Substituting -9223372036854775807 for unknown date format '@'` and writes a member mtime that is
  not the commit time (an archive that cannot reproduce the published hash); (b) nothing stops on
  failure: when `cross` fails the `tar | gzip` pipeline still writes `mk-<VER>-aarch64-linux-musl.tar.gz`
  with no member and the recipe's status is 0 (gzip's).

## 2. Decided behaviour

### 2.1 Every job runs the compiler it is labelled with (I11)

In ci.yml, each job that installs a toolchain with `dtolnay/rust-toolchain` (`build`, `fmt`,
`vectors-roundtrip`):

- the install step gets `id: toolchain` (the `vectors-roundtrip` step, today a bare
  `- uses: dtolnay/rust-toolchain@stable`, also gets `name: Install Rust stable`);
- directly after it, before `Swatinem/rust-cache` (so the cache key uses the lane's `rustc -vV`), two
  steps, identical text in all three jobs, both with `if: steps.docs.outputs.docs_only != 'true'`,
  `shell: bash` (Windows lanes default to pwsh) and `env: LANE_TOOLCHAIN: ${{ steps.toolchain.outputs.name }}`
  (the action's documented output "suitable for use with `cargo +...`"; passed through `env`, not
  spliced into the script):
  1. `name: Use the lane's toolchain, not rust-toolchain.toml`, `run: echo "RUSTUP_TOOLCHAIN=$LANE_TOOLCHAIN" >> "$GITHUB_ENV"`.
     RUSTUP_TOOLCHAIN outranks the file for every later step (nextest, doc-tests, clippy, rustfmt).
  2. `name: Check the lane runs its labelled compiler`, a `run: |` block that sets
     `got=$(rustc --version)` and `want=$(rustc +"$LANE_TOOLCHAIN" --version)`, prints
     `lane $LANE_TOOLCHAIN runs: $got`, and when they differ prints
     `::error::lane $LANE_TOOLCHAIN runs '$got', not '$want'` and exits 1. This is the consultant's
     "log/assert the resolved compiler" regression: if step 1 is ever dropped, every lane whose
     label is not the file's pin goes red here.
- A three-line comment above step 1 says why (PROPOSED wording): "rust-toolchain.toml (the release
  pin) outranks the `rustup default` the action sets, so without this step every lane would run
  the file's compiler (bh-mk-I11). RUSTUP_TOOLCHAIN outranks the file."
- `rust-toolchain.toml` stays (the reproducible musl build and `cargo vendor` rely on 1.85.0;
  `musl-binaries.yml`, `freebsd-compile-gate` and `musl-check` install 1.85.0, which equals the
  file, and are unchanged). The fmt-job comment L130-137 sentence "A committed rust-toolchain.toml
  would override this pin too (rustup file-beats-default precedence) — keep the repo free of one."
  is replaced by (PROPOSED): "The repo's rust-toolchain.toml (the 1.85.0 release pin) would
  override this pin too (rustup file-beats-default precedence), so the job sets RUSTUP_TOOLCHAIN
  from the installed toolchain (bh-mk-I11)." The header comment L5-6 is unchanged.
  `fuzz/rust-toolchain.toml` L6-14 says "the repo deliberately has no ROOT rust-toolchain.toml"
  and that the fmt job "still sees no toolchain file"; both are false since e3c3984. Those nine
  comment lines are replaced by exactly (PROPOSED):
  `# INTENTIONAL. The repo ROOT also has a rust-toolchain.toml (1.85.0, the` /
  `# release-build pin); CI jobs at the root override it with RUSTUP_TOOLCHAIN set` /
  `# from the toolchain they install (see the fmt job in .github/workflows/ci.yml,` /
  `# bh-mk-I11). rustup uses the nearest file upward from the cwd, so inside fuzz/` /
  `# this fuzz/-SCOPED file wins and the root file never applies. Do NOT "promote"` /
  `# this file to the repo root.` (six lines; L1-5 and L15-23 unchanged; fuzz-smoke.yml calls
  `cargo +nightly-2026-04-27` in `fuzz/`, so it is unaffected).
- Out of scope, unchanged: `release-on-tag` step "Install Rust (stable) for man-tarball build"
  (L403-405) also builds with 1.85.0, but it is not a gate (it renders man pages); a FOLLOWUPS Nit
  is proposed in the author's report.

### 2.2 Stable lanes block; the beta lane is labelled non-blocking (T45)

- `build` job: add `continue-on-error: ${{ matrix.toolchain == 'beta' }}` (job level), and the
  name becomes
  `build (${{ matrix.toolchain }} on ${{ matrix.os }}${{ matrix.toolchain == 'beta' && ', non-blocking' || '' }})`,
  so beta checks read e.g. `build (beta on ubuntu-latest, non-blocking)` (PROPOSED label text,
  architect confirms). The `CHECK_NAME` env of the docs-only step (L88) carries the same expression,
  character for character (ci/docs-only.sh looks its own check run up by that name).
- A two-line comment above the name (PROPOSED): "T45 (bh-mk-I11): stable and 1.85 lanes block; the
  beta lane is informational, labelled in its name and allowed to fail."
- Stable and 1.85 lanes: unchanged blocking behaviour (a failure fails the workflow run). Branch
  protection is unchanged: its one required context `build (stable on ubuntu-latest)` keeps its
  name (CLAUDE.md L89). The beta lane is kept (T45 permits keeping or dropping it; keeping is the
  smaller change; PROPOSED, architect confirms).
- Consequence decided here: stable now runs rustc 1.98.0 locally, and its clippy fails on one lint
  (section 4). The FIX fixes that lint; it adds no `#[allow]`.

### 2.3 A release step fails when no release exists (I12)

The command that creates the release ends `|| gh release view <tag> >/dev/null` instead of
`|| true`, at all three sites (exact texts):

- ci.yml L400: `gh release create "$TAG" --generate-notes || gh release view "$TAG" >/dev/null`,
  with a comment above it (PROPOSED): "A failed create is fine only when another tag workflow
  created the release meanwhile (bh-mk-I12); otherwise this step fails."
- musl-binaries.yml L350:
  `run: gh release view "$REF_NAME" >/dev/null 2>&1 || gh release create "$REF_NAME" --generate-notes || gh release view "$REF_NAME" >/dev/null`
- release.yml L458 (also folds the stray run of spaces after the first `||`):
  `gh release view "$TAG" >/dev/null 2>&1 || gh release create "$TAG" --title "$TAG" --generate-notes || gh release view "$TAG" >/dev/null`

Behaviour: release exists -> 0, no create; create succeeds -> 0; create fails but another workflow
created it meanwhile (the race the `|| true` was hiding) -> 0; create fails and the release still
does not exist -> the final `gh release view` fails, its own stderr (not silenced) is in the log,
and the step fails (`bash -e`). Stdout of the final view is discarded as in the first.

### 2.4 The aarch64 recipe stops on failure and uses one epoch (I13)

Section 9.3's fence (L307-321) is replaced by exactly (PROPOSED, printed text per B8):

```sh
(
  set -euo pipefail
  SOURCE_DATE_EPOCH=$(git show -s --format=%ct <tagged-commit-SHA>)
  export SOURCE_DATE_EPOCH
  LC_ALL=C TZ=UTC \
  CARGO_BUILD_RUSTFLAGS="--remap-path-prefix=/project=/build --remap-path-prefix=/cargo=/cargo" \
  CFLAGS="-ffile-prefix-map=/project=/build -ffile-prefix-map=/cargo=/cargo" \
  CFLAGS_aarch64_unknown_linux_musl="-ffile-prefix-map=/project=/build -ffile-prefix-map=/cargo=/cargo" \
  cross build --locked --offline --release \
    --target aarch64-unknown-linux-musl -p mk-cli --bin mk \
    --config 'source.crates-io.replace-with="vendored-sources"' \
    --config 'source.vendored-sources.directory="vendor"'
  tar --sort=name --owner=0 --group=0 --numeric-owner --mtime="@$SOURCE_DATE_EPOCH" \
    -cf - -C target/aarch64-unknown-linux-musl/release mk \
    | gzip -n -9 > mk-<VER>-aarch64-linux-musl.tar.gz
)
```

and directly after the fence, before the existing "The same TWO-block ..." paragraph, a new
paragraph (PROPOSED):

> The parentheses run the recipe in a subshell with `set -euo pipefail`: a failed
> build stops it with a nonzero status before any archive is written, and the
> shell you paste it into keeps its own options. `SOURCE_DATE_EPOCH` is set and
> exported once, so `cross` and `tar` use the same commit time.

Why this shape: a bare `set -euo pipefail` pasted into an interactive shell (the triage's scratch
fix) would close the reader's terminal on the first failure; the subshell confines it. The
assignment and `export` are two lines because `export X=$(cmd)` hides a failing `git show`.
`cross` still receives the epoch and flags in its environment (Cross.toml `[build.env]`
passthrough), as today. The x86_64 recipe (section 4) already runs `bash -euxo pipefail` inside the
container and is unchanged.

## 3. Spec delta

No SPEC file changes. The printed recipe in `docs/verify-reproducibility.md` 9.3 is the only
holder-facing text (2.4).

## 4. Code (anchors at 9b0b365; builder writes the change)

- `.github/workflows/ci.yml`: 2.1 (L92-97 build install, L168-173 fmt install, L197-198
  vectors-roundtrip install; L130-137 comment), 2.2 (L66 name, job-level `continue-on-error`,
  L88 `CHECK_NAME`), 2.3 (L400).
- `.github/workflows/musl-binaries.yml` L350, `.github/workflows/release.yml` L458: 2.3.
- `docs/verify-reproducibility.md` L307-321 plus the new paragraph: 2.4.
- `fuzz/rust-toolchain.toml` L6-14 comment: 2.1.
- `crates/mk-cli/src/cmd/encode.rs` L147 and L160: clippy 1.98 `unnecessary_unwrap`
  (`key_source.expect("guarded by the enclosing if").1` after `if key_source.is_some()`). Replace
  `if key_source.is_some() {` with `if let Some((_, given)) = key_source {` and delete the L160
  `let given = ...;` line. No behaviour change (`key_source` is `Option<(&str, &str)>`, Copy; its
  later use at L426 is unaffected). This is the only clippy finding on stable 1.98.0 at the base
  (`cargo +stable clippy --locked --keep-going --workspace --all-targets -- -D warnings`; with the
  change, rc 0 on 1.98.0 and on 1.85.0).
- `scripts/mk-fg-ci-gates-rows.sh` (committed with this document) and its `SCRIPTS.md` row; the
  `ci/code-docs-exempt.txt` line for it (it reads `docs/verify-reproducibility.md` and the
  workflows but is not run by CI).

Files this FIX changes (for build ordering): `.github/workflows/ci.yml`,
`.github/workflows/musl-binaries.yml`, `.github/workflows/release.yml`,
`docs/verify-reproducibility.md`, `fuzz/rust-toolchain.toml`, `crates/mk-cli/src/cmd/encode.rs`,
plus (already on this branch) `scripts/mk-fg-ci-gates-rows.sh`, `SCRIPTS.md`,
`ci/code-docs-exempt.txt`, `design/FIX_mk_fg_ci_gates.md`.

## 5. Rows

Committed runner: `scripts/mk-fg-ci-gates-rows.sh red|green` (its header lists what it does and
does not cover). It reads the real workflow and doc text of the checkout: I11 rows emulate each
lane in a scratch RUSTUP_HOME (whose `toolchains/` links to yours; nothing is installed), apply the
job's own two steps (2.1) and compare unqualified `rustc --version` with `rustc +<label>`, and run
the check step again without the pin step, which must fail; I12 rows
run each site's command under `bash -e` with a mocked `gh`; I13 rows run the 9.3 fence in a scratch
git repo (HEAD committed 2020-01-01, epoch 1577836800) with `cross` mocked. Scratch under
`target/mk-fg-rows` (override `WORK`).

Runs: `red` at the base -> `ALL ROWS MATCH red` (12 rows); `green` with the scratch implementation
of sections 2 and 4 (applied, run, reverted; patch kept at
/scratch/cache/mk-fg-red/scratch-green.patch) -> `ALL ROWS MATCH green` (12 rows).

| row | what | RED at 9b0b365 (seen) | GREEN (seen on the scratch fix) |
|---|---|---|---|
| `i11_build_lane_runs_label` | build lane, label `stable` | `unqualified: rustc 1.85.0 (4d91de4e4 2025-02-17) \| labelled: rustc 1.98.0 (88d9e12ae 2026-08-18) \| wired=0 check-step=absent check-without-pin=absent` | `unqualified: rustc 1.98.0 ... \| labelled: rustc 1.98.0 ... \| wired=1 check-step=0 check-without-pin=1` |
| `i11_fmt_lane_runs_label` | fmt job, label `1.95.0` | `unqualified: rustc 1.85.0 ... \| labelled: rustc 1.95.0 (59807616e 2026-04-14)`, wired=0, absent, check-without-pin=absent | both `rustc 1.95.0`, wired=1, check-step=0, check-without-pin=1 |
| `i11_vectors-roundtrip_lane_runs_label` | vectors-roundtrip, label `stable` | `unqualified: rustc 1.85.0 ... \| labelled: rustc 1.98.0`, wired=0, absent, check-without-pin=absent | both `rustc 1.98.0`, wired=1, check-step=0, check-without-pin=1 |
| `i11_beta_lane_labelled_non_blocking` | build job name, `continue-on-error`, `CHECK_NAME` (static) | name `build (${{ matrix.toolchain }} on ${{ matrix.os }})`, continue-on-error empty | name and CHECK_NAME = the 2.2 expression, continue-on-error `${{ matrix.toolchain == 'beta' }}` |
| `i12_ci_release_on_tag_fails_without_release` | ci.yml step block, 4 gh cases | `exists rc=0 (creates=0) created rc=0 raced rc=0 still-missing rc=0` | `... raced rc=0 still-missing rc=1` |
| `i12_musl_ensure_release_fails_without_release` | musl-binaries.yml L350 | same as above, still-missing rc=0 | still-missing rc=1 |
| `i12_release_yml_ensure_release_fails_without_release` | release.yml L458 | same, still-missing rc=0 | still-missing rc=1 |
| `i13_recipe_build_ok_epoch_from_commit` | cross ok, caller has no epoch | `rc=0 member_mtime=none` (tar: `Substituting -9223372036854775807 for unknown date format '@'`) | `rc=0 member_mtime=1577836800`, cross saw epoch 1577836800 and the remap flags |
| `i13_recipe_ignores_stale_caller_epoch` | caller exports `SOURCE_DATE_EPOCH=1` | `rc=0 member_mtime=1` | `member_mtime=1577836800` |
| `i13_recipe_build_fails_nonzero_no_archive` | cross returns 23 | `rc=0 archive_exists=yes` | `rc=23 archive_exists=no` |
| `i13_recipe_missing_binary_nonzero` | cross returns 0, no binary | `rc=0` | `rc=2` (tar) |
| `i13_recipe_reports_failure_to_caller_without_set_e` | the caller's `$?` and `$-` after a failed build | `after rc=0 opts=hBc` | `after rc=23 opts=hBc` (no `e`: nothing leaks) |

Commands as run (repo root, `PATH="$HOME/.cargo/bin:$PATH"`):
`bash scripts/mk-fg-ci-gates-rows.sh red` (base) and `bash scripts/mk-fg-ci-gates-rows.sh green`
(fix). Also run at the base, outside the rows: `cargo +stable clippy --locked --keep-going
--workspace --all-targets -- -D warnings` -> rc 101, one error at encode.rs:160:29
(`clippy::unnecessary_unwrap`); `cargo +stable nextest run --locked --workspace --no-fail-fast`
-> 470 passed; `cargo +stable test --locked --workspace --doc` -> rc 0; `cargo +1.95.0 fmt --check
--all` -> rc 0. With the fix: stable clippy rc 0, 1.85.0 clippy rc 0, `actionlint` on the three
workflows clean (as at the base).

## 6. Tests that change

None. No Rust test reads the workflows or the 9.3 recipe; the encode.rs change is behaviour-neutral
(the `encode` refusal texts are unchanged; the full suite confirms). The rows script is the test
for this FIX (CI/doc rows, not CLI rows). The beta lane and the hosted runner behaviour are
confirmed on the PR's own CI run: the builder checks in that run's logs that each lane's "Check
the lane runs its labelled compiler" step printed `lane stable runs: rustc 1.<N>...` (the hosted
stable), `lane beta runs: rustc 1.<N+1>.0-beta...`, `lane 1.85 runs: rustc 1.85.<x>`, `lane 1.95.0
runs: rustc 1.95.0`, and records them in the PR description.

## 7. Toolkit manual lockstep

None (no `mk` flag or printed CLI text changes).

## 8. FOLLOWUPS (listed in the author's report, not appended here)

Lines for `design/FOLLOWUPS.md` (to be direct-pushed after merge) are in the author's report.

## 9. Gate

`bash scripts/mk-fg-ci-gates-rows.sh green` -> `ALL ROWS MATCH green`, then the full CI set on the
pin and on stable: `cargo nextest run --locked --workspace --no-fail-fast`, `cargo test --locked
--doc --workspace`, `cargo +1.95.0 fmt --all -- --check`, `cargo clippy --locked --workspace
--all-targets -- -D warnings`, `cargo +stable clippy --locked --workspace --all-targets -- -D
warnings`, `actionlint .github/workflows/*.yml`, `bash ci/code-docs-check.sh`,
`bash ci/docs-only.test.sh`, `bash ci/code-docs-check.test.sh`; then the PR's hosted CI green on
every blocking lane, with the section 6 log lines recorded.
