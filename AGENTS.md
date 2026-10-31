# mnemonic-key

Paths below are relative to this Git root, including when working inside a crate.

## Start here

Read [CLAUDE.md](CLAUDE.md) explicitly; AGENTS.md does not load it automatically.
Read applicable ancestor guidance too. Existing process rules remain applicable;
verify historical architecture/status statements against source.
Read the shared [guide](https://github.com/bg002h/mnemonic-toolkit/blob/a2e633b5384c6c61b3d9394379818b5ecd7d376d/docs/architecture/README.md) and [workflow](https://github.com/bg002h/mnemonic-toolkit/blob/a2e633b5384c6c61b3d9394379818b5ecd7d376d/docs/architecture/WORKFLOW.md).
For sibling checkouts/worktrees, prefer local `../mnemonic-toolkit/docs/architecture/`;
remote fallbacks pin a delivered guide revision. See its workflow for refreshing the pin.
Check Git status, the current branch, Cargo.toml and Cargo.lock before editing.
Preserve unrelated work and use explicit task-owned paths when staging.

## Where behavior lives

- [KeyCard](crates/mk-codec/src/key_card.rs) and its bytecode/string layers own public-key representation and recovery.
- [CLI](crates/mk-cli/src/main.rs) owns key-file intake, normalization, derivation and reporting.
- Preserve xpub/origin consistency in [compact encoding](crates/mk-codec/src/bytecode/xpub_compact.rs).
  A policy stub is not sufficient proof of every membership claim in a sparse descriptor.
- Inspect [chunk framing](crates/mk-codec/src/string_layer/chunk.rs) and the resolved
  consumer version before treating encoded strings as deterministic or exposing corrections.
- The CLI's md dependency may accept fewer wire versions than the sibling md checkout.
  Read Cargo.lock and the resolved decoder when changing `--from-md1` behavior.

## Contracts and checks

Read [base spec](design/SPEC_mk_v0_1.md), [empty origins](design/SPEC_mk_no_path_support.md),
and [chunk IDs](design/SPEC_chunk_set_id_verification.md).
Use codec vectors, external xpub oracle, chunk-ID and CLI verification tests;
CI and `ci/repro/vendor-freshness.sh` define current invocations.
The shared BCH extraction plan is retired; check the other implementations when fixing
a BCH defect. CLI changes also require the toolkit manual and utility GUI mirrors.
