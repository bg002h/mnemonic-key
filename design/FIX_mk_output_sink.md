# FIX-mk-output-sink-2026-11: a stdout that cannot take mk's output (crash sweep F1, Important)

Small-fix track. Repo: mnemonic-key (branch `fix/mk-output-sink`, from main
`2fc367c560013667180ee19572bbfad544d8cd88`), plus one sentence in the toolkit
manual `docs/manual/src/40-cli-reference/44-mk-cli.md` in lockstep (a separate
toolkit change; the mk build does not edit the toolkit).

Inputs: coordinator `agent-reports/grade-mk-crash-sweep-1.md` (Q1 F1 rulings,
Q4 the rule, Q5 the fix; these bind) and, from the same grade, the built and
run fix `f1-fix.diff` and rows `f1-red-rows.diff` (copied byte for byte into
section 3 and section 4 below). The architect ruling on the exit sets is
`agent-reports/architect-crash-sweep-exit-sets.md`.

Everything below is transcribed from the grade. The grade built this
exact diff and ran it both ways (RED: 6 tests run, 1 passed, 5 failed; GREEN:
workspace 405 tests run, 405 passed). Where the grade did not decide a point
that a holder can see, it is a STOP in section 10, not a choice.

**No STOP is open:** S1 is ruled (section 10). Every other point is decided.

## 1. Spec delta (exact texts)

There is no change to the wire spec (`design/SPEC_mk_v0_1.md`): this is CLI
behaviour only. The holder-visible texts are these three, all from the grade:

1. Stderr line (Q1 ruling b, and Q5 "Stderr line"), printed once, at exit,
   with a newline:

   > mk: cannot write the output: <error>

   `<error>` is the `io::Error`'s Display, e.g.
   `No space left on device (os error 28)`. The prefix is `mk:` (mr's form,
   `45-mr.md:48`), not mk's own `error:`.

2. CHANGELOG, `crates/mk-cli/CHANGELOG.md`, under `[Unreleased]` after the
   line `Next release: 0.15.0`, in `### Changed` (it appears verbatim in the
   diff of section 3.2):

   > **A stdout that cannot take the output no longer panics.** Every verb and `--help`/`--version` wrote with `println!`, which panicked (exit 101, "failed printing to stdout") on a full disk or `> /dev/full`. Now any stdout write error other than a closed pipe prints `mk: cannot write the output: <error>` on stderr and exits 1; a verb that was already failing keeps its own code. A reader that closes the pipe early (`mk address … | head -3`) only stops the output, and the exit code is the verb's own, not death by SIGPIPE (141) as in 0.14.0; on Windows, where SIGPIPE does not exist, this removes the closed-pipe panic. `tests/output_errors.rs`.

3. Manual sentence (section 8).

Version: the diff bumps mk-cli to `0.15.0-dev` (`Cargo.toml`, `Cargo.lock`);
`tests/version_is_honest.rs` requires that bump once `[Unreleased]` is
non-empty. No other mk change is in flight (the brief's statement; the base
`[Unreleased]` is empty and the only other mk worktrees are detached analysis
checkouts at the same head), so no other version claim competes. The release
prep that drops `-dev` is not part of this fix.

## 2. Behaviour, refusals, exit codes, error variants

The rule (grade Q4, and Q1 rulings a to e). One write path records the first
stdout error; `--help` and `--version` go through the same path.

- Closed reader (`ErrorKind::BrokenPipe`): the output stops, nothing is
  printed about it, the exit code is the verb's own. No death by signal: mk
  drops `restore_default_sigpipe` (section 3.6).
- Any other stdout write error, from a verb or `--help`/`--version`: the
  output stops; at exit `mk: cannot write the output: <error>` is printed on
  stderr. A code that means "the run succeeded and stdout is the result" (0,
  and `repair`'s 5) becomes **1** (mk's existing I/O-error code); a failing
  code is kept.
- Stderr write errors are ignored (this covers `2>&1 | head` once SIGPIPE is
  not restored).
- No new exit code, no new error variant, no flag change.

| # | situation | stdout | stderr | exit | row |
|---|---|---|---|---|---|
| 1 | closed reader, verb's code 0 (`gui-schema`, `vectors`, `encode`, `address`) | stops, no text | no `cannot write`, no panic | 0 | `closed_reader_exits_with_the_verbs_code` |
| 1b | closed reader, `repair` that repaired (code 5) | stops | none | 5 (not 1, not 141) | `closed_reader_exits_with_the_verbs_code` |
| 2 | other stdout error (`/dev/full`, ENOSPC), verb's code 0 | stops | `mk: cannot write the output: No space left on device (os error 28)` last line | 1 | `full_stdout_exits_1_with_the_line` |
| 2b | the same, `repair` that repaired (code 5) | stops | the line | 1 (not 5) | `full_stdout_after_a_repair_exits_1_not_5` |
| 3 | the same, verb already failing (`address --json --count 1000001`: the exit-64 envelope goes to stdout) | stops | the line | 64 (kept) | `full_stdout_keeps_a_failing_code` |
| 4 | `--help` / `--version`, closed reader | stops | none | 0 | `closed_reader_exits_with_the_verbs_code` |
| 5 | `--help` / `--version` (and `decode --help`), other stdout error | stops | the line | 1 | `full_stdout_exits_1_with_the_line` |
| 6 | stderr write error (stdout and stderr both closed: `encode`'s notes, `vectors --out`'s "wrote N vector file(s)") | as case 1 | ignored | 0 | `closed_reader_on_stdout_and_stderr_exits_0` |
| 7 | clap parse error (printed on stderr) | none | clap's text; a stderr failure is ignored | 64 (unchanged) | none new (no change in behaviour) |

A short write (`ulimit -f 1`) leaves a cut file but exits 1 with
`mk: cannot write the output: File too large (os error 27)` (grade, run on
the fixed binary; not a row).

## 3. Production changes (exact)

The six blocks 3.1 to 3.6 are `f1-fix.diff` split by file, in the diff's own
order, byte for byte: concatenated they equal the grade's file
(sha256 `0bb0fb557ebb5144b7cae1ab7a05b333fec6627d180d6dc798bc62cc6f4cc5c8`, checked by the gate). Apply them with `git apply`.
They touch `src/out.rs` (new, 74 lines), `src/main.rs`,
`src/process_hardening.rs` (the SIGPIPE function removed), `CHANGELOG.md`,
`Cargo.toml`, `Cargo.lock`. Block 3.7 is not part of the grade's file.

### 3.1 `Cargo.lock` (0.15.0-dev)

```diff
diff --git a/Cargo.lock b/Cargo.lock
index 4f254cb..bcdd745 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -537,7 +537,7 @@ dependencies = [
 
 [[package]]
 name = "mk-cli"
-version = "0.14.0"
+version = "0.15.0-dev"
 dependencies = [
  "assert_cmd",
  "bitcoin",
```

### 3.2 `crates/mk-cli/CHANGELOG.md`

```diff
diff --git a/crates/mk-cli/CHANGELOG.md b/crates/mk-cli/CHANGELOG.md
index 5619f4a..99fbceb 100644
--- a/crates/mk-cli/CHANGELOG.md
+++ b/crates/mk-cli/CHANGELOG.md
@@ -9,6 +9,21 @@ and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.h
 
 ## [Unreleased]
 
+Next release: 0.15.0
+
+### Changed
+
+- **A stdout that cannot take the output no longer panics.** Every verb and
+  `--help`/`--version` wrote with `println!`, which panicked (exit 101,
+  "failed printing to stdout") on a full disk or `> /dev/full`. Now any stdout
+  write error other than a closed pipe prints
+  `mk: cannot write the output: <error>` on stderr and exits 1; a verb that was
+  already failing keeps its own code. A reader that closes the pipe early
+  (`mk address … | head -3`) only stops the output, and the exit code is the
+  verb's own, not death by SIGPIPE (141) as in 0.14.0; on Windows, where
+  SIGPIPE does not exist, this removes the closed-pipe panic.
+  `tests/output_errors.rs`.
+
 ## [0.14.0] — 2026-11-04
 
 ### Added — chunk_set_id verification and correction reporting (2026-08-31)
```

### 3.3 `crates/mk-cli/Cargo.toml`

```diff
diff --git a/crates/mk-cli/Cargo.toml b/crates/mk-cli/Cargo.toml
index 01191db..030fe79 100644
--- a/crates/mk-cli/Cargo.toml
+++ b/crates/mk-cli/Cargo.toml
@@ -1,6 +1,6 @@
 [package]
 name = "mk-cli"
-version = "0.14.0"
+version = "0.15.0-dev"
 edition.workspace = true
 rust-version.workspace = true
 license.workspace = true
```

### 3.4 `crates/mk-cli/src/main.rs`

```diff
diff --git a/crates/mk-cli/src/main.rs b/crates/mk-cli/src/main.rs
index 7dfc4f5..bc745a5 100644
--- a/crates/mk-cli/src/main.rs
+++ b/crates/mk-cli/src/main.rs
@@ -6,6 +6,10 @@
 
 #![allow(missing_docs)] // mk-cli is binary-only; mirror md-cli/ms-cli precedent.
 
+// First, so its print macros shadow std's in every module below.
+#[macro_use]
+mod out;
+
 mod cmd;
 mod error;
 mod format;
@@ -59,8 +63,6 @@ enum Command {
 fn main() -> ExitCode {
     // argv-hardening: deny other-UID /proc/$PID/cmdline reads + core dumps.
     process_hardening::set_non_dumpable();
-    // Die quietly on EPIPE instead of panicking mid-bundle (see the fn's docs).
-    process_hardening::restore_default_sigpipe();
     let cli = match Cli::try_parse() {
         Ok(cli) => cli,
         Err(e) => {
@@ -69,10 +71,16 @@ fn main() -> ExitCode {
             // Output is on stdout and the canonical Unix exit is 0. The
             // catch-all 64 below preserves the carve-out for real parse
             // errors.
-            e.print().ok();
+            // --help and --version go to stdout under the same write-error
+            // rule as the verbs (`out`); a parse error goes to stderr.
+            if let Err(w) = e.print() {
+                if !e.use_stderr() {
+                    out::failed(&w);
+                }
+            }
             return match e.kind() {
                 clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
-                    ExitCode::SUCCESS
+                    ExitCode::from(out::exit_code(0))
                 }
                 _ => ExitCode::from(64),
             };
@@ -94,13 +102,14 @@ fn main() -> ExitCode {
         Command::GenMan(a) => cmd::gen_man::run(a),
     };
 
-    match result {
-        Ok(code) => ExitCode::from(code),
+    let code = match result {
+        Ok(code) => code,
         Err(e) => {
             emit_error(&e, json_mode);
-            ExitCode::from(e.exit_code())
+            e.exit_code()
         }
-    }
+    };
+    ExitCode::from(out::exit_code(code))
 }
 
 fn is_json_mode(cmd: &Command) -> bool {
```

### 3.5 `crates/mk-cli/src/out.rs` (new)

```diff
diff --git a/crates/mk-cli/src/out.rs b/crates/mk-cli/src/out.rs
new file mode 100644
index 0000000..c8529bb
--- /dev/null
+++ b/crates/mk-cli/src/out.rs
@@ -0,0 +1,74 @@
+//! The one stdout path. Every `print!`/`println!` in mk-cli resolves to the
+//! macros below (declared `#[macro_use]` first in `main.rs`, so they shadow
+//! std's in every module), and so do `eprint!`/`eprintln!`.
+//!
+//! std's macros panic (exit 101) when a write fails. Here a write error stops
+//! the output instead. A reader that closed the pipe (`mk … | head -3`,
+//! `ErrorKind::BrokenPipe`) only stops it: the exit code is the verb's own.
+//! Any other stdout error (a full disk, `> /dev/full`) is kept, and `main`
+//! prints "mk: cannot write the output: <error>" and exits 1 (crash sweep F1,
+//! grade-mk-crash-sweep-1). A stderr write error is ignored.
+//! `tests/output_errors.rs` pins both cases and that no other file of the
+//! crate names stdout.
+
+use std::io::Write;
+use std::sync::OnceLock;
+use std::sync::atomic::{AtomicBool, Ordering};
+
+static CLOSED: AtomicBool = AtomicBool::new(false);
+static WRITE_ERROR: OnceLock<String> = OnceLock::new();
+
+/// Writes to stdout; after the first failure, writes nothing.
+pub(crate) fn stdout_write(args: std::fmt::Arguments<'_>) {
+    if CLOSED.load(Ordering::Relaxed) {
+        return;
+    }
+    let mut o = std::io::stdout().lock();
+    if let Err(e) = o.write_fmt(args).and_then(|()| o.flush()) {
+        failed(&e);
+    }
+}
+
+/// Records a failed stdout write (also used for clap's `--help`/`--version`).
+pub(crate) fn failed(e: &std::io::Error) {
+    CLOSED.store(true, Ordering::Relaxed);
+    if e.kind() != std::io::ErrorKind::BrokenPipe {
+        let _ = WRITE_ERROR.set(e.to_string());
+    }
+}
+
+/// Writes to stderr, ignoring a failure.
+pub(crate) fn stderr_write(args: std::fmt::Arguments<'_>) {
+    let _ = std::io::stderr().lock().write_fmt(args);
+}
+
+/// The process exit code: `code`, unless a stdout write failed for a reason
+/// other than a closed pipe. Then the error is printed, and a code that says
+/// the output is the result (0, and `repair`'s 5) becomes 1; a failing code
+/// is kept.
+pub(crate) fn exit_code(code: u8) -> u8 {
+    match WRITE_ERROR.get() {
+        None => code,
+        Some(e) => {
+            stderr_write(format_args!("mk: cannot write the output: {e}\n"));
+            if code == 0 || code == 5 { 1 } else { code }
+        }
+    }
+}
+
+macro_rules! print {
+    ($($a:tt)*) => { $crate::out::stdout_write(format_args!($($a)*)) };
+}
+macro_rules! println {
+    () => { $crate::out::stdout_write(format_args!("\n")) };
+    ($($a:tt)*) => { $crate::out::stdout_write(format_args!("{}\n", format_args!($($a)*))) };
+}
+// Unused today; defined so a future `eprint!` cannot reach std's.
+#[allow(unused_macros)]
+macro_rules! eprint {
+    ($($a:tt)*) => { $crate::out::stderr_write(format_args!($($a)*)) };
+}
+macro_rules! eprintln {
+    () => { $crate::out::stderr_write(format_args!("\n")) };
+    ($($a:tt)*) => { $crate::out::stderr_write(format_args!("{}\n", format_args!($($a)*))) };
+}
```

### 3.6 `crates/mk-cli/src/process_hardening.rs`

```diff
diff --git a/crates/mk-cli/src/process_hardening.rs b/crates/mk-cli/src/process_hardening.rs
index e570d6b..1ff5afd 100644
--- a/crates/mk-cli/src/process_hardening.rs
+++ b/crates/mk-cli/src/process_hardening.rs
@@ -68,35 +68,6 @@ pub fn set_non_dumpable() {
     }
 }
 
-/// Restore the default SIGPIPE disposition, which Rust's runtime sets to
-/// `SIG_IGN` before `main`.
-///
-/// With SIGPIPE ignored, a `println!` into a closed pipe returns `Err` and the
-/// `print!` machinery PANICS. `mk encode --keys big.txt | head` therefore
-/// printed a Rust panic and exit 101 after emitting a partial bundle -- and a
-/// large batch (a 1000-record file emits ~3000 lines) is exactly what an
-/// operator pages. Restoring `SIG_DFL` makes the process die quietly on EPIPE
-/// the way every other Unix filter does (R2/I, 2026-08-21).
-///
-/// Safety: `signal(2)` with `SIG_DFL` is async-signal-safe and this runs before
-/// any thread is spawned.
-///
-/// A no-op on Windows, which has no SIGPIPE. Gating this INSIDE the function
-/// (as `set_non_dumpable` does) rather than at the call site is what keeps
-/// `main` free of `cfg`; the first version omitted the gate entirely and broke
-/// all three Windows builds, which a Linux-only local gate cannot see.
-pub fn restore_default_sigpipe() {
-    // Unix only: SIGPIPE does not exist on Windows, and Rust does not install
-    // the SIG_IGN disposition there, so there is nothing to restore. Gated the
-    // same way `set_non_dumpable` gates its platform primitives, rather than
-    // gating the CALL SITE, so main() stays platform-agnostic.
-    #[cfg(unix)]
-    unsafe {
-        // SAFETY: single-threaded at this point; SIG_DFL is the kernel default.
-        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
-    }
-}
-
 #[cfg(test)]
 mod tests {
     #[cfg(target_os = "linux")]
```

### 3.7 `design/FOLLOWUPS.md` (the build's FOLLOWUPS edits; not in the grade's diff)

Three entries appended at the end of the file: this fix's own, closed by the
build; the grade's `crash-sweep-mk-address-json-memory` (grade L93-101,
verbatim); and the architect's `mk-codec-doc-comment-exit` (architect Q4 E5,
verbatim). The block is tagged `diff followups` so the gate applies it
separately from the grade's diff. The `mk-codec-doc-comment-exit` entry is
only filed here; the doc-comment edit it describes is not part of this fix.

```diff followups
diff --git i/design/FOLLOWUPS.md w/design/FOLLOWUPS.md
index 651e69c..83ba7af 100644
--- i/design/FOLLOWUPS.md
+++ w/design/FOLLOWUPS.md
@@ -612,3 +612,25 @@ This repo commits a `vendor/` tree consumed by the `--offline --locked` reproduc
 - **Spec:** mnemonic-toolkit `design/SPEC_vendor_freshness_shared.md`.
 - **Status:** resolved -- shipped by this PR
 - Companion: mnemonic-toolkit design/FOLLOWUPS.md::vendor-freshness-shared
+
+### `mk-output-sink` — a stdout that cannot take the output panicked (exit 101) in every mk verb
+
+- **Surfaced:** 2026-11-13, crash sweep; graded in coordinator `agent-reports/grade-mk-crash-sweep-1.md` Q1 (F1, Important) and Q4 (the family rule).
+- **Spec:** `design/FIX_mk_output_sink.md`.
+- **What:** `println!` panicked on a full disk or `> /dev/full`, and a closed pipe killed the process with SIGPIPE (141) on unix and panicked on Windows. Now one write path (`src/out.rs`) stops the output on a write error; a closed reader leaves the verb's own exit code; any other error prints `mk: cannot write the output: <error>` and exits 1 (a failing code is kept).
+- **Status:** resolved -- shipped by this PR
+- **Companions:** the same rule for ms and md, after their triage (coordinator brief); the toolkit manual sentence in `44-mk-cli.md`, toolkit `design/FOLLOWUPS.md::mk-manual-when-stdout-fails` (see the FIX document, section 8 and S1, ruled).
+
+### `crash-sweep-mk-address-json-memory` — `mk address --json` at the 1,000,000 cap peaks at 3.5 GB (`deferred-minor`)
+
+- **Surfaced:** 2026-11-13, crash sweep (architect `agent-reports/architect-crash-sweep-exit-sets.md` Q5); graded in coordinator `agent-reports/grade-mk-crash-sweep-1.md` Q3.
+- **Where:** `crates/mk-cli/src/cmd/address.rs:256-290` (`emit_text`, `emit_json` take every row at once); the cap's doc comment `address.rs:200-202` and `tests/bughunt.rs:85-89` ("never a multi-gigabyte allocation").
+- **What:** every row is built before any is printed; in `--json` each becomes a `serde_json::Value` map, about 21 times the output size. Measured (debug, bip84 card): `--count 1000000 --json` 1,765 MB peak, 81.9 MB stdout; `--range 0,999999 --chain both --json` 3,522 MB peak, 163.8 MB stdout, 62.9 s; the latter under `prlimit --as=2G` aborts after 59.8 s (rc 134, no stdout, stderr only "memory allocation of 632 bytes failed"). Text mode also collects (100 MB at 1,000,000). Fails closed (no output, no wrong data); needs a near-cap `--json` request and a machine with less free memory than about 1.8 GB per million rows.
+- **Owed:** the toolkit's `address` (the migration, verb placement 2026-11-01) writes each row as it is derived, text and JSON alike (the JSON envelope's `addresses` array streamed), with the cap's doc comment true. RED row: `address --range 0,999999 --chain both --json` under `prlimit --as=1G` exits 0 with 2,000,000 rows; it fails on the mk 2fc367c5 code (rc 134, measured at 2G).
+- **Status:** `deferred-minor`. **Owner:** the `address` move to the toolkit. **Tier:** `v1+`.
+
+### `mk-codec-doc-comment-exit` — `CliError::Codec` doc comment says exit 2; the arm returns 1 (architect 2026-11-13)
+
+- Surfaced: coordinator `agent-reports/architect-crash-sweep-exit-sets.md` Q4 E5.
+- What: `crates/mk-cli/src/error.rs` L22 "Exit 2 (or 3 for `UnsupportedVersion`)" -> "Exit 1 (or 3 for `UnsupportedVersion`); `mk repair` exits 2 from its own bypass". `exit_code` L126 is 1 since 0.14.0.
+- Owner: the next mk-cli cycle.
+- Status: `deferred-minor`.
+- Tier: `mk`.
```

## 4. RED rows

The rows are `crates/mk-cli/tests/output_errors.rs` (6 tests, new file),
byte for byte `f1-red-rows.diff` (sha256 `95fa725cd3d8cb6dc4a7e854dd6ea1d854218ed81c4d233ce50f2c604301a00b`). They supersede the
triage's `stdout_write_error.rs` and `closed_pipe.rs`; do not commit those.
Apply the block, then run each command in the `sh` block on the unmodified
code, from the repo root with `$HOME/.cargo/bin` first in PATH (the pin is
1.85.0). The text after `## expect:` is a substring of the failing run's
output.

```diff
diff --git a/crates/mk-cli/tests/output_errors.rs b/crates/mk-cli/tests/output_errors.rs
new file mode 100644
index 0000000..00c903c
--- /dev/null
+++ b/crates/mk-cli/tests/output_errors.rs
@@ -0,0 +1,215 @@
+//! What mk does when stdout cannot take its output (crash sweep F1 and the
+//! family rule, grade-mk-crash-sweep-1; manual `44-mk-cli.md` "Exit codes").
+//!
+//! - A reader that closed the pipe only stops the output: no text, no
+//!   signal, the exit code is the verb's own.
+//! - Any other write error (`/dev/full`, ENOSPC), from a verb or `--help`:
+//!   "mk: cannot write the output: <error>" on stderr, exit 1; a verb that
+//!   was already failing keeps its code.
+//! - No file of the crate but `src/out.rs` names stdout.
+#![cfg(target_os = "linux")]
+
+use std::fs::OpenOptions;
+use std::os::fd::{FromRawFd, OwnedFd};
+use std::os::unix::process::ExitStatusExt;
+use std::process::{Command, Output, Stdio};
+
+const XPUB: &str = "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a";
+const ENCODE: [&str; 9] = [
+    "encode",
+    "--xpub",
+    XPUB,
+    "--origin-path",
+    "m/84'/0'/0'",
+    "--origin-fingerprint",
+    "aabbccdd",
+    "--policy-id-stub",
+    "11223344",
+];
+const FULL: &str = "mk: cannot write the output: No space left on device (os error 28)";
+
+fn mk() -> Command {
+    Command::new(env!("CARGO_BIN_EXE_mk"))
+}
+
+fn card() -> Vec<String> {
+    let o = mk().args(ENCODE).output().unwrap();
+    assert_eq!(o.status.code(), Some(0));
+    String::from_utf8(o.stdout)
+        .unwrap()
+        .lines()
+        .map(str::to_owned)
+        .collect()
+}
+
+/// The card with one character of its first string changed: repair exits 5.
+fn damaged() -> Vec<String> {
+    let mut c = card();
+    let mut b = c[0].clone().into_bytes();
+    b[12] = if b[12] == b'q' { b'p' } else { b'q' };
+    c[0] = String::from_utf8(b).unwrap();
+    c
+}
+
+fn args(verb: &[&str], card: &[String]) -> Vec<String> {
+    verb.iter()
+        .map(|s| s.to_string())
+        .chain(card.iter().cloned())
+        .collect()
+}
+
+fn closed_pipe() -> OwnedFd {
+    let mut fds = [0i32; 2];
+    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
+    unsafe { libc::close(fds[0]) }; // the reader is gone before mk starts
+    unsafe { OwnedFd::from_raw_fd(fds[1]) }
+}
+
+fn on_full(a: &[String]) -> Output {
+    let full = OpenOptions::new().write(true).open("/dev/full").unwrap();
+    mk().args(a)
+        .stdout(Stdio::from(full))
+        .stderr(Stdio::piped())
+        .output()
+        .unwrap()
+}
+
+fn on_closed(a: &[String], stderr_too: bool) -> Output {
+    let err = if stderr_too {
+        Stdio::from(closed_pipe())
+    } else {
+        Stdio::piped()
+    };
+    mk().args(a)
+        .stdout(Stdio::from(closed_pipe()))
+        .stderr(err)
+        .output()
+        .unwrap()
+}
+
+fn full_case(a: &[String], code: i32) {
+    let o = on_full(a);
+    let err = String::from_utf8_lossy(&o.stderr);
+    assert!(!err.contains("panicked"), "{a:?}: panic: {err}");
+    assert_eq!(
+        o.status.code(),
+        Some(code),
+        "{a:?}: status {:?}: {err}",
+        o.status
+    );
+    assert_eq!(err.lines().last(), Some(FULL), "{a:?}: stderr: {err}");
+}
+
+fn closed_case(a: &[String], code: i32, stderr_too: bool) {
+    let o = on_closed(a, stderr_too);
+    let err = String::from_utf8_lossy(&o.stderr);
+    assert_eq!(
+        o.status.signal(),
+        None,
+        "{a:?}: killed by a signal: {:?}",
+        o.status
+    );
+    assert_eq!(
+        o.status.code(),
+        Some(code),
+        "{a:?}: status {:?}: {err}",
+        o.status
+    );
+    assert!(
+        !err.contains("panicked") && !err.contains("cannot write"),
+        "{a:?}: {err}"
+    );
+}
+
+fn s(v: &[&str]) -> Vec<String> {
+    v.iter().map(|x| x.to_string()).collect()
+}
+
+#[test]
+fn full_stdout_exits_1_with_the_line() {
+    let c = card();
+    for a in [
+        s(&["gui-schema"]),
+        s(&["vectors"]),
+        s(&["vectors", "--pretty"]),
+        s(&ENCODE),
+        args(&["decode"], &c),
+        args(&["decode", "--json"], &c),
+        args(&["inspect"], &c),
+        args(&["verify"], &c),
+        args(&["repair"], &c),
+        args(&["address", "--count", "2"], &c),
+        args(&["derive", "--path", "m/0/5"], &c),
+        s(&["--help"]),
+        s(&["--version"]),
+        s(&["decode", "--help"]),
+    ] {
+        full_case(&a, 1);
+    }
+}
+
+#[test]
+fn full_stdout_after_a_repair_exits_1_not_5() {
+    let d = damaged();
+    let o = mk().args(args(&["repair"], &d)).output().unwrap();
+    assert_eq!(o.status.code(), Some(5), "fixture: repair should exit 5");
+    full_case(&args(&["repair"], &d), 1);
+}
+
+#[test]
+fn full_stdout_keeps_a_failing_code() {
+    // A usage error in --json mode prints its envelope on stdout (exit 64).
+    full_case(
+        &args(&["address", "--json", "--count", "1000001"], &card()),
+        64,
+    );
+}
+
+#[test]
+fn closed_reader_exits_with_the_verbs_code() {
+    let c = card();
+    for a in [
+        s(&["gui-schema"]),
+        s(&["vectors"]),
+        s(&ENCODE),
+        args(&["address", "--count", "2"], &c),
+        s(&["--help"]),
+        s(&["--version"]),
+    ] {
+        closed_case(&a, 0, false);
+    }
+    closed_case(&args(&["repair"], &damaged()), 5, false);
+}
+
+#[test]
+fn closed_reader_on_stdout_and_stderr_exits_0() {
+    // encode prints its stderr notes after the card; `vectors --out` prints
+    // "wrote N vector file(s)" with `eprintln!`.
+    closed_case(&s(&ENCODE), 0, true);
+    let dir = tempfile::tempdir().unwrap();
+    let d = dir.path().to_str().unwrap();
+    closed_case(&s(&["vectors", "--out", d]), 0, true);
+}
+
+#[test]
+fn only_out_rs_names_stdout() {
+    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
+    let mut bad = Vec::new();
+    let mut stack = vec![dir];
+    while let Some(d) = stack.pop() {
+        for e in std::fs::read_dir(&d).unwrap() {
+            let p = e.unwrap().path();
+            if p.is_dir() {
+                stack.push(p);
+            } else if p.extension().is_some_and(|x| x == "rs") && !p.ends_with("src/out.rs") {
+                let t = std::fs::read_to_string(&p).unwrap();
+                for n in ["stdout(", "std::print", "std::eprint", "core::print"] {
+                    if t.contains(n) {
+                        bad.push(format!("{}: {n}", p.display()));
+                    }
+                }
+            }
+        }
+    }
+    assert!(bad.is_empty(), "stdout named outside src/out.rs: {bad:?}");
+}
```

```sh
cargo nextest run --locked -j 8 -E 'test(/(^|::)full_stdout_exits_1_with_the_line$/)' ## expect: panicked at library/std/src/io/stdio.rs
cargo nextest run --locked -j 8 -E 'test(/(^|::)full_stdout_after_a_repair_exits_1_not_5$/)' ## expect: panicked at library/std/src/io/stdio.rs
cargo nextest run --locked -j 8 -E 'test(/(^|::)full_stdout_keeps_a_failing_code$/)' ## expect: panicked at library/std/src/io/stdio.rs
cargo nextest run --locked -j 8 -E 'test(/(^|::)closed_reader_exits_with_the_verbs_code$/)' ## expect: killed by a signal: ExitStatus(unix_wait_status(13))
cargo nextest run --locked -j 8 -E 'test(/(^|::)closed_reader_on_stdout_and_stderr_exits_0$/)' ## expect: killed by a signal: ExitStatus(unix_wait_status(13))
```

| row | expected after GREEN | on the base code (measured by the grade, `red3.log`) |
|---|---|---|
| `full_stdout_exits_1_with_the_line` | 14 invocations (nine verbs, `--help`, `--version`, `decode --help`) each exit 1 with the line last on stderr, no panic | FAIL: `thread 'main' panicked at library/std/src/io/stdio.rs:1123:9` / `failed printing to stdout: No space left on device (os error 28)` |
| `full_stdout_after_a_repair_exits_1_not_5` | the repair fixture exits 5 on a good stdout; on a full stdout exit 1 | FAIL: the same panic |
| `full_stdout_keeps_a_failing_code` | exit 64 and the line | FAIL: the same panic |
| `closed_reader_exits_with_the_verbs_code` | exit 0 (5 for `repair`), no signal | FAIL: `killed by a signal: ExitStatus(unix_wait_status(13))` |
| `closed_reader_on_stdout_and_stderr_exits_0` | exit 0, no signal | FAIL: signal 13 |
| `only_out_rs_names_stdout` | PASS | PASS on the base too: a source-scan guard, not a RED row. Its teeth are plants P7 and P8 (below). |

Whole-file RED (grade): **6 tests run: 1 passed, 5 failed**. The rows are
`cfg(target_os = "linux")`.

### Tests can fail (the grade's nine plants)

Each plant was made against the fix; each fails at least one row (grade
`plants.txt`; the gate does not re-run them).

| plant | what is planted | fails |
|---|---|---|
| P1 | BrokenPipe is recorded as a write error | `closed_reader_exits_with_the_verbs_code`, `closed_reader_on_stdout_and_stderr_exits_0` |
| P2 | no 0 to 1 mapping | `full_stdout_after_a_repair_exits_1_not_5`, `full_stdout_exits_1_with_the_line` |
| P3 | `repair`'s 5 is kept | `full_stdout_after_a_repair_exits_1_not_5` |
| P4 | `--help`/`--version` not routed through the sink | `full_stdout_exits_1_with_the_line` |
| P5 | a std `eprintln!` | `closed_reader_on_stdout_and_stderr_exits_0` |
| P6 | every code becomes 1 | `full_stdout_keeps_a_failing_code` |
| P7 | `std::io::stdout()` in `vectors.rs` | `only_out_rs_names_stdout` |
| P8 | one `std::println!` site in `address.rs` | `closed_reader_exits_with_the_verbs_code`, `full_stdout_exits_1_with_the_line`, `only_out_rs_names_stdout` |
| P9 | the line is not printed | `full_stdout_after_a_repair_exits_1_not_5`, `full_stdout_exits_1_with_the_line`, `full_stdout_keeps_a_failing_code` |

## 5. Tests that change

None changes. One file is added: `crates/mk-cli/tests/output_errors.rs`
(section 4). The triage's `stdout_write_error.rs` and `closed_pipe.rs` are
not committed. `restore_default_sigpipe` has no test of its own (the grade's
diff removes only the function).

## 6. Build order

1. Add the rows (section 4) as their own commit; run the RED commands: five FAIL for the stated reasons, the scan row PASSES.
2. Apply sections 3.1 to 3.6 (the grade's diff). Run the six rows: GREEN.
3. Apply section 3.7 (FOLLOWUPS).
4. Gate (section 7).
5. Toolkit manual sentence (section 8): not part of this PR; it goes in the toolkit PR that pins the first mk-cli release carrying this fix (S1, ruled).
6. CI on the PR must show the legs the grade did not run (section 9).

## 7. Gate

`scripts/mk-output-sink-fix-gate.sh` runs the whole of it: it extracts this
document's blocks, runs every RED row as printed on the base, applies the code
blocks, shows the rows and the full mk-cli suite green, and has a self-test
that goes red on a broken row. Its header states what it does not cover.
The commands it runs, each to a file, then grep:

- `cargo nextest run --locked -j 8` expecting **405 tests run: 405 passed**
  (grade, `green-final.log`: base 399 plus the 6 rows).
- `cargo test --locked --doc`
- `cargo fmt --check`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo nextest run --locked -j 8 -E 'binary(output_errors)'`: **6 tests run: 6 passed** (a short or empty count is a FAIL).
- Then `coordinator/scripts/mutation-gate.sh <worktree> <base>` before any review (not in the script).

## 8. Toolkit manual lockstep (44-mk-cli.md)

Insert in toolkit `docs/manual/src/40-cli-reference/44-mk-cli.md` after
L609 (the `64` row, the last row of the "Exit codes" table), after one blank
line, verbatim from the grade:

> A reader that closes the pipe early (`mk address … | head -3`) only stops the output: the exit code is the verb's own. Any other write error on stdout (a full disk, `> /dev/full`), from a verb or `--help`, prints `mk: cannot write the output: <error>` on stderr and exits 1; a verb that was already failing keeps its own code.

(L609 checked on toolkit master 47baf6f2: it is the `| 64 | CLI usage error ...`
row.) The sentence does not depend on the release F1 re-pin of that table's
rows (grade Q5; architect Q3), which may land first. The toolkit installer
pins mk 0.13.0 today and release item F1 re-pins it to 0.14.0
(`agent-reports/architect-release-f1.md` L105-116, the lockstep manual rows);
toolkit FOLLOWUPS entry `mk-manual-exit-table-0-14` is being filed. It lands with the toolkit pin of the first mk-cli release carrying this fix (S1, ruled).

Toolkit companion entry, as the text a toolkit FOLLOWUPS PR files (the manual
sentence above is held verbatim):

> ### `mk-manual-when-stdout-fails` — at the mk re-pin carrying the output sink, `44-mk-cli.md` states what mk does when stdout fails (repo: **mnemonic-toolkit**; owning phase: **the toolkit pin bump to the mk-cli release carrying mnemonic-key `mk-output-sink`**)
> - Surfaced: coordinator agent-reports/grade-ms-md-output-errors.md section 4 (S1). Companion: mnemonic-key design/FOLLOWUPS.md::mk-output-sink.
> - What: insert in docs/manual/src/40-cli-reference/44-mk-cli.md, after the last row of `## Exit codes` (today L609), the paragraph of section 8 of mnemonic-key `design/FIX_mk_output_sink.md` verbatim (the sentence above), in the same commit as the scripts/install.sh mk pin. Plain text, no (unreleased: …) marker.
> - Status: `open`.
> - Tier: `docs`, `cross-repo`.

No mk flag or help text changes, so the mnemonic-gui mk schema mirror needs no
change. The stderr line and the exit codes are mk's only printed-text change.

## 9. Checks recorded

- **Version:** `Cargo.toml` and `Cargo.lock` 0.14.0 to 0.15.0-dev (in the diff).
- **Evidence (grade):** RED 6 run, 1 passed, 5 failed (`red3.log`); GREEN workspace 405 run, 405 passed (`green-final.log`); clippy `--workspace --all-targets -D warnings` rc 0; `cargo +1.95.0 fmt --check` rc 0; `cargo test --doc` rc 0. On the fixed binary: `mk --help | head -5` gave 0 nonzero in 200 runs; `address --count 100000 | head -3` rc 0; `ulimit -f 1` left 512 bytes, rc 1, `mk: cannot write the output: File too large (os error 27)`.
- **Not run by the grade, so CI on the PR must show them** (the rows are Linux-only; `out.rs` is std-only and std maps Windows' closed-pipe errors to BrokenPipe):
  1. **Windows**: the closed-pipe case must give the verb's own exit code and no panic. This leg must confirm std's mapping; there is no Windows row, so read it first.
  2. macOS.
  3. musl.
  4. release-mode build and tests.
  5. FreeBSD.
- **Gate coverage:** see the script header; it does not re-run the nine plants, the CI legs above, the toolkit manual, or `mutation-gate.sh`.

## 10. STOP (none open)

- **S1 (ruled 2026-11-13, coordinator `agent-reports/grade-ms-md-output-errors.md` section 4).** The section 8 sentence goes in the toolkit PR that moves `scripts/install.sh`'s mk pin (`mk-cli-v0.13.0`, L61) to the first mk-cli release carrying this fix (0.15.0 or later). It goes in the same commit as that pin and the `44-mk-cli.md` L12 install line, and it may not merge before that pin. Insert it after the last row of the `## Exit codes` table (L601; today L609). That is not the release-F1 PR, which pins 0.14.0, unless F1 is built after mk 0.15.0 ships and pins it. Plain text, no `(unreleased: …)` marker: the lint reads that marker only on flag rows, so a behaviour marker could never fail. This PR files the cross-repo pair: mnemonic-key `design/FOLLOWUPS.md::mk-output-sink` (a Companion line) and toolkit `design/FOLLOWUPS.md::mk-manual-when-stdout-fails` (form below). The same rule holds for ms (`43-ms.md`) and md (`42-md.md`).
