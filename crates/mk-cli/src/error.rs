//! `CliError` enum + exit-code mapping.
//!
//! The exit codes follow the constellation exit-code spec
//! (mnemonic-engrave `design/SPEC_constellation_exit_codes.md` §3); the
//! numbers live in `crate::exit`. Each variant is placed by the spec's
//! ordered tests, and `exit_code_table` pins every variant to its class
//! (rule R13). The JSON error envelope is a CLI convention with no mk SPEC
//! section, pinned by `tests/` and the CHANGELOG.
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
    /// mk-codec rejected the input. Exit 2 (INVALID), including
    /// `UnsupportedVersion` (a newer format is an invalid input here).
    Codec(mk_codec::Error),
    /// md-codec rejected an `--from-md1` input.
    MdCodec(md_codec::Error),
    /// mk1 string is well-formed but its declared version is newer
    /// than this build understands. Exit 2 (INVALID).
    ///
    /// Currently unconstructed in v0.2: mk-codec surfaces unknown versions
    /// as `Codec(Error::UnsupportedVersion(_))` which already exits 2 via
    /// the `exit_code()` mapping. This variant is reserved for cases where
    /// mk-cli detects a future-format condition the codec didn't surface
    /// (e.g., a v0.3 wire-format-version detection that pre-empts the
    /// codec's bytecode-layer call).
    #[allow(dead_code)]
    FutureFormat(String),
    /// `verify` mode with `--xpub` / `--origin-*` / `--policy-id-stub` /
    /// `--from-md1` flags found a mismatch between the decoded card and
    /// the user-supplied expected value. Exit 1 (NO).
    ContentMismatch {
        /// Field name whose value disagreed (`xpub`, `origin_fingerprint`,
        /// `origin_path`, `policy_id_stubs`).
        field: String,
        /// Expected value, formatted for human display.
        expected: String,
        /// Actual decoded value, formatted for human display.
        actual: String,
    },
    /// CLI usage error, judged from the command line alone (missing required
    /// argument, mutually-exclusive flags both supplied, a bad setting such as
    /// `--count`). Exit 64; never combines.
    UsageError(String),
    /// An input is wrong for this verb whatever the flags: a malformed xpub,
    /// fingerprint, path or stub value, a key file record, empty input, a card
    /// the verb cannot use. Exit 2 (INVALID).
    Invalid(String),
    /// The input is well-formed but does not match what it should (a key that
    /// is not a cosigner of the policy it is stamped with, an origin that
    /// contradicts its own policy). Exit 1 (NO).
    Mismatch(String),
    /// A safety or policy refusal a flag, network, channel or different
    /// destination would waive (an output that is an input, a network
    /// mismatch). Exit 8 (REFUSED).
    Refused(String),
    /// I/O error (stdin read failed, output file write failed). Exit 32 (IO).
    IoError(std::io::Error),
    /// I/O error with context (an input file that cannot be read). Exit 32.
    Io(String),
    /// A failure no class covers: a bug (an internal serialization failure,
    /// a corrupt built-in corpus). Exit 101 (INTERNAL); never combines.
    Internal(String),
    /// Cycle E (`mk1-repair-set-level-reverify`, F4) — THE FUNDS FIX. `mk
    /// repair` per-string-corrected a `chunk_set_id` group that is
    /// complete-and-consistent (every index `0..total_chunks` present
    /// exactly once), but the corrected group does NOT reassemble through
    /// `mk_codec::decode` — the per-chunk BCH correction(s) aliased to a
    /// DIFFERENT valid codeword, not the original card. Exit 2. Mirrors the
    /// toolkit's `RepairError::SetReassemblyMismatch`.
    SetReassemblyMismatch {
        /// Human-readable identifier of the failing group (e.g.
        /// `"chunk_set_id 0x12345"` or `"input 2, a single string, not chunked"`), so a
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
            CliError::Invalid(_) => "InvalidInput",
            CliError::Mismatch(_) => "Mismatch",
            CliError::Refused(_) => "Refused",
            CliError::IoError(_) | CliError::Io(_) => "IoError",
            CliError::Internal(_) => "Internal",
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
            CliError::UsageError(m)
            | CliError::Invalid(m)
            | CliError::Mismatch(m)
            | CliError::Refused(m)
            | CliError::Internal(m) => m.clone(),
            CliError::IoError(e) => format!("io error: {e}"),
            CliError::Io(m) => format!("io error: {m}"),
            CliError::SetReassemblyMismatch { group, detail } => format!(
                "each chunk corrected individually, but the set does not reassemble ({group}): \
                {detail} — the correction(s) may have aliased to a DIFFERENT valid card; this \
                output is NOT trustworthy"
            ),
        }
    }

    /// Exit code, by the ordered tests of the exit-code spec §3.
    ///
    /// Every invalid artifact is 2, on every verb (an uncorrectable string,
    /// a wrong HRP, a newer version, an md1 the `--from-md1` binding cannot
    /// read). `SetReassemblyMismatch` is the miscorrection rejection (THE
    /// FUNDS FIX): a complete corrected set that does not reassemble. It is
    /// 2 and must never become a success code.
    pub fn exit_code(&self) -> u8 {
        use crate::exit::*;
        match self {
            CliError::Codec(_) | CliError::MdCodec(_) => INVALID,
            CliError::FutureFormat(_) => INVALID,
            CliError::ContentMismatch { .. } | CliError::Mismatch(_) => NO,
            CliError::UsageError(_) => USAGE,
            CliError::Invalid(_) => INVALID,
            CliError::Refused(_) => REFUSED,
            CliError::IoError(_) | CliError::Io(_) => IO,
            CliError::Internal(_) => INTERNAL,
            CliError::SetReassemblyMismatch { .. } => INVALID,
        }
    }

    /// The same error, its message prefixed with `prefix` (a file and line,
    /// a record number), keeping its class.
    pub fn prefixed(self, prefix: &str) -> CliError {
        let m = format!("{prefix}{}", self.message());
        match self {
            CliError::UsageError(_) => CliError::UsageError(m),
            CliError::Mismatch(_) | CliError::ContentMismatch { .. } => CliError::Mismatch(m),
            CliError::Refused(_) => CliError::Refused(m),
            CliError::IoError(_) | CliError::Io(_) => CliError::Io(m),
            CliError::Internal(_) => CliError::Internal(m),
            _ => CliError::Invalid(m),
        }
    }

    /// An error reading an input file or stdin: text that is not UTF-8 is an
    /// invalid input (2); anything else is the system's (32).
    pub fn reading(what: &str, e: &std::io::Error) -> CliError {
        if e.kind() == std::io::ErrorKind::InvalidData {
            CliError::Invalid(format!("{what}: {e}"))
        } else {
            CliError::Io(format!("{what}: {e}"))
        }
    }

    /// The JSON error envelope, as one line.
    ///
    /// `exit_code` is a PARAMETER rather than `self.exit_code()` because the
    /// process status can carry more bits than the error's own class (16 when
    /// a corrected value was used, spec R1/R6). The envelope must report the
    /// code the process actually exits with (spec R14: a bit sum), or a
    /// `--json` consumer reads one number and its shell reads another.
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
        mk_codec::Error::MasterParentFingerprintNonzero { .. } => "MasterParentFingerprintNonzero",
        mk_codec::Error::MasterFingerprintMismatch { .. } => "MasterFingerprintMismatch",
        mk_codec::Error::OriginParentFingerprintMismatch { .. } => {
            "OriginParentFingerprintMismatch"
        }
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
        // Text that is not UTF-8 (stdin read with `read_to_string`) is the
        // input's fault, not the system's.
        if e.kind() == std::io::ErrorKind::InvalidData {
            CliError::Invalid(format!("the input is not UTF-8 text: {e}"))
        } else {
            CliError::IoError(e)
        }
    }
}

/// `Result` alias for `mk-cli` handlers.
pub type Result<T> = core::result::Result<T, CliError>;

#[cfg(test)]
mod exit_code_table {
    use super::CliError;
    use crate::exit::*;

    /// Spec R13: every variant maps to its §3 class, pinned at the source.
    ///
    /// The integration tests measure BEHAVIOUR through the binary; this pins
    /// the mapping itself, so an arm edited without a matching CLI path -- or
    /// a variant moved while "tidying" a neighbour -- reds here even if no
    /// verb currently constructs it. A new variant must be added here: the
    /// `match` in `every_variant_is_listed` does not compile without it.
    ///
    /// **The `SetReassemblyMismatch` row is the funds-safety one.** It is the
    /// miscorrection rejection: `mk repair` corrected every chunk of a complete
    /// group individually, but the group does not reassemble, so a correction
    /// may have aliased to a DIFFERENT valid card. It must never become a
    /// success code.
    fn cases() -> Vec<(CliError, u8, &'static str)> {
        vec![
            (
                CliError::Codec(mk_codec::Error::UnsupportedVersion(9)),
                INVALID,
                "a future wire version is an invalid input (was 3)",
            ),
            (
                CliError::Codec(mk_codec::Error::MixedCase),
                INVALID,
                "an invalid artifact is 2",
            ),
            (
                CliError::Codec(mk_codec::Error::BchUncorrectable("5 errors".into())),
                INVALID,
                "uncorrectable is 2 on every verb",
            ),
            (
                CliError::MdCodec(md_codec::Error::WireVersionMismatch { got: 9 }),
                INVALID,
                "an md1 the --from-md1 binding cannot read is an invalid input",
            ),
            (
                CliError::FutureFormat("v9".into()),
                INVALID,
                "future format",
            ),
            (
                CliError::ContentMismatch {
                    field: "xpub".into(),
                    expected: "a".into(),
                    actual: "b".into(),
                },
                NO,
                "a verify mismatch is a comparison that came out no (was 4)",
            ),
            (
                CliError::Mismatch("not a cosigner".into()),
                NO,
                "membership: a plan it should belong to",
            ),
            (CliError::UsageError("bad flags".into()), USAGE, "usage"),
            (
                CliError::Invalid("bad xpub".into()),
                INVALID,
                "an input value",
            ),
            (
                CliError::Refused("output is an input".into()),
                REFUSED,
                "a destination refusal",
            ),
            (
                CliError::IoError(std::io::Error::other("boom")),
                IO,
                "io (was 1)",
            ),
            (CliError::Io("--in x: not found".into()), IO, "input file"),
            (CliError::Internal("serialize".into()), INTERNAL, "a bug"),
            (
                CliError::SetReassemblyMismatch {
                    group: "chunk_set_id 0x12345".into(),
                    detail: "does not reassemble".into(),
                },
                INVALID,
                "THE FUNDS FIX: a miscorrection rejection is 2",
            ),
        ]
    }

    /// The table itself (spec R13): each variant's class, stated beside the
    /// variant in an exhaustive match. A new variant does not compile until
    /// it is given a class here.
    // BEGIN expected_class
    fn expected_class(e: &CliError) -> (&'static str, u8) {
        match e {
            CliError::Codec(_) => ("Codec", INVALID),
            CliError::MdCodec(_) => ("MdCodec", INVALID),
            CliError::FutureFormat(_) => ("FutureFormat", INVALID),
            CliError::ContentMismatch { .. } => ("ContentMismatch", NO),
            CliError::UsageError(_) => ("UsageError", USAGE),
            CliError::Invalid(_) => ("Invalid", INVALID),
            CliError::Mismatch(_) => ("Mismatch", NO),
            CliError::Refused(_) => ("Refused", REFUSED),
            CliError::IoError(_) => ("IoError", IO),
            CliError::Io(_) => ("Io", IO),
            CliError::Internal(_) => ("Internal", INTERNAL),
            CliError::SetReassemblyMismatch { .. } => ("SetReassemblyMismatch", INVALID),
        }
    }
    // END expected_class

    #[test]
    fn every_arm_maps_to_its_documented_code() {
        for (e, want, why) in cases() {
            assert_eq!(e.exit_code(), want, "{why} ({:?})", e.kind());
            assert_eq!(expected_class(&e).1, want, "{why}: the table disagrees");
        }
    }

    /// Exhaustiveness guard: every arm of `expected_class` (read from this
    /// file's own text, so the count is not a literal) has a sample in
    /// `cases()` whose `exit_code()` is compared with the arm's class.
    #[test]
    fn every_variant_is_listed() {
        let src = include_str!("error.rs");
        let begin = src.find("// BEGIN expected_class").unwrap();
        let end = src.find("// END expected_class").unwrap();
        let arms: std::collections::BTreeSet<&str> = src[begin..end]
            .lines()
            .filter(|l| l.contains("=> (\""))
            .filter_map(|l| l.split('"').nth(1))
            .collect();
        assert!(arms.len() >= 12, "the arm parser found too few: {arms:?}");
        let seen: std::collections::BTreeSet<&str> = cases()
            .iter()
            .map(|(e, _, _)| {
                let (name, class) = expected_class(e);
                assert_eq!(e.exit_code(), class, "{name}");
                name
            })
            .collect();
        let missing: Vec<_> = arms.difference(&seen).collect();
        assert!(
            missing.is_empty(),
            "variants with no row in cases(): {missing:?}"
        );
    }

    #[test]
    fn non_utf8_input_is_invalid_not_io() {
        let e = CliError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ));
        assert_eq!(e.exit_code(), INVALID);
    }

    #[test]
    fn prefixed_keeps_the_class() {
        for (e, want, _) in cases() {
            assert_eq!(e.prefixed("x: ").exit_code(), want);
        }
    }
}
