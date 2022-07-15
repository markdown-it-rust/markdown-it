/// This part of API is not documented and not stable.
pub mod internals;

use std::borrow::Cow;
use derivative::Derivative;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::Node;
use crate::parser::internals::block;
use crate::parser::internals::erasedset;
use crate::parser::internals::inline;
use crate::parser::internals::mdurl::{self, AsciiSet};
use crate::parser::internals::ruler::Ruler;
use crate::parser::internals::sourcemap::SourcePos;
use crate::parser::internals::syntax_base;
use crate::parser::syntax_base::builtin::Root;

use self::internals::erasedset::ErasedSet;

pub type Rule = fn (&mut Node, &MarkdownIt);

#[derive(Derivative)]
#[derivative(Debug)]
pub struct MarkdownIt {
    pub ruler: Ruler<&'static str, Rule>,
    pub block: block::BlockParser,
    pub inline: inline::InlineParser,
    #[derivative(Debug="ignore")]
    pub validate_link: fn (&str) -> bool,
    #[derivative(Debug="ignore")]
    pub normalize_link: fn (&str) -> String,
    #[derivative(Debug="ignore")]
    pub normalize_link_text: fn (&str) -> String,
    pub env: erasedset::ErasedSet,
    pub max_nesting: u32,
}

////////////////////////////////////////////////////////////////////////////////
// This validator can prohibit more than really needed to prevent XSS. It's a
// tradeoff to keep code simple and to be secure by default.
//
// If you need different setup - override validator method as you wish. Or
// replace it with dummy function and use external sanitizer.
//
static BAD_PROTO_RE : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"(?i)^(vbscript|javascript|file|data):"#).unwrap()
);

static GOOD_DATA_RE : Lazy<Regex> = Lazy::new(||
    Regex::new(r#"(?i)^data:image/(gif|png|jpeg|webp);"#).unwrap()
);

fn validate_link(str: &str) -> bool {
    !BAD_PROTO_RE.is_match(str) || GOOD_DATA_RE.is_match(str)
}

fn normalize_link(str: &str) -> String {
    const ASCII : AsciiSet = AsciiSet::from(r#";/?:@&=+$,-_.!~*'()#"#);
    mdurl::encode(str, ASCII, true)
}

fn normalize_link_text(str: &str) -> String {
    str.to_owned()
}

fn normalize_text(src: &str) -> Cow<str> {
    if src.contains([ '\r', '\0' ]) {
        Cow::Owned(src.to_owned()
                      .replace("\r\n", "\n")
                      .replace('\r', "\n")
                      .replace('\0', "\u{FFFD}"))
    } else {
        Cow::Borrowed(src)
    }
}

impl MarkdownIt {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&self, src: &str) -> Node {
        let mut node = Node::new(Root {
            content: normalize_text(src).to_string(),
            env: ErasedSet::new(),
        });
        node.srcmap = Some(SourcePos::new(0, src.len()));

        for rule in self.ruler.iter() {
            rule(&mut node, self);
        }
        node
    }
}

impl Default for MarkdownIt {
    fn default() -> Self {
        let mut md = Self {
            ruler: Ruler::new(),
            block: block::BlockParser::new(),
            inline: inline::InlineParser::new(),
            validate_link,
            normalize_link,
            normalize_link_text,
            env: erasedset::ErasedSet::new(),
            max_nesting: 100,
        };
        syntax_base::builtin::add(&mut md);
        md
    }
}

pub fn new() -> MarkdownIt {
    MarkdownIt::default()
}
