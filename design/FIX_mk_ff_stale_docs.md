# FIX_mk_ff_stale_docs: bughunt-1119 F-F, stale normative and consumer docs (bh-mk-I5, bh-mk-I14)

Small-fix track, grade I (both keys). Class D (docs; no crate behaviour changes). Repo mnemonic-key,
branch `fix/mk-ff-stale-docs`, base `origin/main` 9b0b365 ("Merge pull request #30 from
bg002h/build/mk19"). No toolkit CLI-manual lockstep (no `mk` flag or printed text changes).
Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22.

Inputs (decided; not re-ruled here): coordinator `agent-reports/architect-bughunt-1119-questions.md`
row #43 and T44 ("replace the formula with the current one; keep a labelled legacy note only if a
tagged release printed the old ID"); grader `agent-reports/bughunt-1119/grade-mk.md` (I5, I14, F-F);
triage `triage-mk-t2.md` (I5) and `triage-mk-t3.md` (I14).

## 1. Defects (at 9b0b365; rows in section 4 run and seen)

- **bh-mk-I5.** The mk1 recovery flow's cryptographic step prints a recipe no current tool
  computes, and cites a helper md-codec no longer has:
  - `design/SPEC_mk_v0_1.md` L366-372 (§5 step 4): `wallet_instance_id = SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]`
    over the 78-byte BIP 32 serializations; L379 (implementation note): "the
    `compute_wallet_instance_id(canonical_bytecode, xpubs)` helper is provided by md-codec v0.8.0+".
  - `bip/bip-mnemonic-key.mediawiki` L39 (glossary), L47 (naming note item 2), L602 (recovery
    step 4) print the same formula; L48 says "(The Wallet Instance ID is unaffected.)"; L37 says the
    WalletPolicyId covers "not the concrete cosigner xpubs".
  - Value (row N4): for the keyed policy card `KEYED_POLICY_A` (tests/from_md1_set.rs) with its
    two xpubs X0, X1, the printed recipe gives `c86829b5ab2090a5b712b7ba1e64386c`; the card's
    WalletPolicyId (`md_codec::compute_wallet_policy_id`, md-codec 0.42.0) is
    `38bd7cec8059cdc4553c2836d7e8e303`. A recovery tool following the SPEC rejects the correct anchor.
  - md-codec history (checked with `git grep` per tag in descriptor-mnemonic): `compute_wallet_instance_id`
    was added in md-codec v0.8.0 (35c81198) and removed in v0.12.0 (5350f8a7).
- **bh-mk-I14.** `docs/MK_CODEC_RUST_API.md` fails as printed: L12 pins `tag = "mk-codec-v0.2.2"`,
  a tag that does not exist (`git ls-remote --tags origin`: none; the latest mk-codec tag is
  `mk-codec-v0.5.1` = e3eca63); the manifest has no `bitcoin` dependency though the example `use`s
  `bitcoin::bip32`; L48 parses the elided literal `"xpub6CatWdiZi..."`, which panics
  (`Base58(Decode(InvalidCharacterError { invalid: 46 }))`); L141-142 say the guide "targets v0.2.2".

## 2. Decided behaviour

- **T44 legacy check: no legacy section.** No tagged release printed the old ID. mk: none of the 39
  tags carries `instance_id`/`WalletInstanceId` in `crates/`. md: the md-codec tags v0.8.0-v0.11.0
  exported the library function only; no md-cli source at any tag calls it. mnemonic-toolkit: no
  commit ever added `compute_wallet_instance_id` or `WalletInstanceId` to code. So the formula is
  replaced, not kept as a labelled legacy section. The one history item that described md-codec
  v0.8.0 (BIP naming note item 2) keeps its history sentence but no longer prints the formula.
- **The current formula.** The Wallet Instance ID is the WalletPolicyId of the assembled descriptor:
  `md_codec::compute_wallet_policy_id` over the policy with each `@N` slot's concrete xpub filled
  in. md's own BIP states this (md origin/main 1097371b, `bip/bip-mnemonic-descriptor.mediawiki`
  §"Policy identifier", L899-904: "There is no separate on-card 'Wallet Instance ID' value and no
  separate hash domain: the per-instance identity is the same WalletPolicyId, computed over a
  preimage that includes the xpubs"; the preimage MUST use the 65-byte wire form). Row N4 shows the
  recipe works: the template (pubkeys removed) with X0, X1 plugged back in reproduces `38bd7cec...`.
- **The term "Wallet Instance ID" stays** (README L78, DECISIONS L96, SPEC L231/L377/L412, BIP
  L49/L637/L651 use it as the name of the step-4 check, and they stay true); only its definition changes.
- **Fingerprint presence is part of the recipe.** The WalletPolicyId is presence-significant on
  fingerprints: the same assembly without fingerprints gives `949c621144d15257e4cc94921c8dbbc3`
  (row N4). The new step 4 names the source (key card's `origin_fingerprint` or the policy card's, reject if they differ) and the fallback when no card carries the fingerprint the expected identity used (another record of the wallet, else first-address verification; architect-mk-ff-i2). Swapped slots give `780f6305cae4a2412ab398ff78dc426a`.
- **Guide pin: tag `mk-codec-v0.5.1`** (latest tag; the triage's proven GREEN). crates.io has
  0.5.0 at most and the repo is at 0.6.0 unreleased, so a 0.6.0 pin would fail as printed. The
  guide's existing "Validating a decoded card" section already names mk-codec 0.6.0 for
  `KeyCard::validate()`; the Stability paragraph says so (PROPOSED). The example xpub is the
  completed value of the guide's own prefix: the public BIP-84 test-vector account key
  `m/84'/0'/0'` (master fingerprint 73c5da0a, the fingerprint the example already passes),
  `xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V`
  (derived in a scratch test with rust-bitcoin 0.32 and seen to match the prefix and fingerprint).

## 3. Text delta (exact; anchors at 9b0b365)

All new sentences are PROPOSED (architect confirms). Each "replace" is one exact substring,
present once in its file.

### 3.1 `design/SPEC_mk_v0_1.md`

- L366-372 (the step-4 item, from `4. Compute the **Wallet Instance ID** for the assembled wallet:`
  through the line ending `Reject on mismatch.`; the `5. Accept ...` line stays) become exactly the
  three `+` lines of the first SPEC hunk in Appendix B (step text; a blank line; a second paragraph
  indented three spaces). The new step reads: "Compute the Wallet Instance ID for the assembled
  wallet: the WalletPolicyId of the assembled descriptor, md_codec::compute_wallet_policy_id(&assembled)
  (md SPEC v0.13 §5.3). With concrete xpubs in its @N slots the identity's preimage covers them,
  so this one value is the per-instance check; there is no separate instance hash (md BIP §"Policy
  identifier"). The preimage takes each xpub's 65-byte wire form (chain code || compressed public
  key), not its 78-byte BIP 32 serialization. / The value is presence-significant on fingerprints as well as xpubs. An @N's origin fingerprint in the assembled descriptor is its key card's origin_fingerprint or the policy card's fingerprint for that @N; if both cards carry one and they differ, reject the assembly. An @N that neither card gives one has none (a privacy-preserving key card carries none). Compare against the wallet identity the user expected to recover (e.g., a separately-anchored WalletPolicyId, its 12-word phrase from WalletPolicyId::to_phrase, or a digital backup record). Reject on mismatch, with one exception: when some @N has no fingerprint, the expected identity may have been computed with one, so first take those fingerprints from another record of the same wallet (its descriptor backup, or the master fingerprint of that cosigner's seed) and recompute. If no record supplies them, step 4 cannot confirm the wallet: say so, and verify the first address derived from the assembled descriptor against an independently-recorded expected address (§6) before any funds move. Fingerprints do not change addresses."
- L379, replace the whole line with:
  `**Implementation note:** step 4 is md-codec's `compute_wallet_policy_id` applied to the assembled descriptor. mk1 implementations SHOULD call it directly rather than reimplementing the preimage construction.`

### 3.2 `bip/bip-mnemonic-key.mediawiki`

1. L37, replace `an encoder-divergence-free hash of an MD-encoded policy ''template'' — placeholders + shared path, not the concrete cosigner xpubs (a one-to-many relationship to actual wallets, not a wallet instance).`
   with `an encoder-divergence-free hash of an MD-encoded policy: the template, the per-<code>@N</code> paths and, when the card carries them, the concrete fingerprints and xpubs (presence-significant). A template card's value is therefore one-to-many over actual wallets; a card with concrete xpubs identifies one wallet instance (see '''Wallet Instance ID''').`
   (Not in the grader's line list; needed because the old sentence contradicts the new L39.)
2. L39, replace the whole bullet with:
   `* '''Wallet Instance ID''': the 16-byte '''WalletPolicyId''' (<code>md_codec::compute_wallet_policy_id</code>) of the assembled descriptor — the policy template with each <code>@N</code> slot's concrete xpub filled in (and its origin fingerprint, from the MK or MD card as recovery step 4 states). Its preimage covers those keys, so it identifies a wallet instance — a template plus a specific cosigner xpub set; there is no separate instance hash (MD BIP §"Policy identifier"). Recovery tools compute it on demand from policy + assembled xpubs.`
3. L47, replace the whole item with:
   `# Defined a new derived '''Wallet Instance ID''' as a recovery-time computation, documented in the MD BIP but not carried by any physical card. Its v0.8.0 formula is retired: the Wallet Instance ID is now the WalletPolicyId of the assembled descriptor (see the glossary).`
4. L48, replace `(The Wallet Instance ID is unaffected.)` with
   `(The Wallet Instance ID is the same WalletPolicyId, computed over the descriptor with its concrete xpubs; see the glossary.)`
5. L602, replace `Compute the Wallet Instance ID = <code>SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]</code> where <code>canonical_xpub_serialization</code> is the concatenation of each <code>@N</code>-placeholder's resolved 78-byte xpub serialization in placeholder-index order.`
   with `Compute the Wallet Instance ID = <code>md_codec::compute_wallet_policy_id(assembled_descriptor)</code>, the WalletPolicyId of the assembled descriptor (its preimage takes each xpub's 65-byte wire form, not the 78-byte BIP 32 serialization). The value is presence-significant on fingerprints and xpubs: an <code>@N</code>'s origin fingerprint is its MK card's <code>origin_fingerprint</code> or the MD policy card's fingerprint for that <code>@N</code>; if both cards carry one and they differ, reject the assembly; an <code>@N</code> that neither card gives one has none. Verify the Wallet Instance ID matches the wallet the user expected to recover (e.g., compared against a separately-anchored WalletPolicyId phrase, or against a digital backup record). If it does not match while some <code>@N</code> has no fingerprint, first take those fingerprints from another record of the same wallet (its descriptor backup, or the master fingerprint of that cosigner's seed) and recompute. If no record supplies them, the Wallet Instance ID cannot confirm the wallet: say so, and instead of the next step verify the first address derived from the assembled descriptor against an independently-recorded expected address (§"Privacy considerations") before any funds move. Fingerprints do not change addresses.`
6. L602, replace `compared against a separately-anchored Wallet Instance ID phrase, or against`
   with `compared against a separately-anchored WalletPolicyId phrase, or against`.

### 3.3 `docs/MK_CODEC_RUST_API.md`

- L12, replace the line with two lines:
  `mk-codec = { git = "https://github.com/bg002h/mnemonic-key", tag = "mk-codec-v0.5.1" }`
  `bitcoin = "0.32"`
- L48, replace `"xpub6CatWdiZi..."` with
  `"xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V"`.
- L141-142, replace `` `mk-codec` is at v0.2 (post-cycle close-out). v0.1 of the manual`` / ``targets v0.2.2; semver-major bumps may break the API.`` with the three lines
  ``The manifest above pins the tag `mk-codec-v0.5.1`, the latest tagged`` /
  ``release; the `KeyCard::validate()` section describes mk-codec 0.6.0.`` /
  `Semver-major bumps may break the API.` (the rest of the paragraph stays).

### 3.4 Not changed

README L78, DECISIONS L92/L96, CLAUDE.md L51, `gen_mk_vectors.rs` L991 comment,
IMPLEMENTATION_PLAN and PLAN_stub_formula_walletpolicyid (history): they use the term, not the
formula. BIP L605 / SPEC L375 ("Step 2 is the indexing aid — template-level") are left as they are
(see the report's Nit).

## 4. Rows

Committed runner (this branch): `scripts/mk-ff-stale-docs-rows.sh red|green` (D rows grep/awk the
SPEC and BIP; C rows build a consumer crate from the guide AS PRINTED with `cargo +1.85.0 run`,
needs network; header says what it does not cover). Runs:

- at 9b0b365, `red` -> `ALL ROWS MATCH red`:
  `D1_docs_name_missing_helper (defect present: yes)`, `D2_docs_print_xpub_concat_recipe (yes)`,
  `D3_spec_step4_lacks_wallet_policy_id (yes)`, `D3_bip_step4_lacks_wallet_policy_id (yes)`,
  `C1_guide_as_printed rc=128 (tag mk-codec-v0.2.2 not found)` (stderr
  `fatal: couldn't find remote ref refs/tags/mk-codec-v0.2.2`; rc is 101 when cargo fetches
  without the git CLI, so the row accepts any nonzero rc with the tag in stderr),
  `C2_tag_fixed_only rc=101 (E0433 bitcoin)`, `C3_tag_and_bitcoin_fixed rc=101 (elided xpub panics)`.
- with section 3 applied in scratch (reverted, not committed), `green` -> `ALL ROWS MATCH green`:
  D1-D3 `defect present: no`; `C1_guide_as_printed rc=0 (2 mk1 line(s))`, the two lines being
  `mk1qpe9m4pqqsqsqqqqqpeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qjt8k8r9fgxmzmht`
  and `mk1qpe9m4pp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6n0yhwh6mr79jfqallmwff`.

Nextest rows (new file `crates/mk-cli/tests/bh1119_stale_docs.rs`, text in Appendix A; run by the
author: at 9b0b365 7 FAIL + 1 PASS, with section 3 applied 8 PASS). Command:
`cargo nextest run --locked -p mk-cli --test bh1119_stale_docs --no-fail-fast`.

| row (test fn) | at 9b0b365 | after section 3 |
|---|---|---|
| N1 `ff_i5_docs_name_no_missing_helper` | FAIL `design/SPEC_mk_v0_1.md names md-codec's removed compute_wallet_instance_id` | PASS |
| N2 `ff_i5_docs_print_no_xpub_concat_recipe` | FAIL `design/SPEC_mk_v0_1.md prints the retired 78-byte xpub recipe` | PASS |
| N3 `ff_i5_recovery_step4_names_wallet_policy_id` | FAIL `SPEC §5 step 4 does not name compute_wallet_policy_id:` | PASS |
| N4 `ff_i5_assembled_policy_id_reproduces_keyed_card` | PASS (fact row: 38bd7cec... for keyed and assembled, 949c6211... without fingerprints, 780f6305... swapped, c86829b5... for the retired recipe) | PASS |
| N5 `ff_i14_guide_pins_no_stale_tag` | FAIL `the guide still pins or names mk-codec v0.2.2` | PASS |
| N6 `ff_i14_guide_manifest_names_bitcoin` | FAIL ``guide manifest lacks `bitcoin = "0.32"`:`` | PASS |
| N7 `ff_i14_guide_example_xpub_encodes` | FAIL `guide example xpub "xpub6CatWdiZi..." does not parse: base58 encoding error` | PASS |
| N8 `ff_i5_step4_names_fingerprint_source` | FAIL `SPEC §5 step 4 does not state "origin_fingerprint":` (also at 6388394's Appendix B) | PASS |

Coverage map: I5 SPEC step 4 -> D2, D3, N2, N3, N4; I5 SPEC note -> D1, N1; I5 BIP L39/L47/L602
-> D1, D2, D3, N1, N2, N3; I5 BIP L37/L48 consistency -> review only (no machine row); I14 tag ->
C1, N5; I14 bitcoin -> C2, N6; I14 xpub -> C3, N7; I14 Stability paragraph -> N5; I5 step-4 fingerprint source/fallback -> N8.

## 5. Tests that change

- New: `crates/mk-cli/tests/bh1119_stale_docs.rs` (Appendix A, verbatim; it is `cargo +1.95.0 fmt` clean). It reads
  `bip/bip-mnemonic-key.mediawiki` and `docs/MK_CODEC_RUST_API.md`, which `ci/docs-only-paths.txt`
  treats as docs today, so the same commit adds two `-` rules to that file under a comment
  `# read by crates/mk-cli/tests/bh1119_stale_docs.rs`: `- ^bip/bip-mnemonic-key\.mediawiki$` and
  `- ^docs/MK_CODEC_RUST_API\.md$` (`ci/code-docs-check.sh` fails without them; the SPEC is
  already protected).
- No existing test changes. `spec_cites_resolve.rs` reads SPEC headings, and section 3 adds no
  heading and no `SPEC §` cite in source. The builder confirms with the full suite.
- No version bump and no CHANGELOG entry: no crate's behaviour changes (`version_is_honest.rs`
  reads `crates/mk-cli/CHANGELOG.md`, untouched).

## 6. Files the build changes (for build ordering)

`design/SPEC_mk_v0_1.md`, `bip/bip-mnemonic-key.mediawiki`, `docs/MK_CODEC_RUST_API.md`,
`crates/mk-cli/tests/bh1119_stale_docs.rs` (new), `ci/docs-only-paths.txt`. Already on this branch
with the document: `scripts/mk-ff-stale-docs-rows.sh`, `ci/code-docs-exempt.txt` (exempts the rows
script, which CI never runs), `SCRIPTS.md` (its row).

## 7. Toolkit manual lockstep

None for `docs/manual/src/40-cli-reference/` (no `mk` flag or printed text changes). The toolkit's
technical manual carries the same stale texts (glossary `61-glossary.md` L427-429 prints the retired
formula; `22-mk1-wire-format.md` L177 the pre-WalletPolicyId stub formula; `examples/Cargo.toml` L13
pins `mnemonic-key-mk-codec-v0.2.2`): filed as a toolkit FOLLOWUPS line in the author's report, not
fixed here.

## 8. FOLLOWUPS (listed in the author's report, not appended here)

Lines for `design/FOLLOWUPS.md` (direct-pushed after merge, per the coordinator rule) and one for the
toolkit are in the author's report.

## 9. Gate

`scripts/mk-ff-stale-docs-rows.sh green` -> `ALL ROWS MATCH green`, then the full CI set:
`bash ci/code-docs-check.sh`, `bash ci/docs-only.test.sh`, `bash ci/code-docs-check.test.sh`,
`cargo nextest run --locked --workspace --no-fail-fast`, `cargo test --locked --doc --workspace`,
`cargo +1.95.0 fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`.

Appendix B is authoritative for every byte of section 3 (inline code spans included); the
gate checks it applies: `git apply --check` of Appendix B at 9b0b365 succeeds.

## Appendix A: `crates/mk-cli/tests/bh1119_stale_docs.rs` (run as section 4 says)

```rust
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
```

## Appendix B: section 3 as a patch against 9b0b365 (the GREEN run's scratch edit)

```diff
diff --git i/bip/bip-mnemonic-key.mediawiki w/bip/bip-mnemonic-key.mediawiki
index f069d31..4c790a1 100644
--- i/bip/bip-mnemonic-key.mediawiki
+++ w/bip/bip-mnemonic-key.mediawiki
@@ -34,9 +34,9 @@ MK reuses BIP 93's BCH plumbing (same generator polynomials, same alphabet, same
 * '''xpub''': a BIP 32 extended public key. The full 78-byte serialized form is the binary representation of a Base58Check-decoded xpub. MK encodes a 73-byte ''compact'' form on the wire (see §"xpub encoding"); the full form is reconstructed at decode time.
 * '''Origin fingerprint''': the 4-byte master-key fingerprint identifying the seed from which the xpub was derived. Verbatim from BIP 380 origin notation <code>[fp/...]</code>. Optional in MK v0.1; presence is signaled by the bytecode header's fingerprint flag.
 * '''Origin path''': the BIP 32 derivation path from the master seed to the xpub. E.g., <code>m/48'/0'/0'/2'</code>.
-* '''Policy ID''' (a.k.a. '''WalletPolicyId'''): the 16-byte canonical-''expanded'' policy identity <code>md_codec::compute_wallet_policy_id(descriptor)</code> (md SPEC v0.13 §5.3), an encoder-divergence-free hash of an MD-encoded policy ''template'' — placeholders + shared path, not the concrete cosigner xpubs (a one-to-many relationship to actual wallets, not a wallet instance). It '''supersedes''' the earlier v0.8.0 "Policy ID" = <code>SHA-256(canonical_bytecode)[0..16]</code> (the encoding-sensitive md1 bytecode hash <code>Md1EncodingId</code>, itself renamed from "wallet ID" in [https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.8.0 md-codec v0.8.0]); the bytecode hash predated WalletPolicyId and is not stable under re-encoding the same logical wallet. See the naming note below (audit I1, 2026-06-10).
+* '''Policy ID''' (a.k.a. '''WalletPolicyId'''): the 16-byte canonical-''expanded'' policy identity <code>md_codec::compute_wallet_policy_id(descriptor)</code> (md SPEC v0.13 §5.3), an encoder-divergence-free hash of an MD-encoded policy: the template, the per-<code>@N</code> paths and, when the card carries them, the concrete fingerprints and xpubs (presence-significant). A template card's value is therefore one-to-many over actual wallets; a card with concrete xpubs identifies one wallet instance (see '''Wallet Instance ID'''). It '''supersedes''' the earlier v0.8.0 "Policy ID" = <code>SHA-256(canonical_bytecode)[0..16]</code> (the encoding-sensitive md1 bytecode hash <code>Md1EncodingId</code>, itself renamed from "wallet ID" in [https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.8.0 md-codec v0.8.0]); the bytecode hash predated WalletPolicyId and is not stable under re-encoding the same logical wallet. See the naming note below (audit I1, 2026-06-10).
 * '''Policy ID stub''': the top 4 bytes of a '''form-aware''' MD identity — the '''WalletPolicyId''' for a keyed wallet-policy card, the '''WalletDescriptorTemplateId''' for a keyless template card. Carried by MK cards as an indexing aid. See "Policy ID stubs" and "Linkage to MD".
-* '''Wallet Instance ID''': the derived 16-byte value <code>SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]</code> uniquely identifying a wallet instance — i.e., a template plus a specific cosigner xpub set. Defined in [https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.8.0 md-codec v0.8.0]'s BIP for cross-tool consistency. Recovery tools compute it on demand from policy + assembled xpubs.
+* '''Wallet Instance ID''': the 16-byte '''WalletPolicyId''' (<code>md_codec::compute_wallet_policy_id</code>) of the assembled descriptor — the policy template with each <code>@N</code> slot's concrete xpub filled in (and its origin fingerprint, from the MK or MD card as recovery step 4 states). Its preimage covers those keys, so it identifies a wallet instance — a template plus a specific cosigner xpub set; there is no separate instance hash (MD BIP §"Policy identifier"). Recovery tools compute it on demand from policy + assembled xpubs.
 * '''<code>chunk_set_id</code>''': a 20-bit per-encoding tag carried in the chunked-string header, derived from the payload (see below), used for reassembly mismatch detection. Each chunk of a multi-chunk MK card carries the same <code>chunk_set_id</code>; a decoder rejects assemblies with mismatched values. (md-codec v0.8.x originally called this field "wallet identifier"; [https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0 md-codec v0.9.0] renamed it to <code>chunk_set_id</code> across both the BIP draft and the reference implementation. Wire format unchanged.)
 
 ===Naming note (Policy ID vs Wallet Instance ID, and <code>chunk_set_id</code>)===
@@ -44,8 +44,8 @@ MK reuses BIP 93's BCH plumbing (same generator polynomials, same alphabet, same
 The 16-byte hash MD computes as <code>SHA-256(canonical_bytecode)[0..16]</code> covers the BIP 388 wallet-policy ''template'' only — placeholders and shared path, not the concrete cosigner xpubs. Two distinct wallets that share an identical template (same multisig shape, different cosigner sets) collide on this value. md-codec versions ≤ 0.7.x called this value "wallet ID" — a one-to-one name for a one-to-many relationship that confused readers. MK's design exposed the issue, and [https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.8.0 md-codec v0.8.0] ships the resolution:
 
 # Renamed the existing 16-byte value to '''Policy ID'''. The 16-byte value, the 4-byte stub, and the 12-word BIP 39 anchor encoding are byte-identical to v0.7.x; the rename is naming-only.
-# Defined a new derived '''Wallet Instance ID''' = <code>SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]</code> as a recovery-time computation, documented in the MD BIP but not carried by any physical card.
-# '''md-codec v0.13 (audit I1, 2026-06-10):''' the Policy ID — and therefore the 4-byte stub — is now the '''WalletPolicyId''' (<code>compute_wallet_policy_id</code>), a canonical-''expanded'' hash that is stable under re-encoding the same logical wallet (origin/use-site elision, override-vs-baseline path placement). This is a '''value''' change scoped to the linkage primitive: it '''supersedes''' the v0.8.0 bytecode-hash Policy ID, so a stub computed under the old <code>SHA-256(canonical_bytecode)</code> formula no longer matches. The byte-identity to v0.7.x noted above held only through the v0.8.0 naming-only rename; v0.13 changes the value. (The Wallet Instance ID is unaffected.)
+# Defined a new derived '''Wallet Instance ID''' as a recovery-time computation, documented in the MD BIP but not carried by any physical card. Its v0.8.0 formula is retired: the Wallet Instance ID is now the WalletPolicyId of the assembled descriptor (see the glossary).
+# '''md-codec v0.13 (audit I1, 2026-06-10):''' the Policy ID — and therefore the 4-byte stub — is now the '''WalletPolicyId''' (<code>compute_wallet_policy_id</code>), a canonical-''expanded'' hash that is stable under re-encoding the same logical wallet (origin/use-site elision, override-vs-baseline path placement). This is a '''value''' change scoped to the linkage primitive: it '''supersedes''' the v0.8.0 bytecode-hash Policy ID, so a stub computed under the old <code>SHA-256(canonical_bytecode)</code> formula no longer matches. The byte-identity to v0.7.x noted above held only through the v0.8.0 naming-only rename; v0.13 changes the value. (The Wallet Instance ID is the same WalletPolicyId, computed over the descriptor with its concrete xpubs; see the glossary.)
 # This BIP uses the renamed nomenclature throughout and treats Policy ID stubs as a template-level indexing filter, with Wallet Instance ID as the cryptographic per-instance check at descriptor-reassembly time.
 
 A separate naming-only rename — md-codec's chunked-string-header field "wallet identifier" → '''<code>chunk_set_id</code>''' — landed in [https://github.com/bg002h/descriptor-mnemonic/releases/tag/md-codec-v0.9.0 md-codec v0.9.0] (docs-and-symbols-only; wire format unchanged). MK uses the renamed name <code>chunk_set_id</code> throughout this BIP and in the reference implementation, in agreement with the md1 BIP draft as published.
@@ -599,7 +599,7 @@ Recovery flow (the canonical sequence wallet software SHOULD follow):
 ## Decode and extract its Policy ID stubs.
 ## Reject the card unless <code>policy_stub</code> matches one of its stubs.
 # For each accepted MK card, plug its xpub into the corresponding <code>@N</code> slot in the policy template (matched by origin path, by user assignment, or by the cosigner-name metadata that future MK versions may carry).
-# Reconstruct the full descriptor with concrete xpubs. Compute the Wallet Instance ID = <code>SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]</code> where <code>canonical_xpub_serialization</code> is the concatenation of each <code>@N</code>-placeholder's resolved 78-byte xpub serialization in placeholder-index order. Verify the Wallet Instance ID matches the wallet the user expected to recover (e.g., compared against a separately-anchored Wallet Instance ID phrase, or against a digital backup record).
+# Reconstruct the full descriptor with concrete xpubs. Compute the Wallet Instance ID = <code>md_codec::compute_wallet_policy_id(assembled_descriptor)</code>, the WalletPolicyId of the assembled descriptor (its preimage takes each xpub's 65-byte wire form, not the 78-byte BIP 32 serialization). The value is presence-significant on fingerprints and xpubs: an <code>@N</code>'s origin fingerprint is its MK card's <code>origin_fingerprint</code> or the MD policy card's fingerprint for that <code>@N</code>; if both cards carry one and they differ, reject the assembly; an <code>@N</code> that neither card gives one has none. Verify the Wallet Instance ID matches the wallet the user expected to recover (e.g., compared against a separately-anchored WalletPolicyId phrase, or against a digital backup record). If it does not match while some <code>@N</code> has no fingerprint, first take those fingerprints from another record of the same wallet (its descriptor backup, or the master fingerprint of that cosigner's seed) and recompute. If no record supplies them, the Wallet Instance ID cannot confirm the wallet: say so, and instead of the next step verify the first address derived from the assembled descriptor against an independently-recorded expected address (§"Privacy considerations") before any funds move. Fingerprints do not change addresses.
 # Reject if mismatch. Accept and proceed to address derivation if match.
 
 Step 2 is the indexing aid — fast filter, template-level. Step 4 is the cryptographic per-instance check. Both are required for safe recovery.
diff --git i/design/SPEC_mk_v0_1.md w/design/SPEC_mk_v0_1.md
index 1350765..c707f10 100644
--- i/design/SPEC_mk_v0_1.md
+++ w/design/SPEC_mk_v0_1.md
@@ -363,20 +363,16 @@ A key card with Policy ID stubs `[stub_1, ..., stub_N]` declares: "this xpub is
    a. Decode and extract its Policy ID stubs.
    b. Reject the card unless `policy_stub` matches one of its stubs.
 3. For each accepted key card, plug its xpub into the corresponding `@N` slot in the policy template (matched by origin path or by user assignment).
-4. Compute the **Wallet Instance ID** for the assembled wallet:
+4. Compute the **Wallet Instance ID** for the assembled wallet: the WalletPolicyId of the assembled descriptor, `md_codec::compute_wallet_policy_id(&assembled)` (md SPEC v0.13 §5.3). With concrete xpubs in its `@N` slots the identity's preimage covers them, so this one value is the per-instance check; there is no separate instance hash (md BIP §"Policy identifier"). The preimage takes each xpub's 65-byte wire form (chain code || compressed public key), not its 78-byte BIP 32 serialization.
 
-   ```text
-   wallet_instance_id = SHA-256(canonical_bytecode || canonical_xpub_serialization)[0..16]
-   ```
-
-   where `canonical_xpub_serialization` is the concatenation of each `@N`-resolved xpub's full 78-byte BIP 32 serialization, in placeholder-index order. Compare against the wallet identity the user expected to recover (e.g., a separately-anchored Wallet Instance ID, or against a digital backup record). Reject on mismatch.
+   The value is presence-significant on fingerprints as well as xpubs. An `@N`'s origin fingerprint in the assembled descriptor is its key card's `origin_fingerprint` or the policy card's fingerprint for that `@N`; if both cards carry one and they differ, reject the assembly. An `@N` that neither card gives one has none (a privacy-preserving key card carries none). Compare against the wallet identity the user expected to recover (e.g., a separately-anchored WalletPolicyId, its 12-word phrase from `WalletPolicyId::to_phrase`, or a digital backup record). Reject on mismatch, with one exception: when some `@N` has no fingerprint, the expected identity may have been computed with one, so first take those fingerprints from another record of the same wallet (its descriptor backup, or the master fingerprint of that cosigner's seed) and recompute. If no record supplies them, step 4 cannot confirm the wallet: say so, and verify the first address derived from the assembled descriptor against an independently-recorded expected address (§6) before any funds move. Fingerprints do not change addresses.
 5. Accept and proceed to address derivation.
 
 Step 2 is the indexing aid (fast filter, template-level). Step 4 is the cryptographic per-instance check. Both are required for safe recovery.
 
 Note the division of labour this implies: **step 2 is allowed to be form-sensitive because it is only an index.** A stub mismatch between the keyed and keyless forms of the same wallet costs the operator a re-stub, not their funds — step 4 recomputes the Wallet Instance ID from the assembled descriptor and is what actually decides whether the recovered wallet is the intended one. A stub that matched would still be rejected at step 4 if the assembly were wrong.
 
-**Implementation note:** the `compute_wallet_instance_id(canonical_bytecode, xpubs)` helper is provided by md-codec v0.8.0+. mk1 implementations integrating with md-codec ≥ 0.8.0 SHOULD use that helper directly rather than reimplementing the SHA-256 construction.
+**Implementation note:** step 4 is md-codec's `compute_wallet_policy_id` applied to the assembled descriptor. mk1 implementations SHOULD call it directly rather than reimplementing the preimage construction.
 
 ### 5.1 Authority precedence (mk1 ↔ md1 path information)
 
diff --git i/docs/MK_CODEC_RUST_API.md w/docs/MK_CODEC_RUST_API.md
index 4855e29..8f892e7 100644
--- i/docs/MK_CODEC_RUST_API.md
+++ w/docs/MK_CODEC_RUST_API.md
@@ -9,7 +9,8 @@ This chapter covers the public API surface for direct Rust use.
 
 ```toml
 [dependencies]
-mk-codec = { git = "https://github.com/bg002h/mnemonic-key", tag = "mk-codec-v0.2.2" }
+mk-codec = { git = "https://github.com/bg002h/mnemonic-key", tag = "mk-codec-v0.5.1" }
+bitcoin = "0.32"
 ```
 
 ## Public surface
@@ -45,7 +46,7 @@ it via `KeyCard::new(...)`:
 use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
 use mk_codec::{KeyCard, encode};
 
-let xpub: Xpub = "xpub6CatWdiZi...".parse().unwrap();
+let xpub: Xpub = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V".parse().unwrap();
 let card = KeyCard::new(
     vec![[0u8; 4]],                                   // policy_id_stubs (non-empty)
     Some(Fingerprint::from([0x73, 0xc5, 0xda, 0x0a])), // origin_fingerprint (None for privacy-preserving mode)
@@ -138,8 +139,9 @@ fixture regeneration; it is a maintainer tool, not a library module
 
 ## Stability
 
-`mk-codec` is at v0.2 (post-cycle close-out). v0.1 of the manual
-targets v0.2.2; semver-major bumps may break the API. Track the
+The manifest above pins the tag `mk-codec-v0.5.1`, the latest tagged
+release; the `KeyCard::validate()` section describes mk-codec 0.6.0.
+Semver-major bumps may break the API. Track the
 crate's CHANGELOG for breaking changes; minor bumps add features
 without breaking existing callers.
 
```
