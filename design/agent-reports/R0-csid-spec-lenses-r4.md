# R0 r4 — two never-run lenses over `design/SPEC_chunk_set_id_verification.md` @ `c4900a7`

**Verdict: 0 Critical / 5 Important / 4 Minor / 1 Nit.**

Lens 1 (adversarial construction) and Lens 2 (`mk repair` exit-5 bless path)
were the two lenses r1's closure note named as never-run. Both are exercised
here against the real code and a working mk1 minter, not against the prose.
Contract-completeness and the wording walk are **out of scope by brief** and
were not re-derived; the settled rulings (warnings not refusals, opaque-id
guarantee, vectors-only formulas, post-cycle legs) are taken as given.

Everything below labelled *measured* was executed today at `c4900a7`
(mnemonic-key) / `origin/main` (descriptor-mnemonic). Reproduction commands
are inline.

## Method note — the harness this report rests on

A byte-exact mk1 minter/parser was re-implemented in Python from
`crates/mk-codec/src/string_layer/bch.rs` (GEN_REGULAR/GEN_LONG,
`POLYMOD_INIT`, `MK_REGULAR_CONST`, `MK_LONG_CONST`, `hrp_expand`,
`bytes_to_5bit`) and **validated against the real CLI**: re-minting the
84-byte canonical bytecode of a card produced by
`mk encode --xpub … --origin-path "m/84'/0'/0'" --origin-fingerprint aabbccdd
--policy-id-stub 11223344` reproduced both mk1 strings byte-for-byte
(`re-mint … matches the CLI output: True`). Every "WORKS" construction below
was minted with that harness and then fed to the real `target/debug/mk`.

Independent confirmations of the spec's own numbers (all recomputed, none
transcribed):

- v0.1 corpus: **19 chunked vectors, 19/19 declared ≠ derived** — the spec's
  claim, verified. `V1_bip48_mainnet_1_stub_with_fp: declared=12345
  derived=83bb2` — exactly as the spec states.
- `derive_chunk_set_id` = top-20 bits of `SHA-256(canonical_bytecode)`,
  MSB-first: reproduced against live CLI output (`declared=94b47
  derived=94b47` on an unpinned mint; `declared=12345 derived=94b47` on
  `--chunk-set-id 0x12345`).
- `mk encode --chunk-set-id` mints in **total silence** (stderr carries only
  the display-grouping echo and the watch-only note) — R3's premise, verified.

---

# LENS 1 — construct a card set that DEFEATS the warning

Dispositions below are **WORKS** (a set exists that is tampered or
non-conforming and warns nowhere) or **BLOCKED** (with the exact mechanism).

## L1-1 — Full re-mint substitution — **WORKS** (trivial, by construction)

An attacker who can put plates in front of the operator re-mints the card
they want seated: `mk encode --xpub <attacker xpub> …` with no
`--chunk-set-id`. Declared == derived by construction, so **no surface
warns, ever**. The warning detects *inconsistency between a stamped id and
content*; it cannot detect *authenticity*, because the id is a function of
the content the attacker chose.

## L1-2 — Re-mint that PRESERVES the victim's id label — **WORKS, measured 0.20 s**

The interesting half: the substituted card can also be made to answer to the
victim's five-hex label, so plate markings, operator notes, and a
`--seat @i=<id>` directive all still "match".

Measured: 20 bits of id, ground over the fully operator-chosen 4-byte
`policy_id_stub` (bytecode offset 2..6):

```
target=94b47 found stub=00050e62 after 331363 tries in 0.20s
(1,675,550 tries/s, single-core CPython)
```

Then minted with the real CLI and confirmed end to end:

```
clean.txt  : declared=94b47 derived=94b47 stub=11223344
collide.txt: declared=94b47 derived=94b47 stub=00050e62
```

Two **different, entirely valid** key cards (different wallet binding), both
carrying `94b47`, both silent under the proposed warning. Grinding the
attacker's *own key* instead of the stub is the same 2^20 cost (they hold
2^20 xpubs trivially).

**Not a funds path in today's surfaces** — and this is worth stating so the
finding is not inflated: if both cards are supplied together they merge into
one group and refuse (R5 arm 1/3), and `--seat` only chooses among seatings
`satisfies()` already permits, so a grinded card must still satisfy the slot
declaration. The defect is a **claim**, not a behaviour:

> **[Minor L1-M1]** The "why warnings" section lists **"(4) deliberate
> tamper"** among the things a clean mismatch arises from, which invites a
> future reader (or a future `--strict` cycle) to treat the warning as a
> tamper control. It is not one, and cannot be: a 20-bit id is grindable in
> 0.20 s and a full re-mint is undetectable by construction. **Fix:** one
> non-goal sentence — *"the warning is a consistency check on how the id was
> chosen, never an authenticity check; a re-minted card always passes, and
> the 20-bit id can be ground onto any target in ~2^20 work (measured 0.2 s)."*

## L1-3 — Non-conforming plate stamped from the CANONICAL re-encode — **WORKS**

The operand is `derive(encode_bytecode(decoded_card))`, so the check
compares the stamped id against *what this version's encoder would emit* —
not against the bytes on the plates. The bytecode layer accepts at least two
non-canonical forms (both verified by code-read **and** executed):

- **explicit-path escape for a path that IS in the standard table.**
  `decode_path` (`bytecode/path.rs:101`) routes `0xFE` to
  `decode_explicit_path` with no check that the decoded path is absent from
  `STANDARD_PATHS`.
- **non-minimal LEB128 components.** `leb128_decode_u32`
  (`bytecode/path.rs:148`) has no minimality check; it only bails at
  `shift >= 35`. `[0x80, 0x00]` decodes to 0 just as `[0x00]` does.

Minted and run against the real binary:

| plate | on-wire bytecode | declared | `mk decode` |
| --- | --- | --- | --- |
| canonical | 84 B, path = `03` | `94b47` | OK |
| **A1** — explicit path, self-consistent id | 100 B, path = `fe03d4808080088080808008 8080808008` | `91ff6` = derive(plate bytes) | **OK, identical KeyCard** |
| **A2** — same plate, id from the canonical re-encode | 100 B | `94b47` = derive(re-encode) | **OK, identical KeyCard** |

A1 **warns** (derived `94b47` ≠ declared `91ff6`) — the F-212 drift tripwire
working exactly as designed. **A2 warns nowhere**, and it is a
non-conforming plate. Impact is low (the decoded `KeyCard` is identical, so
no funds consequence), but it falsifies a sentence the spec states as fact:

> **[Minor L1-M2]** The Comparison section says *"A card whose payload does
> not re-encode to the bytes on the plates is therefore IN scope for the
> warning."* **Measured false** (A2). Only a card whose *stamped id*
> disagrees with the re-encode is in scope. **Fix:** replace with *"a card
> whose stamped id disagrees with the re-encode is in scope; a
> non-conforming plate whose id was computed from the canonical re-encode is
> not, and is out of the warning's reach by design."*

## L1-4 — The operand is VERSION-DEPENDENT — **Important**

> **[Important L1-I1]** `encode_bytecode` is not a fixed function. Any change
> to it re-derives every id, and **the spec has no clause freezing it or
> treating a change to it as a mismatch-generating wire event** (grepped:
> the spec mentions canonicalization only as *foreign*-encoder drift, line
> 74; nothing about this encoder moving).

This is not hypothetical. **The standard path table has already grown once:**

```
crates/mk-codec/src/bytecode/path.rs:53
  (0x16, "m/48'/1'/0'/1'"), // v0.2.0+; was reserved-pending in v0.1.x
git log -S: fd6a407 feat(mk-codec v0.2.0): wire-additive 0x16 path indicator
CHANGELOG.md:170 "brings mk1's standard-table dictionary to its full 14
                  entries, matching md-codec v0.9.0+'s table"
```

Before v0.2.0 that path had no indicator, so `lookup_path` returned `None`
and the encoder emitted the 17-byte explicit form; today it emits one byte.
That is exactly the A1/A2 shape, and the two encodings of one card derive
**different ids** (measured: `91ff6` vs `94b47`).

Consequence if the table (or any other canonicalization detail) moves again:
**every genuine, correctly-minted 0.5.x plate of the affected shape warns
forever**, and the frozen remedy text tells the operator

> *"To fix it, re-mint: run mk encode again without --chunk-set-id …"*

i.e. **re-engrave a plate that is fine** — the precise defect shape the
constellation already ruled against in this same seat path
(`descriptor-mnemonic/crates/md-cli/src/seat/input.rs:106`: *"a diagnosis of
the wrong problem whose remedy is re-engraving a plate that is fine"*).

**Fix (spec text, no code):** state that the canonical bytecode encoding is
**frozen for id purposes** from mk-codec 0.5, that any future change to
`encode_bytecode`'s output — *including adding a `STANDARD_PATHS` entry* —
is a mismatch-generating event that must be handled as a wire-compat
question, not a minor addition; and add an executable row to the extension
corpus that pins the table's current 14 entries so a future addition trips a
test rather than a field warning.

## L1-5 — Rendering: `{:x}` vs `{:05x}` — **Important**

> **[Important L1-I2]** The rendering rule says *"bare lowercase hex (e.g.
> `12345`, `ef12f`), matching the existing 'chunk-set 12345' diagnostic
> surface."* The cited surface is **`{id:05x}` — five digits, zero-padded**,
> and the spec's paraphrase drops the padding while all four example ids
> happen to have no leading zero.

Measured, the three surfaces an operator compares against:

| surface | format | file |
| --- | --- | --- |
| md seating refusals + card labels | `write!(f, "{id:05x}")` | `md-cli/src/seat/input.rs:59` |
| `md --seat @i=<id>` **input** | rejects any token whose digit count ≠ 5 | `md-cli/src/seat/directive.rs:68` |
| `mk repair` reject message | `format!("chunk_set_id 0x{csid:05x}")` | `mk-cli/src/cmd/repair.rs:267` |

`--seat` does not merely tolerate five digits, it **refuses** four:

```
--seat @{i}: `{rhs}` is {n} hex digit(s); a chunk-set id is exactly five.
```

So an id rendered `{:x}` is a token the sibling CLI rejects — on the very
surface (contract 3's unconditional `mk inspect` print, "so the warning's
value has a cross-check surface") whose purpose is letting the operator
carry the id between tools. It also collides with W14's anti-transcription
ruling: the operator is being handed a differently-shaped id in each tool.

**The case is live in the existing corpus** — 3 of the 19 rows that become
the pinned-by-design MISMATCH half carry an id below `0x10000`:

```
V7_max_path_components_no_fp : declared=78901  derived=03994  → "3994"
V9_bip44_mainnet_1_stub_with_fp: declared=9a012 derived=04cf9 → "4cf9"
V16_bip86_testnet_1_stub_no_fp: declared=01789 → "1789"       derived=dc033
```

(1 in 16 ids has a leading-zero nibble.) None of the spec's *named* new rows
— `1b1ba`, `ef12f`, `12345` — exercises it, so the extension corpus as
specified cannot catch it, and the `warning_text` rows would freeze the
wrong rendering.

**Fix:** say **"exactly five lowercase hex digits, zero-padded (`{:05x}`) —
the token `GroupId::Display` prints and `--seat @i=` accepts; never `{:x}`"**,
and require at least one extension-corpus row whose derived id is `< 0x10000`.

## L1-6 — Constructions that are BLOCKED (with the blocking mechanism)

| construction | disposition | mechanism (measured) |
| --- | --- | --- |
| **Re-present a chunked card as a single string** to dodge "chunked input only" | **BLOCKED** | Capacity. Max payload in one `SingleString` mk1 string = **56 B**; smallest possible valid card bytecode = **80 B** (`mk encode --privacy-preserving`, 1 stub, table path → measured 80 B, 2 chunks). A 56-byte `SingleString` card fed to `mk decode` → `error: unexpected end of bytecode`, exit 1. No valid KeyCard can ever be single-string, so the scoping is structurally safe. |
| **Re-present as a 1-chunk chunked set** (`total_chunks = 1`) | **BLOCKED** | Capacity. Max payload in one `Chunked` string = **53 B** < 80 B. Minting one raises at the BCH layer (`bad len 149`; the data part would need 149 symbols, cap is 108). |
| **Chunks' individual header ids diverging from the group id post-reassembly** | **BLOCKED** | `reassemble_from_chunks` compares every chunk's `chunk_set_id` to chunk 0's and returns `ChunkSetIdMismatch` before any bytecode exists. Live: splicing clean chunk 0 + pinned chunk 1 → `error: chunk_set_id mismatch across chunks`, exit 1. Post-reassembly all chunks provably share one declared id, so "the GROUP id" is well-defined. In md, grouping is *by* the id (`group_key_of`), so divergence cannot even enter one group. |
| **Coincidental `derived == declared` on a damaged card** | **BLOCKED in practice** | Any payload change fails the 4-byte cross-chunk hash (2^-32) before the id is consulted; any header-region change splits the group. The reachable version is the *deliberate* grind, L1-2. |
| **Batch mint pinning N cards to one id** (`--chunk-set-id` + `--keys`) | **BLOCKED** | Already mutually exclusive, with the right reason on the tin: `mk-cli/src/cmd/encode.rs:154` — *"--chunk-set-id pins ONE card's 20-bit id; N cards cannot share it."* Contract 5's singular wording is therefore safe. |
| **A decoded card whose re-encode ERRORS** (leaving the operand undefined) | **BLOCKED** | `encode_bytecode`'s four error arms are all symmetric with the decoder: stub count `1..=255` both sides (`u8` on the wire); path components capped at `MAX_PATH_COMPONENTS = 10` both sides; `XpubOriginPathMismatch` unreachable because `reconstruct_xpub` *builds* depth/child from `origin_path`; `encode_xpub_compact` is infallible and the version↔network map is bijective (2↔2). The operand always exists. *(Argued from the four arms + the bijection, not executed — the only claim in this report not run.)* |

---

# LENS 2 — the `mk repair` exit-5 bless path

All four states were run against the pinned card (declared `12345`, derived
`94b47`), damaged with a single-character substitution at data-part
position 27 of chunk 0.

| supply | exit | stdout | stderr | is a card decoded? |
| --- | --- | --- | --- | --- |
| pinned card, **undamaged**, both chunks | **0** | corrected chunks | watch-only note only | **NO** |
| pinned card, damaged, **both** chunks (Bless) | **5** | `# Repair report` + corrected chunks | watch-only note only | **YES** (`repair.rs:410`) |
| pinned card, damaged, **one chunk alone** (Candidate) | **5** | report + the one chunk | `warning: correction UNVERIFIED …` | **NO** |
| complete group that re-decodes `Err` (Reject) | 2 | *suppressed* | error | attempted, failed |

## L2-1 — repair warns on ONE of its three success states — **Important**

> **[Important L2-I1]** `classify_mk1_set` is engaged only `if
> any_correction` (`repair.rs:187`), and a card is decoded only inside it,
> only for a group that is **complete AND consistent**. So the mismatch
> warning is reachable **only on the Bless path**. On the exit-0 path (input
> already valid) and on the Candidate path (the documented per-plate,
> single-plate workflow) **no decode happens at all**, so a pinned/mismatched
> card passes through `mk repair` completely silent. The spec says neither
> that this is intended nor what to do instead.

Contract 2 says *"All six mk1-consuming verbs … On declared ≠ derived,
chunked input only: one stderr warning"*, and Acceptance says *"Per-surface
golden rows (all six mk verbs …): the warning fires on the mismatch rows and
is ABSENT on their clean twins."* **Measured, that acceptance criterion is
false as written for `repair`**: feed a mismatch row's strings to `mk repair`
unchanged and nothing fires, because nothing decodes (exit 0 above). Contract
2's own repair clause ("fires on its re-verified (blessed) output") already
implies the narrower behaviour — the two sentences disagree, and neither
states the consequence.

The implementer is therefore pushed to one of two bad outcomes, both
compliant with the text as it stands:

1. add a fresh decode on the clean path — which changes `mk repair`'s exit-0
   contract and introduces a new failure mode for the partial-group supply
   that repair deliberately supports; or
2. quietly write the repair golden row with *damaged* input (the corpus
   schema — `canonical_bytecode_hex`, mk1 string set, `declared_csid`,
   `derived_csid`, `expect_mismatch_warning`, `warning_text` — has no damaged
   variant, so the harness would have to damage a string itself), leaving the
   undamaged and partial cases undefined and untested.

**Fix:** state the coverage explicitly — *"on `mk repair` the warning fires
only on the blessed re-verify (a group that was corrected, complete and
reassembled); an already-valid supply (exit 0) and a partial/single-plate
supply (Candidate, exit 5) do not decode and do not warn — the operator sees
it on the `mk decode` the UNVERIFIED advisory already sends them to"* — and
correct the Acceptance clause so the repair row is scoped to the blessed
input rather than "the mismatch rows".

## L2-2 — Bless-path composition is coherent on exit codes, ambiguous in prose — **Important**

Exit-code composition is **clean, no ambiguity**: 5 means "corrections
applied" and is orthogonal to the warning; the Reject path returns 2 with all
output suppressed (so a warning could never accompany a rejected batch); and
the two stderr `warning:` lines **can never co-occur** — Bless has no
UNVERIFIED advisory and Candidate has no decoded card. That is a real
positive result: no stream carries both signals.

> **[Important L2-I2]** What is ambiguous is the *sentence*. On the blessed
> path the operator sees, on stdout, `# Repair report / mk1 chunk 0: 1
> correction at position 27: 'q' -> 'g'` — "I corrected your card, all good"
> — and on stderr, immediately after, *"this key card's stamped chunk-set id
> (12345) was not derived from its content, which computes 94b47."* Nothing
> in the frozen content says **when** the mismatch arose. The natural read is
> *"the repair changed my card's identity"* or *"the correction was wrong"* —
> especially since the correction is to the payload and the id is a header
> field, and since this is the one verb whose whole job is telling the
> operator what it altered. Per W16 (human sentence first, coherent to the
> operator), the repair surface needs one clause placing the mismatch at
> **mint time**, e.g. *"this was minted this way; the repair did not change
> it."*

Acceptance already permits this (*"surface framing may differ"*), so R6 is
not violated by fixing it — but the spec does not **require** it, and once
the corpus rows freeze `warning_text` this is the last moment to bind it.
Note the wording walk (W13–W16) covered the mk decode-side warning, the mint
warning and the merged-cards refusal — **the repair surface was never
walked**. Folds cheaply together with L2-I1 as one paragraph.

## L2-3 — the JSON envelope on the repair path is unaddressed — **Important**

> **[Important L2-I3]** The spec gives `mk verify --json` an additive
> `"chunk_set_id": {declared, derived, matches}` field with `schema_version`
> held at 1, and says **nothing at all** about the other JSON surfaces. For
> `repair` that silence is load-bearing in both directions.

Measured facts the spec must rule on:

- `mk repair --json` emits `RepairJson { schema_version, kind,
  corrected_chunks, repairs }` whose schema is a **cross-CLI contract**:
  *"schema MUST byte-match toolkit's standalone `RepairJson` at
  `mnemonic-toolkit/src/cmd/repair.rs:162-183` (D27 cross-CLI parser reuse).
  Field order is part of the schema"* (`repair.rs:529-534`). An implementer
  reading "verify's envelope gains the field" and generalising would break
  that contract against **another repo**, which nothing in this repo's suite
  tests.
- Its `schema_version` is the **string** `"1"`, while every other mk-cli
  envelope (including repair's own *error* envelope, `error.rs:144`) uses the
  **integer** `1`. Any instruction phrased as "schema_version stays 1" is
  ambiguous on this surface.
- Conversely, if the field is omitted, a `--json` consumer of `mk repair`
  gets **no mismatch signal at all** (the warning is stderr-only) while a
  `--json` consumer of `mk verify` gets one — an asymmetry R6's "same
  warning everywhere" does not resolve for machine surfaces.
- The same gap exists for `mk inspect --json` (`inspect.rs:98`): contract 3
  says inspect *prints* the stamped id unconditionally, and inspect has a
  JSON envelope the spec never mentions, so the id's presence there is
  implementer's choice.

**Fix:** one sentence per surface — recommended: *"`mk repair --json` and
the other JSON envelopes are UNCHANGED this cycle; the mismatch reaches
machine consumers only via `mk verify --json`, because repair's envelope is
a byte-match contract with `mnemonic repair --json` (D27) that this cycle
does not renegotiate. `mk inspect`'s unconditional stamped-id print is
text-mode; its `--json` envelope gains `chunk_set_id` / gains nothing
[choose]."*

## L2-4 — miscorrection never reaches the warning — **Minor**

> **[Minor L2-M1]** The spec lists *"(3) beyond-budget miscorrection"* among
> the sources the warning acts as a tripwire for. Traced through `repair`,
> every miscorrection case is absorbed **before** the warning could fire: a
> miscorrection in the payload region leaves the group complete and
> consistent but fails the 4-byte cross-chunk hash → **Reject, exit 2, all
> output suppressed**; a miscorrection in the header's csid symbols splits
> the group → **Candidate, exit 5 + UNVERIFIED advisory, no decode**; a
> miscorrection that survives both would have to beat the 32-bit cross-chunk
> hash. The warning contributes nothing to miscorrection detection in
> practice. The parenthetical *"(rare; payload damage is largely caught by
> the existing 4-byte cross-chunk hash)"* is hedged rather than wrong, so
> this is a sharpening, not a correction — but as written it credits the
> warning with a tripwire it does not provide.

## L2-5 — "one stderr warning" is singular; two surfaces are plural — **Minor**

> **[Minor L2-M2]** `mk decode/inspect/verify/derive/address` each decode
> exactly one card, so "one stderr warning" is exact there. **`mk repair` is
> batch-capable** (multi-group, folded verdicts) and **md's seat path takes
> many cards**, so both can produce N mismatching cards in one invocation.
> The spec does not say one-warning-per-mismatching-group, nor fix the
> emission order. The declared id inside the warning does identify the card
> (that much is fine), but repair's stdout report is indexed by *chunk
> index*, which the warning never mentions, so a batch operator cannot line
> the two up. **Fix:** "one warning per mismatching group, in the surface's
> existing group order."

## L2-6 — the blessed card is currently discarded — **Nit**

`classify_mk1_set` matches `Ok(_) => GroupVerdict::Bless` (`repair.rs:410`),
so the decoded card the warning needs is thrown away one line after it
exists. Implementation detail, no spec change needed — flagged only so the
implementer plumbs `Ok(card)` out rather than adding a second decode.

---

# Summary table

| id | sev | one line |
| --- | --- | --- |
| L1-I1 | **Important** | The operand is version-dependent and the spec never freezes it; `STANDARD_PATHS` already grew at v0.2.0 (`0x16`), and a future change makes genuine plates warn forever with a "re-mint" remedy. |
| L1-I2 | **Important** | "bare lowercase hex" drops the zero-padding its own cited precedent has; `{:x}` yields a 4-digit token `md --seat` refuses; 3 of the 19 corpus rows carry an id `< 0x10000` and none of the named new rows does. |
| L2-I1 | **Important** | `mk repair` warns only on the blessed path; exit-0 and Candidate supplies never decode, so a mismatch card is silent — and the Acceptance clause requiring the warning on repair's mismatch row is false as written. |
| L2-I2 | **Important** | On the blessed path the frozen wording reads as "the repair changed the id"; the repair surface was never walked and needs one mint-time clause (Acceptance already permits per-surface framing). |
| L2-I3 | **Important** | The spec rules on `verify --json` only; `repair --json` is a byte-match cross-CLI contract (D27) with a *string* `schema_version`, and `inspect --json` is unaddressed. |
| L1-M1 | Minor | "(4) deliberate tamper" invites reading the warning as a tamper control; measured, a substitute card can be ground onto any target id in 0.20 s. |
| L1-M2 | Minor | "A card whose payload does not re-encode to the bytes on the plates is therefore IN scope" is measurably false (construction A2). |
| L2-M1 | Minor | "(3) beyond-budget miscorrection" is absorbed by Reject/Candidate before the warning can fire. |
| L2-M2 | Minor | "one stderr warning" is singular on two batch-capable surfaces (repair, md seat). |
| L2-N1 | Nit | The blessed decode's card is discarded at `repair.rs:410`; plumb `Ok(card)`, do not add a second decode. |

**These two lenses do not close.** Both remain open on the Importants above;
a fold that adds (a) an encoder-freeze clause + a table-pinning row, (b) the
`{:05x}` rendering rule + a leading-zero corpus row, (c) one paragraph
scoping repair's coverage, framing and JSON, closes all five.
