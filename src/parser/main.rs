use derivative::Derivative;
use once_cell::sync::OnceCell;
use std::error::Error;
use std::fmt::Display;

use crate::common::ruler::Ruler;
use crate::common::sourcemap::SourcePos;
use crate::common::TypeKey;
use crate::parser::block::{self, BlockParser, BlockRuleError};
use crate::parser::core::{Root, *};
use crate::parser::extset::MarkdownItExtSet;
use crate::parser::inline::{self, InlineParser, InlineRuleError};
use crate::parser::linkfmt::{LinkFormatter, MDLinkFormatter};
use crate::{Node, Result};

#[derive(Clone)]
#[doc(hidden)]
pub struct RuleStruct {
    marker: TypeKey,
    run: fn (&mut Node, &MarkdownIt),
    try_run: fn (&mut Node, &MarkdownIt) -> Result<()>,
}

// use (Vec<A>, Vec<B>, Vec<C>) instead of Vec<(A, B, C)> for cache locality,
// since only one thing will be accessed at a time, and code is hot
struct RuleStructVecs {
    marker: Vec<TypeKey>,
    run: Vec<fn (&mut Node, &MarkdownIt)>,
    try_run: Vec<fn (&mut Node, &MarkdownIt) -> Result<()>>,
}

impl RuleStructVecs {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            marker: Vec::with_capacity(capacity),
            run: Vec::with_capacity(capacity),
            try_run: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, rule: RuleStruct) {
        self.marker.push(rule.marker);
        self.run.push(rule.run);
        self.try_run.push(rule.try_run);
    }
}

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

    ruler: Ruler<TypeKey, RuleStruct>,
    #[derivative(Debug = "ignore")]
    compiled_rules: OnceCell<RuleStructVecs>,
}

impl MarkdownIt {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse input string and return syntax tree.
    ///
    /// You can convert that node to html/xhtml string by using `.render()` and
    /// `.xrender()` respectively.
    ///
    /// Note that this function cannot produce errors, it will always return
    /// correct markdown syntax tree. It makes this function suitable for parsing
    /// arbitrary user input.
    ///
    pub fn parse(&self, src: &str) -> Node {
        let rules = self.compiled_rules.get_or_init(|| self.compile());
        let mut node = Node::new(Root::new(src.to_owned()));
        node.srcmap = Some(SourcePos::new(0, src.len()));

        for rule in rules.run.iter() {
            rule(&mut node, self);
        }
        node
    }

    /// Parse input string and return syntax tree.
    ///
    /// You can convert that node to html/xhtml string by using `.render()` and
    /// `.xrender()` respectively.
    ///
    /// This function can fail if any of the plugins fail. Any error in a custom
    /// rule will be propagated into the result. It makes this function suitable
    /// for parsing documents you wrote, where added validation helps to ensure
    /// you didn't do any syntax mistakes.
    ///
    pub fn try_parse(&self, src: &str) -> Result<Node> {
        let rules = self.compiled_rules.get_or_init(|| self.compile());
        let mut node = Node::new(Root::new(src.to_owned()));
        node.srcmap = Some(SourcePos::new(0, src.len()));

        for (idx, rule) in rules.try_run.iter().enumerate() {
            rule(&mut node, self).map_err(|err| {
                if err.is::<BlockRuleError>() || err.is::<InlineRuleError>() {
                    err
                } else {
                    err.context(CoreRuleError {
                        name: rules.marker[idx],
                    })
                }
            })?;
        }
        Ok(node)
    }

    pub fn add_rule<T: CoreRule>(&mut self) -> RuleBuilder<RuleStruct> {
        self.compiled_rules = OnceCell::new();
        let item = self.ruler.add(TypeKey::of::<T>(), RuleStruct {
            marker: TypeKey::of::<T>(),
            run: T::run,
            try_run: T::try_run,
        });
        RuleBuilder::new(item)
    }

    pub fn has_rule<T: CoreRule>(&self) -> bool {
        self.ruler.contains(TypeKey::of::<T>())
    }

    pub fn remove_rule<T: CoreRule>(&mut self) {
        self.compiled_rules = OnceCell::new();
        self.ruler.remove(TypeKey::of::<T>());
    }

    fn compile(&self) -> RuleStructVecs {
        let compiled_rules = self.ruler.compile();
        let mut result = RuleStructVecs::with_capacity(compiled_rules.len());

        for rule in compiled_rules {
            result.push(rule);
        }
        result
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
            compiled_rules: OnceCell::new(),
        };
        block::builtin::add(&mut md);
        inline::builtin::add(&mut md);
        md
    }
}

// Root node should always be of the type `Root`,
// but custom rules may insert weird stuff instead.
#[derive(Debug)]
pub(crate) struct RootNodeWrongType;
impl Display for RootNodeWrongType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Root node of the AST is expected to have type `Root`, but the type was changed by one of the custom rules.")
    }
}
impl Error for RootNodeWrongType {}
