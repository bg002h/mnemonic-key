
# Re-run after: ci: skip a docs-only push to a PR that already passed (rule A')

PR #14. Applies ci-skip-constellation SPEC.md (GREEN at e8f831d), Part A.

## Change

```
 .github/workflows/ci.yml      |   6 ++
 .github/workflows/release.yml |  13 +++-
 ci/code-docs-check.sh         | 165 ++++++++++++++++++++++++++++++++++++++++++
 ci/code-docs-check.test.sh    | 128 ++++++++++++++++++++++++++++++++
 ci/code-docs-exempt.txt       |   4 +
 ci/docs-only-paths.txt        |   5 ++
 ci/docs-only.sh               | 158 +++++++++++++++++++++++++++++-----------
 ci/docs-only.test.sh          | 103 ++++++++++++++++++++++++--
 ci/shared-ci.sha256           |   4 +
 9 files changed, 534 insertions(+), 52 deletions(-)
```

## Gate

`gate-public: 0 failure(s)`

## Local checks

```
ci/docs-only.sh: OK
ci/docs-only.test.sh: OK
ci/code-docs-check.sh: OK
ci/code-docs-check.test.sh: OK
docs-only.test.sh: 73 passed, 0 failed
code-docs-check.test.sh: 41 passed, 0 failed
code-docs check: ok (93 files scanned, 109 docs files)
```

## L1 (code head dc7563dfa630c014466b8d4f7db8b80d0cc26a90)

```
run 40036476083    release                        pull_request   success    ok
run 40036477116    CI                             pull_request   success    ok
  release/release test (macos-latest)              success    ok
  release/build mk (macos-aarch64)                 success    ok
  release/release test (ubuntu-latest)             success    ok
  release/build mk (macos-x86_64)                  success    ok
  release/build mk (windows-x86_64)                success    ok
  release/assemble + release                       skipped    ok (allowed skip)
  CI/freebsd compile-gate (whole-crate)            success    ok
  CI/musl compile/test (x86_64-unknown-linux-musl) success    ok
  CI/fmt (pinned 1.95.0)                           success    ok
  CI/musl compile/test (aarch64-unknown-linux-musl) success    ok
  CI/build (stable on ubuntu-latest)               success    ok
  CI/build (1.85 on windows-latest)                success    ok
  CI/build (beta on ubuntu-latest)                 success    ok
  CI/build (stable on windows-latest)              success    ok
  CI/build (1.85 on ubuntu-latest)                 success    ok
  CI/build (beta on windows-latest)                success    ok
  CI/build (stable on macos-latest)                success    ok
  CI/build (1.85 on macos-latest)                  success    ok
  CI/build (beta on macos-latest)                  success    ok
  CI/vectors-roundtrip                             success    ok
  CI/release-on-tag                                skipped    ok (allowed skip)
pr-checks: PASS (mnemonic-key dc7563dfa630c014466b8d4f7db8b80d0cc26a90)
```
