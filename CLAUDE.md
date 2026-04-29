# CLAUDE.md — mk1 (`mnemonic-key`) repo notes for Claude Code sessions

This file is auto-loaded by Claude Code when starting a session in this repository.

## Project at a glance

`mk1` is a Bitcoin BIP-style codex32-derived backup format for individual extended public keys (xpubs). HRP `mk`, designed to engrave alongside `md1` (sibling repo `bg002h/descriptor-mnemonic` at `/scratch/code/shibboleth/descriptor-mnemonic`) for foreign-xpub multisig recovery.

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

`md1` is the sibling format. mk1 references md1 at the BIP level (path-dictionary mirror, chunked-header structure, Wallet Instance ID construction) but does NOT depend on md-codec as a Rust crate (D-13 fork-now-refactor-later). The shared `mc-codex32` extraction trigger is "both formats v1.0 with cross-validated conformance vectors" (closure Q-9).

When mk1 work surfaces an action item that affects md1, follow the established mirror pattern:

- The primary entry lives in mk1's `design/FOLLOWUPS.md` at tier `cross-repo`.
- A companion entry is mirrored into `descriptor-mnemonic/design/FOLLOWUPS.md` so md-codec sessions discover the action item natively.
- Both entries cite each other (`Companion:` line in each).
- Renames or wire-format changes that gate mk1's BIP submission (e.g., `chunk-set-id-rename`) carry an explicit `Sequencing requirement:` line.

Currently open mk1-surfaced items affecting md1: see `design/FOLLOWUPS.md` (tier `cross-repo`).

## Practical tips

- `cargo test -p mk-codec` runs all tests. As of `32f273e`, 46 unit tests pass + 4 ignored scaffolds for Phase 5 work.
- `cargo clippy` and `cargo fmt` may be shimmed via a stale `~/.cargo/bin/rustup` looking for a nightly toolchain that isn't installed. System-installed clippy/fmt at `/usr/bin` work but aren't picked up by the shim. Workaround: skip clippy/fmt gates if the shim errors; the user is aware.
- The repo has its own `Cargo.toml` workspace at `crates/mk-codec`. md-codec is in a separate repo and a separate workspace.
- Bitcoin crate is `bitcoin = "0.32"`. `DerivationPath`'s `Display` does NOT include the `m/` prefix; use structural comparison (`path.parse::<DerivationPath>() == other`) rather than string formatting when matching paths against the standard table.
