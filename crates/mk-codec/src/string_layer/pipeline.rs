//! Public encode/decode entry points: `KeyCard` ↔ `Vec<String>`.
//!
//! The encoder is the layer-3 boundary at which the canonical bytecode
//! produced by [`crate::bytecode::encode_bytecode`] is wrapped in
//! BCH-checksummed mk1 strings. Two emission paths:
//!
//! - **Single string** — bytecode fits in
//!   [`crate::consts::SINGLE_STRING_LONG_BYTES`] (= 56). Emits a single
//!   `mk1`-prefixed string with a 2-symbol header and no cross-chunk hash.
//! - **Chunked** — bytecode exceeds the single-string ceiling. Appends the
//!   4-byte `cross_chunk_hash`, splits the resulting stream into chunks
//!   of at most [`crate::consts::CHUNKED_FRAGMENT_LONG_BYTES`] (= 53)
//!   bytes, and emits one `mk1` string per chunk with an 8-symbol header.
//!
//! v0.1 emit policy: each emitted string's per-chunk BCH code variant
//! (regular vs long) is auto-selected by
//! [`crate::string_layer::bch::encode_5bit_to_string`] from the resulting
//! 5-bit-symbol data-part length. For typical mk1 cards (≈84 bytes
//! bytecode → 88-byte stream → fragments of 53 + 35 bytes), this means
//! chunk 0 lands in long-code territory and the trailing short chunk
//! falls back to regular code. Decoders accept either per-chunk
//! variant — mixed-code emit is wire-permitted by design.

use crate::bytecode::{decode_bytecode, encode_bytecode};
use crate::consts::SINGLE_STRING_LONG_BYTES;
use crate::error::{Error, Result};
use crate::key_card::KeyCard;
use crate::string_layer::bch::{
    bytes_to_5bit, decode_string, encode_5bit_to_string, five_bit_to_bytes,
};
use crate::string_layer::chunk::{ChunkFragment, reassemble_from_chunks, split_into_chunks};
use crate::string_layer::header::{MAX_CHUNK_SET_ID, StringLayerHeader, VERSION_V0_1};

/// Draw a fresh 20-bit `chunk_set_id` from the system CSPRNG via
/// [`getrandom`]. The OS entropy source is used to avoid pulling a
/// full RNG framework into the codec — `getrandom` is the same crate
/// that backs `rand`'s `OsRng`, so the entropy quality is identical.
///
/// Per closure Q-5, the `chunk_set_id` is opaque and only used for
/// reassembly mismatch detection, so any uniformly-distributed 20-bit
/// value is sufficient. Failure to read entropy is treated as an
/// unrecoverable system error and panics; this matches the failure
/// mode of `rand::thread_rng()` and is acceptable for an encode call
/// because no key material has been emitted at the point of failure.
fn fresh_chunk_set_id() -> u32 {
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf).expect("OS CSPRNG must be available for mk1 encode");
    u32::from_be_bytes(buf) & MAX_CHUNK_SET_ID
}

/// Encode a `KeyCard` into one or more `mk1`-prefixed strings.
///
/// Multi-chunk encodings draw a fresh 20-bit `chunk_set_id` from the
/// system CSPRNG (`OsRng`). Use [`encode_with_chunk_set_id`] to pin the
/// value for deterministic output (vector regeneration, conformance tests).
pub fn encode(card: &KeyCard) -> Result<Vec<String>> {
    let bytecode = encode_bytecode(card)?;
    encode_bytecode_stream(&bytecode, None)
}

/// Like [`encode`], but with an explicit `chunk_set_id` override.
///
/// `chunk_set_id` MUST fit in 20 bits (`0..=0x000F_FFFF`); otherwise
/// returns [`Error::ChunkedHeaderMalformed`]. The override is only
/// consulted on the chunked path; single-string encodings have no
/// `chunk_set_id` field, so the value is silently ignored.
pub fn encode_with_chunk_set_id(card: &KeyCard, chunk_set_id: u32) -> Result<Vec<String>> {
    let bytecode = encode_bytecode(card)?;
    encode_bytecode_stream(&bytecode, Some(chunk_set_id))
}

fn encode_bytecode_stream(bytecode: &[u8], chunk_set_id: Option<u32>) -> Result<Vec<String>> {
    if bytecode.len() <= SINGLE_STRING_LONG_BYTES {
        // SingleString path: 2-symbol header + bytes_to_5bit(bytecode).
        let header = StringLayerHeader::SingleString {
            version: VERSION_V0_1,
        };
        let mut data_5bit = header.to_5bit_symbols();
        data_5bit.extend(bytes_to_5bit(bytecode));
        let s = encode_5bit_to_string(&data_5bit)?;
        return Ok(vec![s]);
    }

    // Chunked path: derive (or use override) chunk_set_id, then split.
    let csid = match chunk_set_id {
        Some(v) => {
            if v > MAX_CHUNK_SET_ID {
                return Err(Error::ChunkedHeaderMalformed(format!(
                    "chunk_set_id {v:#x} exceeds 20-bit field"
                )));
            }
            v
        }
        None => fresh_chunk_set_id(),
    };

    let chunks = split_into_chunks(bytecode, csid)?;
    let mut strings = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        let mut data_5bit = chunk.header.to_5bit_symbols();
        data_5bit.extend(bytes_to_5bit(&chunk.fragment));
        strings.push(encode_5bit_to_string(&data_5bit)?);
    }
    Ok(strings)
}

/// Decode one or more `mk1`-prefixed strings into a `KeyCard`.
///
/// Supports both single-string and chunked inputs:
/// - One string with `SingleString` header → decode bytecode directly.
/// - One or more strings with `Chunked` headers → reassemble with
///   cross-chunk-hash verification, then decode the bytecode.
///
/// Mixing `SingleString` and `Chunked` headers across a multi-string
/// input is rejected with [`Error::ChunkedHeaderMalformed`].
pub fn decode(strings: &[&str]) -> Result<KeyCard> {
    if strings.is_empty() {
        return Err(Error::ChunkedHeaderMalformed(
            "empty input string list".to_string(),
        ));
    }

    // Decode each string at the BCH layer; collect (header, fragment_bytes).
    let mut parsed: Vec<(StringLayerHeader, Vec<u8>)> = Vec::with_capacity(strings.len());
    for s in strings {
        let decoded = decode_string(s)?;
        let data_5bit = decoded.data();
        let (header, consumed) = StringLayerHeader::from_5bit_symbols(data_5bit)?;
        let payload_5bit = &data_5bit[consumed..];
        let fragment = five_bit_to_bytes(payload_5bit).ok_or(Error::MalformedPayloadPadding)?;
        parsed.push((header, fragment));
    }

    let first_is_single = matches!(parsed[0].0, StringLayerHeader::SingleString { .. });
    if first_is_single {
        if parsed.len() != 1 {
            return Err(Error::ChunkedHeaderMalformed(
                "multiple strings supplied with SingleString header".to_string(),
            ));
        }
        let (_, bytecode) = parsed.into_iter().next().expect("len == 1");
        return decode_bytecode(&bytecode);
    }

    // Chunked path: consume all into ChunkFragment list and reassemble.
    let chunks: Vec<ChunkFragment> = parsed
        .into_iter()
        .map(|(header, fragment)| ChunkFragment { header, fragment })
        .collect();
    let bytecode = reassemble_from_chunks(chunks)?;
    decode_bytecode(&bytecode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::test_helpers::synthetic_xpub;
    use bitcoin::bip32::{DerivationPath, Fingerprint};
    use std::str::FromStr;

    fn fixture_card_typical_chunked() -> KeyCard {
        // 1 stub + std-table indicator + fingerprint + 73-byte compact xpub
        // = 84 bytes; this exceeds SINGLE_STRING_LONG_BYTES (= 56) and
        // therefore lands in the chunked path. (`xpub_compact` alone is
        // already 73 bytes, so no realistic mk1 card fits in a single
        // string — SingleString remains reachable only through hand-
        // constructed sub-card test inputs.) The "singlestring_fits" name
        // is historical and predates the closure-locked compact-73 form.
        let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
        KeyCard {
            policy_id_stubs: vec![[0x11, 0x22, 0x33, 0x44]],
            origin_fingerprint: Some(Fingerprint::from([0xAA, 0xBB, 0xCC, 0xDD])),
            origin_path: path.clone(),
            xpub: synthetic_xpub(&path),
        }
    }

    fn fixture_card_explicit_path_long() -> KeyCard {
        // Explicit-path forces a longer bytecode; tests multi-chunk path
        // explicitly even though typical cards already chunk.
        let path = DerivationPath::from_str("9999'/1234'/56'/7'/0/1/2/3").unwrap();
        KeyCard {
            policy_id_stubs: vec![[0xDE, 0xAD, 0xBE, 0xEF]],
            origin_fingerprint: Some(Fingerprint::from([0x01, 0x02, 0x03, 0x04])),
            origin_path: path.clone(),
            xpub: synthetic_xpub(&path),
        }
    }

    #[test]
    fn round_trip_typical_card_chunked() {
        let card = fixture_card_typical_chunked();
        let strings = encode_with_chunk_set_id(&card, 0x12345).unwrap();
        let parts: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
        let recovered = decode(&parts).unwrap();
        assert_eq!(recovered, card);
    }

    #[test]
    fn round_trip_explicit_path_chunked() {
        let card = fixture_card_explicit_path_long();
        let strings = encode_with_chunk_set_id(&card, 0xABCDE).unwrap();
        assert!(strings.len() >= 2, "explicit-path card must chunk");
        let parts: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
        let recovered = decode(&parts).unwrap();
        assert_eq!(recovered, card);
    }

    #[test]
    fn deterministic_encoding_with_explicit_chunk_set_id() {
        // encode_with_chunk_set_id MUST be byte-deterministic; this is the
        // property Phase 6 vector regeneration depends on.
        let card = fixture_card_typical_chunked();
        let s1 = encode_with_chunk_set_id(&card, 0x12345).unwrap();
        let s2 = encode_with_chunk_set_id(&card, 0x12345).unwrap();
        assert_eq!(s1, s2);
    }

    #[test]
    fn random_chunk_set_id_decodes_round_trip() {
        // encode (CSPRNG-derived chunk_set_id) round-trips even though we
        // don't pin the chunk_set_id value — the decoder doesn't care
        // about the value, only that it's consistent across chunks.
        let card = fixture_card_typical_chunked();
        let strings = encode(&card).unwrap();
        let parts: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
        let recovered = decode(&parts).unwrap();
        assert_eq!(recovered, card);
    }

    #[test]
    fn random_chunk_set_id_fits_20_bits() {
        // Inspect the produced strings' chunk_set_id field; assert it's
        // masked to 20 bits, no spillover from a u32 RNG.
        let card = fixture_card_typical_chunked();
        let strings = encode(&card).unwrap();
        // The first chunk's parsed header carries the chunk_set_id.
        let s0 = &strings[0];
        let decoded = decode_string(s0).unwrap();
        let (header, _consumed) = StringLayerHeader::from_5bit_symbols(decoded.data()).unwrap();
        match header {
            StringLayerHeader::Chunked { chunk_set_id, .. } => {
                assert!(
                    chunk_set_id <= MAX_CHUNK_SET_ID,
                    "chunk_set_id {chunk_set_id:#x} > 20-bit max"
                );
            }
            StringLayerHeader::SingleString { .. } => {
                // Card unexpectedly fit in single-string; nothing to check.
            }
        }
    }

    #[test]
    fn encode_with_chunk_set_id_rejects_oversized_value() {
        let card = fixture_card_typical_chunked();
        let r = encode_with_chunk_set_id(&card, 0x10_0000);
        assert!(matches!(r, Err(Error::ChunkedHeaderMalformed(_))));
    }

    #[test]
    fn decode_rejects_chunk_set_id_mismatch() {
        let card = fixture_card_typical_chunked();
        let strings = encode_with_chunk_set_id(&card, 0x12345).unwrap();
        // Re-encode under a different chunk_set_id and splice in chunk 1.
        let other = encode_with_chunk_set_id(&card, 0x67890).unwrap();
        let mixed: Vec<&str> = vec![strings[0].as_str(), other[1].as_str()];
        assert!(matches!(decode(&mixed), Err(Error::ChunkSetIdMismatch)));
    }

    #[test]
    fn decode_rejects_perturbed_cross_chunk_hash() {
        // Bit-flip the last byte of the last chunk's encoded string within
        // the data part. The BCH layer will correct it to *something* (since
        // BCH t=4 covers single substitutions), but we want the cross-chunk
        // hash to mismatch. Easier: produce strings, then re-build chunks
        // manually with the cross-chunk hash perturbed at the byte level.

        let card = fixture_card_typical_chunked();
        let bytecode = encode_bytecode(&card).unwrap();
        // Force-chunk and perturb the cross-chunk hash byte (last byte of
        // the last chunk's fragment in the post-split layout).
        let mut chunks = split_into_chunks(&bytecode, 0).unwrap();
        let last = chunks.last_mut().unwrap();
        let last_idx = last.fragment.len() - 1;
        last.fragment[last_idx] ^= 0xFF;

        // Re-encode each (now-perturbed) chunk to a string, then decode.
        let strings: Vec<String> = chunks
            .into_iter()
            .map(|chunk| {
                let mut data_5bit = chunk.header.to_5bit_symbols();
                data_5bit.extend(bytes_to_5bit(&chunk.fragment));
                encode_5bit_to_string(&data_5bit).unwrap()
            })
            .collect();
        let parts: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
        // Reassembly must reject the cross-chunk hash mismatch (or a
        // downstream bytecode-decode error if the perturbation also
        // disturbed the bytecode tail; surface either as a defined
        // rejection rather than a silent false-positive).
        match decode(&parts) {
            Err(Error::CrossChunkHashMismatch) => (),
            other => panic!("expected CrossChunkHashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn decode_rejects_singlestring_padding_bits_nonzero() {
        // Construct a SingleString-style mk1 string whose 5-bit payload
        // doesn't byte-align (trailing pad bits non-zero).
        // Use a bytecode of 1 byte, then pad with a stray 5-bit symbol that
        // sets the pad bits non-zero.
        let header = StringLayerHeader::SingleString {
            version: VERSION_V0_1,
        };
        // 1 byte (e.g., 0x00) → 2 5-bit symbols (00, 00).  Adding a third
        // 5-bit symbol with non-zero low 2 bits inflates the data to 3
        // payload symbols whose final pad bits are non-zero.
        let mut data_5bit = header.to_5bit_symbols();
        data_5bit.extend([0u8, 0u8, 0b00011u8]); // last symbol's low 2 bits = 11
        let s = encode_5bit_to_string(&data_5bit).unwrap();
        let r = decode(&[&s]);
        assert!(matches!(r, Err(Error::MalformedPayloadPadding)));
    }

    #[test]
    fn decode_rejects_empty_input() {
        assert!(matches!(decode(&[]), Err(Error::ChunkedHeaderMalformed(_))));
    }
}
