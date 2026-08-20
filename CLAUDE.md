# CLAUDE.md — mk1 (`mnemonic-key`) repo notes for Claude Code sessions

This file is auto-loaded by Claude Code when starting a session in this repository.

## Project at a glance

`mk1` is a Bitcoin BIP-style codex32-derived backup format for individual extended public keys (xpubs). HRP `mk`, designed to engrave alongside `md1` (sibling repo `bg002h/descriptor-mnemonic` at `/scratch/code/shibboleth/descriptor-mnemonic`) for foreign-xpub multisig recovery. As of 2026-05-03 a third sibling format `ms1` (HRP `ms`, repo `bg002h/mnemonic-secret`) is in design — secret material (BIP-39 entropy / BIP-32 master seed / xpriv) using BIP-93 codex32 directly via `rust-codex32`. The three formats engrave together as a coherent backup bundle (md1 = template, mk1 = xpubs, ms1 = secret).

**Wire format is locked at v0.1.** Q-1..Q-10 closures landed 2026-04-29 in `docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md` (an opus-reviewed closure-design pass). Implementation work tracked in `design/IMPLEMENTATION_PLAN_mk_v0_1.md` on branch `feature/v0.1.0-implementation`.

## Active work

- **Branch:** `feature/v0.1.0-implementation`
- **Plan:** [`design/IMPLEMENTATION_PLAN_mk_v0_1.md`](design/IMPLEMENTATION_PLAN_mk_v0_1.md) (8 phases)
- **Closure design (locks):** [`docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md`](docs/superpowers/specs/2026-04-29-mk1-open-questions-closure-design.md)
- **SPEC:** [`design/SPEC_mk_v0_1.md`](design/SPEC_mk_v0_1.md) — wire-format spec, post-closure
- **DECISIONS:** [`design/DECISIONS.md`](design/DECISIONS.md) — D-1..D-15, Q-1..Q-10 closed
- **BIP draft:** [`bip/bip-mnemonic-key.mediawiki`](bip/bip-mnemonic-key.mediawiki) — content-complete, pre-submission
- **FOLLOWUPS:** [`design/FOLLOWUPS.md`](design/FOLLOWUPS.md) — pre-bip-submission audit gates + cross-repo items

Phases 1–4 shipped (commits `e9e9afc..32f273e` on the feature branch). Phases 5–8 remaining: string layer (BCH fork from md-codec), vector corpus, release plumbing, reconciliation. See the plan for per-phase task breakdown.

## Workflow conventions

- **Default to ultracode (multi-agent orchestration) — refined policy** (2026-06-17, after an architect panel; verdict: keep default-ON, refine per-phase). Standing user directive, project-wide across the m-format constellation + seedhammer fork; does NOT require the per-turn `ultracode` keyword. **Default ON for every *substantial* task; token cost is not a constraint.** Trivial one-line/mechanical edits, version bumps, and plain Q&A run solo. **Per-phase pattern:** (1) **research/recon** — fan out parallel subagents; any agent handling **external protocol facts** (BIP-39, BCH/codec semantics, NDEF, RP2350 OTP, SDK behavior) MUST verify them against **authoritative source text**, not just the draft doc (guards against false-consensus on plausible-but-wrong facts — the "1 valid last word" class). (2) **design/spec/plan** — single author + the mandatory R0 loop. (3) **implementation** — a *single* subagent executes the GREEN plan in a worktree (NOT parallel re-implementations); TDD. (4) **post-implementation** — a **mandatory, non-deferrable** independent adversarial execution review over the whole diff (R0 = plan correctness; this catches implementation-introduced regressions TDD misses). (5) if Agent-API dispatch fails mid-session, **flag it explicitly** and defer the formal review to API recovery — never silently substitute inline self-review. Composes with — does not replace — the R0 gate; verbatim agent reports persist to `design/agent-reports/`.

The user established this workflow on md1 v0.6 / v0.7 and explicitly asked mk1 to follow it:

1. **Per-phase opus review.** After each phase commit, dispatch a `superpowers:code-reviewer` subagent with `model: opus` to verify the work. Brief the agent like a colleague who didn't see the design conversation; cite the specific files + spec sections to cross-check.
2. **Save reports to disk.** Every reviewer subagent MUST save its full report to `design/agent-reports/v0-1-phase-<P>-review-<commit>.md` (the user has explicitly required this; if a subagent skips the disk-save step, save it manually). File-naming convention: `design/agent-reports/README.md`.
3. **Apply critical / important findings inline.** Should-address items get fixed before moving to the next phase, in a follow-up commit titled e.g. `style/fix(mk-codec phase N): apply Phase N review fixes (commit <SHA> review)`.
4. **Collect deferred items in `design/FOLLOWUPS.md`.** Anything not fixed inline goes in FOLLOWUPS at the appropriate tier.
5. **Stop only on real design questions.** Otherwise work independently — the user prefers per-phase autonomous progress with status updates at natural break points, not turn-by-turn confirmation.
6. **Test discipline (Phases 3–6).** Tests land before impl within each task; `#[ignore]`-marked scaffolds in earlier phases get un-ignored in the phase that lands the corresponding code path.
7. **Parallel tool calls.** When making multiple non-conflicting operations (file edits to different sections, independent reads, independent writes), batch them in a single response — don't sequence them. (Captured in auto-memory `feedback_parallel_tool_calls.md`.)
8. **Per-phase commit cadence.** Each phase produces a feature commit + a fixup commit (after review). Phase 4 was the exception with a third nit-cleanup commit; that's fine when nits are genuinely deferred from the main fixup pass.

## Cross-repo coordination with md1 (`descriptor-mnemonic`)

`md1` is the sibling format. mk1 references md1 at the BIP level (path-dictionary mirror, chunked-header structure, Wallet Instance ID construction) but does NOT depend on md-codec as a Rust crate (D-13 fork-now-refactor-later). The previously-planned `mc-codex32` shared-crate extraction (closure Q-9: "both formats v1.0 with cross-validated conformance vectors") is **RETIRED as of 2026-05-03**: ms1 adopts BIP-93 codex32 directly via `rust-codex32` and md1↔mk1's HRP-mixed BCH with per-format target residues isn't upstreamable to that crate, so there is no shared code worth extracting. The cross-repo *pattern* (HRP-mixed BCH + per-format target residue) will be documented in a future cross-repo `PATTERNS.md`; the BCH primitives stay forked between md1 and mk1 indefinitely. See `design/FOLLOWUPS.md` entry `mc-codex32-extraction-retired-2026-05-03` for the full record.

When mk1 work surfaces an action item that affects md1, follow the established mirror pattern:

- The primary entry lives in mk1's `design/FOLLOWUPS.md` at tier `cross-repo`.
- A companion entry is mirrored into `descriptor-mnemonic/design/FOLLOWUPS.md` so md-codec sessions discover the action item natively.
- Both entries cite each other (`Companion:` line in each).
- Renames or wire-format changes that gate mk1's BIP submission (e.g., `chunk-set-id-rename`) carry an explicit `Sequencing requirement:` line.

Currently open mk1-surfaced items affecting md1: see `design/FOLLOWUPS.md` (tier `cross-repo`).

## Manual coverage

The end-user manual for the m-format star lives in the sibling `bg002h/mnemonic-toolkit` repo at `docs/manual/`. v0.2 of the manual mirrors the `mk-cli` flag surface under `docs/manual/src/40-cli-reference/44-mk-cli.md`. The Rust API reference for `mk-codec` lives at `docs/MK_CODEC_RUST_API.md` in this repo + on docs.rs once mk-codec hits crates.io. **Any change to mk-cli's flag surface must update the manual chapter in lockstep with the implementing PR** (per the cross-repo `manual-cli-surface-mirror` invariant in `mnemonic-toolkit/design/FOLLOWUPS.md`). The same invariant covers `mk-codec` Rust API changes (new `pub` items, removed re-exports, signature changes, `#[non_exhaustive]` field additions): update both `docs/MK_CODEC_RUST_API.md` here and the manual side in lockstep. See `design/FOLLOWUPS.md` entry `manual-cli-surface-mirror` for the canonical record; primary entry lives in the toolkit repo.

## Practical tips

- `cargo test -p mk-codec` runs all tests. As of `32f273e`, 46 unit tests pass + 4 ignored scaffolds for Phase 5 work.
- Formatting is gated by CI's dedicated `fmt` job pinned to **1.95.0** (the canonical fmt toolchain — see `.github/workflows/ci.yml`). Format locally with `cargo +1.95.0 fmt --all` (NOT the floating local default, which may be a newer nightly that formats differently). When bumping the pin, re-run the fmt in the same commit. Do NOT add a rust-toolchain.toml (it would hijack the 3-toolchain CI matrix — rustup file-beats-default precedence).
- The repo has its own `Cargo.toml` workspace at `crates/mk-codec`. md-codec is in a separate repo and a separate workspace.
- Bitcoin crate is `bitcoin = "0.32"`. `DerivationPath`'s `Display` does NOT include the `m/` prefix; use structural comparison (`path.parse::<DerivationPath>() == other`) rather than string formatting when matching paths against the standard table.

## Pushing `main` — agents stage, the maintainer may not have to

**A required status check binds to a COMMIT SHA, not a branch.** So a commit
pushed straight to `main` carries no check when branch protection is evaluated:
it reports "expected", and the push is **bypassed** rather than satisfied. That
is a chicken-and-egg in the rule, not a lapse. `strict: false` is what makes it
fixable — GitHub asks only whether the commit carries a *passing context*, so let
the SHA earn one first:

```sh
git push origin main:refs/heads/ci/staging      # builds this exact SHA
gh run watch <id> --repo bg002h/mnemonic-key    # wait for the context below
git push origin main                            # no bypass message = satisfied
git push origin --delete ci/staging
```

**The required context here is `build (stable on ubuntu-latest)`** — one cell of
the 3×3 build matrix. It is NOT `test (rust + go)`; that is the sibling
`mnemonic-engrave` repo's context and copying its block here would wait forever
on a check that never reports. `.github/workflows/ci.yml` builds `ci/**` for this
purpose, and `release-on-tag` is gated on tag refs, so a `ci/**` push can never
publish.

**The asymmetry is deliberate** (ruled 2026-08-15: *"You are not permitted to
bypass, but I am."*). `enforce_admins` is `false` here on purpose — it is the
maintainer's own escape hatch, and **it is not to be flipped**. The no-bypass
rule binds **automation**: an agent uses the staging path above every time, and
reports a "Bypassed rule violations" message as a failure rather than papering
over it. Without the `ci/**` trigger an agent would have no compliant way to push
here at all, which is why that trigger exists.

## Parallel execution — this machine has 24 CPU cores

**Standing directive (2026-08-19): consider parallel execution for ALL tests,
cache generation and long calculations.** The defaults use almost none of the
box. Measured constellation-wide the same day: **824s → 204s (~4×)**.

- **Rust — `cargo nextest run --locked`**, not `cargo test`. `cargo test` runs
  each test *binary* serially; nextest spreads them over all cores. Per-repo
  measurements: mnemonic-toolkit 256s→49s, descriptor-mnemonic 40s→27s,
  mnemonic-engrave 33s→16s, mnemonic-secret 2s→0.3s. `cargo-nextest` 0.9.140 is
  installed.
- **Go — shard the package.** `-parallel` does NOTHING unless tests call
  `t.Parallel()`; the fork's `gui` package has 886 test funcs and zero of them.
  `mnemonic-engrave/scripts/gui-shard-test.sh <pkg> 24` took `./gui/` from 493s
  to 112s. It enumerates its partition from `go test -list` and **asserts the
  union is exhaustive before running**, so it cannot silently drop a test — any
  replacement must do the same.
- **Long independent work** — cache/corpus generation, fixture derivation, batch
  rendering — is a candidate too. Ask whether it is CPU-bound and independent
  before running it in a loop.

**Two measured cautions.** `--release` is ~32× faster at test *execution*
(25.4s → 0.775s on one workspace) but drops `debug_assertions`; suites relying
on overflow checks or assertion panics — mutation tests especially — stop
detecting things. Use it for iteration, never as the gate. And check what
`/tmp` is before building there: on this box it is a 32 GB tmpfs, and a scratch
worktree's `target/` filled it and killed a running test.

**Never run the same suite twice** to collect counts and failures separately.
Capture once to a file, then grep it — otherwise every measurement costs double.
