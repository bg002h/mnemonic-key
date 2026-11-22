//! Architect rows (bh #17 Q1): depth-1 origin fingerprint vs xpub.parent_fingerprint.
//! K12 refused at mint; K13/K14 controls; V3 decode still reads a legacy card, validate reports it.
use std::str::FromStr;

use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::Secp256k1;
use mk_codec::{KeyCard, decode, encode};

const ROOT: &str = "tpubD6NzVbkrYhZ4WMCcYXEgDrM1AFATSGmrFyMnA2Dgx1Y551VueaE3iemHhQW6MaZfCm67ek9JkvZboSbAjuvGk7foa2PinzkrKyRnxyLHWQ2";
const STUB: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];
const LEGACY_D1: [&str; 2] = [
    "mk1qpkg6wpqqsqaatd7al02m0h0lcqsqpp4sl8h28nkaq2rw39uktvh6uenflm3rxqw9jllmjktc059yn75stew08xve94t2q3eze2hn68kjszz",
    "mk1qpkg6wppqgx00lerqaq42pmr6f79l0erqcejj2g8umwpudmkpn4h093lwzzehv35azeqll55lad4xzcy4",
];
const D1_DISPLAY: &str = "origin fingerprint deadbeef is not the depth-1 key's parent fingerprint 751e76e8; at depth 1 the parent is the master";

fn child() -> Xpub {
    let secp = Secp256k1::verification_only();
    Xpub::from_str(ROOT)
        .unwrap()
        .ckd_pub(&secp, ChildNumber::Normal { index: 0 })
        .unwrap()
}
fn card(fp: Option<[u8; 4]>) -> KeyCard {
    KeyCard::new(
        vec![STUB],
        fp.map(Fingerprint::from),
        DerivationPath::from_str("m/0").unwrap(),
        child(),
    )
}
fn round_trips(c: &KeyCard) {
    let s = encode(c).unwrap();
    let refs: Vec<&str> = s.iter().map(String::as_str).collect();
    assert_eq!(&decode(&refs).unwrap(), c);
}

#[test]
fn kv_depth1_false_origin_fp_refused() {
    let e = encode(&card(Some(STUB))).expect_err("encode accepted the card");
    assert_eq!(e.to_string(), D1_DISPLAY);
}

#[test]
fn kv_depth1_true_origin_fp_encodes() {
    round_trips(&card(Some([0x75, 0x1e, 0x76, 0xe8])));
}

#[test]
fn kv_depth1_no_origin_fp_encodes() {
    round_trips(&card(None));
}

#[test]
fn kv_validate_reports_legacy_depth1_card() {
    let c = decode(&LEGACY_D1).expect("a legacy depth-1 card still decodes");
    assert_eq!(c, card(Some(STUB)));
    assert_eq!(c.validate().unwrap_err().to_string(), D1_DISPLAY);
}
