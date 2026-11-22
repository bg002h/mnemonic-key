//! FIX_mkc_keycard_validation rows V1-V2: `KeyCard::validate` on decoded cards
//! (T46: decode still reads a legacy invalid-master card; validate reports it).
use mk_codec::decode;

const LEGACY_I8: [&str; 2] = [
    "mk1qpeffzpqqqqaatd7allqqpp4sl8szqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqnehen8u5yagl2mw0p43h3j",
    "mk1qpeffzppl8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqte3jjj9mtsugd0x98xyvntv",
];
const LEGACY_I9: [&str; 2] = [
    "mk1qp32rnpqqsqaatd7al02m0h0lcqqgdv8euqqqqqqqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqsyr6hhd3h4f5yu7sp",
    "mk1qp32rnpp0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vg4peejgreyhlqwm83g90",
];

#[test]
fn kv_validate_reports_legacy_master_cards() {
    let c8 = decode(&LEGACY_I8).expect("legacy I8 card still decodes");
    assert_eq!(
        c8.validate()
            .expect_err("validate accepted I8 card")
            .to_string(),
        "depth-0 key has parent fingerprint 01010101; BIP 32 requires 00000000 for a master key"
    );
    let c9 = decode(&LEGACY_I9).expect("legacy I9 card still decodes");
    assert_eq!(
        c9.validate()
            .expect_err("validate accepted I9 card")
            .to_string(),
        "origin fingerprint deadbeef is not the depth-0 key's own fingerprint 751e76e8"
    );
}

#[test]
fn kv_validate_ok_on_every_corpus_vector() {
    let doc: serde_json::Value = serde_json::from_str(mk_codec::test_vectors::V0_1_JSON).unwrap();
    let mut n = 0;
    for v in doc["vectors"].as_array().unwrap() {
        let Some(strings) = v["expected"]["strings"].as_array() else {
            continue;
        };
        let refs: Vec<&str> = strings.iter().map(|s| s.as_str().unwrap()).collect();
        decode(&refs).unwrap().validate().unwrap();
        n += 1;
    }
    assert!(n > 0);
}
