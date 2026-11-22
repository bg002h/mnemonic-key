# FIX F-C: KeyCard validation (mk-codec), bughunt-1119 mk I8, I9, I10

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Track: **full, class B (funds-adjacent)**: a card records
key metadata a wallet imports. Base: `origin/main` 8a738ab353d7ee9910cdf4376d762050f7d4c6e0 (mk-cli 0.17.0,
mk-codec 0.5.1). Ruling: T46 (architect-bughunt-1119-questions.md:94). Source rows: grade-mk.md:36-38, :53, :68;
triage-mk-t2 (I8, I9, findings 1-2), triage-mk-t3 (I10); consultant report
`design/agent-reports/bug-hunt-2026-11-19.md`.

Everything executable here was run: the RED rows on the base (section 6), and a scratch prototype of sections 3-5
with the rows (GREEN, section 8). The prototype is not part of this commit; its diff is kept outside the repo at
`/scratch/cache/wt-mkc-keycard-validation-gen/prototype-designC.patch` for reviewers. Its production code is not
rustfmt-clean, so a builder writes the production code from this document, not by applying that patch.

## 0. Ruling, scope and PROPOSED decisions

T46, verbatim: "mint refuses; `mk verify` reports the card invalid with the exit it already uses for an invalid
card; `mk decode` reads it with a warning ... The FIX picks the variant name." Grader constraint (grade-mk.md:53,
from triage t2 finding 1): the depth-0 parent-fingerprint check is unconditional; only the self-fingerprint check
may exempt the toolkit's zero-chain-code WIF key.

The three defects at base (RED evidence in section 6):

- **I8.** `mk encode` mints, and `mk verify` passes, a depth-0 key whose parent fingerprint is not `00000000`.
  BIP 32 lists that serialization as invalid (test vector 5, "zero depth with non-zero parent fingerprint").
- **I9.** `mk_codec::encode` mints a depth-0 card whose origin fingerprint is not the key's own fingerprint, and the
  false origin round-trips unchanged. mk-cli already refuses this at mint (encode.rs:249-292, exit 64); library
  callers do not.
- **I10.** `mk_codec::encode` accepts a raw `ChildNumber::Normal { index }` or `Hardened { index }` with
  `index >= 2^31` and writes it with bit 31 set, so the card decodes to a different path (`Normal{0x80000001}`
  decodes as `1'`).

PROPOSED decisions (architect confirms). Each one is used as written below:

- P-a **Variant names and texts:** `Error::MasterParentFingerprintNonzero` and `Error::MasterFingerprintMismatch`
  (section 2). I10 reuses `Error::InvalidPathComponent` with a new message. The architect added
  `OriginParentFingerprintMismatch` (R4, T7).
- P-b **The zero-chain-code exemption is narrow.** A depth-0 key with an all-zero chain code may carry origin
  fingerprint `00000000` (what the toolkit writes for a WIF; mnemonic-toolkit cmd/bundle.rs:799-821) or its own
  fingerprint. Any other origin fingerprint is refused. The broader reading ("an all-zero chain code skips the
  check") would also accept `deadbeef` on such a key, a false origin that nothing produces today.
- P-c **`mk encode` refuses I8 with exit 64** (UsageError), in the same pre-mint loop that already refuses a
  depth-0 fingerprint mismatch with 64 (encode.rs:261-292), and following B2 ("refuse contradictory ... key
  records exit 64"). The codec refusal is the library backstop.
- P-d **The checks live in a new public method `KeyCard::validate(&self) -> Result<()>`.** Mint entry points call
  it. `KeyCard::canonical_payload_bytes` stays permissive about the two depth-0 rules, because mk-cli derives the
  chunk-set id of a decoded card by re-encoding it (cmd/mod.rs:107-110). If those rules were enforced there, every
  legacy invalid card would silently lose its chunk-set-id comparison: measured on the prototype, `mk inspect
  <LEGACY_I8>` dropped its `chunk_set_id:        ca522 (stamped)` line. Row C9 guards against this.
- P-e **mk-codec becomes 0.6.0 (SemVer-minor), and the corpus family token rolls to `mk-codec 0.6`.** V19 must be
  regenerated, which changes the corpus SHA, and the BIP's own rule (bip:714) says a patch bump keeps the SHA.
- P-f **Holder-visible texts T1-T7 (T5 and T7 RULED by the architect)** (section 2) and the manual texts (section 5.6).

Architect rulings (2026-11-22): R4 (depth-1) and the reader warnings in inspect/address/derive are in scope.

## 1. Rules (normative for this FIX)

Definitions: "depth-0 card" means `xpub.depth == 0`, which after rule S4 also means an empty `origin_path`.
"Key fingerprint" means `xpub.fingerprint()` (the first 4 bytes of HASH160 of the compressed public key).
"All-zero chain code" means `xpub.chain_code.to_bytes() == [0u8; 32]`.

Structural checks, in this order. They are today's checks plus R1:

- S1 `policy_id_stubs` empty, or more than 255 of them: `InvalidPolicyIdStubCount` (unchanged, encode.rs:25-30).
- S2 more than 10 path components: `PathTooDeep` (unchanged, encode.rs:42-60).
- **R1 (I10, new).** Every `origin_path` component, in either form, must have `index < 2^31`. The first offending
  component (counting from 1) fails with `InvalidPathComponent(T3)`.
- S4 xpub depth/child vs path: `XpubOriginPathMismatch` (unchanged, encode.rs:62-79).

Key-origin rules (new), checked after S4: R2 and R3 on a depth-0 card, R4 on a depth-1 card:

- **R2 (I8).** `xpub.parent_fingerprint` must be `00000000`, or the card fails with `MasterParentFingerprintNonzero`.
  The rule is unconditional: an all-zero chain code does not exempt a key from it.
- **R3 (I9).** When `origin_fingerprint` is `Some(fp)`, `fp` must equal the key fingerprint. Exception: if the chain
  code is all zero, `fp == 00000000` is also accepted (P-b). Otherwise the card fails with
  `MasterFingerprintMismatch { origin_fingerprint: fp, key_fingerprint }`. When `origin_fingerprint` is `None` there
  is no check.
- **R4 (depth-1, architect Q1).** On a depth-1 card, when `origin_fingerprint` is `Some(fp)`, `fp` must equal
  `xpub.parent_fingerprint` (the parent of a depth-1 key is the master), or the card fails with
  `OriginParentFingerprintMismatch { origin_fingerprint: fp, parent_fingerprint }`. No exemption. When it is
  `None`, there is no check.

Where each rule applies:

| Entry point | S1, S2, R1, S4 | R2, R3, R4 |
|---|---|---|
| `KeyCard::validate` (new, pub) | yes | yes |
| `bytecode::encode_bytecode`, `encode`, `encode_with_chunk_set_id` (mint) | yes | yes |
| `KeyCard::canonical_payload_bytes` (identity of a card, valid or legacy) | yes | **no** (P-d) |
| `decode`, `decode_bytecode`, `KeyCard::from_canonical_payload_bytes` | unchanged: R2/R3 violations still decode (T46); R1 cannot arise from decode | no |

Order inside `validate` and mint: S1, S2, R1, S4, R2, R3, R4. The first failure is returned.

## 2. Exact texts and exit codes

| id | where | exact text (PROPOSED, P-f) |
|---|---|---|
| T1 | `Display` of `MasterParentFingerprintNonzero { parent_fingerprint }` | `depth-0 key has parent fingerprint {parent_fingerprint}; BIP 32 requires 00000000 for a master key` |
| T2 | `Display` of `MasterFingerprintMismatch { origin_fingerprint, key_fingerprint }` | `origin fingerprint {origin_fingerprint} is not the depth-0 key's own fingerprint {key_fingerprint}` |
| T3 | payload of `InvalidPathComponent` for R1; `{n}` is the 1-based position, `{idx}` is the decimal `index` field | `component {n} index {idx} is out of the BIP 32 range 0..2^31` (Display: `invalid path component: ` + T3) |
| T4 | mk-cli `UsageError` for R2 at mint | `xpub {xpub} is at depth 0 but carries parent fingerprint {pfp}; BIP 32 requires 00000000 for a master key, so the xpub is not a valid BIP 32 key and no card was minted.` |
| T5 (RULED) | mk-cli `mk decode`, `mk inspect`, `mk address`, `mk derive` stderr line, `{e}` = T1, T2 or T7 | `warning: this card is invalid ({e}); it is read as written, mk verify reports it invalid, and mk encode refuses to mint it` |
| T6 | mk-cli `mk verify` | the existing codec-error path: text mode prints `error: {e}` on stderr; `--json` prints the error envelope on stdout with `kind` = variant name, `message` = T1, T2 or T7, `exit_code` 1, `details` null |
| T7 (RULED) | `Display` of `OriginParentFingerprintMismatch { origin_fingerprint, parent_fingerprint }` | `origin fingerprint {origin_fingerprint} is not the depth-1 key's parent fingerprint {parent_fingerprint}; at depth 1 the parent is the master` |

Fingerprints render as 8 lowercase hex digits, which is bitcoin 0.32's `Fingerprint` `Display`; mk-cli's
`fmt_fingerprint` gives the same.

Holder-visible outcomes. "Base" was measured on 8a738ab; "after" was measured on the prototype. The constants are
defined in section 6.

| command | base | after |
|---|---|---|
| `mk encode --xpub BAD_TPUB --origin-path m --privacy-preserving --policy-id-stub deadbeef` | exit 0, mints the LEGACY_I8 strings | exit 64, stdout empty, stderr `error: ` + T4 (pfp `01010101`) |
| same with `--origin-fingerprint 751e76e8` instead of `--privacy-preserving` | exit 0, mints | exit 64, same T4 (R2 is checked before the existing fingerprint check) |
| `mk encode --xpub ROOT --origin-path m --origin-fingerprint deadbeef ...` | exit 64 (existing depth-0 check) | unchanged |
| `mk verify LEGACY_I8...` | exit 0, `OK: mk1 string(s) decode cleanly ...` | exit 1, stdout empty, stderr `error: ` + T1 |
| `mk verify --json LEGACY_I9...` | exit 0, `"ok":true` | exit 1, stdout `{"error":{"details":null,"exit_code":1,"kind":"MasterFingerprintMismatch","message":"<T2>"},"schema_version":1}` |
| `mk decode LEGACY_I8...` (text or `--json`) | exit 0, card printed, no warning | exit 0, stdout byte-identical to base, stderr begins with T5 (T1), then the existing watch-only note |
| `mk verify LEGACY_D1...` | exit 0 OK | exit 1, stderr `error: ` + T7 |
| `mk decode LEGACY_D1...` | no warning | T5 (T7) |
| `mk inspect|address|derive LEGACY_I8...` | no warning | stdout unchanged, T5 (T1) on stderr, exit 0 |
| `mk verify SENTINEL_ZERO...`, `mk decode SENTINEL_ZERO...` | exit 0, no warning | unchanged |
| `mk inspect LEGACY_I8...` | prints `chunk_set_id:        ca522 (stamped)` | unchanged (guard C9) |

The `--json` key order on that verify row is serde_json's sorted-map order, which is how today's envelope is
printed. Row C4 asserts the fields, not the bytes.

## 3. mk-codec changes (crates/mk-codec)

1. `src/error.rs`: add three variants after `XpubOriginPathMismatch` (:181-194), each with a doc comment:
   - `MasterParentFingerprintNonzero { parent_fingerprint: Fingerprint }`, `#[error]` = T1. Doc: "A depth-0 key whose
     parent fingerprint is not 00000000 (BIP 32 serializes a master key with a zero parent fingerprint). Refused at
     mint and reported by `KeyCard::validate`. A decoder still reads such a card (cards minted before mk-codec
     0.6.0 may carry it)."
   - `MasterFingerprintMismatch { origin_fingerprint: Fingerprint, key_fingerprint: Fingerprint }`, `#[error]` = T2.
     Doc: "A depth-0 card whose origin fingerprint is not the key's own. The one accepted exception is an all-zero
     chain code with origin fingerprint 00000000 (a key built from a raw WIF). Refused at mint, reported by
     `KeyCard::validate`, and still decoded."
   - `OriginParentFingerprintMismatch { origin_fingerprint: Fingerprint, parent_fingerprint: Fingerprint }`,
     `#[error]` = T7. Doc: "A depth-1 card whose origin fingerprint is not the key's parent fingerprint (at depth 1
     the parent is the master). Refused at mint, reported by `KeyCard::validate`, still decoded."
   - `Fingerprint` is `bitcoin::bip32::Fingerprint`, imported the way `ChildNumber` is.
   - In the unit test `parameterized_variants_render` (:208-250), add two cases: parent `01010101` renders as T1
     with `01010101`, and origin `deadbeef`/key `751e76e8` renders as T2. Add a parameterized render case: origin
     `deadbeef`, parent `751e76e8`.
2. `src/bytecode/encode.rs`: split today's body into three private functions:
   - `check_structure(card)`: S1, S2, then R1, then S4. R1 is a loop over `card.origin_path` with an `enumerate()`
     position. It reads `index` from either variant and returns `InvalidPathComponent(format!(T3))`. Keep every
     existing comment.
   - `check_key_rules(card)`: R2, R3, R4.
   - `serialize(card)`: today's :81-98, unchanged bytes.

   Wire them up as follows:
   - `pub fn encode_bytecode` = `check_structure` + `check_key_rules` + `serialize`.
   - `pub(crate) fn validate_card` = `check_structure` + `check_key_rules`.
   - `pub(crate) fn encode_canonical_payload` = `check_structure` + `serialize`.

   The `encode_bytecode` doc comment gains one sentence: "Refuses a card that fails `KeyCard::validate`."
3. `src/key_card.rs`:
   - Add `pub fn validate(&self) -> Result<()>` (calls `validate_card`) with this doc: "Check this card against every
     rule `encode` enforces (`design/SPEC_mk_v0_1.md` §4, encoder-side invariants), including the depth-0 key rules
     a decoder does not reject. A card returned by `decode` can fail it: cards minted before mk-codec 0.6.0 may
     carry an invalid depth-0 key. Readers report that (`mk verify` exits 1, `mk decode` warns) rather than refuse
     to read the card."
   - `canonical_payload_bytes` calls `encode_canonical_payload` instead of `encode_bytecode`. Its doc gains: "Unlike
     `encode`, this does not apply the depth-0 key rules of [`KeyCard::validate`], so a decoded legacy card still
     has a canonical identity."
   - In the `new` doc's "Field invariants enforced at encode time" list (:68-83), add two bullets:
     - "each `origin_path` component's index MUST be below 2^31 in either form; otherwise
       [`crate::Error::InvalidPathComponent`]"
     - "a depth-0 key MUST have parent fingerprint 00000000 and, when `origin_fingerprint` is present, carry its
       own fingerprint (an all-zero chain code may carry 00000000 instead); see [`KeyCard::validate`]"
     - "a depth-1 key with a present `origin_fingerprint` MUST carry its parent fingerprint there"
4. `src/bin/gen_mk_vectors.rs`:
   - In `synthetic_xpub` (:1008), the parent fingerprint becomes `00000000` when `depth == 0`, and stays
     `[0x10, 0x20, 0x30, seed_byte]` otherwise.
   - V19 `origin_fingerprint` (:386) becomes `Some([0xF1, 0x50, 0xC1, 0x3C])`. That is the key fingerprint of the
     seed-0x13 key, measured.
   - V19's description (:380-384) gains a final sentence after `child_number=Normal{0}.`: ` A valid BIP 32 master
     (mk-codec v0.6.0+): parent fingerprint 00000000 and origin fingerprint equal to the key's own (f150c13c).`
     (one leading space, inside the same string literal).
5. `src/consts.rs:54`: `GENERATOR_FAMILY` becomes `"mk-codec 0.6"`. Append to the doc comment above it: "Rolled to
   `mk-codec 0.6` with the regeneration that made V19 a valid BIP 32 master."
6. Regenerate: `cargo run --locked --bin gen_mk_vectors --features gen-vectors` from the workspace root. The
   measured diff, which a builder must reproduce exactly:
   - In both JSONs, the `family_token` line becomes `"mk-codec 0.6"`.
   - In `v0.1.json` V19 only, the new values are:
     - `description` per item 4.
     - `canonical_bytecode_hex` `0401d00d0000f150c13cfe000488b21e00000000b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9b9031d16453b3ab3132acb0a5bc16cc49690d819a585267a15cd5a064e2a0ad40599`.
     - `strings` `mk1qpxj4upqqsqaqrgqqrc4psfulcqqfz9jrcqqqqqqhxumnwdehxumnwdehxumnwdehxumnwdehxumnwdehxumnwdehxusxefszgr0gwdjt787`,
       `mk1qpxj4uppr5ty2we6kvfj4jc2t0qke3ykjrvpnfv9yeaptn26qe8z5zk5qkvl2jthyuqfgfu5rs3648l`.
     - `origin_fingerprint` `f150c13c`.
     - `xpub` `xpub661MyMwAqRbcGPgULiHq6QtVJ7tA327NAGwK6sXax65t9WQr7eFkv38RwD2h4CE4BoTN9Cm5oFej6kvZ4RCiexBvjNu5CdDSTiTgaA7daSs`.
   - The other 40 vectors and all 21 csid_ext rows are byte-identical. The new SHA-256s are `v0.1.json`
     `7591561f61863f91a65f00f8b004d958ee03eb4a2272f593fb48c54c2e5c6caa` and `csid_ext_v0.1.json`
     `3d3d12efabaf00c48401c33469813641a6f942e0b23124b5f6075e529b3d0500`.
   - Any other diff is a build error to stop on.
7. `Cargo.toml` `version = "0.6.0"`; mk-cli's dependency line becomes `mk-codec = { path = "../mk-codec", version =
   "0.6.0" }`. Refresh `Cargo.lock` for the workspace members only (`cargo update --workspace --offline`), and check
   that the lock diff is the mk-codec version line alone.
8. `CHANGELOG.md` (crates/mk-codec), under `## [Unreleased]`:
   - A bold lead line: `**SemVer-minor: \`encode\` refuses cards it used to mint (BIP 32 invalid depth-0 keys,
     out-of-range path indices). Wire layout unchanged; decoding unchanged.**`
   - `### Added`:
     - `KeyCard::validate`
     - `Error::MasterParentFingerprintNonzero`
     - `Error::MasterFingerprintMismatch`
     - `Error::OriginParentFingerprintMismatch`
   - `### Changed`:
     - R1-R3 in one sentence each (texts T1-T3).
     - `canonical_payload_bytes` keeps accepting a decoded legacy card.
     - `GENERATOR_FAMILY` is `mk-codec 0.6`; V19 is regenerated as a valid master; both corpus SHAs are re-pinned.
   - `### Compatibility`:
     - "Decoders: unaffected; a legacy card that breaks R2 or R3 still decodes, and `KeyCard::validate` reports it."
     - "`mnemonic-toolkit`: its WIF cards (all-zero chain code, origin fingerprint 00000000) stay valid."

## 4. mk-cli changes (crates/mk-cli)

1. `src/error.rs:202`: after the `XpubOriginPathMismatch` arm, add the arms `MasterParentFingerprintNonzero { .. }
   => "MasterParentFingerprintNonzero"` and `MasterFingerprintMismatch { .. } => "MasterFingerprintMismatch"`, and
   the arm `OriginParentFingerprintMismatch { .. } => "OriginParentFingerprintMismatch"`.
   `exit_code` is unchanged: `Codec(_)` gives 1.
2. `src/cmd/encode.rs:261`:
   - Where: inside the `for (fp, path, xpub) in &cards` loop, as its first statement, before `let Some(declared) =
     fp else { continue };`.
   - Condition: `xpub.depth == 0`, the path has 0 components, and the parent fingerprint is not `00000000`.
   - Action: return `CliError::UsageError(T4)`, with `{xpub}` the xpub's `Display` and `{pfp}` from
     `fmt_fingerprint`.
   - Effect: it applies to `--xpub` and to every `--keys`/`--in` record, in input order. It runs before any card is
     minted or written, so nothing is minted.
3. `src/cmd/verify.rs:75`: immediately after `warn_corrections_applied(...)`, add `card.validate()?;`. An invalid
   card therefore exits 1 even when `--xpub`/`--origin-*`/`--policy-id-stub`/`--from-md1` are given (invalid takes
   precedence over a mismatch). The chunk-set-id and corrections warnings still print first.
4. Add `pub fn warn_if_invalid(card: &mk_codec::KeyCard)` in `src/cmd/mod.rs`, which `eprintln!`s T5 on `Err(e)`
   from `card.validate()`. Call it in `src/cmd/decode.rs:49` (immediately after `warn_corrections_applied(...)`), in
   `src/cmd/inspect.rs` right after `warn_chunk_set_id_mismatch(csid, &card);` (:53), and in `src/cmd/address.rs`
   (:84) and `src/cmd/derive.rs` (:54) right after `warn_chunk_set_id_mismatch(chunk_set_id_comparison(&strings,
   &card), &card);`. The warning prints in text and `--json` mode alike. Stdout and exit codes are unchanged.
5. `src/cmd/mod.rs:108`: `derived_chunk_set_id` uses `card.canonical_payload_bytes().ok()?` in place of
   `mk_codec::bytecode::encode_bytecode(card).ok()?`. Its doc comment gains: "`canonical_payload_bytes`, not
   `encode_bytecode`: a decoded legacy card that fails the depth-0 key rules still has a canonical re-encode, and
   its chunk-set-id comparison must not silently vanish."
6. `CHANGELOG.md` (crates/mk-cli), under `## [Unreleased]`, `### Changed`:
   - `mk encode` refuses a depth-0 xpub whose parent fingerprint is not 00000000 (exit 64).
   - `mk verify` reports a card with an invalid depth-0 key as invalid (exit 1, kinds `MasterParentFingerprintNonzero`
     / `MasterFingerprintMismatch` / `OriginParentFingerprintMismatch`).
   - `mk decode`, `inspect`, `address` and `derive` still read such a card and warn on stderr.
   - Requires mk-codec 0.6.0.

   mk-cli's own version is the release step's to set, not this FIX's.

No flag changes, so the GUI schema mirror is not touched.

## 5. Spec, BIP, API doc and manual texts (exact)

### 5.1 `design/SPEC_mk_v0_1.md`

- After :274 ("Each component encodes a u32 ..."), insert a new paragraph:
  "Encoders MUST reject a component whose BIP 32 index (the 31-bit value, without the hardened marker) is 2³¹ or
  more, in either the normal or the hardened form, with `Error::InvalidPathComponent` (`component <n> index <i> is
  out of the BIP 32 range 0..2^31`, n counted from 1). Such a value cannot be written as one u32 without colliding
  with the hardened marker: a normal index 2³¹+1 would be written as the hardened component 1' and decode as a
  different path (mk-codec 0.6.0+)."
- After :338 (the `XpubOriginPathMismatch` encoder-side invariant), insert a new paragraph:
  "Encoder-side invariant (depth-0 key validity; mk-codec 0.6.0+): for a depth-0 card (empty `origin_path`),
  encoders MUST reject an `xpub.parent_fingerprint` other than `00000000` (BIP 32 serializes a master key with a
  zero parent fingerprint) with `Error::MasterParentFingerprintNonzero`. When `origin_fingerprint` is present,
  encoders MUST also reject one that differs from the key's own fingerprint (the first 4 bytes of HASH160 of
  `xpub.public_key`) with `Error::MasterFingerprintMismatch`. The second rule has one exception: a key with an
  all-zero chain code (the depth-0 form a toolkit builds from a raw WIF, which has no master) may carry origin
  fingerprint `00000000`. The first rule has none. Unlike the invariants above, these are detectable at decode, but
  a decoder MUST NOT reject them: cards minted before mk-codec 0.6.0 may carry them and must stay readable. A
  reader reports them instead. In the reference implementation `KeyCard::validate` returns the error, `mk verify`
  exits 1 with it, and `mk decode` prints the card with a warning. For a depth-1 card (one `origin_path` component),
  when `origin_fingerprint` is present, encoders MUST reject one that differs from `xpub.parent_fingerprint`
  (`Error::OriginParentFingerprintMismatch`): the parent of a depth-1 key is the master."

### 5.2 `bip/bip-mnemonic-key.mediawiki`

- After :507 (the LEB128 sizing paragraph), insert a new paragraph:
  "An encoder MUST reject a path component whose BIP 32 index (the 31-bit value, without the hardened marker) is
  2<sup>31</sup> or more, in either the normal or the hardened form (<code>Error::InvalidPathComponent</code>). Such
  a value cannot be written as one u32 without colliding with the hardened marker: a normal index
  2<sup>31</sup>+1 would be written as the hardened component <code>1'</code> and decode as a different path."
- After :538 (the compact-73 encoder-side invariant), insert a new paragraph:
  "'''Encoder-side invariant (depth-0 and depth-1 key origin).''' For a depth-0 card (empty <code>origin_path</code>), an
  encoder MUST reject an <code>xpub.parent_fingerprint</code> other than <code>00000000</code>
  (<code>Error::MasterParentFingerprintNonzero</code>): BIP 32 serializes a master key with a zero parent
  fingerprint. When <code>origin_fingerprint</code> is present, the encoder MUST also reject one that differs from
  the key's own fingerprint, the first 4 bytes of HASH160 of <code>xpub.public_key</code>
  (<code>Error::MasterFingerprintMismatch</code>). The second rule has one exception: a key with an all-zero chain
  code (the depth-0 form built from a raw WIF, which has no master) may carry origin fingerprint
  <code>00000000</code>. The first rule has none. These conditions are visible to a decoder, but a decoder MUST NOT
  reject them, because cards minted by earlier encoders may carry them. A reader SHOULD report such a card as
  invalid and MAY still display its fields. For a depth-1 card (one <code>origin_path</code> component), when
  <code>origin_fingerprint</code> is present, encoders MUST reject one that differs from
  <code>xpub.parent_fingerprint</code> (<code>Error::OriginParentFingerprintMismatch</code>): the parent of a depth-1
  key is the master."
- At :712, append to the paragraph: " The depth-0 card is a valid BIP 32 master (parent fingerprint
  <code>00000000</code>, origin fingerprint equal to the key's own), as regenerated under the <code>mk-codec
  0.6</code> token."

### 5.3 `docs/MK_CODEC_RUST_API.md`

After the "Decoding an mk1 card" section (ends :86), insert a new section:

    ## Validating a decoded card

    `decode` reads every card a conformant encoder could have written, including cards minted before
    mk-codec 0.6.0 whose depth-0 or depth-1 key origin breaks these rules (a nonzero parent fingerprint, or an origin
    fingerprint that is not the key's own). `KeyCard::validate()` applies every rule `encode` enforces and
    returns `Error::MasterParentFingerprintNonzero`, `Error::MasterFingerprintMismatch` or `Error::OriginParentFingerprintMismatch` for such a card.
    Call it before trusting a decoded card's origin; `mk verify` does, and exits 1 on the error.

### 5.4 `design/FOLLOWUPS.md`

There is no edit in the PR. The line that closes item (a) of `mk1-bip-presubmission-nits` is listed in the author
report and is direct-pushed after merge.

### 5.5 Comments and the coverage mirror

`tests/error_coverage.rs` gets the following, each in its existing place:

- `ErrorVariantName` gains `MasterParentFingerprintNonzero` and `MasterFingerprintMismatch` (after :77), and the
  third variant `OriginParentFingerprintMismatch`.
- The Display prefixes are `"depth-0 key has parent fingerprint"` and `"origin fingerprint"` (after :108); the third
  variant has prefix `"origin fingerprint"` (shared with T2; both are exempt).
- `is_exempt` gains the variants (all three) with the rationale "encoder-side invariant (T46): `decode` still reads a
  legacy card that breaks it, so no negative vector can pin it; `KeyCard::validate` and `mk verify` report it."

### 5.6 Toolkit manual lockstep (`mnemonic-toolkit` `docs/manual/src/40-cli-reference/44-mk-cli.md`, origin/master)

- `mk encode`, after the paragraph "At least one of `--policy-id-stub` or `--from-md1` is required ..." (:52-53),
  add: "A depth-0 xpub (`--origin-path m`, or a key record with no path) must be a valid BIP 32 master: its parent
  fingerprint must be `00000000`, and a declared origin fingerprint must be the key's own. At depth 1 a declared
  origin fingerprint must be the xpub's parent fingerprint. `mk encode` refuses any of these with exit 64 and mints
  nothing."
- `mk decode` Output, after "`origin_fingerprint` is `null` for privacy-preserving cards." (:166), add: "A card
  that breaks one of these rules (minted before mk-codec 0.6.0) still reads: `mk decode`, `mk inspect`, `mk address`
  and `mk derive` print it as read and add a `warning: this card is invalid (...)` line on stderr."
- `mk verify` (:443-446): replace "Exits 0 with `OK: mk1 string(s) decode cleanly ...` on success, exits 4 with a
  `ContentMismatch` error envelope when an expected-* flag disagrees with the decoded value." with "Exits 0 with
  `OK: mk1 string(s) decode cleanly ...` on success. Exits 1 when the card breaks one of these rules (kind
  `MasterParentFingerprintNonzero`, `MasterFingerprintMismatch` or `OriginParentFingerprintMismatch`), before any expected-* comparison. Exits 4 with a
  `ContentMismatch` error envelope when an expected-* flag disagrees with the decoded value."

This lands in the toolkit's next manual commit after the mk PR merges, following the established mk -> toolkit
order. A mk-cli flag does not change.

## 6. RED rows

Constants (testnet only; no seed involved):

- `ROOT` = `tpubD6NzVbkrYhZ4WMCcYXEgDrM1AFATSGmrFyMnA2Dgx1Y551VueaE3iemHhQW6MaZfCm67ek9JkvZboSbAjuvGk7foa2PinzkrKyRnxyLHWQ2`:
  depth 0, parent `00000000`, chain code `01`x32, public key = the secp256k1 generator, key fingerprint `751e76e8`.
  The fields were decoded independently with a base58check script.
- `BAD_TPUB` = `tpubD6PRKLEwwo1MaZJYCnVjQHmEig2xBZVzkFQrrQFz5VA3GLdKmtYiB6tM7y1uTRthqM4dxRRK528DnR3qXiSopvZj238hq6e7Ek7WjNDscen`:
  ROOT with parent fingerprint `01010101` (the testnet form of BIP 32 test vector 5's defect).
- `LEGACY_I8` (minted by base `mk encode --xpub BAD_TPUB --origin-path m --privacy-preserving --policy-id-stub
  deadbeef`), `LEGACY_I9` (minted by base `mk_codec::encode`: ROOT, origin `deadbeef`, stub `deadbeef`) and
  `SENTINEL_ZERO` (ROOT with an all-zero chain code, origin `00000000`, stub `deadbeef`) are given verbatim in the
  row files below.
- `LEGACY_D1` (minted by base `mk_codec::encode`: ROOT/0, origin `deadbeef`, stub `deadbeef`):
  `mk1qpkg6wpqqsqaatd7al02m0h0lcqsqpp4sl8h28nkaq2rw39uktvh6uenflm3rxqw9jllmjktc059yn75stew08xve94t2q3eze2hn68kjszz`
  and `mk1qpkg6wppqgx00lerqaq42pmr6f79l0erqcejj2g8umwpudmkpn4h093lwzzehv35azeqll55lad4xzcy4` (origin_fp
  `deadbeef`, parent_fp `751e76e8`).
- `D1_TPUB` = `tpubD984EkAGpvBEL17UyUznoEtTRFfVULe1D6QeTARexMtG9Xvjsqo3cV2EuNcnQmAoodvTVoiLC1W8k4tF8zmrPwM8EuEK2KyvuhnYhYurri4`.

The RED commands, run on base 8a738ab with the three files installed. Logs are at
`/scratch/cache/wt-mkc-keycard-validation-gen/final-red-*.log`.

    cargo nextest run --locked --build-jobs 6 -j 6 -p mk-codec --test keycard_validation --no-fail-fast
    cargo nextest run --locked --build-jobs 6 -j 6 -p mk-codec --test keycard_validate --no-fail-fast
    cargo nextest run --locked --build-jobs 6 -j 6 -p mk-cli --test keycard_validation --no-fail-fast
    cargo nextest run --locked --build-jobs 6 -j 6 -p mk-codec --test keycard_depth1 --no-fail-fast
    cargo nextest run --locked --build-jobs 6 -j 6 -p mk-cli --test keycard_readers --no-fail-fast

| row | test (file) | rule | base result (observed) | GREEN |
|---|---|---|---|---|
| K1 | kv_i8_depth0_nonzero_parent_fp_refused (codec keycard_validation) | R2 | FAIL `encode accepted the card: ["mk1qpeffzpqq...` | pass |
| K2 | kv_i8_zero_chain_code_does_not_exempt_parent_fp | R2, no exemption | FAIL `encode accepted the card: ["mk1qpxprgpqq...` | pass |
| K3 | kv_i9_depth0_false_origin_fp_refused | R3 | FAIL `encode accepted the card: ["mk1qp32rnpqq...` | pass |
| K4 | kv_i9_sentinel_foreign_fp_refused | R3, narrow exemption (P-b) | FAIL `encode accepted the card: ["mk1qpljfmpqq...` | pass |
| K5 | kv_sentinel_zero_fp_encodes | R3 exemption | pass (control) | pass |
| K6 | kv_sentinel_own_fp_encodes | R3 | pass (control) | pass |
| K7 | kv_master_own_fp_encodes | R2/R3 valid master, with fp and None | pass (control) | pass |
| K8 | kv_i10_normal_index_ge_2_31_refused | R1 | FAIL `encode accepted the card: ["mk1qpg4y2pqq...` | pass |
| K9 | kv_i10_hardened_index_ge_2_31_refused | R1 | FAIL `encode accepted the card: ["mk1qpg4y2pqq...` | pass |
| K10 | kv_i10_canonical_payload_bytes_refused | R1 on `canonical_payload_bytes` | FAIL `canonical_payload_bytes accepted the card: [0, 1, 222, ...` | pass |
| K11 | kv_i10_index_2_31_minus_1_round_trips | R1 boundary | pass (control) | pass |
| V1 | kv_validate_reports_legacy_master_cards (codec keycard_validate) | T46 decode-still-reads + `validate`, T1, T2 | build FAIL `error[E0599]: no method named \`validate\` found for struct \`KeyCard\`` | pass |
| V2 | kv_validate_ok_on_every_corpus_vector | every positive corpus vector validates (catches a missed V19 regeneration) | build FAIL (same E0599) | pass |
| C1 | kv_cli_i8_encode_refuses_invalid_master (cli keycard_validation) | P-c, T4 | FAIL `assertion \`left == right\` failed: stdout: mk1qpeffzpqq...` left `Some(0)` right `Some(64)` | pass |
| C2 | kv_cli_i8_encode_refuses_invalid_master_with_own_fp | P-c ordering | FAIL `stdout: mk1qpt4gqpqq...`, `Some(0)` vs `Some(64)` | pass |
| C3 | kv_cli_verify_reports_invalid_master_i8 | T46 verify, T6 text | FAIL `stdout: OK: mk1 string(s) decode cleanly ...`, `Some(0)` vs `Some(1)` | pass |
| C4 | kv_cli_verify_json_reports_invalid_master_i9 | T46 verify, T6 json, kind | FAIL `stdout: {"chunk_set_id":{...},"chunks":2,"ok":true,...}`, `Some(0)` vs `Some(1)` | pass |
| C5 | kv_cli_decode_warns_on_invalid_master | T46 decode, T5 | FAIL `stderr: note: stdout is watch-only — public keys only, cannot spend` | pass |
| C6 | kv_cli_decode_json_warns_on_invalid_master | T5 in `--json` mode, stdout unchanged | FAIL (same stderr, no warning) | pass |
| C7 | kv_cli_verify_sentinel_zero_fp_ok | P-b keeps toolkit WIF cards valid | pass (guard) | pass |
| C8 | kv_cli_decode_sentinel_no_warning | P-b | pass (guard) | pass |
| C9 | kv_cli_inspect_keeps_chunk_set_id_on_legacy_card | P-d | pass (guard); FAILS under the naive design (master checks in `canonical_payload_bytes`), measured | pass |
| K12 | kv_depth1_false_origin_fp_refused (codec keycard_depth1) | R4, T7 | build FAIL (file, `error[E0599]: no method named \`validate\`` from V3); probed alone, FAIL `encode` accepts the card (mints LEGACY_D1) | pass |
| K13 | kv_depth1_true_origin_fp_encodes | R4 (control) | build FAIL (same file); control | pass |
| K14 | kv_depth1_no_origin_fp_encodes | R4 (control) | build FAIL (same file); control | pass |
| V3 | kv_validate_reports_legacy_depth1_card | legacy depth-1 card still decodes, `validate` reports T7 | build FAIL `error[E0599]: no method named \`validate\`` | pass |
| C10 | kv_cli_verify_reports_invalid_depth1 (cli keycard_readers) | R4, T6, T7 | FAIL `Some(0)` vs `Some(1)` | pass |
| C11 | kv_cli_verify_json_kind_depth1 | R4, kind `OriginParentFingerprintMismatch` | FAIL `Some(0)` vs `Some(1)` | pass |
| C12 | kv_cli_decode_warns_depth1 | T5 (T7) | FAIL (no warning) | pass |
| C13 | kv_cli_inspect_warns_i8 | Q2, T5 (T1) | FAIL (no warning) | pass |
| C14 | kv_cli_address_warns_i8 | Q2, T5 (T1) | FAIL (no warning) | pass |
| C15 | kv_cli_derive_warns_i8 | Q2, T5 (T1) | FAIL (no warning) | pass |
| C16 | kv_cli_derive_json_warns_depth1_stdout_json | Q2, T5 (T7), stdout JSON | FAIL (no warning) | pass |
| C17 | kv_cli_readers_sentinel_no_warning | P-b guard | pass (guard) | pass |
| C18 | kv_cli_encode_depth1_false_fp_refused_guard | existing CLI check, exit 64 | pass (guard) | pass |

Observed summaries on base: codec `11 tests run: 4 passed, 7 failed`; validate build failure (3× E0599); cli `9
tests run: 3 passed, 6 failed`.
Architect rows, run again by the folder on base: `keycard_depth1` build FAIL (E0599, `validate`); `keycard_readers` `9 tests run: 2 passed, 7 failed` (the 2 passes are guards C17, C18); the changed `keycard_validation` (C5, C6, C8 with the RULED T5) `9 tests run: 3 passed, 6 failed`.

### 6.1 `crates/mk-codec/tests/keycard_validation.rs` (new; sha256 of the file as run: a2e174abc5e3c34becabc22e435f2419f1ef4ccd62d6a9a13f5b0c12a0859b18)

```rust
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
```

### 6.2 `crates/mk-codec/tests/keycard_validate.rs` (new; sha256 1025eb8b04cd806c7d750bc3927788bd1c7da1dd85c5dc3115ee677adc030b5a)

```rust
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
```

### 6.3 `crates/mk-cli/tests/keycard_validation.rs` (new; sha256 5d48a80f5340e195e8d5ec4866f102cb9bc515dc510b8548b2c72282d3fff55c, after the RULED T5 text in C5, C6, C8)

```rust
//! FIX_mkc_keycard_validation rows C1-C9 (bughunt-1119 mk I8, I9; ruling T46):
//! mk encode refuses, mk verify reports invalid (exit 1), mk decode warns.
use assert_cmd::cargo::CommandCargoExt;
use std::process::{Command, Output};

const BAD_TPUB: &str = "tpubD6PRKLEwwo1MaZJYCnVjQHmEig2xBZVzkFQrrQFz5VA3GLdKmtYiB6tM7y1uTRthqM4dxRRK528DnR3qXiSopvZj238hq6e7Ek7WjNDscen";
const LEGACY_I8: [&str; 2] = [
    "mk1qpeffzpqqqqaatd7allqqpp4sl8szqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqnehen8u5yagl2mw0p43h3j",
    "mk1qpeffzppl8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqte3jjj9mtsugd0x98xyvntv",
];
const LEGACY_I9: [&str; 2] = [
    "mk1qp32rnpqqsqaatd7al02m0h0lcqqgdv8euqqqqqqqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqsyr6hhd3h4f5yu7sp",
    "mk1qp32rnpp0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vg4peejgreyhlqwm83g90",
];
const SENTINEL_ZERO: [&str; 2] = [
    "mk1qp2enrpqqsqaatd7auqqqqqqlcqqgdv8euqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqy8h8vagvc68yhng5",
    "mk1qp2enrpp0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7v9ve3hu53hjkuceu46e8y",
];
const I8_ENCODE_ERR: &str = "error: xpub tpubD6PRKLEwwo1MaZJYCnVjQHmEig2xBZVzkFQrrQFz5VA3GLdKmtYiB6tM7y1uTRthqM4dxRRK528DnR3qXiSopvZj238hq6e7Ek7WjNDscen is at depth 0 but carries parent fingerprint 01010101; BIP 32 requires 00000000 for a master key, so the xpub is not a valid BIP 32 key and no card was minted.\n";
const I8_DISPLAY: &str =
    "depth-0 key has parent fingerprint 01010101; BIP 32 requires 00000000 for a master key";
const I9_DISPLAY: &str =
    "origin fingerprint deadbeef is not the depth-0 key's own fingerprint 751e76e8";

fn mk(args: &[&str]) -> Output {
    Command::cargo_bin("mk")
        .unwrap()
        .args(args)
        .output()
        .unwrap()
}
fn s(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}

#[test]
fn kv_cli_i8_encode_refuses_invalid_master() {
    let o = mk(&[
        "encode",
        "--xpub",
        BAD_TPUB,
        "--origin-path",
        "m",
        "--privacy-preserving",
        "--policy-id-stub",
        "deadbeef",
    ]);
    assert_eq!(o.status.code(), Some(64), "stdout: {}", s(&o.stdout));
    assert_eq!(s(&o.stdout), "");
    assert_eq!(s(&o.stderr), I8_ENCODE_ERR);
}

#[test]
fn kv_cli_i8_encode_refuses_invalid_master_with_own_fp() {
    let o = mk(&[
        "encode",
        "--xpub",
        BAD_TPUB,
        "--origin-path",
        "m",
        "--origin-fingerprint",
        "751e76e8",
        "--policy-id-stub",
        "deadbeef",
    ]);
    assert_eq!(o.status.code(), Some(64), "stdout: {}", s(&o.stdout));
    assert_eq!(s(&o.stderr), I8_ENCODE_ERR);
}

#[test]
fn kv_cli_verify_reports_invalid_master_i8() {
    let o = mk(&["verify", LEGACY_I8[0], LEGACY_I8[1]]);
    assert_eq!(o.status.code(), Some(1), "stdout: {}", s(&o.stdout));
    assert_eq!(s(&o.stdout), "");
    assert_eq!(s(&o.stderr), format!("error: {I8_DISPLAY}\n"));
}

#[test]
fn kv_cli_verify_json_reports_invalid_master_i9() {
    let o = mk(&["verify", "--json", LEGACY_I9[0], LEGACY_I9[1]]);
    assert_eq!(o.status.code(), Some(1), "stdout: {}", s(&o.stdout));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["error"]["kind"], "MasterFingerprintMismatch");
    assert_eq!(v["error"]["message"], I9_DISPLAY);
    assert_eq!(v["error"]["exit_code"], 1);
}

#[test]
fn kv_cli_decode_warns_on_invalid_master() {
    let o = mk(&["decode", LEGACY_I8[0], LEGACY_I8[1]]);
    assert_eq!(o.status.code(), Some(0));
    assert!(s(&o.stdout).starts_with(&format!("xpub:                {BAD_TPUB}\n")));
    let want = format!(
        "warning: this card is invalid ({I8_DISPLAY}); it is read as written, mk verify reports it invalid, and mk encode refuses to mint it\n"
    );
    assert!(s(&o.stderr).starts_with(&want), "stderr: {}", s(&o.stderr));
}

#[test]
fn kv_cli_verify_sentinel_zero_fp_ok() {
    let o = mk(&["verify", SENTINEL_ZERO[0], SENTINEL_ZERO[1]]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", s(&o.stderr));
}

#[test]
fn kv_cli_decode_sentinel_no_warning() {
    let o = mk(&["decode", SENTINEL_ZERO[0], SENTINEL_ZERO[1]]);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        !s(&o.stderr).contains("this card is invalid"),
        "stderr: {}",
        s(&o.stderr)
    );
}

#[test]
fn kv_cli_decode_json_warns_on_invalid_master() {
    let o = mk(&["decode", "--json", LEGACY_I9[0], LEGACY_I9[1]]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["origin_fingerprint"], "deadbeef");
    assert_eq!(v["schema_version"], 1);
    let want = format!(
        "warning: this card is invalid ({I9_DISPLAY}); it is read as written, mk verify reports it invalid, and mk encode refuses to mint it\n"
    );
    assert!(s(&o.stderr).starts_with(&want), "stderr: {}", s(&o.stderr));
}

#[test]
fn kv_cli_inspect_keeps_chunk_set_id_on_legacy_card() {
    let o = mk(&["inspect", LEGACY_I8[0], LEGACY_I8[1]]);
    assert_eq!(o.status.code(), Some(0));
    assert!(
        s(&o.stdout).contains("\nchunk_set_id:        ca522 (stamped)\n"),
        "stdout: {}",
        s(&o.stdout)
    );
}
```

### 6.4 `crates/mk-codec/tests/keycard_depth1.rs` (new, architect rows K12-K14, V3; sha256 1cb0c8b276f47471fafcc1df4aa97bb01330db2ee8476138a786973fe198acc6)

```rust
//! Architect rows (bh #17 Q1): depth-1 origin fingerprint vs xpub.parent_fingerprint.
//! K12 refused at mint; K13/K14 controls; V3 decode still reads a legacy card, validate reports it.
use std::str::FromStr;

use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use bitcoin::secp256k1::Secp256k1;
use mk_codec::{KeyCard, decode, encode};

const ROOT: &str = "tpubD6NzVbkrYhZ4WMCcYXEgDrM1AFATSGmrFyMnA2Dgx1Y551VueaE3iemHhQW6MaZfCm67ek9JkvZboSbAjuvGk7foa2PinzkrKyRnxyLHWQ2";
const STUB: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];
const LEGACY_D1: [&str; 2] = [
    "mk1qpkg6wpqqsqaatd7al02m0h0lcqsqpp4sl8h28nkaq2rw39uktvh6uenflm3rxqw9jllmjktc059yn75stew08xve94t2q3eze2hn68kjszz",
    "mk1qpkg6wppqgx00lerqaq42pmr6f79l0erqcejj2g8umwpudmkpn4h093lwzzehv35azeqll55lad4xzcy4",
];
const D1_DISPLAY: &str = "origin fingerprint deadbeef is not the depth-1 key's parent fingerprint 751e76e8; at depth 1 the parent is the master";

fn child() -> Xpub {
    let secp = Secp256k1::verification_only();
    Xpub::from_str(ROOT)
        .unwrap()
        .ckd_pub(&secp, ChildNumber::Normal { index: 0 })
        .unwrap()
}
fn card(fp: Option<[u8; 4]>) -> KeyCard {
    KeyCard::new(
        vec![STUB],
        fp.map(Fingerprint::from),
        DerivationPath::from_str("m/0").unwrap(),
        child(),
    )
}
fn round_trips(c: &KeyCard) {
    let s = encode(c).unwrap();
    let refs: Vec<&str> = s.iter().map(String::as_str).collect();
    assert_eq!(&decode(&refs).unwrap(), c);
}

#[test]
fn kv_depth1_false_origin_fp_refused() {
    let e = encode(&card(Some(STUB))).expect_err("encode accepted the card");
    assert_eq!(e.to_string(), D1_DISPLAY);
}

#[test]
fn kv_depth1_true_origin_fp_encodes() {
    round_trips(&card(Some([0x75, 0x1e, 0x76, 0xe8])));
}

#[test]
fn kv_depth1_no_origin_fp_encodes() {
    round_trips(&card(None));
}

#[test]
fn kv_validate_reports_legacy_depth1_card() {
    let c = decode(&LEGACY_D1).expect("a legacy depth-1 card still decodes");
    assert_eq!(c, card(Some(STUB)));
    assert_eq!(c.validate().unwrap_err().to_string(), D1_DISPLAY);
}
```

### 6.5 `crates/mk-cli/tests/keycard_readers.rs` (new, architect rows C10-C18; sha256 142c36dcec68f15df36e1455e5cf68cd2b4ed4302d55f97ac9f6fa2fed88d980)

```rust
//! Architect rows (bh #17 Q1, Q2): mk verify / decode on a legacy depth-1 card;
//! the T5 warning in inspect, address and derive; mk encode depth-1 guard.
use assert_cmd::cargo::CommandCargoExt;
use std::process::{Command, Output};

const LEGACY_D1: [&str; 2] = [
    "mk1qpkg6wpqqsqaatd7al02m0h0lcqsqpp4sl8h28nkaq2rw39uktvh6uenflm3rxqw9jllmjktc059yn75stew08xve94t2q3eze2hn68kjszz",
    "mk1qpkg6wppqgx00lerqaq42pmr6f79l0erqcejj2g8umwpudmkpn4h093lwzzehv35azeqll55lad4xzcy4",
];
const D1_TPUB: &str = "tpubD984EkAGpvBEL17UyUznoEtTRFfVULe1D6QeTARexMtG9Xvjsqo3cV2EuNcnQmAoodvTVoiLC1W8k4tF8zmrPwM8EuEK2KyvuhnYhYurri4";
const LEGACY_I8: [&str; 2] = [
    "mk1qpeffzpqqqqaatd7allqqpp4sl8szqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqnehen8u5yagl2mw0p43h3j",
    "mk1qpeffzppl8wthtz45p3ftn58pvrs9xlumvkuu2xet8egzkcklqte3jjj9mtsugd0x98xyvntv",
];
const SENTINEL_ZERO: [&str; 2] = [
    "mk1qp2enrpqqsqaatd7auqqqqqqlcqqgdv8euqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqy8h8vagvc68yhng5",
    "mk1qp2enrpp0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7v9ve3hu53hjkuceu46e8y",
];
const D1_DISPLAY: &str = "origin fingerprint deadbeef is not the depth-1 key's parent fingerprint 751e76e8; at depth 1 the parent is the master";
const I8_DISPLAY: &str =
    "depth-0 key has parent fingerprint 01010101; BIP 32 requires 00000000 for a master key";

fn mk(args: &[&str]) -> Output {
    Command::cargo_bin("mk")
        .unwrap()
        .args(args)
        .output()
        .unwrap()
}
fn s(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).unwrap()
}
fn t5(e: &str) -> String {
    format!(
        "warning: this card is invalid ({e}); it is read as written, mk verify reports it invalid, and mk encode refuses to mint it\n"
    )
}
fn warns(verb: &[&str], card: [&str; 2], e: &str) {
    let mut a = verb.to_vec();
    a.extend_from_slice(&card);
    let o = mk(&a);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", s(&o.stderr));
    assert!(s(&o.stderr).contains(&t5(e)), "stderr: {}", s(&o.stderr));
}

#[test]
fn kv_cli_verify_reports_invalid_depth1() {
    let o = mk(&["verify", LEGACY_D1[0], LEGACY_D1[1]]);
    assert_eq!(o.status.code(), Some(1), "stdout: {}", s(&o.stdout));
    assert_eq!(s(&o.stdout), "");
    assert_eq!(s(&o.stderr), format!("error: {D1_DISPLAY}\n"));
}

#[test]
fn kv_cli_verify_json_kind_depth1() {
    let o = mk(&["verify", "--json", LEGACY_D1[0], LEGACY_D1[1]]);
    assert_eq!(o.status.code(), Some(1), "stdout: {}", s(&o.stdout));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["error"]["kind"], "OriginParentFingerprintMismatch");
}

#[test]
fn kv_cli_decode_warns_depth1() {
    warns(&["decode"], LEGACY_D1, D1_DISPLAY);
}

#[test]
fn kv_cli_inspect_warns_i8() {
    warns(&["inspect"], LEGACY_I8, I8_DISPLAY);
}

#[test]
fn kv_cli_address_warns_i8() {
    warns(
        &["address", "--count", "1", "--address-type", "p2wpkh"],
        LEGACY_I8,
        I8_DISPLAY,
    );
}

#[test]
fn kv_cli_derive_warns_i8() {
    warns(&["derive", "--index", "0"], LEGACY_I8, I8_DISPLAY);
}

#[test]
fn kv_cli_derive_json_warns_depth1_stdout_json() {
    let o = mk(&[
        "derive",
        "--json",
        "--index",
        "0",
        LEGACY_D1[0],
        LEGACY_D1[1],
    ]);
    assert_eq!(o.status.code(), Some(0));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["parent_xpub"], D1_TPUB);
    assert!(
        s(&o.stderr).contains(&t5(D1_DISPLAY)),
        "stderr: {}",
        s(&o.stderr)
    );
}

#[test]
fn kv_cli_readers_sentinel_no_warning() {
    for verb in [
        &["inspect"][..],
        &["address", "--count", "1", "--address-type", "p2wpkh"][..],
        &["derive", "--index", "0"][..],
    ] {
        let mut a = verb.to_vec();
        a.extend_from_slice(&SENTINEL_ZERO);
        let o = mk(&a);
        assert_eq!(
            o.status.code(),
            Some(0),
            "{verb:?} stderr: {}",
            s(&o.stderr)
        );
        assert!(
            !s(&o.stderr).contains("this card is invalid"),
            "{verb:?} stderr: {}",
            s(&o.stderr)
        );
    }
}

#[test]
fn kv_cli_encode_depth1_false_fp_refused_guard() {
    let o = mk(&[
        "encode",
        "--xpub",
        D1_TPUB,
        "--origin-path",
        "m/0",
        "--origin-fingerprint",
        "deadbeef",
        "--policy-id-stub",
        "deadbeef",
    ]);
    assert_eq!(o.status.code(), Some(64), "stdout: {}", s(&o.stdout));
}
```

## 7. Tests that change

On the prototype, without the repairs below, the fix reds exactly these 6 existing tests (measured; the triage t2
census agrees):

- `bytecode::encode::tests::accepts_consistent_depth0_card` and `depth0_card_round_trips` (R2):
  `src/bytecode/test_helpers.rs:35` `synthetic_xpub` uses parent fingerprint `00000000` when the path is empty, and
  `[0xAA, 0xBB, 0xCC, 0xDD]` otherwise. Both tests then pass unedited. They use `origin_fingerprint: None`.
- `proptest_roundtrip::keycard_roundtrip`, `proptest_roundtrip::decode_never_panics_on_corrupted_encoding` and
  `bch_adversarial::t2c_five_to_eight_error_corruption_never_returns_original` (R2, R3), repaired in
  `tests/common/mod.rs`:
  - `:98`: the parent fingerprint is `00000000` when `depth == 0`, otherwise the strategy's `pfp`.
  - `:117`: the card's `fp` becomes `match xpub.depth { 0 => fp.map(|_| xpub.fingerprint()), 1 =>
    fp.map(|_| xpub.parent_fingerprint), _ => fp }` (measured: needed, see architect B1), so every generated depth-0
    card is a valid master and every generated depth-1 card has a true origin. Doc comments on
    `xpub_strategy`/`keycard_strategy` say so. Builder note: a red run writes `tests/*.proptest-regressions`. Do
    not commit them.
- `vectors::every_vector_round_trips` (V19), fixed by the regeneration in section 3 item 6. Re-pins:
  - `tests/vectors.rs:41` `V0_1_SHA256` becomes `7591561f61863f91a65f00f8b004d958ee03eb4a2272f593fb48c54c2e5c6caa`.
  - `tests/vectors.rs:133` `"mk-codec 0.5"` becomes `"mk-codec 0.6"`.
  - `tests/csid_ext_vectors.rs:46` `CSID_EXT_SHA256` becomes
    `3d3d12efabaf00c48401c33469813641a6f942e0b23124b5f6075e529b3d0500`.

Edited without being red:

- `src/error.rs` `parameterized_variants_render`: two cases (section 3 item 1).
- `tests/error_coverage.rs`: mirror entries and exemptions (section 5.5).

No mk-cli test changes. All 216 existing mk-cli tests passed on the prototype unedited.

## 8. Gate

GREEN measured on the prototype with these rows: `cargo nextest run --locked --build-jobs 6 -j 6 --workspace
--no-fail-fast` gave `459 tests run: 459 passed`. The base suite has 424 tests and the rows add 35. Doc tests,
clippy `-D warnings`, the local vectors-roundtrip (41 files equal the corpus) and `ci/code-docs-check.sh` were all
green. Gate line for the builder (run from the worktree root, output captured once to a file and grepped):

    export PATH=$HOME/.cargo/bin:$PATH && cargo --version | grep -q '^cargo 1\.85\.' && cargo nextest run --locked --build-jobs 6 -j 6 --workspace --no-fail-fast && cargo test --locked --workspace --doc && cargo clippy --locked --workspace --all-targets -- -D warnings && cargo +1.95.0 fmt --check --all && bash ci/docs-only.test.sh && sha256sum --check --strict ci/shared-ci.sha256 && bash ci/code-docs-check.test.sh && bash ci/code-docs-check.sh && (cd design && sha256sum -c display-grouping-vectors.tsv.sha256) && cargo build --locked -p mk-cli && mkdir -p ../mkc-kv-vectors-out && ./target/debug/mk vectors --out ../mkc-kv-vectors-out && test "$(jq -S '.vectors | sort_by(.name)' crates/mk-codec/src/test_vectors/v0.1.json)" = "$(jq -s -S 'sort_by(.name)' ../mkc-kv-vectors-out/*.json)"

Expect `459 tests run: 459 passed`. What the gate does not cover:

- The Windows, macOS, musl, FreeBSD and release-mode CI legs, which a builder gets as green CI on the PR head.
- The toolkit manual sentences.
- Any downstream pin bump.

## 9. Build order

1. RED commit: the three row files (section 6) only. Re-run the three RED commands and confirm the section 6
   results.
2. GREEN commit:
   - sections 3 and 4 (codec, CLI, generator, consts, Cargo/lock, CHANGELOGs);
   - the regenerated corpus, with the diff checked against section 3 item 6;
   - the section 7 repairs and re-pins;
   - the section 5 texts (SPEC, BIP, API doc, error_coverage).
   Run the gate.
3. After merge: the toolkit manual commit (section 5.6), and the FOLLOWUPS lines in the author report
   (direct-push).

## 10. Not covered

- `KeyCard::new` stays permissive (`#[non_exhaustive]` constructor, documented as such).

## 11. Open questions for the architect (not decided here; the FIX is complete without them)

Q1 RULED: R4, in this FIX. Q2 RULED: inspect/address/derive warn (architect-bh-tk15-mkc17.md B1, B2).
