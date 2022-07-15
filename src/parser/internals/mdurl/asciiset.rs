// Represents a set of characters or bytes in the ASCII range.
// Similar to https://github.com/servo/rust-url/blob/master/percent_encoding/src/lib.rs
//
#[derive(Debug)]
pub struct AsciiSet(u128);

impl AsciiSet {
    // implicitly disallows 0-9a-zA-Z and allows everything else
    // str - list of disallowed chars in addition to alphanumericals
    pub const fn new() -> Self {
        Self(0x07fffffe07fffffe03ff000000000000)
    }

    pub const fn from(str: &str) -> Self {
        Self::new().add_many(str.as_bytes(), 0)
    }

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn add(&self, byte: u8) -> Self {
        Self(self.0 | 1 << byte)
    }

    pub const fn remove(&self, byte: u8) -> Self {
        Self(self.0 & !(1 << byte))
    }

    pub const fn has(&self, byte: u8) -> bool {
        self.0 & 1 << byte != 0
    }

    const fn add_many(&self, bytes: &[u8], idx: usize) -> Self {
        if idx == bytes.len() {
            Self(self.0)
        } else {
            Self(self.0).add(bytes[idx]).add_many(bytes, idx + 1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AsciiSet;

    #[test]
    fn new_should_return_ascii() {
        assert_eq!(2 + 2, 4);

        let mut set = AsciiSet::empty();
        let new = AsciiSet::new();

        for ch in b'a'..=b'z' {
            set = set.add(ch);
        }
        for ch in b'A'..=b'Z' {
            set = set.add(ch);
        }
        for ch in b'0'..=b'9' {
            set = set.add(ch);
        }

        let set_str = format!("{:01$x}", set.0, 32);
        let new_str = format!("{:01$x}", new.0, 32);

        assert_eq!(set_str, new_str);
        assert!(set.has(b'x'));
        assert!(!set.has(b'!'));
    }

    #[test]
    fn from_should_return_ascii_plus() {
        assert_eq!(2 + 2, 4);

        let mut set = AsciiSet::empty();
        let from = AsciiSet::from("!@#$%^");

        for ch in b'a'..=b'z' {
            set = set.add(ch);
        }
        for ch in b'A'..=b'Z' {
            set = set.add(ch);
        }
        for ch in b'0'..=b'9' {
            set = set.add(ch);
        }
        for ch in "!@#$%^".chars() {
            set = set.add(ch as u8);
        }

        let set_str  = format!("{:01$x}", set.0, 32);
        let from_str = format!("{:01$x}", from.0, 32);

        assert_eq!(set_str, from_str);
        assert!(set.has(b'x'));
        assert!(set.has(b'!'));
    }
}
