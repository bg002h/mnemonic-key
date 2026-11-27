//! bughunt-1119 F-E (design/FIX_mk_fe_keyed_policy.md): `mk encode` keyed-policy
//! membership, repeated key records and repeated stubs (bh-mk-I2, bh-mk-I15,
//! sweep-1122 row 15, mk-dup-stub). Each test fn is named exactly as its row in the
//! FIX's section 5; the rows' shell twin is `scripts/mk-fe-keyed-policy-rows.sh`.
//! Public keys only.

use std::path::Path;
use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

const X0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const X1: &str = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk";
const X2: &str = "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR";
const TV1M: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
const LEAFM: &str = "xpub661MyMwAqRbcEYS8w7XLSVeEsBXy79zSzH1J8vCdxAZningWLdN3zgtU6QzvJsNBNF5QPBBBg1yVF2LKrcfGdJq86PeLWDMUCYatZPzQu8R";
/// X0's chain code and key re-serialized at depth 0 (parent fp 00000000, child 0).
const X0D0: &str = "xpub661MyMwAqRbcGQnC8zMGwRc4EXYJCgXrxx9kXtw1RXqu4TcW26PxHqssgp6sU4N5CR5o9QcUZPG31fzPeUHEVPkFit1WpVZTmqZLzpvZG2s";
/// Keyless `wpkh` template, stub 559e64b2.
const TPL: &str = "md1yqpqqxzq2qwfv8urt848e";
const P0: &str = "48'/0'/0'/2'";
const M0: &str = "m/48'/0'/0'/2'";
const M1: &str = "m/48'/0'/1'/2'";
const M9: &str = "m/48'/0'/9'/2'";
const MSIG: &str = "m/48'/1'/0'/2'";

const A: [&str; 4] = [
    "md1fj5r4pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapguu6w2y2jygprc",
    "md1fj5r4psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wnj3gvx34m0wnt",
    "md1fj5r4pshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuyz646z0r98kyg",
    "md1fj5r4pslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqrfrshqj9459eg",
];
const SPARSE: [&str; 3] = [
    "md1fmpkgpqpq2tvyyy4qqxppsg2z6zw0za5zs4gy4m5rru59s2su80aq5sat59s7xnghe",
    "md1fmpkgpq09q2hyxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhs8mg2jx2us05u6",
    "md1fmpkgpqkumapumfj075dhzfzvynh66n94j5lcxlmx9ayav9mj0jjqgknzrt66ggpxt",
];
const REP: [&str; 4] = [
    "md1fyw09pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapgv29jfttksg5j3",
    "md1fyw09psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wryw50fs300mzz",
    "md1fyw09pshsdlkvt6f6cthyl98waqcl9pvz58pmltjs9tjrg0g2z0agd4urfpzanhaq3lcd7yx9mq49jevw95",
    "md1fyw09psut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2vkk2nlqmlvch5n4shwf72gqmn8m0alwh8lcz",
];
const NOFP: [&str; 4] = [
    "md1fhd3vpspq2tvyyy4qqxppsg29wpyhwsv0jskp2rsal4egz4ep5859p875x67p5s3w6jy5p92uf5xrp",
    "md1fhd3vpswwl5z8lphut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2vkk2nlqmlvch0wqccjd2vncvg",
    "md1fhd3vps4yav9mj0jnn9svfgll7zvw5qnkxgeal5wa4wy86389z3gq870g9n6rjwuqgwh986tdk7dxr",
    "md1fhd3vpslkxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhacczakufnwezg4mz",
];
const ELIDED: [&str; 4] = [
    "md1fwlhkpspqggqpsgvzzshs3eutks264w7vm52uzfwaqcl9pvz58pmltjs9tjrg0g2z0agqv9emggu04pqgg",
    "md1fwlhkpsw67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e20urqdw3qtwekpkf7e",
    "md1fwlhkpsh7e30f8tpwunu5uevrz28llsnr4qya3jx00arhdt3p75feg52qpln6pv7sunhqpxvxsstm2usu6",
    "md1fwlhkpscrmrwqjfp99zcyrd98kjcu8vgdtu6gt04upy3z0n8ek4aj5kk9satjt7uvlfas3hpkyv6wk",
];
const SIGNET: [&str; 4] = [
    "md1fx6kzpspq2tvywz2sqrqscy9p0prnchdq442auehg4cyja6p372zc9gwrh7h9q2hyxs7s5mmuyg5wnu8kyf",
    "md1fx6kzpsgnl2rd0q6gghvalgy07r0ck4wcczrgalt7lhxlg0x6vnl4rdcjgnpya7k5edv4x5l8gpre2xywzw",
    "md1fx6kzpshcxlmx9ayav9mj0jnn9svfgll7zvw5qnkxgeal5wa4wy86389z3gq870g9n6rjwpwmec4za9gk4u",
    "md1fx6kzpsmsrmrwqjfp99zcyrd98kjcu8vgdtu6gt04upy3z0n8ek4aj5kk9satjt7uvqwz5x5s5nw4z9m",
];

struct Out {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// A temp dir holding every fixture md1 file and key file of the FIX's section 5.
fn fx() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let w = |n: &str, s: String| std::fs::write(d.path().join(n), s).unwrap();
    let lines = |c: &[&str]| c.join("\n") + "\n";
    w("a.md1", lines(&A));
    w("sparse.md1", lines(&SPARSE));
    w("rep.md1", lines(&REP));
    w("nofp.md1", lines(&NOFP));
    w("elided.md1", lines(&ELIDED));
    w("signet.md1", lines(&SIGNET));
    w(
        "twice.txt",
        format!("[73c5da0a/{P0}]{X0}\n[73c5da0a/{P0}]{X0}\n"),
    );
    w(
        "twice_h.txt",
        format!("[73c5da0a/{P0}]{X0}\n# same record, h spelling\n[73c5da0a/48h/0h/0h/2h]{X0}\n"),
    );
    w(
        "rep13.txt",
        format!("[73c5da0a/{P0}]{X0}\n[73c5da0a/48'/0'/1'/2']{X1}\n[73c5da0a/{P0}]{X0}\n"),
    );
    w(
        "twoorigins.txt",
        format!("[73c5da0a/{P0}]{X0}\n[73c5da0a/48'/0'/1'/2']{X0}\n"),
    );
    w(
        "twokeys.txt",
        format!("[73c5da0a/{P0}]{X0}\n\n[73c5da0a/{P0}]{X1}\n"),
    );
    w(
        "twomasters.txt",
        format!("[3442193e/]{TV1M}\n[3442193e/]{LEAFM}\n"),
    );
    w("batch9.txt", format!("[73c5da0a/48'/0'/9'/2']{X0}\n"));
    w(
        "distinct.txt",
        format!("[73c5da0a/{P0}]{X0}\n[73c5da0a/48'/0'/1'/2']{X1}\n[73c5da0a/48'/0'/2'/2']{X2}\n"),
    );
    w(
        "cosigners.txt",
        format!("[73c5da0a/{P0}]{X0}\n[aabbccdd/{P0}]{X1}\n"),
    );
    d
}

fn run_in(dir: &Path, args: &[&str], stdin: Option<&str>) -> Out {
    use std::io::Write;
    use std::process::Stdio;
    let mut c = Command::cargo_bin("mk").unwrap();
    c.current_dir(dir)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut ch = c.spawn().unwrap();
    {
        let mut si = ch.stdin.take().unwrap();
        if let Some(s) = stdin {
            si.write_all(s.as_bytes()).unwrap();
        }
    }
    let o = ch.wait_with_output().unwrap();
    Out {
        code: o.status.code(),
        stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
    }
}

fn run(args: &[&str]) -> Out {
    let d = fx();
    run_in(d.path(), args, None)
}

/// `--xpub` card flags followed by `extra`.
fn card<'a>(x: &'a str, path: &'a str, fp: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut v = vec![
        "encode",
        "--xpub",
        x,
        "--origin-path",
        path,
        "--origin-fingerprint",
        fp,
    ];
    v.extend_from_slice(extra);
    v
}

/// The exit-code spec §3 class of each refusal text: a card whose origin
/// contradicts its own keyed policy is a comparison that came out no (1, test
/// 4); a repeated record, key, origin or stub, and a policy that repeats a key,
/// are invalid inputs (2).
fn class_of(text: &str) -> (i32, &'static str) {
    if text.contains("contradicts its own policy") {
        (1, "Mismatch")
    } else {
        (2, "InvalidInput")
    }
}

fn refused(o: &Out, text: &str) {
    assert_eq!(o.code, Some(class_of(text).0), "stderr: {}", o.stderr);
    assert_eq!(o.stdout, "", "nothing on stdout");
    assert_eq!(o.stderr, format!("error: {text}\n"));
}

fn mints(o: &Out) {
    assert_eq!(o.code, Some(0), "stderr: {}", o.stderr);
    assert!(o.stdout.starts_with("mk1"), "stdout: {}", o.stdout);
}

fn t_path(x: &str, n: u32, stub: &str, policy: &str, card: &str) -> String {
    format!(
        "xpub {x} is @{n} of the wallet policy with stub {stub}, whose origin path for it is {policy}; the card declares {card}. A card whose origin contradicts its own policy is refused; no card was minted."
    )
}
fn t_path_std(x: &str, n: u32, stub: &str, policy: &str, card: &str) -> String {
    format!(
        "xpub {x} is @{n} of the wallet policy with stub {stub}, which states no origin path for it, so its policy id uses the standard origin path for its template, {policy}; the card declares {card}. A card whose origin contradicts its own policy is refused; no card was minted. If {card} is this key's origin, re-encode the md1 with the keys' real origins (md encode --path, or the origin in the template) and mint against the new md1."
    )
}
fn t_fp(x: &str, n: u32, stub: &str, policy_fp: &str, card_fp: &str) -> String {
    format!(
        "xpub {x} is @{n} of the wallet policy with stub {stub}, whose master fingerprint for it is {policy_fp}; the card declares {card_fp}. A card whose origin contradicts its own policy is refused; no card was minted."
    )
}
fn t_repkey(stub: &str, x: &str, slots: &str) -> String {
    format!(
        "the wallet policy with stub {stub} holds xpub {x} at {slots}; BIP 388 requires the keys of a policy to be pairwise distinct, so the policy is refused and no card was minted."
    )
}
fn t_rec(src: &str, j: u32, i: u32) -> String {
    format!(
        "{src}:{j}: this record repeats line {i}; a key is carded once, so a repeated record is refused and no card was minted."
    )
}
fn t_key2(src: &str, j: u32, i: u32, origin: &str) -> String {
    format!(
        "{src}:{j}: this record's key is also on line {i}, with origin {origin}; one key has one origin, so the two records contradict each other and no card was minted."
    )
}
fn t_origin2(src: &str, j: u32, origin: &str, i: u32) -> String {
    format!(
        "{src}:{j}: this record's origin {origin} is also on line {i}, with a different key; one origin derives one key (BIP 32), so the two records contradict each other and no card was minted."
    )
}
fn t_stub(stub: &str) -> String {
    format!(
        "policy stub {stub} is bound twice; a card lists each policy once, so a stub repeated across --policy-id-stub, --from-md1 and --from-md1-set is refused and no card was minted."
    )
}
fn json_err(text: &str) -> String {
    let (code, kind) = class_of(text);
    format!(
        "{{\"error\":{{\"details\":null,\"exit_code\":{code},\"kind\":\"{kind}\",\"message\":\"{text}\"}},\"schema_version\":1}}\n"
    )
}

// ---- bh-mk-I2: card origin against its own keyed policy ----

#[test]
fn fe_i2_path_contradicts_policy() {
    let o = run(&card(X0, M9, "73c5da0a", &["--from-md1-set", "a.md1"]));
    refused(&o, &t_path(X0, 0, "38bd7cec", M0, M9));
}

#[test]
fn fe_i2_path_contradicts_policy_json() {
    let o = run(&card(
        X0,
        M9,
        "73c5da0a",
        &["--from-md1-set", "a.md1", "--json"],
    ));
    assert_eq!(o.code, Some(1));
    assert_eq!(o.stderr, "");
    assert_eq!(o.stdout, json_err(&t_path(X0, 0, "38bd7cec", M0, M9)));
}

#[test]
fn fe_i2_fingerprint_contradicts_policy() {
    let o = run(&card(X0, M0, "deadbeef", &["--from-md1-set", "a.md1"]));
    refused(&o, &t_fp(X0, 0, "38bd7cec", "73c5da0a", "deadbeef"));
}

#[test]
fn fe_i2_second_slot_contradicts_policy() {
    let o = run(&card(X1, M1, "73c5da0a", &["--from-md1-set", "a.md1"]));
    refused(&o, &t_path(X1, 1, "38bd7cec", M0, M1));
}

#[test]
fn fe_i2_sparse_slot_contradicts_policy() {
    let o = run(&card(X0, M9, "73c5da0a", &["--from-md1-set", "sparse.md1"]));
    refused(&o, &t_path(X0, 0, "3a7f6520", M0, M9));
}

#[test]
fn fe_i2_batch_record_contradicts_policy() {
    let o = run(&["encode", "--keys", "batch9.txt", "--from-md1-set", "a.md1"]);
    refused(&o, &t_path(X0, 0, "38bd7cec", M0, M9));
}

#[test]
fn fe_i2_policy_repeats_key() {
    let o = run(&card(X0, M0, "73c5da0a", &["--from-md1-set", "rep.md1"]));
    refused(&o, &t_repkey("58e73a11", X0, "@0, @1"));
}

#[test]
fn fe_i2_elided_origin_path_contradicts_policy() {
    let o = run(&card(X0, M9, "73c5da0a", &["--from-md1-set", "elided.md1"]));
    refused(&o, &t_path_std(X0, 0, "38bd7cec", M0, M9));
}

#[test]
fn fe_i2_elided_origin_signet_card() {
    let o = run(&card(
        X0,
        MSIG,
        "73c5da0a",
        &["--from-md1-set", "elided.md1"],
    ));
    refused(&o, &t_path_std(X0, 0, "38bd7cec", M0, MSIG));
}

#[test]
fn fe_i2_elided_origin_master_card() {
    let o = run(&[
        "encode",
        "--xpub",
        X0D0,
        "--origin-path",
        "m",
        "--privacy-preserving",
        "--from-md1-set",
        "elided.md1",
    ]);
    refused(&o, &t_path_std(X0D0, 0, "38bd7cec", M0, "m"));
}

// ---- sweep row 15: key-record set checks ----

#[test]
fn fe_keys_identical_record_twice() {
    let o = run(&[
        "encode",
        "--keys",
        "twice.txt",
        "--policy-id-stub",
        "deadbeef",
    ]);
    refused(&o, &t_rec("twice.txt", 2, 1));
}

#[test]
fn fe_keys_identical_record_h_spelling() {
    let o = run(&[
        "encode",
        "--keys",
        "twice_h.txt",
        "--policy-id-stub",
        "deadbeef",
    ]);
    refused(&o, &t_rec("twice_h.txt", 3, 1));
}

#[test]
fn fe_keys_one_key_two_origins() {
    let o = run(&[
        "encode",
        "--keys",
        "twoorigins.txt",
        "--policy-id-stub",
        "deadbeef",
    ]);
    refused(
        &o,
        &t_key2("twoorigins.txt", 2, 1, &format!("[73c5da0a/{P0}]")),
    );
}

#[test]
fn fe_keys_one_origin_two_keys() {
    let o = run(&[
        "encode",
        "--keys",
        "twokeys.txt",
        "--policy-id-stub",
        "deadbeef",
    ]);
    refused(
        &o,
        &t_origin2("twokeys.txt", 3, &format!("[73c5da0a/{P0}]"), 1),
    );
}

#[test]
fn fe_keys_one_origin_two_masters() {
    let o = run(&[
        "encode",
        "--keys",
        "twomasters.txt",
        "--policy-id-stub",
        "deadbeef",
    ]);
    refused(&o, &t_origin2("twomasters.txt", 2, "[3442193e/]", 1));
}

#[test]
fn fe_keys_in_flag_and_stdin_same_checks() {
    let d = fx();
    let body = std::fs::read_to_string(d.path().join("twice.txt")).unwrap();
    let o = run_in(
        d.path(),
        &["encode", "--in", "-", "--policy-id-stub", "deadbeef"],
        Some(&body),
    );
    refused(&o, &t_rec("<stdin>", 2, 1));
}

#[test]
fn fe_keys_identical_record_non_adjacent() {
    let o = run(&[
        "encode",
        "--keys",
        "rep13.txt",
        "--policy-id-stub",
        "deadbeef",
    ]);
    refused(&o, &t_rec("rep13.txt", 3, 1));
}

// ---- mk-dup-stub ----

#[test]
fn fe_stub_repeated_policy_id_stub() {
    let o = run(&card(
        X0,
        M0,
        "73c5da0a",
        &[
            "--policy-id-stub",
            "deadbeef",
            "--policy-id-stub",
            "deadbeef",
        ],
    ));
    refused(&o, &t_stub("deadbeef"));
}

#[test]
fn fe_stub_repeated_non_adjacent() {
    let o = run(&card(
        X0,
        M0,
        "73c5da0a",
        &[
            "--policy-id-stub",
            "deadbeef",
            "--policy-id-stub",
            "11111111",
            "--policy-id-stub",
            "deadbeef",
        ],
    ));
    refused(&o, &t_stub("deadbeef"));
}

#[test]
fn fe_stub_repeated_non_adjacent_tail() {
    let o = run(&card(
        X0,
        M0,
        "73c5da0a",
        &[
            "--policy-id-stub",
            "deadbeef",
            "--policy-id-stub",
            "11111111",
            "--policy-id-stub",
            "11111111",
        ],
    ));
    refused(&o, &t_stub("11111111"));
}

#[test]
fn fe_stub_repeated_from_md1() {
    let o = run(&card(
        X0,
        M0,
        "73c5da0a",
        &["--from-md1", TPL, "--from-md1", TPL],
    ));
    refused(&o, &t_stub("559e64b2"));
}

#[test]
fn fe_stub_repeated_across_channels() {
    let o = run(&card(
        X0,
        M0,
        "73c5da0a",
        &["--policy-id-stub", "559e64b2", "--from-md1", TPL],
    ));
    refused(&o, &t_stub("559e64b2"));
}

// ---- controls: unchanged ----

#[test]
fn fe_control_true_origin_mints() {
    let o = run(&card(X0, M0, "73c5da0a", &["--from-md1-set", "a.md1"]));
    mints(&o);
    assert!(
        o.stderr.contains("1 of them carded here, 1 not carded"),
        "{}",
        o.stderr
    );
}

#[test]
fn fe_control_privacy_preserving_mints() {
    let o = run(&[
        "encode",
        "--xpub",
        X0,
        "--origin-path",
        M0,
        "--privacy-preserving",
        "--from-md1-set",
        "a.md1",
    ]);
    mints(&o);
}

#[test]
fn fe_control_policy_without_fingerprints_mints() {
    let o = run(&card(X0, M0, "deadbeef", &["--from-md1-set", "nofp.md1"]));
    mints(&o);
}

#[test]
fn fe_control_sparse_unfilled_slot_mints() {
    let o = run(&card(X1, M1, "73c5da0a", &["--from-md1-set", "sparse.md1"]));
    mints(&o);
}

#[test]
fn fe_control_complete_policy_nonmember_refused() {
    let o = run(&card(
        X2,
        "m/48'/0'/2'/2'",
        "73c5da0a",
        &["--from-md1-set", "a.md1"],
    ));
    assert_eq!(o.code, Some(1));
    assert_eq!(o.stdout, "");
    assert!(
        o.stderr
            .contains("is not a cosigner of the wallet policy with stub 38bd7cec"),
        "{}",
        o.stderr
    );
}

#[test]
fn fe_control_distinct_records_mint() {
    let o = run(&[
        "encode",
        "--keys",
        "distinct.txt",
        "--policy-id-stub",
        "deadbeef",
    ]);
    mints(&o);
    assert!(o.stderr.contains("card 3 of 3"), "{}", o.stderr);
}

#[test]
fn fe_control_both_cosigners_mint() {
    let o = run(&[
        "encode",
        "--keys",
        "cosigners.txt",
        "--from-md1-set",
        "a.md1",
    ]);
    mints(&o);
    assert!(o.stderr.contains("card 2 of 2"), "{}", o.stderr);
    assert!(
        !o.stderr.contains("not carded"),
        "no coverage note: {}",
        o.stderr
    );
}

#[test]
fn fe_control_two_distinct_stubs_mint() {
    let o = run(&card(
        X0,
        M0,
        "73c5da0a",
        &["--policy-id-stub", "deadbeef", "--from-md1", TPL],
    ));
    mints(&o);
}

#[test]
fn fe_control_chunk_set_given_twice_unchanged() {
    let o = run(&card(
        X0,
        M0,
        "73c5da0a",
        &["--from-md1-set", "a.md1", "--from-md1-set", "a.md1"],
    ));
    assert_eq!(o.code, Some(2), "{}", o.stderr);
    assert_eq!(o.stdout, "");
    assert!(
        o.stderr
            .contains("chunk set incomplete: got 8 chunks, expected 4"),
        "{}",
        o.stderr
    );
}

#[test]
fn fe_control_md_default_elided_origin_mints() {
    let o = run(&card(X0, M0, "73c5da0a", &["--from-md1-set", "elided.md1"]));
    mints(&o);
    assert!(
        o.stderr
            .contains("policy 38bd7cec has 2 cosigner(s); 1 of them carded here, 1 not carded"),
        "{}",
        o.stderr
    );
}

#[test]
fn fe_control_signet_card_against_path_md1_mints() {
    let o = run(&card(
        X0,
        MSIG,
        "73c5da0a",
        &["--from-md1-set", "signet.md1"],
    ));
    mints(&o);
    assert!(
        o.stderr
            .contains("policy 00912ee3 has 2 cosigner(s); 1 of them carded here, 1 not carded"),
        "{}",
        o.stderr
    );
}

// ---- bh-mk-I15: in-process fixtures, no sibling md binary ----

fn chunks_a() -> Vec<String> {
    A.iter().map(|s| s.to_string()).collect()
}

/// KEYED_POLICY_A with @1's key dropped, re-split.
fn chunks_sparse() -> Vec<String> {
    let refs: Vec<&str> = A.to_vec();
    let mut d = md_codec::reassemble(&refs).unwrap();
    let pk = d.tlv.pubkeys.as_mut().unwrap();
    assert_eq!(pk.len(), 2, "fixture must start complete");
    pk.retain(|(i, _)| *i == 0);
    md_codec::split(&d).unwrap()
}

fn mint_from(x: &str, path: &str, chunks: &[String]) -> Out {
    let mut args = vec![
        "encode",
        "--xpub",
        x,
        "--origin-fingerprint",
        "73c5da0a",
        "--origin-path",
        path,
        "--group-size",
        "0",
    ];
    for c in chunks {
        args.push("--from-md1");
        args.push(c);
    }
    let d = tempfile::tempdir().unwrap();
    run_in(d.path(), &args, None)
}

#[test]
fn fe_i15_short_coverage_note_is_printed() {
    let o = mint_from(X0, M0, &chunks_a());
    assert_eq!(o.code, Some(0), "{}", o.stderr);
    assert!(
        o.stderr.contains("1 of them carded here, 1 not carded"),
        "{}",
        o.stderr
    );
}

#[test]
fn fe_i15_nonmember_of_complete_policy_is_refused() {
    let o = mint_from(X2, "m/48'/0'/2'/2'", &chunks_a());
    assert!(
        o.code == Some(1) && o.stderr.contains("is not a cosigner"),
        "{}",
        o.stderr
    );
}

#[test]
fn fe_i15_sparse_policy_admits_unfilled_slot_key() {
    let ch = chunks_sparse();
    assert_eq!(mint_from(X0, M0, &ch).code, Some(0));
    let o = mint_from(X1, M1, &ch);
    assert_eq!(o.code, Some(0), "{}", o.stderr);
}
