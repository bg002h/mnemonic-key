//! Generator for the canonical mk-codec v0.1 vector corpus.
//!
//! Run via:
//!
//! ```text
//! cargo run --bin gen_mk_vectors --features gen-vectors -- \
//!   --output crates/mk-codec/tests/vectors/v0.1.json
//! ```
//!
//! Cross-implementations validate against the JSON file's pinned
//! SHA-256 (in `crates/mk-codec/tests/vectors.rs`) plus per-vector
//! round-trip equality. The output is byte-deterministic in the
//! fixture set so re-runs produce identical files; see the
//! "Canonicality discipline" block below for the rules.
//!
//! ## Canonicality discipline
//!
//! - Keys sorted alphabetically at every nesting level — `serde_json::Map`
//!   is `BTreeMap`-backed by default, which sorts on insertion.
//! - Hex literals lowercase — emitted via the `lowercase_hex` helper here
//!   (the `bitcoin::hashes` crate's hex encoders are also lowercase by
//!   default but re-implementing here keeps the dependency surface small).
//! - Byte-array fields rendered as continuous hex strings (no `0x` prefix,
//!   no separators).
//! - Indentation: 2 spaces — `serde_json::ser::PrettyFormatter::with_indent(b"  ")`.
//! - Line endings: LF; trailing newline at EOF — appended manually.
//! - Per-vector `chunk_set_id` is fixed so chunked encodings are
//!   byte-deterministic.

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use bitcoin::NetworkKind;
use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use mk_codec::{KeyCard, bytecode::encode_bytecode, encode_with_chunk_set_id};
use serde_json::{Value, json};

/// One fixture spec — abstract enough to drop a new vector by adding
/// one entry to [`fixtures`] without touching the emit code below.
struct FixtureSpec {
    /// Vector identifier (e.g., `V1_bip48_mainnet_1_stub`).
    name: &'static str,
    /// One-line human-readable description of what the vector exercises.
    description: &'static str,
    /// Policy ID stub bytes (4 bytes per stub; min 1 stub per closure §4 rule 3).
    policy_id_stubs: Vec<[u8; 4]>,
    /// Master-key fingerprint, or `None` for the privacy-preserving mode
    /// (closure Q-8 — bytecode-header bit 2 cleared).
    origin_fingerprint: Option<[u8; 4]>,
    /// BIP 32 origin path. Must round-trip through `DerivationPath::from_str`
    /// and serialize identically via `Display`.
    origin_path: &'static str,
    /// Mainnet (`NetworkKind::Main`) vs testnet (`NetworkKind::Test`).
    /// Affects the xpub.version field in the compact-73 wire form.
    network: NetworkKind,
    /// Seed byte used to deterministically derive the synthetic
    /// secret key. Distinct seeds produce distinct xpubs across the
    /// vector set, which makes vector inspection tractable when one
    /// fails.
    seed_byte: u8,
    /// Pinned `chunk_set_id` for byte-deterministic chunked encoding.
    /// Per closure Q-5 the 20-bit field is opaque; vectors use
    /// memorable hex digits (0x12345, 0xABCDE, …) rather than zero
    /// to make hand-debugging easier.
    chunk_set_id: u32,
}

fn fixtures() -> Vec<FixtureSpec> {
    vec![
        FixtureSpec {
            name: "V1_bip48_mainnet_1_stub_with_fp",
            description: "1-stub mainnet, BIP 48 segwit-v0 multisig (m/48'/0'/0'/2'), \
                 fingerprint present. Typical multisig recovery card.",
            policy_id_stubs: vec![[0x11, 0x22, 0x33, 0x44]],
            origin_fingerprint: Some([0xAA, 0xBB, 0xCC, 0xDD]),
            origin_path: "48'/0'/0'/2'",
            network: NetworkKind::Main,
            seed_byte: 0x01,
            chunk_set_id: 0x12345,
        },
        FixtureSpec {
            name: "V2_bip84_mainnet_1_stub_with_fp",
            description: "1-stub mainnet, BIP 84 native-segwit single-sig (m/84'/0'/0'), \
                 fingerprint present. Std-table indicator 0x03.",
            policy_id_stubs: vec![[0xC0, 0xFF, 0xEE, 0x00]],
            origin_fingerprint: Some([0xDE, 0xAD, 0xBE, 0xEF]),
            origin_path: "84'/0'/0'",
            network: NetworkKind::Main,
            seed_byte: 0x02,
            chunk_set_id: 0x23456,
        },
        FixtureSpec {
            name: "V3_bip48_testnet_1_stub_with_fp",
            description: "1-stub testnet, BIP 48 testnet multisig (m/48'/1'/0'/2'), \
                 fingerprint present. Std-table indicator 0x15.",
            policy_id_stubs: vec![[0x77, 0x88, 0x99, 0xAA]],
            origin_fingerprint: Some([0x10, 0x20, 0x30, 0x40]),
            origin_path: "48'/1'/0'/2'",
            network: NetworkKind::Test,
            seed_byte: 0x03,
            chunk_set_id: 0x34567,
        },
        FixtureSpec {
            name: "V4_bip84_mainnet_1_stub_no_fp",
            description: "1-stub mainnet, BIP 84 (m/84'/0'/0'), fingerprint omitted \
                 (privacy-preserving mode; bytecode-header bit 2 cleared).",
            policy_id_stubs: vec![[0xAB, 0xCD, 0xEF, 0x01]],
            origin_fingerprint: None,
            origin_path: "84'/0'/0'",
            network: NetworkKind::Main,
            seed_byte: 0x04,
            chunk_set_id: 0x45678,
        },
        FixtureSpec {
            name: "V5_explicit_path_4_components_with_fp",
            description: "1-stub mainnet, explicit-path m/9999'/1234'/56'/7' (forces \
                 the 0xFE explicit-path codec), fingerprint present.",
            policy_id_stubs: vec![[0x55, 0x66, 0x77, 0x88]],
            origin_fingerprint: Some([0x01, 0x02, 0x03, 0x04]),
            origin_path: "9999'/1234'/56'/7'",
            network: NetworkKind::Main,
            seed_byte: 0x05,
            chunk_set_id: 0x56789,
        },
        FixtureSpec {
            name: "V6_3_stubs_mainnet_with_fp",
            description: "3-stub mainnet, BIP 48 multisig — exercises multi-stub \
                 listing that grows the bytecode by 2 × 4 bytes vs V1.",
            policy_id_stubs: vec![
                [0xDE, 0xAD, 0x00, 0x01],
                [0xDE, 0xAD, 0x00, 0x02],
                [0xDE, 0xAD, 0x00, 0x03],
            ],
            origin_fingerprint: Some([0xF0, 0x0D, 0xCA, 0xFE]),
            origin_path: "48'/0'/0'/2'",
            network: NetworkKind::Main,
            seed_byte: 0x06,
            chunk_set_id: 0x67890,
        },
        FixtureSpec {
            name: "V7_max_path_components_no_fp",
            description: "1-stub mainnet, explicit-path at the 10-component cap \
                 (m/0'/1'/2'/3'/4'/5'/6'/7'/8'/9'), fingerprint omitted. \
                 Boundary case for path-cap validation (closure Q-3).",
            policy_id_stubs: vec![[0x90, 0x91, 0x92, 0x93]],
            origin_fingerprint: None,
            origin_path: "0'/1'/2'/3'/4'/5'/6'/7'/8'/9'",
            network: NetworkKind::Main,
            seed_byte: 0x07,
            chunk_set_id: 0x78901,
        },
        FixtureSpec {
            name: "V8_bip87_mainnet_1_stub_with_fp",
            description: "1-stub mainnet, BIP 87 multisig (m/87'/0'/0'), \
                 fingerprint present. Std-table indicator 0x07 (the last \
                 mainnet entry of the closure-locked path dictionary).",
            policy_id_stubs: vec![[0x87, 0x65, 0x43, 0x21]],
            origin_fingerprint: Some([0xBA, 0xDD, 0xCA, 0xFE]),
            origin_path: "87'/0'/0'",
            network: NetworkKind::Main,
            seed_byte: 0x08,
            chunk_set_id: 0x89012,
        },
    ]
}

/// Build a deterministic synthetic xpub for a fixture.
///
/// The resulting xpub is a valid BIP 32 extended public key (real
/// secp256k1 point on the curve), but the parent_fingerprint and
/// chain_code are fixed test values rather than derived from a
/// chain-of-trust. Decoders will accept the xpub at the wire level;
/// real-world recovery would re-verify against an external Wallet
/// Instance ID anchor (per SPEC §5).
fn synthetic_xpub(network: NetworkKind, seed_byte: u8, path: &DerivationPath) -> Xpub {
    let secp = Secp256k1::new();
    let secret_bytes = [seed_byte; 32];
    let sk =
        SecretKey::from_slice(&secret_bytes).expect("non-zero seed must be a valid secret key");
    let pk = PublicKey::from_secret_key(&secp, &sk);
    let components: Vec<ChildNumber> = path.into_iter().copied().collect();
    let depth = components.len() as u8;
    let child_number = components
        .last()
        .copied()
        .unwrap_or(ChildNumber::Normal { index: 0 });
    Xpub {
        network,
        depth,
        // Distinct from seed_byte so an attacker can't trivially derive
        // it from public knowledge of the fixture's secret. Vectors
        // exist for testing wire-format conformance, not security.
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, seed_byte]),
        child_number,
        public_key: pk,
        chain_code: ChainCode::from([seed_byte ^ 0xAA; 32]),
    }
}

fn lowercase_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        // `format_args!` would allocate; build the string manually for
        // determinism + lowercase-hex enforcement.
        const HEX: &[u8; 16] = b"0123456789abcdef";
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0F) as usize] as char);
    }
    s
}

fn fixture_to_value(spec: &FixtureSpec) -> Value {
    let path: DerivationPath =
        DerivationPath::from_str(spec.origin_path).expect("fixture origin_path must parse");
    let xpub = synthetic_xpub(spec.network, spec.seed_byte, &path);

    let card = KeyCard::new(
        spec.policy_id_stubs.clone(),
        spec.origin_fingerprint.map(Fingerprint::from),
        path.clone(),
        xpub,
    );

    let bytecode = encode_bytecode(&card).expect("encode_bytecode succeeds for valid fixture");
    let strings = encode_with_chunk_set_id(&card, spec.chunk_set_id)
        .expect("encode_with_chunk_set_id succeeds for valid fixture");

    let stubs_json: Vec<Value> = spec
        .policy_id_stubs
        .iter()
        .map(|s| Value::String(lowercase_hex(s)))
        .collect();
    let fp_json = match spec.origin_fingerprint {
        Some(fp) => Value::String(lowercase_hex(&fp)),
        None => Value::Null,
    };

    json!({
        "name": spec.name,
        "description": spec.description,
        "input": {
            "chunk_set_id": spec.chunk_set_id,
            "network": match spec.network {
                NetworkKind::Main => "mainnet",
                NetworkKind::Test => "testnet",
            },
            "origin_fingerprint": fp_json,
            "origin_path": format!("m/{}", path),
            "policy_id_stubs": stubs_json,
            "xpub": xpub.to_string(),
        },
        "expected": {
            "canonical_bytecode_hex": lowercase_hex(&bytecode),
            "decoder_correction": "clean",
            "strings": strings,
            "total_chunks": strings.len(),
        },
    })
}

fn main() {
    // Resolve --output (default: crates/mk-codec/tests/vectors/v0.1.json
    // relative to the workspace root, which is the cwd when run via cargo).
    let mut args = env::args().skip(1);
    let mut output_path: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" | "-o" => {
                output_path = Some(PathBuf::from(
                    args.next().expect("--output requires a path"),
                ));
            }
            other => panic!("unrecognised argument: {other}"),
        }
    }
    let output_path =
        output_path.unwrap_or_else(|| PathBuf::from("crates/mk-codec/tests/vectors/v0.1.json"));

    let vectors_json: Vec<Value> = fixtures().iter().map(fixture_to_value).collect();
    let document = json!({
        "schema": 1,
        "family_token": mk_codec::GENERATOR_FAMILY,
        "vectors": vectors_json,
    });

    // Pretty-print with 2-space indent + lowercase hex (already enforced
    // upstream in `lowercase_hex`). `serde_json::Map` is BTreeMap-backed
    // by default so keys sort alphabetically at every level. Default
    // `PrettyFormatter` uses a 2-space indent, matching the canonicality
    // discipline pinned in the module-level docs.
    let mut buf: Vec<u8> = Vec::new();
    serde_json::to_writer_pretty(&mut buf, &document)
        .expect("serializing pre-built Value cannot fail");
    // Trailing newline at EOF, LF line endings.
    buf.push(b'\n');

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("create vector output directory");
    }
    let mut out = fs::File::create(&output_path).expect("create output file");
    out.write_all(&buf).expect("write vector JSON");
    out.flush().expect("flush vector JSON");

    eprintln!(
        "wrote {} vectors to {} ({} bytes)",
        fixtures().len(),
        output_path.display(),
        buf.len()
    );
}
