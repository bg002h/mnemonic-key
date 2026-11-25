//! FIX_mk_ff_stale_docs (bughunt-1119 F-F: bh-mk-I5, bh-mk-I14): the printed
//! recovery recipe and the Rust API guide's consumer example stay runnable.
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use bitcoin::hashes::{Hash, sha256};
use std::path::PathBuf;
use std::str::FromStr;

const X0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const X1: &str = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk";
/// The keyed policy card of tests/from_md1_set.rs (`KEYED_POLICY_A`): @0 = X0, @1 = X1.
const KEYED: [&str; 4] = [
    "md1fj5r4pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapguu6w2y2jygprc",
    "md1fj5r4psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wnj3gvx34m0wnt",
    "md1fj5r4pshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuyz646z0r98kyg",
    "md1fj5r4pslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqrfrshqj9459eg",
];

fn read(rel: &str) -> String {
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .into();
    std::fs::read_to_string(root.join(rel)).unwrap()
}
/// Lines from the first line starting with `start` up to (not including) the first later line starting with `end`.
fn between(text: &str, after: &str, start: &str, end: &str) -> String {
    let from = text.find(after).expect("anchor heading");
    let mut out = String::new();
    let mut on = false;
    for l in text[from..].lines() {
        if on && l.starts_with(end) {
            break;
        }
        if l.starts_with(start) {
            on = true;
        }
        if on {
            out.push_str(l);
            out.push('\n');
        }
    }
    assert!(!out.is_empty(), "block {start:?} not found");
    out
}
fn wire65(x: &str) -> [u8; 65] {
    let x = Xpub::from_str(x).unwrap();
    let mut b = [0u8; 65];
    b[..32].copy_from_slice(x.chain_code.as_bytes());
    b[32..].copy_from_slice(&x.public_key.serialize());
    b
}
fn id(d: &md_codec::Descriptor) -> String {
    hex::encode(md_codec::compute_wallet_policy_id(d).unwrap().as_bytes())
}

#[test]
fn ff_i5_docs_name_no_missing_helper() {
    for f in ["design/SPEC_mk_v0_1.md", "bip/bip-mnemonic-key.mediawiki"] {
        assert!(
            !read(f).contains("compute_wallet_instance_id"),
            "{f} names md-codec's removed compute_wallet_instance_id"
        );
    }
}

#[test]
fn ff_i5_docs_print_no_xpub_concat_recipe() {
    for f in ["design/SPEC_mk_v0_1.md", "bip/bip-mnemonic-key.mediawiki"] {
        assert!(
            !read(f).contains("canonical_xpub_serialization"),
            "{f} prints the retired 78-byte xpub recipe"
        );
    }
}

#[test]
fn ff_i5_recovery_step4_names_wallet_policy_id() {
    let spec = between(
        &read("design/SPEC_mk_v0_1.md"),
        "## §5. Linkage to MD",
        "4. ",
        "5. ",
    );
    let bip = between(
        &read("bip/bip-mnemonic-key.mediawiki"),
        "Recovery flow (the canonical sequence",
        "# Reconstruct the full descriptor",
        "# Reject if mismatch",
    );
    for (n, t) in [("SPEC §5 step 4", spec), ("BIP recovery step 4", bip)] {
        assert!(
            t.contains("compute_wallet_policy_id"),
            "{n} does not name compute_wallet_policy_id:\n{t}"
        );
    }
}

/// The recipe the docs print: the WalletPolicyId of the template with the key cards' xpubs plugged
/// in reproduces the keyed card's own identity; the retired recipe, a missing fingerprint set and
/// swapped slots do not.
#[test]
fn ff_i5_assembled_policy_id_reproduces_keyed_card() {
    let keyed = md_codec::reassemble(&KEYED).unwrap();
    assert_eq!(id(&keyed), "38bd7cec8059cdc4553c2836d7e8e303");
    let mut template = keyed.clone();
    template.tlv.pubkeys = None;
    let mut assembled = template.clone();
    assembled.tlv.pubkeys = Some(vec![(0, wire65(X0)), (1, wire65(X1))]);
    assert_eq!(id(&assembled), "38bd7cec8059cdc4553c2836d7e8e303");
    let mut no_fp = assembled.clone();
    no_fp.tlv.fingerprints = None;
    assert_eq!(id(&no_fp), "949c621144d15257e4cc94921c8dbbc3");
    let mut swapped = template.clone();
    swapped.tlv.pubkeys = Some(vec![(0, wire65(X1)), (1, wire65(X0))]);
    assert_eq!(id(&swapped), "780f6305cae4a2412ab398ff78dc426a");
    let mut pre = md_codec::encode_payload(&keyed).unwrap().0;
    for x in [X0, X1] {
        pre.extend_from_slice(&Xpub::from_str(x).unwrap().encode());
    }
    assert_eq!(
        hex::encode(&sha256::Hash::hash(&pre).to_byte_array()[..16]),
        "c86829b5ab2090a5b712b7ba1e64386c"
    );
}

#[test]
fn ff_i14_guide_pins_no_stale_tag() {
    assert!(
        !read("docs/MK_CODEC_RUST_API.md").contains("v0.2.2"),
        "the guide still pins or names mk-codec v0.2.2"
    );
}

#[test]
fn ff_i14_guide_manifest_names_bitcoin() {
    let guide = read("docs/MK_CODEC_RUST_API.md");
    let manifest = between(&guide, "## Cargo dependency", "[dependencies]", "```");
    let req = read("crates/mk-codec/Cargo.toml")
        .lines()
        .find(|l| l.starts_with("bitcoin = "))
        .expect("mk-codec's bitcoin requirement")
        .to_string();
    assert!(
        manifest.lines().any(|l| l == req),
        "guide manifest lacks `{req}`:\n{manifest}"
    );
}

#[test]
fn ff_i14_guide_example_xpub_encodes() {
    let guide = read("docs/MK_CODEC_RUST_API.md");
    let line = guide
        .lines()
        .find(|l| l.starts_with("let xpub: Xpub = \""))
        .expect("example xpub line");
    let lit = line.split('"').nth(1).unwrap();
    let xpub = Xpub::from_str(lit)
        .unwrap_or_else(|e| panic!("guide example xpub {lit:?} does not parse: {e}"));
    let card = mk_codec::KeyCard::new(
        vec![[0u8; 4]],
        Some(Fingerprint::from([0x73, 0xc5, 0xda, 0x0a])),
        "m/84'/0'/0'".parse::<DerivationPath>().unwrap(),
        xpub,
    );
    let strings = mk_codec::encode(&card).unwrap();
    let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
    assert_eq!(mk_codec::decode(&refs).unwrap().xpub, xpub);
}

/// I-2 (architect-mk-ff-i2): step 4 names where an `@N`'s fingerprint comes from and what to do
/// when no card supplies the one the expected identity was computed with; the fact behind the
/// fallback is that the fingerprints change the identity but not the addresses.
#[test]
fn ff_i5_step4_names_fingerprint_source() {
    let spec = between(
        &read("design/SPEC_mk_v0_1.md"),
        "## §5. Linkage to MD",
        "4. ",
        "5. ",
    );
    let bip = between(
        &read("bip/bip-mnemonic-key.mediawiki"),
        "Recovery flow (the canonical sequence",
        "# Reconstruct the full descriptor",
        "# Reject if mismatch",
    );
    for (n, t) in [("SPEC §5 step 4", spec), ("BIP recovery step 4", bip)] {
        for want in [
            "origin_fingerprint",
            "they differ, reject the assembly",
            "first address",
        ] {
            assert!(t.contains(want), "{n} does not state {want:?}:\n{t}");
        }
    }
    let mut assembled = md_codec::reassemble(&KEYED).unwrap();
    assembled.tlv.pubkeys = Some(vec![(0, wire65(X0)), (1, wire65(X1))]);
    let mut no_fp = assembled.clone();
    no_fp.tlv.fingerprints = None;
    assert_ne!(id(&assembled), id(&no_fp));
    for chain in [0, 1] {
        assert_eq!(
            assembled
                .derive_address(chain, 0, bitcoin::Network::Bitcoin)
                .unwrap(),
            no_fp
                .derive_address(chain, 0, bitcoin::Network::Bitcoin)
                .unwrap()
        );
    }
}
