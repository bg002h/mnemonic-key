# R1 ARCHITECT REVIEW — `IMPLEMENTATION_PLAN_mk_test_hardening.md`

Sonnet feature-dev:code-reviewer, R1 fold-verify. Reviewed against live source (`d9d2ed9`, bitcoin 0.32.8). Persisted verbatim.

## Fold confirmation
- **I1 CONFIRMED.** `decode_string` (`bch.rs:662`) `data_part = &rest[1..]`; `bch_code_for_length(data_part.len())` (`:669`) — `data_part.len() = total − 3` (header inside). `data_part_len = saturating_sub(3)` is exact. 6-stub `multi_chunk_card`: `strings[0]` = 108 (Long, assertion passes), `strings.last()` = 25 (Regular, passes). Consistent with the updated docstring + T2a.
- **I2 CONFIRMED.** 97 → data-part 94 → `bch_code_for_length(94)=None` (reserved gap `bch.rs:120`) → `InvalidStringLength(94)`. T3b code (guard `>97`, trim, `matches!(... Err(Error::InvalidStringLength(94)))`) compiles (`String: FromIterator<&char>` stable); no dangling control flow / shadowing. `105` alternative removed (the only surviving `105` is the corrective text rejecting it).
- **M1 CONFIRMED.** Vacuous `prop_assume!(strings.len() >= 1)` gone; the surviving `prop_assume!(len > 11 + n_errors)` is the correct non-vacuous guard.
- **M2 CONFIRMED.** Non-compiling `[u8;1].into()` `stubs_255` line gone; only the `[i as u8, (i>>8) as u8, 0xAB, 0xCD]` form + obsolete note removed.
- **M3 CONFIRMED.** All 4 `into_iter().copied()` → `path.as_ref().to_vec()` (xpub_strategy + 3 fixtures). `DerivationPath: AsRef<[ChildNumber]>` exists in 0.32.8. No survivors.

## Critical: None.  Important: None.
## Minor
- **Self-Review staleness** (folded post-R1): the Placeholder-scan paragraph said "two `(verify at impl)` notes (T3b trim target; rust-bitcoin API names)"; after I2, only the API-names note remains open → corrected to "one note." No implementation impact (task code + impl note are authoritative).

## VERDICT: GREEN (0C / 0I)
All folds correct; no fold-introduced drift. Plan cleared for execution (subagent-driven or inline).
