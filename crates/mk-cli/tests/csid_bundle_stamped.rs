//! FOLLOWUPS `mk-chunk-set-id-bundle-stamped`: the R2 chunk-set-id warning
//! stays silent on a card whose stamped id is the toolkit's bundle id (BIP
//! "Note (toolkit slot-XOR)": the leading 20 bits of the first
//! policy_id_stub, XOR a cosigner slot 0..=31).
//!
//! Every card below is real `mnemonic bundle` 0.106.0 output on signet
//! (abandon..about and legal-winner..yellow test seeds), except
//! `SLOT32_PIN`, minted by `mk encode --chunk-set-id 4449e` (base ^ 0x20).
//! The 32-key cards come from a `--descriptor-file` bundle of
//! `wsh(or_d(multi(1,<16 keys>),multi(1,<16 keys>)))`; md1 caps a policy at
//! 32 keys, so slot 31 is the highest slot a bundle can mint.

use assert_cmd::Command;
use serde_json::Value;

const WARN: &str = "was not derived from its content";

/// bip84 single-sig, stub 444be2c9, stamped 444be (slot 0), derived 7fa49.
const SINGLE_SIG: [&str; 2] = [
    "mk1qpg397pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycf",
    "mk1qpg397pp3c8mu0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02ysk3l6fyxg6m66tmvhzh70a",
];

/// 2-of-2 wsh-sortedmulti, cosigner 1: stub da7bb37a, stamped da7ba (slot 1).
const MULTISIG_SLOT1: [&str; 2] = [
    "mk1qpmfa6pqqspd57an0td8hvm6hp5gmughqs6c0ncrum7pwruvf52der5dqacn6cr6q75j7myc33vauqn492273ntu57puxundl6n2wydhmvkv",
    "mk1qpmfa6pp2ekqyzxyy66kcdj5qqnzvlehu09qzyn2e9k6rck8qjpk5a48k7xujdvssc06flc7pyw66x3chdks",
];

/// 32-key bundle, cosigner 16: stub 18ed4d04, stamped 18ec4 (slot 16).
const SLOT16: [&str; 5] = [
    "mk1qprrkyyqqssp3m2dqsvw6ngyrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk56jqg26rvap7w0p3m",
    "mk1qprrkyypqsvw6ngyrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk56pqca4xsg42hu7ezxp96qzr8",
    "mk1qprrkyyzrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk56prnchdq4lsykzqgpqqgsxqgpqqgjzqgpqqgs2qgpqqgqs6cwu8adyjqmlhj0ten",
    "mk1qprrkyyrellwthfg38k494jll3tncxm0d5n4lyh6hqvv0hu4xek46fxtw4jhdnu0t9ls86kqnw572zdzesr48swqtua6cmg8skf3g78f5rd3",
    "mk1qprrkyyyazcsfhxfvyvdw58ztetvgdlum80xjmfag5eqpm7m62lwehr",
];

/// 32-key bundle, cosigner 31: stub 18ed4d04, stamped 18ecb (slot 31).
const SLOT31: [&str; 5] = [
    "mk1qprrktyqqssp3m2dqsvw6ngyrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk564rkg2z87t6tgxch",
    "mk1qprrktypqsvw6ngyrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk56pqca4xsgjff7wcf9tpl892t",
    "mk1qprrktyzrrk56pqca4xsgx8df5zp3m2dqsvw6ngyrrk56prnchdq4lsykzqgpqqgsxqgpqqgn7qgpqqgs2qgpqqgqs6cwwt29uglqp7uwslr",
    "mk1qprrktyrelnwxru8d9nwydqvpd9ap97z3e9vxtauyyppgvekgv5tuvntheq8zhp0hm8syexzca3tntenu423a3nw2vrrsyrj9ga96jtlw6wq",
    "mk1qprrktyy6cwadgrwmulghhs3rg46khlksrx9g33rsyxqdql4936v6ct",
];

/// The SINGLE_SIG content pinned by `mk encode --chunk-set-id 4449e`:
/// 444be ^ 0x20, one past the highest slot. Not a bundle id: still warns.
const SLOT32_PIN: [&str; 2] = [
    "mk1qpg3y7pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms4yl6gyved8e7cyt",
    "mk1qpg3y7pp3c8mu0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02ysk3l6fyxgdfy5pzj2569lu",
];

fn run(verb: &[&str], strings: &[&str]) -> std::process::Output {
    Command::cargo_bin("mk")
        .unwrap()
        .args(verb)
        .args(strings)
        .output()
        .expect("invoke mk")
}

fn err(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn out(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn bundle_card_is_silent_on_every_read_verb() {
    for verb in [
        &["decode"][..],
        &["inspect"][..],
        &["verify"][..],
        &["derive", "--index", "0"][..],
        &["address"][..],
    ] {
        let o = run(verb, &SINGLE_SIG);
        assert_eq!(o.status.code(), Some(0), "mk {verb:?}: {}", err(&o));
        assert!(
            !err(&o).contains(WARN),
            "mk {verb:?} stderr warned: {}",
            err(&o)
        );
        assert!(
            !out(&o).contains(WARN),
            "mk {verb:?} stdout warned: {}",
            out(&o)
        );
    }
}

#[test]
fn bundle_card_inspect_still_prints_the_stamped_id() {
    let o = run(&["inspect"], &SINGLE_SIG);
    assert!(
        out(&o).contains("chunk_set_id:        444be (stamped)"),
        "{}",
        out(&o)
    );
}

#[test]
fn bundle_cards_at_slots_1_16_31_are_silent() {
    for (name, card) in [
        ("slot 1", &MULTISIG_SLOT1[..]),
        ("slot 16", &SLOT16[..]),
        ("slot 31", &SLOT31[..]),
    ] {
        let o = run(&["decode"], card);
        assert_eq!(o.status.code(), Some(0), "{name}: {}", err(&o));
        assert!(!err(&o).contains(WARN), "{name} warned: {}", err(&o));
    }
}

#[test]
fn a_pin_one_past_the_last_slot_still_warns() {
    let o = run(&["decode"], &SLOT32_PIN);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        err(&o).contains(
            "warning: this key card's stamped chunk-set id (4449e) was not derived from its \
             content, which computes 7fa49."
        ),
        "{}",
        err(&o)
    );
}

#[test]
fn verify_json_names_a_bundle_stamped_id() {
    let o = run(&["verify", "--json"], &SINGLE_SIG);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let v: Value = serde_json::from_str(out(&o).trim()).expect("json");
    assert_eq!(
        v["chunk_set_id"],
        serde_json::json!({
            "declared": "444be",
            "derived": "7fa49",
            "matches": false,
            "bundle_stamped": true
        })
    );
    let o = run(&["verify", "--json"], &SLOT32_PIN);
    let v: Value = serde_json::from_str(out(&o).trim()).expect("json");
    assert_eq!(v["chunk_set_id"]["bundle_stamped"], Value::Bool(false));
}

#[test]
fn repair_of_a_bundle_card_does_not_warn() {
    // SINGLE_SIG chunk 1 with one substitution at position 12 ('u' -> 'a').
    let damaged = [
        SINGLE_SIG[0],
        "mk1qpg397pp3c8ma0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02ysk3l6fyxg6m66tmvhzh70a",
    ];
    let o = run(&["repair"], &damaged);
    assert_eq!(o.status.code(), Some(5), "{}", err(&o));
    assert!(out(&o).contains(SINGLE_SIG[1]), "{}", out(&o));
    assert!(!err(&o).contains(WARN), "repair warned: {}", err(&o));
}
