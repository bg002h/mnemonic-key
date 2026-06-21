# Changelog

All notable changes to the `mk-cli` crate (the standalone `mk` binary) are
documented here. `mk-cli` versions independently of the `mk-codec` library; this
file is the source of truth for `mk-cli` release notes.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.1] — 2026-06-21

**SemVer-PATCH — two bug fixes in `mk` output hygiene (constellation bug-hunt cycle-12). No new flags, no wire/format change, `mk-codec` untouched.**

- **M12 — `mk repair` no longer emits invalid mixed-case `mk1` for all-uppercase input.** `reconstruct_corrected` (`src/cmd/repair.rs`) previously spliced the input's uppercase `MK1` prefix with lowercase bech32 data, producing a mixed-case string that `mk decode` rejects (`Error::MixedCase`, exit 2). The prefix is now lowercased so the emitted string is uniformly all-lowercase (codec canonical) and re-decodes to the same xpub. Lowercase input is unaffected (idempotent).
- **L20 — `classify_code_variant` off-by-one corrected.** A 96-symbol data-part (total length 99) was mislabeled `"regular"` when the authoritative `bch_code_for_length` (mk-codec) puts the "long" band at 96..=108. The threshold is now `93 + "mk1".len()` (data-part ≤ 93 ⇒ regular; ≥ 96 ⇒ long). Affects the display-only `code_variant`/`chunk_variants` fields of `mk decode`/`encode`/`inspect`; no funds/wire impact.

## [0.10.0] — 2026-06-19

**SemVer-MINOR — `policy_id_stub` derivation is now FORM-AWARE: a keyless template md1 binds on `WalletDescriptorTemplateId`, a keyed wallet-policy md1 on `WalletPolicyId`. Aligns `mk --from-md1` with `mnemonic-toolkit` #28 (`bundle --md1-form=template`).**

`mk encode --from-md1` and `mk verify --from-md1` previously derived the 4-byte
stub from `md_codec::compute_wallet_policy_id` **unconditionally**. For a
**keyless template** md1 (`!is_wallet_policy()` — e.g. a single-sig template
bundle, or any plain `pkh`/`wsh(...)` template with no `Pubkeys` TLV) that
computed the WRONG identity. `derive_stub_from_md1` now discriminates on
`md_codec::Descriptor::is_wallet_policy()`, mirroring the toolkit's
`bundle_binding_stub` (toolkit #28):

- **keyless template** (`!is_wallet_policy()`) → top 4 bytes of
  `md_codec::compute_wallet_descriptor_template_id` (md SPEC §8.1, key-stable
  BIP-388 template identity);
- **keyed wallet-policy** (`is_wallet_policy()`) → top 4 bytes of
  `md_codec::compute_wallet_policy_id` (md SPEC v0.13 §5.3 — the pre-#28 path,
  unchanged).

So a stub minted via `mk --from-md1` from a toolkit-emitted **template** bundle
now agrees byte-for-byte with the stub the toolkit stamped on the same card.

**Behavior change:** a stub previously stamped via `--from-md1` from a *keyless
template* md1 no longer matches (it now resolves to the template-id, not the
policy-id) — hence MINOR. Keyed wallet-policy md1s are unaffected.

No `mk-codec` change and **no `md-codec` pin bump** — both
`compute_wallet_descriptor_template_id` and `is_wallet_policy()` are public and
re-exported at the pinned `md-codec-v0.34.0`, and all stub goldens are
byte-stable across `md-codec` 0.34.0 → 0.37.0.

### Changed

- `crates/mk-cli/src/cmd/mod.rs`: `derive_stub_from_md1` gains the
  `!is_wallet_policy()` → `compute_wallet_descriptor_template_id` branch (form
  dispatch); rustdoc updated.
- `crates/mk-codec/src/key_card.rs`: `KeyCard::policy_id_stubs` field rustdoc
  corrected — the stub is no longer "always the WalletPolicyId" but the
  form-aware canonical identity.

### Added

- `crates/mk-cli/tests/template_id_stub.rs`: form-aware cells — keyless template
  `mk encode`/`mk verify` use the template-id stub (RED before the fix), and a
  keyed wallet-policy md1 still uses the policy-id stub (regression). Goldens
  are frozen INDEPENDENT literals (audit-I1 discipline).
- `crates/mk-cli/tests/round_trip.rs`: `from_md1_derivation` golden updated to
  the template-id stub (`PKH_BASIC_MD1` is a keyless `pkh` template).

## [0.9.0] — 2026-06-15

**SemVer-MINOR — standardized mstring display-grouping; `mk encode` text output is now space/5 print-once (was unbroken — corrective alignment with the other CLIs). Part of the cross-constellation `display-grouping-render-strip-v1` cycle (P3).**

### Added

- **`mk encode --group-size <u16>`** (default `5`, `0` = unbroken) + **`--separator <space|hyphen|comma>`** (keyword or literal `" "|-|,`, default `space`) — insert a separator every N characters in each emitted mk1 string. SPEC §3/§5. The default `mk encode` text output is now **space/5, single line, print-once** (previously UNBROKEN — a corrective default-output change bringing `mk` into line with `ms`/`md`), hence MINOR. `--json` ALWAYS carries the canonical **unbroken** string(s).
- **Separator-stripping intake on all six mk1-intake subcommands** (`decode`/`inspect`/`verify`/`repair`/`derive`/`address`) via the shared `read_mk1_strings`, on both the positional and `-`→stdin paths: a grouped or unbroken card both re-ingest. Strips ALL whitespace + `-` + `,` (SPEC §3.2) — previously `read_mk1_strings` only `.trim()`med edge whitespace, so interior separators were rejected. (mk-codec's decode tolerates no separators; this is a pure CLI-layer normalization.)
- Conformance vectors `design/display-grouping-vectors.tsv` (byte-identical copy of the toolkit canonical) + `.sha256`, CI-pinned (`sha256sum -c` in the fmt job) + a bin-crate driver test over every row.

### Notes

stdout text was never a declared-stable interface and `--json` is unaffected. **`mk-codec` is UNCHANGED** (the pure fns `render_grouped`/`strip_display_separators`/`is_display_separator`/`parse_separator` are mk-cli-local — mk-cli is bin-only; the conformance test is a bin-crate `#[cfg(test)]`). The `mk-codec` dep pin stays `0.4.0`. Cross-repo lockstep (toolkit collapse + manuals; `mnemonic-gui` schema-mirror flags + separator dropdown) lands in later phases; FOLLOWUP `display-grouping-render-strip-v1`.

## [0.8.0] — 2026-06-10

**SemVer-MINOR — `policy_id_stub` derivation aligned to the constellation's `WalletPolicyId`.** `mk encode --from-md1` and `mk verify --from-md1` now derive the 4-byte Policy ID stub from `md_codec::compute_wallet_policy_id(descriptor)` (md SPEC v0.13 §5.3 — the canonical-*expanded*, encoding-stable policy identity) instead of the md1 bytecode hash `SHA-256(canonical_bytecode)`. This matches `mnemonic-toolkit`'s `synthesize.rs` stub formula byte-for-byte, so a stub minted via `mk --from-md1` now agrees with toolkit-emitted bundle cards **and** survives a re-encode of the same logical wallet (origin/use-site elision, override-vs-baseline path placement) — which the bytecode hash did not. **Behavior change:** a stub a user previously stamped via the OLD `--from-md1` no longer matches. The bytecode-hash formula predated md-codec v0.13's WalletPolicyId and was stale; SPEC §3.3/§5/§9 + the BIP draft are updated in lockstep. No `mk-codec` change and no `md-codec` pin bump (`compute_wallet_policy_id` is present and byte-stable at the pinned `md-codec-v0.34.0`). Resolves `audit-2026-06-10-backlog` items `stub-formula-divergence` (I1) + `from-md1-test-tautology` (I2).

## [0.7.0] — 2026-05-30

**SemVer-MINOR — SLIP-0132 typed-prefix acceptance (A2).** (Backfilled entry — release commit `ac76f2d`; predated this file's coverage.) Resolved a tracked FOLLOWUP.

## [0.6.1] — 2026-05-30

**SemVer-PATCH — test-only.** (Backfilled entry — release commit `1748bd8`.) Added inert-subcommand negative-test cells (Phase 2). No CLI surface change.

## [0.6.0] — 2026-05-30

**SemVer-MINOR — two new read-only public-derivation subcommands: `mk address` + `mk derive`.** No private keys, no signing (an xpub has none); read-only by construction.

- **`mk address`** — render N receive/change addresses controlled by a card's xpub. The address type is inferred from the origin-path purpose **at canonical single-sig account depth** (`m/44'`→p2pkh, `49'`→p2sh-p2wpkh, `84'`→p2wpkh, `86'`→p2tr) and is overridable with `--address-type`; a card whose origin is **not** at account depth requires the explicit flag (and emits a stderr advisory that addresses are derived relative to the card's xpub). Multisig-cosigner cards (`m/48'`/`m/87'`) are **refused** (single-key addresses would not match the wallet). `--count N` (default 10) / `--range A,B`; `--chain receive|change|both`; `--network` override that must agree with the xpub's network kind (distinguishes signet/regtest); `--json`.
- **`mk derive`** — derive a child xpub at a relative **unhardened** path (`--path m/0/5`, or `--index N` sugar for `m/0/N`); hardened components are rejected (an xpub cannot derive them). Multisig cards are allowed (per-cosigner child derivation is legitimate). The emitted `child_xpub` is composable back through `mk encode`. `--json`.
- New shared `cmd::derive_support` module (`AddressType`/`CliNetwork` value enums, account-depth-gated `infer_address_type`, `render_address` under a `verification_only()` secp). No `mk-codec` change. `mk gui-schema` auto-reflects the new surface (value-enum dropdowns); paired `mnemonic-gui` schema-mirror + manual (`44-mk-cli.md`) + install-pin updates land in lockstep.

## [0.5.0] — 2026-05-30

**SemVer-MINOR — adopt `mk-codec 0.4.0` (mk1 no-path / depth-0 support).** A WIF / non-HD key now round-trips as an mk1 card carrying an empty wire path (no derivation path applies, so none is encoded); the decoder accepts a consistent depth-0 card. `mk-codec 0.4.0` also added the encode-time `XpubOriginPathMismatch` guard (rejects any card whose `xpub.depth`/`child_number` disagree with `origin_path`). (Backfilled changelog entry — the 0.5.0 release predated this file's coverage.)

## [0.4.2] — 2026-05-23

**SemVer-PATCH — process argv-hardening (`PR_SET_DUMPABLE`).** `mk` now calls `prctl(PR_SET_DUMPABLE, 0)` at the top of `main()` (Linux; no-op elsewhere), making `/proc/$PID/` unreadable to OTHER non-root UIDs and disabling core dumps — so a secret passed inline on argv can no longer be harvested by another user via `/proc/$PID/cmdline` or a core file. Residual same-UID window documented + accepted. New `process_hardening` module + `libc` dep. Part of the m-format constellation argv-hardening rollout (mnemonic-toolkit v0.34.7 + md-cli v0.6.1 + ms-cli v0.4.1). Tracked via the toolkit's `argv-overwrite-after-parse` FOLLOWUP closure.

## [0.4.1] — 2026-05-17

Patch release closing `from-md1-derivation-wire-version-skew` (filed
during v0.22.x follow-ups cycle Phase A.1' execution). Standalone
patch — no sibling lockstep.

### Changed

- `crates/mk-cli/Cargo.toml`: `md-codec` dep `=0.32.1` → `=0.34.0` (two
  minor-version jump; stale pin from before the v0.18+ wire-format
  release cycle).
- `crates/mk-cli/tests/round_trip.rs`: `PKH_BASIC_MD1` const refreshed
  from pre-v0.18 wire-format `md1qqpqqxyepwspuepy268e` to v0.34.0
  canonical `md1yqpqqxzq2qwfv8urt848e` (byte-exact with md-codec
  v0.34.0's `tests/vectors/pkh_basic.phrase.txt`).

### Fixed

- `from_md1_derivation` integration cell (`tests/round_trip.rs:45`)
  previously failed `WireVersionMismatch { got: 0 }` against any
  md-codec ≥ v0.18 — the fixture had not been refreshed since v0.17.
  Now passes against md-codec v0.34.0.

### Resolved (FOLLOWUPS)

- `from-md1-derivation-wire-version-skew` — fixture refresh + dep
  bump documented above.

## [0.3.1] — 2026-05-12

### Fixed

- `mk --version` and `mk --help` now exit `0` instead of `64`. The
  v0.3.0 `fn main()` mapped every `Cli::try_parse()` `Err` to
  `ExitCode::from(64)`, but clap returns `Err` for two non-error
  terminations as well — `ErrorKind::DisplayVersion` (`--version`)
  and `ErrorKind::DisplayHelp` (`--help`). The output already
  prints to stdout in those cases; the canonical Unix convention
  is exit 0. The fix branches on `e.kind()` and returns
  `ExitCode::SUCCESS` for the two display variants, preserving the
  catch-all 64 for real parse errors.
  Discovered during `bg002h/mnemonic-gui` v0.2.0 release prep
  (companion: `bg002h/mnemonic-gui`).
- New regression file `tests/version_help_exit_codes.rs` with
  three cells: `version_flag_exits_zero_and_prints_version`,
  `help_flag_exits_zero_and_prints_help`, and
  `unknown_flag_exits_64` (the negative-case backstop).
- `tests/gui_schema.rs` — replaced eight
  `s["name"] == Value::from("...")` patterns inside `.find()`
  closures with the equivalent `s["name"] == "..."`. The
  `clippy::cmp_owned` lint flagged the `Value::from` allocations
  as unnecessary; the v0.3.0 tag-push CI run failed all 9
  build-matrix jobs on this. The fix is mechanical (clippy's own
  suggestion) and unrelated to the exit-code surface, but is
  folded into this patch to make the v0.3.1 tag CI run actually
  green.
- `cargo fmt` applied to `src/main.rs`, `src/cmd/gui_schema.rs`,
  and `tests/gui_schema.rs` — additional pre-existing formatting
  drift that v0.3.0's tag CI flagged via the `Rustfmt` step. The
  formatter's own output; same release-hygiene rationale as the
  clippy fold above.

## [0.3.0] — 2026-05-12

Adds the `mk gui-schema` subcommand for consumption by `mnemonic-gui`'s
schema-mirror gate. Realizes Section C.2 of the `mnemonic-gui` v0.2 plan
(per the cross-repo `mnemonic-gui-schema-mirror` FOLLOWUPS entry).

### Added

- `mk gui-schema` — zero-argument subcommand that prints a machine-readable
  JSON description of the CLI's clap-derive flag surface. The JSON contract
  is the SPEC §7 shape shared across all four sibling CLIs (`md`, `ms`, `mk`,
  `mnemonic`):
  ```json
  {
    "version": 1,
    "cli": "mk",
    "subcommands": [
      { "name": "encode",
        "flags": [ { "name": "--xpub", "required": true, "kind": "text", "choices": null }, ... ],
        "positionals": [] },
      ...
    ]
  }
  ```
  `kind` is one of `"text"` / `"boolean"` / `"number"` / `"dropdown"` /
  `"path"`; complex types map to `"text"`. `choices` is non-null only for
  `"dropdown"`. The `gui-schema` and `help` subcommands are excluded from
  the emitted list.
- `crates/mk-cli/tests/gui_schema.rs` — 7 integration tests pinning the
  envelope shape, required-flag detection, and kind classification.

### Notes

- Wire format, decoder, encoder, vectors-corpus: **byte-identical** to
  `mk-cli` v0.2.0. This release adds a reflective subcommand only; no
  existing subcommand surface, flag, or behavior changes.
- Schema-mirror gate consumers (e.g., `mnemonic-gui`) may now invoke
  `mk gui-schema` instead of regex-extracting tokens from `mk <sub> --help`.
- Cross-repo lockstep: companion entries in `mnemonic-gui/FOLLOWUPS.md` and
  parallel `gui-schema` PRs landing in `descriptor-mnemonic` (md-cli) and
  `mnemonic-secret` (ms-cli) on the same cycle.

## [0.2.0] — 2026-05-08

Initial standalone `mk-cli` release. Provides the `mk` binary with
`encode`, `decode`, `inspect`, `verify`, and `vectors` subcommands.
See the manual chapter `mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md`
for the canonical user-facing flag surface.
