//! `CliError` enum + exit-code mapping.
//!
//! Realizes SPEC §3.5.6 (JSON error envelope) and §3.5.7 (exit-code table)
//! from the `concurrent-cooking-scone` plan.

use serde_json::json;

/// All `mk-cli` failure modes.
///
/// `#[non_exhaustive]` so future variants can land without breaking
/// downstream `match` arms.
#[non_exhaustive]
#[derive(Debug)]
pub enum CliError {
    /// mk-codec rejected the input. Exit 2 (or 3 for `UnsupportedVersion`).
    Codec(mk_codec::Error),
    /// md-codec rejected an `--from-md1` input.
    MdCodec(md_codec::Error),
    /// mk1 string is well-formed but its declared version is newer
    /// than this build understands. Exit 3.
    ///
    /// Currently unconstructed in v0.2: mk-codec surfaces unknown versions
    /// as `Codec(Error::UnsupportedVersion(_))` which already exits 3 via
    /// the `exit_code()` mapping. This variant is reserved for cases where
    /// mk-cli detects a future-format condition the codec didn't surface
    /// (e.g., a v0.3 wire-format-version detection that pre-empts the
    /// codec's bytecode-layer call).
    #[allow(dead_code)]
    FutureFormat(String),
    /// `verify` mode with `--xpub` / `--origin-*` / `--policy-id-stub` /
    /// `--from-md1` flags found a mismatch between the decoded card and
    /// the user-supplied expected value.
    ContentMismatch {
        /// Field name whose value disagreed (`xpub`, `origin_fingerprint`,
        /// `origin_path`, `policy_id_stubs`).
        field: String,
        /// Expected value, formatted for human display.
        expected: String,
        /// Actual decoded value, formatted for human display.
        actual: String,
    },
    /// CLI usage error (missing required argument, mutually-exclusive
    /// flags both supplied, etc.). Exit 64.
    UsageError(String),
    /// I/O error (stdin read failed, output file write failed). Exit 1.
    IoError(std::io::Error),
    /// Cycle E (`mk1-repair-set-level-reverify`, F4) — THE FUNDS FIX. `mk
    /// repair` per-string-corrected a `chunk_set_id` group that is
    /// complete-and-consistent (every index `0..total_chunks` present
    /// exactly once), but the corrected group does NOT reassemble through
    /// `mk_codec::decode` — the per-chunk BCH correction(s) aliased to a
    /// DIFFERENT valid codeword, not the original card. Exit 2. Mirrors the
    /// toolkit's `RepairError::SetReassemblyMismatch`.
    SetReassemblyMismatch {
        /// Human-readable identifier of the failing group (e.g.
        /// `"chunk_set_id 0x12345"` or `"single-string chunk 2"`), so a
        /// batch invocation containing multiple groups tells the user WHICH
        /// one failed and lets them re-run the good group alone.
        group: String,
        /// The underlying `mk_codec::decode` error's `Display` text.
        detail: String,
    },
}

impl CliError {
    /// Stable variant-name string for the JSON `kind` field.
    pub fn kind(&self) -> &'static str {
        match self {
            CliError::Codec(e) => mk_codec_error_kind(e),
            CliError::MdCodec(_) => "MdCodec",
            CliError::FutureFormat(_) => "FutureFormat",
            CliError::ContentMismatch { .. } => "ContentMismatch",
            CliError::UsageError(_) => "UsageError",
            CliError::IoError(_) => "IoError",
            CliError::SetReassemblyMismatch { .. } => "SetReassemblyMismatch",
        }
    }

    /// User-readable single-line message.
    pub fn message(&self) -> String {
        match self {
            CliError::Codec(e) => format!("{e}"),
            CliError::MdCodec(e) => format!("md1 input rejected: {e}"),
            CliError::FutureFormat(m) => m.clone(),
            CliError::ContentMismatch {
                field,
                expected,
                actual,
            } => format!("verify mismatch on {field}: expected {expected}, got {actual}"),
            CliError::UsageError(m) => m.clone(),
            CliError::IoError(e) => format!("io error: {e}"),
            CliError::SetReassemblyMismatch { group, detail } => format!(
                "each chunk corrected individually, but the set does not reassemble ({group}): \
                {detail} — the correction(s) may have aliased to a DIFFERENT valid card; this \
                output is NOT trustworthy"
            ),
        }
    }

    /// Exit code per SPEC §3.5.7.
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Codec(mk_codec::Error::UnsupportedVersion(_)) => 3,
            CliError::Codec(_) | CliError::MdCodec(_) => 2,
            CliError::FutureFormat(_) => 3,
            CliError::ContentMismatch { .. } => 4,
            CliError::UsageError(_) => 64,
            CliError::IoError(_) => 1,
            CliError::SetReassemblyMismatch { .. } => 2,
        }
    }

    /// Optional `details` field for the JSON envelope.
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            CliError::ContentMismatch {
                field,
                expected,
                actual,
            } => Some(json!({
                "field": field,
                "expected": expected,
                "actual": actual,
            })),
            CliError::FutureFormat(m) => Some(json!({ "message": m })),
            CliError::SetReassemblyMismatch { group, detail } => Some(json!({
                "group": group,
                "detail": detail,
            })),
            _ => None,
        }
    }
}

/// Map an `mk_codec::Error` variant to its stable string name for the JSON `kind` field.
fn mk_codec_error_kind(e: &mk_codec::Error) -> &'static str {
    match e {
        mk_codec::Error::InvalidHrp(_) => "InvalidHrp",
        mk_codec::Error::MixedCase => "MixedCase",
        mk_codec::Error::InvalidStringLength(_) => "InvalidStringLength",
        mk_codec::Error::InvalidChar { .. } => "InvalidChar",
        mk_codec::Error::BchUncorrectable(_) => "BchUncorrectable",
        mk_codec::Error::UnsupportedCardType(_) => "UnsupportedCardType",
        mk_codec::Error::MalformedPayloadPadding => "MalformedPayloadPadding",
        mk_codec::Error::ChunkSetIdMismatch => "ChunkSetIdMismatch",
        mk_codec::Error::ChunkedHeaderMalformed(_) => "ChunkedHeaderMalformed",
        mk_codec::Error::MixedHeaderTypes => "MixedHeaderTypes",
        mk_codec::Error::CrossChunkHashMismatch => "CrossChunkHashMismatch",
        mk_codec::Error::UnsupportedVersion(_) => "UnsupportedVersion",
        mk_codec::Error::ReservedBitsSet => "ReservedBitsSet",
        mk_codec::Error::InvalidPolicyIdStubCount => "InvalidPolicyIdStubCount",
        mk_codec::Error::InvalidPathIndicator(_) => "InvalidPathIndicator",
        mk_codec::Error::PathTooDeep(_) => "PathTooDeep",
        mk_codec::Error::InvalidPathComponent(_) => "InvalidPathComponent",
        mk_codec::Error::InvalidXpubVersion(_) => "InvalidXpubVersion",
        mk_codec::Error::InvalidXpubPublicKey(_) => "InvalidXpubPublicKey",
        mk_codec::Error::UnexpectedEnd => "UnexpectedEnd",
        mk_codec::Error::TrailingBytes => "TrailingBytes",
        mk_codec::Error::CardPayloadTooLarge { .. } => "CardPayloadTooLarge",
        mk_codec::Error::XpubOriginPathMismatch { .. } => "XpubOriginPathMismatch",
        // `mk_codec::Error` is `#[non_exhaustive]`; keep a fallback.
        _ => "Unknown",
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.message())
    }
}

impl std::error::Error for CliError {}

impl From<mk_codec::Error> for CliError {
    fn from(e: mk_codec::Error) -> Self {
        CliError::Codec(e)
    }
}

impl From<md_codec::Error> for CliError {
    fn from(e: md_codec::Error) -> Self {
        CliError::MdCodec(e)
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::IoError(e)
    }
}

/// `Result` alias for `mk-cli` handlers.
pub type Result<T> = core::result::Result<T, CliError>;
