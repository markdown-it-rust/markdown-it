// Regexps to match html elements
//
#![allow(non_upper_case_globals)]
use const_format::formatcp;
use once_cell::sync::Lazy;
use regex::Regex;

const attr_name     : &str = r#"[a-zA-Z_:][a-zA-Z0-9:._-]*"#;

const unquoted      : &str = r#"[^"'=<>`\x00-\x20]+"#;
const single_quoted : &str = r#"'[^']*'"#;
const double_quoted : &str = r#""[^"]*""#;

const attr_value    : &str = formatcp!("(?:{unquoted}|{single_quoted}|{double_quoted})");

const attribute     : &str = formatcp!("(?:\\s+{attr_name}(?:\\s*=\\s*{attr_value})?)");

const open_tag      : &str = formatcp!("<[A-Za-z][A-Za-z0-9\\-]*{attribute}*\\s*/?>");

const close_tag     : &str = r#"</[A-Za-z][A-Za-z0-9\-]*\s*>"#;
const comment       : &str = r#"<!---->|<!--(?:-?[^>-])(?:-?[^-])*-->"#;
const processing    : &str = r#"<[?][\s\S]*?[?]>"#;
const declaration   : &str = r#"<![A-Z]+\s+[^>]*>"#;
const cdata         : &str = r#"<!\[CDATA\[[\s\S]*?\]\]>"#;

pub static HTML_TAG_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        formatcp!("^(?:{open_tag}|{close_tag}|{comment}|{processing}|{declaration}|{cdata})")
    ).unwrap()
});

pub static HTML_OPEN_CLOSE_TAG_RE : Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        formatcp!("^(?:{open_tag}|{close_tag})")
    ).unwrap()
});

pub static HTML_LINK_OPEN : Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^<a[>\s]"#).unwrap()
});

pub static HTML_LINK_CLOSE : Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^</a\s*>"#).unwrap()
});
