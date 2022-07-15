use crate::Node;
use crate::parser::MarkdownIt;
use super::Root;

pub fn add(md: &mut MarkdownIt) {
    md.ruler.add("builtin::block_parser", rule)
        .before_all();
}

pub fn rule(node: &mut Node, md: &MarkdownIt) {
    let mut root = std::mem::take(node);
    let data = root.cast_mut::<Root>().expect("expecting root node to always be Root");
    let source = std::mem::take(&mut data.content);
    let mut env = std::mem::take(&mut data.env);

    root = md.block.parse(source.as_str(), root, md, &mut env);
    let data = root.cast_mut::<Root>().unwrap();
    data.content = source;
    data.env = env;
    *node = root;
}
