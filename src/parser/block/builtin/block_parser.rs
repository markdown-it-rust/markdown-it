use crate::{MarkdownIt, Node};
use crate::parser::core::{CoreRule, Root};

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<BlockParserRule>()
        .before_all();
}

pub struct BlockParserRule;
impl CoreRule for BlockParserRule {
    fn run(root: &mut Node, md: &MarkdownIt) {
        let mut node = std::mem::take(root);
        let data = node.cast_mut::<Root>().expect("expecting root node to always be Root");
        let source = std::mem::take(&mut data.content);
        let mut env = std::mem::take(&mut data.env);

        node = md.block.parse(source.as_str(), node, md, &mut env);
        let data = node.cast_mut::<Root>().unwrap();
        data.content = source;
        data.env = env;
        *root = node;
    }
}
