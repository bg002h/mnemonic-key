# Bug hunt 2026-11-25 (Fable): mnemonic-key

Report-only review of `mnemonic-key` at `113d8b6`, part of a constellation-wide hunt across all 11 bg002h repositories. Nothing in this repository was changed by the hunt; this file is the only addition.

**How it was done.** Eight Fable reviewers each audited an area (one of them the boundaries between repositories), reproducing what they could with locally built binaries or scratch tests that were not committed. Three further Fable verifiers then tried to disprove every High and Medium finding (and several Low ones) and re-graded them. Grades below are the verifiers' final grades where they ran.

**Grades.** Critical: can lose funds or leak a seed in ordinary use. High: wrong wallet, addresses or keys, or secret exposure, plausible in normal use, or recovery broken. Medium: needs an unusual but realistic input, or fails in a way a careful user might not notice. Low: fails closed, hardening, or a small exposure. Info: context, not a bug.

**Categories.** security (secret or key exposure, injection, integrity of what is trusted), funds (wrong address, amount, descriptor or recovery that could lose or strand coins), other.

**Status.** "Confirmed (reproduced)" means a command or test showed it. "Confirmed by reading" means the code path was traced but not executed. "Reviewer-confirmed, not re-verified" means the first reviewer reproduced or read it and no verifier re-checked it.

**Totals:** 6 Low. No Critical findings.

## Summary

| ID | Grade | Category | Status | Finding |
|---|---|---|---|---|
| mk#2 | Low | funds | Confirmed (reproduced) | `mk verify --from-md1` says "inputs match" for a key that is not a cosigner |
| mk#4 | Low | security | Confirmed (reproduced) | A mis-pasted xprv is echoed in full in the error text (and `--json`) |
| mk#1 | Low | other | Confirmed (reproduced) | `--from-md1` (md-codec 0.42.0 pin) cannot read wire-version-8 md1 cards |
| mk#3 | Low | other | Reviewer-confirmed, not re-verified | Non-ASCII characters in an md1 are read as codex32 symbols (old md-codec pin) |
| X6 | Low | other | Reviewer-confirmed, not re-verified | Non-ASCII characters read as codex32 symbols by mk-codec 0.4.1/0.5.0 used by consumers |
| SH#4 | Low | other | Confirmed (reproduced) | Related: the seedhammer Go port of `mk.Encode` lacks mk-codec 0.6.0's origin refusals |

IDs: `md#`, `mk#`, `ms#`, `toolkit#`, `G`, `refugium#`, `wallet#`, `E`, `T`, `SH#`, `M` are per-repository findings; `X` findings are at a boundary between repositories and appear in every report they touch.

## Findings

## mk#2: `mk verify --from-md1` says "inputs match" for a key that is not a cosigner

**Grade:** Low · **Category:** funds · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `mk`)

### 2. `mk verify --from-md1` reports "inputs match" for a key that is not in the keyed policy — Low/Medium — CONFIRMED

**Evidence**
- `crates/mk-cli/src/cmd/verify.rs:140-160`: `--from-md1` is reduced to a stub via `derive_stub_from_md1_card` (`verify.rs:146`) and compared as a multiset; the decoded policy's cosigner set is discarded.
- `crates/mk-cli/src/cmd/encode.rs:340-365`: `mk encode` with the same md1 refuses the same key ("is not a cosigner of the wallet policy").
- `verify.rs:251-253`: the success line says "(and any --xpub / --origin-* / --policy-id-stub / --from-md1 inputs match)".

**Impact**
With the full keyed policy card in hand, `mk verify` still accepts a card that only carries the right 4-byte stub: a card minted with `--policy-id-stub` for an unrelated xpub (or with a contradictory origin/fingerprint) verifies "OK", exit 0, `{"ok":true}`. An operator checking a plate set against the wallet's md1 before relying on it gets a pass for a key the wallet does not contain. The help text does describe `--from-md1` as a stub source, and the 2026-11-19 report treated verify's stub comparison as documented; the gap is that encode enforces membership/origin for the identical input pair and verify, the checking verb, does not.

**Reproduction**
```
V=/home/user/descriptor-mnemonic/crates/md-codec/tests/vectors
grep ^md1 $V/keyed_compose_preset_decaying_multisig.phrase.txt > keyed.md1     # 9 chunks, WalletPolicyId 3b6909e0...
X=xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a   # not a cosigner
$MK encode --xpub $X --origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a --from-md1-set keyed.md1
error: xpub xpub6Bme... (origin 84'/0'/0') is not a cosigner of the wallet policy with stub 3b6909e0; that policy declares 4 key(s). ...
$MK encode --xpub $X --origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a --policy-id-stub 3b6909e0 > foreign.txt   # rc=0
ARGS=""; for c in $(cat keyed.md1); do ARGS="$ARGS --from-md1 $c"; done
$MK verify --in foreign.txt $ARGS
OK: mk1 string(s) decode cleanly (and any --xpub / --origin-* / --policy-id-stub / --from-md1 inputs match)
rc=0
$MK verify --in foreign.txt $ARGS --json
{"chunk_set_id":{...},"chunks":2,"ok":true,"policy_id_stubs":["3b6909e0"],"schema_version":1}
```

**Fix direction**: in `verify`, when a `--from-md1` card decodes to a complete keyed policy, run the same membership and slot origin/fingerprint checks `encode` runs (`check_card_against_slots`) and exit 4 on a contradiction; keep stub-only comparison for keyless templates and say so in the OK line.

**Verifier re-check** (source: `verify-2`)

### mk#2: `mk verify --from-md1` passes a key that is not a cosigner

**Verdict: CONFIRMED. Grade: Low (write-up said Low/Medium). Category: funds.**

Re-run with the 9-chunk keyed policy `keyed_compose_preset_decaying_multisig`: `mk encode … --from-md1-set
keyed.md1` refuses the foreign xpub ("is not a cosigner of the wallet policy with stub 3b6909e0", rc=64); a
card for the same xpub minted with `--policy-id-stub 3b6909e0` then gives

```
$ mk verify --in foreign.txt --from-md1 <9 chunks>
OK: mk1 string(s) decode cleanly (and any --xpub / --origin-* / --policy-id-stub / --from-md1 inputs match)   rc=0
```

Low because the help says what is compared ("Expected `policy_id_stub` derived from md1 strings … compared as
a multiset") and the repo's Known hazards say a stub is never proof of membership; reaching the state needs a
card deliberately or mistakenly minted with a bare stub. The defect is the OK line's "inputs match" when the
full keyed policy was in hand, and encode and verify disagreeing on the same input pair.

## mk#4: A mis-pasted xprv is echoed in full in the error text (and `--json`)

**Grade:** Low · **Category:** security · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `mk`)

### 4. A mis-pasted extended private key is echoed in full in the error text — Low — CONFIRMED

**Evidence**
- `crates/mk-cli/src/slip132.rs:129`: `CliError::UsageError(format!("invalid xpub {s:?}: {e}"))` quotes the whole argument; the key-file reader repeats it with the file name and line.
- Reached from `mk encode --xpub`, `mk encode --in/--keys` and `mk verify --xpub`.

**Impact**
If an operator passes an `xprv`/`tprv` (or a SLIP-132 private form) where the xpub belongs — an easy slip when exporting from a wallet — mk prints the complete private key on stderr (and inside a `--json` error envelope's message). For the key-file path the secret was in a file, not on argv, so the echo is the first time it reaches the terminal, a CI log or a wrapper's captured stderr. `mk` otherwise has no private-key surface (FOLLOWUPS `md-mk-private-key-surface-watch`), so nothing guards this.

**Reproduction** (BIP-32 test vector 1 master xprv, not a real key)
```
$MK encode --xpub xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi \
    --origin-path m --origin-fingerprint 3442193e --policy-id-stub aabbccdd
error: invalid xpub "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi": unknown version magic bytes: [4, 136, 173, 228]
rc=64
printf '[3442193e/]%s\n' "$XPRV" > k.txt; $MK encode --in k.txt --policy-id-stub aabbccdd
error: k.txt:1: invalid xpub "xprv9s21ZrQH143K3QTDL4...MPHi": unknown version magic bytes: [4, 136, 173, 228]
rc=64
```

**Fix direction**: detect the private version bytes (xprv/tprv and the SLIP-132 private set) before parsing and refuse with a fixed message that names the mistake without quoting the value; never echo an unparsed key argument in full.

**Verifier re-check** (source: `verify-2`)

### mk#4: mis-pasted xprv echoed in the error

**Verdict: CONFIRMED. Grade: Low. Category: security.**

```
$ printf '[3442193e/]%s\n' "$XPRV" > k.txt; mk encode --in k.txt --policy-id-stub aabbccdd
error: k.txt:1: invalid xpub "xprv9s21ZrQH143K3QTDL4LXw…yuGBxrMPHi": unknown version magic bytes: [4, 136, 173, 228]   rc=64
$ … --json
{"error":{…"message":"k.txt:1: invalid xpub \"xprv9s21Z…MPHi\": unknown version magic bytes…"},"schema_version":1}   rc=64
```

The whole private key is printed, in the text error and inside the `--json` envelope on stdout. On the
key-file route the key had not been on argv. It needs the user to supply a private key where a public one
belongs, and it fails closed, so Low.

## mk#1: `--from-md1` (md-codec 0.42.0 pin) cannot read wire-version-8 md1 cards

**Grade:** Low · **Category:** other · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `mk`)

### 1. `--from-md1` cannot read wire-version-8 md1 cards (pinned md-codec 0.42.0) — Medium — CONFIRMED — cross-repo (descriptor-mnemonic)

**Evidence**
- `crates/mk-cli/Cargo.toml:34` pins `md-codec = "0.42.0"`; `Cargo.lock:510-511` resolves 0.42.0.
- `vendor/md-codec/src/header.rs:42` accepts only wire version 4 (`WireVersionMismatch` otherwise).
- descriptor-mnemonic (`c8cde23`, md-codec 0.48.7) has minted wire version 8 since md-codec 0.46.0 (CHANGELOG "A second taproot internal-key kind, at wire version 8", the `UNSPENDABLE(liana)` taproot shapes).
- `crates/mk-cli/src/cmd/mod.rs:343` (`decode_md1_card`) is the only route to a membership- and origin-checked mint.

**Impact**
A key card cannot be bound to (or verified against) any Liana-unspendable taproot policy that the sibling `md` mints today. `mk encode --from-md1[-set]` and `mk verify --from-md1` exit 1. The only way left to mint is `--policy-id-stub <hex>`, which the repo's own Known hazards say is not membership-checked, so the cosigner/origin/fingerprint guards (`encode.rs:340-365`, `check_card_against_slots`) are skipped for exactly these wallets. It is a refusal, not a silent mis-read.

**Reproduction**
```
V=/home/user/descriptor-mnemonic/crates/md-codec/tests/vectors
grep '^md1' $V/keyed_tr_liana_kofn_recovery.phrase.txt > liana.md1
$MK encode --xpub xpub6DXuQW1Q2Jpa1R4knHcUvkygN7KwRMuPvpiuSQiqrbK3rEjKdmbwKQbdR8rsPwN9vaRdoCJ3BktbxzwBm3y7mZX5jTBFgZbkV54FcqzxDPw \
    --origin-path "m/48'/0'/0'/3'" --origin-fingerprint 73c5da0a --from-md1-set liana.md1
error: md1 input rejected: wire-format version mismatch: got 8, expected 4
rc=1
```
The xpub is `@0` of that policy (its conformance JSON). Same result for the single-string card `md1gzfdsssj5qqcreqygvtam2lm5fenw4v` (`liana_taproot.phrase.txt`).

**What held (checked, no defect)**: a scratch program linked against the vendored md-codec 0.42.0 (`work-mkms/mdskew`) decoded every `*.phrase.txt` in descriptor-mnemonic's vector corpus and compared the id mk would stamp against the 0.48.7 `*.conformance.json`: 46 of 46 wire-version-4 cards give the identical `wallet_policy_id` / `wallet_descriptor_template_id`; the only 3 refusals are the wire-version-8 cards. So the skew produces no silently different stub on current version-4 cards.

**Fix direction**: bump mk-cli's md-codec to the sibling's 0.48.x (with the miniscript patch the md CLAUDE.md "Pins" hazard requires) and add a wire-version-8 `--from-md1` row; until then, name the version limit in the error and the manual.

**Verifier re-check** (source: `verify-3`)

### mk#1 — mk-cli's pinned md-codec 0.42.0 cannot read wire-version-8 md1

- Verdict: **CONFIRMED-but-downgraded**. Grade: **Low**. Category: **other**.
- Re-run: `mk encode --xpub xpub6DXuQW1…DPw --origin-path "m/48'/0'/0'/3'" --origin-fingerprint 73c5da0a
  --from-md1-set liana.md1` -> `error: md1 input rejected: wire-format version mismatch: got 8, expected 4`, rc=1.
  `crates/mk-cli/Cargo.toml:34` and `Cargo.lock:510-512` resolve md-codec 0.42.0;
  `vendor/md-codec/src/header.rs:38-44` accepts version 4 only.
- Why Low: it is a clean refusal, and the skew is a documented hazard (mnemonic-key CLAUDE.md, "md dependency
  skew"). The cost is real, though: for Liana-unspendable taproot policies the only mint route left is
  `--policy-id-stub`, which skips the membership and origin checks. Worth a pin bump before those shapes are used.

## mk#3: Non-ASCII characters in an md1 are read as codex32 symbols (old md-codec pin)

**Grade:** Low · **Category:** other · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `mk`)

### 3. A non-ASCII character in an md1 string is read as a codex32 symbol by `--from-md1` (md-codec 0.42.0) — Low — CONFIRMED — cross-repo (descriptor-mnemonic)

**Evidence**
- `vendor/md-codec/src/codex32.rs:73`: `let lc = c.to_ascii_lowercase() as u8;` truncates the code point, so U+0171 `ű` reads as `q` (0x71) and U+0238 `ȸ` as `8`.
- Fixed upstream in md-codec 0.48.5 ("A non-ASCII character no longer aliases a codex32 symbol", `crates/md-codec/src/codex32.rs:73` now checks `is_ascii`); mk-cli still resolves 0.42.0. mk's own mk1 reader refuses the same character (`error: invalid character ű at position 0`).

**Impact**
`mk encode --from-md1`, `--from-md1-set` and `mk verify --from-md1` accept an md1 string containing characters that are not in the alphabet and treat it as the card it aliases, at exit 0 with no note. The resulting stub is that of the aliased card, so no wrong binding results; the defect is that a transcription the sibling `md` now refuses is accepted here (validation skew on policy input).

**Reproduction**
```
$MK encode --xpub $X --origin-path "m/84'/0'/0'" --origin-fingerprint 73c5da0a --from-md1 md1yqpqqxzq2qwfv8urt848e
mk1qpztanpqqsq4t8nykfeutks2qvzg3vs7zq...qpy6guqnnkkwy3gmfp6
mk1qpztanppfdkdzdssxt9fh54wh8vsp2jdghv74kq2e9prxaxy2xnj2ng8vcf0k049epauw233j8nms      rc=0
$MK encode ... --from-md1 'md1yűpqqxzq2qwfv8urt848e'      # U+0171 in place of q
(identical two mk1 lines)                                  rc=0
$MK encode ... --from-md1 'md1yqpqqxzq2qwfv8urt84ȸe'      # U+0238 in place of 8
(identical two mk1 lines)                                  rc=0
$MK verify --in pk.txt --from-md1 'md1yűpqqxzq2qwfv8urt848e'
OK: mk1 string(s) decode cleanly (...)                     rc=0
```

**Fix direction**: closes with the md-codec bump of finding 1; add a regression row in mk-cli for a non-ASCII md1 on all three intake flags.

## X6: Non-ASCII characters read as codex32 symbols by mk-codec 0.4.1/0.5.0 used by consumers

**Grade:** Low · **Category:** other · **Status:** Reviewer-confirmed, not re-verified

**Reviewer write-up** (source: `cross`)

### X6. Non-ASCII characters are still read as codex32 symbols by the pinned ms-codec and mk-codec

- **Severity:** Low
- **Status:** CONFIRMED (probes)
- **Repos:** mnemonic-toolkit and mnemonic-gui (ms-codec 0.7.0, mk-codec 0.4.1); mnemonic-refugium and
  refugium-wallet (ms-codec 0.9.0, mk-codec 0.5.0); mnemonic-engrave and descriptor-mnemonic (mk-codec 0.5.0);
  refugium-wallet also md-codec 0.48.4 (the md fix is in 0.48.5; CHANGELOG only)
- **Evidence and reproduction:**
  - ms: fixed in ms-codec 0.10.1 (bughunt ms #7). `ms-codec/src/decode.rs:184` (0.7.0) and `:208` (0.9.0):
    `let lc = c as u8;`. A valid card with two characters changed, one of them to U+0133 (which truncates to `3`,
    the right symbol there):
    ```
    $ probe-tk ms-fix "ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc8sh9lf92n5fq3ĳs"
    Ok(("entr", "Entr([0, 1, ..., 15])", 1))          # 0.7.0: accepted, one correction reported
    $ probe-rf ms-fix  (same string)   -> Ok(..., 1)   # 0.9.0
    $ probe-msh ms-fix (same string)   -> Err(Error("codex32 parse error: InvalidChar('ĳ')"))   # HEAD
    ```
    The toolkit reaches this through `repair` (`crates/mnemonic-toolkit/src/repair.rs:1467,1540`). Its
    `apply_ms_corrections` (`repair.rs:1507-1527`) rebuilds the string from `chunk.to_lowercase()` (Unicode, not
    ASCII), so the "corrected" card it prints keeps the non-ASCII character (and shifts positions for a character
    whose lowercase is two characters, such as U+0130): the printed repair is not a valid ms1.
  - mk: fixed in mk-codec 0.5.1. `string_layer/bch.rs:664` (0.4.1) and `:673` (0.5.0) use `s.to_lowercase()`.
    An all-uppercase card with one `K` replaced by U+212A KELVIN SIGN:
    ```
    $ probe-tk mk-dec MK1QPN2ZNPQQSQ64W7VM4EUT<U+212A>S2Q... MK1QPN2ZNPP0F30...   -> Ok(KeyCard {...})   # 0.4.1
    $ probe-rf  (same)  -> Ok(KeyCard {...})                                         # 0.5.0
    $ probe-mkh (same)  -> Err(InvalidChar { ch: 'K', position: 21 })                # 0.6.0
    ```
- **Impact:** a card typed with a look-alike character is accepted (mk) or "repaired" (ms) instead of refused; in
  the toolkit the printed ms1 repair can be an invalid string. No wrong key or wrong seed results: the checksum
  still has to hold.
- **Fix direction:** move the pins (ms-codec >= 0.10.1, mk-codec >= 0.5.1); make `apply_ms_corrections` use
  ASCII lowercasing.

## SH#4: Related: the seedhammer Go port of `mk.Encode` lacks mk-codec 0.6.0's origin refusals

**Grade:** Low · **Category:** other · **Status:** Confirmed (reproduced)

**Reviewer write-up** (source: `seedhammer`)

### 4. `mk.Encode` mints key cards that mk-codec 0.6.0 refuses to mint

**Severity: Medium (low end). CONFIRMED on the Go side; Rust side from source and changelog.** Rust repo: mnemonic-key (mk-codec `bytecode/encode.rs`).

**Evidence**

- `mk/encode.go:81-131` (`encodeBytecode`) and `:154-180` (`compactFromXpub`) check stub count, depth and child number against the path, network, and fingerprint length only.
- Rust `crates/mk-codec/src/bytecode/encode.rs:123-155` (`check_key_rules`, mk-codec 0.6.0, 2026-11-22) also refuses: a depth-0 key with a non-zero parent fingerprint (`MasterParentFingerprintNonzero`), a depth-0 card whose origin fingerprint is not the key's own (`MasterFingerprintMismatch`), and a depth-1 card whose origin fingerprint is not the xpub's parent fingerprint (`OriginParentFingerprintMismatch`). `mk/mk.go:5` still names "mk-codec 0.2" as what the port tracks.
- Reachable where the fingerprint and xpub come from outside the device: the composer re-mint (`gui/composer_cards.go:41-53`) and `reStubMk1` (`gui/template_engrave.go:41-48`). The device's own derive paths pair the fingerprint with the key correctly.

**Impact.** The device can engrave an mk1 card whose stated origin fingerprint contradicts its own xpub. Rust decoders still read such a card (0.6.0 keeps decode unchanged), so nothing is lost, but the contradiction the Rust encoder now stops reaches steel through the Go port. The third 0.6.0 rule (path index below 2^31) is already met by `bip32.ParsePathElement`.

**Reproduction.**

```
$ cp work-sh/mk_zz_bughunt_test.go.txt mk/zz_bughunt_test.go
$ go test ./mk/ -run TestBH_ -count=1 -v
R3 depth-0, origin fp deadbeef != key fp: Go Encode err=<nil> strings=2
R4 depth-1, origin fp deadbeef != parent fp 5bf9f809: Go Encode err=<nil> strings=2
   Go Decode: fp=deadbeef path=m/0h err=<nil>
```

I did not build `mk`; the Rust refusals are read from `encode.rs` and `crates/mk-codec/CHANGELOG.md` ("Unreleased" block describing 0.6.0).

**Fix direction.** Port `check_key_rules` into `encodeBytecode` (including the all-zero-chain-code exception for the depth-0 rule) and update the provenance pin.

**Verifier re-check** (source: `verify-3`)

### seedhammer#4 — Go `mk.Encode` lacks mk-codec 0.6.0's origin refusals

- Verdict: **CONFIRMED-but-downgraded**. Grade: **Low**. Category: **other**.
- Re-run (`work-v3/sh`, `go test ./mk/ -run TestBH_ -count=1 -v`):
  `R3 depth-0, origin fp deadbeef != key fp: Go Encode err=<nil> strings=2`,
  `R4 depth-1, origin fp deadbeef != parent fp 5bf9f809: Go Encode err=<nil> strings=2`.
  Rust refuses both: `mnemonic-key/crates/mk-codec/src/bytecode/encode.rs:123-155` (`check_key_rules`, 0.6.0).
- Why Low: the device's own derive paths pair fingerprint and key correctly; only a re-mint of an imported card that
  already carries the contradiction reaches it (`gui/composer_cards.go:41-53`, `gui/template_engrave.go:41-48`).
  Decoders still read such a card; no key or address changes. Stale provenance pin (`mk/mk.go:5`, "mk-codec 0.2").

---

_Generated by [Claude Code](https://claude.ai/code) on 2026-11-25. Findings were not fixed; each fix needs its own review under this repository's gates._
