# End-of-Cycle R1 Review — mk-codec depth/child enforcement

Opus code-reviewer. R1 over the folded diff (R0 was RED 0C/2I/2M). Verified against live source. Persisted by controller (review agent read-only).

## Confirmations
- **I1 RESOLVED** — `error_coverage.rs`: `XpubOriginPathMismatch` in `ErrorVariantName` (`:77`), `display_prefix` → `"xpub origin-path mismatch"` (`:108`, a genuine prefix of the live `#[error]` `error.rs:172-175` "xpub origin-path mismatch: …"), `is_exempt` → `Some(reason)` (`:124-129`). Exemption justified: sole construction at `encode.rs:36` (grep confirms no decode-path construction); not producible by `mk_codec::decode`. Gate 2/2.
- **I2 RESOLVED** — `mk-cli/src/error.rs:133`: `XpubOriginPathMismatch { .. } => "XpubOriginPathMismatch"` before `_ => "Unknown"`. Compiles.
- **M1 RESOLVED** — SPEC §6 mk-cli-lockstep note accurate (re-pin 0.3.2 + bump 0.4.3 + two-mirror requirement); matches mk-cli Cargo.toml.
- **No fold drift, both mirrors complete:** live `error.rs` enum = 23 variants; `ErrorVariantName` = same 23 case-for-case; `display_prefix` = 23 arms matching live literals; `mk_codec_error_kind` = 23 explicit arms + fallback. `XpubOriginPathMismatch` the sole recent addition; no other variant missing from either mirror.
- **Core unchanged:** guard (`encode.rs:33-42`, Option-safe, exact inverse of reconstruct), 4 cells, SPEC §3.6/§4 prose consistent (numbered decoder-rules 1-14 NOT polluted), FOLLOWUP `resolved bc4c338`, SemVer mk-codec 0.3.2 / mk-cli 0.4.3 / lock coherent.

## CRITICAL — None.   ## IMPORTANT — None.   ## MINOR — None new.

## VERDICT: GREEN (0C / 0I) — clear to ff-merge to `main`.
Both sibling-mirror Importants mechanically resolved + verified; mirrors complete; fold disturbed no core. (Controller post-fold run: workspace 193/0, error_coverage 2/2, clippy PASS.)
