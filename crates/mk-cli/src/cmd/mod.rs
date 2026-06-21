//! Subcommand handlers for `mk-cli`. Each module is independent and consumes
//! the foundation modules + the `mk-codec` and `md-codec` libraries.

pub mod address;
pub mod decode;
pub mod derive;
pub mod derive_support;
pub mod encode;
pub mod gui_schema;
pub mod inspect;
pub mod repair;
pub mod vectors;
pub mod verify;

use std::str::FromStr;

use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};

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

/// Derive the 4-byte `policy_id_stub` from an md1 string per SPEC §3.3,
/// **FORM-AWARE** (matches the toolkit's `bundle_binding_stub`, #28):
///
/// - a **keyed wallet-policy** md1 (`is_wallet_policy()`) → top 4 bytes of the
///   policy's **WalletPolicyId** (`md_codec::compute_wallet_policy_id`, md SPEC
///   v0.13 §5.3 canonical-expanded, encoder-divergence-free);
/// - a **keyless template** md1 (`!is_wallet_policy()`, e.g. a single-sig
///   `--md1-form=template` bundle) → top 4 bytes of the key-stable
///   **WalletDescriptorTemplateId** (`md_codec::compute_wallet_descriptor_template_id`,
///   md SPEC §8.1, BIP-388 template-only identity).
///
/// In both cases the stub is rooted on a canonical, encoder-divergence-free
/// identity — NOT the md1 bytecode hash, which is encoding-sensitive and would
/// not survive a re-encode of the same logical wallet. Discriminating on
/// `is_wallet_policy()` keeps a stub minted via `mk --from-md1` byte-for-byte
/// in agreement with the toolkit-emitted bundle card for the SAME md1 form
/// (audit I1, 2026-06-10; toolkit #28 `bundle --md1-form=template`).
pub fn derive_stub_from_md1(md1_str: &str) -> Result<[u8; 4]> {
    let descriptor = md_codec::decode_md1_string(md1_str)?;
    let id_bytes = if descriptor.is_wallet_policy() {
        *md_codec::compute_wallet_policy_id(&descriptor)?.as_bytes()
    } else {
        *md_codec::compute_wallet_descriptor_template_id(&descriptor)?.as_bytes()
    };
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&id_bytes[..4]);
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
                // mstring display-grouping (SPEC §3.2): strip ALL whitespace + `-`
                // + `,` so a grouped or unbroken card both re-ingest.
                let s = crate::format::strip_display_separators(line);
                if !s.is_empty() {
                    out.push(s);
                }
            }
        } else if a == "-" {
            // Already consumed stdin; ignore additional `-` markers.
        } else {
            out.push(crate::format::strip_display_separators(a));
        }
    }
    if out.is_empty() {
        return Err(CliError::UsageError(
            "expected at least one mk1 string (positional or via stdin with '-')".into(),
        ));
    }
    Ok(out)
}

/// Detect the BCH code variant for a single mk1 string by its length.
///
/// Boundaries mirror the authoritative `mk_codec::string_layer::bch::
/// bch_code_for_length` (BIP-93/codex32): a **regular** code `BCH(93,80,8)`
/// has a data-part of 14..=93 symbols, and a **long** code `BCH(108,93,8)`
/// has 96..=108 (94–95 are reserved-invalid and never reach a decoded string).
/// An mk1 string adds the `mk1` HRP+separator (3 chars), so the total-length
/// regular ceiling is `93 + "mk1".len()` = 96; anything longer is long.
pub fn classify_code_variant(s: &str) -> &'static str {
    // Regular data-part caps at 93 symbols ⇒ total length ≤ 96. A 96-symbol
    // long-code data-part is total length 99 ⇒ correctly classified "long".
    if s.len() <= 93 + "mk1".len() {
        "regular"
    } else {
        "long"
    }
}

/// Parse an xpub, accepting SLIP-0132 prefixes (normalized to canonical xpub/tpub).
/// Emits a stderr note on normalization; refuses (UsageError) if a non-canonical
/// prefix's implied script type contradicts `origin_path` (when supplied).
pub fn parse_xpub_normalized(s: &str, origin_path: Option<&DerivationPath>) -> Result<Xpub> {
    let (xpub, variant) = crate::slip132::detect_and_normalize(s)?;
    if let Some(v) = variant {
        eprintln!(
            "note: --xpub was a SLIP-0132 {}; normalized to canonical {} — script type is conveyed by the origin path, not the key prefix",
            v.label(),
            v.canonical_label()
        );
        if let Some(path) = origin_path {
            if !v.path_matches(path) {
                return Err(CliError::UsageError(v.mismatch_help(path)));
            }
        }
    }
    Ok(xpub)
}

#[cfg(test)]
mod tests {
    use super::classify_code_variant;

    /// Build a synthetic `mk1<data>` string whose *data-part* (chars after the
    /// `mk1` HRP+separator) has exactly `data_len` symbols. `classify_code_variant`
    /// only looks at the total length, so the data content is irrelevant.
    fn mk1_with_data_len(data_len: usize) -> String {
        format!("mk1{}", "q".repeat(data_len))
    }

    /// L20 — authoritative `mk_codec::bch_code_for_length` boundaries
    /// (`crates/mk-codec/src/string_layer/bch.rs`): regular = 14..=93,
    /// 94..=95 reserved-invalid, long = 96..=108.
    ///
    /// A 96-symbol data-part is `BchCode::Long`, but the pre-fix threshold
    /// (`s.len() <= 96 + "mk1".len()` ⇒ ≤99) mislabeled it "regular".
    #[test]
    fn classify_96_symbol_data_part_is_long() {
        // 96 data symbols → total length 99. Authoritative: Long.
        let s = mk1_with_data_len(96);
        assert_eq!(
            s.len(),
            99,
            "fixture: total length 99 (96 data + 3 hrp/sep)"
        );
        assert_eq!(
            classify_code_variant(&s),
            "long",
            "96-symbol data-part is BCH(108,93,8) long, not regular"
        );
    }

    /// The upper edge of the regular band: a 93-symbol data-part (total 96)
    /// is `BchCode::Regular`.
    #[test]
    fn classify_93_symbol_data_part_is_regular() {
        let s = mk1_with_data_len(93);
        assert_eq!(
            s.len(),
            96,
            "fixture: total length 96 (93 data + 3 hrp/sep)"
        );
        assert_eq!(
            classify_code_variant(&s),
            "regular",
            "93-symbol data-part is BCH(93,80,8) regular"
        );
    }

    /// The maximum long-code data-part (108 symbols, total 111) classifies long.
    #[test]
    fn classify_108_symbol_data_part_is_long() {
        let s = mk1_with_data_len(108);
        assert_eq!(classify_code_variant(&s), "long");
    }
}
