# mk-codec Test-Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add three integration test files to `mk-codec` — a `proptest` round-trip/panic-freedom harness, BCH adversarial coverage (3/4-error correction through public `decode()` + a randomized miscorrection sweep), and an indel reject-contract — closing the survey's themes 1/2/3 for the leanest constellation codec.

**Architecture:** Test-only. A shared `tests/common/mod.rs` exposes `keycard_strategy()` (a precedent-faithful direct-construction `KeyCard` generator) reused by the proptest harness and the randomized BCH sweep. Deterministic BCH/indel cells live in their own files. No production code changes unless a test surfaces a clear bug (then a separate fix-bump per spec §6).

**Tech Stack:** Rust, `proptest = "1"` (new dev-dep), `bitcoin` 0.32, the `mk_codec::{KeyCard, encode, encode_with_chunk_set_id, decode, Error}` public surface.

**Source spec (R0-gate GREEN):** `design/SPEC_mk_codec_test_hardening.md`. **Branch:** `mk-codec-test-hardening` (off `main`). **Verified SHA:** `d9d2ed9`.

**This plan is itself subject to the mandatory opus R0 gate before any task is executed.**

---

## File Structure

- **Create** `crates/mk-codec/tests/common/mod.rs` — shared `keycard_strategy()`, `path_strategy()`, `xpub_strategy()`, `csid_strategy()`, and the `flip_chars` corruption helper. One responsibility: generators + helpers for the new test files.
- **Create** `crates/mk-codec/tests/proptest_roundtrip.rs` — Theme 1 (P1 bijection, P2 panic-freedom).
- **Create** `crates/mk-codec/tests/bch_adversarial.rs` — Theme 2 (T2a/T2b deterministic correction through `decode()`, T2c randomized miscorrection sweep) + T4 (255/256-stub boundary).
- **Create** `crates/mk-codec/tests/indel_reject_contract.rs` — Theme 3 (T3a in-band indel, T3b out-of-band length).
- **Modify** `crates/mk-codec/Cargo.toml` — add `proptest` to `[dev-dependencies]`.
- **Modify** `mnemonic-key/.gitignore` — add `**/proptest-regressions/`.
- **Modify** `mnemonic-key/design/FOLLOWUPS.md` — add 2 entries (depth/child seam; `error.rs:56` doc).

> **Note on `tests/common/mod.rs`:** Rust integration tests share helpers via a `common` submodule. Each test file declares `mod common;` and Cargo will NOT treat `common/mod.rs` as its own test binary (only top-level `tests/*.rs` are test binaries). This is the standard pattern.

---

## Phase 0 — Harness scaffold + Theme 1

### Task 0.1: Add the `proptest` dev-dep + gitignore

**Files:**
- Modify: `crates/mk-codec/Cargo.toml`
- Modify: `mnemonic-key/.gitignore`

- [ ] **Step 1: Add proptest to dev-dependencies**

In `crates/mk-codec/Cargo.toml`, under `[dev-dependencies]`, add:

```toml
proptest = "1"
```

(Mirrors `mnemonic-secret/crates/ms-codec/Cargo.toml:20`.)

- [ ] **Step 2: Add the gitignore line**

Append to `mnemonic-key/.gitignore` (under the Cargo section):

```gitignore
# proptest shrink-regression corpora (per-test-file, nested under tests/)
**/proptest-regressions/
```

- [ ] **Step 3: Verify it resolves**

Run: `cargo build -p mk-codec --tests 2>&1 | tail -3`
Expected: builds (proptest downloaded/compiled), no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/mk-codec/Cargo.toml .gitignore
git commit -m "test(mk-codec): add proptest dev-dep + proptest-regressions gitignore"
```

---

### Task 0.2: Shared generator module (`tests/common/mod.rs`)

**Files:**
- Create: `crates/mk-codec/tests/common/mod.rs`

- [ ] **Step 1: Write the generator module**

Create `crates/mk-codec/tests/common/mod.rs`:

```rust
//! Shared generators + corruption helpers for the mk-codec test-hardening
//! suite. Consumed by `proptest_roundtrip.rs` and `bch_adversarial.rs` via
//! `mod common;`. Cargo does not treat `common/mod.rs` as its own test binary.
#![allow(dead_code)] // each test file uses a subset of these helpers

use std::str::FromStr;

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use mk_codec::KeyCard;
use proptest::prelude::*;

/// The 14 standard-path dictionary entries (mirror of
/// `crates/mk-codec/src/bytecode/path.rs::STANDARD_PATHS`). Generating these
/// exercises the 1-byte standard-path indicator encode mode.
pub const STANDARD_PATHS: &[&str] = &[
    "m/44'/0'/0'",
    "m/49'/0'/0'",
    "m/84'/0'/0'",
    "m/86'/0'/0'",
    "m/48'/0'/0'/2'",
    "m/48'/0'/0'/1'",
    "m/87'/0'/0'",
    "m/44'/1'/0'",
    "m/49'/1'/0'",
    "m/84'/1'/0'",
    "m/86'/1'/0'",
    "m/48'/1'/0'/2'",
    "m/48'/1'/0'/1'",
    "m/87'/1'/0'",
];

/// A derivation path: either a standard dictionary entry (1-byte indicator
/// encode mode) OR a random explicit path of 1..=10 components with random
/// hardened bits (the `0xFE` escape mode). Both round-trip; an explicit path
/// that happens to match a dictionary entry will encode via the indicator
/// (`lookup_path`) — that is correct and not asserted against.
pub fn path_strategy() -> impl Strategy<Value = DerivationPath> {
    let standard = (0..STANDARD_PATHS.len())
        .prop_map(|i| DerivationPath::from_str(STANDARD_PATHS[i]).expect("valid standard path"));

    let explicit = prop::collection::vec(
        (0u32..0x8000_0000u32, any::<bool>()).prop_map(|(idx, hardened)| {
            if hardened {
                ChildNumber::from_hardened_idx(idx).expect("idx < 2^31")
            } else {
                ChildNumber::from_normal_idx(idx).expect("idx < 2^31")
            }
        }),
        1..=10usize,
    )
    .prop_map(DerivationPath::from);

    prop_oneof![standard, explicit].boxed()
}

/// An `Xpub` built by DIRECT struct construction (precedent:
/// `tests/round_trip.rs::synthetic_xpub`). `depth`/`child_number` are derived
/// from `path` so they are consistent by construction (sidesteps the
/// depth/child "lossless by construction" seam — SPEC §1.1). `public_key`,
/// `chain_code`, `parent_fingerprint`, and `network` are strategy-varied.
pub fn xpub_strategy(path: DerivationPath) -> impl Strategy<Value = Xpub> {
    let components: Vec<ChildNumber> = path.into_iter().copied().collect();
    let depth = components.len() as u8;
    let child_number = *components
        .last()
        .expect("path is non-empty (standard entries + explicit 1..=10)");

    (
        any::<[u8; 32]>().prop_filter("valid secp256k1 scalar", |b| {
            SecretKey::from_slice(b).is_ok()
        }),
        any::<[u8; 32]>(),
        any::<[u8; 4]>(),
        any::<bool>(),
    )
        .prop_map(move |(sk_bytes, cc, pfp, mainnet)| {
            let secp = Secp256k1::new();
            let sk = SecretKey::from_slice(&sk_bytes).expect("filtered to valid scalar");
            let pk = PublicKey::from_secret_key(&secp, &sk);
            Xpub {
                network: if mainnet {
                    NetworkKind::Main
                } else {
                    NetworkKind::Test
                },
                depth,
                parent_fingerprint: Fingerprint::from(pfp),
                child_number,
                public_key: pk,
                chain_code: ChainCode::from(cc),
            }
        })
}

/// A valid, encodable, depth/child-consistent `KeyCard`. `policy_id_stubs`
/// length 1..=8 (the 255-stub boundary is a separate deterministic cell, T4).
pub fn keycard_strategy() -> impl Strategy<Value = KeyCard> {
    (
        prop::collection::vec(any::<[u8; 4]>(), 1..=8usize),
        prop::option::of(any::<[u8; 4]>().prop_map(Fingerprint::from)),
        path_strategy(),
    )
        .prop_flat_map(|(stubs, fp, path)| {
            let p = path.clone();
            xpub_strategy(path).prop_map(move |xpub| {
                KeyCard::new(stubs.clone(), fp, p.clone(), xpub)
            })
        })
        .boxed()
}

/// A chunk-set-id within the 20-bit wire cap (`> MAX_CHUNK_SET_ID` →
/// `ChunkedHeaderMalformed`).
pub fn csid_strategy() -> impl Strategy<Value = u32> {
    0u32..=0x000F_FFFFu32
}

/// Flip the bech32 symbol at each `position` (char index) to a guaranteed-
/// different symbol — 'q' (value 0) ↔ 'p' (value 1) — preserving string
/// length (so the BCH length band is unchanged; the flips are pure
/// substitutions). Mirrors the corruption idiom in
/// `src/string_layer/pipeline.rs`'s 5-burst test.
pub fn flip_chars(s: &str, positions: &[usize]) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    for &p in positions {
        chars[p] = if chars[p] == 'q' { 'p' } else { 'q' };
    }
    chars.into_iter().collect()
}
```

- [ ] **Step 2: Verify it compiles (via a throwaway reference)**

`common/mod.rs` is not a test binary on its own, so it only compiles when a `tests/*.rs` references it. Defer the compile check to Task 0.3 Step 2.

> **Implementer note:** the exact rust-bitcoin API names (`ChildNumber::from_hardened_idx`/`from_normal_idx`, `DerivationPath::from(Vec<ChildNumber>)`, `any::<[u8;32]>()`) are the load-bearing assumptions. If the compiler rejects an arity/name, adjust to the equivalent (`ChildNumber::Hardened { index }` literal; `DerivationPath::from_iter`). This is expected proptest-strategy iteration; do NOT change the test SEMANTICS.

- [ ] **Step 3: Commit (after Task 0.3 confirms it compiles)** — folded into Task 0.3 Step 5.

---

### Task 0.3: Theme 1 — `proptest_roundtrip.rs` (P1 bijection + P2 panic-freedom)

**Files:**
- Create: `crates/mk-codec/tests/proptest_roundtrip.rs`
- Test: this file IS the test.

- [ ] **Step 1: Write the failing tests**

Create `crates/mk-codec/tests/proptest_roundtrip.rs`:

```rust
//! Theme 1 (SPEC_mk_codec_test_hardening.md §3) — property tests for the
//! `KeyCard` encode↔decode bijection (P1) and decode panic-freedom (P2).

mod common;

use common::{csid_strategy, keycard_strategy};
use mk_codec::{decode, encode_with_chunk_set_id};
use proptest::prelude::*;

proptest! {
    // P1 — bijection. `decode(encode_with_chunk_set_id(card, csid)) == card`
    // for any card over the full strategy space and any 20-bit csid.
    #[test]
    fn keycard_roundtrip(card in keycard_strategy(), csid in csid_strategy()) {
        let strings = encode_with_chunk_set_id(&card, csid)
            .expect("strategy produces only encodable cards");
        let parts: Vec<&str> = strings.iter().map(String::as_str).collect();
        let recovered = decode(&parts).expect("a freshly-encoded card must decode");
        prop_assert_eq!(recovered, card);
    }

    // P2a — decode never panics on an arbitrary single string.
    #[test]
    fn decode_never_panics_on_arbitrary_string(s in "\\PC*") {
        let _ = decode(&[s.as_str()]); // must return Ok/Err, never panic
    }

    // P2b — decode never panics on an arbitrary list of strings.
    #[test]
    fn decode_never_panics_on_arbitrary_string_list(
        v in prop::collection::vec("\\PC*", 0..6usize)
    ) {
        let parts: Vec<&str> = v.iter().map(String::as_str).collect();
        let _ = decode(&parts); // must not panic
    }

    // P2c — decode never panics on a corrupted-but-real encoding.
    #[test]
    fn decode_never_panics_on_corrupted_encoding(
        card in keycard_strategy(),
        csid in csid_strategy(),
        n_flips in 0usize..30usize,
        seed in any::<u64>(),
    ) {
        let strings = encode_with_chunk_set_id(&card, csid).unwrap();
        // Deterministic pseudo-random flips across the joined first string.
        let mut s: Vec<char> = strings[0].chars().collect();
        let mut x = seed | 1;
        for _ in 0..n_flips.min(s.len().saturating_sub(3)) {
            x ^= x << 13; x ^= x >> 7; x ^= x << 17; // xorshift64
            let idx = 3 + (x as usize % s.len().saturating_sub(3).max(1));
            s[idx] = if s[idx] == 'q' { 'p' } else { 'q' };
        }
        let corrupted: String = s.into_iter().collect();
        let mut parts_owned = strings.clone();
        parts_owned[0] = corrupted;
        let parts: Vec<&str> = parts_owned.iter().map(String::as_str).collect();
        let _ = decode(&parts); // must not panic
    }
}
```

- [ ] **Step 2: Run to verify it compiles AND the strategy is sound**

Run: `cargo test -p mk-codec --test proptest_roundtrip 2>&1 | tail -20`
Expected: **PASS** (this is a property test of *existing* correct behavior — P1/P2 should pass immediately if the strategy generates only encodable cards). If `keycard_roundtrip` fails, read the shrunk counterexample: a `.expect("encodable")` panic means the strategy violated an encoder invariant (stub>255 / path>10 / empty path) — fix the strategy bound, NOT the assertion. A `prop_assert_eq` failure means a genuine bijection bug — STOP and apply spec §6 (clear-bug-inline / defer).

- [ ] **Step 3: (only if a real bijection/panic bug surfaces)** apply SPEC §6 — surface to the user; fix-inline (→ mk-codec PATCH + its own R0) or defer (`#[ignore]` + FOLLOWUP). Otherwise skip.

- [ ] **Step 4: Run clippy with the CI gate**

Run: `cargo clippy -p mk-codec --all-targets -- -D warnings 2>&1 | tail -5`
Expected: clean (matches CI `.github/workflows/ci.yml:58`).

- [ ] **Step 5: Commit**

```bash
git add crates/mk-codec/tests/common/mod.rs crates/mk-codec/tests/proptest_roundtrip.rs
git commit -m "test(mk-codec): theme 1 — KeyCard bijection + decode panic-freedom proptests"
```

---

## Phase 1 — Theme 2 (BCH adversarial) + T4 boundary

### Task 1.1: `bch_adversarial.rs` — T2a deterministic 3/4-error correction (regular + long band)

**Files:**
- Create: `crates/mk-codec/tests/bch_adversarial.rs`

- [ ] **Step 1: Write the multi-chunk fixture + band helpers + T2a**

Create `crates/mk-codec/tests/bch_adversarial.rs`:

```rust
//! Theme 2 (SPEC_mk_codec_test_hardening.md §4) — BCH adversarial coverage:
//! 3/4-error correction THROUGH the public `decode()` (T2a/T2b) and a
//! randomized 5–8-error miscorrection sweep (T2c). mk's guard model: per-chunk
//! `bch_correct_*` re-verify + the 4-byte cross-chunk hash at reassembly
//! (the residual is ~2⁻³² — see T2c). Both BCH codes are t=4.

mod common;

use std::str::FromStr;

use bitcoin::bip32::{DerivationPath, Fingerprint};
use common::{csid_strategy, flip_chars, keycard_strategy, xpub_strategy};
use mk_codec::{Error, KeyCard, decode, encode_with_chunk_set_id};
use proptest::prelude::*;

/// Build a deterministic multi-chunk card large enough that `strings[0]` is a
/// long-code (non-last, full-size) chunk and `strings.last()` is a regular-code
/// chunk. ~6 stubs ⇒ bytecode well over the single-string capacity ⇒ ≥2 chunks.
fn multi_chunk_card() -> KeyCard {
    let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let sk = bitcoin::secp256k1::SecretKey::from_slice(&[0x42u8; 32]).unwrap();
    let pk = bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &sk);
    let comps: Vec<bitcoin::bip32::ChildNumber> = path.into_iter().copied().collect();
    let xpub = bitcoin::bip32::Xpub {
        network: bitcoin::NetworkKind::Main,
        depth: comps.len() as u8,
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, 0x40]),
        child_number: *comps.last().unwrap(),
        public_key: pk,
        chain_code: bitcoin::bip32::ChainCode::from([0xCCu8; 32]),
    };
    KeyCard::new(
        (0u8..6).map(|i| [i, i, i, i]).collect(),
        Some(Fingerprint::from([0xAA, 0xBB, 0xCC, 0xDD])),
        path,
        xpub,
    )
}

/// data-part length (symbols) of a chunked mk1 string: total minus the 3-char
/// `mk1` HRP+separator minus the 8-symbol chunked header. Used only to assert
/// the test actually exercises both BCH code variants.
fn data_part_len(s: &str) -> usize {
    s.chars().count().saturating_sub(3).saturating_sub(8)
}

#[test]
fn t2a_three_and_four_error_correction_through_public_decode() {
    let card = multi_chunk_card();
    let strings = encode_with_chunk_set_id(&card, 0).unwrap();
    assert!(strings.len() >= 2, "fixture must be multi-chunk; got {}", strings.len());

    // strings[0] is a non-last (full-size, long-code) chunk; strings.last()
    // is the regular-code chunk (mirrors the structure documented in
    // src/string_layer/pipeline.rs's 5-burst test).
    let long_dl = data_part_len(&strings[0]);
    let reg_dl = data_part_len(strings.last().unwrap());
    assert!(
        (96..=108).contains(&long_dl),
        "strings[0] must be a long-code chunk (data-part 96..=108); got {long_dl}. \
         Increase the stub count in multi_chunk_card() if this fails."
    );
    assert!(
        (14..=93).contains(&reg_dl),
        "last chunk must be a regular-code chunk (data-part 14..=93); got {reg_dl}"
    );

    // Corrupt 3, then 4, data-part symbols (past the 3-char HRP + 8-symbol
    // header → char-index ≥ 11) in EACH band; BCH t=4 must recover the original.
    for &n in &[3usize, 4usize] {
        let positions: Vec<usize> = (11..11 + n).collect();

        // long-code chunk (strings[0])
        let mut s_long = strings.clone();
        s_long[0] = flip_chars(&strings[0], &positions);
        let parts: Vec<&str> = s_long.iter().map(String::as_str).collect();
        assert_eq!(
            decode(&parts).expect("BCH t=4 corrects the long-code chunk"),
            card,
            "{n}-error correction failed for the long-code chunk"
        );

        // regular-code chunk (strings.last())
        let li = strings.len() - 1;
        let mut s_reg = strings.clone();
        s_reg[li] = flip_chars(&strings[li], &positions);
        let parts: Vec<&str> = s_reg.iter().map(String::as_str).collect();
        assert_eq!(
            decode(&parts).expect("BCH t=4 corrects the regular-code chunk"),
            card,
            "{n}-error correction failed for the regular-code chunk"
        );
    }
}
```

- [ ] **Step 2: Run T2a**

Run: `cargo test -p mk-codec --test bch_adversarial t2a 2>&1 | tail -20`
Expected: **PASS**. If the `long_dl`/`reg_dl` assertions fail, the fixture's chunk sizing is off — bump the stub count in `multi_chunk_card()` until `strings[0]` lands in 96..=108 (it's deterministic, so iterate the constant once). If `decode(...).expect(...)` fails, that is a genuine correction bug → SPEC §6.

- [ ] **Step 3: clippy gate**

Run: `cargo clippy -p mk-codec --all-targets -- -D warnings 2>&1 | tail -5`
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add crates/mk-codec/tests/bch_adversarial.rs
git commit -m "test(mk-codec): theme 2 T2a — 3/4-error correction through public decode (both code variants)"
```

---

### Task 1.2: T2b — checksum-region + mixed correction

**Files:**
- Modify: `crates/mk-codec/tests/bch_adversarial.rs`

- [ ] **Step 1: Append T2b**

Add to `bch_adversarial.rs`:

```rust
#[test]
fn t2b_checksum_region_and_mixed_correction() {
    let card = multi_chunk_card();
    let strings = encode_with_chunk_set_id(&card, 0).unwrap();
    let li = strings.len() - 1;
    let last = &strings[li];
    let total = last.chars().count();

    // The BCH checksum is the trailing 13 symbols (regular code). Corrupt
    // inside the checksum tail (NOT the data part) — exercises the
    // position-translation `k = L-1-d` (src/string_layer/bch_decode.rs:587)
    // that the existing corpus never reaches.
    let checksum_positions: Vec<usize> = (total - 4..total).collect(); // 4 tail symbols
    let mut s_csum = strings.clone();
    s_csum[li] = flip_chars(last, &checksum_positions);
    let parts: Vec<&str> = s_csum.iter().map(String::as_str).collect();
    assert_eq!(
        decode(&parts).expect("BCH corrects checksum-region errors"),
        card,
        "checksum-region 4-error correction failed"
    );

    // Mixed: 2 in the data part + 2 in the checksum tail (total 4 = t-boundary).
    let mixed: Vec<usize> = vec![11, 12, total - 2, total - 1];
    let mut s_mix = strings.clone();
    s_mix[li] = flip_chars(last, &mixed);
    let parts: Vec<&str> = s_mix.iter().map(String::as_str).collect();
    assert_eq!(
        decode(&parts).expect("BCH corrects mixed data+checksum at the t=4 boundary"),
        card,
        "mixed data+checksum 4-error correction failed"
    );
}
```

- [ ] **Step 2: Run T2b**

Run: `cargo test -p mk-codec --test bch_adversarial t2b 2>&1 | tail -20`
Expected: **PASS**. A failure here is a genuine checksum-region correction bug → SPEC §6. (If the trailing-13-symbol assumption is off, confirm the regular-code checksum width via `src/string_layer/bch.rs` and adjust `total - 4` to stay within the checksum tail.)

- [ ] **Step 3: clippy gate** — `cargo clippy -p mk-codec --all-targets -- -D warnings` → clean.

- [ ] **Step 4: Commit**

```bash
git add crates/mk-codec/tests/bch_adversarial.rs
git commit -m "test(mk-codec): theme 2 T2b — checksum-region + mixed-region correction"
```

---

### Task 1.3: T2c — randomized 5–8-error miscorrection sweep

**Files:**
- Modify: `crates/mk-codec/tests/bch_adversarial.rs`

- [ ] **Step 1: Append the T2c proptest**

Add to `bch_adversarial.rs`:

```rust
proptest! {
    // T2c — randomized miscorrection sweep. Corrupt 5–8 distinct symbols in
    // ONE chunk's data part. The robust, non-flaky property is
    // `decode(perturbed) != Ok(original)`: three outcomes are all legal —
    // Err(BchUncorrectable), Err(CrossChunkHashMismatch), or (≈2⁻³², the
    // accepted 4-byte cross-chunk-hash residual) Ok(a DIFFERENT card). The
    // contract under test is "a ≥5-error corruption never SILENTLY returns the
    // original as if clean." Asserting `.is_err()` would flake ~1-in-4.3e9.
    #[test]
    fn t2c_five_to_eight_error_corruption_never_returns_original(
        card in keycard_strategy(),
        csid in csid_strategy(),
        n_errors in 5usize..=8usize,
        seed in any::<u64>(),
    ) {
        let strings = encode_with_chunk_set_id(&card, csid).unwrap();
        prop_assume!(strings.len() >= 1);
        // Target chunk 0; corrupt n distinct data-part positions (char-index ≥ 11
        // for chunked, ≥ 5 for single-chunk — use ≥ 11 and require enough length).
        let s0 = &strings[0];
        let len = s0.chars().count();
        prop_assume!(len > 11 + n_errors);
        let mut positions = Vec::new();
        let mut x = seed | 1;
        while positions.len() < n_errors {
            x ^= x << 13; x ^= x >> 7; x ^= x << 17;
            let idx = 11 + (x as usize % (len - 11));
            if !positions.contains(&idx) { positions.push(idx); }
        }
        let mut perturbed = strings.clone();
        perturbed[0] = flip_chars(s0, &positions);
        let parts: Vec<&str> = perturbed.iter().map(String::as_str).collect();

        match decode(&parts) {
            Ok(recovered) => prop_assert_ne!(
                recovered, card.clone(),
                "≥5-error corruption silently returned the original card"
            ),
            Err(_) => {} // BchUncorrectable / CrossChunkHashMismatch — both legal
        }
    }
}
```

- [ ] **Step 2: Run T2c**

Run: `cargo test -p mk-codec --test bch_adversarial t2c 2>&1 | tail -20`
Expected: **PASS**. A `prop_assert_ne` failure (decode returned `Ok(original)` despite ≥5 errors) is the genuine "silent acceptance of corruption" bug T2c exists to catch → STOP and apply SPEC §6.

- [ ] **Step 3: clippy gate** → clean.

- [ ] **Step 4: Commit**

```bash
git add crates/mk-codec/tests/bch_adversarial.rs
git commit -m "test(mk-codec): theme 2 T2c — 5-8 error miscorrection sweep (cross-chunk-hash guard)"
```

---

### Task 1.4: T4 — 255/256 policy-id-stub boundary (also exercises >2-chunk real cards)

**Files:**
- Modify: `crates/mk-codec/tests/bch_adversarial.rs`

- [ ] **Step 1: Append T4**

Add to `bch_adversarial.rs`:

```rust
#[test]
fn t4_stub_count_boundary_255_roundtrip_256_reject() {
    let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let sk = bitcoin::secp256k1::SecretKey::from_slice(&[0x07u8; 32]).unwrap();
    let pk = bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &sk);
    let comps: Vec<bitcoin::bip32::ChildNumber> = path.into_iter().copied().collect();
    let xpub = bitcoin::bip32::Xpub {
        network: bitcoin::NetworkKind::Main,
        depth: comps.len() as u8,
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, 0x40]),
        child_number: *comps.last().unwrap(),
        public_key: pk,
        chain_code: bitcoin::bip32::ChainCode::from([0xCCu8; 32]),
    };

    // 255 stubs (the encoder's 1-byte stub_count max) — ~1100-byte bytecode ⇒
    // a many-chunk (>2) real-card round-trip.
    let stubs_255: Vec<[u8; 4]> = (0..255u32).map(|i| (i as u8).to_le_bytes().map(|b| b).into()).collect();
    // (simpler, distinct stubs:)
    let stubs_255: Vec<[u8; 4]> = (0..255u16).map(|i| [i as u8, (i >> 8) as u8, 0xAB, 0xCD]).collect();
    let card_255 = KeyCard::new(stubs_255, None, path.clone(), xpub.clone());
    let strings = encode_with_chunk_set_id(&card_255, 1).expect("255 stubs encodes");
    assert!(strings.len() > 2, "255 stubs must produce a >2-chunk card; got {}", strings.len());
    let parts: Vec<&str> = strings.iter().map(String::as_str).collect();
    assert_eq!(decode(&parts).expect("255-stub card decodes"), card_255);

    // 256 stubs — over the 1-byte cap ⇒ encoder rejects.
    let stubs_256: Vec<[u8; 4]> = (0..256u16).map(|i| [i as u8, (i >> 8) as u8, 0xAB, 0xCD]).collect();
    let card_256 = KeyCard::new(stubs_256, None, path, xpub);
    assert!(
        matches!(
            encode_with_chunk_set_id(&card_256, 1),
            Err(Error::InvalidPolicyIdStubCount)
        ),
        "256 stubs must be rejected with InvalidPolicyIdStubCount"
    );
}
```

> **Implementer note:** delete the first (commented-as-simpler) `stubs_255` line — keep only the `[i as u8, (i>>8) as u8, 0xAB, 0xCD]` form (255 distinct 4-byte stubs). Two lines are shown only to make the intent unambiguous; the second shadows the first.

- [ ] **Step 2: Run T4**

Run: `cargo test -p mk-codec --test bch_adversarial t4 2>&1 | tail -20`
Expected: **PASS**.

- [ ] **Step 3: clippy gate** → clean.

- [ ] **Step 4: Commit**

```bash
git add crates/mk-codec/tests/bch_adversarial.rs
git commit -m "test(mk-codec): T4 — 255-stub roundtrip (>2-chunk) + 256-stub reject boundary"
```

---

### Task 1.5: File the `error.rs:56` doc FOLLOWUP

**Files:**
- Modify: `mnemonic-key/design/FOLLOWUPS.md`

- [ ] **Step 1: Append the entry**

Add to `design/FOLLOWUPS.md` (in the open-items section):

```markdown
### `error-bchuncorrectable-doc-says-8-for-long` — `Error::BchUncorrectable` doc reads "8 for long" but both codes are t=4

- **Surfaced:** 2026-05-29, mk-codec test-hardening cycle (theme 2 T2-doc).
- **Where:** `crates/mk-codec/src/error.rs:56` — `/// substitution capacity (4 for regular, 8 for long).`
- **What:** The parenthetical reads as a correction count, but the long code `BCH(108,93,8)` has `t = 4` (the `8` is the designed minimum distance / syndrome count). Both regular and long correct up to 4 substitutions (`string_layer/bch.rs:376,451`; `bch_decode.rs:566` rejects `deg > 4`). Reword to "(t = 4 for both; the trailing 8 in BCH(•,•,8) is the min-distance, not the correction count)".
- **Why deferred:** doc-only; no behavior impact. Fold into any error-surface touch.
- **Status:** `open`
- **Tier:** `docs`
```

- [ ] **Step 2: Commit**

```bash
git add design/FOLLOWUPS.md
git commit -m "docs(followups): file error.rs:56 t=8/long doc inaccuracy (mk test-hardening T2-doc)"
```

---

## Phase 2 — Theme 3 (indel reject-contract)

### Task 2.1: `indel_reject_contract.rs` — T3a (in-band indel) + T3b (out-of-band length)

**Files:**
- Create: `crates/mk-codec/tests/indel_reject_contract.rs`

- [ ] **Step 1: Write the tests**

Create `crates/mk-codec/tests/indel_reject_contract.rs`:

```rust
//! Theme 3 (SPEC_mk_codec_test_hardening.md §5) — indel reject-contract. BCH
//! is substitution-only; an inserted/deleted symbol (length change) must fail
//! closed. This is the contract the toolkit's `repair --max-indel` oracle
//! relies on: `mnemonic-toolkit/crates/mnemonic-toolkit/src/repair.rs:1001`
//! (`Mk1IndelOracle`) + the comment at `:997-1000` ("mk_codec::decode
//! self-corrects t≤4 UNGUARDED, which would defeat the pure-indel rule").
//!
//! Assertion strength (SPEC §5): T3a/T3b pin a FIXED indel verified to error,
//! so `is_err()`/variant-pin is safe. The weaker `!= Ok(original)` is reserved
//! for the randomized T2c sweep (a ≈2⁻³² cross-chunk-hash collision could make
//! an in-band indel return a DIFFERENT valid card) — do NOT randomize T3a.

use std::str::FromStr;

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use mk_codec::{Error, KeyCard, decode, encode_with_chunk_set_id};

fn fixture_card() -> KeyCard {
    let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(&[0x42u8; 32]).unwrap();
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let comps: Vec<ChildNumber> = path.into_iter().copied().collect();
    let xpub = Xpub {
        network: NetworkKind::Main,
        depth: comps.len() as u8,
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, 0x40]),
        child_number: *comps.last().unwrap(),
        public_key: pk,
        chain_code: ChainCode::from([0xCCu8; 32]),
    };
    // a few stubs → multi-chunk, so the indel lands in a chunk fragment.
    KeyCard::new(
        (0u8..6).map(|i| [i, i, i, i]).collect(),
        Some(Fingerprint::from([0xAA, 0xBB, 0xCC, 0xDD])),
        path,
        xpub,
    )
}

// T3a — a single in-band-length indel (insert one symbol, then delete one)
// must fail closed (`Err`), never self-correct into a different valid card.
#[test]
fn t3a_in_band_single_indel_fails_closed() {
    let card = fixture_card();
    let strings = encode_with_chunk_set_id(&card, 0).unwrap();
    let s0 = &strings[0];

    // INSERT one symbol mid-data-part (char-index 15) → length+1.
    let mut chars: Vec<char> = s0.chars().collect();
    chars.insert(15, 'p');
    let inserted: String = chars.into_iter().collect();
    let mut v_ins = strings.clone();
    v_ins[0] = inserted;
    let parts: Vec<&str> = v_ins.iter().map(String::as_str).collect();
    assert!(
        decode(&parts).is_err(),
        "an inserted symbol must fail closed (never Ok); got {:?}",
        decode(&parts)
    );

    // DELETE one symbol mid-data-part (char-index 15) → length-1.
    let mut chars: Vec<char> = s0.chars().collect();
    chars.remove(15);
    let deleted: String = chars.into_iter().collect();
    let mut v_del = strings.clone();
    v_del[0] = deleted;
    let parts: Vec<&str> = v_del.iter().map(String::as_str).collect();
    assert!(
        decode(&parts).is_err(),
        "a deleted symbol must fail closed (never Ok); got {:?}",
        decode(&parts)
    );
}

// T3b — a delete that pushes a chunk's data-part length into the reserved
// 94/95 gap (or otherwise out of band) is a DETERMINISTIC InvalidStringLength.
#[test]
fn t3b_out_of_band_length_is_invalid_string_length() {
    // Construct a single string whose data-part length, after a delete, lands
    // outside any BCH band. The cleanest deterministic case: take a real chunk
    // and truncate it to a length the band table rejects (94 or 95 data-part
    // symbols → None in bch_code_for_length). We force this by trimming the
    // string to total length = 3 (HRP) + 94 (data-part, reserved) and feeding
    // it; decode must surface InvalidStringLength.
    let card = fixture_card();
    let strings = encode_with_chunk_set_id(&card, 0).unwrap();
    // Find a chunk long enough to trim into the reserved gap.
    let long = strings.iter().max_by_key(|s| s.chars().count()).unwrap();
    let chars: Vec<char> = long.chars().collect();
    // Target total length = 3 + 94 = 97 (data-part 94 = reserved-invalid).
    // (If the chunk has an 8-symbol chunked header counted within the data
    //  part band, adjust the target so bch_code_for_length sees 94/95 — verify
    //  against src/string_layer/bch.rs:117 at impl time.)
    if chars.len() > 97 {
        let trimmed: String = chars[..97].iter().collect();
        match decode(&[trimmed.as_str()]) {
            Err(Error::InvalidStringLength(_)) => {}
            other => panic!("reserved-gap length must be InvalidStringLength; got {other:?}"),
        }
    } else {
        panic!("fixture chunk too short to trim into the reserved gap; enlarge fixture_card()");
    }
}
```

> **Implementer note (T3b):** the exact arithmetic mapping `string length → bch_code_for_length`'s `data_part_len` must be confirmed against `src/string_layer/bch.rs:117` + the decode entry (`decode_string`). If the band table's `data_part_len` excludes the HRP/separator only (not the chunked header), the target total length is `3 + 94 = 97`; if it excludes the header too, use `3 + 8 + 94 = 105`. Pick the value that makes `bch_code_for_length` return `None`; this IS deterministic — verify once and pin the constant. The test's *intent* (a reserved-gap length → `InvalidStringLength`) is fixed; only the trim target may need adjustment.

- [ ] **Step 2: Run T3a + T3b**

Run: `cargo test -p mk-codec --test indel_reject_contract 2>&1 | tail -20`
Expected: **PASS**. If T3a returns `Ok` for an in-band indel, that is a genuine fail-OPEN bug that breaks the toolkit's indel-oracle soundness → STOP, SPEC §6 (this would be a high-severity find). If T3b's variant differs, adjust the trim target per the implementer note (deterministic).

- [ ] **Step 3: clippy gate** → clean.

- [ ] **Step 4: Commit**

```bash
git add crates/mk-codec/tests/indel_reject_contract.rs
git commit -m "test(mk-codec): theme 3 — indel reject-contract (toolkit repair --max-indel oracle)"
```

---

### Task 2.2: File the depth/child seam FOLLOWUP (mk primary + toolkit companion)

**Files:**
- Modify: `mnemonic-key/design/FOLLOWUPS.md`
- Modify: `mnemonic-toolkit/design/FOLLOWUPS.md` (companion)

- [ ] **Step 1: Append the mk-codec primary entry** to `mnemonic-key/design/FOLLOWUPS.md`:

```markdown
### `mk1-depth-child-lossless-by-construction-unenforced` — encoder drops xpub.depth/child_number and reconstructs from path WITHOUT validating agreement

- **Surfaced:** 2026-05-29, mk-codec test-hardening cycle (theme-1 strategy design; SPEC §1.1).
- **Where:** `crates/mk-codec/src/bytecode/xpub_compact.rs:4` (drops depth/child), `:85-106` (`reconstruct_xpub` rebuilds them from `origin_path`), `bytecode/encode.rs:44` (`XpubCompact::from_xpub` silently drops). SPEC `design/SPEC_mk_v0_1.md:263,301` claims "lossless by construction" + removes `XpubDepthMismatch`.
- **What:** The "lossless by construction" claim holds ONLY when the caller passes `xpub.depth == origin_path.len()` and `xpub.child_number == origin_path.last()`. Nothing enforces this; a depth-4 xpub + 3-component path silently round-trips to a DIFFERENT xpub. Decide: (a) re-introduce encode-time `XpubDepthMismatch` (genuinely lossless), OR (b) document the lossy contract + pin it with a test. The toolkit compensates downstream (`mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs:494-503` depth check) — option (a) would let it drop that.
- **Why deferred:** behavior/contract decision (likely MINOR bump + toolkit coordination), out of the test-only test-hardening cycle's scope.
- **Status:** `open`
- **Tier:** `v0.4`
- **Companion:** `mnemonic-toolkit` FOLLOWUP `mk1-depth-child-compensating-check-watch`.
```

- [ ] **Step 2: Append the toolkit companion entry** to `mnemonic-toolkit/design/FOLLOWUPS.md`:

```markdown
### `mk1-depth-child-compensating-check-watch` — toolkit depth-check compensates for mk1's unenforced depth/child reconstruction

- **Surfaced:** 2026-05-29, mk-codec test-hardening cycle.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:494-503` (the SPEC §4.5 depth==path check) compensates for `mk-codec`'s unvalidated depth/child reconstruction.
- **What:** If `mk-codec` resolves its `mk1-depth-child-lossless-by-construction-unenforced` FOLLOWUP via option (a) (encode-time `XpubDepthMismatch`), this toolkit-side compensating check may become redundant and can be reviewed for removal. Until then it is load-bearing — do not drop it.
- **Status:** `open`
- **Tier:** `monitoring`
- **Companion:** `mnemonic-key` FOLLOWUP `mk1-depth-child-lossless-by-construction-unenforced`.
```

- [ ] **Step 3: Commit (mk repo)**

```bash
git add design/FOLLOWUPS.md
git commit -m "docs(followups): file mk1 depth/child lossless-by-construction seam (deferred from test-hardening)"
```

> The toolkit companion is committed separately in the `mnemonic-toolkit` repo (it is a different git tree); note it for the cross-repo step at cycle close.

---

## Phase 3 — Verify + end-of-cycle R0

### Task 3.1: Full verification

**Files:** none (verification only).

- [ ] **Step 1: Full test suite**

Run: `cargo test -p mk-codec 2>&1 | tail -15`
Expected: all green — the 3 existing test files + in-src unit tests + the 4 new files. Record the new total.

- [ ] **Step 2: clippy + fmt gates (CI parity)**

Run: `cargo clippy -p mk-codec --all-targets -- -D warnings 2>&1 | tail -5` → clean.
Run: `cargo fmt -p mk-codec -- --check 2>&1 | tail -5` → clean (run `cargo fmt -p mk-codec` if not).

- [ ] **Step 3: Confirm proptest determinism artifacts**

Run: `git status --porcelain crates/mk-codec/tests/`
Expected: no untracked `proptest-regressions/` (gitignored). If any test shrank a counterexample, that's a real finding — investigate before proceeding.

### Task 3.2: End-of-cycle opus R0

- [ ] **Step 1: Dispatch the end-of-cycle architect R0** (opus feature-dev:code-reviewer) over the full diff (`git diff main...HEAD`), checking: the strategy generates only encodable cards; T2a/T2b genuinely exercise both code variants (the band assertions held); T2c's `!= Ok(original)` is correctly implemented; T3a/T3b assertions are sound; no production code changed (test-only); clippy/CI parity. Persist verbatim to `design/agent-reports/mk-test-hardening-end-of-cycle-R0-review.md`. Fold to 0C/0I.

- [ ] **Step 2: Ship decision (SPEC §6)**

If no bug surfaced: commit any R0 folds, then the branch is ready to merge to `main` (test-only, **no version bump**). If a bug was fixed inline: bump mk-codec (PATCH/MINOR), its own R0 on the fix, and refresh the `mnemonic-toolkit` git-dep pin. Surface the outcome + the merge/push decision to the user (do not push without explicit go).

---

## Self-Review

**Spec coverage:** P1/P2 (§3 theme 1) → Task 0.3. T2a/T2b/T2c (§4 theme 2) → Tasks 1.1/1.2/1.3. T2-doc (§4) → Task 1.5. T3a/T3b (§5 theme 3) → Task 2.1. T4 255-stub boundary (§7) → Task 1.4. depth/child seam (§1.1) → Task 2.2. `proptest`+gitignore+`-D warnings` (§3/§6/§7) → Tasks 0.1/0.3. Branch/SemVer/bug-handling (§6) → Task 3.2. End-of-cycle R0 (§8) → Task 3.2. All spec sections mapped.

**Placeholder scan:** every code step contains complete code. The two `(verify at impl)` notes (T3b trim target; rust-bitcoin API names) are explicitly-flagged deterministic-arithmetic / API-arity confirmations, not behavioral TBDs — the test intent is fixed.

**Type consistency:** `keycard_strategy`/`csid_strategy`/`flip_chars`/`STANDARD_PATHS` defined in `common/mod.rs` (Task 0.2) and used by the same names in 0.3/1.1/1.3. `multi_chunk_card`/`data_part_len` defined in 1.1, reused in 1.2/1.3. `KeyCard::new(stubs, fp, path, xpub)` and `Error::{InvalidPolicyIdStubCount, InvalidStringLength, BchUncorrectable, CrossChunkHashMismatch}` used consistently with the verified source.
