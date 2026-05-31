# R0 Architect Review — IMPLEMENTATION_PLAN_mk_derive_address.md

Reviewer: feature-dev:code-reviewer (opus). Reviewed the plan against the R1-GREEN SPEC, real
mk-cli/mk-codec source @ `9ab74f0`, the toolkit reference code, the GUI schema mirror, and
bitcoin 0.32.8 API. Every load-bearing code snippet, API signature, exit-code claim, fixture, and
lockstep target checked against ground truth. The core architecture, every cited bitcoin-0.32 API,
and the `render_address`/`infer_address_type` code all compile and match ground truth.

## Critical

**C1 — The plan's exit-code premise is factually wrong: mk-cli maps ALL clap parse errors to exit
64, not 2. Every test asserting "clap error → exit 2" will fail.** Plan Task 1.3 / 2.2 / self-review
line 288 assume clap's native exit 2 reaches the shell. Ground truth `crates/mk-cli/src/main.rs:62-67`
— clap `Err` is intercepted; `DisplayHelp`/`DisplayVersion` → 0, everything else → `ExitCode::from(64)`.
So `--count`/`--range` conflict and `--path`/`--index` group violations BOTH exit **64**, identical to
runtime `UsageError`. There is NO 2-vs-64 inconsistency. **Fix:** strike the "clap exits 2" language;
state clap parse errors route through main.rs's catch-all → exit 64; §5.5/§5.12 tests assert 64 (+
clap usage message on stderr). Simplifies the plan.

## Important

**I1 — Non-account-depth fixtures (esp. §5.4b leaf `m/84'/0'/0'/0/5`) cannot be built by pairing an
account xpub with a deeper path; `mk_codec::encode` rejects with `XpubOriginPathMismatch`. No
construction recipe given.** Ground truth `mk-codec/src/bytecode/encode.rs:41` — `xpub.depth as usize
!= path_depth || xpub.child_number != expected_child` → reject. `KeyCard::new` doesn't validate; the
failure surfaces at `encode()`. **Fix:** Task 1.1/1.2 must state each card's xpub depth + terminal
child must match `origin_path`. Account fixtures (44'/49'/84'/86' depth-3, 48'/87', testnet tpub) are
liftable from `crates/mk-codec/src/test_vectors/v0.1.json`. The leaf fixture's xpub must be the
forward-derived depth-5 child: `acct.derive_pub(&secp, &p("m/0/5"))` then paired with
`origin_path = m/84'/0'/0'/0/5`. Same for any depth-0 (master) fixture.

**I2 — Plan Task 4.1 targets an ambiguous `CHANGELOG.md`; the repo has two and the root one is the
wrong file.** `/CHANGELOG.md` is the **mk-codec** changelog (unchanged this cycle — must NOT touch).
`crates/mk-cli/CHANGELOG.md` is the **mk-cli** changelog, heading style `## [0.4.2] — <date>`. **Fix:**
pin Task 4.1 to `crates/mk-cli/CHANGELOG.md`, heading `## [0.6.0] — 2026-05-NN`; drop the "if…verify"
hedge (confirmed present).

## Minor
- **M1** — `AddressType` kebab rendering for `P2shP2wpkh` has no in-repo `rename_all` precedent
  (toolkit `ScriptType` uses manual matching). Already mitigated: Task 0.1 Step 1 asserts
  `get_name()=="p2sh-p2wpkh"` and catches a wrong render at Step 4. No change required.
- **M2** — SPEC §4 count rationale muddled: the manual deliberately documents 6 user-facing
  subcommands and excludes `gui-schema` (it did NOT undercount). "Six → eight" target is correct
  (6 + address + derive); fix the SPEC's "current 7 incl repair − undercount" prose.
- **M3** — SPEC §2 mis-cites `Secp256k1::verification_only()` as in `mk-cli/src/cmd/verify.rs` — mk-cli
  verify.rs uses no secp. Valid precedent is toolkit `verify_message.rs:55` (the plan's Authoritative
  facts cites it correctly). Drop the mk-cli verify.rs citation.
- **M4** — Plan Task 5.3: note the GUI mk.rs header (`v0.3.1`) and `pinned-upstream.toml:52`
  (`mk-cli-v0.4.2`) are ALREADY out of sync; bump BOTH to v0.6.0.

## Verified correct (no action)
`KnownHrp::{Mainnet,Testnets,Regtest}` spellings; `render_address` builders byte-match toolkit
`address_search.rs:42-49` (`p2wpkh`/`p2shwpkh` infallible here — `.to_string()` chaining correct);
`to_pub(self)`/`to_x_only_pub(self)` on `Xpub: Copy`; `derive_pub<C:Verification,P:AsRef<[ChildNumber]>>`
+ `Secp256k1<VerifyOnly>: Verification`; `DerivationPath::as_ref()`, `ChildNumber::Hardened{index}`,
`is_hardened()`; `infer_address_type` traced against all §5 cases (no off-by-one, multisig-before-len
ordering correct); gui-schema numeric→"text" tolerated; GUI `repair` genuinely un-mirrored; CI gates
fmt+clippy on stable (fmt authoritative, confirms codec-vs-toolkit note); bitcoin 0.32.8; `Xpub.depth:
u8`, `.network: NetworkKind`; phasing test-before-impl + per-phase review-gates present; every §5 test
maps to a task.

**VERDICT: RED (1C/2I)**

---

## Fold applied (controller, verified @ 9ab74f0)
- **C1:** confirmed main.rs:62-67 → all clap parse errors `ExitCode::from(64)`. Plan Task 1.3/2.2 +
  self-review reworded: clap parse errors exit 64 (unified with UsageError); §5.5/§5.12 tests assert 64.
- **I1:** confirmed encode.rs:41 guard. Tasks 1.1/1.2 gain a fixture-construction note (xpub depth +
  terminal child must match origin_path; account fixtures liftable from test_vectors/v0.1.json; leaf =
  forward-derive `acct.derive_pub(m/0/5)` before encode).
- **I2:** confirmed two CHANGELOGs. Task 4.1 pinned to `crates/mk-cli/CHANGELOG.md`, heading
  `## [0.6.0] — <date>`; root mk-codec changelog untouched.
- **M2/M3:** SPEC §4 count prose corrected (6 documented + 2, gui-schema excluded); SPEC §2 secp
  citation fixed to toolkit `verify_message.rs:55`.
- **M4:** Plan Task 5.3 notes the pre-existing header-vs-pin skew (bump both). M1 left (test mitigates).
