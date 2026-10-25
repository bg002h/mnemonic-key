r1: 0C/0I/0M/3N

# L3 review r1: mnemonic-key PR #15 (B-key), head 1bde80a0d94495262e8c0c23743e001aa199da34

Question: can the change skip `build`'s build steps on an event whose build
output is used, or on a change that is not documentation, or break the
release (tag) path? **No.** No Critical, no Important.

## Evidence, per brief item

1. **Steps.** `build` has seven steps after the guard (toolchain, cross x2,
   cargo build, smoke test, stage, upload); all are build work or depend on it.
   No attestation, signing or cache step is in this job (all are in `assemble`).
   The only consumer of `mk-*` artifacts is `assemble` ("Download all
   artifacts"). It runs only on a tag ref, a `tag` input, or `sign_dry_run`.
   - Tag push: `GITHUB_EVENT_NAME=push`, ref `refs/tags/…`, so
     `ci/docs-only.sh` takes `*) full "push to …"` and the steps run.
   - Dispatch with `tag`: the guard's `if: github.event.inputs.tag == ''` is
     false, `docs_only` is unset, `!= 'true'` holds, and the steps run.
   - Dispatch with `sign_dry_run` and no tag: the event is
     `workflow_dispatch`, so `*) full "event …"` and the steps run.

   A tag push really does reach the guard step: null `inputs.tag` equals
   `''`. Checked live: main push run 40038565186 logged `docs-only: full run
   (not documentation: .github/workflows/ci.yml)` from the same `if:` in
   `release test`.
2. **CHECK_NAME.** `check_name` is an exact-name filter, and `CHECK_NAME` is
   `build mk (<matrix.name>)`, so each leg looks up only its own name. No other
   workflow's job name expands to `build mk (…)`. I grepped every job `name:`
   in `.github/workflows/*.yml`: `ci.yml` has `build (… on …)`, `musl-binaries`
   has `musl-binary (…)`, and `fuzz-smoke` has `cargo fuzz build …`. A leg that
   is skipped at job level concludes `skipped`, which `passed()` rejects. A
   docs-skipped leg concludes `success`, and that is the induction Part A
   settled.
3. **Permissions.** Checkout needs `contents: read`, and so does A′'s
   compare. The check-runs lookup needs `checks: read`. `upload-artifact@v4`
   uses the runtime token, not `GITHUB_TOKEN` scopes. No step in `build` needs
   `contents: write`, `id-token` or `attestations`, and `assemble` keeps the
   workflow-level grant. L1's code head 09dd55f (run 40230917184) was green on
   all three legs with the reduced grant.
4. **Tag suffix and checkout `ref:`, per event:**

   | event | checkout | check-run name | guard |
   | --- | --- | --- | --- |
   | pull_request | `github.sha` (test-merge; A′ needs its two parents, confirmed by L2 run 40231501663) | `build mk (x)` | PR rules |
   | push main / `ci/**` | `github.sha` (was `github.ref`, which could race to a newer tip than `PUSH_BEFORE..HEAD` assumed: an improvement) | `build mk (x)` | push rules |
   | push `refs/tags/mk-cli-v*` | `github.ref` (unchanged) | `build mk (x) (tag)` | full |
   | dispatch, `tag` set | `inputs.tag` (unchanged) | `… (tag)` | guard skipped, so full |
   | dispatch, no `tag`, branch ref | `github.sha` (same commit as the old `github.ref`) | `build mk (x)` | full |
   | dispatch, no `tag`, tag ref | `github.ref` (tag) | `… (tag)` | full |

   A ` (tag)` run never matches the unsuffixed `CHECK_NAME`. Every run that
   carries the unsuffixed name built the very commit its check run attaches
   to. Before this PR, a tag-backfill dispatch attached `build mk (x)` to the
   branch head while building the tag's tree; this PR closes that gap. The
   reverse cannot happen either: tag runs never skip, so an untagged run never
   stands in for one.
5. **PR description:** accurate, and it says it changes `.github/workflows/`.
   One nit below.

## Nits

- N1: The PR description says "the release build jobs" and "Each job now
  has". Only one job (`build`) changes in this repo.
- N2: SPEC-after-b §6.2 says "first default-branch push after merge runs in
  full". That does not apply to B-key's untagged names, which are unchanged.
  The main tip 8853070 already has `build mk (*)` completed/success (API
  checked), so the first docs-only push after the merge may skip at once.
  That is correct behaviour; only the spec's wording is off.
- N3: Before this PR, tag-backfill dispatches (main 9ccd549 and 3af71aa;
  sign-releases a49db39 and 13d5b22) left unsuffixed `build mk (*)` runs on
  commits whose tree they did not build. None is the main tip or a live PR
  head, so no A′ or push lookup can reach them today.
