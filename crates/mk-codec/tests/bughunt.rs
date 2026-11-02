//! Bug-hunt triage rows (mk). Each test asserts the CORRECT behaviour.

use std::str::FromStr;

use bitcoin::bip32::{DerivationPath, Fingerprint};
use mk_codec::bytecode::path::{decode_path, encode_path};
use mk_codec::string_layer::bch::{
    bch_correct_long, bch_correct_regular, bch_verify_long, bch_verify_regular, decode_string,
};
use mk_codec::{KeyCard, decode, encode_with_chunk_set_id};

fn multi_chunk_card() -> KeyCard {
    let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let sk = bitcoin::secp256k1::SecretKey::from_slice(&[0x42u8; 32]).unwrap();
    let pk = bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &sk);
    let comps: Vec<bitcoin::bip32::ChildNumber> = path.as_ref().to_vec();
    let xpub = bitcoin::bip32::Xpub {
        network: bitcoin::NetworkKind::Main,
        depth: comps.len() as u8,
        parent_fingerprint: Fingerprint::from([0x10, 0x20, 0x30, 0x40]),
        child_number: *comps.last().unwrap(),
        public_key: pk,
        chain_code: bitcoin::bip32::ChainCode::from([0xCCu8; 32]),
    };
    KeyCard::new(
        (0u8..6).map(|i| [i, i, i, i]).collect(),
        Some(Fingerprint::from([0xAA, 0xBB, 0xCC, 0xDD])),
        path,
        xpub,
    )
}

/// Grade Q2: minimal LEB128 is an encoder rule, not a decoder refusal. A
/// non-minimal component (`[0x80, 0x00]`, i.e. 0 with a trailing zero group)
/// decodes to the same u32 as the minimal form, and re-encoding gives the
/// minimal bytes `[fe, 01, 00]`.
#[test]
fn bughunt_mk_02_leb128_non_minimal_decodes_to_same_value() {
    let bytes = [0xFEu8, 0x01, 0x80, 0x00];
    let mut cur = &bytes[..];
    let p = decode_path(&mut cur).expect("non-minimal component decodes");
    assert!(cur.is_empty(), "all 4 bytes consumed, left {cur:02x?}");
    assert_eq!(p, DerivationPath::from_str("m/0").unwrap());
    assert_eq!(encode_path(&p), vec![0xFE, 0x01, 0x00]);
}

/// Lead L5: a non-ASCII character must be refused, never decoded.
#[test]
fn bughunt_mk_05_decode_string_non_ascii_refused() {
    let strings = encode_with_chunk_set_id(&multi_chunk_card(), 0).unwrap();
    let mut bad: Vec<String> = Vec::new();
    for s in [&strings[0], strings.last().unwrap()] {
        let chars: Vec<char> = s.chars().collect();
        // Non-ASCII chars with interesting case folds: Kelvin sign
        // (lowercases to ASCII 'k'), U+0130 (lowercases to TWO chars),
        // fullwidth '1' and 'm' (no fold), a 2-byte letter.
        let probes = ['\u{212A}', '\u{0130}', '\u{FF11}', '\u{FF4D}', '\u{00E9}'];
        for &p in &probes {
            // HRP region: positions 0, 1; separator: 2; data: several.
            for pos in [0usize, 1, 2, 3, 4, 10, chars.len() / 2, chars.len() - 1] {
                let mut c = chars.clone();
                c[pos] = p;
                bad.push(c.into_iter().collect());
            }
        }
        // Kelvin sign replacing a genuine 'k' in the data part, if any.
        for (i, &ch) in chars.iter().enumerate().skip(3) {
            if ch == 'k' {
                let mut c = chars.clone();
                c[i] = '\u{212A}';
                bad.push(c.into_iter().collect());
                break;
            }
        }
        // Kelvin sign in the HRP while the rest stays valid: "m<K>1...".
        let mut c = chars.clone();
        c[1] = '\u{212A}';
        bad.push(c.into_iter().collect());
        // Whole-string uppercase + a Kelvin sign for the 'K'.
        let up: String = s.to_uppercase();
        let mut c: Vec<char> = up.chars().collect();
        c[1] = '\u{212A}';
        bad.push(c.into_iter().collect());
    }
    let mut accepted = Vec::new();
    for b in &bad {
        let r = std::panic::catch_unwind(|| decode_string(b).is_ok());
        match r {
            Err(_) => panic!("decode_string panicked on {b:?}"),
            Ok(true) => accepted.push(b.clone()),
            Ok(false) => {}
        }
    }
    for a in &accepted {
        eprintln!("ACCEPTED {}", a.escape_unicode());
    }
    assert!(
        accepted.is_empty(),
        "{} of {} non-ASCII strings were decoded instead of refused; first: {:?}",
        accepted.len(),
        bad.len(),
        accepted.first()
    );
}

/// Grade Q5: the variant and position of a non-ASCII refusal. In the data
/// part it is `InvalidChar` at the 0-based CHARACTER index within the data
/// part (not a byte index, not `InvalidStringLength`); at or before the
/// separator it is `InvalidHrp` carrying the HRP.
#[test]
fn bughunt_mk_05_non_ascii_error_variant_and_position() {
    use mk_codec::Error;
    let strings = encode_with_chunk_set_id(&multi_chunk_card(), 0).unwrap();
    for s in [&strings[0], strings.last().unwrap()] {
        let chars: Vec<char> = s.chars().collect();
        let data_len = chars.len() - 3;
        for p in ['\u{212A}', '\u{00E9}'] {
            for i in [0usize, 7, data_len - 1] {
                let mut c = chars.clone();
                c[3 + i] = p;
                let bad: String = c.into_iter().collect();
                let e = decode_string(&bad).err();
                assert!(
                    matches!(e, Some(Error::InvalidChar { ch, position }) if ch == p && position == i),
                    "{} at data index {i}: got {e:?}",
                    p.escape_unicode()
                );
                assert_eq!(
                    e.unwrap().to_string(),
                    format!("invalid character {p} at position {i}")
                );
            }
        }
        let mut c = chars.clone();
        c[1] = '\u{212A}';
        let bad: String = c.into_iter().collect();
        let e = decode_string(&bad).err();
        assert!(
            matches!(&e, Some(Error::InvalidHrp(h)) if h == "m\u{212A}"),
            "Kelvin sign in the HRP: got {e:?}"
        );
        assert_eq!(e.unwrap().to_string(), "invalid HRP: m\u{212A}");
    }
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545F4914F6CDD1D)
    }
}

/// Lead L6: 5..=8 symbol errors must never yield an accepted correction
/// (a miscorrection to another codeword is ~2^-24 per pattern at best, so
/// zero accepted is expected over 2000 seeded patterns per code).
#[test]
fn bughunt_mk_06_five_to_eight_errors_never_accepted() {
    let card = multi_chunk_card();
    let strings = encode_with_chunk_set_id(&card, 0).unwrap();
    let mut rng = Rng(0x9E3779B97F4A7C15);
    let (mut rejected, mut accepted) = (0usize, 0usize);
    for (which, s) in [(true, &strings[0]), (false, strings.last().unwrap())] {
        let clean = decode_string(s).unwrap();
        assert_eq!(clean.corrections_applied, 0);
        let dwc = clean.data_with_checksum.clone();
        for _ in 0..2000 {
            let n = 5 + (rng.next() % 4) as usize;
            let mut pos: Vec<usize> = Vec::new();
            while pos.len() < n {
                let p = (rng.next() % dwc.len() as u64) as usize;
                if !pos.contains(&p) {
                    pos.push(p);
                }
            }
            let mut w = dwc.clone();
            for &p in &pos {
                w[p] ^= 1 + (rng.next() % 31) as u8;
            }
            let r = if which {
                bch_correct_long("mk", &w)
            } else {
                bch_correct_regular("mk", &w)
            };
            match r {
                Err(_) => rejected += 1,
                Ok(c) => {
                    accepted += 1;
                    let ok = if which {
                        bch_verify_long("mk", &c.data)
                    } else {
                        bch_verify_regular("mk", &c.data)
                    };
                    eprintln!(
                        "ACCEPTED n={n} pos={pos:?} valid_codeword={ok} applied={}",
                        c.corrections_applied
                    );
                }
            }
        }
    }
    eprintln!("L6 rejected={rejected} accepted={accepted}");
    assert_eq!(accepted, 0, "{accepted} 5..8-error words were accepted");
    // And through the public card decode: a corrupted chunk is an error.
    let mut s0: Vec<char> = strings[0].chars().collect();
    for p in [11usize, 13, 17, 19, 23] {
        s0[p] = if s0[p] == 'q' { 'p' } else { 'q' };
    }
    let mut v = strings.clone();
    v[0] = s0.into_iter().collect();
    let parts: Vec<&str> = v.iter().map(String::as_str).collect();
    if let Ok(c) = decode(&parts) {
        assert_eq!(c, card, "5-error chunk decoded to a DIFFERENT card");
    }
}
