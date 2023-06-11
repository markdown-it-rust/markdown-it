//! Add source mapping to resulting HTML, looks like this: `<stuff data-sourcepos="1:1-2:3">`.
//! ```rust
//! let md = &mut markdown_it::MarkdownIt::new();
//! markdown_it::plugins::cmark::add(md);
//! markdown_it::plugins::sourcepos::add(md);
//!
//! let html = md.parse("# hello").render();
//! assert_eq!(html.trim(), r#"<h1 data-sourcepos="1:1-1:7">hello</h1>"#);
//! ```
use crate::common::sourcemap::SourceWithLineStarts;
use crate::parser::block::builtin::BlockParserRule;
use crate::parser::core::{CoreRule, Root};
use crate::parser::inline::builtin::InlineParserRule;
use crate::parser::main::RootNodeWrongType;
use crate::{MarkdownIt, Node, Result};

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<SyntaxPosRule>()
        .after::<BlockParserRule>()
        .after::<InlineParserRule>();
}

#[doc(hidden)]
pub struct SyntaxPosRule;
impl CoreRule for SyntaxPosRule {
    fn try_run(root: &mut Node, _: &MarkdownIt) -> Result<()> {
        let Some(data) = root.cast::<Root>() else {
            return Err(RootNodeWrongType.into());
        };
        let source = data.content.as_str();
        let mapping = SourceWithLineStarts::new(source);

        root.walk_mut(|node, _| {
            if let Some(map) = node.srcmap {
                let ((startline, startcol), (endline, endcol)) = map.get_positions(&mapping);
                node.attrs.push(("data-sourcepos", format!("{}:{}-{}:{}", startline, startcol, endline, endcol)));
            }
            Ok(())
        }).unwrap();
        Ok(())
    }

    fn run(root: &mut Node, md: &MarkdownIt) {
        let _ = Self::try_run(root, md);
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn header_test() {
        // same as doctest, keep in sync!
        // used for code coverage and quicker rust-analyzer hints
        let md = &mut crate::MarkdownIt::new();
        crate::plugins::cmark::add(md);
        crate::plugins::sourcepos::add(md);

        let html = md.parse("# hello").render();
        assert_eq!(html.trim(), r#"<h1 data-sourcepos="1:1-1:7">hello</h1>"#);
    }
}
