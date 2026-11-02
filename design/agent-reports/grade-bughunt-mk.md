**⚠ Important x4: #1 (mk encode silent no-fingerprint card), #3 (mk address abort), #4 (mk encode panic), #5 (non-ASCII accepted by decode_string). #2 graded N (no spec violation). fix-01 as drafted breaks 11 batch tests; use the scoped guard below.**

# Grade + rulings: mk bug hunt (triage c2c4160, RED tests 23828ca)

architect grade-mk-bughunt: 4 answered (Q1, Q2, Q3, Q5; triage has no Q4); 0 prior rulings changed.

Evidence base: worktree `/scratch/code/shibboleth/mk-worktrees/bughunt-triage` @ 23828ca. A scratch copy (`git archive`, not a worktree) is at `/scratch/cache/bughunt/mk/grade-src`, with its target in `/scratch/cache/bughunt/mk/grade-target`, and has fix-02..05 plus the scoped #1 guard applied. External texts: `/scratch/cache/bughunt/mk/bips/bip-0093.mediawiki` and `bip-0173.mediawiki` (curl from bitcoin/bips master).

## 1. fix-01 as drafted is a regression (blocking for the FIX doc)

fix-01 refuses `(None, false)` unconditionally, but the `--keys`/`--in` batch path also reaches that arm, because each record carries its own fingerprint (`encode.rs` 214-219, per-record `Some(r.fingerprint)`).
- All 5 patches applied (`/scratch/cache/bughunt/mk/grade-allfix.log`): `397 tests run: 386 passed, 11 failed`. The 11 failures are 9 `keys_batch` tests and 2 `channels` tests (`in_on_encode_equals_keys`, `in_errors_name_in_not_keys`). The log holds the message `pass --origin-fingerprint <hex> or --privacy-preserving explicitly` 9 times.
- Scoped guard (`/scratch/cache/bughunt/mk/fix-01-scoped.diff`): a new first arm `(None, false) if key_source.is_none() => return Err(UsageError(..))`, with `(None, false) => None` kept as the fallthrough. Result: `397 tests run: 397 passed` (`grade-allfix2.log`), including bughunt_mk_01..06.
- The FIX doc must carry the scoped form and name those 11 tests as the guard against this regression.

## 2. Grades (likelihood test as quoted in the brief)

| # | verb (placement) | grade | reasoning |
|---|---|---|---|
| 1 | `mk encode` (stays in mk) | **I** | Fails open. A single-card encode with neither flag exits 0 and mints a card without the fingerprint, a choice the holder never made. SPEC §6 l.401 says "Encoders SHOULD expose this as an explicit choice rather than a default." Gives a wrong card versus intent, so it cannot be Minor. Printed commands are unaffected: every mr-printed `mk encode` carries `--origin-fingerprint` or `--privacy-preserving` (kitrun.rs, plates.rs, kits.rs, chain_text.rs; manual 45-mr.md l.482, l.4112), and manual 44-mk.md l.61-67 carries `--origin-fingerprint`. Downstream, toolkit bundle.rs:2709-2712 already refuses a card with no fingerprint unless `--privacy-preserving` is set, so the harm stops at the next tool. Still I: a wrong card leaves the encoder. |
| 2 | codec decode (`decode_path`, library) | **N** | Not a wrong path. `leb128_decode_u32` (path.rs 148-171) keeps the overflow checks (shift≥35, value>u32::MAX), so a non-minimal form always decodes to the same u32 as the minimal form, within the 5-byte bound. No check exists that fails to run: SPEC §4 rule 6 (l.326) names only "LEB128 overflow". The spec already allows a second encoding of the same path (§3.5 l.274: an explicit path equal to a table entry "would normally use the 1-byte standard-table indicator", and decode accepts the explicit form). Nothing is bound to the bytes: `chunk_set_id` is opaque ("Decoders MUST accept any 20-bit value", l.114). The precedent agrees: md-codec enforces NonMinimalWireVersion at encode only, "never at decode" (descriptor-mnemonic error.rs ~694-712). What remains is a spec-text gap, closed by Q2. |
| 3 | `mk address` (moves to toolkit) | **I** | Aborts on holder input. `--count 2147483648` or `--range 0,2147483647` gives `memory allocation of 8589934592 bytes failed` with exit 134 (triage evidence). This is not a usage error. The doc comment at address.rs 199-201 says "never ... a multi-gigabyte allocation", so the code claims a check it does not make. C/I in a moving verb is fixed now. |
| 4 | `mk encode --from-md1-set` (stays) | **I** | Panics on holder input: `encode.rs:480` `s[..3]` on a line whose byte 3 is not a char boundary, exit 101. The manual (l.43) says non-md1 lines "are skipped". |
| 5 | codec `decode_string` (stays; reached by `mk decode` and every reading verb) | **I** | The decoded card is right, but a validity check silently does not run. BIP-93 l.68-78: a codex32 string uses "the base-32 character set from BIP-0173". l.82: it "MUST be entirely uppercase or entirely lowercase". l.157-159: non-bech32 characters get a *suggestion* or are treated as *erasures*, never silently mapped. BIP-173 l.192: "Decoders MUST NOT accept ... mixed case". In the code, `case_check` is ASCII-only and the `!c.is_ascii()` check at bch.rs:690 runs *after* `s.to_lowercase()` (l.673). So U+212A (category Lu) passes both checks and becomes `k`. Measured on 23828ca by the triage: `mK1...` with U+212A decodes, exit 0. The brief's rule ("a check that silently does not run stays C/I however rare") decides it. Harm is nil and the fix is a few lines. |

Verb placement and deferral: there are no Minors, so nothing defers to the toolkit migration. #3 is the only finding in a moving verb, and it is fixed now as an I.

## 3. Rulings (exact behaviour)

**Q1 (#1): usage error, not a warning.** Priority 1 (correct results): a warning still emits the card.
- Behaviour: `mk encode` without `--keys`/`--in` refuses when neither `--origin-fingerprint` nor `--privacy-preserving` is given.
  - Exit 64.
  - stderr: `error: pass --origin-fingerprint <hex> or --privacy-preserving explicitly` (fix-01 text, measured on the scoped build).
  - No stdout.
- `--keys`/`--in` are unchanged: each record's `[fp/path]` supplies the fingerprint, and the existing refusals of both flags with `--keys`/`--in` (encode.rs 145-168) stand.
- Spec: §6 l.401. Replace "Encoders SHOULD expose this as an explicit choice rather than a default." with "Encoders SHOULD expose this as an explicit choice rather than a default. The reference CLI makes it required: a single-card `mk encode` that passes neither `--origin-fingerprint` nor `--privacy-preserving` is refused with exit 64."
- Manual `docs/manual/src/40-cli-reference/44-mk-cli.md` (origin/master line numbers):
  - l.31 synopsis becomes `mk encode --xpub <XPUB> --origin-path <PATH> (--origin-fingerprint <HEX> | --privacy-preserving) [OPTIONS]`.
  - l.39 becomes `| --origin-fingerprint <HEX> | 8-hex-char master fingerprint; mutually exclusive with --privacy-preserving; a single-card encode needs one of the two (exit 64 otherwise); refused with --keys/--in, whose records carry their own |`.
  - l.47 becomes `| --privacy-preserving | emit without master fingerprint; mutually exclusive with --origin-fingerprint; a single-card encode needs one of the two (exit 64 otherwise) |`.
  - Each changed row carries the chapter's `(unreleased: mk-cli after <current release>: ...)` marker by its existing convention.
- The clap help strings for both flags change to match. If the GUI schema mirrors help text, that is checked in the FIX doc.

**Q2 (#2): minimal LEB128 is an encoder rule, not a decoder refusal.**
- Refusing at decode adds no correctness, because the same value is always decoded. It could strand a card made by a non-reference encoder, and it costs a code change, a new negative vector and a test. The md-codec precedent points the same way.
- No code change. fix-02 is not applied.
- Spec, §3.5 after l.272, add: "Encoders MUST emit each component in minimal LEB128 (no trailing `0x80`-continued zero groups). Decoders accept a non-minimal component of at most 5 bytes and decode it to the same u32; only overflow is refused (§4 rule 6)."
- Test change: rename `bughunt_mk_02_leb128_non_minimal_refused` to `bughunt_mk_02_leb128_non_minimal_decodes_to_same_value`. It asserts that `[fe,01,80,00]` decodes to the path with component 0, and that `encode_path` of that path yields `[fe,01,00]` (the minimal form; the triage measured exactly this).
- No manual row changes.

**Q3 (#3): cap at 1,000,000 addresses per chain.** That is far above any gap-limit scan and keeps memory bounded. Measured on the scoped build: `--count 1000000 --chain both` completed (2,000,002 stdout lines, exit 0).
- `--count N`, N > 1000000 (after the existing BIP-32 ceiling check):
  - exit 64
  - `error: --count N too large (max 1000000)`
  - measured: `--count 1000001` gives that text with exit 64.
- `--range A,B` with B−A+1 > 1000000 (that is, B−A ≥ 1000000), checked after the existing ceiling check:
  - exit 64
  - `error: --range too large (max 1000000)`
  - measured: `--range 0,1000000` gives that text with exit 64, and `--range 5,1000004` (1,000,000 indices) exits 0.
- The ceiling text is unchanged and still fires first: `--count 2147483649` gives `... exceeds the BIP-32 normal-index ceiling (max 2147483648)`.
- The address.rs 196-201 doc comment is corrected to name the 1,000,000 cap.
- No spec sentence: `mk address` is CLI-only and has no rule in SPEC_mk_v0_1.md. If `design/SPEC_mk_derive_address.md` states the count semantics, the FIX doc adds the cap there too, in one sentence.
- Manual l.501 becomes `| --count <N> | number of addresses per chain, starting at index 0 (default 10, max 1000000); conflicts with --range |`.
- Manual l.502 becomes `| --range <A,B> | inclusive index range A..=B, at most 1000000 indices; conflicts with --count |`.

**Q4: none in the triage.** The brief's "Q1-Q5" maps onto Q1, Q2, Q3 and Q5. #4 needs no ruling: its behaviour is the documented skip, exit 0, with no text change, spec change or manual change (l.43 is already right).

**Q5 (#5): refuse every non-ASCII character, with no Unicode case folding.**
- fix-05 as drafted reports `position` as a byte index over the *whole* string. That breaks `Error::InvalidChar`'s documented meaning ("0-indexed position within the data part (chars after `mk1`)", error.rs 52). So the ruling is:
  - a non-ASCII character after the last `1` gives `Error::InvalidChar{ch, position}`, where position is its 0-based *character* index in the data part. Text: `invalid character <ch> at position <i>`. `mk decode` exit 1, the same as the other codec errors; measured exit 1 on the scratch build.
  - a non-ASCII character at or before the last `1` gives `Error::InvalidHrp(<hrp as typed>)`. Text: `invalid HRP: <hrp>`, exit 1.
  - the character check runs before the length lookup, so a multi-byte character never shows up as `InvalidStringLength`. Every other path is unchanged.
- Implementation shape, which the FIX doc words: replace `s.to_lowercase()` with `s.to_ascii_lowercase()` and validate data-part chars before `bch_code_for_length`.
- Spec, §2.1 after l.38, add: "An mk1 string contains only ASCII characters: the HRP, the separator and the BIP 173 bech32 alphabet, all in one case (BIP 93). Decoders MUST refuse any other character, including a non-ASCII letter whose Unicode lowercase is ASCII (e.g. U+212A KELVIN SIGN), and MUST NOT apply Unicode case folding."
- Tests: keep `bughunt_mk_05` (it asserts refusal only). Add one row that asserts the variant and position for U+212A in the HRP (`InvalidHrp`) and in the data part (`InvalidChar` with its character index).
- No manual row changes.

## 4. FIX grouping

**One FIX doc, small-fix track: `FIX-mk-bughunt-2026-11` (mnemonic-key plus the toolkit manual in lockstep).**
- It covers #1 (scoped guard), #3, #4 and #5, plus the #2 spec sentence and test rename.
- Spec delta: 3 sentences (§2.1, §3.5, §6).
- No new flag, type, command or error variant: `UsageError`, `InvalidChar` and `InvalidHrp` already exist.
- Repos: one, plus the manual (44-mk-cli.md l.31, 39, 47, 501, 502).
- It must list:
  - the 11 batch tests from §1 as tests that must stay green;
  - the RED rows bughunt_mk_01, 03, 04, 05, the renamed 02 and the new Q5 variant row, each with the expected output above.
- Review class: A (holder-facing text and manual) plus a codec decode change. Per Token economy 2, the post-loop whole-diff is one Sonnet slice unless a survivor maps to a spec rule. The Q1 and Q5 rules are spec rules, so expect the Opus s1 if mutants survive there.

No full-track doc is needed.

## 5. Follow-ups (not gating; owning phase in brackets)

- mnemonic-gui `src/schema/mk.rs`: the encode form lets a user submit neither fingerprint flag. After the fix that gets exit 64 (fails closed). Make the pair a required choice. [mnemonic-gui next schema sync]
- Noted, not graded: when stdout was piped, the grouped `mk1qp 22l3p ...` copies and `group size/separator` lines appeared on the terminal (that is, not on stdout). The triage §4 says they are on stdout. Recon should settle where they go. [ownerless]

## 6. Open questions

None for Brian. Every point fell inside the architect remit.

Counts: 5 graded (CONFIRMED); C 0, I 4 (#1, #3, #4, #5), M 0, N 1 (#2); rulings 4 (Q1, Q2, Q3, Q5); FIX docs 1 (small-fix track); Minors deferred to migration 0; follow-ups 2.
