# Review: mk baseline GAP-B hardening (PR #27), Claude Sonnet 5.5 (claude-sonnet-5-5), medium

Base 95d70a5a83e115530a2de6877d50e021f9fe2620, head 414d491c.

1. Scope: the diff changes src/slip132.rs and src/bytecode/path.rs, but only inside `#[cfg(test)] mod tests` (additions only). No other production lines. OK.
2. Tests-can-fail: I re-applied all 15 mutants as scratch edits (path `>`->`==`, `>`->`>=`; inspect.rs:74 ""/xyzzy; mod.rs:355 ""/xyzzy; mod.rs:347 `==`->`!=`; repair.rs:309 arm->false and `==`->`!=`; verify.rs:99 guard true/false and `==`->`!=`; verify.rs:118 `!=`->`==`; slip132.rs:70 `&&`->`||`). Each named test FAILED (nextest summary "0 passed, 1 failed", or 2 failed for the stub pair). All reverted (git status clean). Full workspace nextest at head: 423 passed.
3. Expected values: XPUB_FP abc63537 independently recomputed (hash160 of the xpub pubkey, Python): matches. Stub/paths come from the committed fixture, not echoed from the code under test.
4. Robustness: no timing or randomness; new tests run in ~0.01 s each (debug).
5. repair.rs:309 deviation: confirmed sound. SINGLE_STRING_LONG_BYTES = 56 (consts.rs:33) < 65-byte xpub payload, so a Blessed single-string outcome is unreachable; the Reject test (exit 2, empty stdout) kills both mutants and the precondition assert guards the fixture.

Not run (author-disclosed): vectors-roundtrip, FreeBSD, musl; tests-only change, acceptable.

C / I / M / N = 0 / 0 / 0 / 0
