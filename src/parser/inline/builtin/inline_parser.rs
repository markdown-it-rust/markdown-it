use crate::parser::block::builtin::BlockParserRule;
use crate::parser::core::{CoreRule, Root};
use crate::parser::extset::{InlineRootExtSet, RootExtSet};
use crate::parser::main::RootNodeWrongType;
use crate::{MarkdownIt, Node, NodeValue, Result};

#[derive(Debug)]
/// Temporary node which gets replaced with inline nodes when
/// [InlineParser](crate::parser::inline::InlineParser) is called.
pub struct InlineRoot {
    pub content: String,
    pub mapping: Vec<(usize, usize)>,
    pub ext: InlineRootExtSet,
}

impl InlineRoot {
    pub fn new(content: String, mapping: Vec<(usize, usize)>) -> Self {
        Self { content, mapping, ext: InlineRootExtSet::new() }
    }
}

// this token is supposed to be replaced by one or many actual tokens by inline rule
impl NodeValue for InlineRoot {}

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<InlineParserRule>()
        .after::<BlockParserRule>()
        .before_all();
}

pub struct InlineParserRule;
impl CoreRule for InlineParserRule {
    fn try_run(root: &mut Node, md: &MarkdownIt) -> Result<()> {
        Self::_run::<true>(root, md)?;
        Ok(())
    }

    fn run(root: &mut Node, md: &MarkdownIt) {
        let _ = Self::_run::<false>(root, md);
    }
}

impl InlineParserRule {
    fn _run<const CAN_FAIL: bool>(
        root: &mut Node,
        md: &MarkdownIt,
    ) -> Result<()> {
        fn walk_recursive<const CAN_FAIL: bool>(
            node: &mut Node,
            md: &MarkdownIt,
            root_ext: &mut RootExtSet,
        ) -> Result<()> {
            let mut idx = 0;
            while idx < node.children.len() {
                let child = &mut node.children[idx];
                if let Some(data) = child.cast_mut::<InlineRoot>() {
                    let content = std::mem::take(&mut data.content);
                    let mapping = std::mem::take(&mut data.mapping);
                    let mut inline_ext = std::mem::take(&mut data.ext);

                    let mut root = std::mem::take(child);
                    root.ext = std::mem::take(&mut node.ext);
                    root.children = Vec::new();
                    root = if CAN_FAIL {
                        md.inline.try_parse(content, mapping, root, md, root_ext, &mut inline_ext)?
                    } else {
                        md.inline.parse(content, mapping, root, md, root_ext, &mut inline_ext)
                    };

                    let len = root.children.len();
                    node.children.splice(idx..=idx, std::mem::take(&mut root.children));
                    node.ext = std::mem::take(&mut root.ext);
                    idx += len;
                } else {
                    stacker::maybe_grow(64*1024, 1024*1024, || -> Result<()> {
                        walk_recursive::<CAN_FAIL>(child, md, root_ext)?;
                        Ok(())
                    })?;
                    idx += 1;
                }
            }
            Ok(())
        }

        let Some(data) = root.cast_mut::<Root>() else {
            return Err(RootNodeWrongType.into());
        };
        let mut root_ext = std::mem::take(&mut data.ext);

        // this is invalid if input only contains reference;
        // so if user disables block parser, he must insert smth like this instead
        /*if root.children.is_empty() {
            // block parser disabled, parse as if input was one big inline block
            let data = root.cast_mut::<Root>().unwrap();
            let node = Node::new(InlineRoot {
                content: data.content.clone(),
                mapping: vec![(0, 0)],
            });
            root.children.push(node);
        }*/

        md.inline.compile();
        walk_recursive::<CAN_FAIL>(root, md, &mut root_ext)?;

        let Some(data) = root.cast_mut::<Root>() else {
            return Err(RootNodeWrongType.into());
        };
        data.ext = root_ext;
        Ok(())
    }
}
