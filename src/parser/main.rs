use derivative::Derivative;
use crate::Node;
use crate::common::TypeKey;
use crate::common::ruler::Ruler;
use crate::common::sourcemap::SourcePos;
use crate::parser::block::{self, BlockParser};
use crate::parser::inline::{self, InlineParser};
use crate::parser::extset::MarkdownItExtSet;
use crate::parser::core::Root;
use crate::parser::core::*;
use crate::parser::linkfmt::{LinkFormatter, MDLinkFormatter};

type RuleFn = fn (&mut Node, &MarkdownIt);

#[derive(Derivative)]
#[derivative(Debug)]
/// Main parser struct, created once and reused for parsing multiple documents.
pub struct MarkdownIt {
    /// Block-level tokenizer.
    pub block: BlockParser,

    /// Inline-level tokenizer.
    pub inline: InlineParser,

    /// Link valiator and formatter.
    pub link_formatter: Box<dyn LinkFormatter>,

    /// Storage for custom data used in plugins.
    pub ext: MarkdownItExtSet,

    /// Maximum depth of the generated AST, exists to prevent recursion
    /// (if markdown source reaches this depth, deeply nested structures
    /// will be parsed as plain text).
    /// TODO: doesn't work
    #[doc(hidden)]
    pub max_nesting: u32,

    ruler: Ruler<TypeKey, RuleFn>,
}

impl MarkdownIt {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(&self, src: &str) -> Node {
        let mut node = Node::new(Root::new(src.to_owned()));
        node.srcmap = Some(SourcePos::new(0, src.len()));

        for rule in self.ruler.iter() {
            rule(&mut node, self);
            debug_assert!(node.is::<Root>(), "root node of the AST must always be Root");
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
            link_formatter: Box::new(MDLinkFormatter::new()),
            ext: MarkdownItExtSet::new(),
            max_nesting: 100,
            ruler: Ruler::new(),
        };
        block::builtin::add(&mut md);
        inline::builtin::add(&mut md);
        md
    }
}
