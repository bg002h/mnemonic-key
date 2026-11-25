//! Bug hunt 2026-11-25, mk#2 — `mk verify --from-md1` said "inputs match" for
//! a key that is not a cosigner of the keyed policy it was given.
//!
//! `verify` compared only the 4-byte stub, so a card minted with a bare
//! `--policy-id-stub` for an unrelated xpub (or with a contradictory origin)
//! verified OK against the full keyed policy, while `mk encode` refuses the
//! same pair. Now, against a `--from-md1` policy:
//!
//! - a COMPLETE key list that does not hold the card's key -> exit 4;
//! - a slot holding the card's key whose origin path or master fingerprint
//!   contradicts the card -> exit 4;
//! - a keyless template, or a partial key list that does not hold the key:
//!   membership cannot be decided, so only the stub is compared, as before,
//!   and a stderr note says so (a partial list is never proof either way).
//!
//! Public keys only. Fixtures as in `bh1119_keyed_policy.rs`.

use std::process::{Command, Output};

use assert_cmd::cargo::CommandCargoExt;

const X0: &str = "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf";
const X1: &str = "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk";
const X2: &str = "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR";
const V2_84_MAIN: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
/// Keyless `wpkh` template, stub 559e64b2.
const TPL: &str = "md1yqpqqxzq2qwfv8urt848e";
/// Keyed `wsh(multi(2,@0,@1))`, stub 38bd7cec: @0 `[73c5da0a/48'/0'/0'/2']X0`,
/// @1 `[aabbccdd/48'/0'/0'/2']X1`. Complete key list.
const A: [&str; 4] = [
    "md1fj5r4pspq2tvyyy4qqxppsg2z7z883w6pt24menw3tsf9m5rru59s2su80aw2q4wgdpapguu6w2y2jygprc",
    "md1fj5r4psf875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e2wnj3gvx34m0wnt",
    "md1fj5r4pshsdlkvt6f6cthyl98xtqcj3lluycagp8vv3nmlgam2ug04zw29zsq0u7st858yuyz646z0r98kyg",
    "md1fj5r4pslq8kxupyjz229sgx620d93cwcs6he5skltczfzylx0ndtm9fdvtp6hyhaccqrfrshqj9459eg",
];
/// Sparse keyed policy, stub 3a7f6520: @0 `[73c5da0a]X0`, @1 unkeyed.
const SPARSE: [&str; 3] = [
    "md1fmpkgpqpq2tvyyy4qqxppsg2z6zw0za5zs4gy4m5rru59s2su80aq5sat59s7xnghe",
    "md1fmpkgpq09q2hyxs7s5yl6smtcxjz9m806prlsm794tkxqs6806lhs8mg2jx2us05u6",
    "md1fmpkgpqkumapumfj075dhzfzvynh66n94j5lcxlmx9ayav9mj0jjqgknzrt66ggpxt",
];

const UNDECIDED: &str = "only its stub was compared";

fn mk() -> Command {
    Command::cargo_bin("mk").expect("mk binary")
}

fn desc(o: &Output) -> String {
    format!(
        "code={:?} stdout={:?} stderr={:?}",
        o.status.code(),
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// Mint a card with a BARE stub (not membership-checked), into a temp file.
fn stub_card(
    dir: &tempfile::TempDir,
    xpub: &str,
    path: &str,
    fp: Option<&str>,
    stub: &str,
) -> String {
    let mut args = vec![
        "encode",
        "--xpub",
        xpub,
        "--origin-path",
        path,
        "--policy-id-stub",
        stub,
    ];
    match fp {
        Some(fp) => args.extend(["--origin-fingerprint", fp]),
        None => args.push("--privacy-preserving"),
    }
    let o = mk().args(args).output().unwrap();
    assert_eq!(o.status.code(), Some(0), "mint: {}", desc(&o));
    let p = dir.path().join(format!("card{}.txt", rand_suffix()));
    std::fs::write(&p, &o.stdout).unwrap();
    p.to_str().unwrap().to_string()
}

fn rand_suffix() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    N.fetch_add(1, Ordering::Relaxed)
}

fn verify(card: &str, md1: &[&str], json: bool) -> Output {
    let mut args = vec!["verify".to_string(), "--in".into(), card.into()];
    for c in md1 {
        args.push("--from-md1".into());
        args.push((*c).into());
    }
    if json {
        args.push("--json".into());
    }
    mk().args(args).output().unwrap()
}

#[test]
fn non_cosigner_against_complete_keyed_policy_fails() {
    let d = tempfile::tempdir().unwrap();
    let card = stub_card(&d, X2, "m/48'/0'/2'/2'", Some("73c5da0a"), "38bd7cec");
    let o = verify(&card, &A, false);
    assert_eq!(o.status.code(), Some(4), "{}", desc(&o));
    assert!(o.stdout.is_empty(), "{}", desc(&o));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("not a cosigner"), "{}", desc(&o));
    assert!(err.contains("38bd7cec"), "{}", desc(&o));

    let o = verify(&card, &A, true);
    assert_eq!(o.status.code(), Some(4), "{}", desc(&o));
    let out = String::from_utf8_lossy(&o.stdout);
    assert!(!out.contains("\"ok\":true"), "{}", desc(&o));
    assert!(out.contains("ContentMismatch"), "{}", desc(&o));
}

#[test]
fn cosigner_at_its_policy_origin_passes() {
    let d = tempfile::tempdir().unwrap();
    for (x, fp) in [(X0, Some("73c5da0a")), (X1, Some("aabbccdd")), (X0, None)] {
        let card = stub_card(&d, x, "m/48'/0'/0'/2'", fp, "38bd7cec");
        let o = verify(&card, &A, false);
        assert_eq!(o.status.code(), Some(0), "{}", desc(&o));
        assert!(String::from_utf8_lossy(&o.stdout).starts_with("OK:"));
        assert!(!String::from_utf8_lossy(&o.stderr).contains(UNDECIDED));
    }
}

#[test]
fn cosigner_with_contradicting_path_fails() {
    let d = tempfile::tempdir().unwrap();
    let card = stub_card(&d, X0, "m/48'/0'/9'/2'", Some("73c5da0a"), "38bd7cec");
    let o = verify(&card, &A, false);
    assert_eq!(o.status.code(), Some(4), "{}", desc(&o));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("origin_path"), "{}", desc(&o));
    assert!(err.contains("@0"), "{}", desc(&o));
}

#[test]
fn cosigner_with_contradicting_fingerprint_fails() {
    let d = tempfile::tempdir().unwrap();
    let card = stub_card(&d, X0, "m/48'/0'/0'/2'", Some("deadbeef"), "38bd7cec");
    let o = verify(&card, &A, false);
    assert_eq!(o.status.code(), Some(4), "{}", desc(&o));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("origin_fingerprint"), "{}", desc(&o));
    assert!(err.contains("73c5da0a"), "{}", desc(&o));
}

/// A keyless template carries no keys: stub-only, as before, and said so.
#[test]
fn keyless_template_is_stub_only_and_says_so() {
    let d = tempfile::tempdir().unwrap();
    let card = stub_card(&d, V2_84_MAIN, "m/84'/0'/0'", Some("73c5da0a"), "559e64b2");
    let o = verify(&card, &[TPL], false);
    assert_eq!(o.status.code(), Some(0), "{}", desc(&o));
    assert!(
        String::from_utf8_lossy(&o.stderr).contains(UNDECIDED),
        "{}",
        desc(&o)
    );
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("559e64b2"),
        "{}",
        desc(&o)
    );
}

/// A partial key list is never proof: a key outside its filled slots is
/// neither refused nor claimed, only noted. A key IN a filled slot is
/// checked against that slot.
#[test]
fn sparse_policy_never_claims_membership() {
    let d = tempfile::tempdir().unwrap();
    let card = stub_card(&d, X1, "m/48'/0'/0'/2'", Some("aabbccdd"), "3a7f6520");
    let o = verify(&card, &SPARSE, false);
    assert_eq!(o.status.code(), Some(0), "{}", desc(&o));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains(UNDECIDED), "{}", desc(&o));
    assert!(err.contains("3a7f6520"), "{}", desc(&o));

    let card = stub_card(&d, X0, "m/48'/0'/0'/2'", Some("73c5da0a"), "3a7f6520");
    let o = verify(&card, &SPARSE, false);
    assert_eq!(o.status.code(), Some(0), "{}", desc(&o));
    assert!(
        !String::from_utf8_lossy(&o.stderr).contains(UNDECIDED),
        "{}",
        desc(&o)
    );
}

/// The stub comparison still comes first and is unchanged.
#[test]
fn stub_mismatch_unchanged() {
    let d = tempfile::tempdir().unwrap();
    let card = stub_card(&d, X0, "m/48'/0'/0'/2'", Some("73c5da0a"), "deadbeef");
    let o = verify(&card, &A, false);
    assert_eq!(o.status.code(), Some(4), "{}", desc(&o));
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("policy_id_stubs"),
        "{}",
        desc(&o)
    );
}
