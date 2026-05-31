# `mk address` + `mk derive` Implementation Plan

> **For agentic workers:** per-phase TDD (tests before impl); per-phase opus review to 0C/0I before advancing; persist reviews to `design/agent-reports/`. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Add two read-only public-derivation subcommands to mk-cli — `mk address` (N receive/change addresses from a card) and `mk derive` (a child xpub at a relative unhardened path).

**Architecture:** A shared `cmd/derive_support.rs` module holds `AddressType`, `CliNetwork`, `infer_address_type`, and `render_address` (the DRY core). Each subcommand is a focused `cmd/*.rs` following the existing decode/inspect pattern; both consume a decoded `KeyCard` and a `Secp256k1::verification_only()` context. No private keys, no signing.

**Tech stack:** Rust, `bitcoin 0.32` (already a dep), `clap` derive, `serde_json::json!`. Source spec: `design/SPEC_mk_derive_address.md` (R1 GREEN @ source SHA `ed2596d`). SemVer mk-cli 0.5.0 → 0.6.0.

**Authoritative source facts (verified @ ed2596d):** `KeyCard { policy_id_stubs, origin_fingerprint: Option<Fingerprint>, origin_path: DerivationPath, xpub: Xpub }` (`mk-codec/src/key_card.rs:24`); `mk_codec::decode(&[&str]) -> Result<KeyCard>`; helpers `read_mk1_strings`/`parse_derivation_path`/`fmt_fingerprint` (`mk-cli/src/cmd/mod.rs:80/47/73`); `CliError::UsageError(String) => exit 64` (`error.rs:85`); `json!({"schema_version":1,...})` house style (`decode.rs:55`); `card.xpub.fingerprint()` (`inspect.rs:44`); `Secp256k1::verification_only()` (M3 — not used in mk-cli today; precedent toolkit `verify_message.rs:55`).

---

## File structure

- **Create** `crates/mk-cli/src/cmd/derive_support.rs` — `AddressType`, `CliNetwork`, `AddressTypeInference`, `infer_address_type`, `render_address`, `secp_verify` + unit tests.
- **Create** `crates/mk-cli/src/cmd/address.rs` — `AddressArgs` + `run`.
- **Create** `crates/mk-cli/src/cmd/derive.rs` — `DeriveArgs` + `run`.
- **Modify** `crates/mk-cli/src/cmd/mod.rs` — `pub mod derive_support; pub mod address; pub mod derive;`.
- **Modify** `crates/mk-cli/src/main.rs` — two `Command` arms + dispatch + `is_json_mode`.
- **Modify** `crates/mk-cli/src/cmd/gui_schema.rs` — add `address` + `derive` to `build_schema`.
- **Create** `crates/mk-cli/tests/cli_address.rs`, `crates/mk-cli/tests/cli_derive.rs` — integration tests.
- **Modify** `crates/mk-cli/Cargo.toml` + workspace — version 0.5.0 → 0.6.0; `CHANGELOG.md`.
- **Lockstep (separate paired changes):** `mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md`; `mnemonic-gui/src/schema/mk.rs` + `pinned-upstream.toml`; toolkit `install.sh`/`manual.yml`/`quickstart.yml`.

---

## Phase 0 — shared `derive_support` module

### Task 0.1: `AddressType` + `CliNetwork` enums

**Files:** Create `crates/mk-cli/src/cmd/derive_support.rs`; Modify `crates/mk-cli/src/cmd/mod.rs`.

- [ ] **Step 1 — failing test** (in `derive_support.rs`):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn address_type_kebab_values() {
        use clap::ValueEnum;
        assert_eq!(AddressType::P2shP2wpkh.to_possible_value().unwrap().get_name(), "p2sh-p2wpkh");
        assert_eq!(AddressType::P2wpkh.to_possible_value().unwrap().get_name(), "p2wpkh");
    }
    #[test]
    fn network_lower_values_and_hrp() {
        use clap::ValueEnum;
        assert_eq!(CliNetwork::Mainnet.to_possible_value().unwrap().get_name(), "mainnet");
        assert_eq!(CliNetwork::Regtest.known_hrp(), bitcoin::KnownHrp::Regtest);
    }
}
```
- [ ] **Step 2 — run, expect FAIL** (types absent): `cargo test -p mk-cli --lib derive_support`.
- [ ] **Step 3 — implement:**
```rust
use bitcoin::{KnownHrp, NetworkKind};

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum AddressType { P2pkh, P2shP2wpkh, P2wpkh, P2tr }

#[derive(Copy, Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
#[clap(rename_all = "lower")]
pub enum CliNetwork { Mainnet, Testnet, Signet, Regtest }

impl CliNetwork {
    pub fn network_kind(self) -> NetworkKind {
        match self { CliNetwork::Mainnet => NetworkKind::Main, _ => NetworkKind::Test }
    }
    pub fn known_hrp(self) -> KnownHrp {
        match self {
            CliNetwork::Mainnet => KnownHrp::Mainnet,
            CliNetwork::Testnet | CliNetwork::Signet => KnownHrp::Testnets,
            CliNetwork::Regtest => KnownHrp::Regtest,
        }
    }
    pub fn label(self) -> &'static str {
        match self { CliNetwork::Mainnet=>"mainnet", CliNetwork::Testnet=>"testnet",
                     CliNetwork::Signet=>"signet", CliNetwork::Regtest=>"regtest" }
    }
}
```
(Verify `KnownHrp::Testnets` is the 0.32 variant name during impl — adjust to the actual enum (`KnownHrp::Testnets` in bitcoin 0.32); `network_kind()`/`known_hrp()` mirror toolkit `network.rs:19-46`.)
- [ ] **Step 4 — run, expect PASS.**
- [ ] **Step 5 — commit:** `git add crates/mk-cli/src/cmd/derive_support.rs crates/mk-cli/src/cmd/mod.rs && git commit -m "feat(mk): derive_support AddressType + CliNetwork enums"`

### Task 0.2: `infer_address_type` (the I2 account-depth gate)

**Files:** Modify `crates/mk-cli/src/cmd/derive_support.rs`.

- [ ] **Step 1 — failing test:**
```rust
use bitcoin::bip32::DerivationPath;
use std::str::FromStr;
fn p(s: &str) -> DerivationPath { DerivationPath::from_str(s).unwrap() }

#[test]
fn infer_account_depth_only() {
    assert_eq!(infer_address_type(&p("m/84'/0'/0'")), AddressTypeInference::Inferred(AddressType::P2wpkh));
    assert_eq!(infer_address_type(&p("m/44'/0'/0'")), AddressTypeInference::Inferred(AddressType::P2pkh));
    assert_eq!(infer_address_type(&p("m/49'/0'/0'")), AddressTypeInference::Inferred(AddressType::P2shP2wpkh));
    assert_eq!(infer_address_type(&p("m/86'/0'/0'")), AddressTypeInference::Inferred(AddressType::P2tr));
    // multisig
    assert_eq!(infer_address_type(&p("m/48'/0'/0'/2'")), AddressTypeInference::Multisig);
    assert_eq!(infer_address_type(&p("m/87'/0'/0'")), AddressTypeInference::Multisig);
    // leaf / over-deep → NOT inferred (I2)
    assert_eq!(infer_address_type(&p("m/84'/0'/0'/0/5")), AddressTypeInference::Unknown);
    // empty / non-standard
    assert_eq!(infer_address_type(&p("m")), AddressTypeInference::Unknown);
    assert_eq!(infer_address_type(&p("m/0/0")), AddressTypeInference::Unknown);
}
```
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement:**
```rust
use bitcoin::bip32::{ChildNumber, DerivationPath};

#[derive(Debug, PartialEq, Eq)]
pub enum AddressTypeInference { Inferred(AddressType), Multisig, Unknown }

/// Map a card's origin_path to an address-type inference. Multisig (48'/87')
/// at any depth; single-sig purposes ONLY at canonical account depth (len==3).
pub fn infer_address_type(origin_path: &DerivationPath) -> AddressTypeInference {
    let comps: &[ChildNumber] = origin_path.as_ref();
    let purpose = match comps.first() {
        Some(ChildNumber::Hardened { index }) => *index,
        _ => return AddressTypeInference::Unknown, // empty or non-hardened purpose
    };
    if purpose == 48 || purpose == 87 { return AddressTypeInference::Multisig; }
    if comps.len() != 3 { return AddressTypeInference::Unknown; } // I2 account-depth gate
    match purpose {
        44 => AddressTypeInference::Inferred(AddressType::P2pkh),
        49 => AddressTypeInference::Inferred(AddressType::P2shP2wpkh),
        84 => AddressTypeInference::Inferred(AddressType::P2wpkh),
        86 => AddressTypeInference::Inferred(AddressType::P2tr),
        _ => AddressTypeInference::Unknown,
    }
}
```
- [ ] **Step 4 — run, expect PASS.** **Step 5 — commit.**

### Task 0.3: `render_address` + `secp_verify`

**Files:** Modify `crates/mk-cli/src/cmd/derive_support.rs`.

- [ ] **Step 1 — failing test** (render a known child against an independently-known address):
```rust
// Build a child xpub from a fixed test xpub, render all four types, assert prefixes.
#[test]
fn render_all_four_types_mainnet() {
    use bitcoin::bip32::Xpub; use std::str::FromStr;
    let secp = secp_verify();
    let xpub = Xpub::from_str(TEST_ACCT_XPUB).unwrap(); // a known mainnet account xpub const
    let child = xpub.derive_pub(&secp, &p("m/0/0")).unwrap();
    assert!(render_address(&secp,&child,AddressType::P2wpkh,CliNetwork::Mainnet).starts_with("bc1q"));
    assert!(render_address(&secp,&child,AddressType::P2tr,CliNetwork::Mainnet).starts_with("bc1p"));
    assert!(render_address(&secp,&child,AddressType::P2pkh,CliNetwork::Mainnet).starts_with('1'));
    assert!(render_address(&secp,&child,AddressType::P2shP2wpkh,CliNetwork::Mainnet).starts_with('3'));
    assert!(render_address(&secp,&child,AddressType::P2wpkh,CliNetwork::Regtest).starts_with("bcrt1q"));
}
```
- [ ] **Step 2 — run, expect FAIL.**
- [ ] **Step 3 — implement** (mirror toolkit `address_search.rs:35-51` EXACTLY for the four builders — note `p2pkh`/`p2shwpkh` take `to_pub()` by value, `p2wpkh` takes `&to_pub()`, `p2tr` takes `(secp, to_x_only_pub(), None, hrp)`):
```rust
use bitcoin::secp256k1::{Secp256k1, VerifyOnly};
use bitcoin::bip32::Xpub;
use bitcoin::Address;

pub fn secp_verify() -> Secp256k1<VerifyOnly> { Secp256k1::verification_only() }

pub fn render_address(secp: &Secp256k1<VerifyOnly>, child: &Xpub, ty: AddressType, net: CliNetwork) -> String {
    match ty {
        AddressType::P2pkh => Address::p2pkh(child.to_pub(), net.network_kind()).to_string(),
        AddressType::P2shP2wpkh => Address::p2shwpkh(&child.to_pub(), net.network_kind()).to_string(),
        AddressType::P2wpkh => Address::p2wpkh(&child.to_pub(), net.known_hrp()).to_string(),
        AddressType::P2tr => Address::p2tr(secp, child.to_x_only_pub(), None, net.known_hrp()).to_string(),
    }
}
```
- [ ] **Step 4 — run, expect PASS** (pin `TEST_ACCT_XPUB` to a real mainnet account xpub; the worker derives the expected prefixes empirically from the toolkit's own `convert --to address` to avoid hand-miscomputed vectors). **Step 5 — commit.**

### Task 0.4: Phase-0 review gate
- [ ] Dispatch opus reviewer on `derive_support.rs`; persist to `design/agent-reports/mk-derive-address-phase-0-review.md`; fold to 0C/0I before Phase 1.

---

## Phase 1 — `mk address`

> **Fixture construction (I1 — codec invariant).** `mk_codec::encode` rejects any card whose `xpub.depth != origin_path.len()` or whose `xpub.child_number != last(origin_path)` (`mk-codec/src/bytecode/encode.rs:41`, `XpubOriginPathMismatch`). So a test card's xpub MUST match its `origin_path`:
> - **Account fixtures** (44'/49'/84'/86' at depth 3, plus 48'/87' multisig, testnet `tpub`) are liftable directly from `crates/mk-codec/src/test_vectors/v0.1.json`.
> - **The leaf fixture** `m/84'/0'/0'/0/5` (§5.4b): forward-derive the xpub to depth 5 first — `let leaf = acct.derive_pub(&secp, &p("m/0/5")).unwrap();` (yields `depth==5`, `child_number==Normal{5}`) — then pair with `origin_path = m/84'/0'/0'/0/5` and `mk_codec::encode`. Same care for any depth-0 (master, empty path) fixture.
> - Build cards in a shared `tests/common/mod.rs` helper `make_card(xpub, origin_path) -> mk1_strings` wrapping `mk_codec::encode`.

### Task 1.1: `AddressArgs` clap struct + skeleton run (returns UsageError stubs)

**Files:** Create `crates/mk-cli/src/cmd/address.rs`; Modify `cmd/mod.rs`, `main.rs`.

- [ ] **Step 1 — failing integration test** `crates/mk-cli/tests/cli_address.rs` — start with the happy path (84' card → p2wpkh, count default 10, receive chain):
```rust
use assert_cmd::Command;
// MK1_84_ACCT: an mk1 card encoding a known m/84'/0'/0' account xpub (built once via `mk encode`).
fn mk(args: &[&str]) -> std::process::Output {
    Command::cargo_bin("mk").unwrap().arg("address").args(args).output().unwrap()
}
#[test]
fn account_84_default_p2wpkh_count10() {
    let out = mk(&[MK1_84_ACCT]);
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    assert_eq!(s.lines().filter(|l| l.contains("bc1q")).count(), 10);
}
```
- [ ] **Step 2 — run, expect FAIL** (no `address` subcommand). **Step 3 — implement** the `AddressArgs` struct + `Command::Address` arm + dispatch + `is_json_mode` (mirror decode's wiring), and a `run` that: reads `read_mk1_strings`, `mk_codec::decode`, resolves address-type (§3.1 order: multisig-refuse → explicit → infer → require), resolves network, derives over chain×range, prints. Build `MK1_84_ACCT` by `mk encode`-ing a known xpub in a test helper or a committed fixture.
- [ ] **Step 4 — run, expect PASS. Step 5 — commit.**

### Task 1.2: address-type resolution (multisig refuse, explicit, infer, require) + depth advisory
- [ ] Tests (cli_address.rs): multisig 48'/87' → exit 64 even with `--address-type` (SPEC §5.3); ambiguous/depth-0 → exit 64 (§5.4); leaf m/84'/.../0/5 → exit 64 w/o flag, success+stderr-advisory with `--address-type p2wpkh` (§5.4b); `--address-type` override on an 84' card → p2pkh addresses (§5.2). Implement the resolution fn returning `Result<AddressType, CliError>` + the depth advisory on stderr. Run → PASS → commit.

### Task 1.3: `--count`/`--range`/`--chain`/`--network`
- [ ] Tests: count default 10 + explicit; `--range A,B`; `A>B` → exit 64 (`UsageError`); count/range conflict → **exit 64** (C1: mk-cli's `main.rs:62-67` routes ALL clap parse errors through a catch-all `ExitCode::from(64)`; clap's native exit 2 never reaches the shell — assert 64 + clap usage message on stderr); `--chain receive|change|both` chain indices + both-ordering (receive then change); network inference (mainnet/testnet from xpub version); `--network regtest` → `bcrt1…`; `--network mainnet` on a test xpub → exit 64 (§5.7). Implement. Run → PASS → commit.

### Task 1.4: `--json` envelope
- [ ] Test: `--json` emits `{schema_version:1, xpub, origin_path, address_type, network, addresses:[{chain,index,address}]}`, valid JSON, integer schema_version; error-in-json → stdout error envelope (mirror main.rs `emit_error`). Implement via `serde_json::json!`. Run → PASS → commit.

### Task 1.5: address-correctness vectors
- [ ] Test (§5.8): derived addresses for `MK1_84_ACCT` (and an 86'/49'/44' fixture) match independently-computed values — generate the expected values once via the toolkit `mnemonic convert --from xpub=<acct> --to address --path m/0/i --script-type ...` and paste as constants (cite provenance in a comment). Run → PASS → commit.

### Task 1.6: Phase-1 review gate
- [ ] Opus reviewer on `address.rs` + tests; persist `…phase-1-review.md`; fold to 0C/0I.

---

## Phase 2 — `mk derive`

### Task 2.1: `DeriveArgs` + run (relative unhardened derivation)
- [ ] **Failing test** `crates/mk-cli/tests/cli_derive.rs`: `mk derive MK1_84_ACCT --path m/0/5` → stdout child_xpub + child_fingerprint + depth (§5.10). **Implement** `DeriveArgs { mk1: Vec<String>, path: Option<String>, index: Option<u32>, json: bool }` with a required mutually-exclusive `ArgGroup` over path/index; `Command::Derive` arm + dispatch; `run`: decode → resolve rel path (`--index N` ⇒ `m/0/N`) → reject hardened (iterate `ChildNumber::is_hardened()`) → `xpub.derive_pub` → emit. Run → PASS → commit.

### Task 2.2: unhardened-only + index sugar + group errors + multisig-allowed
- [ ] Tests: `--path m/0'/0` → exit 64 ("cannot derive hardened…") (§5.11); `--index 5` == `--path m/0/5` (§5.12); both/neither path+index → clap group error → **exit 64** (C1, not 2); `mk derive` on a 48' multisig card SUCCEEDS (§5.13). Implement. Run → PASS → commit.

### Task 2.3: `--json` + composability
- [ ] Test: `--json` shape `{schema_version:1, parent_xpub, parent_origin_path, relative_path, child_xpub, child_fingerprint, depth, network}`; `child_xpub` round-trips through `mk encode` (§5.14, smoke). Implement (`child_fingerprint = fmt_fingerprint(&child.fingerprint())`). Run → PASS → commit.

### Task 2.4: Phase-2 review gate
- [ ] Opus reviewer on `derive.rs` + tests; persist `…phase-2-review.md`; fold to 0C/0I.

---

## Phase 3 — gui-schema reflection + workspace wiring

### Task 3.1: `gui_schema.rs` build_schema includes address + derive
- [ ] **Test** (§5.16, `crates/mk-cli/tests/gui_schema.rs` style): `mk gui-schema` JSON contains subcommands `address` + `derive` (assert via `contains`, matching the reflective `:113-133` style). **Implement:** add `SubcommandSchema` entries for both to `cmd/gui_schema.rs::build_schema` (flags + positional + value enums for `--address-type`/`--network`/`--chain`). Run → PASS → commit.

### Task 3.2: clippy + fmt + full mk-cli suite
- [ ] `cargo clippy -p mk-cli --all-targets -- -D warnings` (codecs gate fmt+clippy on stable — `cargo +stable fmt --check --all` is authoritative here, unlike the toolkit); `cargo +stable fmt --all`; `cargo test -p mk-cli --no-fail-fast` 0 failures. Commit any fmt/lint fixes.

---

## Phase 4 — version bump + CHANGELOG + end-of-cycle review + ship

### Task 4.1: bump + CHANGELOG
- [ ] `crates/mk-cli/Cargo.toml` version 0.5.0 → 0.6.0; `Cargo.lock` re-resolve (`cargo build -p mk-cli`); add the release entry to **`crates/mk-cli/CHANGELOG.md`** (the mk-cli changelog; heading `## [0.6.0] — 2026-05-NN`, mirroring the existing `## [0.4.2]` style). I2: do NOT touch the root `/CHANGELOG.md` — that is the mk-codec changelog and mk-codec is unchanged this cycle. Commit.

### Task 4.2: end-of-cycle opus R0
- [ ] Dispatch opus reviewer over the full cycle diff `main..HEAD`; persist `…end-of-cycle-R0-review.md`; fold to 0C/0I (re-dispatch after every fold).

### Task 4.3: ship
- [ ] Clean tree (`git status --porcelain` empty of tracked); `git fetch`; confirm `main == origin/main`; ff-merge `main` ← `mk-derive-address-readonly`; push `main`; tag `mk-cli-v0.6.0`; push tag. (mk-codec unchanged → no mk-codec tag.)

---

## Phase 5 — lockstep (paired changes; same logical cycle)

### Task 5.1: manual mirror (toolkit repo)
- [ ] In `mnemonic-toolkit`, branch; update `docs/manual/src/40-cli-reference/44-mk-cli.md`: add `mk address` + `mk derive` sections (every flag), fix the subcommand count (`:4` "Six" → eight), bump install tag (`:12`) `mk-cli-v0.4.0 → mk-cli-v0.6.0`. Run `make -C docs/manual lint MK_BIN=<v0.6.0 mk> …` flag-coverage. Commit.

### Task 5.2: toolkit sibling-pin
- [ ] In `mnemonic-toolkit`: bump mk pin → `mk-cli-v0.6.0` in `install.sh`, `.github/workflows/manual.yml`, `quickstart.yml`. The `sibling-pin-check.yml` gate enforces parity. Commit (with 5.1, or sequence per the toolkit's no-bump docs-only convention).

### Task 5.3: GUI schema-mirror (mnemonic-gui repo)
- [ ] In `mnemonic-gui`, branch; in `src/schema/mk.rs`: add `address` + `derive` `SubcommandSchema`s, **backfill the missing `repair` `SubcommandSchema`**, bump header `:1` + `pinned_version` `:312` "mk 0.3.1" → "mk 0.6.0" (M4: note these two markers are ALREADY skewed — header/pinned_version say v0.3.1 while `pinned-upstream.toml:52` says v0.4.2 — bump BOTH to v0.6.0); `pinned-upstream.toml:52` `mk-cli-v0.4.2 → mk-cli-v0.6.0`; use `FlagKind::Dropdown` for `--address-type`/`--network`/`--chain` (value sets EXACTLY matching `mk gui-schema`), `FlagKind::Number` for `--count`/`--index`, `FlagKind::Range` for `--range`. Run the `schema_mirror` test against the v0.6.0 binary → 0 drift. Commit.

### Task 5.4: FOLLOWUPS companions
- [ ] mnemonic-key `design/FOLLOWUPS.md` + toolkit companion entry recording the Theme-B mk slice shipped + the GUI repair-backfill done.

---

## Self-review checklist (done at write time)
- Spec coverage: every §5 test (1–16, incl. 4b leaf) maps to a Phase-1/2/3 task ✓.
- No placeholders: load-bearing code (enums, infer, render, json shapes) shown; mechanical test bodies specified with the exact assertion + fixture-provenance instruction ✓.
- Type consistency: `AddressTypeInference`/`AddressType`/`CliNetwork`/`render_address`/`secp_verify` names identical across Phase 0 and consumers ✓.
- Exit codes: all four conditions → `UsageError`/64 AND clap parse errors (conflict/group) ALSO → 64 via `main.rs:62-67`'s catch-all (C1 — no 2-vs-64 split; everything user-error is 64) ✓.
- Lockstep: manual + GUI(+repair backfill) + sibling-pin + FOLLOWUPS all have tasks ✓.
