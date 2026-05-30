# SPEC — mk1 no-path (depth-0) support

**Branch:** `mk-no-path-support` (off `main` `5c2bc8c`)
**Crate:** `mk-codec` 0.3.2 → **0.4.0** (MINOR — wire-additive: a new decodable case); `mk-cli` 0.4.3 → **0.5.0**
**Source ground-truth SHA:** `5c2bc8c` (all citations below re-grepped against this tree).
**Companion:** resolves the toolkit-discovered `mk1-wif-bundle-depth0-invalid-card`; a paired toolkit re-pin cycle follows publication.

---

## §1. Problem

A raw EC private key (WIF) — and any non-HD / master key — has **no BIP-32 derivation
path**. The downstream `mnemonic bundle --slot @N.wif=…` flow builds exactly this
shape for an mk1 KeyCard: a **depth-0 xpub** (`depth 0`, `child_number Normal{0}`,
`parent_fingerprint 0x00000000`, zero chain code, WIF pubkey) with an **empty
`origin_path`** (`m`). This is the correct, faithful representation — *no path
applies, so no path should appear on the wire.*

mk1 today cannot carry it. Two independent failures:

1. **Decode-side (present since v0.1, including the toolkit's current 0.3.1 pin):**
   `encode_path(empty)` emits `[0xFE, 0x00]` (explicit path, 0 components), but
   `decode_explicit_path` rejects `count == 0` as `Error::PathTooDeep(0)`
   (`path.rs:114`), and `reconstruct_xpub` `.expect()`s a non-empty path
   (`xpub_compact.rs:92-95`). So a `bundle --wif` card **encodes but never
   decodes** — it is a write-only card. `verify-bundle` / `inspect`
   (`mk_codec::decode` at `verify_bundle.rs:1225`, `inspect.rs:178`) fail on it.

2. **Encode-side (mk-codec 0.3.2):** the new `XpubOriginPathMismatch` guard
   (`encode.rs:33-42`) rejects the card outright — depth matches (`0 == 0`) but the
   child clause fires: `Some(Normal{0}) != path_child None`. (This is why the
   toolkit re-pin to 0.3.2 was reverted.)

**Goal (user directive):** *permissive on input, expressive on output — when no
path applies, no path is included on the wire,* and that no-path card must
**round-trip** (encode → decode → reconstruct the same depth-0 xpub).

---

## §2. Source ground-truth (verified @ `5c2bc8c`)

- **`consts.rs:27`** — `pub const MAX_PATH_COMPONENTS: u8 = 10;`
- **`bytecode/path.rs:85-98`** — `encode_path`: empty path → not in `STANDARD_PATHS`
  (`lookup_path` returns `None`) → explicit branch pushes `0xFE`, then
  `components.len() as u8` (= `0`), then zero LEB128 components → `[0xFE, 0x00]`.
  No panic.
- **`bytecode/path.rs:112-130`** — `decode_explicit_path`: `if count == 0 || count >
  MAX_PATH_COMPONENTS { return Err(Error::PathTooDeep(count)); }` (`:114`). For
  `count == 0` the component loop (`for _ in 0..count`, `:118`) runs zero times.
- **`bytecode/path.rs:22`** — already `use bitcoin::bip32::{ChildNumber, DerivationPath};`.
- **`bytecode/path.rs:248-258`** — test `rejects_path_count_zero` (asserts the
  current `PathTooDeep(0)` reject; **must invert**).
- **`bytecode/xpub_compact.rs:18`** — already `use bitcoin::bip32::{… ChildNumber …};`.
- **`bytecode/xpub_compact.rs:83-95`** — `reconstruct_xpub`: `depth =
  components.len() as u8`; `child_number = components.last().copied().expect("origin_path
  must be non-empty per SPEC §3.5")`. Rustdoc `:83-84` asserts non-empty.
- **`bytecode/encode.rs:14-18`** — production imports do **not** include
  `ChildNumber` (only the `#[cfg(test)]` mod imports it at `:68`). The guard change
  adds `use bitcoin::bip32::ChildNumber;` to production scope.
- **`bytecode/encode.rs:33-42`** — the guard:
  ```rust
  let path_depth = card.origin_path.into_iter().count();
  let path_child = card.origin_path.into_iter().last().copied();
  if card.xpub.depth as usize != path_depth || Some(card.xpub.child_number) != path_child {
      return Err(Error::XpubOriginPathMismatch { … });
  }
  ```
- **`bytecode/encode.rs:152-170`** — test `rejects_empty_origin_path` (asserts the
  current reject; **must invert**).
- **`bytecode/test_helpers.rs:22-31`** — `synthetic_xpub(path)` derives `depth =
  components.len()` and `child_number = components.last().copied().unwrap_or(ChildNumber::Normal
  { index: 0 })`. So `synthetic_xpub(&DerivationPath::from_str("m"))` is exactly the
  WIF shape: depth 0, child `Normal{0}`. No test-helper change needed.
- **`error.rs`** — `Error::XpubOriginPathMismatch { xpub_depth, path_depth, xpub_child,
  path_child }` already exists (added 0.3.2). **No new error variant** in this cycle.
- **Mirrors already complete (0.3.2), unchanged this cycle:** `tests/error_coverage.rs`
  (`ErrorVariantName::XpubOriginPathMismatch :77`, `display_prefix :108`, `is_exempt
  :124` — exempt because encoder-only / not producible via `decode`; still true),
  `mk-cli/src/error.rs:133` (kind map). No variant added → no mirror edits.
- **`SPEC_mk_v0_1.md`** citations: `:172`, `:229`, `:237`, `:254-261`, `:263`, `:265`,
  `:285`, `:294`, `:303` (full text reproduced in §4).

---

## §3. Design

Three localized mk-codec changes. The wire encoding of "no path" **reuses the
existing explicit form** `[0xFE, 0x00]` (`encode_path` already emits it); only the
decode/reconstruct/guard sides relax. Wire-additive exactly like the `0x16`
precedent: older decoders reject `0xFE 0x00` as `PathTooDeep(0)`, 0.4.0+ accept.

### 3.1 `decode_explicit_path` — accept `count == 0` (`path.rs:114`)

```rust
let count = read_u8(cursor)?;
if count > MAX_PATH_COMPONENTS {
    return Err(Error::PathTooDeep(count));
}
// count == 0 is the no-path / depth-0 root case (e.g. a WIF). The component
// loop below runs zero times → DerivationPath::from(vec![]) = empty path "m".
```

The `count == 0` disjunct is removed. `count > 10` still rejects (`PathTooDeep`).
The existing loop already produces an empty `DerivationPath` for `count == 0`.

### 3.2 `reconstruct_xpub` — empty path → `child_number = Normal{0}` (`xpub_compact.rs:92-95`)

```rust
// depth = 0 for an empty path. child_number defaults to the BIP-32 master
// convention Normal{0} when origin_path is empty (no-path / depth-0 key).
let child_number = components
    .last()
    .copied()
    .unwrap_or(ChildNumber::Normal { index: 0 });
```

`depth = components.len() as u8` already yields `0` for an empty path. The rustdoc
(`:83-84`, `:89-91`) is updated: the "MUST be non-empty" precondition is replaced
with the depth-0 / no-path semantics.

### 3.3 The guard — accept a consistent depth-0 card (`encode.rs:33-42`)

```rust
let path_depth = card.origin_path.into_iter().count();
let path_child = card.origin_path.into_iter().last().copied();
// expected_child mirrors reconstruct_xpub exactly: the terminal component, or
// Normal{0} for an empty path (depth-0 / no-path key). A card encodes iff it
// survives compact-drop + reconstruction unchanged.
let expected_child = path_child.unwrap_or(ChildNumber::Normal { index: 0 });
if card.xpub.depth as usize != path_depth || card.xpub.child_number != expected_child {
    return Err(Error::XpubOriginPathMismatch {
        xpub_depth: card.xpub.depth,
        path_depth: path_depth as u8,
        xpub_child: card.xpub.child_number,
        path_child,
    });
}
```

Adds `use bitcoin::bip32::ChildNumber;` to production scope (`encode.rs:14-18`). The
only change is `Some(card.xpub.child_number) != path_child` →
`card.xpub.child_number != expected_child`. Effect:

| Card | `depth` clause | `child` clause | Verdict | Change? |
|------|----------------|----------------|---------|---------|
| depth-0, empty path, child `Normal{0}` (WIF) | `0 == 0` ok | `Normal{0} == Normal{0}` ok | **accept** | was reject → now accept |
| depth-0, empty path, child `Normal{5}` (non-canonical, won't round-trip) | `0 == 0` ok | `Normal{5} != Normal{0}` | **reject** | unchanged (still reject) |
| depth-4 xpub, depth-3 path (the original drift bug) | `4 != 3` | — | **reject** | unchanged |
| depth-4 xpub, depth-4 path, terminal child mismatch | `4 == 4` ok | `child != terminal` | **reject** | unchanged |
| aligned non-empty (every existing card) | ok | ok | **accept** | unchanged |

The guard remains exactly "does this xpub survive `from_xpub` + `reconstruct_xpub`"
— now correct at depth 0.

### 3.4 Losslessness at depth 0

For the WIF card: compact-73 carries `version`, `parent_fingerprint` (zeros),
`chain_code` (zeros), `public_key` verbatim; drops `depth`/`child_number`.
`reconstruct_xpub(compact, empty)` → `depth 0`, `child Normal{0}`. Both reconstructed
fields equal the originals (the guard guarantees it), so the round-trip is lossless:
`decode(encode(card)).xpub == card.xpub` and `.origin_path` is the empty path. The
zero chain code is a pre-existing toolkit convention for a non-derivable leaf key and
is **out of scope** here — this cycle is about the path/depth, not the chain code.

---

## §4. `SPEC_mk_v0_1.md` edits

All edits land in the same PR; internal consistency checked in §6 Phase 1.

**E1 — `:172`** (payload field-order comment). Before:
`… explicit: 0xFE + count + 1..=10 LEB128 components …` →
`… explicit: 0xFE + count + 0..=10 LEB128 components (count 0 = no-path / depth-0 key) …`.

**E2 — `:229`** (explicit-path layout). Before:
`[component_count: 1 byte; MUST be in 1..=10]` →
`[component_count: 1 byte; MUST be in 0..=10 (0 = no-path / depth-0 root key)]`.

**E3 — `:237`** (cap paragraph). The cap sentence is unchanged
(`Decoders MUST reject component_count > 10 with Error::PathTooDeep`). Append:
*"`component_count == 0` is valid as of v0.4.0 and denotes a key with no derivation
path (`depth 0`); see §3.6. Earlier decoders reject it as `PathTooDeep(0)` — the
addition is wire-additive, like `0x16`."*

**E4 — `:254-261`** (reconstruction rule). Replace the two-line block with:
```
depth        := component_count(origin_path)              (0 for the no-path case)
child_number := last_component(origin_path) (with hardened-bit encoding),
                or Normal{0} when origin_path is empty (depth-0 / no-path key)
```
Keep the following sentence (`For a standard-table indicator … on-wire components.`)
and append: *"For the no-path case (explicit `count == 0`), `depth = 0` and
`child_number = Normal{0}` (the BIP-32 master convention)."*

**E5 — `:263`** ("Why compact-73"). The losslessness claim is unchanged in substance;
append to the parenthetical that the encoder agreement check treats an empty path as
expecting `child_number = Normal{0}` (so a consistent depth-0 card encodes; a
depth-0 card with a non-`Normal{0}` child is rejected).

**E6 — `:285`** (decoder rule 5). Before:
`Has an explicit path with component_count > 10 (or == 0) (Error::PathTooDeep).` →
`Has an explicit path with component_count > 10 (Error::PathTooDeep). component_count
== 0 is valid as of v0.4.0 (no-path / depth-0 key; see §3.6).`

**E7 — `:294`** (encoder-side invariant). Reframe the rule to match §3.3: encoders
MUST reject a card whose `xpub.depth ≠ component_count(origin_path)` OR whose
`xpub.child_number ≠ [last_component(origin_path), or Normal{0} when the path is
empty]`, with `Error::XpubOriginPathMismatch`. Add one sentence: a consistent
depth-0 / no-path card (`depth 0`, empty path, child `Normal{0}`) is **valid** and
encodes; the invariant rejects only genuine disagreement.

**E8 — `:303`** (closing note). Update the parenthetical describing the child clause
to include the empty-path → `Normal{0}` expectation, consistent with E7.

**E9 — `:265` / `:360`** (limit-of-detection notes). No semantic change; the
wrong-indicator hazard is unaffected by the no-path addition. Leave as-is unless a
reference to "`1..=10`" appears (it does not).

---

## §5. SemVer, lockstep, mirrors

- **mk-codec 0.3.2 → 0.4.0 (MINOR).** Wire-additive: a new decodable case
  (`count == 0`). Nothing breaks — existing cards encode/decode identically; the
  guard only *relaxes* (accepts a previously-rejected consistent depth-0 card). The
  forward-incompat (0.3.x decoders reject `0xFE 0x00`) is the MINOR signal, mirroring
  how the `0x16` wire-additive shipped.
- **mk-cli 0.4.3 → 0.5.0**, re-pin `mk-codec = { path = "../mk-codec", version =
  "0.4.0" }`. mk-cli behavior is purely additive (decoding a no-path card now
  succeeds). No CLI flag/subcommand/output-shape change.
- **No new `Error` variant** → `tests/error_coverage.rs`, `mk-cli/src/error.rs` kind
  map, and the toolkit's `friendly.rs` / `error.rs` mirrors need **no** variant
  addition. (The toolkit re-pin cycle adds explicit `XpubOriginPathMismatch` arms as
  hygiene — both toolkit mirrors have `_ =>` fallbacks today.)
- **No GUI schema-mirror lockstep, no manual lockstep:** no clap flag / subcommand /
  dropdown-value / `--help`-surface change anywhere.
- **CHANGELOG:** none present in mk-codec (Glob empty); skip.

---

## §6. Test plan (TDD — tests before impl, per phase)

All cells compile against live types; the three reject-inversions FAIL pre-change.

**Path layer (`bytecode/path.rs` test mod):**
- **T1 (invert `rejects_path_count_zero` → `accepts_path_count_zero_as_empty`):**
  `decode_path(&mut &[0xFE, 0x00][..])` → `Ok(p)` with `p` empty
  (`p.into_iter().count() == 0`). Pre-change: FAILs (was `Err(PathTooDeep(0))`).
- **T2 (`round_trip_empty_path`):** `encode_path(&DerivationPath::from_str("m").unwrap())
  == vec![0xFE, 0x00]`; decode back → empty path.
- **T3 (`rejects_path_too_deep` unchanged):** `count == 11` still `PathTooDeep(11)`.

**Compact layer (`bytecode/xpub_compact.rs` test mod):**
- **T4 (`reconstruct_depth0_empty_path`):** `synthetic_xpub(&empty)` → compact →
  `reconstruct_xpub(&compact, &empty)` → `depth == 0`, `child_number ==
  Normal{0}`, and `parent_fingerprint` / `chain_code` / `public_key` / `network`
  round-trip. Pre-change: panics on `.expect()`.

**Encode layer (`bytecode/encode.rs` test mod):**
- **T5 (invert `rejects_empty_origin_path` → `accepts_consistent_depth0_card`):**
  KeyCard `{ stubs: [[0xAA;4]], fp: None, xpub: synthetic_xpub(&empty), origin_path:
  empty }` → `encode_bytecode(&card).is_ok()`. Pre-change: FAILs (was reject).
- **T6 (`rejects_depth0_noncanonical_child`):** same card but
  `card.xpub.child_number = Normal{5}` → `Err(XpubOriginPathMismatch { path_child:
  None, xpub_child: Normal{5}, xpub_depth: 0, path_depth: 0 })`. Guards against
  accepting a depth-0 card that would NOT round-trip.
- **T7 (`rejects_xpub_depth_mismatch` / `rejects_xpub_child_mismatch_same_depth` /
  `aligned_explicit_path_card_encodes` unchanged):** the three existing guard cells
  stay green — the non-empty-path behavior is unchanged.

**End-to-end round-trip (public bytecode API):**
- **T8 (`depth0_card_round_trips`):** `encode_bytecode(depth0_card)` →
  `decode_bytecode(&wire)` → KeyCard with empty `origin_path` and `xpub.depth == 0`,
  `xpub.child_number == Normal{0}`, equal pubkey/chain_code. (Plan pins the exact
  `decode_bytecode` entry point.) This is the highest-value cell — it proves the WIF
  card the toolkit emits now survives the full mk-codec round-trip.

**Optional (plan-author discretion):** add one canonical no-path KeyCard to the
generated test-vector corpus (`vectors.rs` / `gen_mk_vectors.rs`) for cross-impl
conformance. Deferrable if it materially enlarges the cycle.

**Non-regression:** full `cargo test -p mk-codec` + `cargo test -p mk-cli`; the
aligned-by-construction vector corpus stays green (no existing card has an empty
path); `cargo clippy -p mk-codec -p mk-cli --all-targets -- -D warnings`;
`cargo +stable fmt --check` (scoped).

---

## §7. Phases

**Phase 0 — mk-codec code + tests (TDD).**
0.1 `decode_explicit_path` count==0 relax (§3.1) + T1/T2/T3.
0.2 `reconstruct_xpub` empty→Normal{0} + rustdoc (§3.2) + T4.
0.3 guard expected_child + `use ChildNumber` (§3.3) + T5/T6/T7.
0.4 end-to-end T8.
Per-phase: tests fail first → impl → green → `clippy -D warnings` → `fmt --check`.

**Phase 1 — `SPEC_mk_v0_1.md` edits (§4 E1-E9).** Re-grep each citation before
editing; verify no residual `1..=10` / `(or == 0)` survives; internal-consistency
pass (E4/E7/E8 mutually consistent on the empty→`Normal{0}` rule).

**Phase 2 — version + FOLLOWUP + ship.** `mk-codec` 0.3.2→0.4.0, `mk-cli`
0.4.3→0.5.0 + re-pin; file `mnemonic-key` FOLLOWUP `mk1-no-path-depth0-support`
(Status resolved `<phase-0 SHA>`) with a `mnemonic-toolkit` companion line; flip
nothing else. End-of-cycle opus R0 → GREEN → ff-merge `main` → publish
`mk-codec 0.4.0` + `mk-cli 0.5.0` to crates.io.

**Phase 3 (separate cycle, post-publish) — toolkit re-pin.** Re-pin
`mnemonic-toolkit` `mk-codec` 0.3.1→0.4.0; add explicit `XpubOriginPathMismatch`
arms to `friendly.rs` + `error.rs::mk_codec_exit_code`; fix the two
`verify_bundle.rs` fixtures (depth-4 xpub + bip84 depth-3 path) the guard correctly
rejects; add a `bundle --wif → verify-bundle` round-trip regression; resolve
`mk1-wif-bundle-depth0-invalid-card` + `mk1-depth-child-compensating-check-watch`;
**gate on the full toolkit suite before commit**; ship (PATCH). Its own plan + R0.

---

## §8. FOLLOWUP record

File in `mnemonic-key/design/FOLLOWUPS.md`:
`mk1-no-path-depth0-support` — *"mk1 carries no origin path for a depth-0 / no-path
key (WIF, master). decode accepts explicit `count == 0`; reconstruct → depth 0,
child `Normal{0}`; encode guard accepts a consistent depth-0 card."* Status:
`resolved <phase-0 SHA>`. Companion: `mnemonic-toolkit`
`mk1-wif-bundle-depth0-invalid-card`.
