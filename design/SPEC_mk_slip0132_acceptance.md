# SPEC — mk SLIP-0132 (ypub/zpub) acceptance (Theme A2)

**Repo:** `mnemonic-key` (mk-cli; mk-codec UNTOUCHED). **Branch:** `mk-slip0132-acceptance`. **Source SHA:** `main` `fc2341b` (re-grep citations at impl time — CLAUDE.md).
**FOLLOWUP:** `mk-slip0132-prefix-acceptance` (NET-NEW — file at brainstorm; resolve at ship).
**Status:** **R0 GREEN (0C/0I)** — opus architect review (3 Minors, folded below; §11). Review persisted at `design/agent-reports/mk-slip0132-spec-R0-review.md`. Cleared to plan-doc (which gets its own R0 gate).

**Precedent (R0 find):** the toolkit ALREADY ships a CI-tested 10-entry SLIP-0132 table at `mnemonic-toolkit/crates/mnemonic-toolkit/src/slip0132.rs` (`:54-100,141-154`). mk-cli is upstream of the toolkit (cannot dep it), so `slip132.rs` **duplicates** that proven primitive — model the table + swap logic on it and verify byte-parity (same upstream-duplication pattern as mk-cli's `output_advisory.rs`).

## 1. Goal
`mk-cli` should ingest the SLIP-0132 extended-key prefixes that Sparrow/Coldcard/Electrum routinely export (`ypub`/`zpub`/`Ypub`/`Zpub` + testnet `upub`/`vpub`/`Upub`/`Vpub`), instead of refusing them. Today `mk encode`/`verify --xpub <zpub>` fails at two walls: `parse_xpub`→`Xpub::from_str` (`crates/mk-cli/src/cmd/mod.rs:57-58`) rejects non-canonical versions, and (even if parsed) `version_to_network` (`crates/mk-codec/src/bytecode/xpub_compact.rs:63-67`) only knows `xpub`(`0x0488B21E`)/`tpub`(`0x043587CF`). This is the biggest mk interop friction.

**Decision (from brainstorm):** accept + **normalize to canonical `xpub`/`tpub`** (no wire change — only the 4 version bytes differ; depth/child/fingerprint/chaincode/pubkey are byte-identical), emit a **stderr note** naming the original prefix, and **refuse (exit 64) on a prefix↔origin-path mismatch with an actionable remediation message**. `KeyCard` stores the canonical `xpub`; mk-codec is untouched (stays canonical-only). Normalization is a CLI input-convenience.

## 2. SLIP-0132 version table (the fixed, well-defined set)
| Prefix | Version bytes | Network | Script type / BIP | Implied origin-path predicate |
|---|---|---|---|---|
| `xpub` | `0x0488B21E` | Main | canonical (generic) | **none** (no check) |
| `tpub` | `0x043587CF` | Test | canonical (generic) | **none** (no check) |
| `ypub` | `0x049D7CB2` | Main | P2SH-P2WPKH / BIP-49 | purpose `49'` |
| `zpub` | `0x04B24746` | Main | P2WPKH / BIP-84 | purpose `84'` |
| `Ypub` | `0x0295B43F` | Main | P2WSH-P2SH multisig / BIP-48 | purpose `48'` ∧ script-type index `1'` |
| `Zpub` | `0x02AA7ED3` | Main | P2WSH multisig / BIP-48 | purpose `48'` ∧ script-type index `2'` |
| `upub` | `0x044A5262` | Test | P2SH-P2WPKH / BIP-49 | purpose `49'` |
| `vpub` | `0x045F1CF6` | Test | P2WPKH / BIP-84 | purpose `84'` |
| `Upub` | `0x024289EF` | Test | P2WSH-P2SH multisig / BIP-48 | purpose `48'` ∧ script-type index `1'` |
| `Vpub` | `0x02575483` | Test | P2WSH multisig / BIP-48 | purpose `48'` ∧ script-type index `2'` |

The mainnet non-canonical variants normalize to `xpub` (`0x0488B21E`); testnet variants normalize to `tpub` (`0x043587CF`). (Implementer: verify each version-byte constant against a SLIP-0132 reference before pinning.) The "script-type index" is the **4th** path component of a BIP-48 multisig path `m/48'/coin'/account'/script_type'`. **Network is taken from the prefix** (authoritative for `xpub` vs `tpub`); we do NOT cross-check the path's `coin_type` (looser convention; out of scope).

## 3. Normalization
`detect_and_normalize(s: &str) -> Result<(Xpub, Option<Slip132Variant>)>`:
1. base58check-decode `s`; read the 4-byte version.
2. If it's a known **non-canonical** SLIP-0132 version → swap to the canonical `xpub`/`tpub` bytes for that network, re-encode base58check, `Xpub::from_str` the result, return `(xpub, Some(variant))`.
3. If it's canonical `xpub`/`tpub` → `Xpub::from_str(s)`, return `(xpub, None)`.
4. Else (unrecognized version / not an xpub) → fall through to `Xpub::from_str(s)` so the existing error path/message is preserved (truly-unknown versions still error as today).

`Slip132Variant` carries: the display label (e.g. `"zpub (BIP-84 P2WPKH)"`), the network, and the implied-path predicate (§2).

## 4. stderr note (on every normalization; exit 0)
When a non-canonical variant is detected (and the mismatch check, if applicable, passes), emit ONE stderr line, e.g.:
`note: --xpub was a SLIP-0132 zpub (BIP-84 P2WPKH); normalized to canonical xpub — the engraved card's script type derives from the origin path`
Per-variant label from `Slip132Variant`. Canonical `xpub`/`tpub` → no note (today's behavior). (Style: `eprintln!`-family, matching mk-cli's existing stderr usage.)

## 5. Mismatch refusal (exit 64) — ACTIONABLE
When a non-canonical variant is detected **and an origin path is available** (see §6 for when), evaluate the implied-path predicate (§2) against the path. On failure → **`CliError::UsageError` (exit 64)** BEFORE building the `KeyCard`, with a message that names BOTH sides + the fix:
- single-sig e.g. `error: SLIP-0132/origin-path mismatch — --xpub is a zpub (BIP-84 P2WPKH, expects --origin-path purpose 84', e.g. m/84'/0'/0'), but --origin-path is m/49'/0'/0' (purpose 49' = BIP-49, which is the 'ypub' script type). To engrave a backup, reconcile them: use the zpub at an 84' path, or supply the ypub for this 49' path.`
- multisig e.g. `error: SLIP-0132/origin-path mismatch — --xpub is a Zpub (BIP-48 P2WSH multisig, expects m/48'/<coin>'/<account>'/2'), but --origin-path is m/48'/0'/0'/1' (script-type index 1' = the 'Ypub' P2WSH-P2SH type). Reconcile: use a Ypub for the 1' path, or m/48'/.../2' for the Zpub.`
- predicate-unsatisfiable-because-path-too-short/empty e.g. `error: --xpub is a zpub (BIP-84 P2WPKH) and requires an --origin-path with purpose 84' (e.g. m/84'/0'/0'); got <path-or-"none">. Supply a matching origin path to engrave a backup.`

Canonical `xpub`/`tpub` → no predicate, no refusal (a plain `xpub` at any path is accepted as today — `xpub` makes no script-type claim).

## 6. Where the check runs (encode vs verify)
`parse_xpub` (`cmd/mod.rs`) is shared by `encode` (`cmd/encode.rs:85`) and `verify` (`cmd/verify.rs:53`). Extend it (or add a sibling) to take the origin path + a stderr sink: `parse_xpub_normalized(s, origin_path: Option<&DerivationPath>, stderr) -> Result<Xpub>`.
- **`encode`**: always supplies its `--origin-path` (the card's path; empty if none). A non-canonical variant ⇒ run the predicate (refuse on mismatch). The downstream mk-codec depth/child guard still applies after normalization (independent check).
- **`verify`**: `--origin-path` is OPTIONAL (content matcher). If supplied ⇒ run the predicate (refuse on mismatch). If NOT supplied ⇒ normalize + emit the note + **skip** the path predicate (key-material verification still works; there is no path to validate against). This keeps a bare `verify --xpub <zpub> <mk1>` (key-material check) working.

## 7. Error handling / exit codes
- Mismatch / unsatisfiable predicate → `UsageError` → **exit 64** (per the mk exit-code contract: UsageError=64, established in the mk derive/address cycle).
- Unrecognized extended-key version (not canonical, not SLIP-0132) → unchanged: `Xpub::from_str` error → `UsageError` "invalid xpub …".
- No new `mk-codec::Error` variants (mk-codec untouched).

## 8. Scope boundaries / lockstep
- **mk-codec: UNCHANGED** (no bump/publish/re-pin). Normalization lives entirely in mk-cli.
- **mk-cli: MINOR `0.6.1 → 0.7.0`** (new accepted input class; purely additive — every SLIP-0132 prefix was wholly refused before, so no previously-accepted input changes behavior). crates.io publish + git tag `mk-cli-v0.7.0`.
- **No new clap flag** (`--xpub` already exists; we widen its accepted VALUES) ⇒ **no GUI `schema_mirror` change** (it gates flag-NAME parity) and **no manual flag-coverage change**.
- **toolkit lockstep:** re-pin the mk-cli tag `mk-cli-v0.6.1 → mk-cli-v0.7.0` at the sibling-pin sites (`scripts/install.sh`, `.github/workflows/manual.yml`, `.github/workflows/quickstart.yml` — the `sibling-pin-check.yml` gate; 3 mk-cli sites) → toolkit PATCH. (The toolkit's mk-codec *library* pin is unchanged.)
- **Manual prose:** document SLIP-0132 acceptance in the mk-cli chapter (`mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md`); no flag-coverage lint impact. Check whether any CI-gated transcript ingests a ypub/zpub (none expected) → re-capture only if so.
- **FOLLOWUP:** file `mk-slip0132-prefix-acceptance` (mk repo) → resolve at ship. No sibling companion needed (mk-only).

## 9. Test plan
- **Unit (`slip132.rs` tests):** for each non-canonical prefix, `detect_and_normalize` returns the canonical xpub with byte-identical key material (depth/child/fingerprint/chaincode/pubkey) to the equivalent xpub, plus the right `Slip132Variant`. Canonical `xpub`/`tpub` → `None`. Unrecognized version → error. Implied-path predicate truth table (single-sig purpose; multisig 48'+index).
- **Integration (`tests/`):** `encode --xpub <each prefix> --origin-path <matching> --policy-id-stub deadbeef` → exit 0, emits the note, the card decodes to the same key as the equivalent xpub. Mismatch cells: `zpub + m/49'/0'/0'` → exit 64 + the actionable message; `Zpub + m/48'/0'/0'/1'` → exit 64. No-path: `zpub` + (empty/short path) → exit 64 with the supply-a-path message. Canonical: `xpub` at any path → no note, no check (unchanged). `verify --xpub <zpub> <mk1>` (no path) → normalizes + note, succeeds (key-material match); `verify --xpub <zpub> --origin-path m/49'…` → exit 64.
- **Fixtures:** derive each SLIP-0132 test string by version-swapping a known account-level xpub (the inverse of normalization) at the appropriate depth/path — `ypub`/`zpub` from a depth-3 `m/49'|84'/0'/0'` xpub; `Ypub`/`Zpub` from a depth-4 `m/48'/0'/0'/1'|2'` xpub. Reuse the `cli_address.rs` corpus xpubs where depths line up; forward-derive otherwise.
- Full crate `cargo test -p mk-cli` + `cargo clippy -p mk-cli --all-targets -- -D warnings` green.

## 10. Footguns (carry to plan-doc)
- **Two refusal walls** — the fix must intercept BEFORE `Xpub::from_str` (rust-bitcoin rejects non-canonical versions). `detect_and_normalize` must do the version-swap and re-parse; do not call `Xpub::from_str` on the raw SLIP-0132 string.
- **base58check checksum** — after swapping the 4 version bytes, the base58 checksum MUST be recomputed (don't just splice bytes into the old string).
- **multisig predicate needs the 4th path component** — guard against short paths (a `Zpub` with a <4-component path → refuse via the unsatisfiable-predicate message, not a panic/index-out-of-range).
- **depth/child guard interaction** — after normalization, mk-codec's depth/child guard (shipped) still requires `xpub.depth == origin_path.len()`. The SLIP-0132 purpose check is additional, runs first (at parse), and gives a clearer message; both must pass.
- **verify-without-path** must NOT refuse a SLIP-0132 input (skip the path predicate; note only) — else a legit key-material verify breaks.
- **`xpub`/`tpub` carry no script-type claim** → never run the predicate on canonical inputs (would break the common `xpub`-at-`84'` case that is standard today).
- **clippy `-D warnings` + `missing_docs`** (mk workspace lints) — doc every `pub` item in `slip132.rs`.

## 11. R0 fold (Minors — carry into the plan)
- **M3 (correctness) — match HARDENED components, not bare indices.** The implied-path predicate (§2/§5) must test `ChildNumber::Hardened { index }` (purpose `49'`/`84'`/`48'`; multisig 4th component `1'`/`2'`), NOT the numeric value. `DerivationPath::from_str("m/84/0/0")` parses as NORMAL children — an unhardened `84` must NOT satisfy the `84'` predicate. Encode the predicate against hardened components.
- **M1 (exit code) — mismatch is exit 64 (`UsageError`), deliberately distinct from verify's native value `ContentMismatch` (exit 4, `error.rs:84-85`).** The SLIP-0132/path contradiction is a usage error caught BEFORE the card is built, so 64 is correct. The integration cell for `verify --xpub <zpub> --origin-path m/49'…` must assert exit **64** (not 4), and document the contrast in code.
- **M2 (stderr ordering) — assert the two `note:` lines coexist + order on `encode`.** On `encode`, the SLIP-0132 `note:` fires at parse (`encode.rs:85` region) and the Phase-2 `note: stdout is watch-only` advisory fires after stdout (`encode.rs:97-100`); two distinct lines, no conflict. `verify` is inert (no advisory). Pin both lines + their order in the encode integration cell. (Correct-by-construction; just assert it.)
- **Precedent (use it):** lift the version-byte table + swap logic from `mnemonic-toolkit/src/slip0132.rs` (CI-tested) into `slip132.rs` and add a byte-parity unit test (mirror how `output_advisory.rs` duplicates+parity-tests the advisory). Fixtures: `V2_84_MAIN`@`m/84'/0'/0'` + `V1_48_MULTISIG`@`m/48'/0'/0'/2'` exist in `cli_address.rs`; the `m/49'/0'/0'` ypub fixture is forward-derived.
