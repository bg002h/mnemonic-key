r1: 0C/0I/2M/1N

# FIX review r1: design/FIX_bughunt_mk.md (mk bug hunt), head eac2ca1

Reviewer: independent adversarial review, one Opus round (small-fix track).
Scope: doc §1-§9 (mk-codec, mk-cli) and the toolkit manual lockstep §8
(`docs/manual/src/40-cli-reference/44-mk-cli.md` at f6355e22).
First round, so no findings come from an earlier fix commit.

Verdict: the doc is correct and Sonnet-proof. Every §3 text applied
literally gives the §2 behaviour. Every §4 row is RED on base and GREEN with
§3 applied. Every changed text and refusal, followed literally, gives a right
result or a named exit-64/exit-1 refusal.

## Evidence (executed in scratch worktree review-mk-r1 at eac2ca1, cargo 1.85.0)

- RED on base, §4 filter: 8 run, 3 passed (01 keys/in, 02, 06; §4 expects
  these to pass), 5 failed. This matches §4 and the controller log.
- GREEN with §3 applied exactly (python replace of the §3 texts, then
  `cargo fmt`):
  - `cargo nextest run --locked`: 399 run, 399 passed.
  - Bughunt filter: 8 run, 8 passed.
  - `cargo fmt --check` clean; `cargo clippy --locked --all-targets -- -D warnings`
    clean; `cargo test --locked --doc` ok.
  - The §7 gate counts hold.
- SPEC anchors: SPEC_mk_v0_1.md l.38 ("giving the prefix `mk1`."),
  l.272 ("Each component encodes a u32 BIP 32 child number"), l.401 (the
  "SHOULD expose this as an explicit choice" sentence) and
  SPEC_mk_derive_address.md l.51 ("**Range:**" bullet ending "Mutually
  exclusive (`conflicts_with`).") all exist as §1 cites them. The "§4 rule 6"
  reference resolves to SPEC_mk_v0_1.md l.326 (LEB128 overflow ->
  `InvalidPathComponent`).
- External facts (raw mediawiki via curl, grepped):
  - BIP-93 l.82: "MUST be entirely uppercase or entirely lowercase".
  - BIP-93 l.178: CHARSET `qpzry9x8gf2tvdw0s3jn54khce6mua7l`.
  - BIP-173 l.80: the HRP is US-ASCII [33-126].
  - BIP-173 l.192: a decoder MUST NOT accept mixed case.
  - Together these support "ASCII only, no Unicode folding".
  - BIP-32 l.66: normal indices are 0..2^31-1, so the ceilings 2147483647
    (range end) and 2147483648 (count) are right.
  - LEB128: the 5-byte u32 bound and non-minimal-accept match SPEC l.274 and
    rule 6.
- Toolkit §8 rows applied to 44-mk-cli.md in tk worktree:
  - markdownlint 0 errors; lychee 0 errors.
  - `make lint` shows 10 "stale marker" FAILs that are IDENTICAL on the base
    file. They are environmental: the installed mk 0.13.0 is newer than the
    tagged release markers.
  - `ci/doc-flag-lint.test.sh` gives "12 passed, 3 failed", the same set as
    on the base file (diffed), so this is also environmental. The §8 rows
    add no failure.

## Case tables (executed with the GREEN binary, mk 0.14.0-dev)

### #1 encode fingerprint choice (doc §2 row 1, §3.2)

| input | result | exit |
|---|---|---|
| `--xpub X --origin-path 84h/0h/0h --policy-id-stub 1c0170fe` (no fp flag) | `error: pass --origin-fingerprint <hex> or --privacy-preserving explicitly` | 64 |
| same + `--origin-fingerprint 73c5da0a` | 2 cards, fp carried | 0 |
| same + `--privacy-preserving` | 2 cards, no fp | 0 |
| no stub, no fp | `at least one of --policy-id-stub or --from-md1 is required` (precedes the new arm) | 64 |
| `--keys F` / `--in F`, no fp flag | unchanged (row 01_keys_and_in PASS on base and GREEN) | 0 |
| both fp flags | existing conflict refusal, unchanged | 64 |

### #3 address cap (doc §1.4, §3.3)

All rows used a 2-chunk test-vector card (cli_mk1_repair_reverify CHUNK0/1).

| input | result | exit |
|---|---|---|
| `--count 1000000` | 1,000,000 address lines | 0 |
| `--count 1000001` | `--count 1000001 too large (max 1000000)` | 64 |
| `--count 2147483649` | BIP-32 ceiling text fires first (as §1.4 orders) | 64 |
| `--range 5,1000004` | 1,000,000 lines | 0 |
| `--range 5,1000005` | `--range too large (max 1000000)` | 64 |
| `--range 9,3` | `--range start 9 must be <= end 3` (fires before the cap; there is no u32 underflow) | 64 |
| `--range 2147483000,2147483648` | ceiling text `(2147483647)` | 64 |
| `--range 2147483000,2147483647` | accepted | 0 |

- clap help (S3/S4 texts) renders as §3.3 states.

### #4 `--from-md1-set` byte-3 slice (§3.2)

| input | result |
|---|---|
| a line whose first 3 bytes split a multibyte char | `get(..3)` is None, so the line is skipped (no panic) |
| every line skipped | the existing empty-set error fires |
| an ASCII `md1…` line | unchanged |

### #5 non-ASCII decode (§2, §3.1, S5, S6)

| input | result | exit |
|---|---|---|
| `m` + U+212A + rest of a valid card | `invalid HRP: mK` (U+212A kept, per S5) | 1 |
| U+212A at data index 17 | `invalid character K at position 17` | 1 |
| U+212A at data index 17, then truncated (wrong length) | InvalidChar (it precedes the length lookup, per §3.1) | 1 |
| all-uppercase valid card pair | decodes (one xpub) | 0 |
| an uppercase string with U+212A in the HRP | `invalid HRP` (case_check is ASCII-only, so no MixedCase) | 1 |

- Every cell is decided by the doc. No undecided cell was found.

## Create-input table (printed or advised commands)

| text | command run | result |
|---|---|---|
| §2 refusal `pass --origin-fingerprint <hex> or --privacy-preserving explicitly` | both spellings, real clap flags (see #1) | 0 |
| §3.3 help `(max 1000000)` / `at most 1000000 indices` | `--count 1000000`, `--range 5,1000004` | 0 |
| §8 manual rows 31/39/47/499/500 | flags exist in `mk encode --help` / `mk address --help` | doc-flag-lint adds no failure |
| csid re-mint warning (unchanged text, reached via #1) | `mk encode` without `--chunk-set-id` and without a fp flag | exit 64 named refusal (see M1) |

## Findings

### Minor

- **M1** `crates/mk-cli/src/cmd/mod.rs` l.130-136 (the csid mismatch
  warning; me-cli `csid_warn.rs` has the same text).
  - The warning says "run mk encode again without --chunk-set-id". Followed
    literally after #1 (with the other flags not repeated), it now gives exit
    64 "pass --origin-fingerprint <hex> or --privacy-preserving explicitly".
  - It fails closed with a named refusal, and no printed command fails as
    printed (the warning names no full command).
  - Before #1, the same literal re-mint silently dropped the fingerprint, so
    #1 makes this path safer.
  - Conditions: (a) a card with a stamped, non-derived csid; (b) the holder
    re-mints without repeating the fingerprint flag.
  - Defer: the warning could say "with the same flags".
- **M2** toolkit `docs/manual-gui/src/70-mk/73-encode.md` l.95-110.
  - It still calls `--origin-fingerprint` "Optional" and says omitting both
    flags yields `origin_fingerprint = None`. That is false after #1.
  - It fails closed (exit 64, named refusal). §8 does not list this file.
  - Conditions: (a) a GUI-manual reader; (b) the GUI omits both flags
    (its schema sync is the grade §5 follow-up).
  - File it with that mnemonic-gui follow-up.

### Nit

- **N1** §3.3's one-line insertion in `address.rs` is not
  rustfmt-shaped. `cargo fmt` reflows it, which a builder runs at the gate.
  No behaviour change.

Design (ungraded): the #1 refusal could name the third route (`--keys`/`--in`
records) for holders minting many cards; the S3 text is ruled, so this is not
graded.

C / I / M / N = 0 / 0 / 2 / 1
