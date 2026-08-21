//! Batch key-record input for `mk encode --keys` (F-223).
//!
//! `mk encode` mints ONE card per invocation, so an N-cosigner backup is N
//! invocations and every operator journey wraps it in a shell loop. This module
//! is the loop, moved inside the tool.
//!
//! **Record format is BIP-380 origin notation**, one per line:
//!
//! ```text
//! [73c5da0a/48'/0'/0'/2']xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB…
//! ```
//!
//! Chosen over three parallel repeatable flags (`--xpub` × N, `--origin-path`
//! × N, …) because parallel lists can desync, and a desync here does not fail
//! — it mints a card naming the WRONG master. The same reasoning put
//! `card-index.txt` on the mk1 string rather than on key order in
//! `mnemonic-engrave`'s pathological journey, after an ordering assumption
//! captioned 30 plates with the wrong cosigner. One record carries its
//! fingerprint, path and key together and cannot come apart.
//!
//! Blank lines and `#` comments are ignored, so a key list can be annotated.

use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};

use crate::cmd::{parse_derivation_path, parse_fingerprint, parse_xpub_normalized};
use crate::error::{CliError, Result};

/// One cosigner key: an xpub plus the origin it was derived at.
#[derive(Debug, Clone)]
pub struct KeyRecord {
    pub fingerprint: Fingerprint,
    pub path: DerivationPath,
    pub xpub: Xpub,
}

/// Parse one BIP-380 origin-notation record: `[fingerprint/path]xpub`.
pub fn parse_key_record(line: &str) -> Result<KeyRecord> {
    let s = line.trim();
    let usage = || {
        CliError::UsageError(format!(
            "expected BIP-380 origin notation `[fingerprint/path]xpub`, got {s:?}"
        ))
    };

    let rest = s.strip_prefix('[').ok_or_else(usage)?;
    let (origin, xpub_str) = rest.split_once(']').ok_or_else(usage)?;

    // A key card always carries a path, so a bare `[fingerprint]` is refused
    // rather than defaulted to `m` -- a card engraved at the wrong depth is
    // indistinguishable from a correct one until recovery fails.
    let (fp_str, path_rest) = origin.split_once('/').ok_or_else(|| {
        CliError::UsageError(format!(
            "origin {origin:?} has no derivation path; a key card must declare one \
             (e.g. `[{origin}/48'/0'/0'/2']`)"
        ))
    })?;

    // Reject a use-site suffix (`…]xpub…/0/*`). That is where a key is USED in
    // a policy, not where it was derived; accepting it silently would engrave
    // an origin the wallet never had.
    if xpub_str.contains('/') {
        return Err(CliError::UsageError(format!(
            "key {xpub_str:?} carries a derivation suffix; `--keys` records hold an \
             ORIGIN and a bare xpub, not a use-site path"
        )));
    }
    if xpub_str.is_empty() {
        return Err(CliError::UsageError(format!(
            "record {s:?} declares an origin but no xpub"
        )));
    }

    let fingerprint = parse_fingerprint(fp_str)?;
    let path = parse_derivation_path(&format!("m/{path_rest}"))?;
    let xpub = parse_xpub_normalized(xpub_str, Some(&path))?;
    Ok(KeyRecord {
        fingerprint,
        path,
        xpub,
    })
}

/// Read key records from `path`, or from stdin when `path` is `-`.
///
/// Errors name the source and the 1-based LINE NUMBER. A key list is edited by
/// hand and a rejected record is the common case, so "which line" is the whole
/// value of the message.
pub fn read_key_records(path: &str) -> Result<Vec<KeyRecord>> {
    let (buf, source) = if path == "-" {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
        (buf, "<stdin>".to_string())
    } else {
        let buf = std::fs::read_to_string(path)
            .map_err(|e| CliError::UsageError(format!("--keys {path}: {e}")))?;
        (buf, path.to_string())
    };

    let mut out = Vec::new();
    for (i, raw) in buf.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let rec = parse_key_record(line)
            .map_err(|e| CliError::UsageError(format!("{source}:{}: {}", i + 1, e.message())))?;
        out.push(rec);
    }
    if out.is_empty() {
        return Err(CliError::UsageError(format!(
            "--keys {source}: no key records found (blank lines and `#` comments are ignored)"
        )));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const XPUB: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";

    #[test]
    fn parses_bip380_origin_notation() {
        let r = parse_key_record(&format!("[73c5da0a/48'/0'/0'/2']{XPUB}")).unwrap();
        assert_eq!(r.fingerprint.to_string(), "73c5da0a");
        assert_eq!(r.path.to_string(), "48'/0'/0'/2'");
        assert_eq!(r.xpub.to_string(), XPUB);
    }

    #[test]
    fn accepts_h_hardened_marker() {
        let r = parse_key_record(&format!("[73c5da0a/48h/0h/0h/2h]{XPUB}")).unwrap();
        assert_eq!(r.path.to_string(), "48'/0'/0'/2'");
    }

    #[test]
    fn rejects_missing_brackets() {
        let e = parse_key_record(XPUB).unwrap_err();
        assert!(
            e.message().contains("BIP-380 origin notation"),
            "{}",
            e.message()
        );
    }

    #[test]
    fn rejects_origin_without_path() {
        let e = parse_key_record(&format!("[73c5da0a]{XPUB}")).unwrap_err();
        assert!(
            e.message().contains("no derivation path"),
            "{}",
            e.message()
        );
    }

    #[test]
    fn rejects_use_site_suffix() {
        let e = parse_key_record(&format!("[73c5da0a/48'/0'/0'/2']{XPUB}/0/*")).unwrap_err();
        assert!(e.message().contains("derivation suffix"), "{}", e.message());
    }

    #[test]
    fn rejects_origin_with_no_key() {
        let e = parse_key_record("[73c5da0a/48'/0'/0'/2']").unwrap_err();
        assert!(e.message().contains("no xpub"), "{}", e.message());
    }
}
