use super::AsciiSet;

const DIGITS : &[ u8; 16 ] = b"0123456789ABCDEF";

/// Encode unsafe characters with percent-encoding, skipping already
/// encoded sequences.
///
///  - string        - string to encode
///  - exclude       - list of characters to ignore (in addition to a-zA-Z0-9)
///  - keep_escaped  - don't encode '%' in a correct escape sequence
///
/// ```rust
/// use markdown_it::common::mdurl::{AsciiSet, encode};
///
/// const SAFE_SET : AsciiSet = AsciiSet::from(";/?:@&=+$,-_.!~*'()#");
/// assert_eq!(encode("[hello]", SAFE_SET, true), "%5Bhello%5D");
/// ```
pub fn encode(string: &str, exclude: AsciiSet, keep_escaped: bool) -> String {
    let mut result = Vec::new();
    let bytes = string.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let byte = bytes[i];
        let should_encode = byte >= 0x80 || !exclude.has(byte);

        if keep_escaped && byte == b'%' && i + 2 < len {
            if bytes[i + 1].is_ascii_hexdigit() && bytes[i + 2].is_ascii_hexdigit() {
                result.push(bytes[i]);
                result.push(bytes[i + 1]);
                result.push(bytes[i + 2]);
                i += 3;
                continue;
            }
        }

        if should_encode {
            result.push(b'%');
            result.push(DIGITS[(byte >> 4) as usize]);
            result.push(DIGITS[(byte & 0xF) as usize]);
        } else {
            result.push(byte);
        }

        i += 1;
    }

    // performance note:
    // all characters are in ASCII range because everything >= 0x80 is percent-encoded,
    // so we can use from_utf8_unchecked, but it doesn't improve speed by much
    String::from_utf8(result).unwrap()
}


#[cfg(test)]
mod tests {
    use super::encode;
    use super::AsciiSet;
    const SET : AsciiSet = AsciiSet::from(";/?:@&=+$,-_.!~*'()#");

    #[test]
    fn should_encode_percent() {
        assert_eq!(encode("%%%", SET, true), "%25%25%25");
    }

    #[test]
    fn should_encode_control_chars() {
        assert_eq!(encode("\r\n", SET, true), "%0D%0A");
    }

    #[test]
    fn should_not_encode_parts_of_an_url() {
        assert_eq!(encode("?#", SET, true), "?#");
    }

    #[test]
    fn should_encode_square_brackets_commonmark_tests() {
        assert_eq!(encode("[]^", SET, true), "%5B%5D%5E");
    }

    #[test]
    fn should_encode_spaces() {
        assert_eq!(encode("my url", SET, true), "my%20url");
    }

    #[test]
    fn should_encode_unicode() {
        assert_eq!(encode("φου", SET, true), "%CF%86%CE%BF%CF%85");
    }

    #[test]
    fn should_encode_percent_if_it_doesnt_start_a_valid_escape_seq() {
        assert_eq!(encode("%FG", SET, true), "%25FG");
    }

    #[test]
    fn should_preserve_non_utf8_encoded_characters() {
        assert_eq!(encode("%00%FF", SET, true), "%00%FF");
    }

    #[test]
    fn should_encode_characters_on_the_cache_borders() {
        // protects against off-by-one in cache implementation
        assert_eq!(encode("\x00\x7F\u{80}", SET, true), "%00%7F%C2%80");
    }

    #[test]
    fn arguments_encode_string_unescapedset() {
        assert_eq!(encode("!@#$", AsciiSet::from("@$"), true), "%21@%23$");
    }

    #[test]
    fn arguments_keepescaped_true() {
        assert_eq!(encode("%20%2G", SET, true), "%20%252G");
    }

    #[test]
    fn arguments_keepescaped_false() {
        assert_eq!(encode("%20%2G", SET, false), "%2520%252G");
    }
}
