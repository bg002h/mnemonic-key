# End-of-cycle R1 Re-Review — mk-cli 0.6.0 (`mk address` + `mk derive`)

Reviewer: feature-dev:code-reviewer (opus). Re-review after the end-of-cycle R0 C1 fold
(`mk-derive-address-end-of-cycle-R0-review.md`). Verified `ChildNumber::from_normal_idx` semantics
(valid range `0..=2³¹-1`) against docs.rs (bitcoin 0.32).

## Critical — None.
## Important — None.

### C1 fold verified
1. **Boundary arithmetic exact, no off-by-one** (`address.rs` resolve_indices): `MAX_NORMAL =
   2147483647`; count guard `c > MAX_NORMAL.saturating_add(1)` → `--count 2147483648` succeeds (max
   index 2147483647, valid), `--count 2147483649` → exit 64; range guard `b > MAX_NORMAL` →
   `--range 0,2147483647` succeeds, `--range 0,2147483648` → exit 64. Validation runs before
   collecting (no multi-GB allocation).
2. Address loop maps `from_normal_idx(index)` → `UsageError`; `chain` (0/1) keeps unwrap.
3. `derive.rs --index` maps `from_normal_idx(i)` → `UsageError`; literal `0` keeps unwrap.
4. Regression tests non-vacuous (exercise the new guard on a valid card; assert exit **64** via
   `UsageError → 64` (error.rs:85) → `ExitCode::from(64)` (main.rs:93)).
5. No residual panic path: exhaustive grep — only `chain` (0/1) + literal `0` keep unwrap (both
   compiler-constant in-range); every other unwrap in derive_support is `#[cfg(test)]`; no overflowing
   casts on user input.

### No fold-introduced drift
Happy-path index range unchanged (`--count 10` → 0..10; `--range 2,4` → 3 addrs). `(1u32<<31)-1` is
compile-time, fits u32, no clippy trigger. Fold scope confined to `resolve_indices` + 2 `from_normal_idx`
map_err + 2 tests. R0 MINORs correctly left as-is (M1 count-0 silent-empty non-blocking; M2 depth
anchor sound; M3 all-else-green).

**VERDICT: GREEN (0C/0I)**

The C1 fold is correct, complete, precisely bounded, non-vacuously tested, and introduced no drift.
The hard gate is satisfied — clear to tag/ship (with the §4 lockstep landing in lockstep).
