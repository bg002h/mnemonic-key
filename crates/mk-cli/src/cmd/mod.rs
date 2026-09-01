//! Subcommand handlers for `mk-cli`. Each module is independent and consumes
//! the foundation modules + the `mk-codec` and `md-codec` libraries.

pub mod address;
pub mod decode;
pub mod derive;
pub mod derive_support;
pub mod encode;
pub mod gen_man;
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

/// The 20-bit chunk-set id of `s`, or `None` when `s` is a complete
/// single-string md1 rather than one chunk of a set.
///
/// Reads the wire header directly instead of inferring from a failed decode:
/// a chunk's first symbol carries `[version:4][chunked:1]`, and
/// `ChunkHeader::read` refuses anything whose chunked-flag is clear. A string
/// that is not a chunk therefore returns `None` here without being decoded,
/// and a malformed string returns `None` too -- it is passed through to the
/// codec so the real error text reaches the operator, rather than being
/// reported as a grouping failure.
fn md1_chunk_set_id(s: &str) -> Option<u32> {
    let (bytes, _bit_len) = md_codec::codex32::unwrap_string(s).ok()?;
    let mut r = md_codec::bitstream::BitReader::new(&bytes);
    md_codec::ChunkHeader::read(&mut r)
        .ok()
        .map(|h| h.chunk_set_id)
}

/// The chunk-set id declared on the wire header of a CHUNKED mk1 set, or
/// `None` for single-string (unchunked) input -- there is nothing to compare
/// there.
///
/// Reads only the FIRST string's string-layer header: `strings` has already
/// decoded cleanly via `mk_codec::decode`, whose `reassemble_from_chunks`
/// already proved every chunk agrees on `chunk_set_id`
/// (`Error::ChunkSetIdMismatch` otherwise per SPEC §4 rule 12), so reading
/// one chunk's header is sufficient -- this is not a shortcut that skips
/// that check, only a re-read of a value the codec already verified.
fn declared_chunk_set_id(strings: &[String]) -> Option<u32> {
    let first = strings.first()?;
    let decoded = mk_codec::string_layer::decode_string(first).ok()?;
    let (header, _consumed) =
        mk_codec::string_layer::StringLayerHeader::from_5bit_symbols(decoded.data()).ok()?;
    match header {
        mk_codec::string_layer::StringLayerHeader::Chunked { chunk_set_id, .. } => {
            Some(chunk_set_id)
        }
        // `StringLayerHeader` is `#[non_exhaustive]`; `SingleString` and any
        // future variant alike carry no `chunk_set_id` to compare.
        _ => None,
    }
}

/// SPEC "The comparison": `derive_chunk_set_id(encode_bytecode(decoded_card))`
/// -- the canonical RE-ENCODE of the successfully decoded card, not the raw
/// reassembled bytes (a foreign encoder whose bytecode canonicalization
/// drifts stamps an id consistent with its own bytes; only the re-encode
/// route detects that drift -- SPEC "The comparison").
///
/// `mk_codec::derive_chunk_set_id` fully qualified: `md_codec` exports a
/// same-named function and this crate links both.
fn derived_chunk_set_id(card: &mk_codec::KeyCard) -> Option<u32> {
    let bytecode = mk_codec::bytecode::encode_bytecode(card).ok()?;
    Some(mk_codec::derive_chunk_set_id(&bytecode))
}

/// Compare the declared vs. content-derived `chunk_set_id` for a decoded
/// mk1 set (SPEC R2 / contracts 2-4). `None` for single-string input
/// (nothing declared to compare against). `Some((declared, derived))`
/// otherwise, REGARDLESS of whether they agree -- `mk verify --json`
/// reports the pair even on a match (contract 4), so callers decide what a
/// match vs. a mismatch means.
pub fn chunk_set_id_comparison(
    strings: &[String],
    card: &mk_codec::KeyCard,
) -> Option<(u32, u32)> {
    let declared = declared_chunk_set_id(strings)?;
    let derived = derived_chunk_set_id(card)?;
    Some((declared, derived))
}

/// The frozen R2 mismatch-warning content (SPEC contract 2 draft / R6 "same
/// warning everywhere"): byte-identical to the extension corpus's
/// `warning_text` for the pinned `12345`/`ef12f` row. `{:05x}`: exactly
/// five lowercase hex digits, zero-padded -- the rendering
/// `GroupId::Display` prints and `md --seat @i=` accepts (SPEC "Behavior
/// contracts, per surface").
pub fn chunk_set_id_mismatch_warning(declared: u32, derived: u32) -> String {
    format!(
        "warning: this key card's stamped chunk-set id ({declared:05x}) was not derived from \
         its content, which computes {derived:05x}. The card decodes fine, but diagnostics that \
         name plates by id will call it {declared:05x}. To fix it, re-mint: run mk encode again \
         without --chunk-set-id and the id is derived from the key data automatically."
    )
}

/// Emit the R2 stderr warning for a precomputed `chunk_set_id_comparison`
/// result. A no-op on a match, or on `None` (single-string input, or a
/// re-encode that could not be computed).
///
/// Independently deletable per verb (P1 mutation gate): each call site
/// below is this one line, seated at that verb's own decode call, per
/// plan P1 -- NOT centralized in `read_mk1_strings`, which only reads
/// strings and never decodes.
pub fn warn_chunk_set_id_mismatch(comparison: Option<(u32, u32)>) {
    if let Some((declared, derived)) = comparison {
        if declared != derived {
            eprintln!("{}", chunk_set_id_mismatch_warning(declared, derived));
        }
    }
}

/// Partition `--from-md1` values into CARDS, preserving first-appearance order.
///
/// One `--from-md1` value is one md1 STRING, but one card may be several
/// strings: a keyed wallet policy is 246 data symbols and the codex32 regular
/// code caps a single md1 string at 80, so every keyed card in the
/// constellation arrives as a chunk SET. Chunks are grouped by the 20-bit
/// chunk-set id in their wire header, which means they need not be adjacent or
/// in index order on the command line.
///
/// `--from-md1` remains "one card per POLICY": distinct chunk sets stay
/// distinct cards and yield one stub each, in the order their first chunk
/// appeared. That is what keeps a key card that belongs to two wallets from
/// collapsing into one stub -- see `two_chunk_sets_are_two_cards_in_order`.
pub fn group_md1_cards(values: &[String]) -> Vec<Vec<&str>> {
    let mut groups: Vec<(Option<u32>, Vec<&str>)> = Vec::new();
    for v in values {
        let key = md1_chunk_set_id(v);
        match key.and_then(|k| groups.iter_mut().find(|(g, _)| *g == Some(k))) {
            Some((_, chunks)) => chunks.push(v.as_str()),
            // A non-chunk (`key == None`) always starts its own group: two
            // identical single-string cards are two cards, and `None` is not a
            // set id to merge on.
            None => groups.push((key, vec![v.as_str()])),
        }
    }
    groups.into_iter().map(|(_, v)| v).collect()
}

/// Derive the 4-byte `policy_id_stub` for ONE card, supplied as either a
/// single complete md1 string or the full set of its chunks.
///
/// The stub is **FORM-AWARE** (matches the toolkit's `bundle_binding_stub`, #28):
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
///
/// Chunked input goes through `md_codec::reassemble`, which verifies per-chunk
/// BCH, header consistency, index completeness, and the cross-chunk
/// content-id -- so a short or doctored set is refused here rather than
/// producing a stub from whatever chunks were present.
pub fn derive_stub_from_md1_card(card: &[&str]) -> Result<[u8; 4]> {
    Ok(decode_md1_card(card)?.stub)
}

/// The 65-byte identity md1 stores for a concrete key: 32-byte chain code
/// followed by the 33-byte compressed public key (`TlvSection::pubkeys`).
///
/// Compared as bytes rather than as a base58 xpub string so the check cannot be
/// defeated by an equivalent re-serialization (a SLIP-132 `Zpub`, or a differing
/// depth/child byte) that names the same key.
pub fn xpub_identity_65(x: &Xpub) -> [u8; 65] {
    let mut out = [0u8; 65];
    out[..32].copy_from_slice(&x.chain_code.to_bytes());
    out[32..].copy_from_slice(&x.public_key.serialize());
    out
}

/// One `--from-md1` card, decoded once: its binding stub, plus the cosigner set
/// it declares (`None` for a keyless template, which carries no keys).
pub struct Md1Card {
    pub stub: [u8; 4],
    pub cosigners: Option<Vec<[u8; 65]>>,
}

/// Decode ONE card -- a single complete md1 string, or the full set of its
/// chunks -- into its stub and cosigner set.
///
/// Chunked input goes through `md_codec::reassemble`, which verifies per-chunk
/// BCH, header consistency, index completeness, and the cross-chunk content-id
/// -- so a short or doctored set is refused here rather than producing a stub
/// from whatever chunks were present.
pub fn decode_md1_card(card: &[&str]) -> Result<Md1Card> {
    let descriptor = match card {
        [single] if md1_chunk_set_id(single).is_none() => md_codec::decode_md1_string(single)?,
        chunks => md_codec::reassemble(chunks)?,
    };
    let id_bytes = if descriptor.is_wallet_policy() {
        *md_codec::compute_wallet_policy_id(&descriptor)?.as_bytes()
    } else {
        *md_codec::compute_wallet_descriptor_template_id(&descriptor)?.as_bytes()
    };
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&id_bytes[..4]);

    // The cosigner set, but ONLY when it is complete. `None` means "membership
    // is not decidable from this card", not "the set is empty".
    //
    // `TlvSection::pubkeys` is SPARSE: a policy may carry keys for some `@N`
    // and not others, and `md` emits such a card at exit 0. Treating a partial
    // list as exhaustive refuses a legitimate cosigner whose slot simply is not
    // filled in yet -- with a message asserting they are "not a cosigner",
    // which is false. Refusing a valid mint is the expensive direction, and it
    // pushes a careful operator onto `--policy-id-stub`, which is NOT
    // membership-checked -- the very bypass this check exists to close.
    //
    // So: decidable iff every one of the descriptor's `n` placeholders has a
    // key. Found by an independent review of this check's own fold (R7/B2,
    // 2026-08-21).
    let cosigners = descriptor
        .tlv
        .pubkeys
        .as_ref()
        .filter(|v| v.len() == descriptor.n as usize)
        .map(|v| v.iter().map(|(_, k)| *k).collect());

    Ok(Md1Card { stub, cosigners })
}

/// Format `policy_id_stub` bytes as 8 lowercase hex chars.
pub fn fmt_stub(stub: &[u8; 4]) -> String {
    hex::encode(stub)
}

/// Format a `Fingerprint` as 8 lowercase hex chars.
pub fn fmt_fingerprint(fp: &Fingerprint) -> String {
    hex::encode(fp.to_bytes())
}

/// Read a list of mk1 strings from `--in FILE`, from positional `args`, and
/// from stdin via a positional `"-"`.
///
/// `"-"` may appear as any positional value but is processed once across the
/// list. `--in` is read FIRST so a mixed invocation has one obvious order, and
/// every channel goes through the same per-line strip: mstring display-grouping
/// (SPEC §3.2) means a grouped card and an unbroken card must both re-ingest,
/// which is exactly what a human transcribing from the engraving card produces.
pub fn read_mk1_strings(args: &[String], in_file: Option<&str>) -> Result<Vec<String>> {
    let mut out = Vec::with_capacity(args.len());
    if let Some(path) = in_file {
        let buf = std::fs::read_to_string(path)
            .map_err(|e| CliError::UsageError(format!("--in {path}: {e}")))?;
        push_stripped_lines(&mut out, &buf);
    }
    let mut consumed_stdin = false;
    for a in args {
        if a == "-" && !consumed_stdin {
            consumed_stdin = true;
            let mut buf = String::new();
            std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
            push_stripped_lines(&mut out, &buf);
        } else if a == "-" {
            // Already consumed stdin; ignore additional `-` markers.
        } else {
            out.push(crate::format::strip_display_separators(a));
        }
    }
    if out.is_empty() {
        // SPEC §6h: the remedy must be executable, so it names only channels
        // that exist. `--in FILE` is one of them as of P3.
        return Err(CliError::UsageError(
            "expected at least one mk1 string (positional, via --in FILE, or via stdin with '-')"
                .into(),
        ));
    }
    Ok(out)
}

/// Push each non-empty line of `buf`, with display separators stripped.
///
/// mstring display-grouping (SPEC §3.2): strip ALL whitespace + `-` + `,` so a
/// grouped or unbroken card both re-ingest. A line that strips to empty --- a
/// blank, or a run of separators --- contributes nothing rather than becoming an
/// empty "string" the codec would then reject.
fn push_stripped_lines(out: &mut Vec<String>, buf: &str) {
    for line in buf.lines() {
        let s = crate::format::strip_display_separators(line);
        if !s.is_empty() {
            out.push(s);
        }
    }
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
            "note: the xpub was a SLIP-0132 {}; normalized to canonical {} — script type is conveyed by the origin path, not the key prefix",
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
