//! `mk` — engrave-friendly Bitcoin xpub backups (the `mk1` format).
//!
//! Companion CLI to the `mk-codec` library. See `docs/MK_CODEC_RUST_API.md`
//! for the Rust API reference and the `mnemonic-toolkit` user manual chapter
//! `docs/manual/src/40-cli-reference/44-mk-cli.md` for the canonical surface.

#![allow(missing_docs)] // mk-cli is binary-only; mirror md-cli/ms-cli precedent.

// First, so its print macros shadow std's in every module below.
#[macro_use]
mod out;

mod cmd;
mod error;
mod format;
mod keyfile;
mod output_advisory;
mod process_hardening;
mod slip132;

use std::io::Write;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use error::{CliError, Result};

#[derive(Parser, Debug)]
#[command(
    name = "mk",
    version,
    about = "mk — engrave-friendly Bitcoin xpub backups (mk1 format)"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Encode an xpub + origin metadata into one or more mk1 strings.
    Encode(cmd::encode::EncodeArgs),
    /// Decode mk1 strings back to xpub + origin metadata.
    Decode(cmd::decode::DecodeArgs),
    /// Inspect mk1 strings: structural commentary in addition to decode.
    Inspect(cmd::inspect::InspectArgs),
    /// Verify mk1 strings (BCH check + optional content match).
    Verify(cmd::verify::VerifyArgs),
    /// Print the SHA-pinned v0.1 test-vector corpus as JSON.
    Vectors(cmd::vectors::VectorsArgs),
    /// Emit machine-readable JSON describing the CLI surface (for mnemonic-gui).
    GuiSchema(cmd::gui_schema::GuiSchemaArgs),
    /// Repair mk1 strings via BCH error correction (exit 5 = REPAIR_APPLIED).
    Repair(cmd::repair::RepairArgs),
    /// Render receive/change addresses controlled by a card's xpub (read-only).
    Address(cmd::address::AddressArgs),
    /// Derive a child xpub at a relative unhardened path from a card (read-only).
    Derive(cmd::derive::DeriveArgs),
    /// Generate roff man pages for the CLI into a directory (`--out <DIR>`).
    GenMan(cmd::gen_man::GenManArgs),
}

fn main() -> ExitCode {
    // argv-hardening: deny other-UID /proc/$PID/cmdline reads + core dumps.
    process_hardening::set_non_dumpable();
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Clap returns Err for two non-error terminations: --version
            // (ErrorKind::DisplayVersion) and --help (ErrorKind::DisplayHelp).
            // Output is on stdout and the canonical Unix exit is 0. The
            // catch-all 64 below preserves the carve-out for real parse
            // errors.
            // --help and --version go to stdout under the same write-error
            // rule as the verbs (`out`); a parse error goes to stderr.
            if e.use_stderr() || !out::read_only() {
                if let Err(w) = e.print() {
                    if !e.use_stderr() {
                        out::failed(&w);
                    }
                }
            }
            return match e.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    ExitCode::from(out::exit_code(0))
                }
                _ => ExitCode::from(64),
            };
        }
    };

    let json_mode = is_json_mode(&cli.command);

    let result: Result<u8> = match cli.command {
        Command::Encode(a) => cmd::encode::run(a),
        Command::Decode(a) => cmd::decode::run(a),
        Command::Inspect(a) => cmd::inspect::run(a),
        Command::Verify(a) => cmd::verify::run(a),
        Command::Vectors(a) => cmd::vectors::run(a),
        Command::GuiSchema(a) => cmd::gui_schema::run(a),
        Command::Repair(a) => cmd::repair::run(a),
        Command::Address(a) => cmd::address::run(a),
        Command::Derive(a) => cmd::derive::run(a),
        Command::GenMan(a) => cmd::gen_man::run(a),
    };

    let code = match result {
        Ok(code) => code,
        Err(e) => {
            emit_error(&e, json_mode);
            e.exit_code()
        }
    };
    ExitCode::from(out::exit_code(code))
}

fn is_json_mode(cmd: &Command) -> bool {
    match cmd {
        Command::Encode(a) => a.json,
        Command::Decode(a) => a.json,
        Command::Inspect(a) => a.json,
        Command::Verify(a) => a.json,
        Command::Vectors(_) => false,
        Command::GuiSchema(_) => false,
        Command::Repair(a) => a.json,
        Command::Address(a) => a.json,
        Command::Derive(a) => a.json,
        Command::GenMan(_) => false,
    }
}

fn emit_error(e: &CliError, json_mode: bool) {
    if json_mode {
        // JSON-mode errors go to stdout (one stream) -- a CLI convention pinned by
        // tests; the format SPEC has no section on it (see error.rs).
        println!("{}", e.json_envelope(e.exit_code()));
    } else {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{e}").ok();
    }
}
