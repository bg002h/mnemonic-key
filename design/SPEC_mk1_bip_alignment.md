# SPEC — mk1 BIP↔code alignment

**Repo:** `mnemonic-key` (mk-codec + mk-cli). Source SHA at authoring: `origin/main 1c9fbf7`. mk-codec 0.4.1 / mk-cli 0.12.0.
**Origin:** Fable adversarial BIP-vs-impl review (`mnemonic-toolkit/design/agent-reports/bip-review-mk1-fable-r0.md`); consolidated bug list `mnemonic-toolkit/design/BUGLIST_bip_alignment_cycle_2026-07-10.md` (bucket C + DG ledger).
**Companion cycle:** md1 (`descriptor-mnemonic`) — separate SPEC, same program. The §Checksum algorithm here MUST be init/algorithm-identical to md1's (shared `POLYMOD_INIT 0x23181b3`; only HRP + target constants differ). F-A6's `bch.rs` init-comment fix must be **byte-consistent** with md1's F-A5 companion. The DG-1/2/3 FOLLOWUPs carry companion entries here per the cross-repo rule.
**User order (2026-07-10):** **BIPs → bugs → …** (global). mk1 is BIP-dominant; no A4/scope (md-only). R0 history: round 1 (3I), round 2 (2I), round 3 **GREEN (0C/0I)** — all under `mnemonic-toolkit/design/agent-reports/mk1-bip-alignment-spec-r0-round-{1,2,3}.md`.

## Goal

Make an independent implementer following only `bip/bip-mnemonic-key.mediawiki` reconstruct mk1 cards byte-identically to the shipped `mk-codec`. **This cycle is almost entirely BIP-document work** — the wire format is correct, internally consistent, and vector-pinned in-repo. **No WIRE change, no runtime-BEHAVIOR change, no clap-surface change.** Code touches: F-A6/F-A7 comment/convention corrections, PLUS (Phase 3) a new depth-0 test fixture + corpus regen + one SHA re-pin + the family-token roll — **test/vector-data churn, not behavior** (R0 I-2). Scope claims below are per-phase, not cycle-wide.

## Non-goals (deferred — downgrade ledger)

Erasure-aware decoding, guided recovery, confidence-tier reporting (MK1-I3 / DG-1/2/3) — OUT; the BIP is made honest about their absence. The BCH **substitution**-error correction `mk repair` genuinely performs (t=4 regular) stays normative.

---

## Part 1 — Code touches (mk-codec) — comments/convention ONLY, no behavior change

### F-A6. Code-doc corrections
- `error.rs:57` — "substitution capacity (4 for regular, **8 for long**)" is the MYTH: both codes correct t=4; 8 is the long-code *detection* radius. Fix comment → "4 substitutions (regular and long); 8 is the long-code detection radius, not correction."
- `bch.rs:185-186` init-comment (R0 M-3) — flip "deliberately NOT [BIP-93's init]… starts from 1" to "**IS** BIP-93's published `ms32_polymod` init `0x23181b3` verbatim", while **KEEPING the load-bearing equivalence note** (`0x23181b3` = fold of `hrp_expand("ms")` from 1; ms1/rust-codex32 uses the equivalent init-1 + prepend formulation). Stay consistent with the already-correct test comment at `:857-863`. No behavior change (`POLYMOD_INIT` const is already `0x23181b3`).
- `error.rs:116` "0x16 reserved" comment (R0 M-4) — stale (0x16 assigned since v0.2.0, `path.rs:53`, + in the BIP table `:317`); re-word to reflect assignment, citing `md-path-dictionary-0x16-gap`'s resolved status (`FOLLOWUPS.md:263-269`) as the verification source.
- Any code doc-comment citing a BIP §"Checksum" (nonexistent until this cycle) — after Part 2 adds §Checksum, these cites become valid; ensure they point at the right section name.

### F-A7. `GENERATOR_FAMILY` token — ROLL to "mk-codec 0.4" (R0 I-1 ruling; my round-1 "stable anchor" was wrong)
**Now:** `consts.rs:50 GENERATOR_FAMILY = "mk-codec 0.2"`; crate is 0.4.1; BIP §Test Vectors + `consts.rs:47-49` + `tests/vectors.rs:46-52` all state "minor bumps roll the token."
**R0 correction:** my round-1 "don't churn all SHAs" premise was FALSE — there is exactly ONE pinned corpus SHA (`tests/vectors.rs:41 V0_1_SHA256`), and Phase 3's depth-0 vector (F-V-mk) **regenerates `v0.1.json` and re-pins that SHA this cycle regardless**, so rolling the token has ZERO marginal churn. The Q-10 precedent (v0.2.0's 0x16 + V18) rolled the token; v0.3/v0.4 silently missed their rolls. Coherence hazard: the new depth-0 vector decodes only under v0.4.0+, so stamping the corpus `"mk-codec 0.2"` mislabels its decode-family at the exact moment C-C3 pins it in the BIP.
**Decision:** **ROLL `GENERATOR_FAMILY` → `"mk-codec 0.4"`** inside the Phase-3 regeneration (framed as completing v0.4.0's missed roll, not a patch-triggered roll). Keep Q-10's roll-on-minor convention in the BIP with an **honesty note** (v0.3/v0.4 missed their rolls; corrected here). Rewrite `consts.rs:47-49` + `tests/vectors.rs:46-52` + **`tests/vectors.rs:129-133`** (the `schema_metadata_pinned` test asserts the hardcoded literal `"mk-codec 0.2"`, NOT the const — R0 I-B; it hard-fails Phase-3 regen until edited) + BIP §Test Vectors **mandatorily and byte-consistently** (no "optional comment" — that recreates the A6 self-contradiction class this cycle purges).

---

## Part 2 — mk1 BIP → code alignment (`bip/bip-mnemonic-key.mediawiki`)

Itemized findings + exact cites: review + BUGLIST bucket C. Grouped for the plan:

- **C-C1 (add a normative §Checksum) — THE headline fix.** The BIP currently has NO §Checksum (verified: only §Test Vectors at line 512). Add a full normative section documenting the shipped algorithm from `crates/mk-codec/src/string_layer/bch.rs` (there is a good doc-comment at :279-326 to lift): polymod seed `POLYMOD_INIT = 0x23181b3`; input = `hrp_expand("mk") ‖ data ‖ [0; 13]` (regular) / `[0; 15]` (long); run `polymod_run` with `GEN_REGULAR`/`GEN_LONG`; XOR `MK_REGULAR_CONST` / `MK_LONG_CONST` to extract/verify (verify condition: `polymod(hrp_expand("mk") ‖ data_with_checksum) == MK_*_CONST`). Include `hrp_expand("mk") = [3,3,0,13,11]`, the generator polynomials, the target-constant derivation, the **checksum-symbol extraction order** (13/15 five-bit symbols, big-endian: first symbol = top 5 bits of the XORed residue, `bch.rs:310-312`), the **code-selection threshold** (R0 I-A — state BOTH sides; V1 chunk 0 = 93 pre-checksum data symbols takes the LONG code, so a "≤93→regular" rule is WRONG): **encoder** — pre-checksum data 1–80 → regular (append 13), 81–93 → long (append 15), >93 invalid (data <14 total is rejected by the decoder floor, below) (equivalently `bch.rs:525-545`'s two-step try `data+13`-regular-else-`data+15`-long); **decoder** — total data-part 14–93 → regular, **94–95 reserved-invalid** (`InvalidStringLength`), 96–108 → long (`bch_code_for_length`, `bch.rs:111-124`). Both needed to reproduce V1), and a **worked example against vector V1** (so 2-of-3 good-faith readings can't reject every card). State explicitly this is init/algorithm-identical to md1's §Checksum (shared init; HRP="mk" and targets differ).
- **C-C2 depth-0 / empty-path.** Change `component_count MUST be 1..=10` → **`0..=10`**; specify `count == 0` = no-path/depth-0 root key (e.g. WIF) reconstructing `depth=0, child_number=Normal{0}` (code: `bytecode/path.rs:115-118`, `xpub_compact.rs:86-108`); decoder rule 5 rejects only `> 10` (not `== 0`); explicit-path escape size "3..=52 B" → **"2..=52 B"**; define `child_number` for the empty-path case. Add a depth-0 test vector (F-V-mk).
- **C-C3 embed test vectors.** §Test Vectors currently says "To be written" (line 514) while `crates/mk-codec/src/test_vectors/v0.1.json` ships 18 positive (incl. two 3-chunk) + 22 negative. Embed or normatively pin-by-SHA the positives (per-chunk strings, `canonical_bytecode_hex`, `chunk_set_id`, `total_chunks`) + the negative table + the new depth-0 vector; correct the stale prose + the family-token convention (F-A7).
- **C-I1 encoder invariant.** Port SPEC §4's `XpubOriginPathMismatch` encoder-MUST paragraph (`encode.rs:31-48`; `SPEC_mk_v0_1.md:295-304`), incl. the empty-path `Normal{0}` clause. Delete the false "drift impossible by construction" as stated (lines 364/396) or qualify it.
- **C-I2 long-code substitution capacity 8→4.** Fix lines 29, 480, 504 ("8 for the long code") → 4 (matching line 144 + code `bch.rs` t=4). 8 is detection, not correction.
- **C-I3 erasure/guided-recovery/confidence — DOWNGRADE (DG-1/2/3).** MUST→SHOULD/informative (lines 145-146, 150). Keep substitution-correction normative. Add the code-distance note + FOLLOWUP cites (symmetric with md1 B-I5).
- **C-I4 encoder chunking + line 74.** Specify: fragments = successive **53-byte** slices of `bytecode ‖ hash`, last = remainder (`crates/mk-codec/src/string_layer/chunk.rs:50` `split_into_chunks`); per-fragment 8→5-bit with zero pad; per-chunk code auto-selected by data-part length; "decoders MUST accept any division" (reassembly concatenates). Correct line 74's false "lands in 2 long-code chunks" (reality: chunk 0 long, chunk 1 regular — mixed is normal). Explicitly **contrast md1's variable byte-boundary framing** so implementers don't cross-import (both are byte-boundary; mk1 = fixed 53-byte, md1 = variable).
- **C-I5 §Linkage stub formula.** Rewrite line 400 (superseded "canonical-bytecode SHA-256 prefix") to match lines 37/48/268/404: stub = top-4-bytes of **WalletPolicyId** (keyed md1) **or WalletDescriptorTemplateId** (keyless template md1) — form-aware (`key_card.rs:26-33`). Add the keyless-template stub rule (absent).
- **C-I6 xpub version-byte set.** Enumerate the exact **wire-layer** accepted set `{0x0488B21E (xpub), 0x043587CF (tpub)}` (`xpub_compact.rs:25-28,63-69`); all others → `InvalidXpubVersion`. **Scope this to the WIRE (R0 M-1):** note the reference mk-cli *normalizes* SLIP-0132 prefixes (ypub/zpub/upub/vpub…) on input (`mk-cli/src/slip132.rs`, FOLLOWUP `mk-slip0132-prefix-acceptance`) — the wire never carries them, so a blanket "encoders reject all others" would contradict observable CLI behavior. (BIP currently names only mainnet.)
- **C-M1…M6:** §Decoder-validity omits string-layer errors (InvalidHrp, MixedCase, InvalidChar, InvalidStringLength, BchUncorrectable, empty-input, CardPayloadTooLarge); min data-part length 14 unstated; **the 94–95 reserved-invalid length gap** unstated (R0 I-A — total data-part 94–95 → `InvalidStringLength`); chunk_set_id slot-XOR note (A9 — toolkit XORs slot index, so BIP formula matches only slot 0; document the slot-XOR); family-token (F-A7); code-doc cleanups (F-A6); line-116 structural-audit commitment vs FOLLOWUPS closure.

## Part 3 — test vector (mk-codec)

- **F-V-mk.** Add a **depth-0 / empty-path** vector (none exists; V1–V18 all non-empty) proving the C-C2 `0..=10` rule round-trips. Regenerate via the real vector tooling; all existing vectors must round-trip unchanged.

---

## Ripple / lockstep

- **NO clap-surface change** → no GUI `schema_mirror` change, no manual `40-cli-reference` flag update. (Churn is comments + BIP + Phase-3 vector-data per acceptance-1 — R0 I-B; not "one added vector only".)
- **PATCH release (R0 ruling):** mk-codec 0.4.1→0.4.2 + mk-cli 0.12.0→0.12.1 lockstep publish to crates.io — the regenerated `v0.1.json` is published-library content (`include_str!` + `mk vectors`) and the BIP pins the post-regen corpus, so it must ship. No wire change (depth-0 shipped in 0.4.0).
- **Toolkit: NO ACTION (R0 I-3 — sibling-pin footgun).** `install.sh:41` pins `mk-cli-v0.12.0` as a **FROZEN baseline** — bumping it breaks `sibling-pin-check` post-tag (the v0.75.0 revert+re-cut incident). Toolkit `crates/mnemonic-toolkit/Cargo.toml:33 mk-codec = "0.4.1"` is caret → 0.4.2 is compatible with zero toolkit edits (root `Cargo.toml` has no mk-codec dep). So: **NO install.sh sibling-pin change, NO toolkit release**; the `Cargo.lock` refresh rides the next toolkit cycle.
- **Cross-repo companion:** md1's §Checksum (companion SPEC) MUST stay algorithm-identical; the DG-1/2/3 FOLLOWUPs get companion entries in `descriptor-mnemonic` per the cross-repo rule.

## Acceptance criteria

1. F-A6/F-A7 fixes applied; `cargo test -p mk-codec` + `-p mk-cli` green; NO wire/runtime-behavior change; **V1–V18 strings/bytecode byte-identical** (R0 I-2 — the ONLY invariant; the cycle's sanctioned churn set is exactly: the new depth-0 gen fixture in `src/bin/gen_mk_vectors.rs`, the `v0.1.json` regen, the `tests/vectors.rs:41 V0_1_SHA256` re-pin, the `GENERATOR_FAMILY`→"mk-codec 0.4" roll per F-A7, and the `tests/vectors.rs:129-133 schema_metadata_pinned` literal update).
2. BIP: §Checksum added + verified by a worked example reproducing vector V1; depth-0 rule `0..=10`; vectors embedded/pinned; I1-I6 folded; downgrades ledgered with FOLLOWUP cites; no remaining contradiction (8-vs-4, stub formula, xpub-version set).
3. **Recovery-independence spot-check:** re-dispatch a Fable read of the rewritten §Checksum + §Chunking that reconstructs vector V1's checksum + fragment split from the BIP text alone.
4. FOLLOWUPs filed with md1 companions: DG-1/2/3 (erasure/guided/confidence), + mk1-specific (min-data-part-length doc, xpub-version-set doc if kept). **M-4 hygiene:** the shipping commit annotates `mk1-no-path-depth0-support` (`FOLLOWUPS.md:365`) — C-C2's BIP fix closes the BIP-lockstep gap it left open.
5. Release ritual complete (**PATCH: mk-codec 0.4.1→0.4.2 + mk-cli 0.12.0→0.12.1 lockstep** — R0 ruling; the corpus is published-library content baked via `include_str!` + surfaced by `mk vectors`, so the post-regen corpus must ship in a release); CI `vectors-roundtrip` green. **Toolkit: NO action** (R0 I-3 — see Ripple).

## Phasing (BIPs → bugs; mk1 is BIP-dominant, no code bugs block anything)

- **Phase 1 — BIP prose (first):** C-C1 (§Checksum + worked example reproducing V1), C-C2 (depth-0 `0..=10`), C-C3 (embed the existing v0.1 corpus + I1-I6 + M1-M6). The BIP states correct behavior; the code already conforms (depth-0 works at `path.rs:117`; the checksum constants exist).
- **Phase 2 — code comment touches:** F-A6 (bch.rs/error.rs comment corrections, byte-consistent with md F-A5) — no behavior change.
- **Phase 3 — vector + finalization:** F-V-mk depth-0 vector (new `gen_mk_vectors.rs` fixture) + `v0.1.json` regen + `V0_1_SHA256` re-pin + **F-A7 `GENERATOR_FAMILY`→"mk-codec 0.4" roll** (byte-consistent across `consts.rs:47-49` / `tests/vectors.rs:46-52` / BIP §Test Vectors) + re-sync the BIP §Test Vectors table/pins to the final corpus + FOLLOWUPs (DG companions) + **PATCH release ritual** (mk-codec 0.4.2 / mk-cli 0.12.1).
- Per-phase R0; post-impl whole-diff review. Recovery-independence spot-check (Fable reconstructs V1's checksum from the §Checksum text alone) is a Phase-3 gate.
