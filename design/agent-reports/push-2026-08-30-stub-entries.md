# Push report — mnemonic-key `main`, 2026-08-30

## Pre-flight

- `git -C /scratch/code/shibboleth/mnemonic-key status`: clean working tree, branch `main` ahead of `origin/main` by **3 commits**.
- `git rev-list --count origin/main..main` = **3**.
- HEAD SHA: `9fbbe36f31717fff8bb0ac163c77d2b17fb97121`.
- Commits pushed (newest first):
  - `9fbbe36` spec: park chunk_set_id-verification draft -- 2026-08-19 artifact, R0 not started
  - `f5a18a2` followups: stub entry corrected per converter R0 r3 -- no formula restatement (measured false), three-repo lockstep, canonical-origin obligation (M4)
  - `bcd8505` followups: stub-keyed-wallet-binding-at-mint -- the converter R0's C1 upgrade, pre-v1.0 compat-free window (operator ruling)

## Staging ritual

1. `git push origin main:refs/heads/ci/staging` — succeeded, `* [new branch] main -> ci/staging`.
2. CI run located via `gh run list --repo bg002h/mnemonic-key --branch ci/staging`, matched on full 40-char SHA `9fbbe36f31717fff8bb0ac163c77d2b17fb97121`: run id **33327085412**, workflow `CI`.
   `gh run watch 33327085412 --repo bg002h/mnemonic-key --exit-status` run to completion in the foreground; exit code 0.
   Per-job conclusions (via `gh run view 33327085412 --json jobs`):
   - musl compile/test (x86_64-unknown-linux-musl): success
   - build (stable on macos-latest): success
   - build (1.85 on ubuntu-latest): success
   - build (beta on ubuntu-latest): success
   - musl compile/test (aarch64-unknown-linux-musl): success
   - build (stable on windows-latest): success
   - build (beta on windows-latest): success
   - build (stable on ubuntu-latest): success
   - fmt (pinned 1.95.0): success
   - build (1.85 on windows-latest): success
   - build (1.85 on macos-latest): success
   - build (beta on macos-latest): success
   - freebsd compile-gate (whole-crate): success
   - vectors-roundtrip: success
   - release-on-tag: skipped (expected — not a tag push)

   All 14 substantive jobs green, 0 failures.
3. `git push origin main` — output: `93cebfb..9fbbe36  main -> main`. **No "Bypassed rule violations" message** — the push was satisfied by the CI check on the staged SHA, not bypassed.
4. `git push origin --delete ci/staging` — output: `- [deleted]         ci/staging`.

## Result

Ritual completed exactly as documented, no deviations, no bypass message. `main` on `origin` now points at `9fbbe36`. Staging ref cleaned up.

## Second push (doc fix)

### Pre-flight

- `git -C /scratch/code/shibboleth/mnemonic-key status`: clean working tree, branch `main` ahead of `origin/main` by **2 commits**.
- HEAD SHA: `40efcc7a592b1b47c6071672b07cadf0b8b0c4ae`.
- Commits pushed (newest first):
  - `40efcc7` docs: encode() doc caught up to 0.5.0 -- chunk_set_id is derived, not CSPRNG-drawn
  - `c41770a` report: persist push-agent report -- stub entries + parked draft pushed via staging, 14/14 green

### Staging ritual

1. `git push origin main:refs/heads/ci/staging` — succeeded, `* [new branch] main -> ci/staging`.
2. Two workflows fired for this push, both matched on the full 40-char SHA `40efcc7a592b1b47c6071672b07cadf0b8b0c4ae` (verified via `gh run view --json headSha` before watching):
   - `CI`, run id **33328011150** — `gh run watch 33328011150 --repo bg002h/mnemonic-key --exit-status` run to completion in the foreground, exit code 0. Per-job conclusions (`gh run view --json jobs`): all 14 substantive jobs `success` (freebsd compile-gate, fmt pinned 1.95.0, build × 10 (stable/beta/1.85 × ubuntu/macos/windows), musl compile/test × 2, vectors-roundtrip), `release-on-tag`: `skipped` (expected — not a tag push).
   - `fuzz-smoke`, run id **33328011216** — watched to completion, exit code 0 (`gh run watch` reported "already completed with 'success'" by the time it was checked). Jobs: `cargo fuzz build (compile gate)`: `success`; `cargo fuzz run (60s smoke)`: `skipped`. Verified the skip is expected and not a hidden failure by reading `.github/workflows/fuzz-smoke.yml` lines 74-77 — the `smoke` job carries `if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'`, deliberately excluding `push`/`pull_request` ("off the critical path"); the `build` compile-gate job has no such restriction and ran normally.

   All required jobs green, 0 failures, 0 unexplained skips.
3. `git push origin main` — output: `9fbbe36..40efcc7  main -> main`. **No "Bypassed rule violations" message** — satisfied by the CI check on the staged SHA.
4. `git push origin --delete ci/staging` — output: `- [deleted]         ci/staging`.

### Result

Ritual completed exactly as documented, no deviations, no bypass message. `main` on `origin` now points at `40efcc7`. Staging ref cleaned up.
