# A2 end-of-cycle R0 review — mk SLIP-0132 acceptance

Reviewer: opus architect (end-of-cycle R0, cross-cutting/integration focus).
Date: 2026-06-01.
Repos / ranges reviewed against live source:
- mk: `/scratch/code/shibboleth/mnemonic-key` branch `mk-slip0132-acceptance`, `git diff fc2341b..dfdf111` (HEAD `dfdf111`).
- toolkit: `/scratch/code/shibboleth/mnemonic-toolkit` branch `mk-slip0132-acceptance`, `git diff 4d5ef56..36b08a4` (HEAD `36b08a4`).

## Verdict: RED (1C/0I)

One Critical: the manual's `mk-cli` install command is pinned to a stale `mk-cli-v0.6.0`
that lacks the very feature the same chapter now documents. This violates the CLAUDE.md
manual mirror invariant and is a user-facing self-contradiction. It is a one-line fix
(no code, no test, no re-review of the feature). Everything else in the cycle is GREEN.

---

## Critical

### C1 — Manual install command pins `mk-cli-v0.6.0`; same chapter documents v0.7.0-only SLIP-0132 behavior (self-contradiction + mirror-invariant violation)

- **Where:** `mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md:12`
  ```
  `cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.6.0 --bin mk`.
  ```
- **Evidence:**
  - Canonical pin everywhere else this cycle = `mk-cli-v0.7.0`: `scripts/install.sh` (mk arm), `.github/workflows/manual.yml:77`, `.github/workflows/quickstart.yml:71`, CHANGELOG `[0.38.4]` entry — all v0.7.0 (verified by diff + sibling-pin-check inline run, exit 0).
  - The SAME file then adds (this cycle, commit `36b08a4`) a `### SLIP-0132 prefix acceptance (--xpub)` subsection at lines 72–94 and a `mk verify` cross-reference at lines 349–354 documenting behavior that ships ONLY in mk-cli **v0.7.0**. A reader who follows the install command on line 12 gets v0.6.0, which has no SLIP-0132 acceptance — the documented `mk encode --xpub zpub…` will fail for them.
  - This is NOT caught by any automated gate. `sibling-pin-check.yml` scans only `.github/workflows/*.yml` cargo-install lines (confirmed by reading the workflow body — `for wf in .github/workflows/*.yml`); it does not scan `docs/manual/src/**`. The install command on line 12 is pure prose — not transcript-gated (`verify-examples.sh` runs `.cmd`/`.out` transcript pairs, and no transcript invokes `cargo install`), not value-gated by `lint.sh` (flag-coverage only). cspell + markdownlint pass it (they do not validate tag currency).
  - **Latency root cause / fix-the-class signal:** `git log -S` shows the prior cycle's pin bump `752801f` ("bump mk-cli v0.6.1 … (5 sites)") updated install.sh + CI but left `44-mk-cli.md:12` at v0.6.0; this cycle's `36b08a4` again updated install.sh + CI to v0.7.0 but again skipped line 12. The manual install command has now drifted across TWO cycles (v0.6.0 vs v0.6.1 vs canonical v0.7.0) precisely because it lives outside the sibling-pin-check gate's `.github/workflows/` scope. The CLAUDE.md manual mirror invariant ("any flag/API addition … must update the manual under docs/manual/src/40-cli-reference/ in lockstep") is the discipline that was supposed to cover this; it was missed both times.
- **Fix (this cycle, before tag):** bump `44-mk-cli.md:12` to `--tag mk-cli-v0.7.0`. One-line edit; re-run cspell + markdownlint on the file (already green). No feature re-review needed.
- **Recommended follow-up (file, do not necessarily fix this cycle):** extend the sibling-pin-check gate (or a sibling lint) to also scan `docs/manual/src/**` and `docs/quickstart/**` prose `cargo install --git … --tag <tag> <pkg>` lines against `scripts/install.sh` canonical — the gate's current `.github/workflows/*.yml`-only glob is why this drifted twice undetected. (Toolkit-side FOLLOWUP; companion to `manual-yml-sibling-pin-vs-install-sh-drift-gate`.)

---

## Important

None.

---

## Minor

### M1 — FOLLOWUP narrative paraphrases the stderr note (not the exact emit)

- `mk/design/FOLLOWUPS.md` (`mk-slip0132-prefix-acceptance` entry) quotes the note as
  `note: --xpub was a zpub (BIP-84 P2WPKH); normalized to xpub`, but the live emit
  (`crates/mk-cli/src/cmd/mod.rs`) is
  `note: --xpub was a SLIP-0132 zpub (BIP-84 P2WPKH); normalized to canonical xpub — script type is conveyed by the origin path, not the key prefix`.
  This is a narrative paraphrase inside a FOLLOWUP body — NOT a test assertion and NOT manual
  prose (the manual at `44-mk-cli.md:85` quotes the note byte-exact). No correctness impact; the
  authoritative manual line is correct. Optional tidy.

### M2 — Toolkit-side `slip0132.rs` comment mislabels the `0x0295b43f` variant as "BIP-49 multisig"

- `mnemonic-toolkit/crates/mnemonic-toolkit/src/slip0132.rs:22` comments `Ypub` (`02 95 B4 3F`)
  as "BIP-49 multisig (P2SH-P2WSH)". Per SLIP-0132 the registered description is "Multi-signature
  P2WSH in P2SH" (BIP-48 m/48' path), which is exactly how the new mk-cli `slip132.rs` labels it
  ("BIP-48 P2WSH-P2SH multisig") and gates it (`path_matches` checks `m/48'` + script-type index).
  mk's labels/predicate are the correct ones. The toolkit comment is pre-existing, cosmetic, and
  out of scope for this cycle — noted only so it isn't mistaken for a parity discrepancy. No byte
  disagreement (all 8 version→swap mappings match exactly, see spot-check below).

---

## Cross-repo checks

- **Version ↔ pin consistency — ✓.** mk-cli `Cargo.toml` = `0.7.0`; toolkit `Cargo.toml` = `0.38.4`;
  both toolkit README markers (`README.md:13`, `crates/mnemonic-toolkit/README.md:9`) =
  `toolkit-version: 0.38.4`. All three toolkit mk-cli sibling-pin sites = `mk-cli-v0.7.0`
  (`scripts/install.sh` mk arm, `.github/workflows/manual.yml:77`, `.github/workflows/quickstart.yml:71`).
  md-cli (`descriptor-mnemonic-md-cli-v0.6.2`) and ms-cli (`ms-cli-v0.5.0`) pins untouched. Both
  `Cargo.lock` files bumped in lockstep with their crate. **sibling-pin-check inline run against the
  toolkit tree = exit 0** (4 OK lines, 0 errors). *(The one stale pin is in manual PROSE, outside
  this gate's scope — see C1.)*
- **Feature soundness spot-check — ✓.** All 8 SLIP-0132 version bytes in mk `slip132.rs:106–113`
  byte-match toolkit `slip0132.rs:82–90` (ypub `049D7CB2`, Ypub `0295B43F`, zpub `04B24746`,
  Zpub `02AA7ED3` → xpub-mainnet; upub `044A5262`, Upub `024289EF`, vpub `045F1CF6`,
  Vpub `02575483` → tpub-testnet). Predicate is hardened-component (`ChildNumber::Hardened`,
  `slip132.rs:60–71`), capital-Y/Z gated on `m/48'` + index 1'/2' (matches SLIP-0132 multisig
  semantics, web-confirmed). Encode always normalizes+checks path
  (`encode.rs:85` → `parse_xpub_normalized(_, Some(&path))`); verify is path-OPTIONAL
  (`verify.rs:60` → `want_path.as_ref()`, single-parse). Mismatch = `UsageError` → exit 64
  (`mod.rs` `parse_xpub_normalized`). Private-key SLIP-0132 prefixes (yprv/zprv/…) are absent from
  the match table → fall through to `Xpub::from_str` and are rejected (no mis-normalization into the
  watch-only path). **mk-codec genuinely untouched** — no file under `crates/mk-codec/` in the mk diff
  (verified `git diff --name-only`). No wire change.
- **Manual prose — ✓ (modulo C1).** The new `### SLIP-0132 prefix acceptance (--xpub)` subsection
  (`44-mk-cli.md:72–94`) is accurate: the quoted note at line 85 is byte-exact to the `eprintln!`
  in `cmd/mod.rs`; the multisig/script-type mapping and exit-64 refusal description match the code.
  The verify cross-reference (lines 349–354) resolves: link `#slip-0132-prefix-acceptance---xpub`
  is the correct GFM slug of the heading. **cspell = 0 issues; markdownlint = 0 errors** on the file
  (SLIP/ypub/zpub/upub/vpub already in `.cspell.json`). No CI-gated transcript drifted (no transcript
  exercises this path; verify-examples set unchanged). The install command staleness is C1.
- **No GUI lockstep — ✓.** No clap `Subcommand`/`Args`/`ValueEnum`/`value_parser` change in the mk
  diff (the only new `#[derive]` is on the internal `Slip132Variant` enum). `--xpub` is a plain
  `#[arg(long)] pub xpub: String` (free text, not a dropdown); the accepted-values widening is purely
  runtime inside `parse_xpub_normalized`, invisible to clap-derived `gui-schema`. mk-cli `gui_schema`
  tests pass 8/8. No `mnemonic-gui` change in either diff. schema_mirror not implicated.
- **Audit trail committed — ✓.** mk HEAD tracks `design/SPEC_mk_slip0132_acceptance.md`,
  `design/IMPLEMENTATION_PLAN_mk_slip0132_acceptance.md`, and the 4 reviews
  (`mk-slip0132-spec-R0`, `-plan-R0`, `-plan-R1`, `-phase-A-R0`); `git log --all` for spec-R0 is
  non-empty (`dfdf111`). FOLLOWUP `mk-slip0132-prefix-acceptance` marked `resolved 24ba2c7`;
  `mk-slip0132-byte-parity-test-self-referential` filed `open` with accurate analysis. No dangling
  cross-citation (entry correctly states mk-only / no GUI / no manual companion).
- **Per-crate green — ✓.** `cargo test -p mk-cli` = **73 passed / 0 failed** (incl. the 8
  `cli_slip132` integration cells + the 5 `slip132.rs` unit cells). `cargo clippy -p mk-cli
  --all-targets -- -D warnings` = clean. `cargo test -p mnemonic-toolkit` = **2576 passed /
  0 failed**. `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` = clean.
- **SemVer — ✓.** mk-cli `0.6.1 → 0.7.0` MINOR (additive: widened `--xpub` accepted inputs; new
  module; no removal, no behavior change for canonical xpub which still passes through with no note —
  `encode_canonical_xpub_no_note` asserts this). toolkit `0.38.3 → 0.38.4` PATCH (lockstep re-pin +
  manual prose; zero `.rs` source touched — verified `git diff --name-only | grep '\.rs$'` = empty).
- **Scope / no-regression — ✓.** mk diff touches only mk-cli source/tests + design docs (mk-codec
  untouched). toolkit diff touches only pins, version markers, CHANGELOG, and the one manual chapter
  (no source). Existing mk encode/verify/decode/address/derive/repair behavior intact (round_trip +
  gui_schema + version_help suites green). Phase-2 output-class advisory intact: `output_advisory`
  module unchanged, still wired into encode (`encode.rs:97`); the new
  `encode_emits_both_slip132_note_and_watchonly_advisory` cell asserts the SLIP-0132 note precedes
  the watch-only advisory and both fire — confirming no regression of the Phase-2 advisory.

---

## Ship-readiness

**BLOCKED on C1.** Required before tag:

1. **Fix C1:** edit `mnemonic-toolkit/docs/manual/src/40-cli-reference/44-mk-cli.md:12` →
   `mk-cli-v0.7.0`; re-run cspell + markdownlint on the file (expected: still 0/0). This is a
   docs-only one-liner — no version re-bump needed (the v0.38.4 PATCH already covers manual prose),
   no feature re-review, no GREEN-gate re-dispatch of the implementation (the code is sound).
   Re-run this end-of-cycle R0 only to confirm C1 closed.
2. (Recommended, may defer via FOLLOWUP) Extend the sibling-pin-drift gate to also scan
   `docs/manual/src/**` + `docs/quickstart/**` prose install commands, so this class can't drift a
   third time.

Once C1 is closed, what remains is purely the user-gated mechanical release sequence:
- mk: tag `mk-cli-v0.7.0` at the mk `mk-slip0132-acceptance` HEAD (after merge to its default branch),
  push, publish (mk-cli IS on crates.io per prior cycles).
- toolkit: merge `mk-slip0132-acceptance` → master, tag `mnemonic-toolkit-v0.38.4`, push (toolkit is
  tag-only, NOT on crates.io).
- No GUI lockstep, no md-cli/ms-cli companion.

Both per-crate suites green, both clippy clean, byte-parity verified, scope clean, audit trail
persisted. The cycle is one docs line away from GREEN.
