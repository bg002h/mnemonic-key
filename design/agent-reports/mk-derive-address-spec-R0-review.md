# R0 Architect Review — SPEC_mk_derive_address.md (mk-cli 0.6.0)

Reviewer: feature-dev:code-reviewer (opus). Reviewed `design/SPEC_mk_derive_address.md` against
current source @ `ed2596d` (branch `mk-derive-address-readonly`), bitcoin 0.32.8 API, the toolkit
reference implementations, and the GUI/manual lockstep targets.

**Verification:** every §2 `file:line` citation read against real source; bitcoin-0.32 primitives
confirmed via the toolkit's live `render_address`/`build_address_from_xpub`/`network_from_xpub` +
docs.rs; GUI `schema/mk.rs`, `pinned-upstream.toml`, mk-cli `error.rs`/`gui_schema.rs`/
`tests/gui_schema.rs`, and the manual `44-mk-cli.md` all read.

Load-bearing technical core is SOUND: every §2 citation accurate; the bitcoin-0.32 API the SPEC
assumes all exists with the assumed signatures (`Xpub::derive_pub`, `Xpub.network: NetworkKind`
Main/Test-only, `.fingerprint()`, `.depth`, the four `Address::p2*` builders, `DerivationPath:
FromStr + IntoIterator`, `ChildNumber::is_hardened()`). Relative-derivation semantics, the
multisig `48'/87'` signal, and the network-kind guard are correct.

## Critical

**C1 — The four new error variants violate the published exit-code contract (SPEC §3.1/§3.2 say
"exit 2"; the contract reserves 2 for codec rejection and uses 64 for usage errors).**
`mk-cli/src/error.rs:79-88` + published manual `44-mk-cli.md:397-406` reserve exit **2 exclusively
for "mk1 format violation; codec rejected the input"** (`Codec(_) | MdCodec(_)`), and map all CLI
usage/input errors to **exit 64** (`UsageError(_) => 64`). The four new conditions
(`MultisigCardNotAddressable`, `AddressTypeRequired`, `NetworkXpubMismatch`, `HardenedFromXpub`)
are usage/input errors, structurally identical to the `encode.rs:59` mutual-exclusion guard which
returns `CliError::UsageError` → 64. Shipping as exit 2 contradicts the contract, misleads scripts
branching on "2 = malformed mk1," and diverges from sibling-CLI convention. **Fix:** route all four
through `CliError::UsageError(String)` (exit 64, zero new variants, matches `encode.rs` precedent),
or give dedicated variants **exit 64** and extend error.rs's four match arms. State 64, not 2.

**C2 — The SPEC never specifies updating the four exhaustive `match self` blocks in error.rs; as
written it will not compile.** `error.rs` has four exhaustive matches over `CliError`: `kind()`
(:52-59), `message()` (:64-75), `exit_code()` (:80-87), `details()` (:92-104). In-crate matches, so
adding variants without arms is a compile error. §3 contains no instruction to extend these. **Fix:**
adopt the `UsageError`-reuse path from C1 (no new variants, no error.rs change — strongly preferred,
matches encode/decode where every input error is a `UsageError`), or add an explicit subsection
enumerating new variants AND the four match-arm additions AND the manual error-kind list update.

## Important

**I1 — GUI schema-mirror drift is materially larger than §4 claims; `repair` is entirely missing
from `schema/mk.rs`.** §4 says the pin is "behind (v0.4.2 < 0.5.0)" — reality is worse:
`mnemonic-gui/src/schema/mk.rs` header + `pinned_version` say **"mk 0.3.1"** (lines 1, 312), and
`SUBCOMMANDS` (267-308) lists only inspect/encode/decode/verify/vectors — the entire **`repair`
subcommand is absent** (shipped in `enum Command`, never mirrored). The same paired PR adding
`address`+`derive` must also backfill `repair`, bump `pinned_version` "mk 0.3.1"→"mk 0.6.0"
(schema/mk.rs:312 + header :1), and bump `pinned-upstream.toml:52` v0.4.2→v0.6.0. Exactly the
"lagging indicator accumulates silently" mode CLAUDE.md's `gui-schema-mirror-lockstep-discipline`
warns of. **Fix:** rewrite §4's GUI bullet to enumerate: (a) add `address`+`derive`
SubcommandSchemas, (b) backfill the missing `repair` SubcommandSchema, (c) bump pinned_version string
+ header, (d) bump pinned-upstream.toml:52, (e) dropdown enums (`AddressType`, `CliNetwork`,
`--chain`) use `FlagKind::Dropdown`/value-enum mirroring (gate enforces dropdown-value parity), (f)
`--count`/`--index` → `FlagKind::Number`, `--range` → `FlagKind::Range`.

**I2 — `mk address` on a non-account-level (leaf) card silently emits garbage addresses; no depth
guard.** The §3.1 heuristic keys off `origin_path`'s first component then derives `m/c/i` relative to
whatever xpub the card holds. Correct for an account-depth card (`m/84'/0'/0'`, depth 3); for a leaf
(`m/84'/0'/0'/0/5`, depth 5) it still reads `84'`→p2wpkh and derives `…/0/5/c/i`, addresses that
match no wallet. `xpub.depth` (u8, confirmed) is the signal. **Fix:** add a depth check — require the
card be at canonical account depth for its purpose (3 for 44/49/84/86), OR document that derivation
is relative to the card's xpub and emit a stderr advisory when `xpub.depth` ≠ canonical account
depth.

**I3 — `child_fingerprint` formatting isn't pinned; `fingerprint()` returns `Fingerprint`, not a hex
string.** §3.2 says "`child_fingerprint = child.fingerprint()`" + JSON `"aabbccdd"`. The house
formatter is `fmt_fingerprint(&Fingerprint) -> String` (mod.rs:73), used `fmt_fingerprint(&card.xpub.
fingerprint())` (inspect.rs:44). **Fix:** §3.2 state `fmt_fingerprint(&child.fingerprint())` (note the
`&`), matching inspect.rs.

## Minor
- **M1** — `AddressType` kebab: `P2shP2wpkh` auto-kebab needs `#[clap(rename_all = "kebab-case")]`
  (or per-variant `#[value(name="p2sh-p2wpkh")]`); confirm `CliNetwork` uses `rename_all = "lower"`
  (toolkit network.rs:11).
- **M2** — `parse_range` doesn't exist in mk-cli (SPEC-only); state `--range A,B` is net-new local
  code, `A ≤ B` → `CliError::UsageError` (64).
- **M3** — `mk gui-schema` emits `kind:"text"` for numeric flags (no numeric mapping in classify);
  GUI mirror uses `FlagKind::Number`/`Range`. Gate checks flag-NAME + dropdown-value parity only, so
  tolerated — note it. mk-cli `tests/gui_schema.rs` is reflective (`contains`, lines 113-133), so
  §5 test 16 should target the **GUI repo's** schema_mirror test, not mk-cli's.
- **M4** — manual `44-mk-cli.md:5` count + `:12` install tag `mk-cli-v0.4.0` are stale; lockstep PR
  fixes count + bumps tag → `mk-cli-v0.6.0`.
- **M5** — §3 uses `secp` but never says how to build it: `Secp256k1::verification_only()`
  (verify_message.rs:55) — minimal correct context for `derive_pub`+`p2tr`, reinforces no-sign.
- **M6** — 48'-card single-key override refused even with explicit `--address-type` (§3.1 step 1):
  defensible for mk1 (multisig-cosigner cards), but document it's intentionally not honored + point
  to `mnemonic convert --to address` escape hatch.

## No-sign boundary — confirmed clean
`KeyCard` carries only `xpub` (key_card.rs:53); both subcommands consume `xpub`, derive public
children, render addresses/xpubs/fingerprints. `verification_only()` secp reinforces. Holds
structurally.

## Test plan — adequate, two additions
Add (1) a leaf/over-deep card test (I2), (2) once C1 resolved, tests 3/4/7/11 assert **64** not 2.

**VERDICT: RED (2C/3I)**

---

## Fold applied (controller, verified against source @ ed2596d)
- **C1+C2:** confirmed `error.rs` `UsageError → 64`, codec → 2, and `encode.rs:59/72` precedent.
  Folded to **reuse `CliError::UsageError(String)`** for all four conditions → exit **64**, ZERO new
  variants, no error.rs match-arm changes. §5 tests updated 2→64.
- **I1:** confirmed schema/mk.rs header "mk-cli-v0.3.1" + `repair` absent. §4 GUI bullet rewritten to
  enumerate add address+derive, backfill repair, bump header/pinned_version/pinned-upstream.toml, the
  FlagKind::Dropdown/Number/Range mappings.
- **I2:** §3.1 now gates the purpose-heuristic on `origin_path.len()==3` (canonical single-sig
  account depth) + emits a stderr advisory when the card's depth ≠ canonical account depth (derivation
  is relative to the card's xpub).
- **I3:** §3.2 now `fmt_fingerprint(&child.fingerprint())`.
- **M1-M6** folded: kebab/lower rename_all; --range net-new + UsageError; gui-schema numeric=text
  note + §5 test-16 retarget to GUI repo; manual count+tag; `Secp256k1::verification_only()`; 48'
  override-not-honored doc + toolkit escape hatch.
