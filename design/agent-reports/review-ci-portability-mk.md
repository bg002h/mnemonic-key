0C/0I/1M/0N

# Review: mk bughunt_mk_03 portability (b407fb8)

Diff `5341e64..b407fb8`: one line, `#[cfg_attr(not(target_os = "linux"), ignore = "needs GNU timeout and ulimit -v")]`. Test-only; verdict: fix is sound.

## Where the test runs (workflows read: ci.yml, release.yml, others ubuntu-only)
- ci.yml `build` matrix (stable/beta/1.85 x ubuntu/macos/windows), `cargo nextest run --workspace --no-fail-fast`: runs on ubuntu; ignored on macOS and Windows (the two failing legs; nextest reports it as skipped, no failure).
- ci.yml musl jobs (x86_64 and aarch64 Linux, nextest -p mk-cli): runs (target_os = linux, musl included).
- release.yml `test` matrix (ubuntu, macos): runs on ubuntu, ignored on macOS.
- vendor-freshness, fuzz-smoke, musl-binaries, other ubuntu jobs: no relevant test run or Linux only.
- No workflow passes --run-ignored/--include-ignored (grep), so the ignore is never overridden.

## Can still fail on Linux
Scratch change: disabled both caps in crates/mk-cli/src/cmd/address.rs (`if false && ...` on the count and range 1,000,000 checks), ran `cargo nextest run --locked -p mk-cli --test bughunt -E 'test(bughunt_mk_03)'` (target dir /scratch/cache/bughunt/mk/target). Result: FAIL at bughunt.rs:138, `--count 2147483648 not refused: code=None stderr="memory allocation of 8589934592 bytes failed"`. File restored with git checkout; tree clean apart from the pre-existing untracked build-mk-bughunt-logs/.

## Coverage
The cap logic is platform-independent (address.rs:215, :244); the only test of it is this one (grep of crates/*/tests for "too large"/1000001 finds nothing else), so macOS/Windows lose no distinct platform coverage, since Linux legs of the same matrix run it. The exact-1,000,000 success path is likewise Linux-run.

## Findings
- M1: the ignore reason is applied by cfg, so on macOS/Windows the 1M-address success half and the refusal half are both skipped, though only the refusal loop needs timeout/ulimit. Splitting the success half into an ungated test would keep portable coverage of the inclusive cap. Optional; cost is ~30 s per OS.

0C/0I/1M/0N
