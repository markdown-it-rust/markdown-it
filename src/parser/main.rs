use derivative::Derivative;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::Node;
use crate::common::{ErasedSet, TypeKey};
use crate::common::mdurl::{self, AsciiSet};
use crate::common::ruler::Ruler;
use crate::common::sourcemap::SourcePos;
use crate::parser::block::{self, BlockParser};
use crate::parser::inline::{self, InlineParser};
use crate::parser::core::Root;
use crate::parser::core::*;

type RuleFn = fn (&mut Node, &MarkdownIt);

#[derive(Derivative)]
#[derivative(Debug)]
pub struct MarkdownIt {
    pub block: BlockParser,
    pub inline: InlineParser,
    #[derivative(Debug="ignore")]
    pub validate_link: fn (&str) -> bool,
    #[derivative(Debug="ignore")]
    pub normalize_link: fn (&str) -> String,
    #[derivative(Debug="ignore")]
    pub normalize_link_text: fn (&str) -> String,
    pub env: ErasedSet,
    pub max_nesting: u32,
    ruler: Ruler<TypeKey, RuleFn>,
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

impl MarkdownIt {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&self, src: &str) -> Node {
        let mut node = Node::new(Root {
            content: src.to_owned(),
            env: ErasedSet::new(),
        });
        node.srcmap = Some(SourcePos::new(0, src.len()));

        for rule in self.ruler.iter() {
            rule(&mut node, self);
        }
        node
    }

    pub fn add_rule<T: CoreRule>(&mut self) -> RuleBuilder<RuleFn> {
        let item = self.ruler.add(TypeKey::of::<T>(), T::run);
        RuleBuilder::new(item)
    }

    pub fn has_rule<T: CoreRule>(&mut self) -> bool {
        self.ruler.contains(TypeKey::of::<T>())
    }

    pub fn remove_rule<T: CoreRule>(&mut self) {
        self.ruler.remove(TypeKey::of::<T>());
    }
}

impl Default for MarkdownIt {
    fn default() -> Self {
        let mut md = Self {
            block: BlockParser::new(),
            inline: InlineParser::new(),
            validate_link,
            normalize_link,
            normalize_link_text,
            env: ErasedSet::new(),
            max_nesting: 100,
            ruler: Ruler::new(),
        };
        block::builtin::add(&mut md);
        inline::builtin::add(&mut md);
        md
    }
}
