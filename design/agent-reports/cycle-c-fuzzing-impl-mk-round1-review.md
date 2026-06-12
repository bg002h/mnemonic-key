# Implementation Review — Cycle C phase 3 mk-codec (round 1)

Reviewer: Fable 5 architect agent (a7afe1fc940d87074), 2026-06-12.
Target: uncommitted mk-phase fuzz infra @ mnemonic-key (branch main).
Persisted verbatim per CLAUDE.md convention.

## Verdict: GREEN

0 Critical / 0 Important. The two mk-specific traps (fixed-csi determinism, lock alignment) were verified empirically rather than trusted from the report, and both are clean.

## Critical
- None.

## Important
- None.

## Minor
- **Single-string-target gate is faithful, not vacuous (the flagged design call resolves clean).** Empirically confirmed: a single real mk1 chunk drives all the way through bech32 parse + bytecode-header decode (it reads `total_chunks` from the header) before hitting the `received 1 chunks` check — so the seeds exercise the deep string/header surface the never-panic oracle protects, not trivially-rejected garbage. There is genuinely no single string `decode(&[s])` returns `Ok` for (minimal bytecode ~84B > 56B single-string ceiling). The Ok-gate would be the vacuous choice; the clean-Err gate is the honest one. Correct call; flagging only as documentation-completeness (already well-documented in the gen_corpus module doc).
- The pinned `nightly-2026-04-27` lacks the `rustfmt` component locally, so `cargo +nightly-2026-04-27 fmt --check` errors on a fresh machine (CI doesn't run fmt in fuzz/, so no impact). Sources are rustfmt-clean under stable/default-nightly. Non-blocking.

## Deliverable-conformance table
| Item | Conforms? | Notes |
|---|---|---|
| BUILD: both targets compile, gnu target | Yes | Clean recompile, zero warnings; both binaries present |
| mk1_decode oracle: sentinel `\n` split ≤8, `decode(&[&str])`, fixed-point | Yes | Uses `encode_with_chunk_set_id(card, FIXED_CSI=0)` — NOT `encode`; compares `KeyCard` values via `PartialEq` |
| **csi-determinism trap** | Yes (empirical) | Scratch-verified: wire bytes DO differ with csi (csi is on-wire), but decoded `KeyCard` value is identical for csi=0, 12345, 0xFFFFF — `KeyCard` has 4 fields, none is csi. Oracle cannot false-positive |
| mk1_decode_single oracle: whole input one string, same fixed-point | Yes | Identical wiring; fixed-point branch unreachable for single real card but stays wired (defensive) |
| Re-encode Err = panic (finding), not swallowed | Yes | `.expect("FINDING: ...")` on both re-encode and re-decode + `assert_eq!` |
| **LOCK alignment trap** | Yes (empirical) | Every shared dep mk-codec compiles matches root EXACTLY: bitcoin 0.32.8, bitcoin_hashes 0.14.1, bech32 0.11.1, secp256k1 0.29.1, secp256k1-sys 0.10.1, hex-conservative 0.2.2. Root's extra `hex-conservative 1.1.0` comes ONLY from miniscript 13.0.0 (mk-cli's dep), absent from the mk-codec/fuzz closure. `--locked` build stable |
| `bitcoin = "0.32"` direct fuzz dep — single version | Yes | Resolves to the same 0.32.8 mk-codec uses; no duplicate bitcoin in fuzz lock |
| CORPUS: gen_corpus passes, deterministic, no trailing `\n`, multi-chunk seed | Yes | Byte-identical across two runs; `.parts` seeds end on bech32 chars (no trailing 0x0a); 3-chunk seeds present |
| CI: build gate triggers + smoke matrix (both targets) | Yes | Triggers on fuzz/** + crates/mk-codec/src/** + self; both targets in smoke (mk found NO crash, unlike ms's held-out ms1_decode); upload-artifact@v5; actionlint clean |
| ci.yml path-collision | Yes | ci.yml has no `paths:`, runs `--workspace`/`--all` from root; root members explicit (no glob) — fuzz/ (own `[workspace]`) invisible |
| ISOLATION: root fmt ignores fuzz | Yes (empirical) | Misformatted a fuzz source → root `cargo +1.95.0 fmt --all --check` still exit 0; root build clean; crates/ untouched |
| rust-toolchain.toml cites no-root rule | Yes | Comment cites the ci.yml fmt-job no-root-toolchain rule and "do NOT promote to root" |
| .gitignore: target/artifacts/coverage ignored, corpus+lock committed | Yes | `fuzz/artifacts/` + `fuzz/coverage/` added; `**/target/` covers fuzz/target/ |
| BRING-UP re-proof (independent) | Yes | Planted reachable panic on mk1-prefixed input → fuzzer found it, crash artifact written; reverted byte-identical; clean re-run found nothing |

## Evidence log
- `KeyCard` (key_card.rs:23-54): `#[derive(PartialEq, Eq)]`, 4 fields (policy_id_stubs, origin_fingerprint, origin_path, xpub) — no chunk_set_id field (structural proof of csi-independence).
- csi probe (scratch, reverted): `card == decode(encode_with_chunk_set_id(card, csi))` for csi ∈ {0, 12345, 0xFFFFF} all true; `c0 == c12345` true; `wire(0) != wire(12345)` true — csi is on the wire but not in the value.
- single-chunk probe (scratch, reverted): all chunks of a real card → `Err(ChunkedHeaderMalformed("received 1 chunks, header declares total_chunks = 3"))`; corrupted chunk → clean Err. No single string decodes Ok.
- Lock comparison (exact `[[package]]` parse): all crypto shared deps OK; `hex-conservative 1.1.0` traced to miniscript 13.0.0 only; miniscript absent from fuzz lock.
- gen_corpus: two runs → SHA-256 of all 14 seeds byte-identical; `.parts` last-byte ∈ {0x79,0x6b,0x72,0x76} (no trailing NL); newline counts 1,1,2,2.
- Bring-up: `cargo fuzz run mk1_decode` found planted panic, artifact written; post-revert clean runs reached cov 1086 / 526 over 587k / 2.1M execs, no crash.
- actionlint `.github/workflows/fuzz-smoke.yml` → exit 0.
- Tree-as-found restored: removed 401 fuzzer-discovered scratch corpus entries (back to the 14 generator seeds, byte-identical to gen_corpus output), removed stray empty `fuzz/artifacts/`. Final `git status`: ` M .gitignore`, `?? .github/workflows/fuzz-smoke.yml`, `?? fuzz/`.

GREEN — cleared to commit.
