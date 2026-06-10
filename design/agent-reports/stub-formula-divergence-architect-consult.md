# Architect direction-consult — stub-formula-divergence (audit I1/I2) — 2026-06-10

**Decision: WalletPolicyId (toolkit) is the canonical constellation contract; the mk SPEC §3.3 + mk-cli `derive_stub_from_md1` are STALE and are the fix targets. mk-codec-only cycle. No toolkit code change. No pin bump. No goldens move. Severity LOW.**

## The divergence (confirmed at source)
- toolkit `synthesize.rs:157-159` (+5 sites): `compute_wallet_policy_id(desc).as_bytes()[..4]` — md-codec `identity.rs:172-240`, hashes `canonical_template_tree_bytes || concat(per-@N records)` = canonical-EXPANDED, encoder-divergence-free (md SPEC v0.13 §5.3). ENCODING-STABLE.
- mk-cli `cmd/mod.rs:57-63 derive_stub_from_md1`: `SHA-256(encode_payload(desc))[..4]` = the md1 BYTECODE hash = `Md1EncodingId[..4]` (identity.rs:39-45). ENCODING-SENSITIVE. Matches mk SPEC §3.3 (line 186) + §5 step 1 (line 312), closure Q-2 locked 2026-04-29 (§9 line 385).

## Why WalletPolicyId is canonical
Deciding property = a card-linking stub MUST be stable under re-encoding the same logical wallet (origin/use-site elision, override-vs-baseline path placement, future encoder canonicalization). `encode_payload` (encode.rs:65-92) serializes path_decl + divergent_paths flag as-supplied → two byte-distinct md1 encodings of the same wallet → different bytecode hashes → different stubs. `compute_wallet_policy_id` absorbs exactly that divergence (identity.rs:106-113; pinned by `walletpolicyid_stable_across_origin_elision`/`_use_site_elision`, identity.rs:571-605). The bytecode hash fails the property by construction.

## Why the mk SPEC is stale (not a real disagreement)
`compute_wallet_policy_id` shipped in md-codec v0.13/phase-4 (`d8ceb90`), AFTER the mk Q-2 closure (2026-04-29). The mk SPEC could only cite the bytecode hash because that was the only md identity primitive available then. The toolkit, written later, correctly adopted the newer encoding-stable primitive.

## Severity LOW — unreachable from the shipped bundle path
Toolkit mints via `KeyCard::new` with the `compute_wallet_policy_id` stub (synthesize.rs:157/189/242/422/594); `self_check_bundle` recomputes the same (bundle.rs:2078-2096) → internally consistent, self-check passes. Divergence bites ONLY manual `mk verify --from-md1` (verify.rs:107 → spurious ContentMismatch) / `mk encode --from-md1` (encode.rs:69 → self-check-rejected card) against a toolkit md1. No shipped bundle / fielded card is wrong.

## Pin verdict — NO bump
`compute_wallet_policy_id` is publicly exported at the `md-codec-v0.34.0` tag (mk-cli's pin) and byte-identical to 0.35.0: the only `md-codec/src` delta between the two tags is `chunk.rs` (additive BCH hardening), outside the hash-input path (identity/canonicalize/encode/bitstream/varint/tree/origin_path/use_site_path/tag/tlv all unchanged). Bump to 0.35 = optional hygiene, decoupled.

## Fix scope (one mk-codec cycle)
Spec: SPEC_mk_v0_1.md §3.3 (:186), §5 step 1 (:312) rewrite to WalletPolicyId; §9 Q-2 (:385) annotate superseded; BIP draft lockstep. Impl: derive_stub_from_md1 → compute_wallet_policy_id (the 2 callers unchanged). Test: round_trip.rs:44-78 de-tautologized to a hardcoded golden (identity.rs:546-553 pattern). No goldens move (test_vectors stubs are arbitrary literals). FOLLOWUPS: promote+resolve both slugs w/ corrected rationale; toolkit cross-repo note. MINOR bump + CHANGELOG (old --from-md1 stubs no longer match).

## Label caveat for the SPEC author
No literal "§3.5.1" heading exists in SPEC_mk_v0_1.md — the bytecode mandate is §3.3 (:186) + §5 step 1 (:312). The phantom §3.5.1 cite appears only in mod.rs:56 + round_trip.rs:45 doc-comments; repoint both to §3.3.
