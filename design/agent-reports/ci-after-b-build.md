
# Re-run after: ci: guard the release build jobs with ci/docs-only.sh

PR #15. Copies ci-skip-constellation reference/after-b/B-key (SPEC-after-b.md).

## Change

```
 .github/workflows/release.yml | 30 ++++++++++++++++++++++++------
 1 file changed, 24 insertions(+), 6 deletions(-)
```

## Gate

`make-after-b-reference --check: PASS (12 files)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (93 files scanned, 111 docs files)
PASS B-key .github/workflows/release.yml: exactly the §4.2 edits to build
after-b-check B-key: PASS
```

## L1 (code head 09dd55f8830dfa60236a911aefc90c206d5e3315)

```
run 40230917184    release                        pull_request   success    ok
run 40230917350    CI                             pull_request   success    ok
  release/build mk (macos-aarch64)                 success    ok
  release/release test (ubuntu-latest)             success    ok
  release/release test (macos-latest)              success    ok
  release/build mk (macos-x86_64)                  success    ok
  release/build mk (windows-x86_64)                success    ok
  release/assemble + release                       skipped    ok (allowed skip)
  CI/fmt (pinned 1.95.0)                           success    ok
  CI/build (stable on windows-latest)              success    ok
  CI/musl compile/test (aarch64-unknown-linux-musl) success    ok
  CI/build (beta on ubuntu-latest)                 success    ok
  CI/freebsd compile-gate (whole-crate)            success    ok
  CI/musl compile/test (x86_64-unknown-linux-musl) success    ok
  CI/build (1.85 on ubuntu-latest)                 success    ok
  CI/build (1.85 on macos-latest)                  success    ok
  CI/build (1.85 on windows-latest)                success    ok
  CI/build (beta on macos-latest)                  success    ok
  CI/build (stable on ubuntu-latest)               success    ok
  CI/build (beta on windows-latest)                success    ok
  CI/build (stable on macos-latest)                success    ok
  CI/vectors-roundtrip                             success    ok
  CI/release-on-tag                                skipped    ok (allowed skip)
pr-checks: PASS (mnemonic-key 09dd55f8830dfa60236a911aefc90c206d5e3315)
```

## L2 (docs head 431d0f1cc5ce9b5ce34cadc03554eb68ed2f8f72, previous head 09dd55f8830dfa60236a911aefc90c206d5e3315)

- run 40231501663 job 121284017743 `release/release test (macos-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'release test (macos-latest)' passed on it; skipping the guarded steps`
- run 40231501663 job 121284017884 `release/build mk (macos-aarch64)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build mk (macos-aarch64)' passed on it; skipping the guarded steps`
- run 40231501663 job 121284017928 `release/build mk (windows-x86_64)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build mk (windows-x86_64)' passed on it; skipping the guarded steps`
- run 40231501663 job 121284017973 `release/build mk (macos-x86_64)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build mk (macos-x86_64)' passed on it; skipping the guarded steps`
- run 40231501663 job 121284017975 `release/release test (ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'release test (ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284017912 `CI/musl compile/test (x86_64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'musl compile/test (x86_64-unknown-linux-musl)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284017997 `CI/musl compile/test (aarch64-unknown-linux-musl)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'musl compile/test (aarch64-unknown-linux-musl)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018003 `CI/freebsd compile-gate (whole-crate)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'freebsd compile-gate (whole-crate)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018018 `CI/build (stable on ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build (stable on ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018023 `CI/fmt (pinned 1.95.0)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'fmt (pinned 1.95.0)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018037 `CI/build (1.85 on windows-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build (1.85 on windows-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018055 `CI/build (beta on macos-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build (beta on macos-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018061 `CI/build (beta on ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build (beta on ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018084 `CI/build (stable on windows-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build (stable on windows-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018089 `CI/build (1.85 on ubuntu-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build (1.85 on ubuntu-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018120 `CI/build (stable on macos-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build (stable on macos-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018170 `CI/build (beta on windows-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build (beta on windows-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284018190 `CI/build (1.85 on macos-latest)`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'build (1.85 on macos-latest)' passed on it; skipping the guarded steps`
- run 40231501625 job 121284130801 `CI/vectors-roundtrip`: `docs-only: all 1 files changed since the previous PR head 09dd55f8830dfa60236a911aefc90c206d5e3315 are documentation, the merged base 8853070b7c2b6903be439cbabf55ac5165f7ff50 is an ancestor of it, and 'vectors-roundtrip' passed on it; skipping the guarded steps`
