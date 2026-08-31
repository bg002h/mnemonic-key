# R0 review r1 — `design/SPEC_chunk_set_id_verification.md` @ `d6847df`

**Verdict: 4C / 6I / 5M / 3N — NOT GREEN.**

One question answered: is the spec sound, complete and internally consistent as a
behavior contract, and can every acceptance gate actually fail? Scope held to the
artifact; the settled list in the brief was taken as given and not re-derived.

Everything below marked *measured* was run this session against the four trees at
the spec's own baselines (mnemonic-key `9ff8922`+`d6847df`, descriptor-mnemonic
`7eca44b6`, seedhammer `5f02773c`, mnemonic-engrave `1103d9ee`). Commands and
outputs are quoted inline.

---

## Critical

### C1 — Contracts 2–3 name three verbs; **five further surfaces reassemble a card and stay silent**, narrowing W11 ("Same warning everywhere")

**Measured.** Every one of these calls `mk_codec::decode(&refs)` on a chunked set
and is not named by any contract:

| surface | site |
| --- | --- |
| `mk address` | `crates/mk-cli/src/cmd/address.rs:82` |
| `mk derive` | `crates/mk-cli/src/cmd/derive.rs:50` |
| `mk repair` (bless / reject re-verify) | `crates/mk-cli/src/cmd/repair.rs` module doc §, "re-verifies by reassembling through `mk_codec::decode`" |
| `me seal` | `mnemonic-engrave/crates/me-cli/src/seal/record.rs:253` — `('k', _) => mk_codec::decode(&set)` |
| `me sysw` | `mnemonic-engrave/crates/me-cli/src/sysw/record.rs:212` — `('k', _) => mk_codec::decode(&set).is_ok()` |

**Failing scenario, constructed.** W6 records the operator's own restore-
verification instinct verbatim: *"I would probably know an address from wallet or
a wallet id."* So the operator holding a mis-stamped plate runs the verb that
serves that instinct — `mk address` — gets a correct address, exit 0, and **no
warning**, because the only three verbs the spec instruments are ones they had no
reason to run. The same plate through `me seal` is silently sealed into a
payload. The R2 tripwire is absent from exactly the surfaces the walk identified
as the operator's first moves.

Worse for the gate: Acceptance scopes golden rows to "contracts 2–6", so no
assertion in the cycle can ever fail for these five. The hole is invisible to the
suite by construction.

**Remedy direction (advisory).** `crates/mk-cli/src/cmd/mod.rs::read_mk1_strings`
is already the shared intake for all six mk1-consuming subcommands
(decode/inspect/verify/repair/derive/address — stated in `design/FOLLOWUPS.md:122`);
seating the recompute there makes "every mk surface" structural rather than an
enumeration that decays. me-cli's three surfaces need naming individually, or an
explicitly *ruled* exemption per verb — silence by omission is what W11 forbids.

---

### C2 — Contract 8 is not implementable this cycle: **me-cli links mk-codec 0.4.1, which has no derivation at all**

**Measured.**

- `mnemonic-engrave/crates/me-cli/Cargo.toml:39` → `mk-codec = "0.4"`
- `mnemonic-engrave/Cargo.lock` → `mk-codec 0.4.1`, `source = "registry+…crates.io-index"`
- `grep -rn "derive_chunk_set_id" ~/.cargo/registry/src/*/mk-codec-0.4.1/src/` → **zero hits**
- `mk-codec-0.4.1/src/string_layer/pipeline.rs:34-45` → *"Draw a fresh 20-bit `chunk_set_id` from the system CSPRNG"*, `fn fresh_chunk_set_id()`

Contract 8 says `me bundle` "emits the same warning … engrave leg, this cycle."
The function it needs does not exist in the crate me-cli links, and the id it
would compare against is drawn from `OsRng` in that version. Delivering contract
8 therefore requires:

1. a **manifest bump 0.4 → 0.5+** across the release that changed csid semantics
   (`feat(mk-codec 0.5.0)!:` per the recon) — semver-breaking by the crate's own
   marking, not a patch pickup;
2. a **crates.io publish** of mk-codec if contract 1 adds any API — and crate
   publishes are operator-gated: mnemonic-engrave `design/FOLLOWUPS.md` F-424,
   verbatim, *"Crate publishes are operator-gated — never overnight work."*
   F-424 also documents that this exact lag pattern (me-cli on a stale published
   codec) is currently a live, twice-blocked follow-up for md-codec.

Neither appears anywhere in the spec. Nor does mnemonic-engrave's suite: the
Acceptance bullet names `cargo nextest run --locked` for **mnemonic-key and
descriptor-mnemonic only**, so the crate contract 8 changes is never gated — while
its own fixtures are built around the pinned id that will now warn
(`me-cli/src/bundle.rs:725-732` re-encodes a card with
`encode_with_chunk_set_id(&card, 0x12345)`; the golden
`tests/vectors/bundle-md1-mk1.json` carries `0x12345` at three lines).

**Remedy direction.** Either state the publish + bump as an explicit in-cycle
precondition with the operator gate named, and add mnemonic-engrave to the
Acceptance suite list — or move contract 8 to the same post-cycle burndown as the
device leg (W12 already established that scheduling shape), leaving R6's engrave
leg documented rather than half-promised.

---

### C3 — R5's three situations are **not exhaustive**; the measured merged-cards case falls through every branch

**Measured, live this session.** Two 2-chunk cards, both pinned to `0x12345`:

```
mk encode --xpub xpub661MyMwAqRbcFtXgS5sYJ… --origin-fingerprint 3442193e \
  --origin-path m --policy-id-stub 0badf00d --chunk-set-id 0x12345   # plate A
mk encode --xpub xpub661MyMwAqRbcFW31YEwpk… --origin-fingerprint bd16bee5 \
  --origin-path m --policy-id-stub 0badf00d --chunk-set-id 0x12345   # plate B
```

Then supply **chunk 0 of A + chunk 1 of B**:

```
$ mk decode <A0> <B1>
error: cross-chunk integrity hash mismatch
exit=1
```

Header forensics for that input: received = 2, declared = 2 → **not situation
(a)** (which requires received < declared). No duplicate chunk index, totals
agree → **not situation (b)**. The group does **not** reassemble → **not
situation (c)**. Nothing in contract 6 fires. This is the merged-cards case the
rewrite exists to name, in its most dangerous form — two real plates' halves,
past every header check.

Three further uncovered exits from `reassemble_from_chunks`
(`crates/mk-codec/src/string_layer/chunk.rs`):

- `:250` — `chunk_index {idx} >= total_chunks {total}`
- `:264` — `Error::MixedHeaderTypes`
- post-reassembly bytecode decode errors (chunk layer succeeds, `decode_bytecode`
  fails) — the group "reassembles" yet (c)'s warning branch does not apply

Also note an ordering fact the classifier must not inherit: mk-codec checks
`chunks.len() != total` **before** the duplicate-index and totals-disagree checks
(`chunk.rs:216-222`), so `{e}` alone can never distinguish (a) from (b) — measured,
`mk decode A0 A1 B0 B1` → *"received 4 chunks, header declares total_chunks = 2"*
while `mk decode A0 B0` → *"duplicate chunk_index 0"*. md-cli must compute its own
forensics, which the spec assumes ("header forensics the tool already possesses")
but which `seat::input::group_key_of` currently discards — it returns only
`GroupId`, dropping `chunk_index` and `total_chunks`
(`descriptor-mnemonic/crates/md-cli/src/seat/input.rs:158-161`).

Because Acceptance simultaneously requires that *"the retired message string
appears in none of them"*, these inputs get either no message or an unspecified
one. That is the original defect — one outcome, no correct remedy — reissued in
new wording, which is exactly the failure mode the brief names.

**Remedy direction.** Add an explicit fourth, terminal *otherwise* situation that
carries the codec error verbatim plus a neutral remedy ("these pieces carry one
id but do not form one card; re-scan each plate separately"), and state the
classifier's **evaluation order** so (a)/(b)/(otherwise) are provably disjoint
and total over the error set at `chunk.rs:216-286`.

---

### C4 — **Every chunked vector in the corpus the spec proposes to extend is non-derived.** There is no clean row, and fixing that contradicts the Acceptance

**Measured.** `mk vectors` → `schema: 2`, `family_token: "mk-codec 0.5"`, 41
vectors, of which 19 are chunked and carry `canonical_bytecode_hex`. Recomputing
`top20(SHA-256(canonical_bytecode))` for each and comparing to
`input.chunk_set_id`:

```
chunked-with-bytecode: clean 0   mismatch 19
V1_bip48_mainnet_1_stub_with_fp    declared 0x12345  derived 0x83bb2
V2_bip84_mainnet_1_stub_with_fp    declared 0x23456  derived 0xf479a
V3_bip48_testnet_1_stub_with_fp    declared 0x34567  derived 0xc8ea7
…19 of 19
```

The derivation used is confirmed against the live encoder, not assumed: minting
V1's inputs **unpinned** yields `mk1qpswajp…`, whose 8-symbol chunked header
(`header.rs:23`, symbols `[version, type, csid×4, total-1, index]`) decodes to
csid symbols `[16,14,29,18]` = `0x83bb2` — byte-identical to the computed value.

**Consequences, all unaddressed:**

1. Extending the corpus "per chunked vector" with `derived_csid`, `declared_csid`
   and `expect_mismatch_warning` as the spec directs sets
   `expect_mismatch_warning = true` on **all 19**. Acceptance's other half —
   *"ABSENT on the unpinned twins"* — has **no vectors to run against**. The clean
   half of the golden gate cannot be constructed from the corpus as it stands.
2. The only way to get clean rows is to **re-mint the 19 golden string sets with
   derived ids**, which churns the pinned corpus SHA
   (`crates/mk-codec/tests/vectors.rs:41`, `V0_1_SHA256 = "c3a13b67…"`, enforced by
   `vector_file_sha256_matches_pin` at `:110`), plus `clean_count >= 18` at `:169`
   and every downstream consumer of those strings. That is squarely a change to
   existing csid tests — which the same Acceptance bullet forbids ("**zero changes
   to existing csid tests**").
3. The three seed rows in the spec's table are **not corpus rows** — they are
   ad-hoc mints from the walk. So the artifact's stated "executable anchor" is
   disjoint from the corpus the Acceptance actually runs.

**Remedy direction.** Rule the corpus question explicitly: either (i) regenerate
with derived ids, re-pin `V0_1_SHA256`, and replace the "zero changes" clause with
a named, enumerated list of the intended golden churn (this also collides with the
open FOLLOWUPS nit at `design/FOLLOWUPS.md:481`, which already wants a V19 re-pin —
batch them); or (ii) leave the 19 as the *mismatch* half and add a small set of new
clean chunked vectors alongside, stating that the legacy 19 are pinned-by-design.
Either way the spec must say which, because the two produce different suites.

---

## Important

### I1 — "Normative-by-vector" points at rows that contain no strings; the byte-exact half of the gate cannot fail

The spec states: *"Warning text is normative-by-vector: the exact strings live in
the acceptance rows below, not in prose."* The rows below are a four-column table
— `card | declared | derived | warn` — with **no text column**, and the Acceptance
adds only `derived_csid`, `declared_csid`, `expect_mismatch_warning` to the corpus
schema. So "each warning fires **byte-exact** on its vector" has no referent.

**Failing scenario.** The implementer writes any sentence, freezes it as the
golden, and the assertion passes forever — a gate whose expected value is authored
by the thing under test. (The mutation gate still catches *absence*; only the
wording half is vacuous.) This also silently voids the "frozen only when the rows
are" clause: the rows can never freeze wording they cannot hold.

**Remedy.** Add a `warning_text` field to the corpus row, or drop "byte-exact" and
assert on what R6 parity actually needs — the `(declared, derived)` pair and the
remedy sentence, which the me-bundle parity bullet already isolates as the
non-negotiable content.

---

### I2 — Contract 6 cannot be implemented under Acceptance's "zero changes to existing csid tests"

**Measured** in descriptor-mnemonic, two live assertions on the retired string:

- `crates/md-cli/src/seat/input.rs:310-313` — `assert!(msg.contains("do not reassemble"), "the refusal is the reassembly one, not a seating one: {msg}")`
- `crates/md-cli/tests/seating_vectors.rs:846` — `assert!(e.contains("do not reassemble"), "{e}")`, in the named vector `v_collide_reaches_the_command`, which also asserts `e.contains("chunk-set 12345")` at `:845`

Plus two doc sites that the rewrite falsifies without touching:
`seat/input.rs:1-25` (module doc: *"refuse exactly like this — re-mint one of
them"*) and `seat/input.rs:106`, and `tests/seating_vectors.rs:107`.

Contract 6 replaces that message. The Acceptance forbids changing the tests that
pin it. An implementer reading both literally has no legal move.

**Remedy.** Scope the "zero changes" clause to the two *named* guarantees it
actually means (`an_explicit_chunk_set_id_still_wins`,
`canonical_payload_is_chunk_set_id_invariant`, already listed separately), and
enumerate the tests and doc blocks contract 6 is *expected* to rewrite — the
"a diff falsifies text it never touches" class applies to all four sites above.

---

### I3 — The comparison's left operand is undefined, and R6 parity depends on which one is chosen

"Recompute the id from content" admits two implementations, and both are reachable
today:

- **(a)** `derive_chunk_set_id(reassembled_bytecode)` — the bytes actually on the
  plates. Only available inside mk-codec's decode path.
- **(b)** `derive_chunk_set_id(&encode_bytecode(&decoded_card))` — a re-encode of
  the decoded card. Available to md-cli **right now with zero mk-codec change**:
  both `derive_chunk_set_id` (`crates/mk-codec/src/lib.rs:52`) and
  `encode_bytecode` (`crates/mk-codec/src/bytecode/mod.rs:28`) are already `pub`,
  and md-cli pins `mk-codec = "0.5"`.

They are **not equivalent**, and they diverge precisely on R2's primary
justification. A foreign encoder whose *bytecode canonicalization* drifts (the
F-212 shape, W10) mints bytecode `B'` and stamps `derive(B')`. Route (a) computes
`derive(B')`, finds it equal to the stamp, and **stays silent** — the drift
tripwire misses. Route (b) computes `derive(encode_bytecode(decode(B')))` =
`derive(B)` ≠ stamp, and **fires**.

Contract 1's parenthetical — *"Mechanism — new API vs. enriched return — is the
implementer's choice at plan time"* — explicitly licenses four surfaces to choose
differently. Under R6 ("same warning everywhere") that means the same plate warns
in `md descriptor` and not in `mk decode`, or vice versa, with no test able to
notice because the corpus rows are per-card, not per-surface.

**Remedy.** Pin the operand in the spec: name which byte string is hashed, and
say whether a card that round-trips to different canonical bytes is in or out of
scope for the warning. This is also what decides whether contract 1 is needed at
all — route (b) makes contracts 5 and 6 implementable with no mk-codec change and
no publish, which is materially cheaper than the spec's current shape.

---

### I4 — Contract 7's "the Go `mk/` port consumes the same vector corpus" describes a mechanism that does not exist

**Measured** in `seedhammer/mk/`: four files (`encode.go`, `encode_test.go`,
`mk.go`, `mk_test.go`). **No `testdata/` directory, no `go:embed`, no JSON
reader.** `mk_test.go` hand-transcribes vector strings into a `parityVectors`
table and a `TestDecodeNegative` case list; every transcribed chunked string
begins `mk1qpzg69p…` — csid `0x12345`, i.e. pre-0.5 non-derived, as the recon
already flagged for the 7 parity vectors.

So "consumes the same vector corpus and must reproduce every derived id" is not a
convergence step but a **new cross-repo vendoring + lockstep mechanism**. The
constellation's precedent for exactly this (the descriptor seam) required a
vendored vector file, a dedicated seam test, and an unmerged branch, and is still
tracked as `F-425` in mnemonic-engrave. The spec allocates it one clause and
declares R1 means "no normative codec change locksteps, so the vectors are the
in-cycle fork surface" — which understates the fork work rather than removing it.

Two further unstated consequences: adding corpus fields changes `schema` (today
`2`), and `GENERATOR_FAMILY` "rolls on minor-version bumps" per
`gen_mk_vectors.rs:1-8` — any fork-side reader must accept both.

**Remedy.** Either specify the vendoring (file path in the fork, who regenerates,
what gates the SHA — mirroring the F-425 seam pattern), or scope contract 7 to
what is genuinely in reach this cycle: a Go-side unit assertion that
`top20(sha256(bytecode))` reproduces a handful of pinned rows, with the full
corpus ingestion filed as its own follow-up alongside the device leg.

---

### I5 — Contract 3 reports into `mk verify`'s stdout / versioned JSON envelope, but Acceptance calls contracts 2–6 "golden-**stderr** rows"

**Measured.** `crates/mk-cli/src/cmd/verify.rs:190-207` — `emit_ok` writes with
`println!` (stdout) in text mode and, under `--json`, emits
`{"schema_version": 1, "ok": true, "chunks": …, "policy_id_stubs": […]}`. That
envelope is a documented cross-CLI contract (`cmd/repair.rs:35`: *"`schema_version`,
`kind`, `corrected_chunks`, `repairs` … so cross-CLI"*), shared by decode, derive,
encode, repair and the error path (`error.rs:144`).

Three unresolved questions the Acceptance cannot express:

1. Does verify's JSON gain a mismatch field, and does `schema_version` roll?
   (Contracts 2 and 4 sensibly use stderr, which keeps `--json` stdout clean;
   contract 3 breaks that pattern without saying so.)
2. Is an English sentence on stderr "the same warning content" as a JSON boolean?
   R6 parity is only spelled out for me bundle.
3. Contract 6's outputs are **exit-1 refusals**, not warnings — they have no
   "unpinned twin" to be ABSENT on, so the bullet's second clause is meaningless
   for them.

As written, the Acceptance is unrunnable for three of the five contracts it names
without a channel ruling first.

---

### I6 — The mismatch-channel enumeration omits the largest class: **every card minted before mk-codec 0.5.0 carries a CSPRNG id**

The spec asserts *"A clean mismatch is only ever MINTED"* and lists four channels
(pin leakage, encoder drift, beyond-budget miscorrection, tamper). There is a
fifth, and it is the biggest: **vintage.**

**Measured.** mk-codec 0.4.1 `string_layer/pipeline.rs:34-45` draws the id from
`OsRng` (`fresh_chunk_set_id`); derivation arrives only at 0.5.0 (2026-08-14).
Any plate minted by a 0.4-vintage tool has declared ≠ derived with probability
1 − 2⁻²⁰. In-repo artifacts of exactly this vintage exist today: the fork's seven
`parityVectors` and me-cli's `bundle-md1-mk1.json`.

**Failing scenario.** An operator scans a correctly minted, correctly engraved,
pre-0.5 plate. Contract 2's draft tells them the card *"was not minted normally"*
and to expect confusing diagnostics *"until it is re-minted"* — an accusation plus
a physically destructive remedy, for a plate that is fine. That inverts the
walk's own stated principle (*"None of these are the scanning operator's fault;
refusal would strand a perfectly restorable wallet"*).

**Remedy.** State the vintage boundary as fact (mk-codec ≥ 0.5.0 derives; earlier
mints are legitimately random) and re-word the warning as an observation — "this
card's stamped id was not derived from its content" — with the re-mint advice made
conditional rather than imperative. If the operator can rule that **no** pre-0.5
plate was ever engraved, this drops to Minor; that ruling is not in the walk and
cannot be assumed by a reviewer.

---

## Minor

**M1 — `mk vectors` cannot regenerate anything.** The spec: *"Corpus regeneration
happens at implementation time via `mk vectors`."* Measured: `cmd/vectors.rs:1-25`
is a read-only printer re-exporting the `include_str!`-baked
`mk_codec::test_vectors::V0_1_JSON`. The generator is a separate feature-gated
binary: `cargo run --bin gen_mk_vectors --features gen-vectors -- --output
crates/mk-codec/src/test_vectors/v0.1.json` (`gen_mk_vectors.rs:10-14`).

**M2 — The mutation gate names one comparison where there will be four.**
*"removing the recompute comparison must fail the pinned rows."* After C1/I3 there
are independent comparisons in mk-cli, md-cli, me-cli and the Go port; deleting
md-cli's would not fail mk-codec's rows, so the gate as stated passes while three
of four surfaces are unprotected. Name a per-surface mutation, and — per this
repo's own lesson — require evidence the mutated line *ran*, not merely that the
edit landed.

**M3 — Situation (a) asserts a single card it has not established.** Draft
wording: *"this card says it has 2 pieces; you supplied 1."* Counterexample within
the taxonomy: chunk 0 of a 3-chunk card A + chunk 1 of a 3-chunk card B, both
pinned to one id → received 2 < declared 3, distinct indexes, totals agree → (a)
fires and names one card that does not exist. The remedy ("scan the missing
piece") still advances the operator, so this is wording rather than a taxonomy
hole — but W1's complaint was precisely messages asserting more than they measured.
Prefer "the pieces carrying this id say there should be 3; you supplied 2."

**M4 — `mk inspect` never prints the stamped id.** Measured, full inspect output
for a 2-chunk card: xpub, origin_fingerprint, xpub_fingerprint, origin_path,
components, policy_id_stubs, chunks, per-chunk BCH variants — **no
`chunk_set_id`**. The R2 warning would be the only place the operator ever sees
the value, with nothing to cross-check it against, and W7 already recorded the
operator's confusion about id vocabulary. Consider printing it unconditionally on
inspect, matched or not.

**M5 — "it is already computed on the unpinned path" reads as "no work needed",
and isn't.** True of the codec (`pipeline.rs:80`, `None => derive_chunk_set_id(bytecode)`),
but the pinned arm skips it (`:73-79`) and `mk encode` calls
`encode_with_chunk_set_id` (`cmd/encode.rs:355`), which returns only strings. mk-cli
must derive it itself via `encode_bytecode` + `derive_chunk_set_id` — both public,
so cheap, but it is real work and the parenthetical hides it.

---

## Nits

**N1 — Ruling range and section heading are stale.** The Status line cites
"rulings W1–W10"; the walk closed at **W12**, and the spec body relies on W11
(contract 7's R6 clause) and W12 (the post-cycle scheduling). The heading "The
ruling (operator, 2026-08-31, walk steps 4–5)" covers a section that includes R6,
which is walk step 6.

**N2 — One value, two renderings, one sentence.** Contract 2's draft: *"stamped
chunk-set id (0x12345) …"* then *"tools … will call it 12345."* Pick one form and
use it in both clauses; the whole point of the sentence is that the operator can
match the string against a diagnostic.

**N3 — The point-of-use gloss list drops the terms W1 actually named.** W1's
operator quote is *"What is a keycard?"*, and the walk lists "key card",
"chunk-set id", "re-mint", "pinned". Contract 6 promises glosses for "card",
"chunk", "stamped id" — dropping **"key card"**, **"re-mint"** and **"pinned"**,
all three of which appear in the spec's own draft warnings ("re-minted",
"pins … in place of").

---

## What was checked and found sound (so a later round need not re-derive it)

- The cross-format asymmetry claim holds: md-codec verifies unconditionally, mk
  does not. Confirmed against the recon; not re-measured.
- `md descriptor` and `md address` are the **only** two md-cli verbs accepting
  `--from-mk1` (measured: `md-cli/src/main.rs:296` and `:528` arg groups).
  Contract 5 names both; no md surface is missing.
- `mk encode` is the **only** non-test caller of `encode_with_chunk_set_id` in the
  constellation (measured across all three Rust trees). Contract 4 covers the
  whole mint surface.
- The fork's Go encoder already derives correctly —
  `seedhammer/mk/encode.go:329-334`, `top20` = `h[0]<<12 | h[1]<<4 | h[2]>>4`,
  comment *"NO CSPRNG"* — so contract 7's *encoder* convergence is already true;
  only the vector-ingestion mechanism (I4) is missing.
- Both cited follow-up slugs exist and are well-formed:
  `design/FOLLOWUPS.md:529` (`device-csid-mismatch-warning`) and `:544`
  (`mk-decode-silent-correction-reporting`).
- `mk encode --chunk-set-id` mints in silence today — reproduced: stderr carried
  only *"note: stdout is watch-only…"*. R3's premise holds.
- The derivation formula is reproducible end to end and the walk's seed rows are
  consistent with it (verified independently via header-symbol decode, above).

## Lens closure

This round ran a **contract-completeness + testability** lens. It is not closed
for other questions. Not yet asked of this spec: an adversarial "construct a plate
that defeats the warning" pass, a failure-states pass on the warning's interaction
with `mk repair`'s exit-5 bless path, and a live journey walk of the *post-fix*
messages with the operator (which is what produced W1–W5 in the first place, and
which contract 6's new wording has not been subjected to at all).
