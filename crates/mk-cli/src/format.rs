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
