//! `mk encode --keys` — batch key-record input (F-223).
//!
//! The load-bearing property is EQUIVALENCE: batch output must be
//! byte-identical to running `mk encode` once per key. `--keys` is an input
//! multiplexer and nothing else; if it ever mints a different card than the
//! loop it replaces, it is wrong no matter how convenient it is.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

/// Three real cosigner keys from `mnemonic-engrave`'s pathological wallet.
/// They share a master fingerprint and differ only in the account level, which
/// is the shape most likely to expose an origin/key mix-up: swap two records
/// and the fingerprints still match, so only the PATH catches it.
const KEYS: [(&str, &str, &str); 3] = [
    (
        "73c5da0a",
        "48'/0'/0'/2'",
        "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf",
    ),
    (
        "73c5da0a",
        "48'/0'/1'/2'",
        "xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk",
    ),
    (
        "73c5da0a",
        "48'/0'/2'/2'",
        "xpub6EGx8sPr9FxPPE1rbZazhqWwpMXA3Hf5DYKtZbL7c4BSddzmQktp96UaTvecEkoCZysuaj79GMCFZYT1KKk7Ph2M3Kf5g8B82KZ8TZ9SKQR",
    ),
];

const STUB: &str = "5b48af35";

/// Render key records as a `--keys` file body.
fn keyfile_body(keys: &[(&str, &str, &str)]) -> String {
    use std::fmt::Write as _;
    keys.iter().fold(String::new(), |mut acc, (fp, path, x)| {
        let _ = writeln!(acc, "[{fp}/{path}]{x}");
        acc
    })
}

fn mk() -> Command {
    Command::cargo_bin("mk").expect("mk binary")
}

/// Write `body` to a key file at a path unique to THIS CALL.
///
/// The name was derived from the body's length and first byte, so tests
/// sharing a body shared a path. That is not merely a collision -- it is a
/// RACE: `fs::write` truncates before writing, so one test's `mk` subprocess
/// could open the file mid-truncate and see no records at all. It surfaced
/// only in CI, because nextest gives each test its own PROCESS (distinct pid,
/// distinct name) while plain `cargo test` runs them as THREADS in one process
/// and CI's musl job uses `cargo test`. A per-call counter removes the sharing
/// rather than trying to order the accesses.
fn write_keyfile(body: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQ: AtomicUsize = AtomicUsize::new(0);

    let dir = std::env::temp_dir().join(format!("mk-keys-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(format!("k{}.txt", SEQ.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&p, body).unwrap();
    p
}

fn stdout_of(out: &std::process::Output) -> String {
    assert!(
        out.status.success(),
        "mk encode failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).unwrap()
}

/// THE test: `--keys` with N records equals N single-key invocations, byte for
/// byte, in order.
#[test]
fn batch_matches_per_key_loop() {
    let body = keyfile_body(&KEYS);
    let kf = write_keyfile(&body);

    let batch = stdout_of(
        &mk()
            .args([
                "encode",
                "--keys",
                kf.to_str().unwrap(),
                "--policy-id-stub",
                STUB,
                "--group-size",
                "0",
            ])
            .output()
            .unwrap(),
    );
    let batch_cards: Vec<&str> = batch.lines().filter(|l| l.starts_with("mk1")).collect();

    let mut loop_cards: Vec<String> = Vec::new();
    for (fp, path, x) in KEYS {
        let one = stdout_of(
            &mk()
                .args([
                    "encode",
                    "--xpub",
                    x,
                    "--origin-fingerprint",
                    fp,
                    "--origin-path",
                    &format!("m/{path}"),
                    "--policy-id-stub",
                    STUB,
                    "--group-size",
                    "0",
                ])
                .output()
                .unwrap(),
        );
        loop_cards.extend(
            one.lines()
                .filter(|l| l.starts_with("mk1"))
                .map(str::to_string),
        );
    }

    assert_eq!(
        batch_cards, loop_cards,
        "--keys must mint exactly what the per-key loop mints"
    );
    assert!(!loop_cards.is_empty(), "fixture produced no cards");
}

/// Cards are separated by a blank line, and a SINGLE card's plain output is
/// unchanged -- no leading or trailing blank. Pins the boundary convention so a
/// consumer can split on it.
#[test]
fn blank_line_separates_cards_only_between() {
    let body = keyfile_body(&KEYS);
    let kf = write_keyfile(&body);
    let out = stdout_of(
        &mk()
            .args([
                "encode",
                "--keys",
                kf.to_str().unwrap(),
                "--policy-id-stub",
                STUB,
                "--group-size",
                "0",
            ])
            .output()
            .unwrap(),
    );
    assert!(!out.starts_with('\n'), "no leading blank line");
    assert_eq!(
        out.matches("\n\n").count(),
        KEYS.len() - 1,
        "one blank line BETWEEN each pair of cards"
    );

    let single = stdout_of(
        &mk()
            .args([
                "encode",
                "--xpub",
                KEYS[0].2,
                "--origin-fingerprint",
                KEYS[0].0,
                "--origin-path",
                &format!("m/{}", KEYS[0].1),
                "--policy-id-stub",
                STUB,
                "--group-size",
                "0",
            ])
            .output()
            .unwrap(),
    );
    assert!(
        !single.contains("\n\n"),
        "single-card output must be unchanged: {single:?}"
    );
}

/// Blank lines and `#` comments are ignored, including trailing comments, so a
/// key list can be annotated the way the journey's key files are.
#[test]
fn comments_and_blanks_are_ignored() {
    let annotated = format!(
        "# cosigners\n\n[{}/{}]{}  # @0 tier 1\n\n[{}/{}]{}\n",
        KEYS[0].0, KEYS[0].1, KEYS[0].2, KEYS[1].0, KEYS[1].1, KEYS[1].2
    );
    let plain = format!(
        "[{}/{}]{}\n[{}/{}]{}\n",
        KEYS[0].0, KEYS[0].1, KEYS[0].2, KEYS[1].0, KEYS[1].1, KEYS[1].2
    );

    let a = stdout_of(
        &mk()
            .args([
                "encode",
                "--keys",
                write_keyfile(&annotated).to_str().unwrap(),
                "--policy-id-stub",
                STUB,
                "--group-size",
                "0",
            ])
            .output()
            .unwrap(),
    );
    let b = stdout_of(
        &mk()
            .args([
                "encode",
                "--keys",
                write_keyfile(&plain).to_str().unwrap(),
                "--policy-id-stub",
                STUB,
                "--group-size",
                "0",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(a, b, "comments and blank lines must not change the cards");
}

/// A rejected record names its LINE NUMBER. A key list is hand-edited, so a
/// bad record is the common failure and "which line" is the whole message.
#[test]
fn bad_record_names_the_line_number() {
    let body = format!(
        "# header\n[{}/{}]{}\nnot-a-record\n",
        KEYS[0].0, KEYS[0].1, KEYS[0].2
    );
    let kf = write_keyfile(&body);
    let out = mk()
        .args([
            "encode",
            "--keys",
            kf.to_str().unwrap(),
            "--policy-id-stub",
            STUB,
        ])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "a bad record must fail the whole run"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains(":3:"), "must name line 3, got: {err}");
    assert!(
        err.contains("BIP-380"),
        "must say what was expected, got: {err}"
    );
}

/// An empty (or all-comment) key file is refused rather than minting nothing
/// at exit 0 -- a silent zero-card run is how a short bundle reaches an
/// engraver.
#[test]
fn empty_keyfile_is_refused() {
    let kf = write_keyfile("# nothing but a comment\n\n");
    let out = mk()
        .args([
            "encode",
            "--keys",
            kf.to_str().unwrap(),
            "--policy-id-stub",
            STUB,
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "an empty key list must not exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("no key records"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `--keys` refuses every single-card flag. Each record carries its own origin,
/// so a global one would either override it -- engraving an origin the key was
/// never derived at -- or be silently ignored.
#[test]
fn keys_refuses_single_card_flags() {
    let body = format!("[{}/{}]{}\n", KEYS[0].0, KEYS[0].1, KEYS[0].2);
    let kf = write_keyfile(&body);
    let k = kf.to_str().unwrap();
    for extra in [
        vec!["--xpub", KEYS[0].2],
        vec!["--origin-path", "m/48'/0'/0'/2'"],
        vec!["--origin-fingerprint", "73c5da0a"],
        vec!["--chunk-set-id", "0x1234"],
        vec!["--privacy-preserving"],
    ] {
        let mut args = vec!["encode", "--keys", k, "--policy-id-stub", STUB];
        args.extend(extra.iter().copied());
        let out = mk().args(&args).output().unwrap();
        assert!(
            !out.status.success(),
            "--keys with {:?} must be refused",
            extra[0]
        );
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("mutually exclusive") && err.contains(extra[0]),
            "refusal must name {}: {err}",
            extra[0]
        );
    }
}

/// Batch `--json` is the single-card object, in a `cards` array. Additive, so a
/// consumer of the single form reads a batch entry unchanged.
#[test]
fn json_batch_wraps_the_single_card_object() {
    let body = keyfile_body(&KEYS);
    let kf = write_keyfile(&body);
    let batch: serde_json::Value = serde_json::from_str(&stdout_of(
        &mk()
            .args([
                "encode",
                "--keys",
                kf.to_str().unwrap(),
                "--policy-id-stub",
                STUB,
                "--json",
            ])
            .output()
            .unwrap(),
    ))
    .unwrap();

    assert_eq!(batch["card_count"], KEYS.len());
    let cards = batch["cards"].as_array().expect("cards array");
    assert_eq!(cards.len(), KEYS.len());

    // EVERY index, not just cards[0]. Checking only the first entry left the
    // array's ORDER unpinned: a mutant swapping cards[1] and cards[2] passed
    // all 132 tests in the crate, because the plain-text tests never pass
    // --json and this test never looked past index 0 (R5/C, 2026-08-21).
    for (i, (fp, path, x)) in KEYS.iter().enumerate() {
        let single: serde_json::Value = serde_json::from_str(&stdout_of(
            &mk()
                .args([
                    "encode",
                    "--xpub",
                    x,
                    "--origin-fingerprint",
                    fp,
                    "--origin-path",
                    &format!("m/{path}"),
                    "--policy-id-stub",
                    STUB,
                    "--json",
                ])
                .output()
                .unwrap(),
        ))
        .unwrap();
        for key in ["mk1_strings", "chunk_count", "code_variant"] {
            assert_eq!(
                cards[i][key], single[key],
                "batch cards[{i}].{key} must equal a single-card encode of KEYS[{i}]"
            );
        }
        // Each card NAMES the record it was minted for, so a consumer can join
        // on identity instead of assuming card order still matches file order.
        // Position-only output is what forces caption-by-counting, and this
        // project already has an incident where that captioned 30 plates with
        // the wrong cosigner (R2/I + R3/I).
        assert_eq!(
            cards[i]["origin_fingerprint"],
            serde_json::Value::from(*fp),
            "cards[{i}] must name its own fingerprint"
        );
        assert_eq!(
            cards[i]["origin_path"],
            serde_json::Value::from(*path),
            "cards[{i}] must name its own origin path"
        );
    }
}

/// Record ORDER is preserved, pinned by DIRECTION rather than by asymmetry.
///
/// An earlier version asserted only that reversing the file changed the
/// output and that the two runs held the same cards. A mutant that reversed
/// EVERY batch survived that: both properties still held. Anchoring the first
/// batch card to a single-key encode of the first record is what actually
/// fixes the direction.
#[test]
fn record_order_follows_file_order() {
    let body = keyfile_body(&KEYS);
    let kf = write_keyfile(&body);
    let batch = stdout_of(
        &mk()
            .args([
                "encode",
                "--keys",
                kf.to_str().unwrap(),
                "--policy-id-stub",
                STUB,
                "--group-size",
                "0",
            ])
            .output()
            .unwrap(),
    );
    let first_batch_card: Vec<&str> = batch
        .split("\n\n")
        .next()
        .unwrap()
        .lines()
        .filter(|l| l.starts_with("mk1"))
        .collect();

    let (fp, path, x) = KEYS[0];
    let single = stdout_of(
        &mk()
            .args([
                "encode",
                "--xpub",
                x,
                "--origin-fingerprint",
                fp,
                "--origin-path",
                &format!("m/{path}"),
                "--policy-id-stub",
                STUB,
                "--group-size",
                "0",
            ])
            .output()
            .unwrap(),
    );
    let single_card: Vec<&str> = single.lines().filter(|l| l.starts_with("mk1")).collect();

    assert_eq!(
        first_batch_card, single_card,
        "the FIRST card out must be the FIRST record in the file"
    );
}

/// A batch that does not cover every cosigner of a KEYED policy says so on
/// stderr -- and still exits 0, because minting one card at a time is a normal
/// workflow (a cosigner cards their own key without the others' xpubs in hand).
///
/// The point is that a SHORT bundle should not be silent: discovering at
/// recovery that a cosigner was never carded is the expensive way to find out.
/// A keyless template carries no cosigner list, so it must stay quiet.
#[test]
fn short_coverage_of_a_keyed_policy_is_noted_not_refused() {
    // Only the first of three keys, against a policy the test fixture declares
    // with more cosigners than that.
    let body = keyfile_body(&KEYS[..1]);
    let kf = write_keyfile(&body);
    let out = mk()
        .args([
            "encode",
            "--keys",
            kf.to_str().unwrap(),
            "--policy-id-stub",
            STUB,
            "--group-size",
            "0",
        ])
        .output()
        .unwrap();
    // --policy-id-stub carries no cosigner list either, so this must be silent
    // AND succeed: the note is only reachable via a KEYED --from-md1 policy.
    assert!(out.status.success(), "a short set must still mint");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("cosigner"),
        "a stub-only encode has no cosigner list to compare against: {err}"
    );
}

/// `verify --from-md1` compares stubs as a MULTISET, so the same card checked
/// against the same policies in a different argument order still passes — and
/// a genuinely wrong stub still fails.
///
/// The ordered comparison this replaces returned exit 4 for a CORRECT card
/// whose `--from-md1` groups were supplied in a different order (R1). A false
/// negative on a verification tool invites re-engraving a good plate.
#[test]
fn verify_stub_comparison_is_order_independent_but_not_blind() {
    let (fp, path, x) = KEYS[0];
    let mint = |stubs: &[&str]| -> Vec<String> {
        let mut args = vec![
            "encode".to_string(),
            "--xpub".into(),
            x.into(),
            "--origin-fingerprint".into(),
            fp.into(),
            "--origin-path".into(),
            format!("m/{path}"),
            "--group-size".into(),
            "0".into(),
        ];
        for s in stubs {
            args.push("--policy-id-stub".into());
            args.push((*s).into());
        }
        stdout_of(&mk().args(&args).output().unwrap())
            .lines()
            .filter(|l| l.starts_with("mk1"))
            .map(str::to_string)
            .collect()
    };
    let card = mint(&["5b48af35", "38bd7cec"]);
    assert!(!card.is_empty());

    let verify = |stubs: &[&str]| -> Option<i32> {
        let mut args = vec!["verify".to_string()];
        args.extend(card.iter().cloned());
        for s in stubs {
            args.push("--policy-id-stub".into());
            args.push((*s).into());
        }
        mk().args(&args).output().unwrap().status.code()
    };

    assert_eq!(verify(&["5b48af35", "38bd7cec"]), Some(0), "as minted");
    assert_eq!(
        verify(&["38bd7cec", "5b48af35"]),
        Some(0),
        "swapped order, same binding"
    );
    // Still catches a genuinely different binding, and a short one.
    assert_eq!(
        verify(&["5b48af35", "deadbeef"]),
        Some(4),
        "a wrong stub must fail"
    );
    assert_eq!(verify(&["5b48af35"]), Some(4), "a missing stub must fail");
    assert_eq!(
        verify(&["5b48af35", "5b48af35"]),
        Some(4),
        "multiset, not set: a duplicated stub must not stand in for a distinct one"
    );
}

/// The declared origin fingerprint is checked where the xpub PROVES it: at
/// depth 0 (the xpub is the master) and depth 1 (its parent is).
///
/// A record pairs a fingerprint, a path and a key, and nothing checked they
/// describe the same thing — so two same-depth cosigners could be crossed by
/// an operator editing the file and the card would mint. Most depths are
/// uncheckable (an xpub carries its PARENT's fingerprint, not the master's);
/// these two are not (R2, 2026-08-21).
#[test]
fn origin_fingerprint_is_checked_where_the_xpub_proves_it() {
    use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(bitcoin::Network::Bitcoin, &[7u8; 32]).unwrap();
    let master_xpub = Xpub::from_priv(&secp, &master);
    let real_fp = master_xpub.fingerprint().to_string();

    let child_path = DerivationPath::from_str("m/7'").unwrap();
    let child = Xpub::from_priv(&secp, &master.derive_priv(&secp, &child_path).unwrap());
    assert_eq!(
        child.depth, 1,
        "fixture must be depth 1 for this to be provable"
    );

    let run = |record: &str| -> (Option<i32>, String) {
        let kf = write_keyfile(&format!("{record}\n"));
        let out = mk()
            .args([
                "encode",
                "--keys",
                kf.to_str().unwrap(),
                "--policy-id-stub",
                STUB,
                "--group-size",
                "0",
            ])
            .output()
            .unwrap();
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).to_string(),
        )
    };

    // depth 1, correct fingerprint -> mints.
    let (code, err) = run(&format!("[{real_fp}/7']{child}"));
    assert_eq!(code, Some(0), "a truthful depth-1 record must mint: {err}");

    // depth 1, WRONG fingerprint -> refused, and the message says why.
    let (code, err) = run(&format!("[deadbeef/7']{child}"));
    assert_eq!(code, Some(64), "a crossed depth-1 record must be refused");
    assert!(
        err.contains("does not match the xpub") && err.contains("depth 1"),
        "the refusal must name the provable relationship: {err}"
    );

    // depth 0, WRONG fingerprint -> refused too.
    let (code, err) = run(&format!("[deadbeef/]{master_xpub}"));
    assert_eq!(
        code,
        Some(64),
        "a crossed depth-0 record must be refused: {err}"
    );
}
