# mk1 v0.1 plan review #1 — pre-implementation

**Status:** DONE_WITH_CONCERNS
**Reviewer:** Claude Opus 4.7 (1M context)
**Date:** 2026-04-29
**Plan commit:** uncommitted (working-tree `design/IMPLEMENTATION_PLAN_mk_v0_1.md`)
**File(s):**
- `/scratch/code/shibboleth/mnemonic-key/design/IMPLEMENTATION_PLAN_mk_v0_1.md` (under review)
- `/scratch/code/shibboleth/mnemonic-key/docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md` (locks)
- `/scratch/code/shibboleth/mnemonic-key/design/SPEC_mk_v0_1.md`
- `/scratch/code/shibboleth/mnemonic-key/design/DECISIONS.md`
- `/scratch/code/shibboleth/mnemonic-key/design/FOLLOWUPS.md`
- `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/{lib,error,key_card}.rs`
- `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/encoding/bch_decode.rs` (fork source)
**Role:** reviewer (plan)

## Summary

No blockers. The plan is structurally sound, dependency ordering Phase 3→4→5→6 holds, every locked closure item maps to a concrete edit/file/step, and pre-BIP-submission audit items are correctly held in `FOLLOWUPS.md`. Found **5 should-address** issues (mostly TDD-discipline gaps and one closure-fidelity drift in the standard-table dictionary), **3 nits**, and several confirmations. Phase 1 may proceed without modification once should-address #1 (sanity-test scaffolding) is patched into the plan text.

## Issues

### #1 — should-address — Phase 3 Step 3.1.2 sanity test is not actually executable

The constants reproduce correctly: I verified `SHA-256(b"shibbolethnumskey") >> (256-65) == 0x1062435f91072fa5c` and `>> (256-75) == 0x41890d7e441cbe97273`, and both fit in u128 (65 and 75 bits respectively). But the Rust sanity-test sketch on lines 391–406 of the plan is broken: it tries to pack the top 192 bits of the digest into a `u128` via `(n as u128) << 64 | u64::from_be_bytes(digest[16..24])` and then never actually shifts to top-65/top-75. The "(Implementation refined during execution...)" disclaimer pushes this onto the implementer who will discover the constants don't fit a single u128 if they try to reproduce 256→{65,75} via a single shift. **Resolution:** rewrite the sketch to use a 32-byte big-endian integer staged via `[u8; 32]` (or `num-bigint`/`primitive-types`), then right-shift by `256-65` / `256-75`, cast to `u128`, compare to constants. Otherwise the implementer either skips this test (regression risk) or burns time figuring out the correct staging.

### #2 — should-address — Phase 4 standard-table dictionary content drifts from md1's actual `Tag::SharedPath` table

Plan Step 4.2.1 (line 543) says: "same 14 entries as md1's `Tag::SharedPath` (mainnet 0x01..0x07, testnet 0x11..0x17), plus the spec's mk1-specific BIP 48-nested + BIP 87 etc. per §3.5". But SPEC §3.5 (lines 128–139) already lists 7 mainnet + 7 testnet = 14 entries *including* BIP 48-nested + BIP 87 — they are not "extra". The "plus" wording suggests a >14-entry table will land, which would silently drift mk1's standard-table contents from md1's `Tag::SharedPath` and break the closure-design Q-3 framing ("mirrors md1's `Tag::SharedPath` precedent"). **Resolution:** rewrite the bullet as "same 14 entries as md1's `Tag::SharedPath` (mainnet 0x01..0x07, testnet 0x11..0x17); confirm exact dictionary by reading md1's source before copying". Add a Phase 3/4 task to verify md1's table values byte-for-byte before forking.

### #3 — should-address — TDD discipline asymmetry in Phase 4: round-trip without sad-path-first

Phase 4 claims TDD ("each gets a property-shape test (round-trip) and the relevant sad-path tests... before the impl lands") but Step 4.4.5 ("Sad-path tests") sits *after* Steps 4.4.2 (encoder) and 4.4.3 (decoder), implying impl-then-test. This contradicts the per-phase TDD claim in the plan header (line 9). The same pattern appears in Phase 5 — Step 5.4.3 round-trip tests come after 5.4.1/5.4.2 implementations. Phase 3's `#[ignore]`-marked sad-path scaffolds (Step 3.2.4) are the right pattern; Phase 4/5 should adopt the same shape: stub the public function returning `Err(todo)`, write the failing sad-path test first, then implement. **Resolution:** reorder Steps 4.4.2 ↔ 4.4.5 in narrative (or merge into "test-then-impl" sub-steps); same for Phase 5 5.4.

### #4 — should-address — Public API gap: no inspection surface for compact-73 form pre-reconstruction

The closure Q-7 lock makes `XpubCompact` the canonical on-wire representation, with full-78 reconstructed at decode time. The plan's `KeyCard.xpub: Xpub` (inherited from current scaffold) means a caller who decodes a card has no way to inspect the literal compact-73 bytes that came off the wire — only the post-reconstruction `Xpub`. For BIP-388 foreign-xpub-recovery flows, this is fine; but a debugging tool, a "did this card encode v.s. what's on the wire" verifier, or anyone implementing `decoder-error-variant-parity` audit (FOLLOWUPS.md) needs the pre-reconstruction form. **Resolution:** either expose `pub fn decode_compact(strings: &[&str]) -> Result<(KeyCard, XpubCompact)>` (or attach `compact: XpubCompact` as an inspectable field on a `DecodeReport`/`KeyCard` extension), or document explicitly in the plan that compact-73 is internal-only for v0.1 and add a `v0.1-nice-to-have` FOLLOWUPS entry.

### #5 — should-address — Phase 5 chunk-set-id bit width and randomness source are unspecified

Step 5.4.1 line 756 says "generate `chunk_set_id` (random `u32` masked to 20 bits)" — but does not specify which RNG. Two real options: `bitcoin::secp256k1::rand` (already in the dep tree via `bitcoin`), or `rand` directly. Picking the wrong one creates a dep-bloat surprise; deferring to "implementer's choice" means review-time churn. Also: Step 5.3.1 splits the bytecode into "chunks of approximately equal long-code-fragment size (53 bytes each)" — the closure §2.4 allows regular-code chunks at 45 bytes, but the plan's encoder unconditionally picks long-code. That's a defensible default but deserves a one-liner: "v0.1 always emits long-code chunks; regular-code chunked emission is deferred to v0.2." **Resolution:** name the RNG; pin the regular-vs-long-code chunked-fragment policy explicitly.

### #6 — nit — Phase 3 capacity-constants naming inconsistency with closure

Plan line 374 uses `CHUNKED_FRAGMENT_REGULAR_BYTES`, line 377 `CHUNKED_FRAGMENT_LONG_BYTES`. Closure §2.4 calls these "chunked-fragment regular code: 45 bytes per fragment" — fine, but the SPEC ripple in Phase 1 should land matching identifiers, otherwise the spec uses one name and the code uses another. Worth aligning during Phase 1 so reviewers in Phase 3 don't have to re-litigate naming.

### #7 — nit — Plan declares Phase 3 lib.rs re-exports list with `...` ellipsis

Line 465: `pub use consts::{HRP, NUMS_DOMAIN, MK_REGULAR_CONST, MK_LONG_CONST, MAX_PATH_COMPONENTS, ...};` — the `...` is going to translate to "implementer guesses". List explicitly: at minimum the four capacity constants + `MAX_CHUNKS` + `GENERATOR_FAMILY`.

### #8 — nit — `FingerprintFlagPayloadDisagreement` is a long, awkward variant name

Plan line 432. The closure §4 ripple wording ("encoder/decoder MUST agree on fingerprint flag presence vs payload presence") doesn't constrain the name. Suggestion: `FingerprintFlagMismatch` or `FingerprintFlagInconsistent` parallels existing variant naming (`XpubDepthMismatch` was the prior style). This becomes part of the public API once landed; cheap to fix now.

## Confirmations

- **NUMS constants reproduce.** `SHA-256(b"shibbolethnumskey")` top-65 = `0x1062435f91072fa5c`, top-75 = `0x41890d7e441cbe97273`, both fit in u128 — verified with Python. The plan's locked hex values match.
- **Closure-fidelity (Q-1, Q-2, Q-3, Q-7, Q-8):** every locked item maps to a concrete plan edit. NUMS in §1.1.1, 4-byte stub rationale tightening in §1.1.6, cap-10 in §1.1.8 + §3.2.2, compact-73 in §1.1.9 + §4.3, bit-2 fingerprint flag in §1.1.4 + §4.1 + §4.4.1.
- **`chunk_set_id` rename used from day 1 in mk1.** Plan Step 1.1.3, Step 5.2.1, and Step 5.5.3 all use `chunk_set_id` — no leftover "wallet identifier" usage in mk1's own files. Cross-repo coordination (md1 rename) correctly held in `FOLLOWUPS.md` as `chunk-set-id-rename` / tier `cross-repo`.
- **`XpubDepthMismatch` removal.** Plan Step 3.2.1 explicitly removes the variant; Step 1.1.10 removes SPEC rule 8. Pair holds.
- **D-13 fork-not-share strategy in the plan.** Phase 5 explicitly forks BCH primitives from md-codec, file-comments the fork date and the `mc-codex32` extraction trigger (Q-9). Eventual-shared-crate alignment is preserved without premature extraction.
- **Pre-BIP-submission audit items correctly deferred.** All four (NUMS structural audit, HRP collision, BIP cross-references, error-variant ↔ negative-vector parity) are in `FOLLOWUPS.md` at tier `pre-bip-submission`; plan does not try to land any of them in v0.1.
- **Dependency ordering Phase 3 → 4 → 5 → 6 holds.** Phase 4 imports Phase 3's `consts` + `Error`; Phase 5 imports Phase 4's `encode_bytecode`/`decode_bytecode` + `KeyCard`; Phase 6 vector harness uses Phase 5's public `encode`/`decode`. No backward dep.
- **Cross-format-alignment plumbing for future `mc-codex32`.** Phase 5's `string_layer/{bch,header,chunk}.rs` module split is the same shape that md-codec uses (`encoding/bch_decode.rs`, chunking in dedicated module). When D-13's extraction happens, the directory structure is already amenable. D-14 captures the bit-allocation pattern lock at the decisions level.
- **Phase 1 = docs-only, Phase 2 = docs-only, Phase 7 = release plumbing.** Test-discipline claim (header line 9) accurately reflects which phases are TDD vs not.

## Open observations

- Phase 6's vector schema (line 854) declares `"schema": 1` and uses a JSON shape; mk1's vectors should explicitly declare endianness, bytecode-hex case (lower vs upper), and JSON-key sort discipline before the SHA pin lands. Otherwise patch-version differences in JSON serialization tooling can roll the SHA. md1's vector schema is the natural reference; cross-checking it is a pre-Phase-6 task worth adding.
- Plan's per-phase Opus review dispatches are all narrative ("Dispatch Opus reviewer") rather than concrete subagent prompts. Acceptable at plan stage; phase-execution time will need to materialize the prompts. v0.7's reference plan does the same so this is consistent with the methodological reference.
- The plan does not mention `cargo clippy` / `cargo fmt` discipline. md-codec's CI surface presumably enforces these; mk-codec will inherit. Worth a one-liner under "test discipline" for completeness.
- Phase 4 Step 4.4.1 changes `KeyCard.origin_fingerprint` from `Fingerprint` to `Option<Fingerprint>`. This is a breaking API change to the existing scaffold. Pre-1.0 it's fine, but the commit message in 4.5.2 should call it out as such (it currently does not).
- The plan does not stage a `feature/v0.1.0-implementation` branch creation step before Phase 1. Plan header (line 7) declares the branch but no `git checkout -b` step exists. Likely caught by the executing-plans skill, but worth an explicit Phase-0 if the convention is to mirror v0.7's plan exactly.

## Verdict

**DONE_WITH_CONCERNS.** Phase 1 may proceed once Issue #1 is patched into the plan text (the broken sanity-test sketch will trip the implementer). Issues #2–#5 are should-address before reaching their respective phases (Phase 4 for #2 + #3, Phase 5 for #3 + #5, public-API-impacting #4 before Phase 4 lands `KeyCard` changes). Nits are inline-cheap or FOLLOWUPS-trackable.
