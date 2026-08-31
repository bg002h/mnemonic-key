# R0 — IMPLEMENTATION_PLAN_chunk_set_id_verification.md (r2, fold-check)

**Artifact:** `design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md` @ `bdd2349`
**Prior:** `design/agent-reports/R0-csid-plan-r1.md` @ `8e53fd1` (0C/2I/3M/2N)
**Fold commit:** `bdd2349` — `git diff 8e53fd1..bdd2349` = 33 insertions / 11 deletions,
all within P0/P1/P3/P4 of this one file.
**Scope:** tight fold-check only — (1) does bdd2349 discharge each r1 finding,
(2) did it introduce a new defect/contradiction. NOT a fresh audit; r1's
phase-order/gate-can-fail/mutation-isolation/Rust-primary/no-build-gate
verdicts stand, unre-derived.

**Verdict: 0 Critical / 0 Important / 1 Minor (residual, tier-kept) + 1 NEW
Minor (internal contradiction) / 2 Nit (closed). R0 CLOSES** (Minors/Nits do
not gate per project severity rules).

---

## Per-finding disposition

### I1 — verify text-mode stdout report — **FIXED**
P1 gains (plan lines 80-85): *"`mk verify` reports the mismatch in BOTH modes
(spec contract 4, plan r1 I1): text mode's `emit_ok` stdout verdict
(`crates/mk-cli/src/cmd/verify.rs:190`, today prints `OK: mk1 string(s)
decode cleanly`) must carry the pair + remedy on a mismatch — a P1 RED
asserts verify's STDOUT changes, not only stderr; `--json` gains the
additive `chunk_set_id` object..."* — machine-checked: `verify.rs:190` is
`fn emit_ok(...)` (confirmed by direct read), a more precise citation than
r1's own `:202` (the string literal inside it). Both remedy halves present:
a RED assertion on stdout, and an IMPL target (`emit_ok`'s text branch,
named function + today's exact string). Resolves r1's open question
("stderr-warning-and-stdout-verdict, or stdout-only") to **both fire** —
verify stays in P1's per-verb stderr list *and* gains the stdout assertion.
Cosmetic-only nit: the RED/IMPL split blurs into one paragraph rather than
the phase's usual two, but substance is unambiguous.

### I2 — classification-order acceptance row — **FIXED**
P3 RED gains (lines 104-107): *"plus a classification-ORDER row (spec
Acceptance, plan r1 I2): a supply matching TWO arms' raw predicates (e.g.
incomplete AND duplicate-index) must land in the EARLIER arm — this is what
catches an arm-precedence bug that per-situation rows miss."* Matches r1's
remedy verbatim ("arms 1&2 raw predicates ... lands in arm 1").

### M1 — corpus access mechanism for md-cli/Go — **PARTIAL** (tier kept: Minor)
P0 gains (lines 60-64): *"Access mechanism (plan r1 M1): bake the file via
`include_str!` into a `test_vectors::csid_ext` module mirroring the existing
`V0_1_JSON` pattern, so mk-cli/md-cli tests and the Go parity test all read
the same pinned bytes..."*

- **mk-cli (P1): sound.** Machine-checked `crates/mk-codec/src/test_vectors/mod.rs:16`:
  `pub const V0_1_JSON: &str = include_str!("v0.1.json");`, and `lib.rs:42:
  pub mod test_vectors;` — unconditionally public, not `cfg(test)`-gated.
  Mirroring this for `csid_ext` gives mk-cli (same workspace) a real,
  already-proven access path.
- **Go (P4): the claim is false, and contradicts the plan's own unchanged
  text three lines later.** P4 (untouched by this diff) still reads:
  *"a Go unit test asserting `top20(sha256(bytecode))` reproduces the
  extension corpus's clean pinned rows (**hand-carried** like existing
  `parityVectors`)."* Go cannot consume a Rust `include_str!` constant under
  any mechanism — "hand-carried" (manual constant duplication) is the only
  route, exactly as P4 already said before this fold. P0's new sentence
  claiming Go "read[s] the same pinned bytes" is incompatible with P4's own
  wording in the same document.
- **md-cli (P3): unaddressed/moot as stated.** Because `test_vectors` is a
  `pub mod`, a *published* mk-codec crate technically could expose
  `csid_ext` to md-cli's separate-repo dependency (`mk-codec = "0.5"` —
  r1-verified) — but only after an actual crates.io publish bumping that
  version, a step P0 does not mention and which sits against the plan's own
  "no mk-codec change, no publish" framing (scoped in the plan to the derive
  functions, but the corpus module is new surface P0 adds to the same
  crate). More directly: it's moot regardless, because P3's RED bullet
  (lines 102-110, unchanged by this fold) asserts only the hardcoded W15/W16
  wording elements — it never asserts content against the corpus's
  `warning_text`/`derived_csid` fields the way P1's RED explicitly does
  ("content asserts the corpus `warning_text`", line 73). Nothing in P3
  reads `csid_ext` at all in the plan as written.

Net: M1's remedy ("state it, or R6 parity rests on an unstated, drift-prone
hand copy") is satisfied for the one consumer (mk-cli) where a mechanism was
needed and is real; for the other two named consumers the fold asserts a
mechanism that is either false (Go) or unwired/unused in the phase that
would need it (md-cli). Non-blocking (P4's own text is unambiguous on its
own and would correctly override the new P0 sentence for an implementer who
reads that far), but it is a genuine internal contradiction — see NEW below.

### M2 — "shared decode point" wording — **FIXED**, and does not reintroduce
### the decay risk the review brief asked about
P1 IMPL now reads (lines 74-79): *"compute `derived` per-verb at each verb's
own decode call (NOT a single shared mutation of `read_mk1_strings`, which
only reads strings and does not decode — r1 M2 wants per-surface comparisons
that are independently deletable...)"* — drops the repair-hazardous "shared
decode point" phrasing r1 flagged, keeps only the spec-endorsed route.

**New-defect check asked for explicitly:** does per-verb-at-chokepoint
contradict r1's shared-intake fact or the spec's "structural, not an
enumeration that decays" language (spec `8e53fd1` line 136-137)? **No.**
Machine-checked the spec text directly: *"the recompute seats at that
chokepoint (**or equivalently at each verb's decode call**) so 'every mk
surface' is structural, not an enumeration that decays (r1 C1)."* The spec
itself names per-verb-decode-call as the other structural option, not a
decayable one — the fold picked the spec's own second-named route. Coherent.

### M3 — P4 checkout disambiguation — **FIXED**
P4 gains (lines 125-128): names the canonical target
(`/scratch/code/shibboleth/seedhammer`, baseline `5f02773c`) explicitly, and
excludes `seedhammer-corpus-sync`, `wt-s5-skeptic-copy`, `seedhammer-ref-v1.4.2`
by name. Machine-checked: standalone checkout HEAD is `5f02773` (matches);
the `third_party/seedhammer` submodule (the specific confusable r1 named) is
still at a *different* commit, `713aee2e` (`git submodule status`,
mnemonic-engrave) — confirming the risk r1 flagged is real and current. The
fold's exclusion list does not name the submodule specifically (it names
three other scratch copies instead, one of which — `seedhammer-ref-v1.4.2`
— doesn't even contain `mk/encode.go`, machine-checked). This is a residual
gap in the *exclusion* list, but non-blocking: the remedy's core ask ("name
the target path") is satisfied unambiguously by the positive instruction,
which does not depend on the exclusion list being exhaustive.

### N1 — production emission site — **FIXED**
P3 IMPL gains `seat/input.rs:206` as "the actual R5 rewrite target."
Machine-checked at descriptor-mnemonic `044e33d4` (plan's pinned baseline,
current HEAD): the `map_err` block building the retired "re-mint one of
them" string spans lines 203-209; line 206 falls inside it. Citation holds.

### N2 — mk_codec/md_codec name collision — **FIXED**
P3 IMPL gains: *"Use `mk_codec::derive_chunk_set_id` explicitly (plan r1 N2
— `md_codec` exports a same-named function...)."* Matches remedy verbatim.

---

## New-defect / contradiction check (explicit asks)

1. **M2 vs. r1's shared-intake fact / spec's anti-decay language:** no
   contradiction — spec line 136-137 explicitly sanctions per-verb-decode-call
   as equally structural (quoted above). Coherent.
2. **I1's `emit_ok` change vs. verify's exit code / `--json` contract:** no
   risk. The fold states *"NO other JSON envelope changes"* explicitly, and
   P1's RED preamble ("exit code unchanged") already scopes the whole phase,
   verify included — the new stdout assertion doesn't touch exit status or
   add JSON fields beyond the one already scoped in I1's own text.
3. **NEW (Minor) — M1's fold text contradicts P4's unchanged text.** P0 (new)
   claims *"mk-cli/md-cli tests and the Go parity test all read the same
   pinned bytes"*; P4 (unchanged, three paragraphs later) still says the Go
   rows are *"hand-carried like existing `parityVectors`"* — hand-carrying
   and reading-the-same-bytes are mutually exclusive descriptions of the same
   mechanism. Non-blocking (P4's own explicit instruction is what an
   implementer would follow; it doesn't depend on the P0 sentence), filed
   as tier-Minor since it's a plan-clarity defect, not a functional one — no
   phase's RED/IMPL/MUTATION content actually depends on the false claim
   being true.
4. **Double-assignment check:** none found. Each edited clause (I1, I2, M1,
   M2, M3, N1, N2) sits in exactly one phase; no phase's responsibility was
   duplicated or reassigned elsewhere in the document.

---

## Verdict counts

| Tier | r1 | r2 disposition |
|---|---|---|
| Critical | 0 | 0 |
| Important | 2 (I1, I2) | 0 — both FIXED |
| Minor | 3 (M1, M2, M3) | 1 residual (M1 PARTIAL, tier kept) + 1 NEW (P0/P4 contradiction) — M2, M3 FIXED |
| Nit | 2 (N1, N2) | 0 — both FIXED |

**0 Critical / 0 Important → R0 CLOSES.** The two residual Minor items
(M1's md-cli/Go gap, and the new P0-vs-P4 Go-access contradiction it
produced) are recorded for the implementer's benefit but do not gate: P4's
own unchanged "hand-carried" instruction is unambiguous on its own, and no
phase's test content depends on the false "same pinned bytes" claim for
md-cli or Go.
