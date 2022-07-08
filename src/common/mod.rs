pub mod entities;

use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;


const UNESCAPE_MD_RE : &str = r##"\\([!"#$%&'()*+,\-./:;<=>?@\[\\\]^_`{|}~])"##;
const ENTITY_RE      : &str = r##"&([A-Za-z#][A-Za-z0-9]{1,31});"##;

static DIGITAL_ENTITY_TEST_RE : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"(?i)^&#(x[a-f0-9]{1,8}|[0-9]{1,8})$"#).unwrap()
);
static UNESCAPE_ALL_RE        : Lazy<Regex> = Lazy::new(||
    Regex::new(&format!("{UNESCAPE_MD_RE}|{ENTITY_RE}")).unwrap()
);

pub fn is_valid_entity_code(code: u32) -> bool {
    // broken sequence
    if code >= 0xD800 && code <= 0xDFFF { return false; }
    // never used
    if code >= 0xFDD0 && code <= 0xFDEF { return false; }
    if (code & 0xFFFF) == 0xFFFF || (code & 0xFFFF) == 0xFFFE { return false; }
    // control codes
    if code <= 0x08 { return false; }
    if code == 0x0B { return false; }
    if code >= 0x0E && code <= 0x1F { return false; }
    if code >= 0x7F && code <= 0x9F { return false; }
    // out of range
    if code > 0x10FFFF { return false; }
    return true;
}

fn replace_entity_pattern(str: &str) -> Option<String> {
    if let Some(entity) = entities::ENTITIES_HASH.get(str) {
        Some((*entity).to_owned())
    } else if DIGITAL_ENTITY_TEST_RE.is_match(str) {
        let code = if str.starts_with('x') || str.starts_with('X') {
            u32::from_str_radix(&str[1..], 16).unwrap()
        } else {
            u32::from_str_radix(str, 10).unwrap()
        };

        if is_valid_entity_code(code) {
            Some(char::from_u32(code).unwrap().into())
        } else {
            None
        }
    } else {
        None
    }
}

pub fn unescape_all(str: &str) -> String {
    // TODO: cow
    if !str.contains('\\') && !str.contains('&') { return str.to_owned(); }

    UNESCAPE_ALL_RE.replace_all(str, |captures: &regex::Captures| {
        let s = captures.get(0).unwrap().as_str();
        if let Some(m) = captures.get(1) {
            // \" -> "
            m.as_str().to_owned()
        } else if let Some(replacement) = replace_entity_pattern(s) {
            // &quot; -> "
            replacement
        } else {
            s.to_owned()
        }
    }).to_string()
}

pub fn escape_html(str: &str) -> Cow<str> {
    // this should escape following characters: " < > &
    html_escape::encode_double_quoted_attribute(str)
}


// Helper to unify [reference labels].
//
pub fn normalize_reference(str: &str) -> String {
    static SPACE_RE : Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

    // Trim and collapse whitespace
    //
    let str = SPACE_RE.replace_all(str.trim(), " ");

    // .toLowerCase().toUpperCase() should get rid of all differences
    // between letter variants.
    //
    // Simple .toLowerCase() doesn't normalize 125 code points correctly,
    // and .toUpperCase doesn't normalize 6 of them (list of exceptions:
    // İ, ϴ, ẞ, Ω, K, Å - those are already uppercased, but have differently
    // uppercased versions).
    //
    // Here's an example showing how it happens. Lets take greek letter omega:
    // uppercase U+0398 (Θ), U+03f4 (ϴ) and lowercase U+03b8 (θ), U+03d1 (ϑ)
    //
    // Unicode entries:
    // 0398;GREEK CAPITAL LETTER THETA;Lu;0;L;;;;;N;;;;03B8;
    // 03B8;GREEK SMALL LETTER THETA;Ll;0;L;;;;;N;;;0398;;0398
    // 03D1;GREEK THETA SYMBOL;Ll;0;L;<compat> 03B8;;;;N;GREEK SMALL LETTER SCRIPT THETA;;0398;;0398
    // 03F4;GREEK CAPITAL THETA SYMBOL;Lu;0;L;<compat> 0398;;;;N;;;;03B8;
    //
    // Case-insensitive comparison should treat all of them as equivalent.
    //
    // But .toLowerCase() doesn't change ϑ (it's already lowercase),
    // and .toUpperCase() doesn't change ϴ (already uppercase).
    //
    // Applying first lower then upper case normalizes any character:
    // '\u0398\u03f4\u03b8\u03d1'.toLowerCase().toUpperCase() === '\u0398\u0398\u0398\u0398'
    //
    // Note: this is equivalent to unicode case folding; unicode normalization
    // is a different step that is not required here.
    //
    // Final result should be uppercased, because it's later stored in an object
    // (this avoid a conflict with Object.prototype members,
    // most notably, `__proto__`)
    //
    str.to_lowercase().to_uppercase()
}

// Finds last occurrence of `char` in `source`, returns number of characters from
// that last occurrence. If char is not found, return number of characters total.
//
// Examples:
// "abcde", 'e' -> 0
// "abcde", 'b' -> 3
// "abcde", 'z' -> 5
//
pub fn rfind_and_count(source: &str, char: char) -> usize {
    let mut result = 0;
    for c in source.chars().rev() {
        if c == char { break; }
        result += 1;
    }
    result
}

// Calculate number of spaces from `pos` to first non-space character
// or EOL. Tabs are expanded to variable number of spaces with tabstop = 4.
// Returns relative indent and offset of first non-space character.
pub fn find_indent_of(line: &str, mut pos: usize) -> (usize, usize) {
    let mut chars = line[pos..].chars();
    let mut indent = 0;

    loop {
        match chars.next() {
            Some('\t') => {
                let bs_count = rfind_and_count(&line[..pos], '\t');
                indent += 4 - bs_count % 4;
                pos += 1;
            }
            Some(' ') => {
                indent += 1;
                pos += 1;
            }
            _ => return ( indent, pos ),
        }
    }
}

// Input: a string of characters (presumed whitespaces, can be anything), where each one of
// them contributes 1 to indent (except for tabs, whose width may vary with tabstop = 4).
//
// This function returns any trailing whitespace with total length of `indent`.
//
// If an indent would split a tab, that tab is replaced with 4 spaces.
//
// Example: cut_right_whitespace_with_tabstops("\t\t", 6) would return "  \t" (two preceding
// spaces) because first tab gets expanded to 6 spaces.
//
pub fn cut_right_whitespace_with_tabstops(source: &str, indent: i32) -> Cow<str> {
    let (num_spaces, start) = calc_right_whitespace_with_tabstops(source, indent);

    if num_spaces > 0 {
        let mut result = " ".repeat(num_spaces as usize);
        result += &source[start..];
        Cow::Owned(result)
    } else {
        Cow::Borrowed(&source[start..])
    }
}

pub fn calc_right_whitespace_with_tabstops(source: &str, mut indent: i32) -> (usize, usize) {
    let mut start = source.len();
    let mut chars = source.char_indices().rev();

    while indent > 0 {
        match chars.next() {
            Some((pos, '\t')) => {
                // previous tab is guaranteed to finish at 0 modulo 4,
                // so we can finish counting there
                let indent_from_start = rfind_and_count(&source[..pos], '\t');
                let tab_width = 4 - indent_from_start as i32 % 4;

                if indent < tab_width {
                    return ( indent as usize, start );
                }

                indent -= tab_width;
                start = pos;
            }
            Some((pos, _)) => {
                indent -= 1;
                start = pos;
            }
            None => {
                start = 0;
                break;
            }
        }
    }

    ( 0, start )
}

#[cfg(test)]
mod tests {
    use super::cut_right_whitespace_with_tabstops as cut_ws;
    use super::rfind_and_count;
    use super::find_indent_of;

    #[test]
    fn rfind_and_count_test() {
        assert_eq!(rfind_and_count("", 'b'), 0);
        assert_eq!(rfind_and_count("abcde", 'e'), 0);
        assert_eq!(rfind_and_count("abcde", 'b'), 3);
        assert_eq!(rfind_and_count("abcde", 'z'), 5);
        assert_eq!(rfind_and_count("abcεπ", 'b'), 3);
    }

    #[test]
    fn find_indent_of_simple_test() {
        assert_eq!(find_indent_of("a", 0), (0, 0));
        assert_eq!(find_indent_of(" a", 0), (1, 1));
        assert_eq!(find_indent_of("   a", 0), (3, 3));
        assert_eq!(find_indent_of("    ", 0), (4, 4));
        assert_eq!(find_indent_of("\ta", 0), (4, 1));
        assert_eq!(find_indent_of(" \ta", 0), (4, 2));
        assert_eq!(find_indent_of("  \ta", 0), (4, 3));
        assert_eq!(find_indent_of("   \ta", 0), (4, 4));
        assert_eq!(find_indent_of("    \ta", 0), (8, 5));
    }

    #[test]
    fn find_indent_of_with_offset() {
        assert_eq!(find_indent_of("   a", 2), (1, 3));
        assert_eq!(find_indent_of("    a", 2), (2, 4));
        assert_eq!(find_indent_of("  \ta", 2), (2, 3));
        assert_eq!(find_indent_of("   \ta", 2), (2, 4));
        assert_eq!(find_indent_of("    \ta", 2), (6, 5));
        assert_eq!(find_indent_of("     \ta", 2), (6, 6));
    }

    #[test]
    fn find_indent_of_tabs_test() {
        assert_eq!(find_indent_of("  \t \ta", 1), (7, 5));
        assert_eq!(find_indent_of("  \t \ta", 2), (6, 5));
        assert_eq!(find_indent_of("  \t \ta", 3), (4, 5));
        assert_eq!(find_indent_of("  \t \ta", 4), (3, 5));
    }

    #[test]
    fn cut_ws_simple() {
        assert_eq!(cut_ws("abc", -1), "");
        assert_eq!(cut_ws("abc", 0), "");
        assert_eq!(cut_ws("abc", 1), "c");
        assert_eq!(cut_ws("abc", 2), "bc");
        assert_eq!(cut_ws("abc", 3), "abc");
        assert_eq!(cut_ws("abc", 4), "abc");
    }

    #[test]
    fn cut_ws_unicode() {
        assert_eq!(cut_ws("αβγδ", 1), "δ");
        assert_eq!(cut_ws("αβγδ ", 3), "γδ ");
    }

    #[test]
    fn cut_ws_expands_partial_tabs() {
        assert_eq!(cut_ws("\t", 1), " ");
        assert_eq!(cut_ws("\t", 2), "  ");
        assert_eq!(cut_ws("\t", 3), "   ");
        assert_eq!(cut_ws("\t\t\t", 5), " \t");
        assert_eq!(cut_ws("\t\t\t", 7), "   \t");
    }

    #[test]
    fn cut_ws_retains_full_tabs() {
        assert_eq!(cut_ws("\t\t\t", 4), "\t");
        assert_eq!(cut_ws("\t\t\t", 8), "\t\t");
    }

    #[test]
    fn cut_ws_proper_tabstops() {
        assert_eq!(cut_ws("a\t", 1), " ");
        assert_eq!(cut_ws("a\t", 2), "  ");
        assert_eq!(cut_ws("a\t", 3), "\t");
        assert_eq!(cut_ws("ab\t", 3), "b\t");
        assert_eq!(cut_ws("abc\t", 3), "bc\t");
    }

    #[test]
    fn cut_ws_proper_tabstops_nested() {
        assert_eq!(cut_ws("a\tb\t", 2), "  ");
        assert_eq!(cut_ws("a\tb\t", 3), "\t");
        assert_eq!(cut_ws("a\tb\t", 4), "b\t");
        assert_eq!(cut_ws("a\tb\t", 5), " b\t");
        assert_eq!(cut_ws("a\tb\t", 6), "  b\t");
        assert_eq!(cut_ws("a\tb\t", 7), "\tb\t");
        assert_eq!(cut_ws("a\tb\t", 8), "a\tb\t");
    }

    #[test]
    fn cut_ws_different_tabstops_nested() {
        assert_eq!(cut_ws("abc\tde\tf\tg", 3), "  g");
        /*assert_eq!(cut_ws("abc\tde\tf\tg", 4), "\tg");
        assert_eq!(cut_ws("abc\tde\tf\tg", 5), "f\tg");
        assert_eq!(cut_ws("abc\tde\tf\tg", 6), " f\tg");
        assert_eq!(cut_ws("abc\tde\tf\tg", 7), "\tf\tg");
        assert_eq!(cut_ws("abc\tde\tf\tg", 9), "de\tf\tg");
        assert_eq!(cut_ws("abc\tde\tf\tg", 10), "\tde\tf\tg");*/
    }
}
