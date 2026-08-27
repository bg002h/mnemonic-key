//! mstring display-grouping helpers (SPEC §3/§5, mstring-display-grouping cycle).
//!
//! Pure, dependency-free fns local to `mk-cli` (bin-only). Pinned byte-for-byte
//! to the canonical conformance vectors (`design/display-grouping-vectors.tsv`,
//! checksum-gated in CI) shared with the toolkit + sibling CLIs.

/// True for any display separator on intake: ALL Unicode whitespace + `-` + `,`
/// (SPEC §3.2). None appear in the codex32 alphabet or the `mk`/`1` structural
/// chars, so stripping is unambiguous.
pub fn is_display_separator(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == ','
}

/// Insert `separator` after every `group_size` chars (SPEC §3.1). `group_size == 0`
/// returns the input unchanged. Single line; ASCII-safe.
pub fn render_grouped(s: &str, group_size: usize, separator: char) -> String {
    if group_size == 0 {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + s.len() / group_size);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && i % group_size == 0 {
            out.push(separator);
        }
        out.push(ch);
    }
    out
}

/// Strip every display separator (SPEC §3.2) — used on intake before decode.
/// Idempotent; strips ONLY separators (a malformed card is never silently
/// "cleaned" into validity).
pub fn strip_display_separators(s: &str) -> String {
    s.chars().filter(|&c| !is_display_separator(c)).collect()
}

/// Human-readable name for a display separator, for the engraving card's
/// `separator:` line. A card that reported `separator: ' '` would be telling an
/// operator to type a quote mark.
pub fn separator_name(c: char) -> &'static str {
    match c {
        ' ' => "space",
        '-' => "hyphen",
        ',' => "comma",
        '\t' => "tab",
        _ => "whitespace",
    }
}

/// Write the stderr **engraving card** (SPEC §6c).
///
/// §6a rules that `encode`'s stdout is the canonical artifact ungrouped and
/// nothing else, which evicts the grouped form; §6c requires `md` and `mk` --
/// which had no card at all, only a one-line `note:` -- to grow one, *"since
/// after D4 that is the only place it exists"*. stdout is what a pipeline reads;
/// this is what a human transcribes onto a plate.
///
/// Shape follows **`ms`'s** card, measured: plain `label: value` lines on stderr
/// with no prefix character, grouped string first because that is the thing being
/// transcribed. `mnemonic bundle`'s `#`-prefixed card is the other
/// in-constellation precedent and is deliberately NOT followed -- its `#` mirrors
/// the comment headers on its own stdout, a surface `mk` does not have.
///
/// The caller emits the existing output-class advisory AFTER this, so that
/// advisory stays the last stderr line it has always been.
pub fn write_engraving_card<W: std::io::Write>(
    w: &mut W,
    cards: &[Vec<String>],
    group_size: usize,
    separator: char,
) {
    for (i, strings) in cards.iter().enumerate() {
        // A blank line BETWEEN cards, never before the first or after the last.
        // It used to sit on stdout, where §6a no longer admits it -- and it was
        // the only signal that `mk encode --keys` had silently accepted the same
        // BIP-380 record twice (F-311: duplicates mint two byte-identical cards
        // that share one chunk-set id, so the boundary is not recoverable from
        // the headers). The human-facing card is where that signal belongs.
        if i > 0 {
            let _ = writeln!(w);
        }
        for s in strings {
            let _ = writeln!(w, "{}", render_grouped(s, group_size, separator));
        }
    }
    let _ = writeln!(w, "group size: {group_size}");
    let _ = writeln!(w, "separator: {}", separator_name(separator));
}

/// Parse `--separator`: the keyword `space`, or the literal `" "`.
///
/// **`hyphen` and `comma` were removed (SPEC §6c: whitespace only, everywhere).**
/// The argument is cross-tool rather than per-tool, and that distinction is the
/// whole of it: `mk`'s own intake strips `-` and `,` as happily as whitespace
/// (see [`strip_display_separators`]), so a hyphen-grouped mk1 round-trips
/// through `mk` at exit 0 -- measured. But `mt` strips whitespace and NOTHING
/// else, so an operator who carries the habit between tools has a card `mt`'s own
/// verbs refuse, discovered after the plates are cut. The cost of the uniform
/// rule is two cosmetic options; the cost of getting it wrong is a plate.
///
/// The shared display-grouping conformance corpus still carries `hyphen` and
/// `comma` rows and is untouched by this: its consumer is the [`conformance`]
/// test below, which maps each keyword to a `char` itself and calls
/// [`render_grouped`], which takes a `char` and has no keyword vocabulary at all.
/// The narrowing is at the CLI's parser, one layer up.
///
/// clap value-parser; rejection is an exit-64 parse error (mk-cli maps all clap
/// errors to 64, `main.rs`).
pub fn parse_separator(s: &str) -> Result<char, String> {
    match s {
        "space" | " " => Ok(' '),
        // Named separately from the catch-all so the refusal can say what
        // replaced them. A message that only said "invalid" would leave an
        // operator with a working command line and no next step (SPEC §6h:
        // remedy text must be executable).
        "hyphen" | "-" | "comma" | "," => Err(format!(
            "separator {s:?} was removed: display grouping is whitespace-only across the \
             constellation, because a hyphen- or comma-grouped card is refused by tools that \
             strip whitespace and nothing else. Use `--separator space` (the default), or drop \
             the flag."
        )),
        other => Err(format!(
            "invalid separator {other:?}; expected `space` (or the literal \" \") -- display \
             grouping is whitespace-only"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_grouped_separators_and_unbroken() {
        assert_eq!(render_grouped("abcdefghij", 5, ' '), "abcde fghij");
        assert_eq!(render_grouped("abcdefghij", 5, '-'), "abcde-fghij");
        assert_eq!(render_grouped("abcdefghij", 5, ','), "abcde,fghij");
        assert_eq!(render_grouped("abcdefghij", 0, ' '), "abcdefghij");
        assert_eq!(render_grouped("abcde", 5, ' '), "abcde");
        assert_eq!(render_grouped("abcdefg", 3, '-'), "abc-def-g");
        assert_eq!(render_grouped("", 5, ' '), "");
    }

    #[test]
    fn strip_display_separators_ws_hyphen_comma() {
        assert_eq!(strip_display_separators("ab cd-ef,gh"), "abcdefgh");
        assert_eq!(strip_display_separators("mk1\tqp\r\nzr"), "mk1qpzr");
        let once = strip_display_separators("a b-c,d");
        assert_eq!(strip_display_separators(&once), once);
    }

    #[test]
    fn parse_separator_accepts_whitespace_only() {
        assert_eq!(parse_separator("space").unwrap(), ' ');
        assert_eq!(parse_separator(" ").unwrap(), ' ');
        assert!(parse_separator("bogus").is_err());
    }

    /// The retired keywords are refused, and the refusal NAMES what replaced
    /// them -- an "invalid separator" message alone would leave an operator with
    /// a command line that does not work and no next step (SPEC §6h).
    #[test]
    fn retired_separator_keywords_name_their_replacement() {
        for retired in ["hyphen", "-", "comma", ","] {
            let e = parse_separator(retired).expect_err("must be refused");
            assert!(
                e.contains("space"),
                "refusal for {retired:?} must name `space`; got {e:?}"
            );
        }
    }
}

/// Same canonical display-grouping vectors as the toolkit + the other siblings
/// (copy is checksum-pinned in CI). Proves mk-cli's render/strip match
/// byte-for-byte. SPEC §8. Bin-crate unit test (mk-cli is bin-only).
#[cfg(test)]
mod conformance {
    use super::{render_grouped, strip_display_separators};

    fn decode(f: &str) -> String {
        if f == "<empty>" {
            return String::new();
        }
        f.replace("<sp>", " ")
            .replace("<tab>", "\t")
            .replace("<lf>", "\n")
            .replace("<cr>", "\r")
    }

    fn sep(k: &str) -> char {
        match k {
            "space" => ' ',
            "hyphen" => '-',
            "comma" => ',',
            "none" => ' ',
            o => panic!("unknown separator keyword: {o}"),
        }
    }

    #[test]
    fn conformance_vectors_pass() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../design/display-grouping-vectors.tsv"
        );
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let mut lines = text.lines();
        assert_eq!(
            lines.next().expect("header"),
            "op\tinput\tgroup_size\tseparator\texpected\tnote",
            "vector header drift"
        );
        let mut n = 0usize;
        for (i, line) in lines.enumerate() {
            if line.is_empty() {
                continue;
            }
            let c: Vec<&str> = line.split('\t').collect();
            assert_eq!(c.len(), 6, "row {} not 6 fields: {line:?}", i + 2);
            let (op, input, gs, s, exp, note) =
                (c[0], decode(c[1]), c[2], c[3], decode(c[4]), c[5]);
            let gs: usize = gs
                .parse()
                .unwrap_or_else(|_| panic!("row {}: bad group_size", i + 2));
            let got = match op {
                "render" => render_grouped(&input, gs, sep(s)),
                "strip" => strip_display_separators(&input),
                o => panic!("row {}: unknown op {o:?}", i + 2),
            };
            assert_eq!(got, exp, "row {} ({note})", i + 2);
            n += 1;
        }
        assert!(n >= 20, "expected >=20 rows, got {n}");
    }
}
