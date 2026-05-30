# End-of-Cycle R0 Review — mk-codec depth/child enforcement

Opus code-reviewer, mandatory end-of-cycle gate before ff-merge. Branch `mk-depth-child-enforcement`. Reviewed `git diff main...HEAD` against live source. Persisted by controller (review agent had read-only tooling).

## Confirmations (file:line)
- **Guard correct + sole chokepoint** (`encode.rs:29-42`): depth + `Some(child)!=path_child`, exact inverse of `reconstruct_xpub` (`xpub_compact.rs:88,92-95`); empty-path Option-safe (no panic); reject uses full-usize `path_depth` (truncated `as u8` only in the diagnostic field). `pipeline.rs:57,68` both call `encode_bytecode` first; `encode_bytecode_stream` opaque-bytes. 100% emission coverage.
- **Variant** (`error.rs:164-185`): `XpubOriginPathMismatch{xpub_depth:u8,path_depth:u8,xpub_child:ChildNumber,path_child:Option<ChildNumber>}`, bytecode-group (non-alphabetized correct), `ChildNumber` imported, thiserror valid, all fields doc'd (`missing_docs=warn`), `#[non_exhaustive]` additive.
- **Tests non-vacuous** (`encode.rs:125-190`): 4 cells, the 3 reject cells fail pre-guard; cell 2 (child-mismatch-same-depth) is the previously-silent case a depth-only check misses.
- **No regression**: all fixtures path-aligned via `synthetic_xpub`; real-xpub corpus aligned-by-construction (`vectors.rs:179` would hard-fail otherwise); 193/0 workspace.
- **SPEC edits internally consistent** (`SPEC_mk_v0_1.md:263/265/294/303`): no residual "impossible by construction"; encoder-side-only framing preserved; the numbered decoder-rules list (1-14) NOT polluted (both encoder invariants are non-numbered paragraphs).
- **FOLLOWUP flip** (`FOLLOWUPS.md:290`): `resolved bc4c338`; toolkit companion kept as defense-in-depth.
- **SemVer**: mk-codec 0.3.2 + mk-cli 0.4.3 + lock consistent; PATCH correct.

## CRITICAL — None.

## IMPORTANT (both folded post-review)
- **I1 — `tests/error_coverage.rs` mirror-enum drift.** The new variant was NOT added to the hand-maintained `ErrorVariantName` mirror (`:54-77`) / `display_prefix` / `is_exempt`, violating the file's own maintenance rule (`:17-23`). Iteration silently skips missing entries → does not fail CI, but defeats the exhaustiveness gate. **FOLDED:** added `XpubOriginPathMismatch` to the enum + `display_prefix` ("xpub origin-path mismatch") + `is_exempt` (encoder-side invariant, not reachable via decode). Gate now 2/2.
- **I2 — mk-cli `mk_codec_error_kind` `Unknown` drift.** The new variant (reachable from `mk encode`) fell through to `kind:"Unknown"` in the JSON envelope (`mk-cli/src/error.rs:109-136`). **FOLDED:** added `XpubOriginPathMismatch{..} => "XpubOriginPathMismatch"` arm.

## MINOR
- **M1** SPEC/plan under-scoped the mk-cli surface (the 0.4.3 bump + kind map were out-of-plan). **FOLDED:** added a §6 mk-cli-lockstep note documenting the two-mirror requirement.
- **M2** `encode.rs` test-banner cites "SPEC §5" (enforcement-SPEC test section) while the variant rustdoc cites "SPEC_mk_v0_1.md §4" (format-SPEC invariant) — both correct in frame; cosmetic, left as-is.

## VERDICT: RED (0C / 2I / 2M) → folded; re-dispatch R1.
The core fix is correct + clean; RED solely on the two sibling-mirror drifts (I1 exhaustiveness gate, I2 mk-cli kind), both mechanical and now folded. **Lesson (fix-the-class): adding an `mk_codec::Error` variant requires updating BOTH `error_coverage.rs` mirrors AND `mk-cli` `mk_codec_error_kind` — the design-level R0s structurally can't catch sibling-mirror drift; the end-of-cycle R0 over the full diff does.**
