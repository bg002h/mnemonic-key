//! FIX_mkc_keycard_validation rows K1-K11 (bughunt-1119 mk I8, I9, I10): mint-side
//! KeyCard validity. Keys are testnet; ROOT is a depth-0 tpub whose public key is
//! the secp256k1 generator (key fingerprint 751e76e8).
use std::str::FromStr;

use bitcoin::bip32::{ChainCode, ChildNumber, DerivationPath, Fingerprint, Xpub};
use mk_codec::{KeyCard, decode, encode};

const ROOT: &str = "tpubD6NzVbkrYhZ4WMCcYXEgDrM1AFATSGmrFyMnA2Dgx1Y551VueaE3iemHhQW6MaZfCm67ek9JkvZboSbAjuvGk7foa2PinzkrKyRnxyLHWQ2";
const STUB: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];

fn root() -> Xpub {
    Xpub::from_str(ROOT).unwrap()
}
fn master_card(xpub: Xpub, fp: Option<[u8; 4]>) -> KeyCard {
    KeyCard::new(
        vec![STUB],
        fp.map(Fingerprint::from),
        DerivationPath::master(),
        xpub,
    )
}
fn refused(card: &KeyCard, want: &str) {
    let e = encode(card).expect_err("encode accepted the card");
    assert_eq!(e.to_string(), want);
}
fn round_trips(card: &KeyCard) {
    let s = encode(card).unwrap();
    let refs: Vec<&str> = s.iter().map(String::as_str).collect();
    assert_eq!(&decode(&refs).unwrap(), card);
}

#[test]
fn kv_i8_depth0_nonzero_parent_fp_refused() {
    let mut x = root();
    x.parent_fingerprint = Fingerprint::from([1, 1, 1, 1]);
    refused(
        &master_card(x, None),
        "depth-0 key has parent fingerprint 01010101; BIP 32 requires 00000000 for a master key",
    );
}

#[test]
fn kv_i8_zero_chain_code_does_not_exempt_parent_fp() {
    let mut x = root();
    x.parent_fingerprint = Fingerprint::from([1, 1, 1, 1]);
    x.chain_code = ChainCode::from([0u8; 32]);
    refused(
        &master_card(x, None),
        "depth-0 key has parent fingerprint 01010101; BIP 32 requires 00000000 for a master key",
    );
}

#[test]
fn kv_i9_depth0_false_origin_fp_refused() {
    refused(
        &master_card(root(), Some(STUB)),
        "origin fingerprint deadbeef is not the depth-0 key's own fingerprint 751e76e8",
    );
}

#[test]
fn kv_i9_sentinel_foreign_fp_refused() {
    let mut x = root();
    x.chain_code = ChainCode::from([0u8; 32]);
    let own = x.fingerprint();
    refused(
        &master_card(x, Some(STUB)),
        &format!("origin fingerprint deadbeef is not the depth-0 key's own fingerprint {own}"),
    );
}

#[test]
fn kv_sentinel_zero_fp_encodes() {
    let mut x = root();
    x.chain_code = ChainCode::from([0u8; 32]);
    round_trips(&master_card(x, Some([0, 0, 0, 0])));
}

#[test]
fn kv_sentinel_own_fp_encodes() {
    let mut x = root();
    x.chain_code = ChainCode::from([0u8; 32]);
    let own = x.fingerprint().to_bytes();
    round_trips(&master_card(x, Some(own)));
}

#[test]
fn kv_master_own_fp_encodes() {
    round_trips(&master_card(root(), Some([0x75, 0x1e, 0x76, 0xe8])));
    round_trips(&master_card(root(), None));
}

fn depth1(n: ChildNumber) -> KeyCard {
    let mut x = root();
    x.depth = 1;
    x.child_number = n;
    x.parent_fingerprint = Fingerprint::from([0x75, 0x1e, 0x76, 0xe8]);
    KeyCard::new(vec![STUB], None, DerivationPath::from(vec![n]), x)
}

#[test]
fn kv_i10_normal_index_ge_2_31_refused() {
    refused(
        &depth1(ChildNumber::Normal { index: 0x8000_0001 }),
        "invalid path component: component 1 index 2147483649 is out of the BIP 32 range 0..2^31",
    );
}

#[test]
fn kv_i10_hardened_index_ge_2_31_refused() {
    refused(
        &depth1(ChildNumber::Hardened { index: 0x8000_0001 }),
        "invalid path component: component 1 index 2147483649 is out of the BIP 32 range 0..2^31",
    );
}

#[test]
fn kv_i10_canonical_payload_bytes_refused() {
    let e = depth1(ChildNumber::Normal { index: 0x8000_0001 })
        .canonical_payload_bytes()
        .expect_err("canonical_payload_bytes accepted the card");
    assert_eq!(
        e.to_string(),
        "invalid path component: component 1 index 2147483649 is out of the BIP 32 range 0..2^31"
    );
}

#[test]
fn kv_i10_index_2_31_minus_1_round_trips() {
    round_trips(&depth1(ChildNumber::Normal { index: 0x7FFF_FFFF }));
    round_trips(&depth1(ChildNumber::Hardened { index: 0x7FFF_FFFF }));
}

#[test]
fn kv_i10_index_exactly_2_31_refused() {
    for n in [
        ChildNumber::Normal { index: 0x8000_0000 },
        ChildNumber::Hardened { index: 0x8000_0000 },
    ] {
        refused(
            &depth1(n),
            "invalid path component: component 1 index 2147483648 is out of the BIP 32 range 0..2^31",
        );
    }
}
