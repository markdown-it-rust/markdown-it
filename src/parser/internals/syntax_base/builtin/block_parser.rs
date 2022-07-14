use crate::Node;
use crate::parser::MarkdownIt;
use crate::parser::internals::env::Env;
use super::Root;

pub fn add(md: &mut MarkdownIt) {
    md.ruler.add("builtin::block_parser", rule)
        .before_all();
}

pub fn rule(node: &mut Node, md: &MarkdownIt) {
    let mut root = std::mem::take(node);
    let mut env = root.env.remove::<Env>().unwrap_or_default();
    let data = root.cast_mut::<Root>().expect("expecting root node to always be Root");
    let source = std::mem::take(&mut data.content);

    root = md.block.parse(source.as_str(), root, md, &mut env);
    root.env.insert(env);
    root.cast_mut::<Root>().unwrap().content = source;
    *node = root;
}
