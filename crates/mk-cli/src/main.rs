//! `mk` — engrave-friendly Bitcoin xpub backups (the `mk1` format).
//!
//! Companion CLI to the `mk-codec` library. See `docs/MK_CODEC_RUST_API.md`
//! for the Rust API reference and the `mnemonic-toolkit` user manual chapter
//! `docs/manual/src/40-cli-reference/44-mk-cli.md` for the canonical surface.

#![allow(missing_docs)] // mk-cli is binary-only; mirror md-cli/ms-cli precedent.

mod cmd;
mod error;

use std::io::Write;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use serde_json::json;

use error::{CliError, Result};

#[derive(Parser, Debug)]
#[command(
    name = "mk",
    version,
    about = "mk — engrave-friendly Bitcoin xpub backups (mk1 format)"
)]
struct Cli {
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
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // Clap usage error: print to stderr (clap's default), exit 64.
            e.print().ok();
            return ExitCode::from(64);
        }
    };

    let json_mode = is_json_mode(&cli.command);

    let result: Result<()> = match cli.command {
        Command::Encode(a) => cmd::encode::run(a),
        Command::Decode(a) => cmd::decode::run(a),
        Command::Inspect(a) => cmd::inspect::run(a),
        Command::Verify(a) => cmd::verify::run(a),
        Command::Vectors(a) => cmd::vectors::run(a),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            emit_error(&e, json_mode);
            ExitCode::from(e.exit_code())
        }
    }
}

fn is_json_mode(cmd: &Command) -> bool {
    match cmd {
        Command::Encode(a) => a.json,
        Command::Decode(a) => a.json,
        Command::Inspect(a) => a.json,
        Command::Verify(a) => a.json,
        Command::Vectors(_) => false,
    }
}

fn emit_error(e: &CliError, json_mode: bool) {
    if json_mode {
        // JSON-mode errors go to stdout (one stream) per SPEC §3.5.6.
        let envelope = json!({
            "schema_version": 1,
            "error": {
                "kind": e.kind(),
                "message": e.message(),
                "exit_code": e.exit_code(),
                "details": e.details(),
            },
        });
        let s = serde_json::to_string(&envelope).expect("error envelope serializes");
        println!("{s}");
    } else {
        let mut stderr = std::io::stderr().lock();
        writeln!(stderr, "{e}").ok();
    }
}
