//! Subcommand handlers for `mk-cli`. Each module is independent and consumes
//! the foundation modules + the `mk-codec` and `md-codec` libraries.

pub mod decode;
pub mod derive_support;
pub mod encode;
pub mod gui_schema;
pub mod inspect;
pub mod repair;
pub mod vectors;
pub mod verify;

use std::str::FromStr;

use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use bitcoin::hashes::{Hash, sha256};

use crate::error::{CliError, Result};

/// Parse a 4-byte hex `[u8; 4]` (8 hex chars) for a `policy_id_stub`.
pub fn parse_stub_hex(s: &str) -> Result<[u8; 4]> {
    let bytes = hex::decode(s)
        .map_err(|e| CliError::UsageError(format!("--policy-id-stub: invalid hex {s:?}: {e}")))?;
    if bytes.len() != 4 {
        return Err(CliError::UsageError(format!(
            "--policy-id-stub: expected 4 bytes (8 hex chars), got {} bytes ({s:?})",
            bytes.len()
        )));
    }
    Ok([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Parse `--origin-fingerprint` 8-hex-chars → `Fingerprint`.
pub fn parse_fingerprint(s: &str) -> Result<Fingerprint> {
    let bytes = hex::decode(s).map_err(|e| {
        CliError::UsageError(format!("--origin-fingerprint: invalid hex {s:?}: {e}"))
    })?;
    if bytes.len() != 4 {
        return Err(CliError::UsageError(format!(
            "--origin-fingerprint: expected 4 bytes (8 hex chars), got {} bytes",
            bytes.len()
        )));
    }
    Ok(Fingerprint::from([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Parse `--origin-path` (e.g., `m/48'/0'/0'/2'`) → `DerivationPath`.
pub fn parse_derivation_path(s: &str) -> Result<DerivationPath> {
    DerivationPath::from_str(s).map_err(|e| {
        CliError::UsageError(format!("--origin-path: invalid derivation path {s:?}: {e}"))
    })
}

/// Parse an xpub string to `Xpub`.
pub fn parse_xpub(s: &str) -> Result<Xpub> {
    Xpub::from_str(s).map_err(|e| CliError::UsageError(format!("invalid xpub {s:?}: {e}")))
}

/// Derive the 4-byte `policy_id_stub` from an md1 string per SPEC §3.5.1.
pub fn derive_stub_from_md1(md1_str: &str) -> Result<[u8; 4]> {
    let descriptor = md_codec::decode_md1_string(md1_str)?;
    let (bytecode_bytes, _bit_len) = md_codec::encode_payload(&descriptor)?;
    let hash = sha256::Hash::hash(&bytecode_bytes);
    let stub: [u8; 4] = hash.as_byte_array()[..4].try_into().expect("4-byte slice");
    Ok(stub)
}

/// Format `policy_id_stub` bytes as 8 lowercase hex chars.
pub fn fmt_stub(stub: &[u8; 4]) -> String {
    hex::encode(stub)
}

/// Format a `Fingerprint` as 8 lowercase hex chars.
pub fn fmt_fingerprint(fp: &Fingerprint) -> String {
    hex::encode(fp.to_bytes())
}

/// Read a list of mk1 strings: positional `args` minus a leading `"-"`,
/// which means "read one string per line from stdin." `"-"` may appear
/// as any positional value but is processed once across the list.
pub fn read_mk1_strings(args: &[String]) -> Result<Vec<String>> {
    let mut out = Vec::with_capacity(args.len());
    let mut consumed_stdin = false;
    for a in args {
        if a == "-" && !consumed_stdin {
            consumed_stdin = true;
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
            for line in buf.lines() {
                let s = line.trim();
                if !s.is_empty() {
                    out.push(s.to_string());
                }
            }
        } else if a == "-" {
            // Already consumed stdin; ignore additional `-` markers.
        } else {
            out.push(a.clone());
        }
    }
    if out.is_empty() {
        return Err(CliError::UsageError(
            "expected at least one mk1 string (positional or via stdin with '-')".into(),
        ));
    }
    Ok(out)
}

/// Detect the BCH code variant for a single mk1 string. Lengths from
/// `mk-codec::consts`: regular = 1+3+93 = 97 chars, long = 1+3+105 = 109.
/// Approximated by the data-part length alone since the same fields apply
/// to both single-string and chunked headers in v0.1.
pub fn classify_code_variant(s: &str) -> &'static str {
    // mk1 strings start with `mk1` (3 chars). The locked v0.1 lengths are
    // 90 (regular fragment + header + bch) and 108 (long fragment + header + bch),
    // but we only care about distinguishing the two variants for output.
    if s.len() <= 96 + "mk1".len() {
        "regular"
    } else {
        "long"
    }
}
