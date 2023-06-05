use crate::parser::core::{CoreRule, Root};
use crate::parser::main::RootNodeWrongType;
use crate::{MarkdownIt, Node, Result};

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<BlockParserRule>()
        .before_all();
}

pub struct BlockParserRule;
impl CoreRule for BlockParserRule {
    fn try_run(root: &mut Node, md: &MarkdownIt) -> Result<()> {
        Self::_run::<true>(root, md)?;
        Ok(())
    }

    fn run(root: &mut Node, md: &MarkdownIt) {
        let _ = Self::_run::<false>(root, md);
    }
}

impl BlockParserRule {
    fn _run<const CAN_FAIL: bool>(root: &mut Node, md: &MarkdownIt) -> Result<()> {
        let mut node = std::mem::take(root);
        let Some(data) = node.cast_mut::<Root>() else {
            return Err(RootNodeWrongType.into());
        };
        let source = std::mem::take(&mut data.content);
        let mut ext = std::mem::take(&mut data.ext);

        md.block.compile();
        node = if CAN_FAIL {
            md.block.try_parse(source.as_str(), node, md, &mut ext)?
        } else {
            md.block.parse(source.as_str(), node, md, &mut ext)
        };
        *root = node;

        let Some(data) = root.cast_mut::<Root>() else {
            return Err(RootNodeWrongType.into());
        };
        data.content = source;
        data.ext = ext;

        Ok(())
    }
}
