# mk SLIP-0132 acceptance (A2) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development (or executing-plans). Steps use checkbox (`- [ ]`) syntax.

**Goal:** `mk encode`/`verify --xpub` accept SLIP-0132 prefixes (ypub/zpub/Ypub/Zpub + testnet upub/vpub/Upub/Vpub), normalizing to canonical xpub/tpub, with a stderr note and refuse-on-prefix↔path-mismatch (exit 64, actionable message). mk-codec untouched.

**Architecture:** New `crates/mk-cli/src/slip132.rs` duplicates the CI-tested table from `mnemonic-toolkit/src/slip0132.rs` (mk-cli is upstream, can't dep the toolkit; byte-parity by unit test). A shared `parse_xpub_normalized(s, origin_path)` helper in `cmd/mod.rs` (normalize → note → mismatch-refuse) replaces `parse_xpub` in `encode` + `verify`. No wire change.

**Tech Stack:** Rust, clap, `bitcoin::base58`/`bip32`, `assert_cmd`. CI: `cargo clippy --all-targets -- -D warnings` + `missing_docs`.

**Spec:** `design/SPEC_mk_slip0132_acceptance.md` (R0 GREEN). **Spec review:** `design/agent-reports/mk-slip0132-spec-R0-review.md`.
**Plan R0 gate:** **GREEN** — R0 RED (1C/1I) → R1 GREEN (0C/0I); `design/agent-reports/mk-slip0132-plan-R{0,1}-review.md`. Cleared for code (per-phase + end-of-cycle R0 still apply).
**Source SHA:** mk `main` `fc2341b` (re-grep before each edit). **Ship order:** Phase A (mk-cli → `mk-cli-v0.7.0`) → Phase B (toolkit re-pin → `mnemonic-toolkit-v0.38.4`). Per-phase + end-of-cycle opus reviews; tags/publish gated on user authorization.

---

## Phase A — mk-cli (repo `mnemonic-key`, branch `mk-slip0132-acceptance`)

### Task A1: `slip132.rs` module + wire `encode` (core lands + live callers)

> mk-cli is bin-only — a `pub fn` exercised only in `#[cfg(test)]` still trips `dead_code` under `-D warnings` (Phase-2 lesson). So the module + its first real caller (`encode`) land together.

**Files:** Create `crates/mk-cli/src/slip132.rs`; modify `crates/mk-cli/src/main.rs` (`mod slip132;`), `crates/mk-cli/src/cmd/mod.rs` (add `parse_xpub_normalized`), `crates/mk-cli/src/cmd/encode.rs` (use it); Test: `crates/mk-cli/tests/cli_slip132.rs` (new).

- [ ] **Step 1: Write the failing encode happy-path integration test.** Create `crates/mk-cli/tests/cli_slip132.rs`. Build a zpub fixture by version-swapping the corpus `V2_84_MAIN` (a depth-3 `m/84'/0'/0'` xpub from `cli_address.rs:17`) to the zpub version bytes, via a local inverse-swap helper:

```rust
use std::process::Command;
use assert_cmd::cargo::CommandCargoExt;
use bitcoin::base58;

const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";

/// Re-version a canonical xpub string into a SLIP-0132 form (inverse of normalize).
fn to_slip132(xpub_str: &str, version: [u8; 4]) -> String {
    let mut data = base58::decode_check(xpub_str).unwrap();
    data[0..4].copy_from_slice(&version);
    base58::encode_check(&data)
}
const ZPUB_V: [u8; 4] = [0x04, 0xB2, 0x47, 0x46];
const NOTE_ZPUB: &str = "note: --xpub was a SLIP-0132 zpub";

#[test]
fn encode_accepts_zpub_with_matching_path() {
    let zpub = to_slip132(V2_84_MAIN, ZPUB_V);
    let out = Command::cargo_bin("mk").unwrap()
        .args(["encode", "--xpub", &zpub, "--origin-path", "m/84h/0h/0h",
               "--policy-id-stub", "deadbeef", "--privacy-preserving"])
        .output().unwrap();
    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains(NOTE_ZPUB), "missing SLIP-0132 note; stderr={stderr}");
    // The emitted mk1 must decode to the SAME key as the canonical xpub at this path.
    let canon = Command::cargo_bin("mk").unwrap()
        .args(["encode", "--xpub", V2_84_MAIN, "--origin-path", "m/84h/0h/0h",
               "--policy-id-stub", "deadbeef", "--privacy-preserving"])
        .output().unwrap();
    assert_eq!(out.stdout, canon.stdout, "zpub-derived mk1 must equal xpub-derived mk1");
}
```

- [ ] **Step 2: Run it; verify FAIL** (`InvalidXpubVersion`/usage error today): `cargo test -p mk-cli --test cli_slip132 encode_accepts_zpub_with_matching_path` → FAIL.

- [ ] **Step 3: Create `crates/mk-cli/src/slip132.rs`.** Model on `mnemonic-toolkit/crates/mnemonic-toolkit/src/slip0132.rs` (`normalize_xpub_prefix` :66-95). Re-grep that file to confirm the 8 version-byte constants before pasting.

```rust
//! SLIP-0132 extended-key prefix acceptance (input normalization).
//!
//! Duplicates the CI-tested table from `mnemonic-toolkit/src/slip0132.rs`
//! (mk-cli is upstream of the toolkit and cannot depend on it; byte-parity is
//! guarded by `slip132_version_bytes_match_slip0132` below). Decode-swap-reencode
//! at the base58check layer — key material is unchanged; only the 4 version bytes.

use std::str::FromStr;

use bitcoin::base58;
use bitcoin::bip32::{ChildNumber, DerivationPath, Xpub};

use crate::error::{CliError, Result};

const XPUB_MAINNET: [u8; 4] = [0x04, 0x88, 0xB2, 0x1E];
const TPUB_TESTNET: [u8; 4] = [0x04, 0x35, 0x87, 0xCF];

/// A detected non-canonical SLIP-0132 variant + its implied origin-path shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slip132Variant {
    Ypub, Zpub, YpubMultisig, ZpubMultisig,       // mainnet  → xpub
    Upub, Vpub, UpubMultisig, VpubMultisig,       // testnet  → tpub
}

impl Slip132Variant {
    /// Display label for the stderr note, e.g. "zpub (BIP-84 P2WPKH)".
    pub fn label(self) -> &'static str {
        use Slip132Variant::*;
        match self {
            Ypub => "ypub (BIP-49 P2SH-P2WPKH)",
            Zpub => "zpub (BIP-84 P2WPKH)",
            YpubMultisig => "Ypub (BIP-48 P2WSH-P2SH multisig)",
            ZpubMultisig => "Zpub (BIP-48 P2WSH multisig)",
            Upub => "upub (testnet BIP-49 P2SH-P2WPKH)",
            Vpub => "vpub (testnet BIP-84 P2WPKH)",
            UpubMultisig => "Upub (testnet BIP-48 P2WSH-P2SH multisig)",
            VpubMultisig => "Vpub (testnet BIP-48 P2WSH multisig)",
        }
    }
    /// Canonical neutral form this variant normalizes to ("xpub" or "tpub").
    pub fn canonical_label(self) -> &'static str {
        use Slip132Variant::*;
        match self { Upub | Vpub | UpubMultisig | VpubMultisig => "tpub", _ => "xpub" }
    }
    /// Does `path` satisfy this variant's implied (HARDENED) shape? (R0 M3)
    pub fn path_matches(self, path: &DerivationPath) -> bool {
        let c: &[ChildNumber] = path.as_ref();
        let h = |x: Option<&ChildNumber>, idx: u32|
            matches!(x, Some(ChildNumber::Hardened { index }) if *index == idx);
        use Slip132Variant::*;
        match self {
            Ypub | Upub => h(c.first(), 49),
            Zpub | Vpub => h(c.first(), 84),
            YpubMultisig | UpubMultisig => h(c.first(), 48) && h(c.get(3), 1),
            ZpubMultisig | VpubMultisig => h(c.first(), 48) && h(c.get(3), 2),
        }
    }
    /// Actionable remediation message when `path` does not match (R0 §5).
    pub fn mismatch_help(self, path: &DerivationPath) -> String {
        use Slip132Variant::*;
        let (expects, alt) = match self {
            Ypub | Upub => ("purpose 49' (e.g. m/49'/0'/0')", "supply the zpub/xpub for a different script type"),
            Zpub | Vpub => ("purpose 84' (e.g. m/84'/0'/0')", "supply the ypub for a 49' path"),
            YpubMultisig | UpubMultisig => ("m/48'/<coin>'/<account>'/1'", "use a Zpub for a 2' path, or xpub"),
            ZpubMultisig | VpubMultisig => ("m/48'/<coin>'/<account>'/2'", "use a Ypub for a 1' path, or xpub"),
        };
        format!(
            "SLIP-0132/origin-path mismatch — --xpub is a {} which expects --origin-path {}, but --origin-path is {}. \
             To engrave a backup, reconcile them: match the path to the prefix, or {}.",
            self.label(), expects, path, alt
        )
    }
}

/// Detect a SLIP-0132 prefix, normalize to canonical xpub/tpub, and parse.
/// Returns `(canonical Xpub, Some(variant))` for SLIP-0132 input,
/// `(Xpub, None)` for canonical xpub/tpub. Unrecognized versions fall through
/// to `Xpub::from_str`'s existing error (preserves today's message).
pub fn detect_and_normalize(s: &str) -> Result<(Xpub, Option<Slip132Variant>)> {
    use Slip132Variant::*;
    let from_str = |s: &str| -> Result<Xpub> {
        Xpub::from_str(s).map_err(|e| CliError::UsageError(format!("invalid xpub {s:?}: {e}")))
    };
    let Ok(data) = base58::decode_check(s) else { return Ok((from_str(s)?, None)); };
    if data.len() < 4 { return Ok((from_str(s)?, None)); }
    let ver: [u8; 4] = data[0..4].try_into().unwrap();
    let (swap, variant) = match ver {
        [0x04, 0x9D, 0x7C, 0xB2] => (XPUB_MAINNET, Ypub),
        [0x04, 0xB2, 0x47, 0x46] => (XPUB_MAINNET, Zpub),
        [0x02, 0x95, 0xB4, 0x3F] => (XPUB_MAINNET, YpubMultisig),
        [0x02, 0xAA, 0x7E, 0xD3] => (XPUB_MAINNET, ZpubMultisig),
        [0x04, 0x4A, 0x52, 0x62] => (TPUB_TESTNET, Upub),
        [0x04, 0x5F, 0x1C, 0xF6] => (TPUB_TESTNET, Vpub),
        [0x02, 0x42, 0x89, 0xEF] => (TPUB_TESTNET, UpubMultisig),
        [0x02, 0x57, 0x54, 0x83] => (TPUB_TESTNET, VpubMultisig),
        _ => return Ok((from_str(s)?, None)), // canonical xpub/tpub OR unknown
    };
    let mut swapped = data;
    swapped[0..4].copy_from_slice(&swap);
    let reencoded = base58::encode_check(&swapped);
    Ok((from_str(&reencoded)?, Some(variant)))
}
```

- [ ] **Step 4: Register** `mod slip132;` in `crates/mk-cli/src/main.rs` (module block).

- [ ] **Step 5: Add `parse_xpub_normalized` to `crates/mk-cli/src/cmd/mod.rs`** (a sibling of `parse_xpub`; the new helper adds the note + check). **NOTE (R0 C1):** `parse_xpub` has ONLY the encode + verify callers — after both are rewired (encode here, verify in A3) it becomes dead code that FAILS `-D warnings` in this bin-only crate, so A3 deletes it. (Do not claim a "non-card caller"; there is none.)

```rust
/// Parse an xpub, accepting SLIP-0132 prefixes (normalized to canonical xpub/tpub).
/// Emits a stderr note on normalization; refuses (UsageError) if a non-canonical
/// prefix's implied script type contradicts `origin_path` (when supplied).
pub fn parse_xpub_normalized(s: &str, origin_path: Option<&DerivationPath>) -> Result<Xpub> {
    let (xpub, variant) = crate::slip132::detect_and_normalize(s)?;
    if let Some(v) = variant {
        eprintln!(
            "note: --xpub was a SLIP-0132 {}; normalized to canonical {} — the engraved card's script type derives from the origin path",
            v.label(), v.canonical_label()
        );
        if let Some(path) = origin_path {
            if !v.path_matches(path) {
                return Err(CliError::UsageError(v.mismatch_help(path)));
            }
        }
    }
    Ok(xpub)
}
```
(Add `use bitcoin::bip32::DerivationPath;` if not already imported.)

- [ ] **Step 6: Wire `encode`** (`crates/mk-cli/src/cmd/encode.rs`): the path is parsed at ~:84 (`--origin-path` is required). Replace `let xpub = parse_xpub(&args.xpub)?;` (~:85) with:
```rust
let xpub = parse_xpub_normalized(&args.xpub, Some(&path))?;
```
(Update the `use` of `parse_xpub` → `parse_xpub_normalized` at `:11`.)

- [ ] **Step 7: Run the happy-path test + clippy.** `cargo test -p mk-cli --test cli_slip132 encode_accepts_zpub_with_matching_path` → PASS. `cargo clippy -p mk-cli --all-targets -- -D warnings` → clean. **CRITICAL (R0 I1c):** the gate here is `dead_code`, not `missing_docs` (the latter is already crate-allowed). Every `slip132` `pub` item — the `Slip132Variant` enum + `label`/`canonical_label`/`path_matches`/`mismatch_help`/`detect_and_normalize` — must be reachable from NON-test code via the encode→`parse_xpub_normalized` path (a `#[cfg(test)]`-only use does NOT keep bin-target items live). `mismatch_help` is reached on the refuse branch (a real use). Doc every pub item anyway.

- [ ] **Step 8: Commit** (stage explicitly):
```
git add crates/mk-cli/src/slip132.rs crates/mk-cli/src/main.rs crates/mk-cli/src/cmd/mod.rs crates/mk-cli/src/cmd/encode.rs crates/mk-cli/tests/cli_slip132.rs
git commit -m "feat(mk-cli): accept SLIP-0132 prefixes on encode (normalize + note) (A2)"
```

### Task A2: unit tests (byte-parity + predicate) + encode mismatch/multisig/canonical cells

**Files:** `crates/mk-cli/src/slip132.rs` (`#[cfg(test)] mod tests`), `crates/mk-cli/tests/cli_slip132.rs`.

- [ ] **Step 1: Unit tests in `slip132.rs`.** (a) `slip132_version_bytes_match_slip0132`: assert each of the 8 match-arm version byte-arrays equals the SLIP-0132 literal (byte-parity drift guard, mirroring the toolkit). (b) `normalize_zpub_yields_same_key`: `detect_and_normalize(zpub)` returns `Some(Zpub)` and an `Xpub` whose `depth`/`child_number`/`fingerprint`/`public_key`/`chain_code` equal those of the canonical xpub. (c) `canonical_xpub_is_none`: `detect_and_normalize(xpub)` → `None`. (d) `path_predicate_truth_table`: hardened `m/84'/0'/0'` matches `Zpub` but `m/84/0/0` (unhardened) does NOT (R0 M3); `m/49'/0'/0'` matches `Ypub` not `Zpub`; `m/48'/0'/0'/2'` matches `ZpubMultisig` not `YpubMultisig`; a 2-component path matches no multisig variant (short-path guard). Build fixtures via an inverse-swap helper or `Xpub` construction.

- [ ] **Step 2: Run unit tests** → all pass. `cargo test -p mk-cli --lib slip132` (or `--bin mk` per the crate's test target — re-grep how mk-cli runs unit tests; Phase-2 used bin-target).

- [ ] **Step 3: encode integration cells** (`tests/cli_slip132.rs`): add
  - `encode_zpub_path_mismatch_refuses`: `encode --xpub <zpub> --origin-path m/49'/0'/0' …` → exit **64**, stderr contains `SLIP-0132/origin-path mismatch` + `expects --origin-path purpose 84'`.
  - `encode_Zpub_multisig_match`: `--xpub <Zpub from V1_48_MULTISIG> --origin-path m/48'/0'/0'/2' …` → exit 0 + note.
  - `encode_Zpub_multisig_index_mismatch`: `<Zpub> --origin-path m/48'/0'/0'/1'` → exit 64.
  - `encode_canonical_xpub_no_note`: `--xpub V2_84_MAIN --origin-path m/84'/0'/0' …` → exit 0, stderr does NOT contain `SLIP-0132`.
  (Add `Zpub` version const `[0x02,0xAA,0x7E,0xD3]` + reuse `V1_48_MULTISIG` from `cli_address.rs:19`.)

- [ ] **Step 4: Run all + clippy.** `cargo test -p mk-cli --test cli_slip132` + unit + `cargo clippy -p mk-cli --all-targets -- -D warnings` → green.

- [ ] **Step 5: Commit.**
```
git add crates/mk-cli/src/slip132.rs crates/mk-cli/tests/cli_slip132.rs
git commit -m "test(mk-cli): SLIP-0132 byte-parity + predicate units + encode mismatch/multisig/canonical cells (A2)"
```

### Task A3: wire `verify` + verify cells + stderr-ordering cell

**Files:** `crates/mk-cli/src/cmd/verify.rs`, `crates/mk-cli/tests/cli_slip132.rs`.

- [ ] **Step 1: Failing verify cells.** Add to `tests/cli_slip132.rs`:
  - `verify_zpub_without_path_ok`: build a card (via `mk encode` of the canonical xpub at m/84'/0'/0'), then `verify --xpub <zpub> <card…>` (NO `--origin-path`) → exit 0 (key-material match), stderr contains the note, NO mismatch error.
  - `verify_zpub_path_mismatch_refuses`: `verify --xpub <zpub> --origin-path m/49'/0'/0' <card…>` → exit **64** (UsageError — distinct from verify's value-`ContentMismatch` exit 4; R0 M1), stderr contains `SLIP-0132/origin-path mismatch`.

- [ ] **Step 2: Run; verify FAIL** (verify still rejects zpub today).

- [ ] **Step 3: Wire `verify`** (`crates/mk-cli/src/cmd/verify.rs`): `--origin-path` is `Option<String>` (:31); the xpub is parsed at ~:53. Parse the origin path (if `Some`) BEFORE the xpub, then:
```rust
let want_path: Option<DerivationPath> = match &args.origin_path {
    Some(p) => Some(parse_derivation_path(p)?),
    None => None,
};
// ... where the xpub is parsed (~:53):
let want = parse_xpub_normalized(expected, want_path.as_ref())?;
```
Reuse `want_path` for the existing origin_path content-match block (`:84-93`): that block currently re-parses `args.origin_path` via `parse_derivation_path` — replace its re-parse with the already-computed `want_path` (e.g. `if let Some(want) = &want_path { if *want != card.origin_path { … } }`) so the path is parsed once. Update the `use parse_xpub` → `parse_xpub_normalized` at `:11`.

- [ ] **Step 3b: Delete the now-orphaned `parse_xpub`** from `crates/mk-cli/src/cmd/mod.rs` (R0 C1 — both callers are now rewired; leaving it = `dead_code` → `-D warnings` failure). Remove the `pub fn parse_xpub` (`:57-59`) and confirm no remaining references (`grep -rn parse_xpub crates/mk-cli/src` → only `parse_xpub_normalized`).

- [ ] **Step 4: stderr-ordering cell (R0 M2).** Add `encode_emits_both_slip132_note_and_watchonly_advisory`: `encode --xpub <zpub> --origin-path m/84h/0h/0h …` → stderr contains BOTH `note: --xpub was a SLIP-0132 zpub` AND the Phase-2 watch-only advisory written with the **em-dash U+2014** exactly: `const WATCH_ONLY: &str = "note: stdout is watch-only \u{2014} public keys only, cannot spend";` (R0 I1b — assert `\u{2014}`, NOT a hyphen, or the `contains` is vacuous). Assert both present AND the SLIP-0132 note's byte offset < the watch-only advisory's offset (SLIP-0132 fires at parse-time, watch-only after stdout — re-grep `encode.rs` to confirm the order).

- [ ] **Step 5: Run all + clippy** → green.

- [ ] **Step 6: Commit.**
```
git add crates/mk-cli/src/cmd/verify.rs crates/mk-cli/src/cmd/mod.rs crates/mk-cli/tests/cli_slip132.rs
git commit -m "feat(mk-cli): accept SLIP-0132 on verify (path-optional); drop orphaned parse_xpub; stderr-ordering cell (A2)"
```

### Task A4: FOLLOWUP + version bump + per-phase review + commit

**Files:** `design/FOLLOWUPS.md`, `crates/mk-cli/Cargo.toml`.

- [ ] **Step 1: File the FOLLOWUP as resolved** in `design/FOLLOWUPS.md`: new entry `mk-slip0132-prefix-acceptance`, Status `resolved` (this cycle), describing the accept+normalize+note+refuse-on-mismatch behavior, mk-cli v0.7.0, mk-codec untouched. No sibling companion (mk-only).
- [ ] **Step 2: Bump** `crates/mk-cli/Cargo.toml` `version = "0.6.1"` → `"0.7.0"` (MINOR — additive). Skip CHANGELOG (mk-cli CHANGELOG not lockstep-maintained — Phase-2 precedent). `cargo build -p mk-cli` to refresh the lockfile; stage `Cargo.lock` if changed.
- [ ] **Step 3: Full crate gate.** `cargo test -p mk-cli` + `cargo clippy -p mk-cli --all-targets -- -D warnings` → green.
- [ ] **Step 4: Per-phase opus review** → persist to `design/agent-reports/mk-slip0132-phase-A-R0-review.md`; loop to 0C/0I.
- [ ] **Step 5: Commit (no tag).**
```
git add design/FOLLOWUPS.md crates/mk-cli/Cargo.toml Cargo.lock
git commit -m "release(mk-cli): v0.7.0 — SLIP-0132 prefix acceptance (A2); resolve FOLLOWUP"
```

---

## Phase B — toolkit lockstep (repo `mnemonic-toolkit`, new branch `mk-slip0132-acceptance`)

> Requires the `mk-cli-v0.7.0` tag to exist on the remote before the toolkit `manual` CI runs (it installs the pinned tag). Local edits + audit don't need it (the gate compares strings); the tag is pushed at the ship gate.

### Task B1: re-pin 3 mk-cli sites + manual prose + toolkit bump + audit

**Files:** `scripts/install.sh:41`, `.github/workflows/manual.yml:77`, `.github/workflows/quickstart.yml:71`, `docs/manual/src/40-cli-reference/44-mk-cli.md`, `crates/mnemonic-toolkit/Cargo.toml`.

- [ ] **Step 1: Re-pin the 3 mk-cli sites** `mk-cli-v0.6.1` → `mk-cli-v0.7.0` (re-grep exact lines; do NOT touch md/ms pins). Run the `sibling-pin-check.yml` inline logic + `actionlint` → exit 0.
- [ ] **Step 2: Manual prose** — add a SLIP-0132 acceptance note to `docs/manual/src/40-cli-reference/44-mk-cli.md` (mk `encode`/`verify --xpub` accept ypub/zpub/…; normalized to canonical xpub; mismatch refused). No flag-coverage lint impact (no new flag).
- [ ] **Step 3: Bump toolkit** `crates/mnemonic-toolkit/Cargo.toml` `0.38.3` → `0.38.4`; update both README version markers (`readme_version_current` gate — Phase-2 lesson); CHANGELOG entry; `cargo build` → stage `Cargo.lock`.
- [ ] **Step 4: Manual audit + suite.** Build all 4 binaries (mnemonic from this branch; md/ms/mk from their default branches — mk from `mk-cli-v0.7.0` source). `make -C docs/manual audit MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=…` → exit 0. Re-sweep `docs/manual/transcripts/` for any `$MK_BIN` ypub/zpub ingest (none expected → no transcript re-capture). `cargo test -p mnemonic-toolkit` + clippy → green.
- [ ] **Step 5: Commit (no tag).**
```
git add scripts/install.sh .github/workflows/manual.yml .github/workflows/quickstart.yml docs/manual/src/40-cli-reference/44-mk-cli.md crates/mnemonic-toolkit/Cargo.toml Cargo.lock README.md crates/mnemonic-toolkit/README.md CHANGELOG.md
git commit -m "chore(pins+manual): re-pin mk-cli v0.7.0 + document SLIP-0132 acceptance (v0.38.4) (A2)"
```

### Task B2: end-of-cycle review + persist design artifacts

- [ ] **Step 1: End-of-cycle opus review** across both repos (mk diff + toolkit diff) → persist to `mnemonic-key/design/agent-reports/mk-slip0132-end-of-cycle-R0-review.md`; loop to 0C/0I.
- [ ] **Step 2: Commit the design audit trail** (CLAUDE.md mandate — Phase-2 lesson: don't leave SPEC/plan/reviews uncommitted): in `mnemonic-key`, `git add design/SPEC_mk_slip0132_acceptance.md design/IMPLEMENTATION_PLAN_mk_slip0132_acceptance.md design/agent-reports/mk-slip0132-*` → commit `docs(design): persist A2 spec + plan + R0 reviews`.

---

## Self-review (spec coverage)
- Spec §1-2 (accept+normalize+table) → A1 (`slip132.rs`). §3 normalize → A1 Step 3. §4 note → A1 Step 5. §5 mismatch+actionable → A1 (`mismatch_help`) + A2/A3 cells. §6 encode/verify boundary → A1 (encode) + A3 (verify path-optional). §7 exit codes → A2/A3 cells (64). §8 lockstep → B1 (3 pins, no GUI, toolkit PATCH). §9 tests → A1-A3. §10/§11 footguns (hardened predicate M3, base58 re-checksum, short-path guard, verify-without-path, canonical-no-check, dead-code, exit-64-vs-4 M1, stderr-order M2, precedent) → A1-A3 cells + the module design.
- **Tag/publish gated on user authorization** — executor stops at the pre-tag commit.
