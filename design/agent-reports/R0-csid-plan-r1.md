# R0 — IMPLEMENTATION_PLAN_chunk_set_id_verification.md (r1)

**Artifact:** `design/IMPLEMENTATION_PLAN_chunk_set_id_verification.md` @ `8e53fd1`
**Spec:** `design/SPEC_chunk_set_id_verification.md` @ `8e53fd1` (== `fcf3971`; `git diff` empty — GREEN 4 lenses)
**Question:** does the plan correctly + completely implement the GREEN spec, with a workable phase order and gates that can actually fail?
**Verdict: 0 Critical / 2 Important / 3 Minor / 2 Nit — DOES NOT CLOSE.**

Reviewer, note what is already machine-verified (do not re-derive): STANDARD_PATHS = **14** entries
(7 mainnet 0x01–0x07 + 7 testnet 0x11–0x17; `bytecode/path.rs:38-55`) → plan's "14-entry table" row is
correct. descriptor-mnemonic `seat/` and `tests/seating_vectors.rs` did **not** move
`7eca44b6..044e33d4` (`git log` empty) → plan's staleness note holds. All P3 churn sites exist at
`044e33d4`: `seat/input.rs:310-313` assert, `seating_vectors.rs:845-846` asserts (in
`v_collide_reaches_the_command` @ `:835`), doc `seat/input.rs:106`, doc `seating_vectors.rs:107`.
mk-codec is `0.5.0`; `encode_with_chunk_set_id` at `encode.rs:355` (pinned arm) confirms P2's
"pinned arm skips derivation". md-cli depends on `mk-codec = "0.5"` and already imports `mk_codec`
(`seat/input.rs:203`) → P3 can call the public pair, no dependency blocker. Standalone
`/scratch/code/shibboleth/seedhammer` is at `5f02773c` (P4's baseline); Go `top20` = top-20-bits of
`sha256(bytecode)`, "NO CSPRNG" (`mk/encode.go:330`), `parityVectors` in `mk/{mk,encode}_test.go` →
P4 claims accurate. Plan has **no** ```rust fences → "no scratch-crate build gate applies" is sound.

---

## Important

### I1 — `mk verify` text-mode STDOUT report (spec contract 4) is unassigned in the plan
**Spec contract 4 (line 161-163):** *"mk verify: text mode reports the mismatch in verify's own
stdout format (content: the pair + remedy). --json gains an optional additive field ..."* — TWO
obligations for verify text mode: (a) the mismatch appears in verify's **stdout verdict**, (b) the
`--json` additive field.

**Plan coverage:** P1 IMPL assigns only (b) ("mk verify --json gains the additive chunk_set_id
object"). P1 RED buckets verify into the generic "golden-**stderr** tests (one per verb:
...verify...)". Nothing in the plan assigns contract-4(a) — verify's **stdout** verdict reporting the
mismatch — to any RED test or IMPL step.

**Concrete failure:** implementer wires the generic stderr warning + the json field and ships. Run
`mk verify` (text mode) on the pinned `12345/ef12f` row: stderr carries the warning, but verify's
stdout verdict still prints `OK: mk1 string(s) decode cleanly` (`verify.rs:202`, `emit_ok`) — a bare
"OK" on stdout while stderr contradicts it, violating contract 4(a). No plan test asserts verify's
stdout, so the omission passes green.

**Remedy:** add a P1 RED row asserting verify's **text-mode stdout verdict** reflects the mismatch
(the (declared,derived) pair + remedy), and an IMPL bullet threading the recompute into `emit_ok`'s
text branch. Resolve whether verify emits stderr-warning-and-stdout-verdict or stdout-verdict-only
(R6 rendering is per-surface) — the plan currently does neither for stdout.

### I2 — the classification-ORDER acceptance row (spec Acceptance, contract 7) is not in P3
**Spec Acceptance (line 280-281):** *"classification order is asserted by a row that satisfies two
situations' raw predicates and must land in the earlier one."* This is a distinct acceptance bullet
from "each of the four situations has a vector."

**Plan coverage:** P3 RED requires "the four-situation classification (r2 C3 — arm 3
unconditioned/total), each with a vector" — i.e. one vector per situation. It does **not** assign the
order-precedence row (a supply matching two arms' raw predicates, pinned to the earlier arm).

**Concrete failure:** a supply that is BOTH incomplete (`received < declared`, arm 2) AND carries a
duplicate chunk_index (arm 1) must classify as arm 1 (*merged cards*). A classifier that tests arm 2
first misclassifies it as *incomplete scan* and emits the wrong remedy ("scan the missing piece")
for a two-card collision. With only one-vector-per-situation, every situation still has a passing
row, so the precedence bug ships green. Precedence is exactly the silent class the ordered taxonomy
(r2 C3) exists to pin.

**Remedy:** add one P3 RED row satisfying arms 1&2 raw predicates and asserting it lands in arm 1
(and, ideally, one arm 2/3 boundary row), per the spec acceptance bullet.

---

## Minor

### M1 — P3/P4 access to the extension-corpus expected values is unspecified
The extension corpus (`csid_ext_v0.1.json`) is generated into **mk-codec**'s src tree
(`crates/mk-codec/src/test_vectors/`, per `gen_mk_vectors.rs:1113`). mk-cli (same repo) can read it;
**md-cli (descriptor-mnemonic) and the Go fork are separate repos** and Cargo/Go deps do not expose
another crate's `src/test_vectors/*.json` as test data. Spec makes the Go mechanism explicit
("hand-carried like existing parityVectors", contract 8) but the **plan is silent on how P3's md-cli
tests obtain `warning_text`/`derived_csid`** for the R6 content assertion. Only viable route today is
hand-carry (JSON ingestion is the post-cycle `go-mk-vector-corpus-ingestion` followup). State it, or
R6 parity across surfaces rests on an unstated, drift-prone hand copy.

### M2 — "shared decode point feeding read_mk1_strings' consumers" is imprecise and repair-hazardous
P1 IMPL: "compute derived at the shared decode point feeding read_mk1_strings' consumers (or each
verb's decode call)". `read_mk1_strings` (`cmd/mod.rs:212`) is **string intake only** — it does not
decode; there is no single shared decode point, and repair decodes specially (blessed re-verify
only, spec line 142-156). Taken literally, a "shared decode" would make repair warn on intake,
contradicting repair's blessed-path-only rule. The parenthetical "(or each verb's decode call)" — the
route the spec actually endorses (line 138-140) — is the correct one; the leading phrasing should be
dropped so an implementer doesn't build the eager-shared-decode variant. (P1 defers repair to P2, so
no live repair regression, but the wording invites a wrong seam.)

### M3 — P4 does not disambiguate WHICH seedhammer working copy it edits
Four `mk/encode.go` checkouts exist (`/scratch/code/shibboleth/seedhammer`,
`.../seedhammer-corpus-sync`, `mnemonic-engrave/third_party/seedhammer` @ `713aee2e`, and two under
`_experiment/`). The plan's baseline `5f02773c` matches the standalone `.../seedhammer` HEAD, but the
mnemonic-engrave submodule is at a **different** commit (`713aee2e`). Name the target path in P4 so
the implementer edits the fork the baseline pins, not the submodule.

---

## Nit

### N1 — the production refusal emission site is omitted from the enumerated churn list
The retired message *"Two DIFFERENT cards pinned … re-mint one of them"* is **emitted in production at
`seat/input.rs:206`** (the `mk_codec::decode(...).map_err` arm) — the single line R5 rewrites. Neither
the spec's contract-7 site list nor the plan's copy of it names `:206`; both list only asserts
(`:310-313`, `:845-846`) and docs (`:106`, `:107`, `:1-25`). The plan mirrors the settled spec verbatim,
so this is not a plan-vs-spec divergence, and P3's four-situation RED tests force the `:206` rewrite
regardless — hence Nit, not blocking. But the enumerated list advertises itself as "sites this rewrite
touches" while omitting the primary one; consider adding `:206` for the implementer's benefit.
("appears in NO test" remains achievable: the retired string is in tests only at the `:107-108` doc
comment, which the list covers.)

### N2 — the mk-vs-md `derive_chunk_set_id` name collision is a live footgun in P3
md-cli already imports **both** `md_codec::chunk::derive_chunk_set_id` (policy-card; `encode.rs:8`,
`vectors.rs:72`) and `mk_codec::decode` (`seat/input.rs:203`). P3 must recompute with the **`mk_codec`**
pair (key cards), never the `md_codec` one. Spec files the collision as out-of-scope; a one-line P3
IMPL note ("use `mk_codec::{derive_chunk_set_id, encode_bytecode}`, not the `md_codec` namesake")
would prevent a wrong-import defect that would still compile.

---

## Completeness map (spec contract/requirement → plan phase)

| Spec item | Plan | OK? |
|---|---|---|
| Contract 1 mk-codec unchanged / no publish | P0 note, machine-verified facts | ✓ |
| Contract 2 six verbs warn @ chokepoint | P1 (5 verbs) + P2 (repair) | ✓ (M2 wording) |
| Contract 3 inspect prints stamped id | P1 | ✓ |
| Contract 4 verify: json field | P1 | ✓ |
| Contract 4 verify: **text-mode stdout report** | — | **I1 missing** |
| Contract 5 mint warning | P2 | ✓ |
| Contract 6 seat warning after reassembly | P3 | ✓ |
| Contract 7 R5 refusal rewrite + churn sites | P3 | ✓ (N1 site, I2 order) |
| Contract 8 Go derivation parity | P4 | ✓ |
| R2/R3/R4/R5/R6 | P1+P3 / P2 / P0 / P3 / cross-surface warning_text | ✓ |
| Acc: golden fires/absent; repair blessed-path | P1/P2 + acceptance | ✓ |
| Acc: contract-7 four vectors + retired-string-absent | P3 | ✓ |
| Acc: contract-7 **order precedence row** | — | **I2 missing** |
| Acc: per-surface mutation gates isolable | P0/P1/P3/P4 (separate repos/crates) | ✓ |
| Acc: two named guarantees + v0.1 byte-unchanged | P0 + acceptance | ✓ |

**Phase order:** P0 (corpus) → P1/P2 (mk-cli) → P3 (md-cli) → P4 (Go). Sound: P0 is the anchor
everything asserts against and comes first; no phase depends on a later one; each phase's "warning
fires on mismatch" RED fails before impl and passes after (RED derives from the fires-half; the
absent-on-clean-twin half passes trivially pre-impl, which is fine). Rust-primary respected — P4 is
convergence (Go already derives correctly), Rust leads. **No Critical.**

**Scope creep:** none found — the plan is a faithful subset of the spec; P0's separate SHA-pin and
v0.1-byte-unchanged assert are spec-mandated, not additive.
