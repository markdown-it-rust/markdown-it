pub mod html_blocks;
pub mod html_re;
pub mod entities;

use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;


const UNESCAPE_MD_RE : &str = r##"\\([!"#$%&'()*+,\-./:;<=>?@\[\\\]^_`{|}~])"##;
const ENTITY_RE      : &str = r##"&([A-Za-z#][A-Za-z0-9]{1,31});"##;

lazy_static! {
    static ref DIGITAL_ENTITY_TEST_RE : Regex = Regex::new(r#"(?i)^&#(x[a-f0-9]{1,8}|[0-9]{1,8})$"#).unwrap();
    static ref UNESCAPE_ALL_RE        : Regex = Regex::new(&format!("{UNESCAPE_MD_RE}|{ENTITY_RE}")).unwrap();
}

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
            u32::from_str_radix(&str, 10).unwrap()
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
    lazy_static! {
        static ref SPACE_RE : Regex = Regex::new(r"\s+").unwrap();
    };

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
