# SPEC — `mk address` + `mk derive` (read-only public derivation surface)

**Repo:** mnemonic-key (mk-cli). **Branch:** `mk-derive-address-readonly` off `main`.
**SemVer:** mk-cli **0.5.0 → 0.6.0** (MINOR — two new subcommands). mk-codec unchanged (**0.4.0** — no codec change).
**Source ground truth verified @ `ed2596d`** (origin/main at spec-write time).

---

## §1. Context & motivation

Theme B of the constellation feature survey ("see it / use it after you recover it"): after recovering an mk1 card the user must reach for an external tool to view the addresses it controls or to derive a child xpub. `mk decode`/`inspect` recover the xpub but the CLI cannot DERIVE from it. This SPEC adds two read-only subcommands:

- **`mk address`** — render N receive/change addresses the card's xpub controls.
- **`mk derive`** — derive a child xpub at a relative (unhardened) path; composable (pipe to `mk encode`).

**Firm product boundary (user-set, 2026-05-30):** read-only **PUBLIC** derivation only — addresses, child xpubs, fingerprints. **No signing**, no private keys, no nonces. An xpub has no private key, so this is structurally guaranteed here. (Companion: the toolkit's `sign-message` / tx-signing surfaces stay deferred.)

---

## §2. Source ground truth (verified @ `ed2596d`)

- **`crates/mk-codec/src/key_card.rs:24`** — `pub struct KeyCard { policy_id_stubs: Vec<[u8;4]>, origin_fingerprint: Option<Fingerprint> (master fp, may be None), origin_path: DerivationPath, xpub: Xpub }`. `xpub` is `bitcoin::bip32::Xpub`. Decode entry `mk_codec::decode(&[&str]) -> Result<KeyCard>`.
- **`crates/mk-cli/src/main.rs:33`** — `enum Command { Encode, Decode, Inspect, Verify, Vectors, GuiSchema, Repair }`. Two new arms `Address(cmd::address::AddressArgs)` + `Derive(cmd::derive::DeriveArgs)` to add (alphabetical placement N/A — mk-cli `Command` is not alphabetized; append in the focused-subcommand list).
- **`crates/mk-cli/src/cmd/mod.rs`** — shared helpers already present: `read_mk1_strings(&[String]) -> Result<Vec<String>>` (`:80`), `parse_derivation_path(&str) -> Result<DerivationPath>` (`:47`), `fmt_fingerprint(&Fingerprint) -> String` (`:73`); `use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub}` (`:14`), `use bitcoin::hashes::{Hash, sha256}` (`:15`).
- **`crates/mk-cli/src/cmd/inspect.rs:44`** — `card.xpub.fingerprint()` computes the xpub's own fingerprint (HASH160 of its pubkey). This is the `mk derive` child-fingerprint primitive.
- **`crates/mk-cli/src/cmd/decode.rs:55`** — `--json` house style is the `serde_json::json!({ "schema_version": 1, "xpub": ..., ... })` macro (NOT serde-derive); `schema_version` is **integer `1`**. Text output uses `println!("origin_path:         {}", ...)` aligned-label style.
- **`crates/mk-cli/Cargo.toml:29`** — `bitcoin = "0.32"` already a dep (no new dep needed). `bitcoin::bip32::Xpub::derive_pub`, `bitcoin::Address::{p2pkh,p2wpkh,p2shwpkh,p2tr}`, `bitcoin::secp256k1::Secp256k1`, `NetworkKind`, `KnownHrp` all available in 0.32 (reference use: toolkit `xpub_search/address_search.rs`). mk-cli does NOT yet import `Address`/`Secp256k1` — these are new imports, not new deps.
- **mk-cli has NO `Network`/`ScriptType` type today** — both are net-new local types (`§3.3`). The toolkit's `CliNetwork`/`ScriptType` MUST NOT be imported (codec is upstream of the toolkit — sibling rule).

---

## §3. Design

### 3.1 `mk address` — N addresses from a card

```
mk address <MK1>... [--address-type <T>] [--count <N> | --range <A,B>]
                    [--chain <receive|change|both>] [--network <NET>] [--json]
```

- **Input:** one or more mk1 chunk strings (positional, via `read_mk1_strings`; stdin `-` per existing convention) → `KeyCard` → `xpub` + `origin_path`. mk1 cards ONLY — a bare xpub is the toolkit's job (out of scope, §6).
- **Address-type resolution** (`AddressType ∈ {p2pkh, p2sh-p2wpkh, p2wpkh, p2tr}`):
  1. **Multisig refuse FIRST** — if `origin_path`'s first component is `48'` or `87'`, REFUSE with advisory (the card is a multisig cosigner xpub; single-key addresses would not match the wallet). Refusal holds even if `--address-type` is given (the card's multisig nature, not a missing flag, is the problem) — the override is intentionally NOT honored here; the advisory points to descriptor tooling AND the `mnemonic convert --to address` escape hatch (M6). `CliError::UsageError(String)` → **exit 64** (C1: usage error, NOT codec-rejection exit 2).
  2. If `--address-type` given → use it (subject to the depth advisory, step 5).
  3. Else **heuristic** — applies ONLY at the canonical single-sig **account depth**: `origin_path.len() == 3` AND the first component is a recognized purpose → `44'→p2pkh`, `49'→p2sh-p2wpkh`, `84'→p2wpkh`, `86'→p2tr`. (I2: keying off purpose ALONE is unsafe — a leaf card `m/84'/0'/0'/0/5` reads `84'` but deriving `m/c/i` relative to a leaf yields addresses matching no wallet; `origin_path.len()` / `xpub.depth` is the guard.)
  4. Else (empty path / non-standard purpose / non-account depth) → ERROR "`--address-type` required (cannot infer)"; `CliError::UsageError` → **exit 64**. Never silently guess on an ambiguous card.
  5. **Depth advisory (I2):** whenever the resolved address-type is used on a card whose `xpub.depth` ≠ the canonical single-sig account depth (3) — i.e. an explicit `--address-type` was supplied on a leaf / over-deep / depth-0 card — emit a stderr advisory that addresses are derived **relative to the card's xpub** and may not match a standard wallet, then proceed (permissive-with-warning; matches the constellation's silent-default-with-stderr-notice convention).
- **Network resolution** (`CliNetwork ∈ {mainnet, testnet, signet, regtest}`):
  - Default from `xpub.network` (`NetworkKind::Main → mainnet`, `NetworkKind::Test → testnet`).
  - `--network` overrides, but MUST agree with the xpub's network-KIND (main vs test): `--network mainnet` on a test-version xpub (or vice-versa) → ERROR `CliError::UsageError` → **exit 64**. Within the test kind, `testnet`/`signet`/`regtest` are all selectable (they share the test version bytes but differ in address HRP: `tb1…` for testnet/signet, `bcrt1…` for regtest). This guards the "wrong-network address" footgun.
- **Range:** `--count N` (default **10**) enumerates indices `0..N`; OR `--range A,B` enumerates `A..=B` (net-new local parse — no `parse_range` exists in mk-cli (M2); `A ≤ B` else `CliError::UsageError` → exit 64). Mutually exclusive (`conflicts_with`).
- **Chain:** `--chain receive|change|both` (default **receive** = chain 0; `change` = chain 1; `both` = chain 0 then chain 1).
- **Derivation:** for each selected chain `c`, each index `i` in range: `dp = m/c/i` (relative to card xpub), `child = xpub.derive_pub(&secp, &dp)`, render `address` per `AddressType` + `CliNetwork`. Hardened never occurs here (chain/index are unhardened by construction).
- **Output (text):** when `both`, group by chain with a header line (`receive (m/0/i):` / `change (m/1/i):`); each row `  <index>  <address>`. Single-chain: rows only.
- **Output (`--json`):**
  ```json
  {"schema_version":1,"xpub":"xpub6...","origin_path":"m/84'/0'/0'",
   "address_type":"p2wpkh","network":"mainnet",
   "addresses":[{"chain":0,"index":0,"address":"bc1q..."},
                {"chain":0,"index":1,"address":"bc1q..."}]}
  ```
  `chain` is the integer BIP chain index (0/1). On error in `--json` mode, the existing `emit_error` stdout-envelope is used (`main.rs` house behavior).

### 3.2 `mk derive` — a child xpub at a relative path

```
mk derive <MK1>... (--path <REL> | --index <N>) [--json]
```

- **Input:** mk1 → `KeyCard` → `xpub`.
- **Path:** `--path` is **relative to the card's xpub** (e.g. `m/0/5` = external/index-5 child), **unhardened only** — detect a hardened component by iterating the parsed `DerivationPath` for any `ChildNumber::is_hardened()`; if found → ERROR `CliError::UsageError("cannot derive hardened children from an xpub")` → **exit 64** (C1). `--index N` is sugar for `--path m/0/N` (external chain). Exactly one of `--path`/`--index` required (clap `ArgGroup` required=true; mutually exclusive).
- **Multisig:** NOT refused — deriving a child xpub from a cosigner xpub is legitimate per-cosigner use.
- **Derivation:** `child = xpub.derive_pub(&secp, &rel_path)`; `child_fingerprint = fmt_fingerprint(&child.fingerprint())` (I3 — `.fingerprint()` returns `Fingerprint`; format via the `cmd/mod.rs:73` house helper, matching `inspect.rs:44`, note the `&`); `depth = child.depth`.
- **Output (text):** aligned-label block (`parent_xpub`, `parent_origin_path`, `relative_path`, `child_xpub`, `child_fingerprint`, `depth`, `network`).
- **Output (`--json`):**
  ```json
  {"schema_version":1,"parent_xpub":"xpub6...","parent_origin_path":"m/84'/0'/0'",
   "relative_path":"m/0/5","child_xpub":"xpub6...","child_fingerprint":"aabbccdd",
   "depth":5,"network":"mainnet"}
  ```
  `child_xpub` is composable: pipe back into `mk encode`.

### 3.3 Shared module `crates/mk-cli/src/cmd/derive_support.rs` (new)

Single source of truth for both subcommands (DRY):
- `enum AddressType { P2pkh, P2shP2wpkh, P2wpkh, P2tr }` — clap `ValueEnum` with `#[clap(rename_all = "kebab-case")]` (M1 — renders `P2shP2wpkh` → `p2sh-p2wpkh`; the default-lower rule would give `p2shp2wpkh`), values `p2pkh|p2sh-p2wpkh|p2wpkh|p2tr`.
- `enum CliNetwork { Mainnet, Testnet, Signet, Regtest }` — clap `ValueEnum` with `#[clap(rename_all = "lower")]` (M1 — matches toolkit `network.rs:11`); `.network_kind()` → `NetworkKind`, `.known_hrp()` → `KnownHrp` accessors mirroring the toolkit's minimal subset (local copy — sibling rule forbids importing the toolkit's).
- `fn infer_address_type(origin_path: &DerivationPath) -> AddressTypeInference` → `Inferred(AddressType)` | `Multisig` (48'/87') | `Unknown` (drives the §3.1 resolution order; `Inferred` ONLY when `origin_path.len() == 3` per I2).
- `fn render_address(secp: &Secp256k1<VerifyOnly>, child: &Xpub, ty: AddressType, net: CliNetwork) -> String` — the four `Address::p2*` builders (`p2pkh`/`p2shwpkh` via `child.to_pub()` + `net.network_kind()`; `p2wpkh` via `&child.to_pub()` + `net.known_hrp()`; `p2tr` via `secp` + `child.to_x_only_pub()` + `None` + `net.known_hrp()`). Mirrors toolkit `address_search.rs::render_address` / `convert.rs::build_address_from_xpub`, re-implemented locally.
- **Secp context (M5):** built once via `Secp256k1::verification_only()` (no signing capability — structurally reinforces the no-private-key boundary; precedent `verify_message.rs:55`).
- Unit-tested in isolation (purpose→type table incl. the `len()==3` gate; render vs known child; network HRP mapping).

---

## §4. SemVer + lockstep

- **mk-cli 0.5.0 → 0.6.0** (MINOR; additive subcommands). mk-codec stays 0.4.0. `Cargo.lock` re-resolved.
- **Manual mirror** — `mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md` MUST gain both subcommands + every flag in lockstep (toolkit-repo paired change; `docs/manual/tests/lint.sh` flag-coverage gates per (binary, subcommand)). Also fix the documented subcommand count (`:4`, "Six" → **eight**: the manual deliberately documents the 6 user-facing subcommands and excludes the GUI-internal `gui-schema`; 6 + `address` + `derive` = 8) and bump the install tag (`:12`) `mk-cli-v0.4.0 → mk-cli-v0.6.0` (M4).
- **GUI schema-mirror** (paired PR on mnemonic-gui) — ⚠️ larger than a pin bump (I1): `mnemonic-gui/src/schema/mk.rs` is at **"mk-cli-v0.3.1"** (header `:1` + `pinned_version` `:312`) and `SUBCOMMANDS` (`:267-308`) lists only inspect/encode/decode/verify/vectors — **`repair` was never mirrored** (accumulated drift; CLAUDE.md `gui-schema-mirror-lockstep-discipline`). The paired PR MUST: (a) add `address` + `derive` `SubcommandSchema`s; (b) **backfill the missing `repair` `SubcommandSchema`** (`--json` + `mk1-strings` positional); (c) bump header `:1` + `pinned_version` `:312` "mk 0.3.1" → "mk 0.6.0"; (d) bump `mnemonic-gui/pinned-upstream.toml:52` `mk-cli-v0.4.2 → mk-cli-v0.6.0`; (e) `AddressType` / `CliNetwork` / `--chain` value enums use `FlagKind::Dropdown` with value sets matching `mk gui-schema` EXACTLY (the gate enforces flag-NAME + dropdown-value parity); (f) `--count`/`--index` → `FlagKind::Number`, `--range` → `FlagKind::Range`. The `schema_mirror` test fires on the pin bump against ALL THREE accumulated subcommands (repair + address + derive). Note (M3): `mk gui-schema` itself emits `kind:"text"` for the numeric flags (no numeric mapping in `gui_schema.rs::classify`); the gate only checks flag-NAME + dropdown-value parity, so this is tolerated.
- **toolkit sibling-pin** — `install.sh` + `.github/workflows/manual.yml` + `quickstart.yml` mk pin → `mk-cli-v0.6.0`; the toolkit `sibling-pin-check.yml` gate enforces this on the next toolkit push.
- **mk gui-schema test** — `crates/mk-cli/tests/gui_schema.rs` auto-reflects the new surface; the new subcommands must appear in `cmd/gui_schema.rs::build_schema`.
- **FOLLOWUPS companions** — file an entry in mnemonic-key `design/FOLLOWUPS.md` + a toolkit companion (the manual mirror is toolkit-side).

---

## §5. Test plan (per-phase TDD)

**`mk address`:**
1. Heuristic resolution — a card at `m/44'/0'/0'` → p2pkh; `49'`→p2sh-p2wpkh; `84'`→p2wpkh; `86'`→p2tr (build card via `mk encode` of a known xpub, assert default address-type).
2. `--address-type` override — `84'` card + `--address-type p2pkh` → legacy addresses.
3. Multisig refuse — `m/48'/0'/0'/2'` and `m/87'/...` cards → **exit 64** (`UsageError`) advisory, even with `--address-type` given.
4. Ambiguous → required — depth-0 (no-path) card and a non-standard-purpose card with no `--address-type` → **exit 64** (`UsageError`, "address-type required").
4b. **Leaf / over-deep card (I2)** — a card at `m/84'/0'/0'/0/5` (depth 5): with no `--address-type` → exit 64 (heuristic gated on `len()==3`); with explicit `--address-type p2wpkh` → succeeds BUT emits the stderr depth advisory.
5. `--count` default 10 + explicit; `--range A,B`; `--range` with `A>B` → exit 64 (`UsageError`); `--count`/`--range` conflict → clap error → **exit 64** (mk-cli `main.rs:62-67` routes all clap parse errors through a 64 catch-all).
6. `--chain receive|change|both` — correct chain indices; `both` ordering (receive then change).
7. Network — inferred mainnet/testnet from xpub version; `--network regtest` → `bcrt1…`; `--network mainnet` on a test xpub → **exit 64** (`UsageError`, network mismatch).
8. Address correctness — derived addresses match independently-computed values for a known xpub (BIP-32 vector or a fixed test xpub) across all four script types.
9. `--json` shape — `addresses[]` of `{chain,index,address}`, integer `schema_version:1`, valid JSON; error-in-json → stdout error envelope.

**`mk derive`:**
10. Relative derivation — `m/0/5` from a known card → expected child xpub + `child_fingerprint` + `depth`.
11. Unhardened-only — `--path m/0'/0` (hardened) → **exit 64** (`UsageError`, "cannot derive hardened children from an xpub").
12. `--index N` sugar == `--path m/0/N`; `--path`/`--index` both / neither → clap group error → **exit 64** (same 64 catch-all).
13. Multisig allowed — `m/48'/...` card + `mk derive` succeeds (NOT refused).
14. `--json` shape — `child_xpub` round-trips through `mk encode` (composability smoke test).

**Shared / lockstep:**
15. `derive_support` unit tests — purpose→type table incl. the `len()==3` gate, `render_address` vs known child, network HRP mapping.
16. `mk gui-schema` output INCLUDES `address` + `derive` (assert via `contains`, matching the reflective style of `mk-cli/tests/gui_schema.rs:113-133`). NOTE (M3): the authoritative flag-NAME/dropdown-value parity gate is the **GUI repo's** `schema_mirror` test (not mk-cli's), exercised on the pin bump — the mk-cli test only confirms the surface is emitted.

---

## §6. Non-goals / boundaries

- **No signing** — firm boundary; no private keys touch stdout (xpub-only by construction).
- **No multisig address derivation** — `mk address` refuses multisig-path cards (descriptor tooling's job).
- **No bare-xpub input** — mk1 cards only; bare xpub → toolkit `convert`/`xpub-search`.
- **No hardened derivation in `mk derive`** — xpub limitation (clear error).
- **Heuristic address-type only for standard single-sig purposes** (44/49/84/86); everything else requires explicit `--address-type`.
- **No `--gap-limit`** — these are deterministic enumeration, not address-use scanning.

---

## §7. Open questions for R0
1. `--chain both` JSON ordering (receive-all-then-change-all vs interleaved) — SPEC picks receive-then-change; confirm.
2. Network-mismatch as a hard error vs a warning — SPEC picks hard error (footgun guard); confirm consistent with mk-cli's other guards.
3. `mk derive` default when neither `--path`/`--index` (SPEC: required group, no default) vs echoing the card xpub — SPEC picks required.
4. Should `--count` default be 10 or 20? SPEC picks 10.
