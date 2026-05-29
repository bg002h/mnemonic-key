# SPEC — mk-codec test-hardening (themes 1/2/3 from the constellation survey)

**Status:** DRAFT — pre-R0
**Repo / branch:** `mnemonic-key`, default branch **`main`**, crate `crates/mk-codec` v0.3.1
**Source ground-truth SHA:** `d9d2ed9` (origin/main at authoring; all line numbers grep-verified against it)
**Recon:** `mnemonic-toolkit/cycle-prep-recon-codec-test-hardening-themes-1-2-3.md`
**Design provenance:** constellation coverage survey → cycle-prep strict-gate recon → brainstorm → opus architect design review (YELLOW → GREEN after 4 must-fix folds, all re-verified against source).

---

## §1 Problem / context

mk-codec encodes xpubs as `mk1` (codex32-family BCH; an xpub spans multiple chunked `mk1` strings). It is the **leanest** constellation codec — ~179 test fns across only 3 integration files (`tests/{round_trip,error_coverage,vectors}.rs`) over ~8.9k LOC — yet it carries a GF(1024) BCH layer, LEB128 path encoding, variable chunking, and a hand-rolled syndrome decoder. The constellation survey found three coverage gaps where complexity most exceeds testing:

1. **No property/fuzz testing** of the `KeyCard` encode↔decode bijection (mk-codec `Cargo.toml` has no `proptest`/`quickcheck`/`fuzz` dep; all round-trip coverage is hand-fixtures).
2. **BCH correction is exercised only at low error counts and never through the public `decode()` path** at 3–4 errors; the miscorrection regime (5+ errors) and the cross-chunk-hash guard that defends it are under-tested.
3. **No indel reject-contract test** — the toolkit's shipped `repair --max-indel` uses `mk_codec::decode` as a verify-or-reject oracle (`Mk1IndelOracle`, `mnemonic-toolkit/src/repair.rs:1001`; the comment at `repair.rs:997-998` notes `decode` "self-corrects t≤4 UNGUARDED, which would defeat the pure-indel rule"), so the toolkit's recovery soundness rests on mk's substitution-only / fail-closed behavior, which mk-codec never tests.

This is a **test-only** cycle (decision locked at brainstorm). It introduces no public-API change. If a new test surfaces a clear, contained bug, it is fixed inline (→ a mk-codec PATCH/MINOR bump + a toolkit git-dep pin refresh, with its own R0); a large/ambiguous bug is deferred (`#[ignore]` + FOLLOWUP). See §6.

### §1.1 Out of scope (filed separately)
The **depth/child "lossless by construction" seam** — the encoder drops `xpub.depth`/`child_number` (`bytecode/xpub_compact.rs:4`) and `reconstruct_xpub` (`:85-106`) rebuilds them purely from `origin_path`, never validating agreement with the input xpub. This is a behavior/contract decision (re-introduce `XpubDepthMismatch` vs document the lossy contract), not test-hardening. **Filed as a new FOLLOWUP** in `mnemonic-key` (primary) + a `mnemonic-toolkit` companion (the toolkit compensates via its own depth check, `synthesize.rs:494-503`). The theme-1 strategy sidesteps it by construction (§3).

---

## §2 Verified ground truth (SHA `d9d2ed9`)

- `KeyCard` derives `PartialEq, Eq` (`key_card.rs:23`) → the bijection `==` is sound.
- Public API (`lib.rs:51`): `encode(&KeyCard) -> Result<Vec<String>>`, `encode_with_chunk_set_id(&KeyCard, u32) -> Result<Vec<String>>`, `decode(&[&str]) -> Result<KeyCard>`. **No `decode_string` or `reassemble` at crate root** (`decode_string` exists only at the BCH layer `string_layer::bch::decode_string`; `reassemble_from_chunks` takes `Vec<ChunkFragment>`, post-BCH — NOT a string entry point).
- Encoder rejects: `policy_id_stubs` empty OR `len > u8::MAX` → `InvalidPolicyIdStubCount` (`bytecode/encode.rs:23,25`). Encodable range **1..=255**.
- Path: explicit-path decode rejects `count == 0 || count > MAX_PATH_COMPONENTS(=10)` → `PathTooDeep` (`bytecode/path.rs:114`, `consts.rs:27`). 14-entry standard-path dictionary (`path.rs:38-55`); `lookup_path` (`:72-81`) maps a matching explicit path back to the 1-byte indicator.
- **Both BCH codes are t=4** (correct up to 4 substitutions/chunk): regular `BCH(93,80,8)` and long `BCH(108,93,8)` — `bch.rs:376,451`; `decode_errors` rejects `deg > 4` (`bch_decode.rs:566`). The `error.rs:56` doc "(4 for regular, 8 for long)" is **misleading** — the `8` is the designed min-distance, not the correction count.
- Cross-chunk integrity = **4-byte** truncated hash `SHA-256(canonical_bytecode)[0..4]` (`consts.rs:45 CROSS_CHUNK_HASH_BYTES = 4`; `chunk.rs:70,189-201`) → `Error::CrossChunkHashMismatch` (`error.rs:97`). Residual miscorrection probability ≈ 2⁻³² (inherent design choice, not a defect).
- `encode()` draws a random chunk-set-id via `getrandom` (`pipeline.rs:45-49`, panics if OS CSPRNG unavailable, non-deterministic); `csid > MAX_CHUNK_SET_ID` (20-bit) → `ChunkedHeaderMalformed` (`pipeline.rs:85-93`).
- mk-codec is `std` (not `no_std`); has **no** `clippy.toml`/`disallowed-methods` (the Date/random ban is toolkit-only). proptest's RNG is unconstrained here.
- proptest precedent: `ms-codec/tests/round_trip.rs:11` uses bare `proptest! {}` default config and commits no `proptest-regressions/` dir; neither sibling gitignores it.

---

## §3 Theme 1 — property/fuzz harness (`tests/proptest_roundtrip.rs` + `tests/common/mod.rs`)

Add `proptest = "1"` to `crates/mk-codec/Cargo.toml` `[dev-dependencies]`.

**Shared generator** — `tests/common/mod.rs`, consumed by theme-1 and theme-2 via `mod common;`:
- `fn keycard_strategy() -> impl Strategy<Value = KeyCard>` producing a **valid, encodable, depth/child-consistent** card:
  - random 32-byte seed → `Xpriv::new_master(network, &seed)` → derive at the chosen path → `xpub` (so `xpub.depth == path.len()` and `xpub.child_number == path.last()`; mirrors `test_helpers.rs::synthetic_xpub` — sidesteps the §1.1 seam by construction).
  - **path** drawn from both encode modes: (a) a random entry of the 14-entry standard dictionary (exercises the 1-byte indicator), and (b) a random explicit path of **1..=10** components with random hardened bits (never empty). NOTE: an explicit path that happens to match a dictionary entry will encode via the indicator (`lookup_path`) — that is correct; the strategy does not assert encoding-mode from input-mode.
  - `policy_id_stubs`: `Vec<[u8;4]>`, length **1..=8** typically (a separate deterministic cell covers 255 — §4).
  - `origin_fingerprint`: random `Some`/`None`.
  - `xpub.network` ∈ {mainnet, testnet} is the **network source of truth** (the path family's mainnet/testnet table slot is cosmetic to the round-trip).

**Properties:**
- **P1 (bijection):** for a strategy-drawn `card` and a strategy-drawn `csid` masked to 20 bits, `decode(&encode_with_chunk_set_id(&card, csid).unwrap().iter().map(String::as_str).collect::<Vec<_>>()) == card`. (Use `encode_with_chunk_set_id`, not `encode()` — deterministic, shrinkable, no `getrandom` panic; mask csid so the `.unwrap()` can't hit `ChunkedHeaderMalformed`.)
- **P2 (panic-freedom):** `mk_codec::decode(&[s])` never panics for an arbitrary `s: String` (`"\\PC*"`) — returns `Err`. Plus a structured-corrupt variant: a valid encoding with random byte/symbol flips, still never panics. (Optional secondary target: the BCH-layer `string_layer::bch::decode_string` on arbitrary `&str`, if we choose to fuzz that layer directly — spec leaves it to P2-primary on the public `decode`.)

`proptest-regressions/` policy: **add `proptest-regressions/` to `mnemonic-key/.gitignore`** (cleaner than ms-codec's silent-untracked state).

---

## §4 Theme 2 — BCH adversarial coverage (`tests/bch_adversarial.rs`)

mk's guard model differs from md/ms: per-chunk, `bch_correct_*` re-verifies the corrected chunk's checksum (`bch.rs:434,487`) and `decode_errors` rejects `deg > 4` (`bch_decode.rs:566`); the residual defense against a 5+-error pattern that BM fits to a *wrong-but-valid* ≤4-degree codeword is the **4-byte cross-chunk hash** at reassembly (`pipeline.rs:288`, `chunk.rs:189-201`).

- **T2a — deterministic 3- and 4-error correction through public `decode()`** (today 3/4 errors are tested only at the raw `decode_long_errors` layer, `bch_decode.rs:779`; ≤2 via `bch_correct_*`). Encode a real card → perturb exactly 3, then 4, symbols in one chunk's **data part** → `decode` returns `Ok(original)` (chunk silently corrected). Cap at **4** — never expect >4 recovery (both codes t=4).
- **T2b — checksum/parity-region + mixed correction.** Perturb 1–4 symbols inside the 13-symbol (regular) / 15-symbol (long) BCH **checksum tail**, and a mixed data+checksum case at the t=4 boundary → assert `Ok(original)`. Exercises the position-translation `k = L-1-d` (`bch_decode.rs:587`) the current corpus never reaches (it only corrupts the data part).
- **T2c — randomized miscorrection sweep (proptest, via `common`).** Corrupt **5–8** random-position symbols in one chunk of a multi-chunk card → assert **`decode(perturbed) != Ok(original_card)`**. Rationale (architect must-fix): three outcomes are all legal — `Err(BchUncorrectable)`, `Err(CrossChunkHashMismatch)`, or (≈2⁻³²) `Ok(a different valid card)`. Asserting `.is_err()` would be flaky ~1-in-4.3e9 and proptest-shrink would chase a non-bug. The real, robust contract is "a ≥5-error corruption never *silently returns the original* as if clean." If a perturbation ever yields `Ok(original)` despite ≥5 changed symbols, that is the clear/contained bug we fix inline (§6).
- **T2-doc** — add a one-line doc-only note (and a FOLLOWUP) that `error.rs:56`'s "(4 for regular, 8 for long)" parenthetical reads as a correction count but means min-distance; both codes are t=4.

---

## §5 Theme 3 — indel reject-contract (`tests/indel_reject_contract.rs`)

Entry point: **`mk_codec::decode(&[&str])`** (NOT `reassemble` — no string-taking `reassemble` exists at mk's crate root; that is md's API shape).

- **T3a — in-band-length single indel.** Take a valid multi-chunk card; in one chunk (a) insert one symbol and (b) delete one symbol such that the chunk's data-part length stays within a valid BCH band → `decode` returns **`Err(_)`** (assert `is_err()`, NOT a specific variant — an indel can legally surface `BchUncorrectable` / `CrossChunkHashMismatch` / `MalformedPayloadPadding`). The toolkit-relevant property: an indel never self-corrects into a *different valid* `Ok`.
- **T3b — band-boundary deterministic fixture.** A delete that pushes a chunk's length into the reserved 94/95 gap (or a length outside any band) → assert the specific **`Err(InvalidStringLength)`** (`bch.rs:669` — this one IS deterministic and safe to pin).
- **T3-doc** — the test file's module doc cross-cites the consumer it protects: `mnemonic-toolkit/src/repair.rs:1001` (`Mk1IndelOracle`) + `:997-998`.

---

## §6 SemVer / branch / lockstep / bug-handling

- **Branch:** all work commits to `main` (mk-codec's default branch).
- **SemVer:** test-only ⇒ **no version bump** (proptest is a `[dev-dependencies]` add; no published-API change). Commit to `main`.
- **Bug-handling (decision locked):** a clear/contained defect surfaced by a new test (T2c is the likeliest, per the convergence-suite precedent) is **fixed inline** → mk-codec PATCH (`0.3.1→0.3.2`) or MINOR if a behavior change, with its own opus R0 on the fix, and a `mnemonic-toolkit` git-dep pin refresh if the toolkit consumes the bumped version. A large/ambiguous defect is **deferred**: mark the test `#[ignore]` with a FOLLOWUP reference, keep the cycle test-only. Either path is surfaced to the user.
- **Lockstep:** none — no clap/CLI/manual/GUI surface change (these are crate-internal tests).
- **FOLLOWUPs filed this cycle:** (a) depth/child seam (§1.1, mk primary + toolkit companion); (b) `error.rs:56` doc inaccuracy (§4 T2-doc, doc-only).

---

## §7 Test inventory (TDD — tests precede/accompany impl each phase)

| ID | File | Pins |
|---|---|---|
| P1 | `proptest_roundtrip.rs` | encode↔decode bijection over the full `KeyCard` space (both path modes, both networks, stub 1..=8, Some/None fp) |
| P2 | `proptest_roundtrip.rs` | `decode(&[s])` never panics on arbitrary / structured-corrupt input |
| T2a | `bch_adversarial.rs` | 3- & 4-error data-part correction through public `decode()` → `Ok(original)` |
| T2b | `bch_adversarial.rs` | checksum-region + mixed-region 1–4-error correction → `Ok(original)` |
| T2c | `bch_adversarial.rs` | 5–8-error single-chunk corruption → `decode != Ok(original)` (cross-chunk-hash guard load-bearing) |
| T3a | `indel_reject_contract.rs` | in-band-length single indel → `is_err()` |
| T3b | `indel_reject_contract.rs` | length-out-of-band delete → `Err(InvalidStringLength)` |
| T4 | `bch_adversarial.rs` or `round_trip.rs` | **255-stub round-trip** (≈21 chunks → >2-chunk real-card coverage) + **256-stub → `Err(InvalidPolicyIdStubCount)`** |

Gate: `cargo test -p mk-codec` green; `cargo clippy -p mk-codec --all-targets` clean; existing 3 test files + in-src unit tests stay green.

---

## §8 Phased plan (per-phase opus review → 0C/0I, persisted to the repo's agent-reports if present, else `design/`)

- **Phase 0 — harness scaffold.** Add `proptest` dev-dep + `.gitignore` line; write `tests/common/mod.rs` (`keycard_strategy`) + P1/P2 (red→green). Confirm `KeyCard: PartialEq`, the API signatures, and that the strategy generates only encodable cards (the §2 caps). This phase alone validates the strategy and likely shakes out any bijection surprise.
- **Phase 1 — Theme 2.** T2a, T2b, T2c (+ T2-doc note + FOLLOWUP). T2c is the highest-risk-of-finding-a-bug; if it goes red, apply §6 bug-handling.
- **Phase 2 — Theme 3 + T4.** T3a, T3b (+ consumer cross-cite), T4 stub-boundary pair. File the §1.1 depth/child FOLLOWUP (mk + toolkit companion).
- **Phase 3 — verify + end-of-cycle R0.** Full `cargo test` + clippy; end-of-cycle opus R0 → 0C/0I; commit to `main` (no bump unless §6 fired).

---

## §9 R0 agenda (what the architect must stress)
1. The strategy generates only encodable+decodable cards (the §2 caps actually hold; no spurious P1 failures).
2. T2c's `!= Ok(original)` framing is the right robust assertion (not `.is_err()`); the 2⁻³² residual is correctly characterized.
3. T2a/T2b: 4-error correction is genuinely reachable through public `decode()` for both code variants; no >4 expectation leaked in.
4. T3a's `is_err()` (not variant-pinned) is correct; T3b's `InvalidStringLength` pin is deterministic.
5. Entry points: `decode(&[&str])` everywhere; no nonexistent `decode_string`/`reassemble` crate-root reference.
6. Nothing mis-scoped (SLIP-0132 correctly excluded; 255-stub correctly folded in; depth/child correctly deferred).

---

## §10 Source citations (verified at `d9d2ed9`)
`key_card.rs:23` (derive), `lib.rs:51` (API), `bytecode/encode.rs:23,25` (stub cap), `bytecode/path.rs:114` + `consts.rs:27` (path cap), `bytecode/xpub_compact.rs:4,85-106` (depth/child seam), `string_layer/bch.rs:376,451` (t=4), `bch_decode.rs:566` (deg>4), `bch_decode.rs:587` (k=L-1-d), `bch_decode.rs:779,811` (raw-layer 4-error / 5-error), `consts.rs:45` + `chunk.rs:70,189-201` (4-byte hash), `error.rs:56,97` (doc + CrossChunkHashMismatch), `pipeline.rs:45-49,85-93,271-348,288` (getrandom, csid cap, existing 5-burst test, guard). Consumer: `mnemonic-toolkit/src/repair.rs:997-998,1001`. Precedent: `mnemonic-secret/crates/ms-codec/tests/round_trip.rs:11`.
