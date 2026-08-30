# SPEC — verify `chunk_set_id` against the payload it is derived from

**Status: DRAFT for R0. 2026-08-19.** Normative admission change — no code
before 0C/0I.

## The gap

`mk-codec` 0.5.0 made `chunk_set_id` **derived**: the top 20 bits of
`SHA-256(canonical_bytecode)`, MSB-first. **Nothing verifies that derivation.**

Both decoders use the csid only to check that all chunks in a *set* carry the
same value — reassembly matching. The fork says so in its own test:
*"the decoder does not validate the csid value."* `DECISIONS.md` D-15 still
describes the field as *"per-encoding random … used for reassembly mismatch
detection, nothing more"*, which D-16 now contradicts.

So the id became a checkable property of the payload, and nothing checks it.

## Why it is worth closing

Operator intent, stated 2026-08-19: *"We want to match things back up someday
and want ids to be deterministic."* A derived id only supports matching if
something verifies the match. Comparing
`csid == derive_chunk_set_id(reassembled_payload)` is one line and catches:

- a card whose **payload was altered** after engraving — any edit moves the hash;
- **chunks of different cards** assembled together, even on a csid collision
  (20 bits is only ~1e6, so collisions are not negligible at corpus scale);
- a card that did **not** come from a conforming encoder.

It costs nothing: the chunk layer already computes that hash for its cross-chunk
integrity suffix.

## BLOCKED — this collides with a deliberate, tested guarantee

**Implemented, measured, and reverted 2026-08-19.** The change is 20 lines and
works. It also breaks **19 tests**, and the important ones are not fixtures:

```
mk-codec::chunk_set_id_determinism  an_explicit_chunk_set_id_still_wins
mk-codec::canonical_payload         canonical_payload_is_chunk_set_id_invariant
```

The first was written by the 0.5.0 cycle itself and says, in a comment:

> Both must still decode to the same card: **the id is opaque to content.**

So the codebase does not merely tolerate explicit csids — it **guarantees** them,
by name, in a test. Verifying the derivation deletes that guarantee. My original
estimate of "10 call sites, regenerate the fixtures" was wrong: this is a design
fork, not a fixture refresh.

### The two coherent positions

**A — the id is OPAQUE to content (today, and tested).**
`encode()` is deterministic, so re-encoding a card reproduces its strings
byte-for-byte; that is what makes cards matchable. `--chunk-set-id` produces
ordinary valid cards with a chosen id. The id groups chunks; it does not attest
to them.

**B — the id is BOUND to content (this spec).**
The id becomes a verifiable integrity property: an altered payload no longer
matches its own id. `--chunk-set-id` inverts into a way to build deliberately
non-conforming cards for negative tests. 19 tests change and one named guarantee
is deleted.

### What decides it

**Position A already delivers the stated goal.** The operator requirement was
*"we want to match things back up someday and want ids to be deterministic"* —
deterministic `encode()` satisfies that: re-encode the card and compare. B adds
**tamper-evidence**, which is a different and smaller property: it catches a
payload altered after engraving, which the cross-chunk hash (already verified,
4 bytes) largely covers too.

So B's marginal gain over A is: catching a card whose payload AND cross-chunk
hash were both rewritten consistently but whose csid was not. That is a narrow
threat, and it costs a documented feature.

**Recommendation: do NOT adopt B on current evidence.** Keep A. If tamper
evidence is wanted, the honest place is a `mk verify --strict` that reports
whether the id is derived, without making non-derived cards undecodable.

**Operator decision required before any code lands.**

## If B is chosen anyway: it is a HARD rejection

**Operator ruling 2026-08-19: "We don't care about old cards. None exist."**

That removes the only real objection. Pre-0.5.0 cards carry a random csid and
would fail verification with probability ≈ 1; none are in circulation, so the
check can reject rather than warn. No advisory mode, no `--strict` flag, no
version gate — those exist to protect a population that does not exist.

## The contradiction this creates, and its resolution

The same release added `mk encode --chunk-set-id <HEX>` and
`encode_with_chunk_set_id`, whose stated purpose is pinning a value for
"vector regeneration and conformance fixtures". **A strict decoder rejects
exactly the cards those produce**, unless the pinned value happens to equal the
derived one.

Measured blast radius:

| where | count | what |
| --- | --- | --- |
| `mnemonic-key` Rust tests | 10 | `encode_with_chunk_set_id(&card, 0x12345 / 0xABCDE)` |
| `mnemonic-engrave` me-cli | 6 | golden `bundle-md1-mk1.json` + `manifest.rs` fixture, csid `0x12345` |
| `seedhammer` | 2 | golden vectors using explicit csids |

**Resolution: the pin's legitimate purpose INVERTS.** It stops being a way to
make ordinary cards with a chosen id and becomes the way to construct a
**deliberately non-conforming card** — which is precisely what is needed to test
the new rejection. Its doc comment must say so, and every fixture that used it
as if the result were valid must be regenerated from `encode()`.

## Normative change

1. `mk_codec::decode` (and the chunked reassembly path) computes
   `derive_chunk_set_id(canonical_bytecode)` over the reassembled payload and
   compares it to the header's `chunk_set_id`.
2. Mismatch → a new `Error::ChunkSetIdNotDerived { expected, found }`, distinct
   from the existing `ChunkSetIdMismatch` (which means *chunks disagree with each
   other*, a different failure the operator fixes differently).
3. **Single-string (unchunked) encodings carry no csid and are unaffected.**
4. `encode_with_chunk_set_id` is documented as producing a card the decoder will
   refuse unless the value matches the derivation.

## Acceptance

- A card from `encode()` round-trips; a card from
  `encode_with_chunk_set_id(card, wrong)` is REFUSED with
  `ChunkSetIdNotDerived`, and the error names both values.
- `ChunkSetIdMismatch` still fires for chunks that disagree with each other, and
  is not shadowed by the new check.
- Mutation: deleting the comparison makes a wrong-csid card decode again.
- All three repos' fixtures regenerated; `me bundle`'s golden shows a derived
  csid, not `0x12345`.
- **Go port converges** (fork `mk/`), per the Rust-primary rule — Rust first with
  vectors, then the port.

## Not in scope

- `md-codec`'s csid. It has always been derived, but whether *it* verifies is a
  separate question and a separate cycle.
- The `D-15` wording fix ("per-encoding random"), which is a docs follow-up.
