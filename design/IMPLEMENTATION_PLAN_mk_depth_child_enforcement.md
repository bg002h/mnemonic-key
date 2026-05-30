# mk-codec depth/child Enforcement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Reject, at encode time, any `KeyCard` whose `xpub.depth`/`child_number` disagree with `origin_path` — so mk1's compact-73 form is genuinely lossless instead of silently reconstructing a different-metadata xpub.

**Architecture:** One encoder-side guard in `encode_bytecode` (the sole xpub-serialization chokepoint), a new additive `XpubOriginPathMismatch` error variant, tests, SPEC corrections, and the FOLLOWUP flip. Encoder-only (the decoder structurally cannot detect this — no on-wire depth).

**Tech Stack:** Rust, `bitcoin = "0.32"` (`Xpub`/`ChildNumber`/`DerivationPath`), `thiserror`.

**Source spec (R0-GREEN):** `design/SPEC_mk_depth_child_enforcement.md`. **Branch:** `mk-depth-child-enforcement` (off `main`). **mk-codec 0.3.1 → 0.3.2 PATCH.** No GUI/manual lockstep. **This plan is subject to the mandatory opus R0 gate before any task runs.**

---

## File Structure
- **Modify** `crates/mk-codec/src/error.rs` — add `use bitcoin::bip32::ChildNumber;` + the `XpubOriginPathMismatch` variant (bytecode-layer group, after `CardPayloadTooLarge`; **NOT alphabetized** — mk-codec's enum is grouped string-layer/bytecode-layer, not alphabetical; do not "fix" the ordering).
- **Modify** `crates/mk-codec/src/bytecode/encode.rs` — the guard in `encode_bytecode` + 4 new test cells (add `use bitcoin::bip32::ChildNumber;` to the test mod).
- **Modify** `crates/mk-codec/Cargo.toml` — version `0.3.1` → `0.3.2`.
- **Modify** `design/SPEC_mk_v0_1.md` — the 4 prose corrections (§3.6 + §4).
- **Modify** `design/FOLLOWUPS.md` — flip `mk1-depth-child-lossless-by-construction-unenforced` to resolved.

---

## Phase 0 — error variant + guard + tests (TDD)

### Task 0.1: the error variant
**Files:** Modify `crates/mk-codec/src/error.rs`.

- [ ] **Step 1:** add the import. The file currently has only `use thiserror::Error;` (`error.rs:12`). Add directly below it:
```rust
use bitcoin::bip32::ChildNumber;
```
- [ ] **Step 2:** add the variant as the LAST variant of `enum Error`, immediately after the `CardPayloadTooLarge { … }` block and before the enum's closing `}`:
```rust
    /// Encoder-side invariant: the supplied `xpub`'s BIP-32 `depth` /
    /// `child_number` disagree with `origin_path` (`depth ≠` component
    /// count, or `child_number ≠` the terminal component). Compact-73
    /// reconstructs both fields from the path on decode, so emitting such a
    /// card would yield a different-metadata xpub. Rejected at encode to keep
    /// compact-73 genuinely lossless. The decoder cannot detect this (no
    /// on-wire depth) — see `design/SPEC_mk_v0_1.md` §4 (encoder-side
    /// invariant) and `design/SPEC_mk_depth_child_enforcement.md`.
    #[error(
        "xpub origin-path mismatch: xpub depth {xpub_depth} / child {xpub_child} \
         vs origin_path depth {path_depth} / last {path_child:?}"
    )]
    XpubOriginPathMismatch {
        /// `xpub.depth` as supplied.
        xpub_depth: u8,
        /// `component_count(origin_path)`.
        path_depth: u8,
        /// `xpub.child_number` as supplied.
        xpub_child: ChildNumber,
        /// Terminal component of `origin_path` (`None` for an empty path).
        path_child: Option<ChildNumber>,
    },
```
- [ ] **Step 3:** Run `cargo build -p mk-codec 2>&1 | tail -5` → builds. (`ChildNumber: Display` satisfies `{xpub_child}`; `Option<ChildNumber>: Debug` satisfies `{path_child:?}`.)
- [ ] **Step 4:** Commit:
```bash
git add crates/mk-codec/src/error.rs
git commit -m "feat(mk-codec): add XpubOriginPathMismatch encoder-side error variant"
```

### Task 0.2: failing tests (TDD — reject cells before the guard)
**Files:** Modify `crates/mk-codec/src/bytecode/encode.rs` (the `#[cfg(test)] mod tests`).

- [ ] **Step 1:** in the test mod, extend the `use bitcoin::bip32::{…}` line to include `ChildNumber`. It is currently:
```rust
    use bitcoin::bip32::{DerivationPath, Fingerprint};
```
Change to:
```rust
    use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint};
```
- [ ] **Step 2:** append these 4 test cells inside the `mod tests` block (before its closing `}`):
```rust
    // ── XpubOriginPathMismatch encoder-side guard (SPEC §5) ──────────────────

    // Cell 1: xpub.depth ≠ component_count(origin_path) → reject.
    #[test]
    fn rejects_xpub_depth_mismatch() {
        let mut card = fixture_card_1stub_with_fp(); // path m/48'/0'/0'/2' → depth 4
        card.xpub.depth = 3;
        assert!(matches!(
            encode_bytecode(&card),
            Err(Error::XpubOriginPathMismatch { xpub_depth: 3, path_depth: 4, .. }),
        ));
    }

    // Cell 2 + 6: same depth, wrong terminal child (the previously-silent case;
    // the fixture is a standard-table path, so this also covers the dictionary
    // child-mismatch). A depth-only check (as the toolkit's does) would MISS this.
    #[test]
    fn rejects_xpub_child_mismatch_same_depth() {
        let mut card = fixture_card_1stub_with_fp(); // terminal child = 2'
        card.xpub.child_number = ChildNumber::Hardened { index: 1 }; // → 1', depth still 4
        assert!(matches!(
            encode_bytecode(&card),
            Err(Error::XpubOriginPathMismatch { .. }),
        ));
    }

    // Cell 3: empty origin_path (depth-0, hand-buildable via pub fields) → reject,
    // no panic; path_child renders None.
    #[test]
    fn rejects_empty_origin_path() {
        let path = DerivationPath::from_str("m").unwrap(); // empty path
        let card = KeyCard {
            policy_id_stubs: vec![[0xAA; 4]],
            origin_fingerprint: None,
            xpub: synthetic_xpub(&path), // depth 0, child Normal{0}
            origin_path: path,
        };
        assert!(matches!(
            encode_bytecode(&card),
            Err(Error::XpubOriginPathMismatch { path_child: None, .. }),
        ));
    }

    // Cell 4: an aligned EXPLICIT-path card (not in the standard table) encodes
    // OK — guards against false-positives on explicit-mode paths. (The existing
    // `encodes_typical_1stub_card_to_84_bytes` covers the standard-table-aligned
    // case = SPEC cell 5; `xpub_compact.rs::round_trip_full_xpub_depth_4` covers
    // the reconstruct round-trip = SPEC cell 4 losslessness.)
    #[test]
    fn aligned_explicit_path_card_encodes() {
        let path = DerivationPath::from_str("m/44'/0'/0'/0/5").unwrap(); // 5 comps, explicit
        let card = KeyCard {
            policy_id_stubs: vec![[0xAA; 4]],
            origin_fingerprint: None,
            xpub: synthetic_xpub(&path), // depth 5, child Normal{5} — aligned
            origin_path: path,
        };
        assert!(encode_bytecode(&card).is_ok(), "aligned explicit-path card must encode");
    }
```
- [ ] **Step 3:** Run `cargo test -p mk-codec --lib bytecode::encode 2>&1 | tail -25`. EXPECTED: `rejects_xpub_depth_mismatch`, `rejects_xpub_child_mismatch_same_depth`, `rejects_empty_origin_path` FAIL (the variant exists but nothing returns it yet — `encode_bytecode` currently returns `Ok`, so `matches!(…, Err(…))` is false); `aligned_explicit_path_card_encodes` PASSES (encode already succeeds). Do NOT commit yet.

### Task 0.3: the guard
**Files:** Modify `crates/mk-codec/src/bytecode/encode.rs`.

- [ ] **Step 1:** in `encode_bytecode`, insert the guard immediately AFTER the two `policy_id_stubs` length checks (after the second `return Err(Error::InvalidPolicyIdStubCount);` block, before `let header = BytecodeHeader { … }`). Placing it at the top (rather than adjacent to `from_xpub`) makes it run before `encode_path`, so the empty-path cell is rejected cleanly without depending on `encode_path`'s empty-input behavior. The guard reads only `card.xpub` + `card.origin_path`:
```rust
    // SPEC §2.2 (encoder-side invariant): compact-73 reconstructs depth/
    // child_number from origin_path on decode; reject any xpub whose depth/
    // child_number disagree, else the emitted card decodes to a different-
    // metadata xpub (the decoder cannot detect — no on-wire depth).
    let path_depth = card.origin_path.into_iter().count();
    let path_child = card.origin_path.into_iter().last().copied();
    if card.xpub.depth as usize != path_depth
        || Some(card.xpub.child_number) != path_child
    {
        return Err(Error::XpubOriginPathMismatch {
            xpub_depth: card.xpub.depth,
            path_depth: path_depth as u8,
            xpub_child: card.xpub.child_number,
            path_child,
        });
    }
```
- [ ] **Step 2:** Run `cargo test -p mk-codec --lib bytecode::encode 2>&1 | tail -25` → all 4 new cells PASS + the existing `encodes_typical_1stub_card_to_84_bytes`/`encodes_card_without_fingerprint_to_80_bytes`/`rejects_zero_stubs`/`deterministic_output` still PASS (the fixture is aligned, so no over-rejection).
- [ ] **Step 3:** Run the FULL mk-codec suite — the guard must not regress any aligned-card test: `cargo test -p mk-codec 2>&1 | tail -15` → all green (incl. `xpub_compact.rs::round_trip_full_xpub_depth_4` and the vector-gen-derived tests, all of which use aligned synthetic xpubs).
- [ ] **Step 4 (CI parity):** `cargo +stable clippy -p mk-codec --all-targets -- -D warnings 2>&1 | tail` → clean; `cargo +stable fmt -p mk-codec` then `cargo +stable fmt --check -- crates/mk-codec/src/error.rs crates/mk-codec/src/bytecode/encode.rs`. **Do NOT run unscoped `cargo fmt`** (the md/ms lesson — it reformats pre-existing files under stable rustfmt drift; format only the two touched files, `git restore` anything else fmt touches). If `cargo +stable fmt --check --all` flags PRE-EXISTING files, surface to the user (chore vs leave) — do not fold into this commit.
- [ ] **Step 5:** Commit:
```bash
git add crates/mk-codec/src/bytecode/encode.rs
git commit -m "fix(mk-codec): enforce xpub depth/child_number agreement with origin_path at encode

compact-73 reconstructs depth/child_number from origin_path on decode; a
mis-aligned xpub silently round-tripped to a different-metadata xpub. Reject at
the encode_bytecode chokepoint with XpubOriginPathMismatch (covers both depth and
terminal child). Encoder-side only — the decoder cannot detect (no on-wire depth).
Resolves mk1-depth-child-lossless-by-construction-unenforced."
```

---

## Phase 1 — SPEC corrections + FOLLOWUP flip

### Task 1.1: SPEC_mk_v0_1.md prose corrections
**Files:** Modify `design/SPEC_mk_v0_1.md` (re-grep the live line numbers; they were 257/263/265/292/301 @ `998f3c9`).

- [ ] **Step 1 — §3.6 "Why compact-73" (was :263):** replace the sentence "Compact-73 is *lossless* — both fields are reconstructible from the path — and saves 5 bytes per card (~one row of typical hand-engraving). The drift class is impossible by construction." with:
```
Compact-73 is *lossless because the encoder enforces agreement*: both fields are
reconstructible from the path, AND `encode` rejects any `xpub` whose `depth` /
`child_number` disagree with `origin_path` (`Error::XpubOriginPathMismatch`),
saving 5 bytes per card (~one row of typical hand-engraving). The drift class is
closed on the emit side by that encoder invariant (§4).
```
- [ ] **Step 2 — §3.6 Limit-of-detection note (was :265):** prepend to that paragraph (do NOT delete it):
```
(As of mk-codec 0.3.2 the **emit** side of this hazard is closed: `encode` rejects
a depth/child-mismatched card outright — see §4's encoder-side invariant — so the
codec can no longer *produce* such a card. The residual limit-of-detection below
applies only to hand-constructed bytecode fed directly to the decoder, which
reconstructs from the path with no on-wire depth to cross-check.)
```
Keep the rest of the note (the §6 out-of-band first-address recommendation) intact.
- [ ] **Step 3 — §4 removed-rule note (was :301):** replace "Note: the v0 spec sketch's `XpubDepthMismatch` rule is removed under compact-73 — `xpub.depth` is no longer carried on-wire, so drift between the depth field and the path is impossible by construction." with:
```
Note: the v0 spec sketch's `XpubDepthMismatch` rule is re-instated under compact-73
as an **encoder-side invariant** `Error::XpubOriginPathMismatch`, covering BOTH
`depth ≠ component_count(origin_path)` and `child_number ≠ last_component(origin_path)`
(the original sketch was depth-only). It is enforced at encode (see the encoder-side
invariant paragraph below); the decoder cannot detect it because `depth`/`child_number`
are not carried on-wire.
```
- [ ] **Step 4 — §4 add the sibling encoder-side invariant** (right after the existing fingerprint-flag "Encoder-side invariant (not a decoder rule)" paragraph, was :292):
```
Encoder-side invariant (not a decoder rule): encoders MUST reject a card whose
`xpub.depth ≠ component_count(origin_path)` OR `xpub.child_number ≠ last_component(origin_path)`
with `Error::XpubOriginPathMismatch`. Compact-73 drops `depth`/`child_number` and the
decoder reconstructs them from `origin_path`, so a mismatched card would decode to a
different-metadata xpub (chain_code/public_key — and therefore addresses — are
unaffected). Like the fingerprint-flag invariant, this is structurally undetectable at
decode (no on-wire depth to compare); a hand-crafted bytecode violating it decodes to a
wrong-but-internally-consistent `KeyCard`, detectable only at the higher Wallet Instance
ID check (§5 step 4).
```
- [ ] **Step 5:** Commit:
```bash
git add design/SPEC_mk_v0_1.md
git commit -m "spec(mk1): re-instate depth/child agreement as an encoder-side invariant (0.3.2)"
```

### Task 1.2: flip the FOLLOWUP
**Files:** Modify `design/FOLLOWUPS.md` (entry at `:284`).

- [ ] **Step 1:** in the `mk1-depth-child-lossless-by-construction-unenforced` entry, change `Status: open` (or its current status line) to:
```
- **Status:** `resolved <SHA>` — mk-codec 0.3.2: `encode_bytecode` rejects depth/child-mismatched cards via the new `Error::XpubOriginPathMismatch` (covers both depth and terminal child); SPEC §3.6/§4 re-framed as an encoder-side invariant. The toolkit's compensating check (`mnemonic-toolkit/.../synthesize.rs:494-503`, companion `mk1-depth-child-compensating-check-watch`) is now reviewable-for-removal but kept as defense-in-depth this cycle.
```
(Use the Phase-0 fix commit SHA for `<SHA>` once known; a placeholder is acceptable until the final sweep.)
- [ ] **Step 2:** Commit:
```bash
git add design/FOLLOWUPS.md
git commit -m "followup(mk): resolve mk1-depth-child-lossless-by-construction-unenforced"
```

---

## Phase 2 — version bump + R0 + ship

### Task 2.1: version bump
**Files:** Modify `crates/mk-codec/Cargo.toml`.

- [ ] **Step 1:** change `version = "0.3.1"` → `version = "0.3.2"`.
- [ ] **Step 2:** `cargo build -p mk-codec 2>&1 | tail -2` (updates Cargo.lock for the version). No mk-codec `CHANGELOG.md` exists (verified `crates/mk-codec/CHANGELOG.md` absent) — skip changelog; if one was added since, append a `### Fixed` entry citing this fix.
- [ ] **Step 3:** Commit:
```bash
git add crates/mk-codec/Cargo.toml Cargo.lock
git commit -m "release(mk-codec): v0.3.2 — encode-time xpub/origin_path depth-child agreement"
```

### Task 2.2: end-of-cycle R0 + ship
- [ ] **Step 1:** full verify — `cargo test -p mk-codec` green; `cargo +stable clippy -p mk-codec --all-targets -- -D warnings` exit 0; `git status --porcelain` clean (no stray reformat).
- [ ] **Step 2:** dispatch the end-of-cycle opus R0 over `git diff main...HEAD`; persist to `design/agent-reports/mk-depth-child-end-of-cycle-R0-review.md`; fold to 0C/0I.
- [ ] **Step 3 (ship):** ff-merge `mk-depth-child-enforcement` → `main`, push (surface to the user — outward). If mk-codec is consumed by the toolkit as a published/pinned dep, note whether a publish + toolkit re-pin is wanted (the toolkit keeps its own depth check, so no functional regression if deferred).

---

## Self-Review
**Spec coverage:** §2.1 guard placement → Task 0.3 (top-of-fn rationale noted). §2.2 semantics (depth + `Some(child)!=path_child`, no u32-normalize, Option) → Task 0.3 Step 1. §2.3 variant + `ChildNumber` import + non-alphabetical placement → Task 0.1. §2.4 edge cases → tests (cell 3 empty-path, cell 4 explicit-aligned; existing tests cover standard-table aligned). §3 SPEC edits → Task 1.1 (all 4). §4 FOLLOWUP flip + toolkit-keep → Task 1.2. §5 tests 1-6 → Task 0.2 (cells 4/5/6 mapped to new+existing tests, noted inline). §6 fix-the-class/CHANGELOG → Task 2.1. §7 phasing → Phases 0/1/2. SemVer 0.3.1→0.3.2 → Task 2.1. All mapped.

**Placeholder scan:** none — all code is complete; the only deferral is the FOLLOWUP `<SHA>` (filled from the Phase-0 commit), which is a mechanical fill, not a design TBD.

**Type consistency:** `XpubOriginPathMismatch { xpub_depth: u8, path_depth: u8, xpub_child: ChildNumber, path_child: Option<ChildNumber> }` is used identically in Task 0.1 (definition), Task 0.2 (test `matches!`), Task 0.3 (construction). `card.origin_path.into_iter()` (yields `&ChildNumber`) + `.copied()` consistent. `synthetic_xpub(&path)` returns an aligned `Xpub`; mutation tests set `card.xpub.depth`/`card.xpub.child_number` directly (pub fields).
