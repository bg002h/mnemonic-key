# Push cycle report — mnemonic-key + descriptor-mnemonic + seedhammer

Executed 2026-08-31 (session date), all three repos pushed in order, all GREEN.
Controller held a commit freeze for the whole window; no repo tip moved during
staging.

## Repo 1 — mnemonic-key

- SHA pushed: `8ae8dd41a668f65cec69cb1b8a0626690fab3fee`
- `git rev-parse HEAD` before staging matched the required tip exactly; tree
  clean.
- Staged: `git push origin main:refs/heads/ci/staging` → new branch created.
- Staging CI run: workflow `CI`, run id **33458632887**, on SHA
  `8ae8dd41a668f65cec69cb1b8a0626690fab3fee`. `gh run watch --exit-status`
  returned success. Full job-conclusion list (`gh run view --json jobs`):

  | job | conclusion |
  | --- | --- |
  | musl compile/test (x86_64-unknown-linux-musl) | success |
  | fmt (pinned 1.95.0) | success |
  | build (beta on macos-latest) | success |
  | build (stable on macos-latest) | success |
  | build (beta on ubuntu-latest) | success |
  | freebsd compile-gate (whole-crate) | success |
  | build (stable on windows-latest) | success |
  | build (1.85 on windows-latest) | success |
  | build (beta on windows-latest) | success |
  | build (stable on ubuntu-latest) | success (required) |
  | build (1.85 on ubuntu-latest) | success |
  | build (1.85 on macos-latest) | success |
  | musl compile/test (aarch64-unknown-linux-musl) | success |
  | vectors-roundtrip | success |
  | release-on-tag | skipped (expected — no tag) |

  Run `conclusion` == `success`. A second workflow, `fuzz-smoke` (run id
  33458632938), also triggered on the same SHA and also concluded `success`.
  No failures anywhere on the SHA.
- Final push: `git push origin main` → output:
  `   40efcc7..8ae8dd4  main -> main` — **no "Bypassed rule violations" line**
  (proof the required-context check on the SHA was what admitted it, not an
  admin bypass).
- Staging cleanup: `git push origin --delete ci/staging` → `- [deleted]
  ci/staging`.
- Post-push `git rev-parse origin/main`:
  `8ae8dd41a668f65cec69cb1b8a0626690fab3fee` (matches).

## Repo 2 — descriptor-mnemonic

- SHA pushed: `d41950fe10f7a4bb4c47ff56ab34da2cdf3bb4a9`
- `git rev-parse HEAD` before staging matched the required tip exactly; tree
  clean.
- Staged: `git push origin main:refs/heads/ci/staging` → new branch created.
- Staging CI run: workflow `CI`, run id **33459077932**, on SHA
  `d41950fe10f7a4bb4c47ff56ab34da2cdf3bb4a9`. `gh run watch --exit-status`
  returned success. Full job-conclusion list (`gh run view --json jobs`):

  | job | conclusion |
  | --- | --- |
  | cargo doc | success |
  | freebsd compile-gate (whole-crate) | success |
  | musl compile/test (aarch64-unknown-linux-musl) | success |
  | cargo test (ubuntu-latest) | success (required) |
  | cargo clippy | success (required) |
  | musl compile/test (x86_64-unknown-linux-musl) | success |
  | cargo fmt | success |
  | cargo test (windows-latest) | success |
  | cargo test (macos-latest) | success |

  Run `conclusion` == `success`. Confirmed via `gh run list ... -q 'select
  headSha=="d41950fe..."'` that `CI` was the only workflow that ran against
  this exact SHA — no other run to check.
- Final push: `git push origin main` → output:
  `   0ce18660..d41950fe  main -> main` — **no "Bypassed rule violations"
  line**.
- Staging cleanup: `git push origin --delete ci/staging` → `- [deleted]
  ci/staging`.
- Post-push `git rev-parse origin/main`:
  `d41950fe10f7a4bb4c47ff56ab34da2cdf3bb4a9` (matches).

## Repo 3 — seedhammer (unprotected)

- SHA pushed: `195df90b1a5fcb77ad74101107c18e063c02ecff`
- `git rev-parse HEAD` before push matched the required tip exactly; tree
  clean.
- Direct push (repo not protected, no staging ritual required):
  `git push origin main` → output: `   5f02773..195df90  main -> main`.
- Post-push `git rev-parse origin/main`:
  `195df90b1a5fcb77ad74101107c18e063c02ecff` (matches).
- Noted (informational only, not gating per the instructions for this
  unprotected repo): two workflows (`Test`, `Build image`) started on this SHA
  immediately after push (run ids 33459508442, 33459508463), both
  `in_progress` at observation time — not watched to completion since a direct
  push to this repo needs no CI gate.

## Outcome

All three repos pushed successfully, in order, with no early stop. No force
push, no `enforce_admins` change, no "Bypassed rule violations" on either
protected repo's final push. Nothing committed by this run (report file
committed separately by the controller, per standing rule).
