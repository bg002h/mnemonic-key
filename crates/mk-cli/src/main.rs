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
mod exit;
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
    about = "mk — engrave-friendly Bitcoin xpub backups (mk1 format)",
    after_help = exit::HELP_LEGEND
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Encode an xpub + origin metadata into one or more mk1 strings.
    #[command(
        after_help = "Exit status: 0 = encoded. 1 = a key contradicts its --from-md1 policy (not a cosigner, or an origin that disagrees with its slot). 2 = an input is invalid. 8 = refused (the output is one of the inputs). 32 = an input file or the output failed. 64 = bad command line."
    )]
    Encode(cmd::encode::EncodeArgs),
    /// Decode mk1 strings back to xpub + origin metadata.
    #[command(
        after_help = "Exit status: 0 = decoded. 2 = an input is invalid, or the card fails the key-origin checks mk verify makes (it is still printed). 16 is added when a BCH correction was applied (so 18 = corrected, then invalid). 32 is added when an input file or stdout failed. 64 = bad command line."
    )]
    Decode(cmd::decode::DecodeArgs),
    /// Inspect mk1 strings: structural commentary in addition to decode.
    #[command(
        after_help = "Exit status: 0 = reported. 2 = an input is invalid, or the card fails the key-origin checks mk verify makes (the report is still printed). 16 is added when a BCH correction was applied (so 18 = corrected, then invalid). 32 is added when an input file or stdout failed. 64 = bad command line."
    )]
    Inspect(cmd::inspect::InspectArgs),
    /// Verify mk1 strings (BCH check + optional content match).
    #[command(
        after_help = "Exit status: 0 = every check passed. 1 = a check said no. 2 = an input is invalid. 16 is added when a BCH correction was applied (16 = passed after a correction, 17 = a check said no, 18 = invalid). 32 is added when an input file or stdout failed. 64 = bad command line."
    )]
    Verify(cmd::verify::VerifyArgs),
    /// Print the SHA-pinned v0.1 test-vector corpus as JSON.
    #[command(
        after_help = "Exit status: 0 = printed. 32 = the output could not be written. 64 = bad command line."
    )]
    Vectors(cmd::vectors::VectorsArgs),
    /// Emit machine-readable JSON describing the CLI surface (for mnemonic-gui).
    #[command(
        after_help = "Exit status: 0 = printed. 32 = the output could not be written. 64 = bad command line."
    )]
    GuiSchema(cmd::gui_schema::GuiSchemaArgs),
    /// Repair mk1 strings via BCH error correction (exit 16 = a correction was applied).
    #[command(
        after_help = "Exit status: 0 = nothing to correct. 16 = a correction was applied (verified by reassembly, or verifiable once the rest of a short chunk set is given). 22 = a corrected string that is not a card (16 + 4 verify before trusting + 2 invalid; it is still printed). 2 = an input is invalid: uncorrectable, not one card's chunk set (chunks under different chunk-set ids that together cover every index of one set are refused as a suspected miscorrection or two cards mixed), or a card that does not decode (nothing printed); a printed card that fails the key-origin checks is 2, or 18 after a correction. 32 is added when an input file or stdout failed. 64 = bad command line."
    )]
    Repair(cmd::repair::RepairArgs),
    /// Render receive/change addresses controlled by a card's xpub (read-only).
    #[command(
        after_help = "Exit status: 0 = printed. 2 = an input is invalid (a multisig, fixed-key or invalid card, or an address type that cannot be inferred). 8 = refused (--network disagrees with the xpub). 16 is added when a BCH correction was applied (so 18 or 24 after a later failure). 32 is added when an input file or stdout failed. 64 = bad command line."
    )]
    Address(cmd::address::AddressArgs),
    /// Derive a child xpub at a relative unhardened path from a card (read-only).
    #[command(
        after_help = "Exit status: 0 = printed. 2 = an input is invalid (a fixed-key or invalid card, or a path the key cannot derive). 16 is added when a BCH correction was applied (so 18 after a later failure). 32 is added when an input file or stdout failed. 64 = bad command line."
    )]
    Derive(cmd::derive::DeriveArgs),
    /// Generate roff man pages for the CLI into a directory (`--out <DIR>`).
    #[command(
        after_help = "Exit status: 0 = written. 32 = the directory could not be written. 64 = bad command line."
    )]
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
            // catch-all 64 (USAGE) below is every real parse error: mk takes
            // no input VALUE through a clap value parser (every card, key,
            // fingerprint and path arrives as a string and is judged by the
            // verb), so no clap error here is an input error (spec decision 2).
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
                _ => ExitCode::from(exit::USAGE),
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

    // Spec R6/R1: a verb that used a BCH-corrected value adds REPAIRED to
    // whatever else it reports (17 = a verify mismatch on a corrected card).
    let repaired = if cmd::correction_used() {
        exit::REPAIRED
    } else {
        0
    };
    let code = match result {
        Ok(code) => exit::add(code, repaired),
        Err(e) => {
            let code = exit::add(e.exit_code(), repaired);
            emit_error(&e, json_mode, code);
            code
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

fn emit_error(e: &CliError, json_mode: bool, code: u8) {
    if json_mode {
        // JSON-mode errors go to stdout (one stream) -- a CLI convention pinned by
        // tests; the format SPEC has no section on it (see error.rs). The
        // envelope carries the process's exit status (spec R14).
        println!("{}", e.json_envelope(code));
    } else {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{e}").ok();
    }
}
