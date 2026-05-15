//! `mk vectors` — emit the SHA-pinned v0.1 mk-codec test-vector corpus.
//!
//! Realizes SPEC §3.5.5. The corpus JSON is re-exported from
//! `mk_codec::test_vectors::V0_1_JSON` (since mk-codec 0.3.0 / mk-cli
//! 0.3.2); the canonical file lives at `crates/mk-codec/src/test_vectors/v0.1.json`
//! and is `include_str!`-baked into mk-codec at compile time. Runtime
//! requires no fixture-path access — `cargo install mk-cli` produces a
//! fully self-contained binary.

use std::path::PathBuf;

use clap::Args;
use serde_json::Value;

use crate::error::{CliError, Result};

const VECTORS_V0_1_JSON: &str = mk_codec::test_vectors::V0_1_JSON;

/// `mk vectors` arguments.
#[derive(Args, Debug)]
pub struct VectorsArgs {
    /// Indent the JSON output for human readability. Ignored when `--out` is supplied.
    #[arg(long)]
    pub pretty: bool,

    /// Optional output directory. When set, writes one `<name>.json` file per
    /// fixture in the corpus's `vectors` array instead of emitting to stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,
}

/// Run `mk vectors`.
pub fn run(args: VectorsArgs) -> Result<()> {
    if let Some(dir) = args.out {
        return write_per_fixture_files(dir, args.pretty);
    }

    if args.pretty {
        let parsed: Value = serde_json::from_str(VECTORS_V0_1_JSON)
            .map_err(|e| CliError::UsageError(format!("vector corpus parse: {e}")))?;
        let pretty = serde_json::to_string_pretty(&parsed)
            .map_err(|e| CliError::UsageError(format!("vector corpus serialize: {e}")))?;
        println!("{pretty}");
    } else {
        print!("{VECTORS_V0_1_JSON}");
        if !VECTORS_V0_1_JSON.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}

fn write_per_fixture_files(dir: PathBuf, pretty: bool) -> Result<()> {
    std::fs::create_dir_all(&dir)?;
    let parsed: Value = serde_json::from_str(VECTORS_V0_1_JSON)
        .map_err(|e| CliError::UsageError(format!("vector corpus parse: {e}")))?;
    let vectors = parsed
        .get("vectors")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            CliError::UsageError("vector corpus missing top-level `vectors` array".into())
        })?;

    let mut written = 0usize;
    for v in vectors {
        let name = v
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| CliError::UsageError("vector entry missing `name` field".into()))?;
        let body = if pretty {
            serde_json::to_string_pretty(v)
        } else {
            serde_json::to_string(v)
        }
        .map_err(|e| CliError::UsageError(format!("vector serialize: {e}")))?;
        let mut path = dir.clone();
        path.push(format!("{name}.json"));
        std::fs::write(&path, body)?;
        written += 1;
    }
    eprintln!("wrote {written} vector file(s) to {}", dir.display());
    Ok(())
}
