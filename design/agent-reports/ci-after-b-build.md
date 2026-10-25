
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
