//! `mk vectors` — emit the SHA-pinned v0.1 mk-codec test-vector corpus.
//!
//! Re-exports the conformance corpus of SPEC §10 (Reference implementation).
//!
//! (Was a `SPEC §3.5.x` cite. The mk SPEC's §3.5 is "Origin path encoding"
//! and has no subsections -- that whole family of cites referred to nothing.
//! A 2026-06-10 audit repointed four of them; a 2026-08-21 review found a
//! fifth, and sweeping the class found eight. See FOLLOWUPS F-224.) The corpus JSON is re-exported from
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
    /// Indent the JSON output for human readability. Also applies to each
    /// per-fixture file when `--out` is supplied.
    #[arg(long)]
    pub pretty: bool,

    /// Optional output directory. When set, writes one `<name>.json` file per
    /// fixture in the corpus's `vectors` array instead of emitting to stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,
}

/// Run `mk vectors`.
pub fn run(args: VectorsArgs) -> Result<u8> {
    if let Some(dir) = args.out {
        return write_per_fixture_files(dir, args.pretty).map(|()| 0);
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
    Ok(0)
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
