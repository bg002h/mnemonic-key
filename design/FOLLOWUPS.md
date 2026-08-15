# mk1 follow-up tracker

Single source of truth for items surfaced during a review or implementation pass that were not fixed in the same commit. Mirrors the convention in the `descriptor-mnemonic` (md1) repo.

## How to use this file

**Format for each entry:**

```markdown
### `audit-2026-06-10-backlog` — verified findings from the first independent Fable constellation audit

- **Surfaced:** 2026-06-10, the 23-agent read-only architecture audit (find → adversarial-verify → synthesize). 48 verified findings constellation-wide (0 critical); this repo's share below. **Full report + per-finding detail (claim/evidence/fix/disposition):** `../../mnemonic-toolkit/design/agent-reports/constellation-architecture-audit-2026-06-10.md` (committed in the toolkit repo). Promote any line to its own `### <id>` entry when worked; resolve here as fixed.
- **This repo's verified findings (4):**
  - **[IMPORTANT] ✓ RESOLVED (mk-cli v0.8.0, 2026-06-10)** `from-md1-test-tautology` — promoted to its own entry below; de-tautologized to a frozen literal golden.
  - **[IMPORTANT] ✓ RESOLVED (mk-cli v0.8.0, 2026-06-10)** `stub-formula-divergence` — promoted to its own entry below; mk-cli aligned to the toolkit's `WalletPolicyId` stub formula.
  - **[obs]** `noncanonical-path-encoding-accepted` — A standard-table path (e.g. m/44'/0'/0') can be decoded either from its 1-byte table indicator or from an explicit 0xFE+count+LEB128 encoding; decode_path accepts both and yields the same DerivationPa (`crates/mk-codec/src/bytecode/path.rs:101-132 (decode_path / decode_explicit_path) vs :85-98 (encode_path)`)
  - **[obs]** `total-chunks-underflow-internal` — `let total_chunks_wire = (total_chunks - 1) & 0x1F;` underflows if total_chunks == 0 (panic in debug, wrap to 255 then &0x1F=31 in release). The only production constructor of Chunked headers is split (`crates/mk-codec/src/string_layer/header.rs:88 (in to_5bit_symbols); sole production constructor crates/mk-codec/src/string_layer/chunk.rs:73`)
- **Status:** open (backlog index; individual items dispositioned in the report). 2 of 4 resolved (`stub-formula-divergence` + `from-md1-test-tautology`, mk-cli v0.8.0); the two `[obs]` items remain.
- **Tier:** audit-backlog.

### `stub-formula-divergence` + `from-md1-test-tautology` — mk1 policy_id_stub aligned to WalletPolicyId (audit I1+I2) (RESOLVED mk-cli v0.8.0)

- **Surfaced:** 2026-06-10 audit (above). **Resolved:** 2026-06-10, mk-cli v0.8.0 (MINOR). Plan + reviews: `design/PLAN_stub_formula_walletpolicyid.md`, `design/agent-reports/stub-formula-divergence-architect-consult.md`, `stub-formula-plan-r0-round{1,2}-review.md`.
- **What:** `derive_stub_from_md1` (`crates/mk-cli/src/cmd/mod.rs`) computed the 4-byte stub as `SHA-256(md_codec::encode_payload(descriptor))[..4]` — the md1 **bytecode** hash (`Md1EncodingId[..4]`, encoding-*sensitive*). The toolkit (`synthesize.rs`, 6 sites) computes `compute_wallet_policy_id(descriptor).as_bytes()[..4]` — the md v0.13 §5.3 **WalletPolicyId** (canonical-*expanded*, encoding-*stable*). Two formulas → a stub minted by `mk --from-md1` did not match a toolkit-emitted card and would not survive a re-encode of the same logical wallet. The companion test `from_md1_derivation` was tautological (recomputed the impl's own bytecode chain, so it could never catch the divergence).
- **CORRECTED rationale (recon overturned the audit's stated cause):** the audit implied "mk SPEC says use WalletPolicyId, mk-cli violates it." The opposite is true on the page — mk SPEC §3.3/§5 + closure Q-2 (locked 2026-04-29) **mandated the bytecode hash**, and mk-cli conformed. The real cause is that the mk SPEC is **stale**: `compute_wallet_policy_id` shipped in md-codec v0.13 (`d8ceb90`) *after* the Q-2 closure, so the SPEC could only cite the one md identity primitive that existed then. WalletPolicyId is canonical because a card-linking stub MUST be stable under re-encoding (pinned by md-codec `walletpolicyid_stable_across_origin_elision`/`_use_site_elision`), which the bytecode hash fails by construction — not because any spec said so. The toolkit was already correct and changed nothing.
- **Fix:** SPEC §3.3 (:186), §5 step 1 (:312), §9 Q-2 (:385) rewritten to WalletPolicyId (Q-2 annotated *superseded*, closure history preserved); BIP draft `bip/bip-mnemonic-key.mediawiki` rewritten in lockstep (glossary, naming-note section, Policy ID stubs, recovery flow); `derive_stub_from_md1` → `compute_wallet_policy_id` (2 callers unchanged); 4 phantom `§3.5.1` doc-cites repointed → §3.3; `from_md1_derivation` flipped to a frozen `EXPECTED_STUB = [0x3d,0x19,0x0a,0xf3]` literal (computed once out-of-band; test body must NOT call `compute_wallet_policy_id`). **Severity LOW:** unreachable from the shipped bundle path (toolkit mints via `KeyCard::new` + `self_check_bundle` both using the WalletPolicyId formula); bites only a user manually running `mk verify/encode --from-md1` against a toolkit md1. **No md-codec pin bump** (`compute_wallet_policy_id` present + byte-stable at the pinned `md-codec-v0.34.0`). **No `mk-codec` / cross-repo code lockstep** — internal to mnemonic-key. Toolkit cross-repo note filed (toolkit already correct).
- **Tier:** resolved.

### `<short-id>` — <one-line title>

- **Surfaced:** <Phase X.Y review of commit SHA>, or <inline TODO at file:line>, or <design discussion 2026-MM-DD>
- **Where:** <file:line> or <design — section name>
- **What:** 1–3 sentences describing the gap or improvement
- **Why deferred:** the reason it didn't ship in the original commit
- **Status:** `open` | `resolved <COMMIT>` | `wont-fix — <one-line reason>`
- **Tier:** `v0.1-blocker` | `v0.1-nice-to-have` | `v0.2` | `pre-bip-submission` | `cross-repo` | `v1+` | `external`
```

The `<short-id>` is a stable handle (e.g., `chunk-set-id-rename`, `nums-structural-audit`). Reference this id from commit messages when closing: `closes FOLLOWUPS.md chunk-set-id-rename`.

## Conventions for adding items

**During a review subagent run:** the reviewer should append to this file (one entry per minor item) and reference it in their report. Reviewers in parallel batches must not write to this file simultaneously — the controller appends afterwards from the consolidated reports.

**During an implementer subagent run:** if the implementer notices a side concern they explicitly chose not to fix in their commit, they append an entry here in the same commit.

**During controller (main-thread) work:** when wrapping a task, the controller verifies all minor items from that task's reviews are either resolved or recorded here.

## Tiers

- **`v0.1-blocker`**: must fix before tagging `mk-codec-v0.1.0`. Failing to fix = ship blocked.
- **`v0.1-nice-to-have`**: should fix before v0.1 if time permits, but won't block release. Document the deferral in v0.1's CHANGELOG/README.
- **`v0.2-nice-to-have`**: should land before v0.2 release if time permits but won't block; document deferral in v0.2's CHANGELOG.
- **`v0.2`**: explicitly deferred to v0.2.
- **`pre-bip-submission`**: not blocking v0.1 release, but MUST be resolved before formal BIP submission per D-11. Examples: NUMS structural audit, HRP collision check.
- **`cross-repo`**: depends on action in `descriptor-mnemonic` repo.
- **`v1+`**: deferred indefinitely.
- **`external`**: depends on work outside both repos.

---

## Open items

### `toolkit-mk-codec-0-5-determinism-note` — `mnemonic-toolkit`'s "mk1 re-encode is NOT string-deterministic" note goes false when it bumps to mk-codec 0.5.0 (companion)

- **Surfaced:** 2026-08-14, alongside D-16. `mk_codec::encode` now derives `chunk_set_id` from the payload, so re-encoding a card reproduces its strings byte for byte. `mnemonic-toolkit/crates/mnemonic-toolkit/src/word_card_adapter.rs` carries a module-doc section headed *"mk1 re-encode is NOT string-deterministic"* stating a re-emitted `mk1` string "is NEVER byte-identical to the original", plus matching inline comments (`:51`, `:152`, `:241`) contrasting mk1 against md1's hash-derived id.
- **Not yet actionable, deliberately.** The toolkit depends on the *published* `mk-codec = "0.4.1"` (`crates/mnemonic-toolkit/Cargo.toml:33`), not a path dependency, so those statements remain TRUE for the code it actually compiles. Editing them now would plant a claim that is false for the pinned codec. Established 2026-08-14 by reading the manifest — after the edit had been made and was then reverted.
- **Fix (when the toolkit bumps to mk-codec ≥ 0.5.0):** rewrite that module-doc section — mk1 becomes string-deterministic like md1 — and consider tightening its round-trip assertions from "compare the recovered payload" to additionally asserting the literal string, which the note says md1 already permits. No behavior change is required either way: the toolkit pins ids explicitly via `derive_mk1_chunk_set_id_for_slot` + `encode_with_chunk_set_id`, so its own output is unaffected by the default.
- **Status:** OPEN, blocked on a toolkit dependency bump. **Tier:** `cross-repo` (producer side: mk-codec 0.5.0, landed; consumer side: toolkit bump + doc rewrite). **Companion:** to be mirrored into `mnemonic-toolkit/design/FOLLOWUPS.md`.

### `sibling-gui-schema-v5-default-value-emission` — `mk gui-schema` emits version-1 JSON (no `default_value`), so mnemonic-gui cannot two-side its `mk` defaults drift gate (companion)

- **Surfaced:** 2026-07-11, mnemonic-gui FOLLOWUP-burndown batch (S2 / constellation-eval #6 extension). mnemonic-gui's `tests/schema_mirror_defaults_drift.rs` gates the toolkit (`mnemonic`, whose `gui-schema` is version 5 — per-flag `default_value` populated) two-sidedly (`default_value` + `choices`), but `mk gui-schema` is still **version 1**: it OMITS the `default_value` key on every flag (R0-verified live at pinned `mk-cli-v0.11.0` — 0 flags carry it). The GUI batch could therefore only extend the gate to `mk` **CHOICES-only** (3 `mk` dropdown flags) plus a SELF-ARMING one-sided guard ("IF the JSON ever carries a non-null `default_value` it must equal the hand mirror"), vacuously green until `mk` emits v5. A true two-sided `mk` defaults gate is infeasible until then.
- **Fix (if pursued):** bump `mk-cli`'s `gui-schema` emitter to v5 parity with the toolkit (populate each flag's `default_value` from its clap-derive default), release, then mnemonic-gui bumps its `pinned-upstream.toml` `mk` pin + re-points the S2 one-sided guard to a full two-sided gate. Needs an `mk-cli` release + a GUI pin bump — a future cross-repo cycle. (The 7 mirror-default backfills that ride this cross-repo item are all `md`/`ms` flags; `mk`'s dropdown defaults, if any, are already correctly mirrored — so `mk`'s value here is future-proofing the gate, 0 day-one backfill delta.)
- **Status:** OPEN. **Tier:** `cross-repo` (producer side: `mk gui-schema` emitter; consumer side: mnemonic-gui re-points the gate once the pin bumps). **Companion:** `mnemonic-gui/FOLLOWUPS.md` (primary) + `descriptor-mnemonic` + `mnemonic-secret` `design/FOLLOWUPS.md` `sibling-gui-schema-v5-default-value-emission`.

### `bsd-process-hardening-parity-procctl-rlimit-core` — `mk`'s `set_non_dumpable()` was a silent no-op on the BSDs (companion)

- **Surfaced:** 2026-06-23, the constellation-wide musl/BSD secret-hygiene recon (toolkit `design/SPEC_bsd_hygiene_and_freebsd_gate.md`, Cycle A). `mk`'s `set_non_dumpable()` in `crates/mk-cli/src/process_hardening.rs` was fenced `#[cfg(target_os = "linux")]` and a silent no-op on FreeBSD/OpenBSD/NetBSD — the anti-core-dump + anti-ptrace-introspection protection did not run, so an `mk` process on a BSD could be ptrace/ktrace-introspected and could drop a core file a secret (passed inline on argv/heap) spills into.
- **Status:** ✓ **RESOLVED (`mk-cli` 0.11.1, 2026-06-23).** Added a BYTE-IDENTICAL (across all four CLI crates) BSD cfg arm: `#[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]` doing (i) FreeBSD-only `procctl(P_PID, 0, PROC_TRACE_CTL, PROC_TRACE_CTL_DISABLE)` and (ii) all-three-BSD `setrlimit(RLIMIT_CORE, {0, 0})`. Best-effort. macOS/Windows remain a documented no-op. No `libc` bump. `mk-codec` NO-BUMP. No CLI flag / subcommand / output-shape change. Linux behavior unchanged.
- **Tier:** `cross-repo`. **Companion:** `mnemonic-toolkit` (primary spec author) + `descriptor-mnemonic` + `mnemonic-secret` `design/FOLLOWUPS.md` `bsd-process-hardening-parity-procctl-rlimit-core`.

### `freebsd-compile-gate-ci` — no CI leg compile-checked `mk`'s FreeBSD build / BSD hardening arm (companion)

- **Surfaced:** 2026-06-23, the BSD recon (Cycle C). Nothing in `mk`'s CI caught a Linux-only syscall/cfg/crate breaking the `cargo install`-on-FreeBSD path or the new BSD hardening arm.
- **Status:** ✓ **RESOLVED (NO-BUMP CI infra, 2026-06-23).** Added a `freebsd-compile-gate` job to `.github/workflows/ci.yml` running WHOLE-CRATE `cargo check --target x86_64-unknown-freebsd -p mk-cli` (NEVER `--lib` — `mk-cli` is bin-only with no `src/lib.rs`; `process_hardening` lives in the bin target, so `--lib` would be silent false-green). `x86_64-unknown-freebsd` is Tier 2 with Host Tools; bare `rustup target add` validated locally (the cross-rs fallback was not needed).
- **Tier:** `cross-repo` / `infra`. **Companion:** `mnemonic-toolkit` (toolkit-primary, `--lib`-correct) + `descriptor-mnemonic` + `mnemonic-secret` `design/FOLLOWUPS.md` `freebsd-compile-gate-ci`.

### `reproducible-builds` — bit-for-bit reproducible `mk` musl release binaries (companion; toolkit-led `reproducible-builds-musl` cycle)

- **Surfaced:** 2026-06-24, the constellation **`reproducible-builds-musl`** cycle (toolkit-led — `mnemonic-toolkit`), P3b (`mk` leg). The pre-cycle `musl-binaries.yml` shipped *static + checksummed* `mk` binaries (the `SHA256SUMS.<arch>` proves integrity) but NOT *bit-for-bit reproducible* ones: the default release build leaks the absolute build path into `.rodata` (`file!()`/panic-`Location` literals — also a `$HOME` PRIVACY leak) and the libsecp256k1 `cc` objects bake in `__DATE__`/`__TIME__` + host paths, so two builds from byte-identical source at different paths/times did NOT byte-match. The published hash was therefore an integrity statement, not a *provenance* one.
- **Where:** `.github/workflows/musl-binaries.yml` (the `musl-binaries` matrix job); `Cargo.lock` (committed); the only C dep is the vendored libsecp256k1 in `secp256k1-sys` (via `secp256k1` 0.29.1).
- **Status:** ✓ **RESOLVED 2026-06-24 — the toolkit-led `reproducible-builds-musl` cycle, P3b (`mk` leg).** The published `mk` musl binaries are now bit-for-bit reproducible:
  - **The `--remap-path-prefix` remap is ADDED**, in the **re-homed** release build (`.github/workflows/musl-binaries.yml` `musl-binaries`), not as a committed `.cargo/config.toml` value (a committed config value passes to rustc verbatim with no `$PWD` expansion → no-op + false assurance). The x86_64 leg runs `cargo build … --remap-path-prefix=/build/src=/build` inside the hermetic container at the fixed `/build/src`; the aarch64 (`cross`) leg uses `--remap-path-prefix=/project=/build` for cross's fixed internal mount. CFLAGS `-ffile-prefix-map` + `SOURCE_DATE_EPOCH` close the secp256k1-sys `cc`-under-musl leaks.
  - **The hermetic build is DELIVERED** via the **digest-pinned container** (`rust:1.85.0@sha256:0ff31c…` + musl-tools, homed in the toolkit's `Dockerfile.repro`, consumed BY BUILT-DIGEST) for x86_64 and the **digest-pinned `cross` image** (`Cross.toml`, `ghcr.io/cross-rs/aarch64-unknown-linux-musl@sha256:702154f5…`) for aarch64. **`rust-toolchain.toml`** (`channel = "1.85.0"`) was ADDED at the repo root so the committed compiler pin matches the `rust:1.85.0` Dockerfile base (md/ms already carried theirs; `mk` did not).
  - **Committed `vendor/`** (the full crates.io dep graph, INERT — no committed `[source]` block) makes the build `--locked --offline`. `mk` is **fork-free** (its transitive `md-codec` dep is the crates.io release, not a git fork; `miniscript` via crates.io `13.0.0`) → the **TWO-block** `--config` `[source]` activation (crates-io + vendored-sources; no git-fork stanza). Offline two-block resolution from committed `vendor/` empirically verified (EXIT 0; dropping `vendored-sources.directory` REDs EXIT 101 — the directory block is load-bearing).
  - **CI proves it** via the toolkit's reusable `reproducible-musl-build.yml` (the `repro` caller job; two-distinct-path double-build + cc-validate + gzip-residue), pinned at toolkit `6e37b18e`, runnable WITHOUT a release tag via `musl-binaries.yml`'s `workflow_dispatch` (on a bare dispatch `github.ref` is the branch, so the tag-gated `musl-binaries` build + release-upload job is skipped — only the gate runs). Per-binary verify recipe authored at `docs/verify-reproducibility.md`. All NO-BUMP (CI-infra + docs); the re-home fires on the next `mk-cli-v*` tag.
- **Tier:** ✓ RESOLVED — `cross-repo` / `infra`. **Companion:** the toolkit `reproducible-builds-musl` cycle (`mnemonic-toolkit` — `cycle-prep-recon-reproducible-builds-musl.md` + the P3 recon `design/P3_RECON_codec_repos.md`; the centralized recipe `reproducible-musl-build.yml` + `Dockerfile.repro` + `ci/repro/*.sh` + `Cross.toml`, pinned at toolkit `6e37b18e`) + `descriptor-mnemonic` (the FIRST codec re-home) + `mnemonic-secret` `design/FOLLOWUPS.md` `reproducible-builds`.

### `mstar-prepolicy-key-backup` — no policy-independent (pre-wallet) public-key backup; mk1 always binds to a policy/template

- **Surfaced:** 2026-06-20, design discussion (SeedHammer template-engraving thread — "what about generating + backing up keys *before* using them in a wallet?").
- **Where:** `crates/mk-codec/src/key_card.rs:24-58` (`KeyCard.policy_id_stubs: Vec<[u8;4]>`, mandatory); `crates/mk-codec/src/bytecode/encode.rs:24` (empty stub set → `Error::InvalidPolicyIdStubCount`). Cross: `descriptor-mnemonic crates/md-codec/src/identity.rs:50-53` (`WalletDescriptorTemplateId` is key-independent + origin-path-invariant).
- **What:** Every mk1 MUST declare ≥1 specific md1 policy/template (`policy_id_stubs` non-empty; the encoder rejects empty). So the constellation has NO artifact for backing up a *public* key (xpub) *before* any wallet/template exists. The canonical pre-wallet backup is the SEED (`ms1`/words/SeedQR) — but that is the SECRET, not a shareable/pre-engravable public key card. The "generate-and-back-up-keys-before-use" (key-first) workflow therefore cannot pre-mint or pre-engrave an mk1: the user must back up the seed, or wait until at least a template SHAPE is agreed (then bind to the key-independent `WalletDescriptorTemplateId` — partial: records the shape, not the `@N` slot→key assignment).
- **Decision needed (design item, NO code pending):** either (a) support an "unbound" / template-agnostic public-key card and define how it later binds to a policy/template + how `verify-bundle` treats a card with no policy binding (an unbound card loses the integrity link that makes a bundle verify as a coherent whole), or (b) declare seed-only (`ms1`) the canonical pre-wallet backup and document it (wont-fix). Re-evaluate alongside the constellation template-engraving work.
- **Analysis (2026-06-20) — leaning option (a), scoped (no decision finalized, no code):** support an UNBOUND mk1 (xpub + *optional* origin, NO `policy_id_stubs`) as an explicitly distinct artifact that makes **no coherence claim**. Load-bearing findings for the eventual brainstorm:
  1. **Watch-only recovery of a standard k-of-n needs only {xpubs, k, script-type, use-site}** — NOT cosigner order (`sortedmulti` sorts the *derived* pubkeys per BIP-67 → order-invariant) and NOT origins (origins are PSBT-signing metadata; they don't affect address derivation). So a 2-of-3 `wsh(sortedmulti)` is reconstructible from just the 3 xpubs + "2-of-3" by enumerating the ~3 standard script types and matching a known address — **no permutation search**.
  2. **The origin is the high-value optional field to KEEP** (drop the stub, keep the origin): the BIP-48 path pins script type (`…/48'/c'/a'/1'`=sh(wsh), `…/2'`=wsh), collapsing the script-type enumeration to one.
  3. **The permutation search** (`mnemonic-engrave` `seedhammer-template-engrave-key-search-time-estimate`) **only bites unsorted `multi` or distinct-origin/override slots** — a corner case, not the main recovery path.
  4. **Primary value = durably backing up the OTHER cosigners' xpubs** (you cannot re-derive those from your own seed) — standard multisig hygiene; for your OWN key the seed/`ms1` already suffices.
  5. **Design shape:** no stub ⇒ `verify-bundle`/`self_check_bundle` treat the card as **unbound** (it cannot prove "these N cards are THIS wallet"); recovery must terminate in an address / `--expect-wallet-id` confirmation.
  6. **Costs:** loses the integrity binding (verifiable coherence traded for pre-policy flexibility); partly redundant with the seed for one's own key; functionally "a steel-durable, BCH-error-corrected xpub + origin" (≈ a BIP-380 key expression).
  7. **It does NOT enable otherwise-impossible recovery** (standard `sortedmulti` is recoverable from xpubs+k regardless) — it is a durable *medium* for the recovery inputs, not an enabler.
- **Why deferred:** open design question (not a defect); held pending the m* multisig-template upgrade per the user's standing hold.
- **Status:** `open`.
- **Tier:** `cross-repo`.
- **Companion (LOCKSTEP — settling one MUST settle the other):** `mnemonic-toolkit/design/FOLLOWUPS.md` `mstar-prepolicy-key-backup`. This entry and its toolkit companion are a BOUND PAIR: any resolution (support an unbound card / wont-fix seed-only / re-scope) MUST update BOTH entries in the same change — neither may be closed alone. Related: `mnemonic-engrave/design/FOLLOWUPS.md` `seedhammer-template-engrave-key-search-time-estimate` (the key-first flow is its primary trigger) + `constellation-template-only-engraving`.

### `display-grouping-render-strip-v1` — ✓ RESOLVED (full cycle shipped; reconciled 2026-06-22) — standardized mstring display-grouping (`mk` CLI flags + intake strip; companion)

- **Surfaced:** 2026-06-15, the cross-constellation **mstring display-grouping** cycle (P3 = mnemonic-key). User-requested standardization of `ms1`/`mk1`/`md1` display output across all four CLIs (`mnemonic`/`md`/`ms`/`mk`).
- **Where:** `crates/mk-cli/src/format.rs` (NEW — `render_grouped`, `strip_display_separators`, `is_display_separator`, `parse_separator`; kept LOCAL to mk-cli, bin-only); `cmd/encode.rs` (`--group-size`/`--separator`); `cmd/mod.rs::read_mk1_strings` (interior strip — covers all 6 mk1-intake subcommands: decode/inspect/verify/repair/derive/address); canonical vectors `design/display-grouping-vectors.tsv` (+ `.sha256`, CI-pinned in the fmt job).
- **What (SHIPPED this cycle, mk-cli 0.9.0):** `mk encode` gains `--group-size <u16>` (default 5, `0`=unbroken) + `--separator <space|hyphen|comma>` (default space); text output is now **space/5 print-once** — a CORRECTIVE default-output change (`mk encode` emitted UNBROKEN before, diverging from the other CLIs). `--json` stays UNBROKEN. `read_mk1_strings` now strips ALL whitespace + `-` + `,` (was edge-only `.trim()`), so a grouped or unbroken card both re-ingest on every mk1-intake subcommand + the `-`→stdin path. **mk-codec UNCHANGED** (fns mk-cli-local; mk-codec tolerates no separators). Drift control = copy-with-checksum conformance vectors (canonical TSV authored in the toolkit; byte-identical copy + `.sha256` here; CI `sha256sum -c` + a bin-crate driver test).
- **Why deferred / residual:** P4 (toolkit) pin-bumps + collapses `format.rs` + regenerates goldens + updates both manuals; P5 (`mnemonic-gui`) `schema_mirror` flags + separator keyword dropdown. The canonical-vector checksum is a lagging drift gate; the leading control is the paired-PR discipline.
- **Status:** ✓ RESOLVED (reconciled 2026-06-22) — full cross-repo cycle shipped: P3 mk-cli 0.9.0 (this repo), P1 md-cli 0.7.0, P2 ms-cli 0.8.0, P4 toolkit v0.56.0, P5 mnemonic-gui v0.41.0. Verified at reconcile: `mk encode --group-size/--separator` live; vectors + `.sha256` present. Canonical record: `../../mnemonic-toolkit/design/FOLLOWUPS.md` (`display-grouping-render-strip-v1`).
- **Tier:** `cross-repo`.
- **Companion:** mnemonic-toolkit `design/SPEC_mstring_display_grouping.md` (canonical spec) + `design/FOLLOWUPS.md` (`display-grouping-render-strip-v1`, filed in P4) + descriptor-mnemonic + mnemonic-secret `design/FOLLOWUPS.md` (`display-grouping-render-strip-v1`, P1/P2).

### `mk-cli-repair-flag` — `mk repair` subcommand mirroring toolkit's `mnemonic repair`

- **Surfaced:** 2026-05-17, mnemonic-toolkit v0.22.0 brainstorm.
- **Where:** `crates/mk-cli/src/cmd/` (NEW subcommand).
- **What:** Add `mk repair <mk1>...` for mk1 BCH error-correction (regular + long codes via the already-public `mk_codec::string_layer::bch_decode::{decode_regular_errors, decode_long_errors}` per v0.3.1). Mirrors toolkit's `mnemonic repair --mk1`. Note: `mk_codec::decode` already does internal BCH correction within the same t=4 capacity, so this is primarily a UX-parity feature (explicit-fix-with-report vs silent-fix-during-decode) rather than a new capability.
- **Status:** `resolved 0ecbf1a` — `mk-cli-v0.4.0` (mnemonic-toolkit v0.22.x follow-ups cycle Phase A.3'; plan `/home/bcg/.claude/plans/nifty-wiggling-gosling.md` §2.A.2). New `cmd/repair.rs` consumes `mk_codec::string_layer::decode_string` (already-public BCH primitive) and surfaces `DecodedString` (`code`/`corrections_applied`/`corrected_positions`/`corrected_char_at`); exit 5 = REPAIR_APPLIED per D26; JSON envelope byte-matches toolkit's `RepairJson` schema per D27. 7 new integration cells in `tests/cli_repair.rs`. D25 handler-signature cascade (6 handlers `Result<()> → Result<u8>`) ships in the same release commit.
- **Resolution:** `0ecbf1a` (this repo) + cross-repo lockstep on `bg002h/mnemonic-toolkit` master (install.sh pin bump + manual chapter `## mk repair` section + companion FOLLOWUP closure) shipping concurrently.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-toolkit` FOLLOWUPS.md `mk-cli-repair-flag` — closed lockstep at the cross-repo `docs(release)` commit on toolkit master.

### `from-md1-derivation-wire-version-skew` — `mk-cli/tests/round_trip.rs:45` `from_md1_derivation` fails `WireVersionMismatch { got: 0 }`

- **Filed:** 2026-05-17 (during v0.22.x follow-ups cycle Phase A.1' execution)
- **Status:** RESOLVED in mk-cli-v0.4.1 — md-codec dep bumped 0.32.1→0.34.0 and fixture refreshed against md-codec v0.34.0's canonical `pkh_basic.phrase.txt` (`md1yqpqqxzq2qwfv8urt848e`, replacing pre-v0.18-wire-format `md1qqpqqxyepwspuepy268e`). `from_md1_derivation` now passes.
- **Severity:** low (pre-existing failure on pristine main; predates this cycle)
- **Body:** the `from_md1_derivation` integration cell at `crates/mk-cli/tests/round_trip.rs:45` decodes an md1 fixture and fails with `WireVersionMismatch { got: 0 }` — the md1 fixture appears to have been authored against a pre-`mk-codec`/`md-codec` wire-version bump and was not refreshed. Cell has been failing silently for at least 3 release cycles. Fix: regenerate the md1 fixture against current md-codec, OR `#[ignore]`-gate the cell until a fresh fixture is available, OR delete the cell if md1-derivation is not in mk-cli's scope.
- **Surfaced by:** mnemonic-toolkit v0.22.x follow-ups cycle Phase A.1' (D25 handler-signature cascade). The cell was untouched by D25 but its failure became visible against the freshly-bumped Cargo.toml.

### `mk-cli-vector-corpus-inlined` — `crates/mk-cli/src/cmd/v0.1.json` is a working copy of `crates/mk-codec/tests/vectors/v0.1.json`

- **Surfaced:** v0.3 `mnemonic-gui` cycle crates.io publish round 2026-05-15. `mk-cli` `cargo publish --dry-run` rejected `include_str!("../../../mk-codec/tests/vectors/v0.1.json")` per cargo's "published package is self-contained" rule (the .crate tarball cannot reach outside the package dir).
- **Where:** `crates/mk-cli/src/cmd/vectors.rs:15` (`include_str!("v0.1.json")`) + `crates/mk-cli/src/cmd/v0.1.json` (34KB; working copy of the canonical mk-codec test fixture).
- **What:** The SHA-pinned v0.1 `mk-codec` test-vector corpus (SPEC §3.5.5) is duplicated in two locations: the canonical at `crates/mk-codec/tests/vectors/v0.1.json` (mk-codec's own tests consume this) and the working copy at `crates/mk-cli/src/cmd/v0.1.json` (mk-cli's `mk vectors` subcommand `include_str!`s this at compile time). When the corpus changes, both copies must sync manually; no compile-time or CI gate verifies byte-equality today.
- **Why deferred:** Quick crates.io publishability fix at commit `135651f`. Proper fixes require coordinated re-publish: (a) factor into `mk-codec` public API as a feature-gated `pub const VECTORS_V0_1_JSON: &str = ...`, then bump mk-codec + mk-cli; OR (b) feature-gate the `mk vectors` subcommand off by default in published `mk-cli` builds (matching the existing `gen-vectors` feature precedent in `mk-codec/Cargo.toml` for `gen_mk_vectors`).
- **Status:** `resolved 33f2ca2` — mk-codec 0.3.0 promotes the corpus to `pub mod test_vectors::V0_1_JSON` (canonical file moved to `crates/mk-codec/src/test_vectors/v0.1.json`); mk-cli 0.3.2 re-imports the const, dropping the 34KB working copy. Both crates published to crates.io 2026-05-15.
- **Tier:** `v1+`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` entry `md-cli-vectors-manifest-inlined` — same pattern in the md-cli side; resolved at descriptor-mnemonic commit `8a52bed` (md-codec 0.33.0 + md-cli 0.5.1).

### `bip-vector-adoption-v0_8` — cross-repo cycle: BIP-vector adoption v0.8.0 (no-scope companion)

- **Surfaced:** 2026-05-13. Cycle SPEC at `mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md`. Plan at `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`. R1 review at `mnemonic-toolkit/design/agent-reports/v0_8_0-phase-0-spec-plan-r1.md`.
- **Where:** No `mk-codec` / `mk-cli` source change. mk-codec's v0.7.1 audit matrix at `design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md` covered the BIP coverage relevant to this crate (BIP-32 xpub derivation delegated to `bitcoin v0.32`); no new gap was surfaced for this cycle by the cross-repo BIP-vector survey at `mnemonic-toolkit/design/agent-reports/v0_8_0-cross-repo-bip-vector-survey.md`.
- **What:** This entry exists for cross-repo audit symmetry per SPEC §5 ("`mnemonic-key` is OUT-OF-SCOPE for v0.8.0 — no new gap; mk-codec's v0.7.1 matrix carries the only relevant coverage and continues to delegate xpub-format / BIP-32 derivation to `bitcoin v0.32`. The `bip-vector-adoption-v0_8` entry in mnemonic-key reads: *'no scope for this cycle; included for cross-repo audit symmetry.'*"). Closes when the cycle's audit-matrix successor doc lands at `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` (Phase 4; even with no coverage delta, the no-op v0.8.0 matrix file replicates the v0.7.1 content with a SUPERSEDED header).
- **Status:** `resolved 6d43115` — PR #10 merged (docs-only no-scope companion; mk-codec carried no source change so no tag). Companion sibling-repo tags: ms-codec-v0.1.2 (mnemonic-secret 527c9c7), md-codec-v0.32.1 (descriptor-mnemonic ef00e07), mnemonic-toolkit-v0.9.1 (f036737).
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md`, `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md` — same `bip-vector-adoption-v0_8` short-id in each.

### `md-mk-private-key-surface-watch` — reopen md/mk Cycle A participation if this repo grows a private-key surface

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 0 R3 architect-review I-R3-4 fold (drop md/mk symmetry-stubs); opened as a standalone tracker entry per Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-md-mk` class. Primary tracker entry in `mnemonic-toolkit/design/FOLLOWUPS.md`.
- **Where:** This repo (`mk-codec` + `mk-cli`). Currently holds xpub / wallet-key material only — no private-key buffer.
- **What:** v0.9.0 Cycle A's secret-memory hygiene work (toolkit + ms repos; tags `mnemonic-toolkit-v0.9.2`, `ms-codec-v0.1.3`, `ms-cli-v0.2.2`, shipped 2026-05-13) dropped the no-scope-symmetry matrix stubs originally planned for md/mk repos because they have no secret material to audit. If this repo later gains a private-key surface (e.g., a future mk-codec xprv passthrough), this FOLLOWUP fires and Cycle A's hygiene discipline (Zeroizing + SAFETY anchors + matrix delta) reopens for mk.
- **Why deferred:** No secret material to audit today.
- **Status:** `open` (monitoring)
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md` (primary tracker), `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md` — same `md-mk-private-key-surface-watch` short-id.

### `manual-cli-surface-mirror` — `mk-codec` public-API changes must mirror to the toolkit-side user manual

- **Surfaced:** 2026-05-07, m-format-star user manual v0.1 release in `bg002h/mnemonic-toolkit` (`manual-v0.1.0` tag; toolkit PR #1).
- **Where:** Cross-repo coordination only; no `mk-codec` source change required at filing time. Future public-API additions (new `pub` items, removed re-exports, signature changes, `#[non_exhaustive]` field additions to `KeyCard`) must touch the manual side in lockstep — for v0.2+ this means both `mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md` (flag surface) and the archived Rust API reference at `mnemonic-key/docs/MK_CODEC_RUST_API.md` (with the toolkit-side `44-mk-codec-rust.md` deleted in toolkit PR 2 of the v0.2 mk-cli cycle).
- **What:** v0.1 of the m-format-star user manual mirrored the *Rust API* surface only because `mk-codec` was library-only. With v0.2 the manual mirrors the `mk-cli` flag surface; the Rust API reference is archived in this repo at `docs/MK_CODEC_RUST_API.md`. Both surfaces are bound by the same lockstep-mirror invariant. The manual's `tests/lint.sh flag-coverage` step grep-gates `mk` flags from v0.2 onward (toolkit PR 2 commit 2a). **Companion:** primary entry `manual-cli-surface-mirror` in `mnemonic-toolkit/design/FOLLOWUPS.md`; sibling companions in `descriptor-mnemonic/design/FOLLOWUPS.md` and `mnemonic-secret/design/FOLLOWUPS.md`.
- **Why filed:** the manual is a separate artifact (its own `manual-v*` versioning); without an explicit mirror invariant, sibling-side API changes would silently drift the manual.
- **Status:** `open` (mirror invariant active for the lifetime of `mnemonic-toolkit/docs/manual/`)
- **Tier:** `cross-repo`

### `mk-cli-v0_2-toolkit-docs-mirror` — ✓ RESOLVED (toolkit PR 2 landed; reconciled 2026-06-22) — toolkit-side docs mirror for mk-cli v0.2 (companion)

- **Surfaced:** 2026-05-08, mk-cli v0.2 cycle (this repo's branch `mk-cli/v0_2`, plan `concurrent-cooking-scone`).
- **Where:** `mnemonic-toolkit` repo branch `mk-cli/docs-v0_2`. Toolkit PR 2 lands the manual chapter `docs/manual/src/40-cli-reference/44-mk-cli.md` (~600 lines), deletes `44-mk-codec-rust.md` (archived in this repo at `docs/MK_CODEC_RUST_API.md` in commit `1c74c70`), extends `tests/lint.sh` + `Makefile` for the 4-CLI shape, and rebuilds the manual / quickstart / ultraquickstart PDFs. Toolkit PR 2 → manual-v0.1.7 + quickstart-v0.1.4 + ultraquickstart-v0.1.2 patch-tag releases.
- **What:** This repo's PR 1 (commit `77bdb2f` adds the binary; commit `1c74c70` archives the Rust API doc; commit on this entry pins the v0.2 manual-mirror language in `CLAUDE.md`). The companion toolkit PR 2 lands the user-facing chapter and the lint-gate update so the four-CLI parity invariant holds end-to-end. **Companion:** primary entry `mk-cli` resolution in `mnemonic-toolkit/design/FOLLOWUPS.md` (moves from "Open" → "Resolved/Closed" with citation `Resolved by mk-cli-v0.2.0`).
- **Why filed:** Same lockstep-pattern as ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility. Filing here makes it discoverable from this repo's tracker; closes when toolkit PR 2 merges.
- **Status:** ✓ RESOLVED (reconciled 2026-06-22) — toolkit PR 2 landed: the manual chapter `docs/manual/src/40-cli-reference/44-mk-cli.md` exists in the toolkit (18 KB), and mk-cli is at v0.10.1 (far past v0.2.0). The 4-CLI manual/lint parity invariant holds end-to-end. Status had lagged since the chapter shipped.
- **Tier:** `cross-repo`

### `ms1-v01-payload-bracket-overflow-prefix-byte-incompatibility` — ms1 v0.1 wire-format plan needs revision (BIP-93 codex32 length-bracket conflict with locked `0x00` prefix byte)

- **Surfaced:** 2026-05-03 pre-SPEC spike in `mnemonic-secret` repo (in conversation; before ms1's SPEC drafted). Companion: primary entry of same id in `mnemonic-secret/design/FOLLOWUPS.md`; mirror in `descriptor-mnemonic/design/FOLLOWUPS.md`.
- **Where:** Cross-repo coordination only; no md1/mk1 wire-format change required. Affects: ms1 v0.1 SPEC (not yet drafted) and downstream `mnemonic-toolkit` (when it lands) — both will need to know which payload kinds ms1 v0.1 actually emits (currently locked as {seed, entr, xprv}, likely to narrow).
- **What:** ms1 v0.1's `0x00` reserved-prefix byte (designed to make the v0.2 share-encoding migration non-breaking for v0.1 strings) pushes 64-B BIP-32 master seeds to 65-B payloads — one byte past BIP-93 codex32's long-code max (rust-codex32 v0.1.0 rejects with `InvalidLength(128)`). `xprv` (78 B) was never inside any BIP-93 bracket. Likely remediation: narrow ms1 v0.1 to `entr`-only payloads; defer `seed`/`xprv` to v0.2+ with their own framing. Awaiting user direction in the ms1 session.
- **Why deferred:** ms1-internal SPEC decision; no mk1 source change. Logged here so future sessions in this repo don't re-litigate the four-format-star payload assumptions when toolkit work begins.
- **Status:** `resolved 2026-05-03 — ms1 v0.1 shipped with Option A remediation: v0.1 narrowed to entr-only; seed/xprv deferred to v0.2+ with own framing. ms-codec v0.1.0 release commit ab374ed in mnemonic-secret; primary FOLLOWUPS entry there records full mechanics. mk1 source unchanged — this entry was a coordination flag only. Four-format-star payload assumptions for downstream toolkit work: ms1 emits entr only in v0.1.`
- **Tier:** `cross-repo`

### `mc-codex32-extraction-retired-2026-05-03` — original shared-crate plan retired in favor of ms1 adopting `rust-codex32` directly

- **Surfaced:** 2026-05-03, ms1 plan-mode brainstorm in the `descriptor-mnemonic` repo (plan file: `/home/bcg/.claude/plans/c-ultimately-what-we-quirky-avalanche.md`). Companion: same-id entry in `descriptor-mnemonic/design/FOLLOWUPS.md`.
- **Where:** Cross-repo design / process. Affects `mnemonic-key/CLAUDE.md` (line 38 retirement language already updated 2026-05-03), `descriptor-mnemonic/CLAUDE.md` (mirrored), `descriptor-mnemonic/design/DECISIONS.md` D-13 (still records "fork-now-refactor-later" — historically accurate, no change needed), and the future cross-repo `PATTERNS.md` doc that will replace the shared-crate plan.
- **What:** Closure Q-9 originally specified that md1 and mk1 would extract their shared BIP-93 BCH plumbing into a third sibling crate `mc-codex32` once both formats hit v1.0 with cross-validated conformance vectors. With the addition of a third sibling format ms1 (HRP `ms`, repo `bg002h/mnemonic-secret`) that adopts BIP-93 codex32 *directly* via Andrew Poelstra's `rust-codex32` crate, the calculus changed: md1 and mk1 use HRP-mixed BCH with per-format target residues that are NOT upstreamable to `rust-codex32`'s vanilla BIP-93 implementation, and ms1 doesn't need them either. There is no longer shared code worth extracting — only a shared *pattern* (HRP-mixed BCH with per-format target residue) that is better captured as documentation. md1↔mk1 BCH plumbing stays forked indefinitely; the pattern will be documented in a future cross-repo `PATTERNS.md`.
- **Why deferred:** Decision was locked during ms1 plan-mode r1..r5 review convergence on 2026-05-03; CLAUDE.md updates landed in lockstep. The `PATTERNS.md` doc itself is non-blocking and can be drafted opportunistically when the next BCH-plumbing concern surfaces in either repo.
- **Status:** `wont-do — superseded by ms1 adopting rust-codex32 directly (2026-05-03 cross-repo decision)`. CLAUDE.md retirement language landed same day.
- **Tier:** `cross-repo`

### `chunk-set-id-rename` — rename "wallet identifier" to `chunk_set_id` in md1 (resolved)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (Q-5(d)).
- **Where:** `descriptor-mnemonic` repo — BIP draft `bip/bip-mnemonic-descriptor.mediawiki` line ~188; `md-codec` reference implementation symbols carrying "wallet identifier" naming; mk1's own SPEC §2.5 already uses `chunk_set_id` per closure lock.
- **What:** md1 v0.8.0 shipped with the 20-bit chunked-header random tag named "wallet identifier" — a name that conflicts with `Policy ID` and `Wallet Instance ID` and means neither. Closure design Q-5 locks the rename to `chunk_set_id` across both repos. Wire format unchanged; this is purely a documentation and code-symbol rename.
- **Why deferred:** Lives in the descriptor-mnemonic repo, not this one. mk1's spec already uses the new name.
- **Sequencing requirement:** the rename MUST land in md-codec (likely a docs-and-symbols-only release, e.g. md-codec v0.9.0) **before** mk1's BIP draft is submitted. mk1's BIP cites md1 by field name; mk1 cannot publish referencing a name md1 itself does not use.
- **Status:** `resolved by md-codec-v0.9.0` ([release](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0), merge commit `9eeb9ab` in `bg002h/descriptor-mnemonic`). The rename landed across ~85 sites / ~150 references in md-codec docs + symbols. mk1's BIP-submission gate is cleared. Cross-update pass on the mk1 side: BIP §"Naming and identifiers" updated past-tense; DECISIONS D-15 sequencing-requirement updated past-tense.
- **Tier:** `cross-repo`

### `md-per-N-path-tag-allocation` — md1's per-`@N` path bytecode tag allocation (Q-4) (resolved)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (Q-4).
- **Where:** `descriptor-mnemonic` repo — md1 bytecode tag table; new tag in unallocated `0x36+` range, or backfill `0x24-0x32`.
- **What:** mk1 declares the authority-precedence semantics (mk1's `origin_path` is authoritative; md1's per-`@N` path is descriptive). The wire-format question of which tag byte md1 uses is an md-repo decision. mk1 cannot answer it.
- **Why deferred:** Lived in the descriptor-mnemonic repo's next phase. md1's parallel entry (`md-per-at-N-path-tag-allocation` in `descriptor-mnemonic/design/FOLLOWUPS.md`) was scheduled whenever per-`@N` paths became a planned md release feature.
- **Status:** `resolved by md-codec-v0.10.0` ([release](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.10.0), merge commit `172830a` in `bg002h/descriptor-mnemonic`). md1 allocated `Tag::OriginPaths = 0x36` and reclaimed header bit 3 as the OriginPaths flag; per-`@N` divergent origin paths are now first-class on the policy card. mk1's BIP §"Authority precedence (MK ↔ MD path information)" pins the cross-format precedence semantics; no mk1-side wire-format change was required. mk1 cross-update pass on 2026-04-29 (post md-codec v0.10.0 ship): BIP §"Authority precedence" updated past-tense; SPEC §5.1 updated past-tense; DECISIONS Q-4 / closure-design §Q-4 + §3 item (2) updated past-tense.
- **Tier:** `cross-repo`

### `nums-structural-audit` — structural-relationship audit of `MK_REGULAR_CONST` / `MK_LONG_CONST` (resolved at md1's bar)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (Q-1, captured as pre-BIP-submission audit item (1)).
- **Where:** design / cryptography review.
- **What:** Verify there are no accidental structural relationships between the locked target constants and the BIP 93 BCH polynomial. Required: weight-distribution analysis under the new target, intersection of mk1 codeword space with md1 and codex32 codeword spaces, confirmation that error-correction guarantees (8-character detection, 4-substitution correction) hold under the new constants.
- **Why deferred:** Not a v0.1 implementation gate; gates formal BIP submission. Andrew Poelstra is the natural reviewer per D-11.
- **Status:** `resolved at md1's bar` (2026-04-29 cross-update pass). md1 / md-codec ship with the same NUMS construction (truncate top-N bits of `SHA-256(domain_string)`) and chose to document the construction in the BIP itself with a Python reproducer rather than commission a separate structural audit; `md`'s BIP §"Why new target constants?" is the audit trail. mk1 already meets that bar: BIP §"Why new target constants?" carries the equivalent reproducer for `b"shibbolethnumskey"`; SPEC §2.3 carries the same; and `consts.rs::tests::nums_constants_reproduce_from_domain` reproduces the construction at runtime (`cargo test`-enforced). The original FOLLOWUPS entry called for an external Poelstra structural review — a higher bar than md1 chose. Per the project's "don't adopt a higher bar than md1" principle, the entry is closed at the audit-trail-in-BIP level. If a future reviewer (Poelstra or other) volunteers a structural pass, it can land as a strengthening note in the BIP without re-opening this gate.
- **Tier:** `pre-bip-submission`

### `slip-0173-register-mk-hrp` — file SLIP-0173 PR registering `mk` HRP (resolved)

- **Surfaced:** 2026-04-29 cross-update pass after closing `hrp-mk-collision-check`. md1 filed a parallel PR (#2011 at satoshilabs/slips) registering `md` as a defensive measure; mk1 follows the same pattern.
- **Where:** [satoshilabs/slips](https://github.com/satoshilabs/slips) PR adding one row to `slip-0173.md`. Draft PR text + diff at `design/SLIP_0173_PR_DRAFT.md`.
- **What:** Defensive registration of the `mk` HRP in SLIP-0173 to close off future collision risk from independent Bitcoin-family projects. The registration is a docs-level act in the SatoshiLabs registry; no code change in mk-codec, no wire-format implications, no binding consequence beyond the registry record.
- **Why deferred:** Single user-action item (file the PR under the maintainer's GitHub account). The `hrp-mk-collision-check` audit at `design/AUDIT_hrp_mk_collision.md` cleared the technical gate; this entry tracks the actual PR filing.
- **Status:** `resolved 2026-04-29 — PR filed at https://github.com/satoshilabs/slips/pull/2012`. The requested action (FILE the PR) is complete; merge state is now tracked externally on SatoshiLabs review cadence and is no longer an mk1-side deferral. Parallel to md1's `slip-0173-register-md-hrp` (PR #2011 at the same repo, also still in external-review state). If #2011 merges first, #2012 will need a one-line rebase to insert `mk` after `md` rather than after `Lightning Network`; otherwise the two PRs are mergeable in either order.
- **Tier:** `pre-bip-submission` (closed; awaiting upstream merge tracked separately)

### `hrp-mk-collision-check` — formal HRP `mk` collision verification (resolved)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (D-9 / pre-BIP-submission audit item (2)).
- **Where:** SLIP-0173 (informal segwit-HRP registry); recent bitcoin-dev mailing-list archives; BIPs PR history.
- **What:** Search for any soft `mk` claim before formal SLIP-0173 registration. None expected, but confirmation is the registration gate. Alternatives `mx`, `mkc`, `mpk` documented in D-9 if collision is found.
- **Why deferred:** Not a v0.1 gate; gates formal HRP registration.
- **Status:** `resolved` — see [`design/AUDIT_hrp_mk_collision.md`](AUDIT_hrp_mk_collision.md). SLIP-0173 has no `mk` registration; closest neighbours (`ms` BIP 93, `md` Mnemonic Descriptor, `mm` Miden, `my` Myriad) are at Hamming distance 1 but BIP 173 HRP-mixing prevents cross-HRP false-positive validation (≈ 2⁻⁶⁵ collision probability), and mk1's NUMS-derived target residues are independent from md1's and codex32's. Formal SLIP-0173 registration of `mk` is folded into the BIP-submission workflow.
- **Tier:** `pre-bip-submission`

### `bip-cross-reference-completeness` — BIP draft cross-reference audit (resolved)

- **Surfaced:** 2026-04-29 mk1 closure-design pass (pre-BIP-submission audit item (3)).
- **Where:** `bip/bip-mnemonic-key.mediawiki` — final cross-reference pass before submission.
- **What:** mk1's BIP draft must cross-reference: BIP 93 (codex32 plumbing reuse), BIP 32 (xpub serialization), BIP 380 (origin notation), BIP 388 (wallet policy / Policy ID semantics), and the published md1 BIP (linkage protocol, shared-parser conventions, `chunk_set_id` field). Any post-rename of "wallet identifier" → `chunk_set_id` in md1 (see `chunk-set-id-rename` above) MUST land before mk1's draft is finalized.
- **Why deferred:** Final pre-submission audit step; depends on `chunk-set-id-rename` landing first.
- **Status:** `resolved` — see [`design/AUDIT_bip_cross_reference_completeness.md`](AUDIT_bip_cross_reference_completeness.md). 74 cross-references audited across 8 categories; 9 drifts found (1 blocker, 3 important, 5 minor) and all 9 fixed inline. Notable fixes: removed phantom `Error::FingerprintFlagMismatch` cite (retired in v0.1.0 Phase 4); added `Error::MixedHeaderTypes` to §"Decoder validity rules" (added in v0.1.1 Phase 1); refreshed the stale "rename in flight" claim for `chunk_set_id` (md-codec v0.9.0/v0.9.1 has shipped); fixed BIP 380 attribution in SPEC §3.2; corrected several internal heading-quote mismatches. `chunk-set-id-rename` cross-repo dependency is now noted as resolved-on-md1-side; mk1's BIP draft is internally consistent and parity-correct with md1 v0.9.1.
- **Tier:** `pre-bip-submission`

### `decoder-error-variant-parity` — Error-variant ↔ negative-vector parity

- **Surfaced:** 2026-04-29 mk1 closure-design opus review pass (pre-BIP-submission audit item (4)).
- **Where:** `crates/mk-codec/src/error.rs` (variants), `crates/mk-codec/tests/vectors/v0.1.json` (corpus).
- **What:** Every reject case in SPEC §4 validity rules MUST map to a uniquely-named `Error` variant in the reference crate, and every variant MUST have at least one planned negative test vector. Mirrors md-codec's 30-negative-vectors-one-per-Error-variant conformance contract.
- **Why deferred:** v0.1 implementation will define the Error variants; the *parity gate* (every variant has a vector, no orphaned variants, no variantless reject paths) is checked just before BIP submission and v1.0 release.
- **Status:** `resolved 1e42354 + 59878ca` (v0.1.1 Phase 3 + Phase 3 review fixup). 22 negative vectors N1..N21, N23 cover every `Error` variant reachable from `decode`'s string-input path; `every_error_variant_has_negative_vector` integration test enforces variant coverage. `Error::CardPayloadTooLarge` is documented exempt (encoder-only — no decoder path can trigger it). The Phase 3 fixup commit `59878ca` reshaped N17 to actually trigger `InvalidPathComponent` (LEB128 overflow at 6 × 0x80) — the original 1e42354 form surfaced as `UnexpectedEnd` and left `InvalidPathComponent` exempt. Compile-time exhaustiveness via strum is recorded as `error-variant-exhaustiveness-gate-strum` for v0.2.
- **Tier:** `pre-bip-submission`

### `md-path-dictionary-0x16-gap` — md1 path dictionary missing testnet 0x16 entry (resolved)

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 2 BIP review (commit 4728230).
- **Where:** `descriptor-mnemonic` repo — md1 BIP `bip-mnemonic-descriptor.mediawiki` §"Path dictionary" lines ~339-349. Testnet rows list 0x11, 0x12, 0x13, 0x14, 0x15, 0x17 — **0x16 omitted** (no testnet pair for mainnet 0x06 = `m/48'/1'/0'/1'`, BIP 48 nested-segwit multisig testnet).
- **What:** Mainnet has 0x06 (`m/48'/0'/0'/1'`, BIP 48 nested-segwit multisig) but the testnet companion 0x16 (`m/48'/1'/0'/1'`) is absent from md1's published BIP table. mk1's spec and BIP both claim "exact dictionary mirrors md1's `Tag::SharedPath` table byte-for-byte"; mk1 inherits the gap. mk1 v0.1 BIP §"Origin path encoding" footnotes this — `0x16` is reserved-pending-md1-update — but the cleanest fix is to add the missing 0x16 row in md1.
- **Why deferred:** Lives in the descriptor-mnemonic repo. Not blocking mk1 v0.1 wire-level interop because no encoder can legitimately emit 0x16 today (md1 would reject).
- **Status:** `resolved by md-codec-v0.9.0 + mk-codec-v0.2.0`. md-codec v0.9.0 ([release](https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0)) added the 0x16 row to md1's path-dictionary table. mk-codec v0.2.0 closed the parallel gap on the mk1 side: added `(0x16, "m/48'/1'/0'/1'")` to `STANDARD_PATHS`, regenerated the corpus with V18 exercising the indicator, rolled `GENERATOR_FAMILY` to `"mk-codec 0.2"` (Q-10: minor bumps roll the family token). Wire-additive: v0.1.x decoders reject v0.2-emitted 0x16 strings.
- **Tier:** `cross-repo`

### `chunked-header-total-chunks-wire-encoding-clarification` — SPEC §2.5 wording on `total_chunks` field

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 5 string-layer implementation.
- **Where:** mk1 SPEC §2.5 ("String-layer header" / chunked variant); mk1 BIP §"String-layer header" / "Chunked".
- **What:** The chunked-header `total_chunks` field was documented as "5 bits, range 1..=32," but 32 distinct values 1..=32 do not fit in 5 bits (which hold 0..=31). The mk-codec v0.1 reference implementation resolves the mismatch by encoding `count - 1` on the wire (wire 0..=31 → semantic 1..=32). The same gap applied to `chunk_set_id` endian convention — "20 bits" was silent on packing order.
- **Resolution (2026-04-29, Phase 5 review fixup):** added explicit "Wire encoding for `total_chunks`" (`count − 1`) and "Wire encoding for `chunk_set_id`" (big-endian 5-bit-symbol order) paragraphs to both `design/SPEC_mk_v0_1.md` §2.5 and `bip/bip-mnemonic-key.mediawiki` §"Chunked header". The reference implementation already encoded both correctly; this is purely a documentation tightening.
- **Status:** `closed`
- **Tier:** `pre-bip-submission`

### `error-variant-exhaustiveness-gate-strum` — replace runtime substring gate with a compile-time variant-iteration check

- **Surfaced:** 2026-04-29 v0.1.1 Phase 3 review (I-1, commit 1e42354).
- **Where:** `crates/mk-codec/tests/vectors.rs::every_error_variant_has_negative_vector`.
- **What:** The milestone v0.1.1 plan §3.3.2 specified an in-crate exhaustive `match` over `Error` variants for compile-time enforcement of negative-vector coverage. The implementation reverted to a runtime substring gate (`assert_variant_covered("...")`) because `#[non_exhaustive]` blocks integration-test exhaustive matching even for in-crate test targets — rustc treats integration tests as separate crates. The runtime gate fails when a vector is missing for a known variant, but it doesn't fire when a *new* variant is added without a corresponding `assert_variant_covered` call. The same gap applies to `error.rs::tests::parameterized_variants_render` and `static_variants_render` — both are hand-curated lists.
- **Why deferred:** Two viable resolutions, both v0.2-grade:
  1. Add `strum = { version = "0.26", features = ["derive"] }` as a dev-dep and `#[derive(strum_macros::EnumIter)]` on `Error`. The test iterates `Error::iter()` and asserts coverage for every variant. This is the path md-codec uses for its `error_coverage` test.
  2. Move the gate into `crates/mk-codec/src/error.rs::tests` (a unit-test module inside the crate), where exhaustive matching IS compile-time-checked even with `#[non_exhaustive]`. Pair with a dynamic JSON-loading helper so the unit test reads the vector corpus.
- **Status:** `resolved 901596a` (v0.2.0 Phase 1). Took option 1 with a small adaptation: instead of deriving `EnumIter` directly on `mk_codec::Error` (parameterized variants make construction-via-strum awkward), used a hand-written mirror enum `ErrorVariantName` at `crates/mk-codec/tests/error_coverage.rs` matching md-codec's exact precedent. Two tests gate the corpus: `every_error_variant_is_exercised_or_explicitly_exempt` (forward direction) and `every_negative_vector_maps_to_a_known_variant` (reverse direction; catches typos / stale vectors).
- **Tier:** `v0.2-nice-to-have`

### `vector-corpus-dictionary-coverage` — v0.1 corpus exercises only 4 of 13 path-dictionary entries

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 6 review (M-1, commit 053a54c).
- **Where:** `crates/mk-codec/tests/vectors/v0.1.json` (V1..V8 fixture set).
- **What:** The v0.1 vector corpus exercises std-table indicators 0x03 (BIP 84), 0x05 (BIP 48 segwit-v0 mainnet), 0x07 (BIP 87), and 0x15 (BIP 48 testnet) plus the 0xFE explicit-path codec. Missing: 0x01 (BIP 44), 0x02 (BIP 49), 0x04 (BIP 86), 0x06 (BIP 48 nested-segwit mainnet), and the testnet entries 0x11, 0x12, 0x13, 0x14, 0x17. A third-party encoder could pass all 8 v0.1 vectors while still mishandling BIP 44/49/86 mainnet inputs.
- **Why deferred:** The internal encoder unit test `bytecode/path::round_trip_all_standard_paths` already cycles every dictionary entry; the gap is in the cross-implementation conformance corpus, not in encoder correctness. Closing the gap is straightforward (one fixture per missing indicator) but expands the corpus from 8 to ~14 vectors; defer to the pre-bip-submission corpus expansion.
- **Status:** `resolved 2417401 + fd6a407` (v0.1.1 Phase 2 + v0.2.0 Phase 2). v0.1.1 added V9..V17 covering 9 of the 10 missing indicators; v0.2.0 added V18 for 0x16 after md-codec v0.9.0 / mk-codec v0.2.0 closed the wire-additive parallel gap. Corpus now exercises every closure-locked path-dictionary entry (14 std-table + 0xFE explicit).
- **Tier:** `pre-bip-submission`

### `cross-chunk-hash-test-fixture-stability` — Phase 5 perturbation test fixture brittleness

- **Surfaced:** 2026-04-29 Phase 5 review (M-3, commit 12c54f8).
- **Where:** `crates/mk-codec/src/string_layer/pipeline.rs` test `decode_rejects_perturbed_cross_chunk_hash`.
- **What:** The test perturbs the last byte of the last chunk's fragment and re-encodes, asserting `CrossChunkHashMismatch`. Under the current fixture this works, but the test depends on the perturbation not landing somewhere the BCH t=4 correction silently un-perturbs into a CRC-valid bytecode. A future fixture change could mask the test. Cleanest fix: perturb in 5-bit-symbol space *after* re-encoding, or pin a perturbation pattern at BCH-distance > 4 from any valid codeword in the chunk's data part.
- **Why deferred:** Test is currently green; the brittleness is potential, not actual. v0.1-nice-to-have.
- **Status:** `resolved 8685608 + 8df9910` (v0.1.1 Phase 1 Task 1.1 + Phase 1 review fixup). Replaced with `decode_rejects_5_symbol_burst_in_last_chunk_data_part` which perturbs at the 5-bit-symbol layer **past the chunked header** (chars 11..16); a 5-symbol burst always exceeds BCH `t = 4` correction radius. Accept set widened to `{CrossChunkHashMismatch, BchUncorrectable}`. The Phase 1 fixup commit `8df9910` moved the perturbation from chars 3..8 (inside the chunked header) to chars 11..16 (post-header) so the test's accept set stays tight against actual code paths.
- **Tier:** `v0.1-nice-to-have`

### `pipeline-decode-mixed-header-error-naming` — `ChunkedHeaderMalformed` variant overloaded

- **Surfaced:** 2026-04-29 Phase 5 review (M-5, commit 12c54f8).
- **Where:** `crates/mk-codec/src/string_layer/pipeline.rs::decode` — the `[SingleString, Chunked, ...]` and `[Chunked, SingleString, ...]` rejection paths surface as `Error::ChunkedHeaderMalformed("…")`. The variant name suggests a chunked-set issue; the actual condition is "header types disagree across the supplied strings." Consider adding a dedicated `MixedHeaderTypes` Error variant (or a more specific `String`-parameterised variant) when the v0.2 wire format admits more chunk types and the discrimination matters.
- **Why deferred:** Reachable only through user error; current message text is clear. Variant proliferation has its own cost. v0.1-nice-to-have.
- **Status:** `resolved 8685608` (v0.1.1 Phase 1 Task 1.2). Added `Error::MixedHeaderTypes`; migrated `pipeline.rs:137` (forward direction) and `chunk.rs:171` (reverse direction); preserved `chunk.rs:124` defense-in-depth as `ChunkedHeaderMalformed`. CHANGELOG calls out the message-text change for downstream consumers.
- **Tier:** `v0.1-nice-to-have`

### `encode-with-chunk-set-id-singlestring-silent-ignore` — explicit `chunk_set_id` is silently dropped

- **Surfaced:** 2026-04-29 Phase 5 review (M-6, commit 12c54f8).
- **Where:** `crates/mk-codec/src/string_layer/pipeline.rs::encode_with_chunk_set_id`.
- **What:** When the bytecode lands in single-string territory, the `chunk_set_id` parameter is silently ignored. This is friendly but masks a Phase-6-vector-regenerator failure mode: if the SingleString-vs-Chunked cutoff drifts, vectors pinned with explicit chunk_set_id may stop testing what they intended. Consider returning `Err(Error::ChunkedHeaderMalformed("chunk_set_id supplied but encoding is SingleString"))` when the override is supplied and the bytecode fits in a single string. Alternative: document that the test harness should assert the `chunked vs single` plan before pinning.
- **Why deferred:** The Phase-6 vector corpus generator (next phase) will surface this if it happens; better to defer the API decision until the regenerator is concrete.
- **Status:** `wont-fix — moot per SPEC §2.4 (SingleString unreachable for v0.1 conforming KeyCards).`
- **Closure note:** Closed during v0.1.1 Phase 1 Task 1.3 (`design/MILESTONE_v0_1_1.md`). The smallest valid v0.1 bytecode is 80 bytes (1+1+4+1+73), already above SINGLE_STRING_LONG_BYTES = 56; the SingleString branch in `encode_with_chunk_set_id` is dead code under the v0.1 wire format, and the `chunk_set_id` argument is therefore never silently dropped under any conforming input.
- **Sequencing requirement:** if a future format extension lands a smaller bytecode (e.g., the Compact-65 mode discussed in SPEC §3.6, which would drop `xpub.version` + `xpub.parent_fingerprint` and bring some bytecodes below 56 bytes), this item MUST be re-opened **before the format extension ships**. The silent-drop semantics is friendly today but masks an encoder-side determinism bug under any wire format that makes SingleString reachable. Any future smaller-bytecode design pass (or a Compact-65-shaped FOLLOWUPS entry) MUST cite this requirement and re-open the issue.
- **Tier:** `v0.1-nice-to-have`

### `path-dictionary-mirror-stewardship` — formalize mk1↔md1 path-dictionary inheritance contract (retired in mk-codec v0.2.2)

- **Surfaced:** 2026-04-29 mk1 v0.1 Phase 2 BIP review open observation (commit 4728230).
- **Where:** mk1 SPEC §3.5; mk1 BIP §"Origin path encoding"; md1 BIP §"Path dictionary".
- **What:** mk1's path dictionary was originally contractually identical to md1's `Tag::SharedPath` table. If md1 allocated new dictionary entries (e.g., closing the 0x16 gap, or adding new BIP-style accounts in future md1 revisions), mk1 was to inherit the allocation by the byte-for-byte mirror clause. The contract was formalized in md-codec v0.9-p3 (commit `abbec54`).
- **Why deferred:** Originally process / stewardship concern, not a v0.1 release blocker.
- **Status:** `resolved 6509f8e` (mk-codec v0.2.2 docs-only patch). md-codec v0.11 (per [`descriptor-mnemonic/design/SPEC_v0_11_wire_format.md`](https://github.com/bg002h/descriptor-mnemonic/blob/main/design/SPEC_v0_11_wire_format.md) §1.4 — "Wire-layer dictionaries (path, use-site-path, shape). Considered and rejected for architectural cleanliness") dropped the md1 path dictionary entirely; md1 now encodes paths explicitly via `OriginPath`. The mirror invariant therefore has no md1-side anchor: there is nothing left to mirror. mk-codec v0.2.2 retired the mirror clause across the mk-codec source doc-comments (`crates/mk-codec/src/bytecode/path.rs::STANDARD_PATHS` rustdoc + module docs), `design/SPEC_mk_v0_1.md` §3.5 (added "Path dictionary divergence note (v0.2.2)"), and `bip/bip-mnemonic-key.mediawiki` §"Origin path encoding". mk1's standard-table dictionary is now documented as **mk1-internal** (standalone). md-codec v0.11+ does not carry a sibling table; the `descriptor-mnemonic/CLAUDE.md` cross-repo coordination block notes the retirement under "Recently retired".
- **Tier:** `cross-repo`

### `mnemonic-gui-schema-mirror` — companion to `bg002h/mnemonic-gui` schema gate

- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`; CI gate at `.github/workflows/schema-mirror.yml`.
- **Where:** This CLI's clap-derive `Args` blocks for every subcommand the GUI surfaces (v0.1: `mk inspect`; v0.2+: encode/decode/verify/test-vectors).
- **What:** The `mnemonic-gui` GUI mirrors this CLI's clap-derive flag surface at pinned tag `mk-cli-v0.2.0`. Any flag add / remove / rename / `conflicts_with` / `required_unless_present_any` change in this repo's CLI surface must land in lockstep with a companion `mnemonic-gui` PR that bumps the schema + the `pinned-upstream.toml` tag for this CLI. The `mnemonic-gui` CI gate runs `cargo install --locked --git <this-repo> --tag <pin>` + `cargo test --test schema_mirror`, so drift surfaces as a CI failure.
- **Status:** `open` (mirror-invariant; tracking only — every flag-surface PR carries this lockstep work).
- **Tier:** `v1 / mirror-invariant`

### `error-bchuncorrectable-doc-says-8-for-long` — `Error::BchUncorrectable` doc reads "8 for long" but both codes are t=4

- **Surfaced:** 2026-05-29, mk-codec test-hardening cycle (theme 2 T2-doc).
- **Where:** `crates/mk-codec/src/error.rs:56` — `/// substitution capacity (4 for regular, 8 for long).`
- **What:** The parenthetical reads as a correction count, but the long code `BCH(108,93,8)` has `t = 4` (the `8` is the designed minimum distance / syndrome count). Both regular and long correct up to 4 substitutions (`string_layer/bch.rs:376,451`; `bch_decode.rs:566` rejects `deg > 4`). Reword to "(t = 4 for both; the trailing 8 in BCH(•,•,8) is the min-distance, not the correction count)".
- **Why deferred:** doc-only; no behavior impact. Fold into any error-surface touch.
- **Status:** `open`
- **Tier:** `docs`

### `mk1-depth-child-lossless-by-construction-unenforced` — encoder drops xpub.depth/child_number and reconstructs from path WITHOUT validating agreement

- **Surfaced:** 2026-05-29, mk-codec test-hardening cycle (theme-1 strategy design; `design/SPEC_mk_codec_test_hardening.md` §1.1).
- **Where:** `crates/mk-codec/src/bytecode/xpub_compact.rs:4` (drops depth/child), `:85-106` (`reconstruct_xpub` rebuilds them from `origin_path`), `bytecode/encode.rs` (`XpubCompact::from_xpub` silently drops). SPEC `design/SPEC_mk_v0_1.md:263,301` claims "lossless by construction" + removes `XpubDepthMismatch`.
- **What:** The "lossless by construction" claim holds ONLY when the caller passes `xpub.depth == origin_path.len()` and `xpub.child_number == origin_path.last()`. Nothing enforces this; a depth-4 xpub + 3-component path silently round-trips to a DIFFERENT xpub. Decide: (a) re-introduce encode-time `XpubDepthMismatch` (genuinely lossless), OR (b) document the lossy contract + pin it with a test. The toolkit compensates downstream (`mnemonic-toolkit/crates/mnemonic-toolkit/src/synthesize.rs:494-503` depth check) — option (a) would let it drop that.
- **Why deferred:** behavior/contract decision (likely MINOR bump + toolkit coordination), out of the test-only test-hardening cycle's scope. The cycle's theme-1 strategy sidesteps it by building the xpub from the path (depth/child consistent by construction).
- **Status:** `resolved bc4c338` — mk-codec 0.3.2: `encode_bytecode` rejects depth/child-mismatched cards via the new `Error::XpubOriginPathMismatch` (covers BOTH depth and terminal child; option (a)). `SPEC_mk_v0_1.md` §3.6/§4 re-framed as an encoder-side invariant (decoder cannot detect — no on-wire depth). The toolkit's compensating check (`mnemonic-toolkit/.../synthesize.rs:494-503`, companion `mk1-depth-child-compensating-check-watch`) is now reviewable-for-removal but kept as defense-in-depth this cycle.
- **Tier:** `v0.4`
- **Companion:** `mnemonic-toolkit` FOLLOWUP `mk1-depth-child-compensating-check-watch`.

### `mk1-no-path-depth0-support` — mk1 carries no origin path for a depth-0 / no-path key (WIF, master)

- **Surfaced:** 2026-05-30, toolkit-side discovery during the mk-codec 0.3.2 re-pin (a `mnemonic bundle --slot wif=…` card builds a depth-0 xpub with an empty `origin_path`; the 0.3.2 guard rejected it, and even pre-guard the wire couldn't decode it). Design directive: "permissive on input, expressive on output; no path applies, no path is included on the wire."
- **Where:** `crates/mk-codec/src/bytecode/path.rs` (`decode_explicit_path` accepted only `count 1..=10`), `xpub_compact.rs:85-106` (`reconstruct_xpub` `.expect()`d a non-empty path), `bytecode/encode.rs` (the 0.3.2 `XpubOriginPathMismatch` guard rejected empty paths via the child clause). `design/SPEC_mk_no_path_support.md` (R1-GREEN spec).
- **What:** A raw WIF / non-HD master key has no derivation path; mk1 must carry none on the wire and round-trip a depth-0 xpub. 0.4.0: `decode_explicit_path` accepts explicit `count == 0` → empty path; `reconstruct_xpub` empty → `depth 0` / child `Normal{0}`; the encode guard accepts a consistent depth-0 card (`expected_child = path_child.unwrap_or(Normal{0})`) while still rejecting genuine disagreement. Wire-additive (reuses `0xFE 0x00`), so MINOR.
- **Why deferred:** N/A — shipped in mk-codec 0.4.0 / mk-cli 0.5.0.
- **Status:** `resolved 82c015e` — mk-codec 0.4.0. SPEC_mk_v0_1.md §3.5/§3.6/§4 updated (E1-E9); `SPEC_mk_depth_child_enforcement.md` superseded-in-part (E10). No new error variant; no GUI/manual lockstep.
- **Tier:** `v0.4`
- **Companion:** `mnemonic-toolkit` FOLLOWUP `mk1-wif-bundle-depth0-invalid-card` (toolkit re-pin to 0.4.0 + error-mirror + verify-bundle round-trip regression).

### `rustfmt-drift-fn-signature-collapse-3-files` — current stable rustfmt collapses fn signatures that origin/main keeps multi-line (CI `fmt --check --all` latent failure)

- **Surfaced:** 2026-05-29, mk-codec test-hardening cycle (Phase 3 verification).
- **Where:** `crates/mk-cli/src/cmd/repair.rs:65`, `crates/mk-cli/tests/cli_repair.rs:366`, `crates/mk-codec/src/string_layer/bch.rs:271` (`polymod_run` signature). All on origin/main.
- **What:** The repo is edition 2024. Current stable/nightly rustfmt collapses short multi-line fn signatures / call args to one line, but origin/main was last formatted with an older rustfmt that kept them multi-line. `cargo fmt --check --all` (CI `.github/workflows/ci.yml:60-61`) with a current `dtolnay/rust-toolchain@stable` therefore flags these 3 files — a latent CI-fmt failure that surfaces on the next push, NOT caused by any feature change. Fix: a one-shot `cargo fmt --all` chore commit (mechanical, no behavior change).
- **Why deferred:** out of scope for the test-only mk-codec test-hardening cycle (touches production source); pre-existing repo-hygiene condition. The cycle's own new test files are edition-2024 fmt-clean.
- **Update (2026-06-10, mk-cli v0.8.0 cycle):** now the **SOLE remaining mnemonic-key CI red** — the companion Windows `vector_file_sha256_matches_pin` failure was fixed (`.gitattributes` stale-path, commit `47de269`). Re-measured: `cargo fmt --all` on current toolchains rewrites **5 files / ~+241 −68** — the drift has GROWN since this entry (the v0.7.0 SLIP-0132 cycle landed `slip132.rs` +92, `tests/cli_slip132.rs` +187, `tests/cli_output_class.rs` +18 under the old rustfmt, plus the original `mod.rs`/`output_advisory.rs`/`bch.rs`). KEY (de-risks the fix): **stable + beta + 1.85 all fail rustfmt IDENTICALLY** across all 3 OSes → they agree on the target format, so a single `cargo fmt --all` is deterministic and satisfies every CI toolchain (the "rustfmts disagree" worry does NOT apply to these files). **Recommend the fix commit ALSO add a `rust-toolchain.toml` pin** (or document a fixed fmt toolchain) so the format stops re-drifting on each rustfmt release — otherwise this recurs every cycle. Own commit, not bundled into a feature release.
- **Status:** **resolved** (2026-06-10 chore commit, NO-BUMP). (1) One-shot `cargo +1.95.0 fmt --all` — exactly 5 files / +241 −68, verified clean under BOTH 1.95.0 and 1.85.0 (R0 round-3 re-proved the one-pass claim empirically); the CURRENT drift set was `mk-cli/src/cmd/mod.rs`, `src/output_advisory.rs`, `src/slip132.rs`, `tests/cli_output_class.rs`, `tests/cli_slip132.rs` — NONE of this entry's eponymous 3 files (they had drifted back to clean under newer rustfmt). (2) Recurrence fix DEVIATES from this entry's rust-toolchain.toml lean, with verified rationale: `dtolnay/rust-toolchain` runs `rustup default`, and rustup precedence puts a committed `rust-toolchain.toml` ABOVE the default — the file would hijack all 9 matrix lanes (the 1.85 MSRV lane would stop testing 1.85). Instead: a dedicated `fmt` CI job pinned to **1.95.0** (bump deliberately + reformat in the same commit), the 9× redundant per-matrix Rustfmt step removed, and `release-on-tag` `needs` gains `fmt` (it was fmt-gated only via the matrix). (3) BONUS unmask-proofing (R0-r2 I3): `ci.yml`'s `vectors-roundtrip` still pinned the pre-move corpus path `tests/vectors/v0.1.json` (same `33f2ca2` move-class as the `47de269` .gitattributes fix) and had been SKIPPED on every run since the fmt red began — curing fmt would have unmasked it as the next CI red; corrected to `src/test_vectors/v0.1.json` in the same commit (+ the README:50 link, the only other live stale reference). Plan + 3 R0 rounds: `design/PLAN_rustfmt_drift_pin.md`, `design/agent-reports/rustfmt-drift-pin-plan-r0-round{1,2,3}-review.md`.
- **Tier:** `ci-hygiene`

### `gitattributes-lf-rule-stale-path-windows-hash-drift` — `.gitattributes` LF rule missed the moved vector corpus (RESOLVED mk-cli v0.8.0 cycle)

- **Surfaced + resolved:** 2026-06-10, mk-cli v0.8.0 ship (CI investigation). Commit `47de269`.
- **What:** `vector_file_sha256_matches_pin` (`crates/mk-codec/tests/vectors.rs:108`) failed on ALL windows-* runners across releases. The `.gitattributes` `eol=lf` rule still targeted `crates/mk-codec/tests/vectors/v0.1.json`, but the corpus had moved to `crates/mk-codec/src/test_vectors/v0.1.json` (cf. `mk-cli-vector-corpus-inlined`); the live file fell through to `* text=auto`, so Windows checkouts (`core.autocrlf=true`) converted LF→CRLF and the byte-level SHA-256 drifted. **Fix:** repoint to `crates/mk-codec/src/test_vectors/*.json` (globbed). `git ls-files --eol` confirms `eol=lf` now applies; index + working tree already LF (no renormalization). CI verified Test green on all 9 matrix jobs (run 27278493675). LESSON: `.gitattributes` path rules are not rename-safe — same citation-drift class as line-ref drift; glob the directory.
- **Status:** `resolved`
- **Tier:** `ci-hygiene`

### `mk-slip0132-prefix-acceptance` — accept SLIP-0132 extended-key prefixes (ypub/zpub/Ypub/Zpub + testnet upub/vpub/Upub/Vpub) on `mk encode` / `mk verify --xpub`

- **Surfaced:** 2026-06-01, mk-cli A2 cycle (SPEC `design/SPEC_mk_slip0132_acceptance.md`; plan `design/IMPLEMENTATION_PLAN_mk_slip0132_acceptance.md`; reviews `design/agent-reports/mk-slip0132-spec-R0-review.md`, `mk-slip0132-plan-R0-review.md`, `mk-slip0132-plan-R1-review.md`).
- **Where:** `crates/mk-cli/src/slip132.rs` (new module — CLI input-convenience normalization); `crates/mk-cli/tests/cli_slip132.rs` (integration cells A2–A3).
- **What:** `mk encode` and `mk verify --xpub` now accept the full SLIP-0132 set: ypub/zpub/Ypub/Zpub (mainnet BIP-49/84 + BIP-48 1′/2′ multisig) and upub/vpub/Upub/Vpub (testnet counterparts). Normalization is a 4-byte version-byte swap at the base58check layer; key material is unchanged. A stderr note names the original prefix (e.g. `note: --xpub was a zpub (BIP-84 P2WPKH); normalized to xpub`). A prefix↔origin-path script-type mismatch (e.g. zpub with `m/49'/0'/0'`) is REFUSED (exit 64 UsageError) with an actionable message naming the disagreement. mk-codec is UNTOUCHED — normalization lives entirely in the CLI (`slip132.rs`), duplicating the toolkit's `slip0132.rs` table; byte-parity is CI-guarded by `slip132_version_bytes_match_slip0132`.
- **Why deferred:** N/A — shipped in mk-cli v0.7.0 (this cycle).
- **Status:** `resolved 24ba2c7` — mk-cli **v0.7.0** (A1–A3 commits `95118e8` + `1772fca` + `24ba2c7`). No sibling companion (mk-only; mk-codec untouched; no GUI/manual lockstep).
- **Tier:** `v0.7`

### `output-type-stderr-advisory-sibling-sweep-mk-md` — add the output-class stderr advisory to this CLI (constellation cycle B, Phase 2)

- **Surfaced:** 2026-05-31, mnemonic-toolkit cycle B Phase 1 ship (mnemonic + ms shipped the always-emit 3-class stderr advisory).
- **What:** Add the always-emit one-line stderr classification of stdout's security nature — byte-identical wording to `mnemonic-toolkit/crates/mnemonic-toolkit/src/secret_advisory.rs` (`warning: stdout carries private key material (can spend) …` / `note: stdout is watch-only …` / `note: stdout is a keyless descriptor template (no keys)`) — to every output-producing subcommand; inert subcommands emit nothing. mk → watch-only (decode/derive/address/inspect). md → template (decode/encode — md1 is the keyless template, the class's first real exercise) + watch-only (address). Cross-repo byte-parity test.
- **Why deferred:** non-secret outputs; benign interim silence (over-caution, no fund-loss path) vs the secret-bearing mnemonic/ms surfaces shipped in Phase 1.
- **Status:** Resolved by the output-class-advisory Phase 2 cycle — mk-cli **v0.6.1** + md-cli **v0.6.2** + toolkit **v0.38.3** add the always-emit 1-line stderr output-class advisory (mk→watch-only; md→template, plus watch-only for `md address`); completes the constellation-wide 'no advisory line ⟺ inert stdout' invariant. Per-phase reviews persisted in mnemonic-toolkit `design/agent-reports/output-type-advisory-phase2-*`.
- **Tier:** `next-cycle`
- **Companion:** `mnemonic-toolkit` FOLLOWUP `output-type-stderr-advisory-sibling-sweep-mk-md` + `output-type-stderr-advisory` (Phase 1).

### `mk-slip0132-byte-parity-test-self-referential` — `slip132_version_bytes_match_slip0132` self-referentially declares its own byte array

- **Surfaced:** 2026-06-01, mk-cli A2 phase review M1 (minor; filed as FOLLOWUP per reviewer recommendation).
- **Where:** `crates/mk-cli/src/slip132.rs` — `slip132_version_bytes_match_slip0132` test (~line 145).
- **What:** The `slip132_version_bytes_match_slip0132` test declares its own 8-entry byte-constant array and round-trips each entry through `detect_and_normalize`, confirming the variant mapping is correct. This catches a one-sided or transposition edit (change one arm in production but forget the test array, or vice versa), but a coordinated edit — the same wrong byte introduced to BOTH the test array AND the matching production arm in `detect_and_normalize` — would pass green while silently mis-normalizing that prefix to the wrong canonical key. Options: (a) pin a published BIP-49/BIP-84 extended-key test vector (a known real ypub or zpub string from the BIP itself) and assert that `detect_and_normalize` produces the expected canonical xpub — a real-key anchor that cannot be faked by editing both sides in sync; (b) `include`-compare the test byte array against the toolkit's `slip0132.rs` table (cross-crate byte-parity, already implied by the module-level comment but not mechanically enforced). Low priority — byte-parity to the CI-tested toolkit table is currently verified by code review; no key-material error has been observed. The self-referential test still catches most accidental edits; this is a hardening gap, not a correctness gap.
- **Why deferred:** cosmetic hardening; no correctness issue; fixing requires sourcing published test vectors or adding a cross-crate comparison.
- **Status:** `resolved` — test-hardening T4-c (2026-07-10, mk-codec/mk-cli test-only NO-BUMP). Delivered option (a) for the **zpub arm**: `cli_slip132.rs::published_bip84_zpub_normalizes_and_matches_own_version_swap` pins the authoritative BIP-84 published zpub (`zpub6rFR7y4Q2Aij…`, bip-0084.mediawiki:75) through `mk encode --xpub` (mk-cli is bin-only → observed via the CLI), anchored to that published string's own base58check payload — a real-key anchor that cannot be faked by a coordinated both-sides edit. **Residual (Low — note, not a re-open):** only the zpub arm (1 of 8 SLIP-0132 entries) is published-vector-anchored; ypub/Ypub/Zpub + testnet arms remain self-referential, and the canonical-target constant `XPUB_MAINNET_V` is still test-local (a three-way coordinated edit is theoretically green, further fenced by coin-type/path checks). R0 GREEN (`mnemonic-toolkit:design/agent-reports/test-hardening-T4-postimpl-whole-diff.md`).
- **Tier:** `hardening`
- **Severity:** Low

### `mk-vectors-pretty-out-help-mismatch` — `mk vectors --pretty` help text wrongly claimed it was "Ignored when `--out` is supplied" (RESOLVED mk-cli v0.10.2)

- **Surfaced:** Wave-3 constellation help-text-correction cycle (W3-3). Canonical entry lives in the toolkit `design/FOLLOWUPS.md` (slug `mk-vectors-pretty-out-help-mismatch`); this is the mk-cli companion record.
- **What:** The clap doc-comment on `--pretty` (`crates/mk-cli/src/cmd/vectors.rs:22` — the `mk vectors --help` text) read "Indent the JSON output for human readability. Ignored when `--out` is supplied." This was WRONG: `write_per_fixture_files` (`vectors.rs:53-83`) branches on `pretty` and pretty-prints each per-fixture file written under `--out`, so `--pretty` IS honored. The help now states `--pretty` "Also applies to each per-fixture file when `--out` is supplied." Pure help-text/doc-comment correction — no flag, no API, no wire/behavior change; the code was already correct.
- **Fix:** reword the `--pretty` doc-comment; pin the corrected contract with a new `vectors_pretty_out_writes_indented_files` integration test (`crates/mk-cli/tests/round_trip.rs`) that asserts `mk vectors --pretty --out <DIR>` writes indented per-fixture files. `mk-codec` untouched; no flag-NAME change, so the GUI `schema_mirror` / `gui-schema` gates are unaffected (gui-schema emits no help text).
- **Status:** `resolved` — mk-cli **v0.10.2** (SemVer-PATCH, help-text-only). Spec `design/SPEC_wave3_mk_vectors_help.md`; R0 review `design/agent-reports/wave3-mk-cli-r0-review.md` (GREEN 0C/0I).
- **Tier:** `ci-hygiene`
- **Companion:** toolkit `design/FOLLOWUPS.md` slug `mk-vectors-pretty-out-help-mismatch` (canonical; flips `open → resolved` in the toolkit doc lane). GUI prose-mirror of the `--pretty` help string (`mnemonic-gui` `VECTORS_FLAGS`) handled in its own decoupled GUI lane — no CI gate compares help prose across repos.

### `canonical-payload-bytes-for-word-card` — additive `KeyCard::canonical_payload_bytes()` accessor (landed NO-BUMP; PATCH+publish at toolkit P6)

- **Surfaced:** 2026-06-25, **P0** of the toolkit's **Word-Card encoding** feature (BIP-39-word ECC re-encoding of `mk1`/`md1` payloads). **Companion:** `mnemonic-toolkit/design/FOLLOWUPS.md::word-card-encoding-finish-plan-and-implement` + plan `design/IMPLEMENTATION_PLAN_word_card_encoding.md` §2/§7-P0.
- **WHAT:** two additive `pub fn`s on `KeyCard` (`crates/mk-codec/src/key_card.rs`): `canonical_payload_bytes(&self) -> Result<Vec<u8>>` (the **deterministic pre-chunking bytecode**, invariant to the per-encode CSPRNG `chunk_set_id` which lives in the string layer) + `from_canonical_payload_bytes(&[u8]) -> Result<KeyCard>`. Pure facades over the already-`pub` `bytecode::encode_bytecode`/`decode_bytecode`; no visibility widening, no wire/behavior change. KATs `tests/canonical_payload.rs` (4, incl. cross-`chunk_set_id` determinism). `main@7cbd5da`; per-phase R0 GREEN (`mnemonic-toolkit/design/agent-reports/word-card-p0-r0-round-1.md`).
- **Status:** ✓ **RELEASED 2026-06-26 — `mk-codec 0.4.1` published to crates.io** (`main@31369a8`, tag `mk-codec-v0.4.1`; CHANGELOG `[0.4.1]`). The toolkit (`mnemonic-toolkit-v0.74.0`) now pins `mk-codec = "0.4.1"` and consumes the accessor in its `mnemonic word-card` CLI. **Tier:** `feature` (cross-repo companion). **DONE.**

### `vendor-freshness-pr-gate` — no PR-time guard that `vendor/` satisfies `Cargo.lock` (companion to mnemonic-toolkit)

This repo commits a `vendor/` tree consumed by the `--offline --locked` reproducible build, but has NO leading PR-time check that it stays in sync with `Cargo.lock` — the same latent bug that broke `mnemonic-toolkit` **v0.74.0**'s reproducible release (a codec dep bump without `cargo vendor` → the tag-triggered repro build could not resolve, caught only at the release tag).

- **Status:** ✓ **RESOLVED (2026-06-28)** — ported `ci/repro/vendor-freshness.sh` + `.github/workflows/vendor-freshness.yml` (TWO-block fork-free form; defensive git-source tripwire added so a future git dep fails closed rather than silently mis-resolving). Empirically verified FRESH→exit 0, STALE→exit 1 (vendor restored byte-clean); workflow runs on PR + push to the default branch, path-filtered. **Tier:** `ci`. **Companion:** `mnemonic-toolkit` `design/FOLLOWUPS.md::vendor-freshness-pr-gate` (RESOLVED there 2026-06-26) + `docs/verify-reproducibility.md`.

### `impl-bch-erasure-decoding-md-mk` — port erasure-aware BCH decoding to mk1 (+ md1 companion), then re-upgrade the BIP MUST

- **Surfaced:** 2026-07-10, cross-repo BIP-alignment cycle. Consolidated bug list `mnemonic-toolkit/design/BUGLIST_bip_alignment_cycle_2026-07-10.md` (downgrade-ledger DG-1); mk1 SPEC `design/SPEC_mk1_bip_alignment.md` §Part-2 C-I3. The mk1 BIP specified erasure-aware BCH decoding as MUST/REQUIRED (lines 145-146). Ground-truth check (this cycle): mk-codec does NOT implement it — `mk repair` (`bch.rs`) performs BCH **substitution**-error correction only (t=4, regular and long code). Erasure decoding exists ONLY in the separate `wc-codec` word-card RS layer, a different codec entirely. This cycle downgraded the BIP MUST → SHOULD/informative rather than ship a normative claim the reference decoder doesn't meet.
- **Why:** shared `POLYMOD_INIT 0x23181b3` BCH construction (algorithm-identical to md1, HRP/target differ) means the code distance supports erasure correction in principle; the decoder just doesn't implement it. Port erasure-marking + erasure-aware decoding to mk-codec's `bch.rs`, then re-upgrade the BIP §Checksum text back to MUST. Fix in lockstep with the md1 companion leg (algorithm parity).
- **Status:** OPEN.
- **Tier:** `feature` / cross-repo.
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` → `impl-bch-erasure-decoding-md-mk` (md1 leg).

### `impl-guided-recovery-md-mk` — implement guided/constrained-radius recovery search for mk1 (+ md1 companion), then re-upgrade the BIP

- **Surfaced:** 2026-07-10, cross-repo BIP-alignment cycle (`mnemonic-toolkit/design/BUGLIST_bip_alignment_cycle_2026-07-10.md` DG-2; mk1 SPEC `design/SPEC_mk1_bip_alignment.md` §Part-2 C-I3, lines 145/150). The mk1 BIP specified guided/constrained-radius recovery search as REQUIRED/SHOULD-adjacent; ground truth: `mk repair` performs BCH substitution-correction only, no structure-elicited candidate search. Downgraded REQUIRED → SHOULD this cycle.
- **Why:** mirrors the md1 companion gap exactly (`repair` does BCH correction, not guided search); `wc-codec`'s single-deletion candidate search (word-card feature) is the nearest prior art to adapt.
- **Status:** OPEN.
- **Tier:** `feature` / cross-repo.
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` → `impl-guided-recovery-md-mk` (md1 leg).

### `impl-confidence-tier-reporting-md-mk` — implement 4-tier confidence/outcome/method reporting for mk1 repair (+ md1 companion), then re-upgrade the BIP

- **Surfaced:** 2026-07-10, cross-repo BIP-alignment cycle (`mnemonic-toolkit/design/BUGLIST_bip_alignment_cycle_2026-07-10.md` DG-3; mk1 SPEC `design/SPEC_mk1_bip_alignment.md` §Part-2 C-I3). The mk1 BIP specified the same 4-tier confidence/outcome/method reporting ladder as MUST; ground truth: `mk repair` reports corrections but no tiers. Downgraded MUST → SHOULD this cycle.
- **Why:** mirrors the md1 companion gap exactly; keep the honest substitution-correction reporting `mk repair` already does, ledger the tier ladder as future work.
- **Status:** OPEN.
- **Tier:** `feature` / cross-repo.
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` → `impl-confidence-tier-reporting-md-mk` (md1 leg).

### `mk1-bip-presubmission-nits` — pre-submission nits for the mk1 BIP (V19 parent-fingerprint, pinned-SHA embedding, BCH notation sweep)

- **Surfaced:** 2026-07-10, cross-repo BIP-alignment cycle (`mnemonic-toolkit/design/BUGLIST_bip_alignment_cycle_2026-07-10.md`; SPEC `design/SPEC_mk1_bip_alignment.md`). Three items deferred as pre-BIP-submission nits, not part of this cycle's alignment/honesty scope:
  - **(a) V19 depth-0 vector's xpub carries a nonzero parent fingerprint** (`0x10203013`) at depth 0. BIP-32 mandates `0x00000000` for master (depth-0) key serializations. Fix: regenerate V19 with `parent_fp = 0` — this churns the V19 fixture AND the pinned corpus SHA-256 (`tests/vectors.rs V0_1_SHA256`, already re-pinned once this cycle for the C-C2 depth-0 vector addition — this nit would require a SECOND re-pin, so batch it into the same regeneration pass if possible).
  - **(b) Embed the literal pinned corpus SHA-256 in the BIP §Test Vectors.** Currently the BIP references the corpus SHA as a floating pointer (cite the value, not just "see `tests/vectors.rs`") — an implementer verifying conformance should be able to check the corpus hash against a value printed IN the BIP text itself, not just in the source tree.
  - **(c) Sweep `BCH(n,k,8)` notation to `BCH(n,k)`** across ~20 mk-codec source-comment / design-doc sites. The BIP's own normative text was already corrected this cycle (minimum distance d≥9; "8" is the *detection* radius, not a code parameter) — see F-A6/MK1-I2 (`error.rs:57`, lines 29/480/504 of the BIP). This item is the mechanical sweep of the STALE `BCH(n,k,8)` shorthand across the remaining source-comment and design-doc sites that weren't touched by this cycle's targeted fixes.
- **Why deferred:** (a) and (b) are pre-BIP-submission polish, not correctness bugs in shipped code (V19 decodes fine; it's a documentation/test-vector-hygiene nit against BIP-32's own MUST). (c) is pure notation cleanup, batchable with any future doc pass rather than urgent.
- **Status:** OPEN. Tier `pre-bip-submission` per this repo's existing tier convention — MUST be resolved before formal BIP submission, not blocking any release.
- **Tier:** `pre-bip-submission`.
