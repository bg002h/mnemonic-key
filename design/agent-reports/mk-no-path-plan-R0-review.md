# R0 Review — IMPLEMENTATION_PLAN_mk_no_path_support.md

Opus architect (feature-dev:code-reviewer), mandatory pre-impl R0 gate. Branch
`mk-no-path-support`, base `main` `5c2bc8c`. Verified plan code against live source +
bitcoin 0.32. Persisted by controller (review agent read-only).

## Headline confirmations (file:line)
- **Task 0.1 before-text matches:** `path.rs:114` is `if count == 0 || count > MAX_PATH_COMPONENTS {`. `read_u8`/`MAX_PATH_COMPONENTS` in scope. `count==0` loop runs zero times → `Ok(DerivationPath::from(vec![]))` = empty path; round-trips `encode_path`'s `[0xFE,0x00]`. Module rustdoc "1-byte component count (1..=10)" at `:18-20`. `DerivationPath`/`FromStr` in test mod.
- **Task 0.2 matches:** `.expect("origin_path must be non-empty …")` at `xpub_compact.rs:92-95`; `ChildNumber` already imported (`:18`). `synthetic_xpub(&empty)` → depth 0 / child `Normal{0}` (`test_helpers.rs:26-31`). T4 compiles.
- **Task 0.3 matches:** guard `encode.rs:33-42` verbatim the plan "before". `ChildNumber` ABSENT from production imports (`:14-18`); added `use` needed-not-duplicate, no collision with test-mod import `:68`. `Error::XpubOriginPathMismatch` fields `{xpub_depth:u8, path_depth:u8, xpub_child:ChildNumber, path_child:Option<ChildNumber>}` → T6 destructure type-checks.
- **Task 0.4:** `crate::bytecode::decode::decode_bytecode` correct (`decode.rs:19`, `pub`), routes through `decode_path`+`reconstruct_xpub`. `key_card.rs` fence `:48-51`.
- **Phase 1 citations all live:** `:172`/`:229`/`:237`/`:257-258`/`:263`/`:285`/`:294`/`:303`; E10 sites `depth_child_enforcement.md:30`+`:57` present.
- **Phase 2:** mk-codec `0.3.2`, mk-cli `0.4.3` + pin `0.3.2` all match; publish order mk-codec→mk-cli correct.
- **No-variant mirrors correct:** `error_coverage.rs`/`mk-cli error.rs:133` already carry the variant; no edits, as planned.
- **MISS-check clean:** empty-path sampling flows only through `xpub_strategy` (the T9-fixed helper); `*comps.last().unwrap()` sites in `bch_adversarial.rs:29,195` use hardcoded non-empty paths. No mk-cli test asserts a no-path card is rejected.

## CRITICAL
**C1 — Task 0.4 Step 2 (T9) `xpub_strategy` edit will NOT compile as written** (plan vs `tests/common/mod.rs:68-70`). The live binding is `let child_number = *components.last().expect("path is non-empty …");`. `components.last()` is `Option<&ChildNumber>`. A literal `.expect → .unwrap_or(ChildNumber::Normal{index:0})` swap fails twice: `Option::<&ChildNumber>::unwrap_or` needs a `&ChildNumber` arg (owned value = E0308), and the leading `*` then derefs a non-reference. Correct rewrite must drop the `*` and insert `.copied()` (as `synthetic_xpub`/the guard already do):
```rust
let child_number = components.last().copied().unwrap_or(ChildNumber::Normal { index: 0 });
```
The plan's instruction "(keep `let child_number = ...` binding)" omits the load-bearing `*`, so a literal substring replacement hard-fails. Fold: specify the `*`-drop + `.copied()` rewrite.

## IMPORTANT — None.

## MINOR
- **M1** off-by-one citations: `rejects_path_count_zero` is `:249-258` (plan `:248`), `rejects_empty_origin_path` `:155-170` (plan `:152`). Land on the right tests; re-grep at edit time (standing instruction).
- **M2** T9 run cmd `--test '*'` exercises the string-layer API (fine); T8 `--lib` is the bytecode-level proof — keep both.
- **M3** `cargo +stable fmt -p mk-codec -p mk-cli -- --check` correct + covers `tests/` target.

## VERDICT: RED (1C/0I/3M)
Plan faithful to the R1-GREEN spec; every citation/type/call-site checks out except C1 (literal T9 edit drops the `*` deref → guaranteed compile failure). One-line fold (drop `*`, add `.copied()`), then re-dispatch. No other blocker.
