# A2 SPEC R0 review — mk SLIP-0132 acceptance

**SPEC:** `design/SPEC_mk_slip0132_acceptance.md`
**Source SHA reviewed:** mk `main` `fc2341b`; toolkit (lockstep target) at its current `master`.
**Reviewer:** opus architect (R0, adversarial, verified against live source + the toolkit's already-shipped+tested SLIP-0132 table + empirical bitcoin-0.32 probes + a SLIP-0132 reference).

## Verdict: GREEN (0C/0I)

0 Critical, 0 Important. 3 Minor (all non-blocking; fold at plan-doc time). The SPEC is sound, byte-correct, and the two-wall premise, the implied-path predicate, the encode/verify boundary, the lockstep count, and the SemVer level all verify against live source. Cleared to proceed to plan-doc (which carries its own R0).

---

## Critical
*(none)*

## Important
*(none)*

## Minor

- **M1 — `verify`'s mismatch path emits a `ContentMismatch`-shaped concern but the SPEC routes the SLIP-0132 mismatch to `UsageError` (exit 64), which differs from `verify`'s native field-mismatch exit (`ContentMismatch` = exit 4).** — `crates/mk-cli/src/error.rs:84` (`ContentMismatch => 4`) vs `:85` (`UsageError => 64`); SPEC §5/§7. — A `verify --xpub <zpub> --origin-path m/49'…` SLIP-0132/path *mismatch* will exit **64**, whereas a `verify` xpub-value mismatch exits **4**. Both are defensible (the SLIP-0132 case is a *usage* contradiction — the user handed an incoherent prefix/path pair, not a key that failed to match a card), but the plan-doc should state this exit-code choice explicitly so a reader doesn't expect 4. The SPEC already commits to 64 (§7) and it is internally consistent with "this is a usage error caught before the card is even built" — **keep 64**, just document the contrast in the plan-doc test matrix. — Fix: add a one-line note in the plan §test-plan that SLIP-0132/path mismatch is `UsageError`(64), distinct from value-`ContentMismatch`(4); add an explicit assertion of `64` (not `4`) in the `verify --xpub <zpub> --origin-path m/49'…` cell so the distinction is regression-locked.

- **M2 — stderr ordering with the v0.6.1 output-class advisory is correct-by-construction but unasserted.** — `encode.rs:85` (parse, where the new note fires) precedes `encode.rs:97-100` (`emit_output_class_advisory(WatchOnly)`); `verify.rs` has NO advisory emit (it is inert — confirmed, no `output_advisory` call in `verify.rs`). — So on `encode`, stderr will carry the SLIP-0132 `note:` line FIRST (emitted during parse), then stdout, then the `note: stdout is watch-only …` advisory line LAST. Two distinct `note:`-prefixed lines, no dedup/ordering conflict — but the integration test should pin BOTH lines' presence + relative order so a future refactor that moves the parse emit doesn't silently reorder them. — Fix: in the `encode --xpub <zpub>` success cell, assert stderr contains the SLIP-0132 note AND the watch-only advisory; optionally assert the SLIP-0132 line appears before the advisory. (No code concern — purely test coverage.)

- **M3 — `Slip132Variant`'s implied-path predicate must be specified as "purpose component is HARDENED `N'`", not just "index == N".** — SPEC §2/§5 say "purpose `49'`/`84'`" and "script-type index `1'`/`2'`" but §3's `Slip132Variant` description ("the implied-path predicate") doesn't pin hardenedness. — A BIP-49/84/48 path's purpose + coin + account + script-type components are all HARDENED; a predicate that matched a non-hardened `m/84/0/0` (purpose `84` normal, not `84'`) would be a latent footgun even though such a path is non-standard. Empirically `DerivationPath::from_str` happily parses `m/84/0/0` as normal children. — Fix: plan-doc should specify the predicate compares against `ChildNumber::Hardened { index: 49|84|48 }` (and script-type `Hardened{1|2}`), i.e. match the exact `ChildNumber` variant, not just the numeric index. (This is the natural reading of the SPEC's `49'` notation; just make it explicit so the implementer doesn't compare bare indices.)

---

## SLIP-0132 version-byte table verification (each constant)

Cross-validated against (a) the SLIP-0132 registered values, and (b) the toolkit's **already-shipped + unit-tested** table in `crates/mnemonic-toolkit/src/slip0132.rs:54-100,141-154` (which round-trips published SLIP-0132/BIP-84 vectors in CI — `slip0132_spec_bitcoin_test_vector_*` tests). Empirically confirmed the zpub→xpub swap with bitcoin 0.32 (`unknown version magic bytes: [4, 178, 71, 70]` = 0x04B24746).

| Prefix | SPEC §2 | SLIP-0132 ref | Toolkit (tested) | Network | Script-type / BIP | ✓/✗ |
|---|---|---|---|---|---|---|
| `xpub` | `0488B21E` | `0488B21E` | `[04,88,B2,1E]` | Main | neutral | ✓ |
| `tpub` | `043587CF` | `043587CF` | `[04,35,87,CF]` | Test | neutral | ✓ |
| `ypub` | `049D7CB2` | `049D7CB2` | `[04,9D,7C,B2]` | Main | P2SH-P2WPKH / BIP-49 | ✓ |
| `zpub` | `04B24746` | `04B24746` | `[04,B2,47,46]` | Main | P2WPKH / BIP-84 | ✓ |
| `Ypub` | `0295B43F` | `0295B43F` | `[02,95,B4,3F]` | Main | P2WSH-P2SH multisig / BIP-48 sti `1'` | ✓ |
| `Zpub` | `02AA7ED3` | `02AA7ED3` | `[02,AA,7E,D3]` | Main | P2WSH multisig / BIP-48 sti `2'` | ✓ |
| `upub` | `044A5262` | `044A5262` | `[04,4A,52,62]` | Test | P2SH-P2WPKH / BIP-49 | ✓ |
| `vpub` | `045F1CF6` | `045F1CF6` | `[04,5F,1C,F6]` | Test | P2WPKH / BIP-84 | ✓ |
| `Upub` | `024289EF` | `024289EF` | `[02,42,89,EF]` | Test | P2WSH-P2SH multisig / BIP-48 sti `1'` | ✓ |
| `Vpub` | `02575483` | `02575483` | `[02,57,54,83]` | Test | P2WSH multisig / BIP-48 sti `2'` | ✓ |

**All 10 constants correct.** Network mapping correct (4 mainnet non-canonical → `xpub`; 4 testnet → `tpub`). Script-type/BIP mapping correct: ypub/zpub = single-sig BIP-49/84; Ypub/Zpub = BIP-48 multisig with script-type index 1' (P2WSH-P2SH) / 2' (P2WSH) — matches the toolkit's `XpubPrefix::YpubMultisig`/`ZpubMultisig` semantics and BIP-48's registered script-type table.

---

## Citations & premises verified

- **Two-wall premise — ✓.** Wall 1: `parse_xpub`→`Xpub::from_str` at `cmd/mod.rs:57-58` (verbatim). Empirically rejects a raw zpub (`unknown version magic bytes: [4, 178, 71, 70]`). Wall 2: `version_to_network` at `mk-codec/src/bytecode/xpub_compact.rs:63-69` knows only `MAINNET_XPUB_VERSION`/`TESTNET_XPUB_VERSION` (`:25,:28`), errors `InvalidXpubVersion` otherwise. `parse_xpub` shared by `encode` (`cmd/encode.rs:85`) + `verify` (`cmd/verify.rs:53`) — confirmed both call sites.
- **encode `--origin-path` REQUIRED, verify OPTIONAL — ✓.** `encode.rs:27-28` `pub origin_path: String` (required, not `Option`); `verify.rs:30-31` `pub origin_path: Option<String>` (optional). `--xpub` is required `String` on encode (`:19-20`), `Option<String>` on verify (`:22-23`). Empty/depth-0 path: `DerivationPath::from_str("")` and `"m"` both parse to a length-0 path with NO panic (empirically verified) — so encode's required `--origin-path ""` legitimately yields the empty path; the predicate's short-path guard (SPEC §10) is therefore genuinely needed and the SPEC flags it.
- **Implied-path predicate (§2/§5) — ✓** (with M3 hardenedness nit). `ypub→49'`, `zpub→84'`, `Ypub→48'∧sti 1'`, `Zpub→48'∧sti 2'` correct per BIP-49/84/48; script-type index = 4th component of `m/48'/coin'/account'/script_type'` correct; treating canonical `xpub`/`tpub` as "no claim, no check" is sound — a plain `xpub` at `m/84'/0'/0'` is the standard case and must NOT be predicate-checked (SPEC §5 last para + §10 explicitly carve this out).
- **Normalization soundness — ✓.** version-swap + base58check re-checksum + `Xpub::from_str` empirically yields the correct depth/child/network with byte-identical key material. base58check re-checksum is mandatory (SPEC §10) — splicing into the old string would fail the checksum. The depth/child guard (`mk-codec/src/bytecode/encode.rs:38-48`) still applies AFTER normalization (it inspects `card.xpub.depth`/`child_number` vs `origin_path`, independent of version bytes — version bytes are not in the guard). SPEC's "purpose check is additive + runs first (at parse) + clearer message; both must pass" holds: a zpub@`m/84'/0'/0'` (depth 3, child 0') normalizes to depth-3/child-0' xpub, satisfying both the new purpose predicate (purpose 84') and the codec guard (depth 3 == 3, child 0' == 0'). The mismatch cell `zpub + m/49'/0'/0'` is caught ONLY by the new predicate (depth/child guard would pass: 3==3, 0'==0') — confirming the predicate adds real value.
- **encode vs verify boundary (§6) — ✓.** verify-without-`--origin-path` → normalize + note, skip predicate, key-material match still runs (`verify.rs:52-61`). This preserves a legit bare `verify --xpub <zpub> <mk1>`. verify-with-`--origin-path` → run predicate, refuse on mismatch. No panic path: the predicate must guard `path.len() >= 1` (single-sig purpose) / `>= 4` (multisig sti) before indexing — SPEC §10 footgun covers it; length-0 path is reachable and must hit the unsatisfiable-predicate message (§5 third bullet), not an index panic.
- **mk-codec genuinely untouched — ✓.** Normalization builds a canonical `Xpub` in mk-cli and hands it to `KeyCard::new(...)` → `mk_codec::encode` (`encode.rs:87-88`) exactly as today. No codec edit is forced; `version_to_network`/`XpubCompact`/the depth-child guard all stay canonical-only and unchanged. No new `mk_codec::Error` variant needed (SPEC §7).
- **Phase-2 advisory coexistence — ✓.** `encode.rs:97-100` emits `OutputClass::WatchOnly` (output_advisory.rs:31 → `note: stdout is watch-only …`) AFTER stdout, AFTER the parse-time SLIP-0132 note. `verify.rs` has NO `output_advisory` call (inert — confirmed by full read of verify.rs). Two distinct stderr `note:` lines on encode, no dedup/order conflict (the SLIP-0132 line fires during parse, the advisory after stdout). verify's inert-ness conflicts with nothing — its SLIP-0132 note is the only advisory-class stderr it would emit. (M2: pin order in tests.)
- **Lockstep (§8) — ✓.** Exactly **3 mk-cli pin sites**, all at `mk-cli-v0.6.1`: `scripts/install.sh:41`, `.github/workflows/manual.yml:77`, `.github/workflows/quickstart.yml:71`. md-cli (`descriptor-mnemonic-md-cli-v0.6.2`) and ms-cli (`ms-cli-v0.5.0`) pins are separate `component_info` lines, untouched by an mk bump. `sibling-pin-check.yml` will gate the 3-site consistency. No clap flag added (`--xpub` VALUES widened, NAME unchanged) → no GUI `schema_mirror` change (it gates flag-NAME parity) and no manual flag-coverage lint change. mk-cli manual chapter exists at `docs/manual/src/40-cli-reference/44-mk-cli.md` (16.5 KB) and contains no ypub/zpub today — the prose addition is genuinely net-new. No CI-gated mk transcript ingests a ypub/zpub (verified — no transcript re-capture needed).
- **SemVer — ✓.** mk-cli MINOR `0.6.1 → 0.7.0` correct: purely additive (every SLIP-0132 prefix was *wholly refused* before — no previously-accepted input changes behavior). Current mk-cli `Cargo.toml` version = `0.6.1` (confirmed). Toolkit re-pin = PATCH (binary-consumer pin bump). mk-codec library pin unchanged (`mk-codec = { path, version = "0.4.0" }`) — no codec re-pin.
- **Fixture feasibility — ✓.** Empirically version-swapped a published SLIP-0132 zpub to a valid xpub and re-parsed (depth 3, child 0'). The inverse (xpub→ypub/zpub/Ypub/Zpub) is exactly the toolkit's tested `apply_xpub_prefix`. Corpus xpubs usable: `V2_84_MAIN` @ `m/84'/0'/0'` (depth 3 → zpub), `V1_48_MULTISIG` @ `m/48'/0'/0'/2'` (depth 4 → Zpub), `V9_44_MAIN` @ `m/44'/0'/0'` (xpub control), and a `m/49'/0'/0'` fixture must be forward-derived (no depth-3 49' corpus xpub present — SPEC §9 already says "forward-derive otherwise"). All fixtures satisfy the codec depth/child guard at their matching paths.

---

## Notes

- **Strongest validation signal:** the toolkit (downstream consumer in the same constellation) already ships `crates/mnemonic-toolkit/src/slip0132.rs` with the identical 10-entry version table, CI-tested against published SLIP-0132/BIP-84 vectors. The mk-cli implementation is a re-implementation of a proven, tested primitive — low risk of a wrong byte slipping through. The plan-doc may note this as a reference (do NOT depend on the toolkit — mk-cli is upstream; duplicate the table, as `output_advisory.rs` already duplicates the toolkit's advisory helper with a cross-repo byte-parity test).
- The SPEC's `xpub`/`tpub` = "no script-type claim" carve-out is the single most important correctness decision and it is correct: BIP-44/49/84/86 wallets commonly store the account key as a neutral `xpub` regardless of purpose; predicate-checking canonical inputs would break the standard `xpub`-at-`84'` case. The SPEC handles this in §5 (last para) and §10 (sixth footgun).
- Plan-doc must carry the §10 footguns verbatim into its test matrix, especially the short/empty-path no-panic guard (length-0 path is reachable via encode's required-but-emptyable `--origin-path`) and the M3 hardenedness specification of the predicate.
- The "verify mismatch is exit 64 not 4" (M1) is a deliberate, defensible choice — fold the note + the explicit `64` assertion and it's locked.
- `missing_docs = "warn"` + `clippy::all = "warn"` are workspace lints (`Cargo.toml:11-15`); CI runs `clippy --all-targets -- -D warnings` (SPEC §9), so every `pub` item in the new `slip132.rs` needs a doc comment — SPEC §10 last footgun covers it.
