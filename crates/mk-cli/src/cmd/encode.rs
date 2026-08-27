//! `mk encode` — produce one or more mk1 strings from xpub + origin metadata.
//!
//! Realizes SPEC §3.3 (Policy ID stub format) from the v0.2 plan.

use bitcoin::bip32::{
    DerivationPath as DerivationPath2, Fingerprint as Fingerprint2, Xpub as Xpub2,
};
use clap::Args;
use mk_codec::KeyCard;
use mk_codec::string_layer::header::MAX_CHUNK_SET_ID;
use serde_json::json;

use crate::cmd::{
    classify_code_variant, decode_md1_card, fmt_fingerprint, fmt_stub, group_md1_cards,
    parse_derivation_path, parse_fingerprint, parse_stub_hex, parse_xpub_normalized,
};
use crate::error::{CliError, Result};

/// `mk encode` arguments.
#[derive(Args, Debug)]
pub struct EncodeArgs {
    /// BIP 32 extended public key (xpub-prefixed string). Required unless
    /// `--keys` supplies the keys in a file.
    #[arg(long, required_unless_present_any = ["keys", "in_file"])]
    pub xpub: Option<String>,

    /// 8-hex-char master fingerprint. Mutually exclusive with `--privacy-preserving`.
    #[arg(long)]
    pub origin_fingerprint: Option<String>,

    /// Derivation path (e.g., `m/48'/0'/0'/2'`). Required unless `--keys`
    /// supplies the origins in a file.
    #[arg(long, required_unless_present_any = ["keys", "in_file"])]
    pub origin_path: Option<String>,

    /// Mint ONE card per key record in FILE (`-` for stdin), instead of the
    /// single card described by `--xpub`/`--origin-path`.
    ///
    /// Each record is BIP-380 origin notation on its own line --
    /// `[fingerprint/path]xpub` -- so a key cannot be separated from the origin
    /// it was derived at. Blank lines and `#` comments are ignored. Every card
    /// receives the same `--policy-id-stub`/`--from-md1` binding.
    #[arg(long, value_name = "FILE")]
    pub keys: Option<String>,

    /// Read the key records from FILE (`-` for stdin) -- `mk encode`'s own
    /// input material, so the same reader `--keys` uses. SPEC §6b. `--keys` is
    /// retained and the two are mutually exclusive.
    #[arg(long = "in", value_name = "FILE")]
    pub in_file: Option<String>,

    /// Write the mk1 artifact to FILE, created 0600, instead of to stdout.
    /// OVERWRITES an existing file (operator ruling, 2026-08-26). SPEC §6b.
    ///
    /// NOTE: `mk vectors --out` and `mk gen-man --out` already exist on this
    /// binary and mean a DIRECTORY. Both meanings are correct for their verbs
    /// and P3 does not unify them; do not "tidy" them together.
    #[arg(long = "out", value_name = "FILE")]
    pub out_file: Option<String>,

    /// Repeatable. Each value is 8 lowercase hex chars (4 bytes).
    #[arg(long)]
    pub policy_id_stub: Vec<String>,

    /// Repeatable. Each value is an md1 string; the stub is derived per SPEC §3.3.
    #[arg(long)]
    pub from_md1: Vec<String>,

    /// Repeatable. Read md1 strings from FILE -- what `md encode --out` writes
    /// -- and bind their stubs exactly as repeated `--from-md1` does. SPEC §10.
    ///
    /// Every line that is not an md1 string is SKIPPED, so a file carrying
    /// `md encode`'s `chunk-set-id:` header, blank lines or `#` comments works,
    /// and so will one written after that header moves to stderr. Display
    /// separators are stripped, so a grouped file works too. A file containing
    /// no md1 string at all is refused rather than bound as nothing.
    ///
    /// Stubs bind in flag order -- `--policy-id-stub`, then `--from-md1`, then
    /// `--from-md1-set` -- not in argv order. Stub order is on the wire, so
    /// mixing the channels in a different sequence mints a different card.
    #[arg(long, value_name = "FILE")]
    pub from_md1_set: Vec<String>,

    /// Encode without master fingerprint. Mutually exclusive with `--origin-fingerprint`.
    #[arg(long)]
    pub privacy_preserving: bool,

    /// Force chunked output even when single-string would fit. (Reserved for v0.2;
    /// mk-codec auto-dispatches today.)
    #[arg(long)]
    pub force_chunked: bool,

    /// Force long-code BCH variant. (Reserved for v0.2; mk-codec auto-dispatches today.)
    #[arg(long)]
    pub force_long_code: bool,

    /// Pin the 20-bit `chunk_set_id` (hex, `0x` prefix optional) instead of
    /// deriving it from the payload. Chunked output only — single-string
    /// encodings carry no such field. For vector regeneration and conformance
    /// fixtures; the derived default is already deterministic, so ordinary
    /// encoding never needs this.
    #[arg(long)]
    pub chunk_set_id: Option<String>,

    /// Insert a separator every N characters in each emitted mk1 string
    /// (0 = unbroken). SPEC §3. Display only; --json stays unbroken.
    #[arg(long, default_value_t = 5)]
    pub group_size: u16,

    /// Separator for the stderr engraving card: `space`, or the literal " ".
    /// Whitespace only (SPEC §6c); `hyphen` and `comma` were removed.
    #[arg(long, default_value = "space", value_parser = crate::format::parse_separator)]
    pub separator: char,

    /// Emit a single JSON object on stdout instead of one mk1 string per line.
    #[arg(long)]
    pub json: bool,
}

/// Run `mk encode`.
pub fn run(args: EncodeArgs) -> Result<u8> {
    if args.privacy_preserving && args.origin_fingerprint.is_some() {
        return Err(CliError::UsageError(
            "--privacy-preserving and --origin-fingerprint are mutually exclusive".into(),
        ));
    }

    if args.keys.is_some() && args.in_file.is_some() {
        return Err(CliError::UsageError(
            "--in and --keys are mutually exclusive; both name the key-record file, so \
             supplying two would leave it ambiguous which one minted the cards. Use --in."
                .into(),
        ));
    }
    // From here the two channels are ONE source, so every check below (the
    // single-card flag refusals, the batch mint path, the JSON envelope shape)
    // sees `--in` exactly as it has always seen `--keys`.
    let key_source: Option<(&str, &str)> = match (&args.in_file, &args.keys) {
        (Some(p), None) => Some((p.as_str(), "--in")),
        (None, Some(p)) => Some((p.as_str(), "--keys")),
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!("guarded above"),
    };

    if key_source.is_some() {
        // Each --keys record carries its own origin, so a global one would have
        // to either override it (engraving an origin the key was not derived
        // at) or be ignored. Both are worse than refusing.
        for (flag, present) in [
            ("--xpub", args.xpub.is_some()),
            ("--origin-path", args.origin_path.is_some()),
            ("--origin-fingerprint", args.origin_fingerprint.is_some()),
            // --chunk-set-id pins ONE card's 20-bit id; N cards cannot share it.
            ("--chunk-set-id", args.chunk_set_id.is_some()),
            ("--privacy-preserving", args.privacy_preserving),
        ] {
            if present {
                let given = key_source.expect("guarded by the enclosing if").1;
                return Err(CliError::UsageError(format!(
                    "{given} and {flag} are mutually exclusive; {}",
                    if flag == "--privacy-preserving" {
                        "a key record always declares a fingerprint, and dropping it \
                         silently is how a card gets engraved wrong -- mint \
                         privacy-preserving cards one at a time"
                    } else {
                        "each key record carries its own origin"
                    }
                )));
            }
        }
    }

    let mut stubs: Vec<[u8; 4]> = Vec::new();
    for s in &args.policy_id_stub {
        stubs.push(parse_stub_hex(s)?);
    }
    // One card per POLICY, not one per string: a keyed wallet policy always
    // arrives as a chunk set, so the values are grouped by chunk-set id first
    // and each GROUP contributes one stub.
    // Each GROUP is one policy card and contributes one stub. The cosigner set
    // is kept alongside so the keys being stamped can be checked against the
    // policy they claim membership in, below.
    // `md` prints md1 in display-grouped form by default, and `mk`'s own mk1
    // intake already strips those separators -- so a copy-pasted md1 string was
    // refused by the one flag that exists to consume it. Normalize first
    // (R1, 2026-08-21).
    let mut from_md1: Vec<String> = args
        .from_md1
        .iter()
        .map(|s| crate::format::strip_display_separators(s))
        .collect();
    // SPEC §10: `--from-md1-set FILE` is the repeated flag, read from what
    // `md encode` wrote. Files append in the order they were given, AFTER any
    // `--from-md1` values -- flag order, not argv order, which is the rule
    // `--policy-id-stub`-before-`--from-md1` already followed. Stub order is on
    // the wire, so this is pinned by a test rather than left to clap.
    for path in &args.from_md1_set {
        from_md1.extend(read_md1_set(path)?);
    }

    let mut policies: Vec<crate::cmd::Md1Card> = Vec::new();
    for card in group_md1_cards(&from_md1) {
        let decoded = decode_md1_card(&card)?;
        stubs.push(decoded.stub);
        policies.push(decoded);
    }
    if stubs.is_empty() {
        return Err(CliError::UsageError(
            "at least one of --policy-id-stub or --from-md1 is required".into(),
        ));
    }

    let fingerprint = match (&args.origin_fingerprint, args.privacy_preserving) {
        (Some(s), false) => Some(parse_fingerprint(s)?),
        (None, true) => None,
        (None, false) => None,
        (Some(_), true) => unreachable!("guarded above"),
    };

    let cards: Vec<(Option<Fingerprint2>, DerivationPath2, Xpub2)> = match key_source {
        Some((path, flag)) => crate::keyfile::read_key_records(path, flag)?
            .into_iter()
            .map(|r| {
                // --privacy-preserving is refused alongside --keys (guarded
                // above), so every batch record engraves its own fingerprint.
                (Some(r.fingerprint), r.path, r.xpub)
            })
            .collect(),
        None => {
            let path = parse_derivation_path(args.origin_path.as_deref().ok_or_else(|| {
                CliError::UsageError("--origin-path is required (or use --keys)".into())
            })?)?;
            let xpub_str = args
                .xpub
                .as_deref()
                .ok_or_else(|| CliError::UsageError("--xpub is required (or use --keys)".into()))?;
            let xpub = parse_xpub_normalized(xpub_str, Some(&path))?;
            vec![(fingerprint, path, xpub)]
        }
    };

    // The declared origin fingerprint, checked where it is PROVABLE from the
    // xpub itself.
    //
    // A record pairs a fingerprint, a path and a key, and nothing checked that
    // they describe the same thing -- so two same-depth cosigners could be
    // crossed by an operator editing the file, and the card would mint. Most
    // depths cannot be checked (the xpub carries its PARENT's fingerprint, not
    // the master's), but two can:
    //   depth 0 -- the xpub IS the master, so its own fingerprint must match;
    //   depth 1 -- its parent IS the master, so parent_fingerprint must match.
    // Narrow, but it is the only mechanical check on whether a record is
    // internally truthful, and it costs nothing (R2, 2026-08-21).
    for (fp, path, xpub) in &cards {
        let Some(declared) = fp else { continue };
        let depth = path.into_iter().count();
        // Only when the xpub's own depth agrees with the declared path. If they
        // disagree the record is structurally inconsistent and THAT is the real
        // error (the encoder reports it precisely); comparing a fingerprint
        // across a depth mismatch compares against a key that is not the one
        // the path describes, and would report a misleading cause. Caught by
        // this fix's own first test.
        if xpub.depth as usize != depth {
            continue;
        }
        let provable = match depth {
            0 => Some(("the xpub is the master key", xpub.fingerprint())),
            1 => Some((
                "the xpub's parent is the master key",
                xpub.parent_fingerprint,
            )),
            _ => None,
        };
        if let Some((why, actual)) = provable {
            if actual != *declared {
                return Err(CliError::UsageError(format!(
                    "origin fingerprint {} does not match the xpub: at depth {depth} {why}, \
                     so the fingerprint must be {}. The record pairs a key with an origin \
                     that is not its own.",
                    fmt_fingerprint(declared),
                    fmt_fingerprint(&actual),
                )));
            }
        }
    }

    // Membership: a key stamped with a KEYED policy's stub must actually be one
    // of that policy's cosigners.
    //
    // The claim an mk1 card makes is "this xpub is intended to serve the policy
    // with this stub" (SPEC 5). Minting that claim for a key the policy does not
    // contain produces a card that looks correct, engraves fine, and is refused
    // at recovery -- a wasted plate at best, and in a batch it is invisible
    // because the other records mint correctly and the run still exits 0.
    // The cosigner set is already parsed in-process, so refusing costs nothing.
    //
    // Only KEYED policies are checked. A keyless template carries no keys
    // (`cosigners == None`), so membership is not decidable from it and every
    // template-form card stays legal -- that is the ordinary bundle workflow.
    for (_fp, path, xpub) in &cards {
        let ident = crate::cmd::xpub_identity_65(xpub);
        for policy in &policies {
            let Some(cosigners) = &policy.cosigners else {
                continue;
            };
            if !cosigners.contains(&ident) {
                return Err(CliError::UsageError(format!(
                    "xpub {xpub} (origin {path}) is not a cosigner of the wallet policy \
                     with stub {}; that policy declares {} key(s). A card stamped with a \
                     policy it is not in is refused at recovery. (A keyless template md1 \
                     carries no keys and is not checked.)",
                    fmt_stub(&policy.stub),
                    cosigners.len(),
                )));
            }
        }
    }

    // Coverage: say so when the keys being carded do not cover every cosigner.
    //
    // A NOTE, not a refusal. Minting one card at a time is a legitimate and
    // common workflow -- a cosigner cards their own key without the others'
    // xpubs in hand -- so refusing an incomplete set would break the ordinary
    // case. But a --keys BATCH that silently produces N cards for N+1
    // cosigners is a short bundle, and a short bundle discovered at recovery
    // is the expensive way to find out. Membership is already enforced above,
    // so every card here is a genuine member; this is only about how many of
    // them are present (R2/I, 2026-08-21).
    for policy in &policies {
        let Some(cosigners) = &policy.cosigners else {
            continue;
        };
        let carded: std::collections::HashSet<[u8; 65]> = cards
            .iter()
            .map(|(_, _, x)| crate::cmd::xpub_identity_65(x))
            .collect();
        let missing = cosigners.iter().filter(|c| !carded.contains(*c)).count();
        if missing > 0 {
            eprintln!(
                "note: policy {} has {} cosigner(s); {} of them carded here, {} not carded",
                fmt_stub(&policy.stub),
                cosigners.len(),
                cosigners.len() - missing,
                missing,
            );
        }
    }

    // One mint path for both routes: a batch card and a single card differ only
    // in where their (fingerprint, path, xpub) came from, so they cannot drift.
    let mut minted: Vec<MintedCard> = Vec::with_capacity(cards.len());
    for (i, (fp, path, xpub)) in cards.into_iter().enumerate() {
        let card = KeyCard::new(stubs.clone(), fp, path.clone(), xpub);
        let encoded = match &args.chunk_set_id {
            Some(s) => mk_codec::encode_with_chunk_set_id(&card, parse_chunk_set_id(s)?),
            None => mk_codec::encode(&card),
        };
        // Name the RECORD. A parse failure already reports its line number, but
        // a failure here happens after parsing -- and without the record an
        // operator has to bisect an 11-line key file to find which cosigner
        // broke (R1, 2026-08-21).
        let strings = encoded.map_err(|e| {
            if let Some((_, flag)) = key_source {
                CliError::UsageError(format!(
                    "{flag} record {} ([{}/{}]): {}",
                    i + 1,
                    fp.as_ref()
                        .map(fmt_fingerprint)
                        .unwrap_or_else(|| "-".into()),
                    path,
                    CliError::from(e).message()
                ))
            } else {
                CliError::from(e)
            }
        })?;
        minted.push(MintedCard {
            fingerprint: fp,
            path,
            strings,
        });
    }

    // SPEC §6a: the artifact is UNGROUPED and stdout carries nothing else --
    // so `me sysw pack` can read what `mk encode` wrote with no
    // `--group-size 0` and no `grep` in between. No blank line between cards
    // either: a blank line is not an artifact. The grouped form a human
    // transcribes, and the card boundary, both move to the stderr engraving
    // card below (§6b: the grouping flags "affect the stderr card only").
    //
    // Built as ONE string rather than printed line by line, because §6b's
    // `--out` writes the same bytes to a file and a second emitter is a second
    // place for the two to drift.
    let artifact = if args.json {
        if key_source.is_some() {
            json_batch(&minted)?
        } else {
            json_single(&minted[0])?
        }
    } else {
        let mut body = String::new();
        for card in &minted {
            for s in &card.strings {
                body.push_str(s);
                body.push('\n');
            }
        }
        body
    };

    // SPEC §6b: "stdout is used when --out is not given." When it IS given the
    // artifact goes to the file ONLY -- writing both would put on stdout the
    // material `--out` exists to keep off it.
    match &args.out_file {
        Some(path) => crate::write::write_private(std::path::Path::new(path), artifact.as_bytes())
            .map_err(|e| CliError::UsageError(format!("--out {path}: {e}")))?,
        None => print!("{artifact}"),
    }

    // The card is a display of the artifact, so `--json` (which is explicitly
    // out of scope this cycle) does not get one.
    if !args.json {
        let grouped: Vec<Vec<String>> = minted.iter().map(|c| c.strings.clone()).collect();
        crate::format::write_engraving_card(
            &mut std::io::stderr(),
            &grouped,
            args.group_size as usize,
            args.separator,
        );
    }
    crate::output_advisory::emit_output_class_advisory(
        crate::output_advisory::OutputClass::WatchOnly,
        &mut std::io::stderr(),
    );
    Ok(0)
}

/// Read the md1 strings out of a `--from-md1-set` file.
///
/// **Every line that is not an md1 string is skipped**, which is what makes this
/// flag independent of which era wrote the file: today `md encode` prints a
/// `chunk-set-id: 0x…` header on stdout ahead of the artifact, and after SPEC
/// §6a that header is on stderr. Blank lines, `#` comments and an operator's own
/// annotations are skipped for the same reason. Display separators are stripped
/// first, so a grouped file and an unbroken file bind identically -- measured on
/// a real four-chunk set, byte-identical mk1 out.
///
/// **A file with NO md1 string in it is refused**, naming the file. Skipping is
/// what makes the flag tolerant; unguarded, it is also what would let a mistyped
/// path to a README bind zero stubs and then fail with a message about flags the
/// operator did supply -- or mint against a `--policy-id-stub` they also passed,
/// silently dropping the wallet they meant to bind.
fn read_md1_set(path: &str) -> Result<Vec<String>> {
    let buf = std::fs::read_to_string(path)
        .map_err(|e| CliError::UsageError(format!("--from-md1-set {path}: {e}")))?;
    let mut out = Vec::new();
    for line in buf.lines() {
        let s = crate::format::strip_display_separators(line);
        if s.len() >= 3 && s[..3].eq_ignore_ascii_case("md1") {
            out.push(s);
        }
    }
    if out.is_empty() {
        return Err(CliError::UsageError(format!(
            "--from-md1-set {path}: no md1 strings found (every line that does not start \
             with `md1` is skipped, so check the path and that the file holds `md encode` \
             output)"
        )));
    }
    Ok(out)
}

/// Parse `--chunk-set-id`: hex, `0x` prefix optional, and it must FIT.
///
/// A value over 20 bits is refused here rather than masked. Silently truncating
/// would emit a card under an id the caller did not ask for, and the caller's
/// whole reason for pinning is that the exact value matters.
fn parse_chunk_set_id(s: &str) -> Result<u32> {
    let body = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    let v = u32::from_str_radix(body, 16)
        .map_err(|e| CliError::UsageError(format!("--chunk-set-id: invalid hex {s:?}: {e}")))?;
    if v > MAX_CHUNK_SET_ID {
        return Err(CliError::UsageError(format!(
            "--chunk-set-id: {s:?} exceeds the 20-bit field (max 0x{MAX_CHUNK_SET_ID:05x})"
        )));
    }
    Ok(v)
}

/// JSON for `--keys`: a `cards` array whose entries are exactly the object
/// single-card `--json` emits. Additive, so a consumer of the single form can
/// read a batch entry without changes.
fn json_batch(minted: &[MintedCard]) -> Result<String> {
    let cards: Vec<_> = minted.iter().map(card_json).collect();
    let envelope = json!({
        "schema_version": 1,
        "card_count": cards.len(),
        "cards": cards,
    });
    let s = serde_json::to_string(&envelope)
        .map_err(|e| CliError::UsageError(format!("json serialization: {e}")))?;
    Ok(format!("{s}\n"))
}

/// The per-card JSON object, shared by the single and batch emitters.
///
/// Carries the card's ORIGIN as well as its strings. Without it the batch hands
/// back N interchangeable blocks whose only link to the input records is
/// position -- so a consumer captioning plates has no choice but to assume card
/// order still matches file order. That assumption is exactly the one this
/// project already has an incident for: 30 plates captioned with the wrong
/// cosigner. Naming each card lets a consumer JOIN on identity instead of
/// counting (R2/I + R3/I, 2026-08-21).
fn card_json(card: &MintedCard) -> serde_json::Value {
    let variant = card
        .strings
        .first()
        .map(|s| classify_code_variant(s))
        .unwrap_or("regular");
    json!({
        "mk1_strings": card.strings,
        "chunk_count": card.strings.len(),
        "code_variant": variant,
        "origin_fingerprint": card.fingerprint.as_ref().map(fmt_fingerprint),
        "origin_path": card.path.to_string(),
    })
}

/// One minted card: the strings, plus the origin they were minted for.
pub struct MintedCard {
    pub fingerprint: Option<Fingerprint2>,
    pub path: DerivationPath2,
    pub strings: Vec<String>,
}

fn json_single(card: &MintedCard) -> Result<String> {
    let mut envelope = card_json(card);
    envelope["schema_version"] = json!(1);
    let s = serde_json::to_string(&envelope)
        .map_err(|e| CliError::UsageError(format!("json serialization: {e}")))?;
    Ok(format!("{s}\n"))
}
