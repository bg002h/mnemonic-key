# mk1 no-path (depth-0) support — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development (or executing-plans). Steps use checkbox (`- [ ]`) syntax.

**Goal:** Let mk1 carry a key with no derivation path (a raw WIF / depth-0 master): decode accepts an explicit `count == 0` path, `reconstruct_xpub` rebuilds a depth-0 xpub (child `Normal{0}`), and the 0.3.2 encode-guard accepts a consistent depth-0 card — so a no-path card round-trips.

**Architecture:** Three localized mk-codec changes (decode relax / reconstruct default / guard inverse-of-reconstruct), wire-additive (reuse `0xFE 0x00`). `mk-codec` 0.3.2 → **0.4.0** (MINOR), `mk-cli` 0.4.3 → **0.5.0**. No new error variant → error-mirrors untouched. No GUI/manual lockstep.

**Tech Stack:** Rust, `bitcoin` 0.32 (`DerivationPath` / `ChildNumber` / `Xpub`), proptest.

**Source SPEC (R1 GREEN):** `design/SPEC_mk_no_path_support.md`. Branch `mk-no-path-support` off `main` `5c2bc8c`. Re-grep every citation before editing.

---

## Phase 0 — mk-codec code + tests (TDD)

### Task 0.1 — `decode_explicit_path` accepts `count == 0`

**Files:**
- Modify: `crates/mk-codec/src/bytecode/path.rs:112-116` + module rustdoc `:18-20`
- Test: `crates/mk-codec/src/bytecode/path.rs` (test mod) — invert `rejects_path_count_zero` (`:248-258`), add `round_trip_empty_path`

- [ ] **Step 1 — Invert the count-zero test (T1) + add round-trip (T2).** Replace the `rejects_path_count_zero` test with:

```rust
#[test]
fn accepts_path_count_zero_as_empty_path() {
    // count = 0 is the no-path / depth-0 case (e.g. a WIF). v0.4.0+: decode
    // returns the empty path; older decoders rejected it as PathTooDeep(0).
    let bytes = vec![0xFE, 0u8];
    let mut cursor: &[u8] = &bytes;
    let decoded = decode_path(&mut cursor).unwrap();
    assert_eq!(decoded.into_iter().count(), 0, "empty path");
    assert!(cursor.is_empty());
}

#[test]
fn round_trip_empty_path() {
    let path = DerivationPath::from_str("m").unwrap(); // empty
    let encoded = encode_path(&path);
    assert_eq!(encoded, vec![0xFE, 0x00]);
    let mut cursor: &[u8] = &encoded;
    let decoded = decode_path(&mut cursor).unwrap();
    assert_eq!(decoded, path);
    assert!(cursor.is_empty());
}
```

- [ ] **Step 2 — Run; expect FAIL.** `cargo test -p mk-codec --lib bytecode::path::tests::accepts_path_count_zero_as_empty_path` → FAIL (currently `Err(PathTooDeep(0))`).

- [ ] **Step 3 — Relax the guard.** In `decode_explicit_path` change `:113-116`:

```rust
    let count = read_u8(cursor)?;
    if count > MAX_PATH_COMPONENTS {
        return Err(Error::PathTooDeep(count));
    }
    // count == 0 → no-path / depth-0 root key (e.g. a WIF). The loop below
    // runs zero times → DerivationPath::from(vec![]) = empty path "m".
```

(Remove the `count == 0 ||` disjunct. The `for _ in 0..count` loop already handles 0.)

- [ ] **Step 4 — Touch the module rustdoc `:18-20`.** "Explicit-path encoding: indicator `0xFE`, 1-byte component count (1..=10), then …" → "… 1-byte component count (0..=10; 0 = no-path / depth-0 key), then …".

- [ ] **Step 5 — Run T1/T2/T3.** `cargo test -p mk-codec --lib bytecode::path` → PASS (incl. unchanged `rejects_path_too_deep` count==11).

- [ ] **Step 6 — Commit.** `git add crates/mk-codec/src/bytecode/path.rs && git commit -m "feat(mk): decode accepts explicit count==0 as the empty (no-path) path"`

### Task 0.2 — `reconstruct_xpub` empty path → `Normal{0}`

**Files:**
- Modify: `crates/mk-codec/src/bytecode/xpub_compact.rs:83-95`
- Test: same file (test mod) — add `reconstruct_depth0_empty_path` (T4)

- [ ] **Step 1 — Write T4 (FAILS via panic today).**

```rust
#[test]
fn reconstruct_depth0_empty_path() {
    let path = DerivationPath::from_str("m").unwrap(); // empty
    let xpub_full = synthetic_xpub(&path); // depth 0, child Normal{0}
    let compact = XpubCompact::from_xpub(&xpub_full);
    let reconstructed = reconstruct_xpub(&compact, &path).unwrap();
    assert_eq!(reconstructed.depth, 0);
    assert_eq!(
        reconstructed.child_number,
        ChildNumber::Normal { index: 0 }
    );
    assert_eq!(reconstructed.parent_fingerprint, xpub_full.parent_fingerprint);
    assert_eq!(reconstructed.chain_code, xpub_full.chain_code);
    assert_eq!(reconstructed.public_key, xpub_full.public_key);
    assert_eq!(reconstructed.network, xpub_full.network);
}
```

- [ ] **Step 2 — Run; expect FAIL (panic on `.expect`).** `cargo test -p mk-codec --lib bytecode::xpub_compact::tests::reconstruct_depth0_empty_path`.

- [ ] **Step 3 — Fix `reconstruct_xpub`.** Replace the `child_number` extraction (`:89-95`):

```rust
    let depth = components.len() as u8;
    // child_number defaults to the BIP-32 master convention Normal{0} when
    // origin_path is empty (a depth-0 / no-path key, e.g. a WIF). For a
    // non-empty path it is the terminal component (exact inverse of the
    // encode-side guard in encode.rs).
    let child_number = components
        .last()
        .copied()
        .unwrap_or(ChildNumber::Normal { index: 0 });
```

And update the fn rustdoc (`:83-84`/`:89-91`): drop "origin_path MUST be non-empty"; state "an empty origin_path (no-path / depth-0 key) yields `depth = 0` and `child_number = Normal{0}`."

- [ ] **Step 4 — Run T4 + existing `round_trip_full_xpub_depth_4`.** `cargo test -p mk-codec --lib bytecode::xpub_compact` → PASS.

- [ ] **Step 5 — Commit.** `git add crates/mk-codec/src/bytecode/xpub_compact.rs && git commit -m "feat(mk): reconstruct_xpub rebuilds a depth-0 xpub from an empty path"`

### Task 0.3 — Guard accepts a consistent depth-0 card

**Files:**
- Modify: `crates/mk-codec/src/bytecode/encode.rs:14-18` (import) + `:33-42` (guard)
- Test: same file (test mod) — invert `rejects_empty_origin_path` (`:152-170`) → `accepts_consistent_depth0_card`; add `rejects_depth0_noncanonical_child`

- [ ] **Step 1 — Rewrite the empty-path test (T5) + add the non-canonical-child reject (T6).** Replace `rejects_empty_origin_path` (`:152-170`) with:

```rust
    // Cell 5 (v0.4.0): a consistent depth-0 / no-path card (empty path, depth 0,
    // child Normal{0} — the WIF shape) now ENCODES. Was rejected pre-0.4.0.
    #[test]
    fn accepts_consistent_depth0_card() {
        let path = DerivationPath::from_str("m").unwrap(); // empty
        let card = KeyCard {
            policy_id_stubs: vec![[0xAA; 4]],
            origin_fingerprint: None,
            xpub: synthetic_xpub(&path), // depth 0, child Normal{0}
            origin_path: path,
        };
        assert!(
            encode_bytecode(&card).is_ok(),
            "consistent depth-0 no-path card must encode"
        );
    }

    // Cell 6: a depth-0 card with a non-canonical terminal child (Normal{5})
    // would NOT round-trip (reconstruct yields Normal{0}) → still rejected.
    #[test]
    fn rejects_depth0_noncanonical_child() {
        let path = DerivationPath::from_str("m").unwrap();
        let mut card = KeyCard {
            policy_id_stubs: vec![[0xAA; 4]],
            origin_fingerprint: None,
            xpub: synthetic_xpub(&path),
            origin_path: path,
        };
        card.xpub.child_number = ChildNumber::Normal { index: 5 };
        assert!(matches!(
            encode_bytecode(&card),
            Err(Error::XpubOriginPathMismatch {
                xpub_depth: 0,
                path_depth: 0,
                path_child: None,
                ..
            }),
        ));
    }
```

- [ ] **Step 2 — Run; expect FAIL.** `accepts_consistent_depth0_card` FAILs pre-change (currently rejected).

- [ ] **Step 3 — Add the production import (`:14-18`).** Add `use bitcoin::bip32::ChildNumber;` to the production `use` block (the `#[cfg(test)]` mod already imports it separately — no collision).

- [ ] **Step 4 — Fix the guard (`:33-42`).** Replace with:

```rust
    // Encoder-side invariant (SPEC_mk_v0_1.md §4): compact-73 reconstructs depth/
    // child_number from origin_path on decode; reject any xpub whose depth/
    // child_number disagree, else the emitted card decodes to a different-
    // metadata xpub (the decoder cannot detect — no on-wire depth). expected_child
    // mirrors reconstruct_xpub exactly: the terminal component, or Normal{0} for an
    // empty path (depth-0 / no-path key, e.g. a WIF).
    let path_depth = card.origin_path.into_iter().count();
    let path_child = card.origin_path.into_iter().last().copied();
    let expected_child = path_child.unwrap_or(ChildNumber::Normal { index: 0 });
    if card.xpub.depth as usize != path_depth || card.xpub.child_number != expected_child {
        return Err(Error::XpubOriginPathMismatch {
            xpub_depth: card.xpub.depth,
            path_depth: path_depth as u8,
            xpub_child: card.xpub.child_number,
            path_child,
        });
    }
```

- [ ] **Step 5 — Run T5/T6/T7.** `cargo test -p mk-codec --lib bytecode::encode` → PASS (the three unchanged cells `rejects_xpub_depth_mismatch`, `rejects_xpub_child_mismatch_same_depth`, `aligned_explicit_path_card_encodes` stay green).

- [ ] **Step 6 — Commit.** `git add crates/mk-codec/src/bytecode/encode.rs && git commit -m "feat(mk): encode-guard accepts a consistent depth-0 (no-path) card"`

### Task 0.4 — End-to-end round-trip (T8) + proptest depth-0 (T9) + key_card doc (M1)

**Files:**
- Test: `crates/mk-codec/src/bytecode/encode.rs` (test mod) — add `depth0_card_round_trips` (T8)
- Modify: `crates/mk-codec/tests/common/mod.rs:39-72` (T9)
- Modify: `crates/mk-codec/src/key_card.rs:46-51` (doc, M1)

- [ ] **Step 1 — T8 (full bytecode round-trip).** Add to `encode.rs` test mod:

```rust
    // Cell 8: the WIF/no-path card survives the full bytecode round-trip
    // (encode_bytecode -> decode_bytecode), proving end-to-end support.
    #[test]
    fn depth0_card_round_trips() {
        use crate::bytecode::decode::decode_bytecode;
        let path = DerivationPath::from_str("m").unwrap();
        let card = KeyCard {
            policy_id_stubs: vec![[0xAA; 4]],
            origin_fingerprint: None,
            xpub: synthetic_xpub(&path),
            origin_path: path.clone(),
        };
        let wire = encode_bytecode(&card).unwrap();
        let decoded = decode_bytecode(&wire).unwrap();
        assert_eq!(decoded.origin_path, path);
        assert_eq!(decoded.xpub.depth, 0);
        assert_eq!(decoded.xpub.child_number, ChildNumber::Normal { index: 0 });
        assert_eq!(decoded.xpub.public_key, card.xpub.public_key);
        assert_eq!(decoded.xpub.chain_code, card.xpub.chain_code);
    }
```

Run: `cargo test -p mk-codec --lib bytecode::encode::tests::depth0_card_round_trips` → PASS (relies on 0.1-0.3).

- [ ] **Step 2 — T9 (proptest depth-0 arm).** In `tests/common/mod.rs`:
  - `path_strategy` (`:55`): `prop_oneof![standard, explicit].boxed()` → `prop_oneof![standard, explicit, Just(DerivationPath::from_str("m").unwrap())].boxed()`.
  - `xpub_strategy` (`:66-70`): the live binding is `let child_number = *components.last().expect("path is non-empty (standard entries + explicit 1..=10)");` — `components.last()` is `Option<&ChildNumber>`, so a literal `.expect→.unwrap_or` swap will NOT compile (`unwrap_or` needs a `&ChildNumber`, and the leading `*` then derefs a non-reference — R0 C1). Replace the whole binding, dropping the `*` and inserting `.copied()` (mirrors `synthetic_xpub` / the production guard):

```rust
    let child_number = components
        .last()
        .copied()
        .unwrap_or(ChildNumber::Normal { index: 0 });
```

Run: `cargo test -p mk-codec --test '*'` (the `keycard_roundtrip` proptest) → PASS, now sampling depth-0.

- [ ] **Step 3 — key_card.rs doc (M1).** In the `xpub` field rustdoc `text` block (`:47-50`), after `child_number := last_component(origin_path)` add a line `child_number := Normal{0} when origin_path is empty (depth-0 / no-path key)`.

- [ ] **Step 4 — Full crate gates.** `cargo test -p mk-codec && cargo test -p mk-cli && cargo clippy -p mk-codec -p mk-cli --all-targets -- -D warnings && cargo +stable fmt -p mk-codec -p mk-cli -- --check` → all green.

- [ ] **Step 5 — Commit.** `git add crates/mk-codec/src/bytecode/encode.rs crates/mk-codec/tests/common/mod.rs crates/mk-codec/src/key_card.rs && git commit -m "test(mk): end-to-end no-path round-trip + proptest depth-0 arm + key_card doc"`

---

## Phase 1 — Doc edits (SPEC §4 E1-E10)

### Task 1.1 — `SPEC_mk_v0_1.md` E1-E9

**Files:** Modify `design/SPEC_mk_v0_1.md` (re-grep each line before editing — citations are `5c2bc8c` snapshots).

- [ ] **Step 1 — Apply E1-E9** exactly per SPEC §4: `:172` (`1..=10`→`0..=10` + note), `:229` (`MUST be in 1..=10`→`0..=10 (0 = no-path)`), `:237` (append count==0-valid note), `:257-258` (reconstruction block + Normal{0}; keep `:254`/`:261`), `:263` (encoder treats empty as Normal{0}), `:285` (`> 10 (or == 0)`→`> 10`; count==0 valid), `:294` (E7 reframe — empty→Normal{0}; consistent depth-0 valid), `:303` (E8 closing note). E9: `:265`/`:360` unchanged.
- [ ] **Step 2 — Consistency grep.** `grep -n '1\.\.=10\|(or == 0\|or `== 0`' design/SPEC_mk_v0_1.md` → no stale survivor in the edited spots. Confirm E4/E7/E8 all state empty→`Normal{0}`.
- [ ] **Step 3 — Commit.** `git add design/SPEC_mk_v0_1.md && git commit -m "docs(mk): SPEC_mk_v0_1 — no-path/depth-0 path codec + encoder-invariant updates"`

### Task 1.2 — `SPEC_mk_depth_child_enforcement.md` superseding note (E10)

**Files:** Modify `design/SPEC_mk_depth_child_enforcement.md` (`:14`, `:30`, `:57`).

- [ ] **Step 1 — Head blockquote.** Insert near the top (after the title): `> **Superseded in part by mk-codec 0.4.0** (`SPEC_mk_no_path_support.md`): an empty origin_path is now a *valid, representable* no-path / depth-0 card (child `Normal{0}`), not a mismatch. The depth/child agreement guard below remains accurate for **genuine** disagreements only.`
- [ ] **Step 2 — Inline `:30` and `:57`.** After each, append: `**(0.4.0: an empty origin_path IS now representable — a consistent depth-0 card with child Normal{0} encodes and round-trips; only genuine disagreement is rejected.)**`
- [ ] **Step 3 — Commit.** `git add design/SPEC_mk_depth_child_enforcement.md && git commit -m "docs(mk): supersede depth-child-enforcement doc for the v0.4.0 no-path carve-out"`

---

## Phase 2 — Version + FOLLOWUP + ship + publish

### Task 2.1 — Version bumps + pin

**Files:** `crates/mk-codec/Cargo.toml:3`, `crates/mk-cli/Cargo.toml:3` + the `mk-codec` pin (`:20`).

- [ ] **Step 1 — Bump.** `mk-codec` `0.3.2`→`0.4.0`; `mk-cli` `0.4.3`→`0.5.0` + pin `mk-codec = { path = "../mk-codec", version = "0.4.0" }`.
- [ ] **Step 2 — Lockstep build.** `cargo build -p mk-codec -p mk-cli && cargo metadata --locked --format-version 1 >/dev/null` (Cargo.lock updates in the same commit).
- [ ] **Step 3 — Commit.** `git add crates/mk-codec/Cargo.toml crates/mk-cli/Cargo.toml Cargo.lock && git commit -m "release(mk-codec): v0.4.0 — mk1 no-path (depth-0) support; mk-cli v0.5.0"`

### Task 2.2 — FOLLOWUP record

**Files:** `design/FOLLOWUPS.md`.

- [ ] **Step 1 — Add entry** `mk1-no-path-depth0-support` (per SPEC §8), `Status: resolved <Phase-0 head SHA>`, `Companion: mnemonic-toolkit mk1-wif-bundle-depth0-invalid-card`. Fill `<SHA>` from `git rev-parse --short HEAD` of the Phase-0 final commit.
- [ ] **Step 2 — Commit.** `git add design/FOLLOWUPS.md && git commit -m "followup(mk): mk1-no-path-depth0-support resolved in v0.4.0"`

### Task 2.3 — End-of-cycle R0 + ship + publish

- [ ] **Step 1 — End-of-cycle opus R0** over the full branch diff → persist to `design/agent-reports/mk-no-path-end-of-cycle-R0-review.md`. Fold to GREEN (0C/0I); re-dispatch after any fold.
- [ ] **Step 2 — Clean-tree check.** `git status --porcelain` empty before the ship sequence.
- [ ] **Step 3 — ff-merge.** `git checkout main && git merge --ff-only mk-no-path-support && git push origin main`.
- [ ] **Step 4 — Publish.** `cargo publish -p mk-codec` (0.4.0), then — after it indexes — `cargo publish -p mk-cli` (0.5.0). Verify both on crates.io.

---

## Phase 3 — Toolkit re-pin (SEPARATE cycle, post-publish)

Tracked by toolkit task; its own plan + R0. Out of scope for this plan. Summary: re-pin `mnemonic-toolkit` `mk-codec` 0.3.1→0.4.0; explicit `XpubOriginPathMismatch` arms in `friendly.rs` + `error.rs::mk_codec_exit_code`; fix the two `verify_bundle.rs` depth-4-xpub/bip84-path fixtures; add a `bundle --wif → verify-bundle` round-trip regression; resolve `mk1-wif-bundle-depth0-invalid-card` + `mk1-depth-child-compensating-check-watch`; **gate on the full toolkit suite**; ship PATCH.

---

## Self-review

- **Spec coverage:** §3.1→Task 0.1; §3.2→0.2; §3.3+§3.3a(path/key_card)→0.3+0.4; §3.4 losslessness asserted by T8; §4 E1-E10→Tasks 1.1/1.2; §5 SemVer→2.1; §6 T1-T9→0.1-0.4; §7 phases→Phases 0-3; §8 FOLLOWUP→2.2. All covered.
- **Placeholder scan:** only `<Phase-0 head SHA>` (filled at 2.2). No TODO/TBD.
- **Type consistency:** `ChildNumber::Normal { index: 0 }` used identically in guard (0.3), reconstruct (0.2), tests (0.2/0.3/0.4); `decode_bytecode` import path `crate::bytecode::decode::decode_bytecode` matches `decode.rs:19`; `synthetic_xpub` empty→depth0/Normal{0} per `test_helpers.rs:26-31`.
