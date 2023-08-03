//! Link validator and formatter

use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::Debug;

pub trait LinkFormatter : Debug + Send + Sync {
    /// Validate link url, return `Some(())` if it is allowed
    /// and `None` if it is a security risk.
    fn validate_link(&self, url: &str) -> Option<()>;

    /// Encode link url to a machine-readable format,
    /// which includes url-encoding, punycode, etc.
    fn normalize_link(&self, url: &str) -> String;

    /// Decode link url to a human-readable format.
    fn normalize_link_text(&self, url: &str) -> String;
}

/// Default link validator and formatter for markdown-it.
///
/// This validator can prohibit more than really needed to prevent XSS. It's a
/// tradeoff to keep code simple and to be secure by default.
///
/// If you need different setup - override validator method as you wish. Or
/// replace it with dummy function and use external sanitizer.
///
#[derive(Default, Debug)]
pub struct MDLinkFormatter;

impl MDLinkFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl LinkFormatter for MDLinkFormatter {
    fn validate_link(&self, url: &str) -> Option<()> {
        // url should be normalized at this point, and existing entities are decoded
        static BAD_PROTO_RE : Lazy<Regex> = Lazy::new(||
            Regex::new(r#"(?i)^(vbscript|javascript|file|data):"#).unwrap()
        );

        static GOOD_DATA_RE : Lazy<Regex> = Lazy::new(||
            Regex::new(r#"(?i)^data:image/(gif|png|jpeg|webp);"#).unwrap()
        );

        if !BAD_PROTO_RE.is_match(url) || GOOD_DATA_RE.is_match(url) {
            Some(())
        } else {
            None
        }
    }

    fn normalize_link(&self, url: &str) -> String {
        mdurl::urlencode::encode(url, mdurl::urlencode::ENCODE_DEFAULT_CHARS, true).into()
    }

    fn normalize_link_text(&self, url: &str) -> String {
        url.to_owned()
    }
}


#[cfg(test)]
mod tests {
    use super::LinkFormatter;
    use super::MDLinkFormatter;

    #[test]
    fn should_allow_normal_urls() {
        let fmt = MDLinkFormatter::new();
        assert!(fmt.validate_link("http://example.org").is_some());
        assert!(fmt.validate_link("HTTPS://example.org").is_some());
    }

    #[test]
    fn should_allow_plain_text() {
        let fmt = MDLinkFormatter::new();
        assert!(fmt.validate_link("javascript").is_some());
        assert!(fmt.validate_link("/javascript:link").is_some());
    }

    #[test]
    fn should_not_allow_some_protocols() {
        let fmt = MDLinkFormatter::new();
        assert!(fmt.validate_link("javascript:alert(1)").is_none());
        assert!(fmt.validate_link("JAVASCRIPT:alert(1)").is_none());
        assert!(fmt.validate_link("vbscript:alert(1)").is_none());
        assert!(fmt.validate_link("VbScript:alert(1)").is_none());
        assert!(fmt.validate_link("file:///123").is_none());
    }

    #[test]
    fn should_not_allow_data_url_except_whitelisted() {
        let fmt = MDLinkFormatter::new();
        assert!(fmt.validate_link("data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7").is_some());
        assert!(fmt.validate_link("data:text/html;base64,PHNjcmlwdD5hbGVydCgnWFNTJyk8L3NjcmlwdD4K").is_none());
    }
}
