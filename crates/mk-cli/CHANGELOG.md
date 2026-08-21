# Changelog

All notable changes to the `mk-cli` crate (the standalone `mk` binary) are
documented here. `mk-cli` versions independently of the `mk-codec` library; this
file is the source of truth for `mk-cli` release notes.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed — from an independent six-lens adversarial review (2026-08-21)

- **`mk encode` refuses an origin path deeper than `MAX_PATH_COMPONENTS` (10).**
  `decode_explicit_path` had always refused `count > 10` with `PathTooDeep`,
  but `encode_path` wrote the count unchecked — so the encoder minted a
  well-formed card that its own decoder refused. A **write-only card**:
  engraved in metal, unrecoverable, produced at exit 0. In a batch it was
  invisible, because the other records mint fine and the run still exits 0
  with a full-looking bundle.

  **The Go port already had this check** (`seedhammer` `mk/encode.go`
  `encodePath`), so the primary was wrong and the downstream port was right —
  this Rust fix is a convergence *onto* the port, the opposite of the usual
  direction.

- **`mk encode --from-md1` refuses a key that is not a cosigner of a KEYED
  policy.** Stamping a card asserts "this xpub is intended to serve the policy
  with this stub" (SPEC §5). Minting that claim for a key the policy does not
  contain produced a card that looks correct, engraves fine, and is refused at
  recovery — with the cosigner set already parsed in-process. Keyless templates
  are unaffected: they carry no keys, so membership is not decidable from them
  and every template-form card stays legal.

- **Piping batch output into a consumer that closes early no longer panics.**
  Rust sets `SIGPIPE` to `SIG_IGN` before `main`, so `println!` into a closed
  pipe returned `Err` and panicked — `mk encode --keys big.txt | head` gave a
  Rust panic and exit 101 after a partial bundle. Now exits 141 silently, like
  every other Unix filter. Measured before/after: exit 101 + 194 stderr bytes →
  exit 141 + 0.

### Added

- **Each card in `--json` output names its own origin** (`origin_fingerprint`,
  `origin_path`), in both the single-card and batch forms. Without it the batch
  handed back N interchangeable blocks whose only link to the input records was
  position, forcing any consumer captioning plates to assume card order still
  matches file order — the assumption this project already has an incident for
  (30 plates captioned with the wrong cosigner). Consumers can now join on
  identity instead of counting. Additive; existing keys unchanged.

### Fixed — tests that passed while the behaviour they name was broken

- `json_batch_wraps_the_single_card_object` checked only `cards[0]`, so a
  mutant swapping `cards[1]`/`cards[2]` passed all 132 tests in the crate. Now
  checks every index, and asserts each card names its own record.
- `verify_from_keyless_template_md1_matches_template_id_stub` never compared
  against the golden stub — it tested self-agreement plus difference from one
  unrelated literal. Two independent corruptions of the identity logic survived
  it while killing its siblings. Now anchored to `EXPECTED_TEMPLATE_STUB`.
- The keyed-policy fixtures minted cards for an xpub that is **not** a cosigner
  of the fixture policy — the very defect the membership check now refuses.
  Switched to a real member; the stub does not depend on which cosigner is
  carded, so the assertions are unchanged.

### Changed — docs

- `docs/MK_CODEC_RUST_API.md` cited `md_codec::compute_policy_id_stub`, which
  **has never existed** (0 occurrences in md-codec's source), and described the
  stub with a formula wrong under every rule this project has shipped. The
  example did not compile. Replaced with the real form-aware dispatch.
- `bip/bip-mnemonic-key.mediawiki` contradicted itself on the exact rule F-128
  fixed: its glossary and "Policy ID stubs" section stated the unconditional
  WalletPolicyId while its own "Linkage to MD" section was already form-aware.
  An implementer reading only the first two would have built a silently wrong
  implementation — in the document furthest along toward external publication.
  All three sections now agree.


### Fixed

- **`mk encode --from-md1` / `mk verify --from-md1` can read a CHUNKED md1 at
  all (F-127).** Two defects, stacked. The vendored `md-codec` was **0.34.0**,
  five wire versions behind, so every chunk was refused with
  `wire-format version mismatch: got 9, expected 4`; bumped to **0.42.0**. With
  that fixed the error only *changed*, to
  `chunk set incomplete: got 1 chunks, expected 4`, because each `--from-md1`
  value was decoded INDEPENDENTLY — a four-chunk card was four incomplete sets.
  Values are now grouped by the 20-bit chunk-set id in their wire header, and
  each GROUP yields one stub.

  Scale, measured: a keyed wallet policy is 246 data symbols against a
  single-string cap of 80, so **every** keyed wallet-policy card is chunked.
  This was not a large-wallet edge case — `--from-md1` was unusable for all of
  them, and only ever worked on keyless templates short enough to fit one
  string.

  `--from-md1` still means one card per POLICY: grouping keys on the set id,
  not on adjacency, so a key card belonging to two wallets still gets two stubs
  in first-appearance order.

### Added

- **`mk encode --keys <FILE>`** — mint one card per key record instead of one
  card per invocation (F-223). Records are BIP-380 origin notation, one per
  line (`[fingerprint/path]xpub`); blank lines and `#` comments are ignored,
  and `-` reads stdin. Every card receives the same
  `--policy-id-stub`/`--from-md1` binding.

  Measured on an 11-cosigner wallet: **11 invocations → 1**, with output
  byte-identical to the loop it replaces.

  The record format was chosen over three parallel repeatable flags because
  parallel lists can desync, and a desync does not fail — it mints a card
  naming the wrong master.

  `--keys` is mutually exclusive with `--xpub`, `--origin-path`,
  `--origin-fingerprint`, `--chunk-set-id` and `--privacy-preserving`: each
  record carries its own origin, so a global one would have to override it or
  be ignored. Privacy-preserving cards are minted one at a time, deliberately —
  a record always declares a fingerprint, and dropping it silently is how a
  card gets engraved wrong.

  Plain output separates cards with a blank line (single-card output is
  unchanged — no leading or trailing blank). `--json` gains a `cards` array
  whose entries are exactly the object the single-card form emits.

  **`mk gui-schema` is byte-identical** — verified by diffing against a build of
  the previous tree. `--keys` is deliberately excluded from the GUI contract:
  the schema describes the form `mnemonic-gui` renders, that form mints ONE
  card, and `mnemonic-gui`'s mirror is hand-written with no automated gate
  against this repo.

### Changed — docs

- **`SPEC_mk_v0_1.md` §3.3 and §5 now state the FORM-AWARE stub rule (F-128)**:
  `WalletPolicyId` for a keyed wallet policy, `WalletDescriptorTemplateId` for a
  keyless template. The spec had named `WalletPolicyId` unconditionally while
  shipped `mk` has always dispatched on the form. §5's recovery flow was the
  load-bearing half — followed literally it would reject **every** card minted
  from a template, presenting as "none of my cards belong to this wallet".

## [0.13.0] — 2026-08-19 — consumes mk-codec 0.5.0 (BREAKING: derived `chunk_set_id`)

### Added

- **`mk encode --chunk-set-id <HEX>`** — pin the 20-bit `chunk_set_id` instead
  of letting it be derived from the payload. Chunked output only; single-string
  encodings carry no such field. For vector regeneration and conformance
  fixtures — the derived default is already deterministic, so ordinary encoding
  never needs it.

### Changed — BREAKING (inherited from mk-codec 0.5.0)

- `chunk_set_id` is now **derived** from the payload
  (top 20 bits of `SHA-256(canonical_bytecode)`, MSB-first) rather than drawn
  from the OS CSPRNG per call. Encoding the same card twice now reproduces the
  same strings; previously it did not, in violation of SPEC §2.5. See the
  `mk-codec` 0.5.0 entry for the full rationale.
- **Fixtures, goldens and transcripts holding a previously-random
  `chunk_set_id` must be regenerated.**

### Added — tests

- T4 external BIP-84/86 address and BIP-32 compact-form oracles.

## [0.12.1] — 2026-07-10 — SemVer-patch; consumes mk-codec 0.4.2 (docs + depth-0 vector; no wire/behavior change).

## [0.11.2] — 2026-06-23

**SemVer-PATCH — musl static-binary release asset + musl build/test CI leg. Ships the first fully-static, dependency-free `mk` Linux binaries (`x86_64-unknown-linux-musl` + `aarch64-unknown-linux-musl`) as GitHub-release tarballs on the `mk-cli-v*` tag, each with a per-arch `SHA256SUMS.<arch>` for offline / air-gapped verification. Also adds a musl compile/test CI leg to `ci.yml` (a separate `musl-check` job) and a dedicated `musl-binaries.yml` workflow triggered ONLY on `mk-cli-v*` (NOT the `mk-codec-v*` codec tag, so a pure codec tag does not build a CLI binary). `mk-codec` UNTOUCHED. No crate source / API / CLI-flag / subcommand change. NOT published to crates.io (binary-asset-only PATCH; the tag ships the binary). The shipped guarantee is *static + checksummed*, not bit-for-bit reproducible.**

### Added

- **musl static-binary release-asset workflow** (`.github/workflows/musl-binaries.yml`, NEW — standalone, fires ONLY on `mk-cli-v*`). Builds `mk` for `x86_64-unknown-linux-musl` (natively with `musl-tools` + `CC_x86_64_unknown_linux_musl=musl-gcc`) and `aarch64-unknown-linux-musl` (via `cross`), tarballs each as `mk-<version>-<arch>-linux-musl.tar.gz`, emits a per-arch `SHA256SUMS.<arch>`, and attaches them to the `mk-cli-v*` release via `gh release upload --clobber` (alongside the `mk-man.tar.gz` the existing `ci.yml` release-on-tag job attaches). Standalone (not folded into `ci.yml`) because `ci.yml` also fires on the `mk-codec-v*` codec tag, which must NOT build a CLI binary. `crt-static` left at its musl default (ON); `-Ctarget-feature=-crt-static` never set (per `rust#135244`). Toolchain pinned `@1.85.0`. The only C dep is the vendored libsecp256k1 in `secp256k1-sys`.
- **musl compile/test CI leg** (`.github/workflows/ci.yml`, new `musl-check` job). `cargo test -p mk-cli --target x86_64-unknown-linux-musl` (native, `musl-tools` + `CC_*=musl-gcc`) + `cross build` for aarch64-musl (build-only). Pinned `@1.85.0` (constellation parity).

## [0.11.1] — 2026-06-23

**SemVer-PATCH — BSD secret-hygiene parity + FreeBSD compile-gate. `set_non_dumpable()` (in `crates/mk-cli/src/process_hardening.rs`) was fenced `#[cfg(target_os = "linux")]` and a silent no-op on the BSDs, so an `mk` process on FreeBSD/OpenBSD/NetBSD could be ptrace/ktrace-introspected and could drop a core file a secret (passed inline on argv/heap) spills into. A second cfg arm restores parity. No new CLI flag / subcommand / output-shape. Linux behavior unchanged (the new arm is cfg-gated off everywhere but the BSDs). `mk-codec` UNTOUCHED. Shipped in lockstep with `mnemonic-toolkit` 0.73.1 / `md-cli` 0.11.1 / `ms-cli` 0.13.1 (byte-identical executable arm in all four CLI crates).**

### Changed

- **`set_non_dumpable()` gains a BSD parity arm** (`crates/mk-cli/src/process_hardening.rs`). Keeps the Linux `prctl(PR_SET_DUMPABLE, 0)` arm; adds a `#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]` arm that does (i) on FreeBSD only, `procctl(P_PID, 0, PROC_TRACE_CTL, PROC_TRACE_CTL_DISABLE)` (disables ptrace/ktrace introspection AND core dumping) and (ii) on all three BSDs, `setrlimit(RLIMIT_CORE, {0, 0})` (hard-zeros the core-dump size). Best-effort (return values ignored). macOS/Windows remain a documented no-op. No `libc` version bump. Compile-gated BSD unit tests added (compile-checked but never executed by the chosen CI).
- **FreeBSD compile-gate added to CI** (`.github/workflows/ci.yml`, new `freebsd-compile-gate` job). Runs a WHOLE-CRATE `cargo check --target x86_64-unknown-freebsd -p mk-cli` (NOT `--lib` — mk-cli is bin-only and its `process_hardening` lives in the bin target; a `--lib` check would be silent false-green). Compile-covers the BSD hardening arm.

## [0.11.0] — 2026-06-23

**SemVer-MINOR — new `mk gen-man` subcommand (man-page self-emission). No wire/format change; `mk-codec` untouched.**

- **`mk gen-man --out <DIR>`** generates roff man pages for the whole `mk`
  CLI tree (`mk.1` + one `mk-<sub>.1` per subcommand) into `<DIR>`,
  creating it if absent. Pages are clap-generated via
  `clap_mangen::generate_to(Cli::command(), &dir)` — the bare, naive call
  with NO pre-`.build()` — so they are binary-faithful by construction and
  carry zero `*-help*.1` shadow pages. Part of the constellation-wide
  man-page rollout (`mnemonic`/`md`/`ms`/`mk` all gain `gen-man`).
- New dependency `clap_mangen = "0.3"` (requires clap `^4.0`; no clap bump —
  already on 4.6.1).
- `install.sh` drops these into the user manpath post-install
  (`~/.local/share/man/man1`), and the `mk-cli-v*` tag attaches an
  `mk-man.tar.gz` release asset.

## [0.10.2] — 2026-06-22

**SemVer-PATCH — help-text correction only. No flag, no API, no wire/behavior change; `mk-codec` untouched.**

- **`mk vectors --pretty` help text corrected.** The clap doc-comment on `--pretty`
  (`src/cmd/vectors.rs`) previously claimed it was "Ignored when `--out` is supplied".
  That was wrong: `write_per_fixture_files` honors `--pretty`, pretty-printing each
  per-fixture file written under `--out`. The help text now states `--pretty` applies
  to the per-fixture files as well. Behavior is unchanged (the code was already correct);
  this fixes the documented contract. A new `vectors_pretty_out_writes_indented_files`
  test pins it. Closes FOLLOWUP `mk-vectors-pretty-out-help-mismatch`.

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
