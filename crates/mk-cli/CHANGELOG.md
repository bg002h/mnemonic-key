# Changelog

All notable changes to the `mk-cli` crate (the standalone `mk` binary) are
documented here. `mk-cli` versions independently of the `mk-codec` library; this
file is the source of truth for `mk-cli` release notes.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
