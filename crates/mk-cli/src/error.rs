//! `CliError` enum + exit-code mapping.
//!
//! The JSON error envelope and the exit-code table are CLI conventions with NO
//! mk SPEC section governing them -- they are pinned by `tests/` and the
//! CHANGELOG, not by the format spec.
//!
//! This module previously claimed to realize two SPEC sections that do not
//! exist; §3.5 is "Origin path encoding" and has no subsections at all. The
//! retired cites were §3.5.6 and §3.5.7. SPEC-CITE-EXEMPT (quoted as retired,
//! not asserted). Origin: the `concurrent-cooking-scone` plan. See FOLLOWUPS
//! F-224.

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

    /// Exit code. A CLI convention pinned by tests, not by the format SPEC.
    ///
    /// **`Codec(_) | MdCodec(_)` is 1, not 2 (SPEC §6f).** An invalid artifact
    /// is 1 across `md`, `ms` and `mnemonic`, and `mk` was the outlier.
    ///
    /// `mk repair` still exits **2** on any codec error, and it no longer
    /// reaches this arm to do it: it returns `Ok(2)` from its own bypass, the
    /// shape `md repair` already ships. That split is by VERB, not by error
    /// kind -- see `cmd/repair.rs` -- and it is the reason this arm could move
    /// without breaking the repair-exit-code contract (F-291).
    ///
    /// `SetReassemblyMismatch` is a SEPARATE arm and stays **2**. It is the
    /// miscorrection rejection, and it is unreachable from the arm above -- a
    /// fact established by mutation rather than by reading: applying the naive
    /// `=> 1` edit alone reds exactly two tests, neither of them one of the four
    /// that pin this variant.
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Codec(mk_codec::Error::UnsupportedVersion(_)) => 3,
            CliError::Codec(_) | CliError::MdCodec(_) => 1,
            CliError::FutureFormat(_) => 3,
            CliError::ContentMismatch { .. } => 4,
            CliError::UsageError(_) => 64,
            CliError::IoError(_) => 1,
            CliError::SetReassemblyMismatch { .. } => 2,
        }
    }

    /// The JSON error envelope, as one line.
    ///
    /// `exit_code` is a PARAMETER rather than `self.exit_code()` because
    /// `mk repair` returns its 2 from a bypass (SPEC §6f, see `cmd/repair.rs`)
    /// while the same `CliError` maps to 1. The envelope must report the code
    /// the process actually exits with, or a `--json` consumer reads one number
    /// and its shell reads another.
    pub fn json_envelope(&self, exit_code: u8) -> String {
        let envelope = json!({
            "schema_version": 1,
            "error": {
                "kind": self.kind(),
                "message": self.message(),
                "exit_code": exit_code,
                "details": self.details(),
            },
        });
        serde_json::to_string(&envelope).expect("error envelope serializes")
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

#[cfg(test)]
mod exit_code_table {
    use super::CliError;

    /// The whole exit-code table, pinned at the source.
    ///
    /// The integration tests measure BEHAVIOUR through the binary; this pins the
    /// mapping itself, so an arm that is edited without a matching CLI path --
    /// or a variant whose code is moved while "tidying" a neighbour -- reds here
    /// even if no verb currently constructs it.
    ///
    /// **The `SetReassemblyMismatch` row is the funds-safety one.** It is the
    /// miscorrection rejection: `mk repair` corrected every chunk of a complete
    /// group individually, but the group does not reassemble, so a correction
    /// may have aliased to a DIFFERENT valid card. It must never become a
    /// success code and it must not follow `Codec(_)` to 1.
    #[test]
    fn every_arm_maps_to_its_documented_code() {
        let cases: Vec<(CliError, u8, &str)> = vec![
            (
                CliError::Codec(mk_codec::Error::UnsupportedVersion(9)),
                3,
                "a future wire version",
            ),
            (
                CliError::Codec(mk_codec::Error::MixedCase),
                1,
                "SPEC 6f: an invalid artifact is 1, not 2",
            ),
            (
                CliError::Codec(mk_codec::Error::BchUncorrectable("5 errors".into())),
                1,
                "uncorrectable is still just an invalid artifact HERE; `mk repair` \
                 returns Ok(2) from its own bypass and never reaches this arm",
            ),
            (
                CliError::MdCodec(md_codec::Error::WireVersionMismatch { got: 9 }),
                1,
                "an md1 the --from-md1 binding cannot read is an invalid artifact",
            ),
            (CliError::FutureFormat("v9".into()), 3, "future format"),
            (
                CliError::ContentMismatch {
                    field: "xpub".into(),
                    expected: "a".into(),
                    actual: "b".into(),
                },
                4,
                "a verify mismatch",
            ),
            (CliError::UsageError("bad flags".into()), 64, "usage"),
            (CliError::IoError(std::io::Error::other("boom")), 1, "io"),
            (
                CliError::SetReassemblyMismatch {
                    group: "chunk_set_id 0x12345".into(),
                    detail: "does not reassemble".into(),
                },
                2,
                "THE FUNDS FIX: a miscorrection rejection stays 2",
            ),
        ];
        for (e, want, why) in cases {
            assert_eq!(e.exit_code(), want, "{why} ({:?})", e.kind());
        }
    }
}
