# Post-impl whole-diff R0 — T4 mk external oracle — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Repo `mnemonic-key` @ `582f007`, uncommitted T4 diff (test-only). Verified by execution; tree left byte-clean.

## 1. Green + gates
`cargo test -p mk-codec` **190/0** (new `xpub_compact_external_oracle` 1); `cargo test -p mk-cli` **108/0** (new `cli_address_bip_vectors` 2; `cli_slip132` 9 = 8+1). Post-revert combined **298/0**. clippy `--workspace --all-targets -D warnings` exit 0 (re-linted after `touch`ing new files to defeat fingerprint cache); `cargo +1.95.0 fmt --all -- --check` clean.

## 2. Pins vs authoritative BIP text (LOAD-BEARING) — all 9 match char-for-char with correct path attribution
BIP-84 zpub (bip-0084:75, `m/84'/0'/0'`), /0/0 `bc1qcr8…` (:80), /0/1 (:85), /1/0 (:90); BIP-86 xpub (bip-0086:92), /0/0 `bc1p5cyxnu…` (:100), /0/1 (:108), /1/0 (:116); BIP-32 tv1 master (bip-0032:222, seed `000102…0f`). CLI-path drive confirmed: `mk encode --xpub <zpub> --origin-path m/84'/0'/0'` chunks into **2 mk1 lines** (both vectors); `encode_card` collects ALL stdout lines (no `lines().next()`); `run_address` forwards every chunk; `mk address <both> --count 2 --chain both` renders all 6 published addresses at correct positions.

## 3. T4-a RED-proof
`84'`↔`49'` arm swap (`derive_support.rs:108-109`) → `bip84_…` FAILED (`rendered=[]`; mutant renders P2SH `3GtVZ…` — the wrong-wallet mode); `bip86_…` unaffected (green, correct). Honesty verified: the same mutation ALSO REDs 5 of cli_address's 15 tests — accurately framed (wrong-at-birth provenance, not "not caught today"). Reverted; `src/` diff empty.

## 4. T4-b independence + RED-proof
Imports: `std`, `bitcoin::bip32::Xpub` (input side only), `mk_codec::bytecode::XpubCompact` (subject), `sha2`. Verify side = hand-rolled base58 decoder + double-SHA256 checksum. **No `bs58` in Cargo.lock** (grep 0); `sha2` already a dev-dep → zero manifest/lock delta. Asserts COMPACT-FORM fields (`compact.{version,parent_fingerprint,chain_code,public_key}`), not reconstructed Xpub. RED: `version`↔`parent_fingerprint` swap (`xpub_compact.rs:47-49`, both `[u8;4]`) → FAILED at version assert (`[0,0,0,0]` vs `[4,136,178,30]`). Also REDs the 2 in-crate round-trips (eager version validation) — the pre-declared marginal-coverage caveat. Reverted; `src/` empty.

## 5. T4-c + FOLLOWUP judgment
Anchors on the PUBLISHED zpub + byte-swaps THAT string's own base58check payload to `0x0488B21E` (`to_slip132` canonical direction), NOT a re-versioned corpus xpub. Asserts `NOTE_ZPUB` on stderr + decoded-card `xpub` field equality. **Recommend flip `mk-slip0132-byte-parity-test-self-referential` → resolved** with residual: only the zpub arm (1 of 8 SLIP-0132 entries) is published-vector-anchored (ypub/Ypub/Zpub/testnet remain self-referential; `XPUB_MAINNET_V` still test-local). Entry was Low/hardening; the named option-(a) remedy is met → keep-open overweights the residual.

## 6. NO-BUMP (final)
`git diff crates/mk-codec/src/ crates/mk-cli/src/` EMPTY; `git diff Cargo.lock` EMPTY; all Cargo.toml unchanged; no clap-surface change.

## Findings
Critical: none. Important: none.
Minor 1: T4-a assertions are set-membership over rendered tokens, not chain/index-bound → a hypothetical receive↔change chain swap passes (outside T4-a's purpose-arm/HRP threat model; matches existing `cli_address.rs .contains()` idiom; positions manually confirmed correct today). A positional variant would close it cheaply — optional.
Minor 2: T4-c anchors only the zpub arm → carry into the FOLLOWUP resolution note.
Minor 3 (observation): both RED mutations also caught by pre-existing tests — the diff's value is spec-external provenance + locality (stated honestly in spec + test headers).

## VERDICT: GREEN (0C/0I)
298/298 green, all 9 pins verified vs authoritative BIP text, both REDs reproduced+reverted, NO-BUMP holds.

---
**SHIP (opus, 2026-07-10):** GREEN. Flipping `mk-slip0132-byte-parity-test-self-referential` → RESOLVED (with the zpub-arm-only residual note) in the shipping commit. Minor-1 (positional assertion) left as-is (matches existing idiom, outside threat model) — noted for a future revisit. Shipping T4 mk direct-FF NO-BUMP.