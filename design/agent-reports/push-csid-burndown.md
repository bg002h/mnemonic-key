# Push burndown — 4-repo CI-staging ritual (2026-08-31)

All four repos pushed in order. No stops; no force; no `enforce_admins` changes;
no `--no-verify`/`--no-gpg-sign`. Nothing committed by the controller (trees
were clean per the freeze).

## 1 — mnemonic-key (branch `main`)

- SHA pushed: `1602e416b5b444cc37cb38f03f107be3cc72a84b`
- Staging run: `33464117878` (workflow `CI`, `ci/staging`)
- Job conclusions (15 total, all success/skipped):
  - `musl compile/test (aarch64-unknown-linux-musl)`: success
  - `build (stable on macos-latest)`: success
  - `freebsd compile-gate (whole-crate)`: success
  - `fmt (pinned 1.95.0)`: success
  - `musl compile/test (x86_64-unknown-linux-musl)`: success
  - `build (stable on ubuntu-latest)`: success (required)
  - `build (beta on windows-latest)`: success
  - `build (1.85 on ubuntu-latest)`: success
  - `build (stable on windows-latest)`: success
  - `build (beta on ubuntu-latest)`: success
  - `build (1.85 on macos-latest)`: success
  - `build (beta on macos-latest)`: success
  - `build (1.85 on windows-latest)`: success
  - `vectors-roundtrip`: success
  - `release-on-tag`: skipped (tag-gated, expected)
  - Run conclusion: `success`
- Final push `git push origin main`: `8ae8dd4..1602e41  main -> main` — no
  "Bypassed rule violations" line.
- Staging cleanup: `ci/staging` deleted.
- `git rev-parse origin/main` after: `1602e416b5b444cc37cb38f03f107be3cc72a84b`

## 2 — descriptor-mnemonic (branch `main`)

- SHA pushed: `54ab1cd606ed98140508ea7e2736e0764d63fd83`
- Staging run: `33464474816` (workflow `CI`, `ci/staging`)
- Job conclusions (9 total, all success):
  - `cargo doc`: success
  - `musl compile/test (aarch64-unknown-linux-musl)`: success
  - `cargo clippy`: success (required)
  - `cargo test (ubuntu-latest)`: success (required)
  - `cargo fmt`: success
  - `freebsd compile-gate (whole-crate)`: success
  - `musl compile/test (x86_64-unknown-linux-musl)`: success
  - `cargo test (windows-latest)`: success
  - `cargo test (macos-latest)`: success
  - Run conclusion: `success`
- Final push `git push origin main`: `d41950fe..54ab1cd6  main -> main` — no
  "Bypassed rule violations" line.
- Staging cleanup: `ci/staging` deleted.
- `git rev-parse origin/main` after: `54ab1cd606ed98140508ea7e2736e0764d63fd83`

## 3 — seedhammer (branch `main`, NOT protected)

- SHA pushed: `2337ed3cdeed03c1bc689e2c986919d72e0907ff`
- No staging ritual (unprotected branch). Direct `git push origin main`:
  `195df90..2337ed3  main -> main`
- `git rev-parse origin/main` after: `2337ed3cdeed03c1bc689e2c986919d72e0907ff`

## 4 — mnemonic-engrave (branch `master`)

- SHA pushed: `3d570c3fec86ef1204310737f674c27f5bcf8a25`
- Staging run: `33464870319` (workflow `release`, `ci/staging`)
- Job conclusions (8 total, all success/skipped):
  - `build me-preview (all targets)`: success
  - `build me (linux-x86_64)`: success
  - `build me (macos-x86_64)`: success
  - `build me (windows-x86_64)`: success
  - `build me (linux-aarch64)`: success
  - `test (rust + go)`: success (required)
  - `build me (macos-aarch64)`: success
  - `assemble + sign + release`: skipped (tag-gated, expected — confirms it did
    not run off a `ci/**` push)
  - Run conclusion: `success`
- Final push `git push origin master`: `9144245..3d570c3  master -> master` —
  no "Bypassed rule violations" line.
- Staging cleanup: `ci/staging` deleted.
- `git rev-parse origin/master` after: `3d570c3fec86ef1204310737f674c27f5bcf8a25`

## Outcome

All four repos landed on their default branch with a clean, whole-suite-green
staging run backing each protected push. No stop condition was hit.
