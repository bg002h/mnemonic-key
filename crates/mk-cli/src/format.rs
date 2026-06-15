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

/// Parse `--separator`: keyword (`space|hyphen|comma`) or literal (`" "|-|,`).
/// SPEC §5. clap value-parser; rejection is an exit-64 parse error (mk-cli maps
/// all clap errors to 64, `main.rs`).
pub fn parse_separator(s: &str) -> Result<char, String> {
    match s {
        "space" | " " => Ok(' '),
        "hyphen" | "-" => Ok('-'),
        "comma" | "," => Ok(','),
        other => Err(format!(
            "invalid separator {other:?}; expected one of: space|hyphen|comma (or the literal char)"
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
    fn parse_separator_keyword_and_literal() {
        assert_eq!(parse_separator("space").unwrap(), ' ');
        assert_eq!(parse_separator(" ").unwrap(), ' ');
        assert_eq!(parse_separator("hyphen").unwrap(), '-');
        assert_eq!(parse_separator("comma").unwrap(), ',');
        assert!(parse_separator("bogus").is_err());
    }
}
