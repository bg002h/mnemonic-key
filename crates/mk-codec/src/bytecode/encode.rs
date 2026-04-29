//! Top-level bytecode encoder: `KeyCard` → canonical `Vec<u8>`.
//!
//! Per `design/SPEC_mk_v0_1.md` §3.2 payload field order (closure Q-6):
//!
//! ```text
//! [bytecode_header   : 1 B]
//! [stub_count        : 1 B; MUST be ≥ 1]
//! [policy_id_stubs   : 4 × N B]
//! [origin_fingerprint: 4 B]   ← present iff bytecode_header bit 2 set
//! [origin_path       : variable]
//! [xpub_compact      : 73 B]
//! ```

use crate::bytecode::header::BytecodeHeader;
use crate::bytecode::path::encode_path;
use crate::bytecode::xpub_compact::{XpubCompact, encode_xpub_compact};
use crate::error::{Error, Result};
use crate::key_card::KeyCard;

/// Encode a `KeyCard` to its canonical bytecode form (pre-chunking).
pub fn encode_bytecode(card: &KeyCard) -> Result<Vec<u8>> {
    if card.policy_id_stubs.is_empty() {
        return Err(Error::InvalidPolicyIdStubCount);
    }
    if card.policy_id_stubs.len() > u8::MAX as usize {
        return Err(Error::InvalidPolicyIdStubCount);
    }

    let header = BytecodeHeader {
        version: 0,
        fingerprint_flag: card.origin_fingerprint.is_some(),
    };

    let mut out: Vec<u8> = Vec::new();
    out.push(header.to_byte());
    out.push(card.policy_id_stubs.len() as u8);
    for stub in &card.policy_id_stubs {
        out.extend_from_slice(stub);
    }
    if let Some(fp) = &card.origin_fingerprint {
        out.extend_from_slice(fp.as_bytes());
    }
    out.extend_from_slice(&encode_path(&card.origin_path));
    let compact = XpubCompact::from_xpub(&card.xpub);
    encode_xpub_compact(&compact, &mut out);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::test_helpers::synthetic_xpub;
    use bitcoin::bip32::{DerivationPath, Fingerprint};
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

    #[test]
    fn encodes_typical_1stub_card_to_84_bytes() {
        let card = fixture_card_1stub_with_fp();
        let wire = encode_bytecode(&card).unwrap();
        // header(1) + stub_count(1) + 1*stub(4) + fp(4) + std-table indicator(1) + xpub_compact(73) = 84
        assert_eq!(wire.len(), 84);
        assert_eq!(wire[0], 0x04, "fingerprint flag set");
        assert_eq!(wire[1], 1, "stub_count = 1");
        assert_eq!(&wire[2..6], &[0xAA; 4], "stub bytes");
        assert_eq!(&wire[6..10], &[0xD3, 0x4D, 0xB3, 0x3F], "fp bytes");
        assert_eq!(wire[10], 0x05, "std-table indicator for m/48'/0'/0'/2'");
    }

    #[test]
    fn encodes_card_without_fingerprint_to_80_bytes() {
        let mut card = fixture_card_1stub_with_fp();
        card.origin_fingerprint = None;
        let wire = encode_bytecode(&card).unwrap();
        // 84 - 4 (omitted fp) = 80
        assert_eq!(wire.len(), 80);
        assert_eq!(wire[0], 0x00, "fingerprint flag unset");
    }

    #[test]
    fn rejects_zero_stubs() {
        let mut card = fixture_card_1stub_with_fp();
        card.policy_id_stubs.clear();
        assert!(matches!(
            encode_bytecode(&card),
            Err(Error::InvalidPolicyIdStubCount),
        ));
    }

    #[test]
    fn deterministic_output() {
        let card = fixture_card_1stub_with_fp();
        let a = encode_bytecode(&card).unwrap();
        let b = encode_bytecode(&card).unwrap();
        assert_eq!(a, b, "encoder must be byte-deterministic");
    }
}
