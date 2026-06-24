//! `mk gen-man` — self-emit roff man pages from the compiled clap `Command`
//! tree.
//!
//! Writes one `*.1` page per (nested) subcommand into `--out <DIR>` via
//! `clap_mangen::generate_to`. The pages are clap-generated, hence
//! **binary-faithful by construction** — there is no content-fidelity gate
//! (the page cannot drift from the binary's actual flag surface).
//!
//! ## Mechanism (SPEC §2 / C-1)
//!
//! The call is the bare, naive form — `clap_mangen::generate_to(Cli::command(),
//! &dir)` with **NO pre-`.build()`**. Under clap_mangen 0.3 + clap 4.6.1 a
//! pre-`.build()` POISONS the output with a `help` pseudo-subcommand SHADOW
//! TREE (spurious `*-help*.1` pages). `generate_to` internally does
//! `disable_help_subcommand(true)` THEN `cmd.build()`; an external `root.build()`
//! run FIRST materializes the `help` subcommands before the internal disable
//! can suppress them. The naive call is clean.

use std::path::PathBuf;

use clap::CommandFactory;

use crate::error::Result;

/// `mk gen-man` arguments.
#[derive(clap::Args, Debug)]
pub struct GenManArgs {
    /// Directory to write `*.1` man pages into (created if absent).
    #[arg(long, value_name = "DIR")]
    pub out: PathBuf,
}

/// Run `mk gen-man`.
pub fn run(args: GenManArgs) -> Result<u8> {
    std::fs::create_dir_all(&args.out)?;
    // Bare, naive generate_to — NO pre-`.build()` (C-1). `Cli::command()` is the
    // UNBUILT tree; `generate_to` builds it internally with the help subcommand
    // disabled.
    clap_mangen::generate_to(crate::Cli::command(), &args.out)?;
    Ok(0)
}
