# FIX-mk-single-print-2026-11: `mk encode` prints each mk1 string once

Model: Claude Opus 5.5 (claude-opus-5-5), 2026-11-22. Author of this FIX document (not the coordinator).
Base: `origin/main` 434d789 (mk-cli 0.16.0). Branch `fix/single-print`.
Folded for review r1 (`coordinator/agent-reports/review-fix-single-print-r1.md`; fold model Claude Sonnet 5.5,
claude-sonnet-5-5, 2026-11-22): I-3 (section 8a: this doc owns the one-place toolkit gui-manual text, no FIX owns
mk's alone), I-4 (section 8: landing order; section 8a-bis: the `{#installer-ahead}` replacement text), M-3 (the
six `--in` help lines, section 3). `origin/main` has since moved to 3755d47 (a FOLLOWUPS line only; the section 3/5
diffs and `single_print.rs` still apply clean and pass).

## 0. Ruling and scope

Brian 2026-11-22 (typed "B"): reverse D4 of mnemonic-engrave `design/SPEC_constellation_cli_uniformity.md`. `ms`,
`md` and `mk` print their string ONCE, on stdout; the grouped stderr engraving card is dropped everywhere. `mt
encode` and `mnemonic bundle` are the model. The uniformity-spec delta is on mnemonic-engrave branch
`fix/single-print` (commit 4f35eb9f: D4 row, "D4 reversed" paragraph, §6a, §6b, §6c, §9a).

In scope here: `mk encode` (the only mk verb that prints a card). Out of scope: every other mk verb (none prints a
copy of its output on stderr; `mk repair` prints its strings once, on stdout, measured 2026-11-22).

### PROPOSED decisions (architect confirms)

- P1. `--group-size` and `--separator` are KEPT and group STDOUT on request, as `mt encode` and `mnemonic bundle`
  do. `--group-size` default changes 5 -> 0 (unbroken). `--separator` keeps its parser (`space` or `" "`; `hyphen`,
  `comma`, `-`, `,` refused at exit 64 with the unchanged message) and default `space`.
  Why not removal: dozens of consumer invocations pass `--group-size 0` (mnemonic-engrave journeys, seedhammer
  oracle, toolkit transcripts); removal breaks each one for no gain, and the toolkit and mt already carry the pair.
- P2. `--out FILE` receives exactly the bytes stdout would have carried (grouped under `--group-size`), as `mt
  encode --out` does; stdout stays empty under `--out` (unchanged).
- P3. `--json` ignores `--group-size` / `--separator` (unchanged).
- P4. `mk encode --keys` with two or more cards: the card boundary (F-311: a duplicated key record mints two
  byte-identical cards sharing one chunk-set id, and the stderr blank line between cards was the only signal) moves
  to one stderr line per card, `card <i> of <n>: lines <a>-<b>`, where `<a>-<b>` are 1-based line numbers of stdout
  (or of the `--out` file). Printed after the artifact, before the watch-only note; not printed for one card; not
  printed under `--json`. Every mk1 card is at least two strings (an xpub-bearing payload exceeds one codex32
  string), so the range form `<a>-<b>` with `a < b` is the only form.
- P5. The watch-only note under `--out` (stdout empty, note still says "stdout is watch-only") is NOT changed here:
  this edit does not touch `output_advisory.rs`. Proposed FOLLOWUPS line in section 9.

## 1. Spec delta (mk-local texts)

The normative text is the uniformity spec (above). mk-local, user-visible:

`mk encode --help`, measured on the GREEN build:

    --group-size <GROUP_SIZE>
        Group each mk1 string on stdout every N characters, for hand engraving (0 = unbroken, the default). `--out` receives the same bytes; `--json` stays unbroken

        [default: 0]

    --separator <SEPARATOR>
        Separator to use with `--group-size`: `space`, or the literal " ". Whitespace only; `hyphen` and `comma` were removed

        [default: space]

`crates/mk-cli/CHANGELOG.md`, under `## [Unreleased]`:

    ### Changed

    - **Each mk1 string is printed once.** `mk encode` no longer repeats its strings on stderr as a grouped
      engraving card, and the `group size:` / `separator:` lines are gone (Brian 2026-11-22, reversing D4 of the
      constellation uniformity spec). `--group-size N` now groups stdout, on request; its default is `0`, so a plain
      `mk encode` prints on stdout exactly what it printed before. `--out FILE` receives the same bytes stdout
      would have carried. With `--keys` and two or more cards, stderr names each card's lines:
      `card <i> of <n>: lines <a>-<b>`. `--json` is unchanged.

## 2. Behaviour: exact bytes per case

Fixture: `T` = `tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X`
(mk-cli `V15_84_TEST`, testnet, m/84'/1'/0', fp 73c5da0a). `A` = `encode --xpub T --origin-fingerprint 73c5da0a
--origin-path "m/84'/1'/0'" --policy-id-stub deadbeef`. `N` = `note: stdout is watch-only — public keys only,
cannot spend` (U+2014). `S1`/`S2` = the two strings below. Measured on the GREEN build; every exit is 0.

    S1 = mk1qpr4pypqqsqaatd7aaeutks2zvzrtp70zqsrqra95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd955pp5xtght5le5r5mgsp
    S2 = mk1qpr4pypp0ftfaywm7czcz5yu0ly5d5gq8dsv0hhg22v48rdkx56n34v4wsw5yjwwwverz9nwfh7gc
    G1 = mk1qp r4pyp qqsqa atd7a aeutk s2zvz rtp70 zqsrq ra95k j6tfd 95kj6 tfd95 kj6tf d95kj 6tfd9 5kj6t fd95k j6tfd 955pp 5xtgh t5le5 r5mgs p
    G2 = mk1qp r4pyp p0ftf aywm7 czcz5 yu0ly 5d5gq 8dsv0 hhg22 v48rd kx56n 34v4w sw5yj wwwve rz9nw fh7gc

| case | stdout | stderr | before (434d789) |
|---|---|---|---|
| `mk A` | `S1\nS2\n` | `N\n` | stderr `G1\nG2\ngroup size: 5\nseparator: space\nN\n` |
| `mk A --group-size 5` (also `--separator space` / `" "`) | `G1\nG2\n` | `N\n` | stdout `S1\nS2\n`; stderr as above |
| `mk A --group-size 0` | `S1\nS2\n` | `N\n` | stderr `S1\nS2\ngroup size: 0\nseparator: space\nN\n` |
| `mk A --group-size 5 --out F` | empty; F (0600) = `G1\nG2\n` | `N\n` | F = `S1\nS2\n`; stderr card as row 1 |
| `mk A --group-size 5 --json` | the unchanged JSON line (`mk1_strings` = [S1, S2], unbroken) | `N\n` | identical (unchanged) |
| `mk encode --keys K --policy-id-stub deadbeef`, K = the record `[73c5da0a/84'/1'/0']T` twice | `S1\nS2\nS1\nS2\n` | `card 1 of 2: lines 1-2\ncard 2 of 2: lines 3-4\nN\n` | stderr `G1\nG2\n\nG1\nG2\ngroup size: 5\nseparator: space\nN\n` |
| `--keys` with one record | the card's lines | `N\n` (no label) | card + 2 lines + N |
| `--keys K --json` | unchanged JSON | `N\n` (no labels) | unchanged |
| refusals, exit codes | unchanged | unchanged | — |

Multi-chunk: every mk1 card in this table is already two chunks (`S1`, `S2`); grouping applies per line.

## 3. Production changes (exact; the GREEN build measured in section 6)

Two files. `format.rs` loses `separator_name` and `write_engraving_card` (no other caller); `encode.rs` changes the
two flag docs and default, renders the artifact grouped, and replaces the card with the `--keys` labels.

```diff
diff --git i/crates/mk-cli/src/cmd/encode.rs w/crates/mk-cli/src/cmd/encode.rs
index 75eb865..278efab 100644
--- i/crates/mk-cli/src/cmd/encode.rs
+++ w/crates/mk-cli/src/cmd/encode.rs
@@ -103,13 +103,14 @@ pub struct EncodeArgs {
     #[arg(long)]
     pub chunk_set_id: Option<String>,
 
-    /// Insert a separator every N characters in each emitted mk1 string
-    /// (0 = unbroken). SPEC §3. Display only; --json stays unbroken.
-    #[arg(long, default_value_t = 5)]
+    /// Group each mk1 string on stdout every N characters, for hand engraving
+    /// (0 = unbroken, the default). `--out` receives the same bytes; `--json`
+    /// stays unbroken.
+    #[arg(long, default_value_t = 0)]
     pub group_size: u16,
 
-    /// Separator for the stderr engraving card: `space`, or the literal " ".
-    /// Whitespace only (SPEC §6c); `hyphen` and `comma` were removed.
+    /// Separator to use with `--group-size`: `space`, or the literal " ".
+    /// Whitespace only; `hyphen` and `comma` were removed.
     #[arg(long, default_value = "space", value_parser = crate::format::parse_separator)]
     pub separator: char,
 
@@ -402,16 +403,11 @@ pub fn run(args: EncodeArgs) -> Result<u8> {
         });
     }
 
-    // SPEC §6a: the artifact is UNGROUPED and stdout carries nothing else --
-    // so `me sysw pack` can read what `mk encode` wrote with no
-    // `--group-size 0` and no `grep` in between. No blank line between cards
-    // either: a blank line is not an artifact. The grouped form a human
-    // transcribes, and the card boundary, both move to the stderr engraving
-    // card below (§6b: the grouping flags "affect the stderr card only").
-    //
-    // Built as ONE string rather than printed line by line, because §6b's
-    // `--out` writes the same bytes to a file and a second emitter is a second
-    // place for the two to drift.
+    // Each mk1 string is printed ONCE, on stdout (Brian 2026-11-22, reversing
+    // D4): there is no stderr engraving card. `--group-size` groups stdout on
+    // request, as `mt encode` and `mnemonic bundle` do; the default (0) is the
+    // canonical, unbroken artifact `me sysw pack` reads. `--out` receives the
+    // same bytes stdout would have carried, built once so the two cannot drift.
     let artifact = if args.json {
         if key_source.is_some() {
             json_batch(&minted)?
@@ -422,7 +418,11 @@ pub fn run(args: EncodeArgs) -> Result<u8> {
         let mut body = String::new();
         for card in &minted {
             for s in &card.strings {
-                body.push_str(s);
+                body.push_str(&crate::format::render_grouped(
+                    s,
+                    args.group_size as usize,
+                    args.separator,
+                ));
                 body.push('\n');
             }
         }
@@ -430,29 +430,26 @@ pub fn run(args: EncodeArgs) -> Result<u8> {
     };
 
     // SPEC §6b: "stdout is used when --out is not given." When it IS given the
-    // artifact goes to the file ONLY -- writing both would put on stdout the
-    // material `--out` exists to keep off it.
+    // artifact goes to the file ONLY.
     match &args.out_file {
         Some(path) => {
-            // The one item P3's boundary table has `mk` ADOPT from the shared
-            // crate. `write` has no root re-export, so the path is qualified --
-            // the unqualified form is an E0425.
             mnemonic_io_lib::write::write_private(std::path::Path::new(path), artifact.as_bytes())
                 .map_err(|e| CliError::UsageError(format!("--out {path}: {e}")))?
         }
         None => print!("{artifact}"),
     }
 
-    // The card is a display of the artifact, so `--json` (which is explicitly
-    // out of scope this cycle) does not get one.
-    if !args.json {
-        let grouped: Vec<Vec<String>> = minted.iter().map(|c| c.strings.clone()).collect();
-        crate::format::write_engraving_card(
-            &mut std::io::stderr(),
-            &grouped,
-            args.group_size as usize,
-            args.separator,
-        );
+    // F-311: a key file carrying the same record twice mints two byte-identical
+    // cards that share one chunk-set id, so the card boundary is not
+    // recoverable from the strings. With two or more cards, stderr names each
+    // card's line range. `--json` carries the cards as separate objects.
+    if !args.json && minted.len() > 1 {
+        let mut first = 1usize;
+        for (i, card) in minted.iter().enumerate() {
+            let last = first + card.strings.len() - 1;
+            eprintln!("card {} of {}: lines {first}-{last}", i + 1, minted.len());
+            first = last + 1;
+        }
     }
     crate::output_advisory::emit_output_class_advisory(
         crate::output_advisory::OutputClass::WatchOnly,
diff --git i/crates/mk-cli/src/format.rs w/crates/mk-cli/src/format.rs
index 83effff..2da5ece 100644
--- i/crates/mk-cli/src/format.rs
+++ w/crates/mk-cli/src/format.rs
@@ -34,59 +34,6 @@ pub fn strip_display_separators(s: &str) -> String {
     s.chars().filter(|&c| !is_display_separator(c)).collect()
 }
 
-/// Human-readable name for a display separator, for the engraving card's
-/// `separator:` line. A card that reported `separator: ' '` would be telling an
-/// operator to type a quote mark.
-pub fn separator_name(c: char) -> &'static str {
-    match c {
-        ' ' => "space",
-        '-' => "hyphen",
-        ',' => "comma",
-        '\t' => "tab",
-        _ => "whitespace",
-    }
-}
-
-/// Write the stderr **engraving card** (SPEC §6c).
-///
-/// §6a rules that `encode`'s stdout is the canonical artifact ungrouped and
-/// nothing else, which evicts the grouped form; §6c requires `md` and `mk` --
-/// which had no card at all, only a one-line `note:` -- to grow one, *"since
-/// after D4 that is the only place it exists"*. stdout is what a pipeline reads;
-/// this is what a human transcribes onto a plate.
-///
-/// Shape follows **`ms`'s** card, measured: plain `label: value` lines on stderr
-/// with no prefix character, grouped string first because that is the thing being
-/// transcribed. `mnemonic bundle`'s `#`-prefixed card is the other
-/// in-constellation precedent and is deliberately NOT followed -- its `#` mirrors
-/// the comment headers on its own stdout, a surface `mk` does not have.
-///
-/// The caller emits the existing output-class advisory AFTER this, so that
-/// advisory stays the last stderr line it has always been.
-pub fn write_engraving_card<W: std::io::Write>(
-    w: &mut W,
-    cards: &[Vec<String>],
-    group_size: usize,
-    separator: char,
-) {
-    for (i, strings) in cards.iter().enumerate() {
-        // A blank line BETWEEN cards, never before the first or after the last.
-        // It used to sit on stdout, where §6a no longer admits it -- and it was
-        // the only signal that `mk encode --keys` had silently accepted the same
-        // BIP-380 record twice (F-311: duplicates mint two byte-identical cards
-        // that share one chunk-set id, so the boundary is not recoverable from
-        // the headers). The human-facing card is where that signal belongs.
-        if i > 0 {
-            let _ = writeln!(w);
-        }
-        for s in strings {
-            let _ = writeln!(w, "{}", render_grouped(s, group_size, separator));
-        }
-    }
-    let _ = writeln!(w, "group size: {group_size}");
-    let _ = writeln!(w, "separator: {}", separator_name(separator));
-}
-
 /// Parse `--separator`: the keyword `space`, or the literal `" "`.
 ///
 /// **`hyphen` and `comma` were removed (SPEC §6c: whitespace only, everywhere).**
diff --git i/crates/mk-cli/src/cmd/address.rs w/crates/mk-cli/src/cmd/address.rs
index 1df0b2e..0b15c94 100644
--- i/crates/mk-cli/src/cmd/address.rs
+++ w/crates/mk-cli/src/cmd/address.rs
@@ -25,7 +25,7 @@ pub struct AddressArgs {
 
     /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
     /// the positional arguments. Display separators are stripped, so a card
-    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
+    /// transcribed in grouped form re-ingests. SPEC §6b.
     #[arg(long = "in", value_name = "FILE")]
     pub in_file: Option<String>,
 
diff --git i/crates/mk-cli/src/cmd/decode.rs w/crates/mk-cli/src/cmd/decode.rs
index 75b68aa..0cc2f74 100644
--- i/crates/mk-cli/src/cmd/decode.rs
+++ w/crates/mk-cli/src/cmd/decode.rs
@@ -25,7 +25,7 @@ pub struct DecodeArgs {
 
     /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
     /// the positional arguments. Display separators are stripped, so a card
-    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
+    /// transcribed in grouped form re-ingests. SPEC §6b.
     #[arg(long = "in", value_name = "FILE")]
     pub in_file: Option<String>,
 
diff --git i/crates/mk-cli/src/cmd/derive.rs w/crates/mk-cli/src/cmd/derive.rs
index c5f3b54..9aef518 100644
--- i/crates/mk-cli/src/cmd/derive.rs
+++ w/crates/mk-cli/src/cmd/derive.rs
@@ -28,7 +28,7 @@ pub struct DeriveArgs {
 
     /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
     /// the positional arguments. Display separators are stripped, so a card
-    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
+    /// transcribed in grouped form re-ingests. SPEC §6b.
     #[arg(long = "in", value_name = "FILE")]
     pub in_file: Option<String>,
 
diff --git i/crates/mk-cli/src/cmd/inspect.rs w/crates/mk-cli/src/cmd/inspect.rs
index c1fc3bf..4737513 100644
--- i/crates/mk-cli/src/cmd/inspect.rs
+++ w/crates/mk-cli/src/cmd/inspect.rs
@@ -32,7 +32,7 @@ pub struct InspectArgs {
 
     /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
     /// the positional arguments. Display separators are stripped, so a card
-    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
+    /// transcribed in grouped form re-ingests. SPEC §6b.
     #[arg(long = "in", value_name = "FILE")]
     pub in_file: Option<String>,
 
diff --git i/crates/mk-cli/src/cmd/repair.rs w/crates/mk-cli/src/cmd/repair.rs
index 87a3b60..815c53b 100644
--- i/crates/mk-cli/src/cmd/repair.rs
+++ w/crates/mk-cli/src/cmd/repair.rs
@@ -76,7 +76,7 @@ pub struct RepairArgs {
 
     /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
     /// the positional arguments. Display separators are stripped, so a card
-    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
+    /// transcribed in grouped form re-ingests. SPEC §6b.
     #[arg(long = "in", value_name = "FILE")]
     pub in_file: Option<String>,
 
diff --git i/crates/mk-cli/src/cmd/verify.rs w/crates/mk-cli/src/cmd/verify.rs
index 87b1d61..274fc70 100644
--- i/crates/mk-cli/src/cmd/verify.rs
+++ w/crates/mk-cli/src/cmd/verify.rs
@@ -28,7 +28,7 @@ pub struct VerifyArgs {
 
     /// Read mk1 strings from FILE, one per line, instead of (or in addition to)
     /// the positional arguments. Display separators are stripped, so a card
-    /// transcribed from the engraving card in grouped form re-ingests. SPEC §6b.
+    /// transcribed in grouped form re-ingests. SPEC §6b.
     #[arg(long = "in", value_name = "FILE")]
     pub in_file: Option<String>,
 
```

Review r1 M-3 is the last six hunks above: the `--in` help of `mk address`, `decode`, `derive`, `inspect`, `repair`
and `verify` said "transcribed from the engraving card in grouped form re-ingests"; there is no stderr card now.

## 4. RED rows

New file `crates/mk-cli/tests/single_print.rs` (verbatim below; rustfmt-clean). Row = test function name.

| row | expected after GREEN | measured on base 434d789 |
|---|---|---|
| `encode_prints_each_string_once` | stdout two unbroken mk1 lines; stderr exactly `N\n` | FAIL: stderr `G1\nG2\ngroup size: 5\nseparator: space\nN\n` (the second copy) |
| `encode_group_size_groups_stdout` | `--group-size 5` stdout = `G1\nG2\n`; stderr `N\n` | FAIL: stdout `S1\nS2\n` (the flag does not reach stdout) |
| `encode_out_writes_the_stdout_bytes` | `--group-size 5 --out F`: stdout empty, F = `G1\nG2\n`, stderr `N\n` | FAIL: stderr carries `G1\nG2\ngroup size: 5\nseparator: space\n` before `N` |
| `encode_keys_labels_each_card_on_stderr` | the duplicate record twice: stdout 4 lines; stderr `card 1 of 2: lines 1-2\ncard 2 of 2: lines 3-4\nN\n` | FAIL: stderr is both grouped cards with a blank line between, then the two card lines |

Command (run, not pinned): `CARGO_TARGET_DIR=<own> cargo nextest run --locked --build-jobs 6 -j 6 -p mk-cli --test
single_print`. Base result: `4 tests run: 0 passed, 4 failed`. The assertion excerpts from that run:

```text
    thread 'encode_keys_labels_each_card_on_stderr' panicked at crates/mk-cli/tests/single_print.rs:105:5:
      left: "mk1qp r4pyp qqsqa atd7a aeutk s2zvz rtp70 zqsrq ra95k j6tfd 95kj6 tfd95 kj6tf d95kj 6tfd9 5kj6t fd95k j6tfd 955pp 5xtgh t5le5 r5mgs p\nmk1qp r4pyp p0ftf aywm7 czcz5 yu0ly 5d5gq 8dsv0 hhg22 v48rd kx56n 34v4w sw5yj wwwve rz9nw fh7gc\n\nmk1qp r4pyp qqsqa atd7a aeutk s2zvz rtp70 zqsrq ra95k j6tfd 95kj6 tfd95 kj6tf d95kj 6tfd9 5kj6t fd95k j6tfd 955pp 5xtgh t5le5 r5mgs p\nmk1qp r4pyp p0ftf 
     right: "card 1 of 2: lines 1-2\ncard 2 of 2: lines 3-4\nnote: stdout is watch-only — public keys only, cannot spend\n"
    thread 'encode_prints_each_string_once' panicked at crates/mk-cli/tests/single_print.rs:65:5:
      left: "mk1qp r4pyp qqsqa atd7a aeutk s2zvz rtp70 zqsrq ra95k j6tfd 95kj6 tfd95 kj6tf d95kj 6tfd9 5kj6t fd95k j6tfd 955pp 5xtgh t5le5 r5mgs p\nmk1qp r4pyp p0ftf aywm7 czcz5 yu0ly 5d5gq 8dsv0 hhg22 v48rd kx56n 34v4w sw5yj wwwve rz9nw fh7gc\ngroup size: 5\nseparator: space\nnote: stdout is watch-only — public keys only, cannot spend\n"
     right: "note: stdout is watch-only — public keys only, cannot spend\n"
    thread 'encode_group_size_groups_stdout' panicked at crates/mk-cli/tests/single_print.rs:74:5:
      left: "mk1qpr4pypqqsqaatd7aaeutks2zvzrtp70zqsrqra95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd95kj6tfd955pp5xtght5le5r5mgsp\nmk1qpr4pypp0ftfaywm7czcz5yu0ly5d5gq8dsv0hhg22v48rdkx56n34v4wsw5yjwwwverz9nwfh7gc\n"
     right: "mk1qp r4pyp qqsqa atd7a aeutk s2zvz rtp70 zqsrq ra95k j6tfd 95kj6 tfd95 kj6tf d95kj 6tfd9 5kj6t fd95k j6tfd 955pp 5xtgh t5le5 r5mgs p\nmk1qp r4pyp p0ftf aywm7 czcz5 yu0ly 5d5gq 8dsv0 hhg22 v48rd kx56n 34v4w sw5yj wwwve rz9nw fh7gc\n"
    thread 'encode_out_writes_the_stdout_bytes' panicked at crates/mk-cli/tests/single_print.rs:87:5:
      left: "mk1qp r4pyp qqsqa atd7a aeutk s2zvz rtp70 zqsrq ra95k j6tfd 95kj6 tfd95 kj6tf d95kj 6tfd9 5kj6t fd95k j6tfd 955pp 5xtgh t5le5 r5mgs p\nmk1qp r4pyp p0ftf aywm7 czcz5 yu0ly 5d5gq 8dsv0 hhg22 v48rd kx56n 34v4w sw5yj wwwve rz9nw fh7gc\ngroup size: 5\nseparator: space\nnote: stdout is watch-only — public keys only, cannot spend\n"
     right: "note: stdout is watch-only — public keys only, cannot spend\n"
```

The RED file, verbatim:

```rust
//! `mk encode` prints each mk1 string ONCE, on stdout (Brian 2026-11-22,
//! reversing D4 of mnemonic-engrave `design/SPEC_constellation_cli_uniformity.md`;
//! `design/FIX_mk_single_print.md`).
//!
//! The grouped stderr engraving card, and its `group size:` / `separator:`
//! lines, are gone. `--group-size N` groups STDOUT on request (default 0,
//! ungrouped), as `mt encode` and `mnemonic bundle` do, and `--out FILE`
//! receives exactly the bytes stdout would have carried. `--keys` with two or
//! more records labels each card on stderr by its line range, so a duplicate
//! record stays visible (F-311) without a second copy of any string.

use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

/// mk-cli's `V15_84_TEST` fixture: a testnet account xpub at m/84'/1'/0',
/// master fingerprint 73c5da0a. Public material only.
const TPUB: &str = "tpubDC9Go1KDateW3gS8VXZ6DD1Xu7PgoTdPcf1MX9Z6qVLiHbaeDJ78swPyuQ8YQY19QjtrzkfkZSXwqCcb7XArtid1iLq8Vy55Ydfm4giZh6X";
const STUB: &str = "deadbeef";
const NOTE: &str = "note: stdout is watch-only \u{2014} public keys only, cannot spend\n";

fn mk() -> Command {
    Command::cargo_bin("mk").expect("mk binary")
}

fn encode(extra: &[&str]) -> std::process::Output {
    let mut args = vec![
        "encode",
        "--xpub",
        TPUB,
        "--origin-fingerprint",
        "73c5da0a",
        "--origin-path",
        "m/84'/1'/0'",
        "--policy-id-stub",
        STUB,
    ];
    args.extend_from_slice(extra);
    mk().args(&args).output().expect("run mk")
}

fn text(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).expect("utf8")
}

/// Every 5 characters a space: the `--group-size 5` form of one line.
fn grouped5(s: &str) -> String {
    let c: Vec<char> = s.chars().collect();
    c.chunks(5)
        .map(|g| g.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn encode_prints_each_string_once() {
    let o = encode(&[]);
    assert_eq!(o.status.code(), Some(0));
    let out = text(&o.stdout);
    let err = text(&o.stderr);
    assert_eq!(out.lines().count(), 2, "two mk1 strings; got {out:?}");
    for l in out.lines() {
        assert!(
            l.starts_with("mk1") && !l.contains(' '),
            "ungrouped mk1; got {l:?}"
        );
    }
    assert_eq!(err, NOTE, "stderr carries only the watch-only note");
}

#[test]
fn encode_group_size_groups_stdout() {
    let plain = text(&encode(&[]).stdout);
    let o = encode(&["--group-size", "5"]);
    assert_eq!(o.status.code(), Some(0));
    let want: String = plain.lines().map(|l| grouped5(l) + "\n").collect();
    assert_eq!(text(&o.stdout), want, "stdout grouped by 5 with a space");
    assert_eq!(text(&o.stderr), NOTE);
}

#[test]
fn encode_out_writes_the_stdout_bytes() {
    let grouped = text(&encode(&["--group-size", "5"]).stdout);
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("card.mk1");
    let o = encode(&["--group-size", "5", "--out", f.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(text(&o.stdout), "", "stdout is empty under --out");
    assert_eq!(std::fs::read_to_string(&f).unwrap(), grouped);
    assert_eq!(text(&o.stderr), NOTE);
}

#[test]
fn encode_keys_labels_each_card_on_stderr() {
    // The same record twice: F-311's duplicate, which mints two byte-identical
    // cards. The labels are the only thing that shows there were two.
    let dir = tempfile::tempdir().unwrap();
    let kf = dir.path().join("keys.txt");
    let rec = format!("[73c5da0a/84'/1'/0']{TPUB}\n");
    std::fs::write(&kf, format!("{rec}{rec}")).unwrap();
    let o = mk()
        .args([
            "encode",
            "--keys",
            kf.to_str().unwrap(),
            "--policy-id-stub",
            STUB,
        ])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let out = text(&o.stdout);
    assert_eq!(
        out.lines().count(),
        4,
        "two cards of two strings; got {out:?}"
    );
    assert_eq!(
        text(&o.stderr),
        format!("card 1 of 2: lines 1-2\ncard 2 of 2: lines 3-4\n{NOTE}")
    );
}
```

## 5. Tests that change

Measured: the full suite run against the section-3 production change with the base tests failed exactly 7 rows
(`/scratch/cache/sp-mk-full1.log`): `encode_grouping_flags::{group_size_no_longer_reaches_stdout,
the_stderr_card_carries_the_grouped_form_by_default, group_size_0_makes_the_card_unbroken_but_still_a_card,
whitespace_separator_still_accepted_both_spellings}`, `keys_batch::{stdout_carries_mk1_lines_only_and_the_boundary_moves_to_the_card,
record_order_follows_file_order}`, `channels::in_accepts_a_grouped_card`. Their change:

| test | change |
|---|---|
| `encode_grouping_flags::group_size_no_longer_reaches_stdout` | deleted; superseded by `single_print::encode_group_size_groups_stdout` |
| `encode_grouping_flags::the_stderr_card_carries_the_grouped_form_by_default` | deleted; superseded by `single_print::encode_prints_each_string_once` |
| `encode_grouping_flags::group_size_0_makes_the_card_unbroken_but_still_a_card` | deleted; there is no card |
| `encode_grouping_flags::whitespace_separator_still_accepted_both_spellings` | passes `--group-size 5` and asserts stdout's first line equals the space-5 regrouping of the default run's first line |
| `encode_grouping_flags` module doc and section headers | rewritten for the single print (no behaviour) |
| `keys_batch::stdout_carries_mk1_lines_only_and_the_boundary_moves_to_the_card` | renamed `stdout_carries_mk1_lines_only_and_stderr_labels_each_card`; the stderr blank-line count is replaced by: exactly one `card <i> of <n>: lines <a>-<b>` line per record, ranges contiguous from 1 and covering stdout |
| `keys_batch::record_order_follows_file_order` | the first card is read as stdout lines `1..=B` from the label `card 1 of N: lines 1-B` instead of from the stderr card |
| `channels::in_accepts_a_grouped_card` | the grouped file is `mk encode --group-size 5`'s stdout instead of the stderr card's `mk1` lines |

Exact diff (`crates/mk-cli/tests/`, excluding the new `single_print.rs`):

```diff
diff --git i/crates/mk-cli/tests/channels.rs w/crates/mk-cli/tests/channels.rs
index 51be014..230587a 100644
--- i/crates/mk-cli/tests/channels.rs
+++ w/crates/mk-cli/tests/channels.rs
@@ -403,14 +403,10 @@ fn in_on_the_reading_verbs_equals_the_positional_run() {
 #[test]
 fn in_accepts_a_grouped_card() {
     let d = tmpdir();
-    let run = mk().args(single_card_args()).output().unwrap();
-    let unbroken = stdout_of(&run);
-    let card = String::from_utf8(run.stderr.clone()).unwrap();
-    let mut grouped = String::new();
-    for l in card.lines().filter(|l| l.starts_with("mk1")) {
-        grouped.push_str(l);
-        grouped.push('\n');
-    }
+    let unbroken = stdout_of(&mk().args(single_card_args()).output().unwrap());
+    let mut grouped_args = single_card_args();
+    grouped_args.extend(["--group-size".to_string(), "5".to_string()]);
+    let grouped = stdout_of(&mk().args(grouped_args).output().unwrap());
     assert!(grouped.contains(' '), "precondition: the card is grouped");
 
     let g = d.path().join("grouped.mk1");
diff --git i/crates/mk-cli/tests/encode_grouping_flags.rs w/crates/mk-cli/tests/encode_grouping_flags.rs
index 09c3c06..ebc9120 100644
--- i/crates/mk-cli/tests/encode_grouping_flags.rs
+++ w/crates/mk-cli/tests/encode_grouping_flags.rs
@@ -1,17 +1,11 @@
-//! `mk encode` display-grouping flags (P3, SPEC §6a/§6b/§6c).
+//! `mk encode` display-grouping flags (single print, Brian 2026-11-22).
 //!
-//! **The grouped form moved off stdout.** §6a rules that `encode`'s stdout is
-//! "the canonical artifact, ungrouped, nothing else"; §6b rules that
-//! `--group-size` / `--separator` "affect the stderr card only"; §6c requires
-//! `md` and `mk` — which had no card at all, only a one-line `note:` — to grow
-//! one, and says the minimum it must carry is the grouped string itself, since
-//! after this change that is the only place it exists.
+//! Each mk1 string is printed ONCE, on stdout. `--group-size N` groups stdout
+//! on request (default 0, the canonical unbroken artifact), as `mt encode` and
+//! `mnemonic bundle` do; there is no stderr engraving card. The stdout/stderr
+//! byte rows live in `single_print.rs`; this file keeps the flag-surface rows.
 //!
-//! So the assertions below are a PAIR at every step: what left stdout, and
-//! where it went. A test that only checked stdout would pass equally against an
-//! implementation that deleted the grouped form outright.
-//!
-//! `--json` is UNCHANGED and out of scope this cycle (§6b).
+//! `--json` is UNCHANGED: it ignores the grouping flags.
 
 use std::process::Command;
 
@@ -96,83 +90,12 @@ fn encode_stdout_is_the_unbroken_artifact_under_default_flags() {
     }
 }
 
-/// `--group-size 5` and `--group-size 0` produce BYTE-IDENTICAL stdout.
-///
-/// This is the assertion that says the flag no longer reaches stdout at all,
-/// rather than that one particular default happens to be unbroken.
-#[test]
-fn group_size_no_longer_reaches_stdout() {
-    let grouped = encode(&["--group-size", "5"]);
-    let unbroken = encode(&["--group-size", "0"]);
-    assert!(grouped.status.success() && unbroken.status.success());
-    assert_eq!(
-        grouped.stdout, unbroken.stdout,
-        "--group-size must not change stdout (SPEC §6b: the stderr card only)"
-    );
-}
-
 // ──────────────────────────────────────────────────────────────────────────
-// §6c — the engraving card, on stderr.
-
+// `--json` ignores the grouping flags.
 // ──────────────────────────────────────────────────────────────────────────
 
-/// The card carries the grouped form, then its two settings, then the advisory.
-///
-/// The grouped line is recomputed from the stdout artifact, so this fails if
-/// the card renders a DIFFERENT string as well as if it renders none.
-#[test]
-fn the_stderr_card_carries_the_grouped_form_by_default() {
-    let out = encode(&[]);
-    assert!(out.status.success(), "stderr={}", stderr_of(&out));
-    let stdout = stdout_of(&out);
-    let stderr = stderr_of(&out);
-
-    for artifact in stdout.lines() {
-        let want = regroup(artifact, 5, ' ');
-        assert!(
-            stderr.lines().any(|l| l == want),
-            "card must carry the space-5 grouped form of {artifact:?}; stderr={stderr:?}"
-        );
-    }
-    assert!(
-        stderr.lines().any(|l| l == "group size: 5"),
-        "card names its group size; stderr={stderr:?}"
-    );
-    assert!(
-        stderr.lines().any(|l| l == "separator: space"),
-        "card names its separator; stderr={stderr:?}"
-    );
-    assert_eq!(
-        stderr.lines().last(),
-        Some(ADVISORY),
-        "the existing advisory stays LAST; stderr={stderr:?}"
-    );
-}
-
-/// `--group-size 0` yields a card whose artifact line is unbroken.
-///
-/// The card is still emitted: an operator who asked for no grouping still needs
-/// to see what to transcribe, and `--no-engraving-card` is deliberately NOT
-/// added to `mk` (§6c names it for `ms` and `mnemonic` only).
-#[test]
-fn group_size_0_makes_the_card_unbroken_but_still_a_card() {
-    let out = encode(&["--group-size", "0"]);
-    assert!(out.status.success(), "stderr={}", stderr_of(&out));
-    let artifact = first_line(&out);
-    let stderr = stderr_of(&out);
-    assert!(
-        stderr.lines().any(|l| l == artifact),
-        "card carries the unbroken string verbatim; stderr={stderr:?}"
-    );
-    assert!(
-        stderr.lines().any(|l| l == "group size: 0"),
-        "card reports the group size it was given; stderr={stderr:?}"
-    );
-    assert_eq!(stderr.lines().last(), Some(ADVISORY));
-}
-
-/// `--json` is out of scope (§6b): the JSON envelope stays the ONLY thing on
-/// stdout, and no card is printed in front of it.
+/// `--json` is unchanged: the JSON envelope is the ONLY thing on stdout, and
+/// stderr carries only the advisory.
 #[test]
 fn json_mode_emits_no_card() {
     let out = encode(&["--json"]);
@@ -225,17 +148,17 @@ fn retired_separator_keywords_are_refused() {
 #[test]
 fn whitespace_separator_still_accepted_both_spellings() {
     for spelling in ["space", " "] {
-        let out = encode(&["--separator", spelling]);
+        let out = encode(&["--group-size", "5", "--separator", spelling]);
         assert!(
             out.status.success(),
             "--separator {spelling:?} must still work; stderr={}",
             stderr_of(&out)
         );
-        let artifact = first_line(&out);
-        let stderr = stderr_of(&out);
-        assert!(
-            stderr.lines().any(|l| l == regroup(&artifact, 5, ' ')),
-            "space-grouped card; stderr={stderr:?}"
+        let artifact = first_line(&encode(&[]));
+        assert_eq!(
+            first_line(&out),
+            regroup(&artifact, 5, ' '),
+            "space-grouped stdout"
         );
     }
 }
diff --git i/crates/mk-cli/tests/keys_batch.rs w/crates/mk-cli/tests/keys_batch.rs
index c48a779..a976cdd 100644
--- i/crates/mk-cli/tests/keys_batch.rs
+++ w/crates/mk-cli/tests/keys_batch.rs
@@ -143,13 +143,13 @@ fn batch_matches_per_key_loop() {
 /// fixture and a two-record file give different totals while the blank count
 /// reproduces invariantly.
 ///
-/// The card boundary moves to the stderr engraving card, asserted here too. That
+/// The card boundary moves to a stderr label per card, asserted here too. That
 /// is not decoration: a key file carrying the same BIP-380 record twice is
 /// accepted at exit 0 and emits two byte-identical cards sharing one chunk-set
 /// id (F-311), so the boundary is NOT recoverable from the headers, and the
-/// blank line was the only signal the duplicate had been accepted.
+/// labels are the only signal the duplicate had been accepted.
 #[test]
-fn stdout_carries_mk1_lines_only_and_the_boundary_moves_to_the_card() {
+fn stdout_carries_mk1_lines_only_and_stderr_labels_each_card() {
     let body = keyfile_body(&KEYS);
     let kf = write_keyfile(&body);
     let output = mk()
@@ -176,14 +176,20 @@ fn stdout_carries_mk1_lines_only_and_the_boundary_moves_to_the_card() {
         "no blank line survives on stdout; got {out:?}"
     );
 
-    // ...and the boundary it carried is now on the human-facing card, once per
-    // pair of cards. `card` also ends with the output-class advisory, which is
-    // not blank, so a trailing blank cannot be miscounted here.
-    assert_eq!(
-        card.lines().filter(|l| l.is_empty()).count(),
-        KEYS.len() - 1,
-        "one blank line BETWEEN each pair of cards, on stderr; got {card:?}"
-    );
+    // ...and the boundary it carried is now a stderr label per card, whose line
+    // ranges tile stdout exactly (single print, 2026-11-22).
+    let mut next = 1usize;
+    let labels: Vec<&str> = card.lines().filter(|l| l.starts_with("card ")).collect();
+    assert_eq!(labels.len(), KEYS.len(), "one label per card; got {card:?}");
+    for (i, l) in labels.iter().enumerate() {
+        let range = l
+            .strip_prefix(&format!("card {} of {}: lines ", i + 1, KEYS.len()))
+            .unwrap_or_else(|| panic!("label {i} malformed: {l:?}"));
+        let (a, b) = range.split_once('-').unwrap();
+        assert_eq!(a.parse::<usize>().unwrap(), next, "ranges are contiguous");
+        next = b.parse::<usize>().unwrap() + 1;
+    }
+    assert_eq!(next - 1, out.lines().count(), "the ranges cover stdout");
 
     let single = stdout_of(
         &mk()
@@ -436,21 +442,18 @@ fn record_order_follows_file_order() {
         .output()
         .unwrap();
     let batch = stdout_of(&output);
-    // SPEC §6a took the blank line off stdout, so a consumer can no longer
-    // `split("\n\n")` the ARTIFACT into cards. The boundary lives on the stderr
-    // engraving card now, and with `--group-size 0` its lines are the same
-    // unbroken strings -- so the first card is still recoverable, from the card.
-    let card = String::from_utf8(output.stderr.clone()).unwrap();
-    let first_batch_card: Vec<&str> = card
-        .split("\n\n")
-        .next()
-        .unwrap()
+    // The boundary is the stderr label `card 1 of N: lines 1-B` (single print,
+    // 2026-11-22): the first card is stdout's lines 1..=B.
+    let labels = String::from_utf8(output.stderr.clone()).unwrap();
+    let first = labels
         .lines()
-        .filter(|l| l.starts_with("mk1"))
-        .collect();
+        .find_map(|l| l.strip_prefix(&format!("card 1 of {}: lines 1-", KEYS.len())))
+        .unwrap_or_else(|| panic!("no label for the first card; stderr={labels:?}"));
+    let b: usize = first.parse().unwrap();
+    let first_batch_card: Vec<&str> = batch.lines().take(b).collect();
     assert!(
         !first_batch_card.is_empty(),
-        "the card must still delimit the first record; stderr={card:?}"
+        "the label must delimit the first record; stderr={labels:?}"
     );
 
     let (fp, path, x) = KEYS[0];
```

## 6. Gate (mk CI jobs, run locally on the GREEN build: section 3 + section 5 + `single_print.rs`)

| CI step | command | result |
|---|---|---|
| test | `cargo nextest run --locked --workspace --no-fail-fast` (`--build-jobs 6 -j 6`, own target dir) | 424 passed, 0 failed (`/scratch/cache/sp-mk-full2.log`; re-run in the fold round on `origin/main` 3755d47 with the M-3 hunks: `/scratch/cache/fold-sp-mk-green.log`, 424 passed, clippy and fmt clean) |
| doc tests | `cargo test --locked --workspace --doc` | ok |
| clippy | `cargo clippy --locked --workspace --all-targets -- -D warnings` | clean |
| fmt | `cargo fmt --all --check` | clean |
| docs-only self-test | `bash ci/docs-only.test.sh` | exit 0 |
| display-grouping vectors | `cd design && sha256sum -c display-grouping-vectors.tsv.sha256` | OK (file untouched) |
| vectors-roundtrip | `mk vectors --out DIR` + the CI `jq` diff against `crates/mk-codec/src/test_vectors/v0.1.json` | equal (debug build; CI uses `--release`, not run here) |
| freebsd compile gate, fuzz-smoke, musl, vendor-freshness | not run | no dependency, `Cargo.lock` or codec change |

The build re-runs the whole gate on its own commits; this table is the author's pre-check, not the build's evidence.

## 7. Build order

1. RED commit: add `crates/mk-cli/tests/single_print.rs` (section 4); run the row command; see 4 FAIL.
2. GREEN commit: section 3 production change, section 5 test changes, CHANGELOG `[Unreleased]` entry (section 1);
   full gate (section 6).
3. Release (versioning is the release step's call; PROPOSED next: mk-cli 0.17.0, a CLI-output change).

## 8. Lockstep

### 8.0 Landing order (review r1 I-4; replaces steps 3-5 of the author report's "Landing order"; steps 1-2 unchanged)

3. mnemonic-gui: `pinned-upstream.toml` md/ms/mk tags + the schema edits of md 8c, ms 8c, mk 8b; schema-mirror; gui
   release.
4. mnemonic-toolkit, ONE commit: bump the md/ms/mk install pins AND the mnemonic-gui install pin to the step-3
   release, with the toolkit lockstep of all three FIX docs (incl. the I-3 sites in md 8b, ms 8b and below, and
   `docs/manual-gui/tests/expected_gui_schema_inventory.json`), so `{#installer-ahead}` never names these three
   releases; the `manual` job incl. doc-flag-lint mutation self-test. The replacement text for the
   `{#installer-ahead}` section (`82-binary-and-launch.md`) is in section 8a-bis below (one place for all three).

Why: with the toolkit installer ahead of the GUI, the installed GUI's schema still says `default_value: Some("5")`
for `--group-size` (mk.rs:71); its `is_at_default` (mnemonic-gui `src/form/invocation.rs:96-121`) suppresses a Number
flag equal to the schema default, so a user who sets 5 sends no `--group-size`, and the new CLI applies 0: unbroken
output with no message. If the toolkit must land first instead, each FIX doc's toolkit section must carry this
`{#installer-ahead}` sentence (PROPOSED text, from the review): "On the `md encode`, `md descriptor --emit md1`,
`ms encode` and `mk encode` forms, this GUI does not send `--group-size 5` (its shown default), and these CLI
releases print the unbroken string unless `--group-size` is given: to print groups of five, run the CLI with
`--group-size 5`."

### 8a. Toolkit manual (`mnemonic-toolkit` `docs/manual/src/40-cli-reference/44-mk-cli.md`), lands in step 4 (the toolkit's one commit)

Line 42 (`--out` row) becomes:

    | `--out <FILE>` | write the mk1 artifact to FILE, created `0600`, instead of stdout; the file holds exactly what stdout would (grouped under `--group-size`); overwrites an existing file (operator ruling 2026-08-26) |

Line 47 (`--group-size` row) becomes:

    | `--group-size <N>` | group each mk1 string on stdout every N characters, for hand engraving (default `0` = unbroken). `--out` receives the same bytes; `--json` stays unbroken. Separator-stripping on intake means grouped and unbroken cards both re-ingest on `decode`/`verify`/etc. |

Line 48 (`--separator` row) becomes:

    | `--separator <space>` | the separator for `--group-size`: `space` (keyword, the default) or the literal `" "`. Whitespace only: `hyphen` and `comma` (and the literals `-` and `,`) are refused (exit 64); a card already grouped with them still decodes |

Lines 70-73 (the first sentence pair of `### Output`), from "Text mode prints only the artifact" through "then the
watch-only note.", become:

    Text mode prints each mk1 string once, on stdout: one per line, one per
    chunk, unbroken by default and grouped under `--group-size`. stderr
    carries only the watch-only note; with `--keys` and two or more cards
    it is preceded by one `card <i> of <n>: lines <a>-<b>` line per card. JSON

(the paragraph continues unchanged from "mode emits a `{ "schema_version", ...`").

No flag is added or removed, so `ci/doc-flag-lint.test.sh` and the manual lint's flag-coverage step are unaffected;
the toolkit's `manual` CI job is the check.

### 8a-ter. Toolkit GUI manual, mk page (review r1 I-3), lands in step 4

`docs/manual-gui/src/70-mk/73-encode.md` (line anchors @cdedd8a2; the gui-manual lints check flag names, outlines and
CLI pins, not defaults, so nothing else catches these):

- line 31 becomes: "— group each mk1 string on stdout every N characters (default 0 = unbroken)"; line 32 becomes: "—
  separator keyword for `--group-size` (`space`; default `space`)".
- line 46: "default `5`" becomes "default `0`", and the sentence "break each emitted `mk1` string into groups" becomes
  "print each `mk1` string on stdout in groups".
- line 57: "Dropdown (`space`|`hyphen`|`comma`; default `space`)" becomes "Dropdown (`space`; default `space`; `hyphen`
  and `comma` were retired in mk-cli 0.14.0 and are refused at exit 64)"; delete the `hyphen` and `comma` outline
  bullets (lines 67-68) and sections (lines 75-86); lines 59-60 "it affects only the rendered card" becomes "it
  affects only the printed form".

The review's I-3 list also covers md 53/54/5A/5E and ms 63 (those are in md 8b and ms 8b) and
`82-binary-and-launch.md` (section 8a-bis).

### 8a-bis. Toolkit GUI manual, `{#installer-ahead}` section (`docs/manual-gui/src/80-troubleshooting/82-binary-and-launch.md`) (review r1 I-4), PROPOSED text, lands in step 4

This is the only authored text in the fold. Under the section 8.0 order the GUI release of step 3 carries the new
md/ms/mk tags, so in the step-4 commit the manual's `[manual-gui]` md/ms/mk tags equal the installer's and
`tests/check_cli_pins.py` (a) calls their `[installer-ahead]` entries stale: delete `md-cli-tag-implied`,
`ms-cli-tag-implied` and `mk-cli-tag-implied` from `pinned-upstream.toml` `[installer-ahead]`. What remains of the
section depends on one fact the builder reads from `scripts/install.sh` and `pinned-upstream.toml` at build time
(not decided here):

Under section 8.0 the GUI's `mnemonic` tag stays `mnemonic-toolkit-v0.105.1` and the installer's stays newer
(`v0.107.0` at cdedd8a2), so the section stays, reduced to `mnemonic`. Lines 45-65 of `82-binary-and-launch.md`
@cdedd8a2 (the opening paragraph through "First `mnemonic`, three outputs:") become the text below; the three
`mnemonic` bullets (lines 67-85) stay unchanged; lines 87-195 (from "Then `ms`, five outputs:" through the end of
the `md` list, including the `mk encode` bullet) are deleted. The sentence "The toolkit installer installs newer
releases of all four (next section)." becomes "The toolkit installer installs a newer release of `mnemonic` (next
section).", and "the next section lists the four it installs newer than these" becomes "the next section lists
the one it installs newer than these". In `pinned-upstream.toml`, the comment's first four lines (116-119, "The
installer runs AHEAD of mnemonic-gui v0.63.0 on four CLIs: ..." through "...shipped before the GUI re-pinned to
them. This manual keeps documenting the GUI's tier above;") become "# The installer runs AHEAD of mnemonic-gui
<the step-3 tag> on one CLI: mnemonic 0.107.0 (argv-secret refusals on five" / "# more flags) shipped before the
GUI re-pinned to it. This manual keeps documenting the GUI's tier above;". STOP instead of editing if, at build
time, the GUI's `mnemonic` tag equals the installer's or the installer's `mnemonic` tag is no longer
`mnemonic-toolkit-v0.107.0` (the text below names 0.107.0 and its behaviour).

    The toolkit installer currently installs one CLI release newer than
    this GUI was built against: `mnemonic-toolkit-v0.107.0` (the GUI's
    label reads `Pinned: mnemonic 0.105.1`). The forms run unchanged. The
    0.107.0 `mnemonic` refuses a secret typed on the command line at exit
    2 for five more flags (`ms-shares combine --share`, `slip39 combine
    --share`, `seed-xor combine --share`, `import-wallet --ms1`,
    `verify-bundle --ms1`), but no form is affected: the GUI sends each
    secret over a private channel (`-` or `@env:`) or, on an OS without
    one, adds `--allow-argv-secret` itself. `mnemonic --version` printing
    the newer version is expected.

    Outputs differ, mostly on invalid or unusual input or on the recovery
    line of a card; none changes a derived wallet. Three `mnemonic`
    outputs:

The builder runs `check_cli_pins.py` (b) and updates every prose mention of a CLI version it names.

### 8b. mnemonic-gui (`src/schema/mk.rs`, `ENCODE_FLAGS`), lands in step 3 (the gui release, AHEAD of the toolkit)

- `--group-size`: `default_value: Some("0")` (the 5 -> 0 change is what makes the step order matter, section 8.0); `help: "Group each mk1 string on stdout every N characters, for hand
  engraving (default 0 = unbroken). Cosmetic — intake strips separators, so any grouping re-ingests."`
- `--separator`: `help: "Separator for --group-size: `space` only (hyphen and comma were retired). Cosmetic —
  non-load-bearing."` (the current text still lists `space|hyphen|comma`, stale since mk-cli 0.14.0).
- `pinned-upstream.toml` `[mk] tag` moves from `mk-cli-v0.13.0` to the release carrying this change. That jump spans
  0.14.0-0.16.0 too; the gui builder runs `schema-mirror` and folds any other drift it reports.

### 8c. Other consumers (no edit required)

- mnemonic-engrave `design/journeys/transcript.sh:103-111` captures `mk encode ... --group-size 0 2>&1` and FATALs
  unless exactly 2 `^mk1` lines appear. Measured with mk 0.16.0: 4 (the card repeats them). After this change: 2.
  The journey is fixed by this change, not broken.
- mnemonic-refugium prints `mk encode ...` commands for operators and reads mk1 "in groups or not"; no code reads
  the card. Its comments saying `--group-size 5` is mk's default become stale (proposed line in section 9).

## 9. Proposed FOLLOWUPS lines (direct-pushed after merge, not in the PR)

    - [mk-out-watch-only-note-says-stdout] status=open grade=M kind=ux owner=none filed=2026-11-22 src=coordinator/agent-reports/fix-doc-single-print.md :: mk encode --out still prints "note: stdout is watch-only" though stdout is empty; md encode --out does the same.

For mnemonic-refugium `design/FOLLOWUPS.md`:

    - [mr-comments-mk-group-size-default-5] status=open grade=N kind=docs owner=none filed=2026-11-22 src=coordinator/agent-reports/fix-doc-single-print.md :: Comments (mr-cli/tests/jf3_items.rs:20, mr-codec/src/plates.rs:293,323) say mk encode prints groups of 5 by default; after the single print (2026-11-22) the default is unbroken and grouping is --group-size 5 on stdout.

## 10. Not covered, STOP

- Not covered: the release workflow, musl and FreeBSD jobs (not run; no dependency change); a reviewer's judgement
  of the PROPOSED items in section 0.
- STOP: none. No consumer parses the stderr card (sweep in `coordinator/agent-reports/fix-doc-single-print.md`).
