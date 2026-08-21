//! `mk gui-schema` — emit a machine-readable JSON description of the CLI
//! surface for consumption by `mnemonic-gui`'s schema-mirror gate.
//!
//! Realizes Section C.2 of the `mnemonic-gui` v0.2 plan (per the cross-repo
//! `mnemonic-gui-schema-mirror` FOLLOWUPS entry). The JSON contract is the
//! SPEC §7 shape shared across all four sibling CLIs (`md`, `ms`, `mk`,
//! `mnemonic`):
//!
//! ```json
//! {
//!   "version": 1,
//!   "cli": "mk",
//!   "subcommands": [
//!     { "name": "encode",
//!       "flags": [ { "name": "--xpub", "required": true, "kind": "text", "choices": null }, ... ],
//!       "positionals": [ { "name": "...", "required": ..., "repeating": ... } ] },
//!     ...
//!   ]
//! }
//! ```
//!
//! `kind` is one of `"text"`, `"boolean"`, `"number"`, `"dropdown"`,
//! `"path"`. `choices` is non-null only for `"dropdown"`. Complex types map
//! to `"text"`. The `gui-schema` subcommand itself is excluded from the
//! emitted list (the GUI never invokes it as a user-facing form).

use clap::{Arg, ArgAction, Command, CommandFactory, ValueHint};
use serde::Serialize;
use serde_json::json;

use crate::error::{CliError, Result};

/// `mk gui-schema` arguments — zero-argument subcommand.
#[derive(clap::Args, Debug)]
pub struct GuiSchemaArgs {}

/// Run `mk gui-schema`.
pub fn run(_args: GuiSchemaArgs) -> Result<u8> {
    let cmd = crate::Cli::command();
    let schema = build_schema(&cmd);
    let s = serde_json::to_string(&schema)
        .map_err(|e| CliError::UsageError(format!("gui-schema serialize: {e}")))?;
    println!("{s}");
    Ok(0)
}

#[derive(Serialize)]
struct Subcommand {
    name: String,
    flags: Vec<Flag>,
    positionals: Vec<Positional>,
}

#[derive(Serialize)]
struct Flag {
    name: String,
    required: bool,
    kind: &'static str,
    choices: Option<Vec<String>>,
}

#[derive(Serialize)]
struct Positional {
    name: String,
    required: bool,
    repeating: bool,
}

fn build_schema(cmd: &Command) -> serde_json::Value {
    let mut subs: Vec<Subcommand> = Vec::new();
    for sub in cmd.get_subcommands() {
        let name = sub.get_name().to_string();
        // Skip auto-generated `help` and the `gui-schema` subcommand itself.
        if name == "help" || name == "gui-schema" {
            continue;
        }
        let mut flags: Vec<Flag> = Vec::new();
        let mut positionals: Vec<Positional> = Vec::new();
        for arg in sub.get_arguments() {
            if arg.is_positional() {
                positionals.push(positional_from_arg(arg));
            } else {
                // Skip help/version auto-flags.
                let id = arg.get_id().as_str();
                if id == "help" || id == "version" {
                    continue;
                }
                if CLI_ONLY_FLAGS.contains(&(name.as_str(), id)) {
                    continue;
                }
                flags.push(flag_from_arg(&name, arg));
            }
        }
        subs.push(Subcommand {
            name,
            flags,
            positionals,
        });
    }
    json!({
        "version": 1,
        "cli": "mk",
        "subcommands": subs,
    })
}

/// Flags that exist on the CLI but are deliberately NOT part of the GUI
/// contract.
///
/// `mk gui-schema` describes the form `mnemonic-gui` RENDERS, not the CLI's
/// whole surface. The GUI's encode form mints ONE card from one xpub; `--keys`
/// mints N from a file and has no sensible form in that shape (a file picker
/// plus an N-card result view is a different screen). Emitting it would tell
/// the GUI to render a control it does not implement.
const CLI_ONLY_FLAGS: &[(&str, &str)] = &[("encode", "keys")];

/// Flags the GUI form requires, even though clap can no longer mark them
/// required because a CLI-only flag above relaxes them.
///
/// `--xpub` and `--origin-path` are `required_unless_present("keys")`, so
/// `Arg::is_required_set()` reports false for both. Within the single-card
/// form the GUI renders, they ARE required, and `mnemonic-gui`'s hand-written
/// mirror (`src/schema/mk.rs`) says so. Without this the schema would flip
/// them to optional and silently desync a repo that has no automated gate
/// against this one.
const REQUIRED_IN_GUI_FORM: &[(&str, &str)] = &[("encode", "xpub"), ("encode", "origin_path")];

fn flag_from_arg(sub: &str, arg: &Arg) -> Flag {
    let long = arg
        .get_long()
        .map(|s| format!("--{s}"))
        .unwrap_or_else(|| format!("--{}", arg.get_id().as_str().replace('_', "-")));
    let id = arg.get_id().as_str();
    let required = arg.is_required_set() || REQUIRED_IN_GUI_FORM.contains(&(sub, id));
    let (kind, choices) = classify(arg);
    Flag {
        name: long,
        required,
        kind,
        choices,
    }
}

fn positional_from_arg(arg: &Arg) -> Positional {
    let name = arg.get_id().as_str().to_string();
    let required = arg.is_required_set();
    let repeating = matches!(arg.get_action(), ArgAction::Append | ArgAction::Count)
        || arg.get_num_args().is_some_and(|n| n.max_values() > 1);
    Positional {
        name,
        required,
        repeating,
    }
}

/// Map a clap `Arg` to the SPEC §7 `kind` + optional `choices`.
///
/// Order of resolution:
/// 1. Boolean action (`SetTrue` / `SetFalse`) ⇒ `"boolean"`.
/// 2. `PossibleValuesParser`-derived choices ⇒ `"dropdown"`.
/// 3. `ValueHint` of `FilePath` / `DirPath` / `AnyPath` / `ExecutablePath` ⇒ `"path"`.
/// 4. Otherwise ⇒ `"text"`. Numeric flags (none in mk-cli today) would map here too;
///    callers needing `"number"` should add a `value_parser!(u32)` and we can extend.
fn classify(arg: &Arg) -> (&'static str, Option<Vec<String>>) {
    if matches!(arg.get_action(), ArgAction::SetTrue | ArgAction::SetFalse) {
        return ("boolean", None);
    }
    let choices: Vec<String> = arg
        .get_possible_values()
        .iter()
        .map(|pv| pv.get_name().to_string())
        .collect();
    if !choices.is_empty() {
        return ("dropdown", Some(choices));
    }
    match arg.get_value_hint() {
        ValueHint::FilePath
        | ValueHint::DirPath
        | ValueHint::AnyPath
        | ValueHint::ExecutablePath => return ("path", None),
        _ => {}
    }
    // `--out` on `vectors` is a `PathBuf` but lacks an explicit ValueHint.
    // Treat any arg whose value-parser type-id matches `PathBuf` as a path.
    if arg.get_value_parser().type_id() == std::any::TypeId::of::<std::path::PathBuf>() {
        return ("path", None);
    }
    ("text", None)
}
