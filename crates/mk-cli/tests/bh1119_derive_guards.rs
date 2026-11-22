//! bughunt-1119 F-D rows (bh-mk-C1, bh-mk-I18): `mk address` / `mk derive` guards.
//! Row names are those of design/FIX_mk_fd_derive_guards.md section 5.

use std::process::Output;
use std::str::FromStr;

use assert_cmd::Command;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::KeyCard;

const LEAF: &str = "tpubD6NzVbkrYhZ4WLczPJWReQycCJdd6YVWXubbVUFnJ5KgU5MDQrD998ZJLT5hqNKRA9cJzkh4D7oXwCpwFUWWWkHR9zPeBk1hnWBJKNhyYsn";
const LEAFM: &str = "xpub661MyMwAqRbcEYS8w7XLSVeEsBXy79zSzH1J8vCdxAZningWLdN3zgtU6QzvJsNBNF5QPBBBg1yVF2LKrcfGdJq86PeLWDMUCYatZPzQu8R";
const X84: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
const TV1M: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";

const C1HEAD: &str = "this card holds one fixed key (depth 0, all-zero chain code; the toolkit writes a WIF key this way) and has no child keys any wallet watches, so mk derives nothing from it. If this key is one of the keys in a multisig wallet, that wallet's addresses come from its md1 policy card (md address), not from this key alone. The key's own address: mnemonic convert --from xpub=";

fn c1_main(ty: &str) -> String {
    format!("{C1HEAD}{LEAFM} --to address --path m --script-type {ty}")
}
fn c1_test(ty: &str, net: &str) -> String {
    format!("{C1HEAD}{LEAF} --to address --path m --script-type {ty} --network {net}")
}

fn card(xpub: &str, origin_path: &str, fp: Option<[u8; 4]>) -> Vec<String> {
    let kc = KeyCard::new(
        vec![[0xde, 0xad, 0xbe, 0xef]],
        fp.map(Fingerprint::from),
        DerivationPath::from_str(origin_path).unwrap(),
        Xpub::from_str(xpub).unwrap(),
    );
    mk_codec::encode(&kc).unwrap()
}

fn mk(sub: &str, cardstrs: &[String], extra: &[&str]) -> Output {
    Command::cargo_bin("mk")
        .unwrap()
        .arg(sub)
        .args(cardstrs)
        .args(extra)
        .output()
        .unwrap()
}

fn s(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

/// exit 64, stdout empty, stderr exactly `error: ` + text.
fn assert_refused(o: &Output, text: &str) {
    assert_eq!(o.status.code(), Some(64), "stderr: {}", s(&o.stderr));
    assert_eq!(s(&o.stdout), "", "stdout must be empty");
    assert_eq!(s(&o.stderr), format!("error: {text}\n"));
}

fn leaf() -> Vec<String> {
    card(LEAF, "m", None)
}
fn leafm() -> Vec<String> {
    card(LEAFM, "m", None)
}
fn acct() -> Vec<String> {
    card(X84, "m/84'/0'/0'", Some([0x73, 0xc5, 0xda, 0x0a]))
}
fn master() -> Vec<String> {
    card(TV1M, "m", Some([0x34, 0x42, 0x19, 0x3e]))
}

#[test]
fn fd_c1_address_refuses_fixed_key_card() {
    let o = mk(
        "address",
        &leaf(),
        &[
            "--address-type",
            "p2wpkh",
            "--network",
            "signet",
            "--count",
            "1",
        ],
    );
    assert_refused(&o, &c1_test("p2wpkh", "signet"));
}

#[test]
fn fd_c1_address_refuses_fixed_key_card_json() {
    let o = mk(
        "address",
        &leaf(),
        &[
            "--address-type",
            "p2wpkh",
            "--network",
            "signet",
            "--count",
            "1",
            "--json",
        ],
    );
    assert_eq!(o.status.code(), Some(64));
    assert_eq!(s(&o.stderr), "");
    let want = format!(
        "{{\"error\":{{\"details\":null,\"exit_code\":64,\"kind\":\"UsageError\",\"message\":\"{}\"}},\"schema_version\":1}}\n",
        c1_test("p2wpkh", "signet")
    );
    assert_eq!(s(&o.stdout), want);
}

#[test]
fn fd_c1_address_refuses_fixed_key_card_mainnet() {
    let o = mk(
        "address",
        &leafm(),
        &["--address-type", "p2wpkh", "--count", "1"],
    );
    assert_refused(&o, &c1_main("p2wpkh"));
}

#[test]
fn fd_c1_derive_refuses_fixed_key_card() {
    let o = mk("derive", &leaf(), &["--index", "0"]);
    assert_refused(&o, &c1_test("TYPE", "NETWORK"));
    let o = mk("derive", &leaf(), &["--path", "m/0/0"]);
    assert_refused(&o, &c1_test("TYPE", "NETWORK"));
}

#[test]
fn fd_c1_derive_refuses_fixed_key_card_mainnet() {
    let o = mk("derive", &leafm(), &["--index", "0"]);
    assert_refused(&o, &c1_main("TYPE"));
}

#[test]
fn fd_c1_refusal_precedes_other_errors() {
    let t = c1_test("TYPE", "NETWORK");
    // (a) no --address-type
    assert_refused(&mk("address", &leaf(), &[]), &t);
    // (b) network kind mismatch (mainnet is not a test-kind network)
    assert_refused(
        &mk(
            "address",
            &leaf(),
            &["--address-type", "p2wpkh", "--network", "mainnet"],
        ),
        &c1_test("p2wpkh", "NETWORK"),
    );
    // (c) hardened, (d) parser-rejected, (e) empty path
    for p in ["m/0'", "/", ""] {
        assert_refused(&mk("derive", &leaf(), &["--path", p]), &t);
    }
}

#[test]
fn fd_i18_derive_refuses_empty_path() {
    for p in ["", "m", "m/"] {
        let o = mk("derive", &acct(), &["--path", p, "--json"]);
        assert_eq!(o.status.code(), Some(64), "path {p:?}");
        let out = s(&o.stdout);
        assert!(out.contains("\"kind\":\"UsageError\""), "{out}");
        assert!(!out.contains("child_xpub"), "{out}");
    }
    let o = mk("derive", &acct(), &["--path", ""]);
    assert_eq!(o.status.code(), Some(64));
    assert_eq!(s(&o.stdout), "");
    assert_eq!(
        s(&o.stderr),
        "error: --path \"\" has no steps: derive needs at least one step (e.g. m/0/5); the card's own key is shown by `mk decode`\n"
    );
}

#[test]
fn fd_c1_master_card_still_derives() {
    let o = mk(
        "address",
        &master(),
        &["--address-type", "p2wpkh", "--count", "1"],
    );
    assert_eq!(o.status.code(), Some(0));
    assert!(s(&o.stdout).contains("bc1qp5wfcq48h6d63wyy9qz0awtpfqwwv4sma86mhz"));
    assert!(s(&o.stderr).contains("not the canonical account depth 3"));
    let o = mk("derive", &master(), &["--index", "0", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(s(&o.stdout).contains("xpub6AvUGrnEpfvJ8L7GLRkBTByQ9uBvUHp9o5VxHrFxhvzV4dSWkySpNaBoLR9FpbnwRmTa69yLHF3QfcaxbWT7gWdwws5k4dpmJvqpEuMWwnj"));
}

#[test]
fn fd_i18_nonempty_path_still_derives() {
    let o = mk("derive", &acct(), &["--path", "m/0/0", "--json"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(s(&o.stdout).contains("xpub6Fec2dLaQAfWYmg8iGSbMv9ay8WfvB8Rc8H9Koys6THpADfT7ewgxtBUc7hvvA8WarLUqwZqwniTmpFCpqmdoS2y3KGSLoNk2TpEQv8oYaf"));
}
