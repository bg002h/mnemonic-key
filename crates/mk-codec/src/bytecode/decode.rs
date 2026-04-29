//! Top-level bytecode decoder: canonical `Vec<u8>` → `KeyCard`.
//!
//! Reverses [`crate::bytecode::encode::encode_bytecode`]; applies the
//! validity rules of `design/SPEC_mk_v0_1.md` §4.

use bitcoin::bip32::Fingerprint;

use crate::bytecode::header::BytecodeHeader;
use crate::bytecode::path::decode_path;
use crate::bytecode::xpub_compact::{decode_xpub_compact, reconstruct_xpub};
use crate::consts::{ORIGIN_FINGERPRINT_BYTES, POLICY_ID_STUB_BYTES};
use crate::error::{Error, Result};
use crate::key_card::KeyCard;

/// Decode canonical bytecode (pre-chunking) into a `KeyCard`.
///
/// Surfaces every SPEC §4 bytecode-layer validity rule via a unique
/// `Error` variant.
pub fn decode_bytecode(bytes: &[u8]) -> Result<KeyCard> {
    let mut cursor: &[u8] = bytes;

    let header_byte = read_u8(&mut cursor)?;
    let header = BytecodeHeader::parse(header_byte)?;

    let stub_count = read_u8(&mut cursor)?;
    if stub_count == 0 {
        return Err(Error::InvalidPolicyIdStubCount);
    }
    let mut policy_id_stubs: Vec<[u8; 4]> = Vec::with_capacity(stub_count as usize);
    for _ in 0..stub_count {
        let stub: [u8; POLICY_ID_STUB_BYTES] = read_array(&mut cursor)?;
        policy_id_stubs.push(stub);
    }

    let origin_fingerprint = if header.fingerprint_flag {
        let fp_bytes: [u8; ORIGIN_FINGERPRINT_BYTES] = read_array(&mut cursor)?;
        Some(Fingerprint::from(fp_bytes))
    } else {
        None
    };

    let origin_path = decode_path(&mut cursor)?;
    let compact = decode_xpub_compact(&mut cursor)?;
    let xpub = reconstruct_xpub(&compact, &origin_path)?;

    if !cursor.is_empty() {
        return Err(Error::TrailingBytes);
    }

    Ok(KeyCard {
        policy_id_stubs,
        origin_fingerprint,
        origin_path,
        xpub,
    })
}

fn read_u8(cursor: &mut &[u8]) -> Result<u8> {
    if cursor.is_empty() {
        return Err(Error::UnexpectedEnd);
    }
    let b = cursor[0];
    *cursor = &cursor[1..];
    Ok(b)
}

fn read_array<const N: usize>(cursor: &mut &[u8]) -> Result<[u8; N]> {
    if cursor.len() < N {
        return Err(Error::UnexpectedEnd);
    }
    let mut buf = [0u8; N];
    buf.copy_from_slice(&cursor[..N]);
    *cursor = &cursor[N..];
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::encode::encode_bytecode;
    use crate::bytecode::test_helpers::synthetic_xpub;
    use bitcoin::bip32::DerivationPath;
    use std::str::FromStr;

    fn fixture_card_1stub_with_fp() -> KeyCard {
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        KeyCard {
            policy_id_stubs: vec![[0xAA; 4]],
            origin_fingerprint: Some(Fingerprint::from([0xD3, 0x4D, 0xB3, 0x3F])),
            xpub: synthetic_xpub(&path),
            origin_path: path,
        }
    }

    fn fixture_card_3stubs_no_fp() -> KeyCard {
        let path = DerivationPath::from_str("m/48'/0'/0'/2'").unwrap();
        KeyCard {
            policy_id_stubs: vec![[0xAA; 4], [0xBB; 4], [0xCC; 4]],
            origin_fingerprint: None,
            xpub: synthetic_xpub(&path),
            origin_path: path,
        }
    }

    fn fixture_card_explicit_path() -> KeyCard {
        let path = DerivationPath::from_str("m/9999'/1234'/56'/7'").unwrap();
        KeyCard {
            policy_id_stubs: vec![[0x11, 0x22, 0x33, 0x44]],
            origin_fingerprint: Some(Fingerprint::from([0xAB, 0xCD, 0xEF, 0x01])),
            xpub: synthetic_xpub(&path),
            origin_path: path,
        }
    }

    #[test]
    fn round_trip_1stub_with_fp() {
        let card = fixture_card_1stub_with_fp();
        let wire = encode_bytecode(&card).unwrap();
        let decoded = decode_bytecode(&wire).unwrap();
        assert_eq!(decoded, card);
    }

    #[test]
    fn round_trip_3stubs_no_fp() {
        let card = fixture_card_3stubs_no_fp();
        let wire = encode_bytecode(&card).unwrap();
        let decoded = decode_bytecode(&wire).unwrap();
        assert_eq!(decoded, card);
    }

    #[test]
    fn round_trip_explicit_path() {
        let card = fixture_card_explicit_path();
        let wire = encode_bytecode(&card).unwrap();
        let decoded = decode_bytecode(&wire).unwrap();
        assert_eq!(decoded, card);
    }

    #[test]
    fn rejects_unsupported_version() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        wire[0] = 0x10; // version=1
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::UnsupportedVersion(1)),
        ));
    }

    #[test]
    fn rejects_reserved_bits_set() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        wire[0] |= 0b0000_0010; // bit 1 = reserved
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::ReservedBitsSet),
        ));
    }

    #[test]
    fn rejects_zero_stub_count() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        wire[1] = 0; // stub_count = 0
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::InvalidPolicyIdStubCount),
        ));
    }

    #[test]
    fn rejects_invalid_path_indicator() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        // path indicator is at offset 1+1+4+4 = 10
        wire[10] = 0x16; // reserved-pending-md1
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::InvalidPathIndicator(0x16)),
        ));
    }

    #[test]
    fn rejects_invalid_xpub_version() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        // xpub_compact version is at offset 1+1+4+4+1 = 11
        wire[11] = 0xDE;
        wire[12] = 0xAD;
        wire[13] = 0xBE;
        wire[14] = 0xEF;
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::InvalidXpubVersion(0xDEADBEEF)),
        ));
    }

    #[test]
    fn rejects_trailing_bytes() {
        let card = fixture_card_1stub_with_fp();
        let mut wire = encode_bytecode(&card).unwrap();
        wire.push(0xFF); // extra byte after xpub
        assert!(matches!(
            decode_bytecode(&wire),
            Err(Error::TrailingBytes),
        ));
    }

    #[test]
    fn rejects_truncated_mid_stub() {
        let card = fixture_card_1stub_with_fp();
        let wire = encode_bytecode(&card).unwrap();
        let truncated = &wire[..4]; // header + count + 2/4 stub bytes
        assert!(matches!(
            decode_bytecode(truncated),
            Err(Error::UnexpectedEnd),
        ));
    }
}
