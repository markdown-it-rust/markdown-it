use crate::Node;
use crate::parser::MarkdownIt;
use crate::parser::internals::sourcemap::CharMapping;
use crate::parser::internals::syntax_base::builtin::Root;

/// Add source mapping to resulting HTML, looks like this: `<stuff data-sourcepos="1:1-2:3">`.
///
/// ```rust
/// let md = &mut markdown_it::parser::new();
/// markdown_it::syntax::cmark::add(md);
/// markdown_it::syntax::sourcepos::add(md);
///
/// let html = md.parse("# hello").render();
/// assert_eq!(html.trim(), r#"<h1 data-sourcepos="1:1-1:7">hello</h1>"#);
/// ```
pub fn add(md: &mut MarkdownIt) {
    md.ruler.add("sourcepos", add_sourcepos);
}


fn add_sourcepos(root: &mut Node, _: &MarkdownIt) {
    let source = root.cast::<Root>().unwrap().content.as_str();
    let mapping = CharMapping::new(source);

    root.walk_mut(|node, _| {
        if let Some(map) = node.srcmap {
            let ((startline, startcol), (endline, endcol)) = map.get_positions(&mapping);
            node.attrs.push(("data-sourcepos", format!("{}:{}-{}:{}", startline, startcol, endline, endcol)))
        }
    });
}


#[cfg(test)]
mod tests {
    #[test]
    fn header_test() {
        // same as doctest, keep in sync!
        // used for code coverage and quicker rust-analyzer hints
        let md = &mut crate::parser::new();
        crate::syntax::cmark::add(md);
        crate::syntax::sourcepos::add(md);

        let html = md.parse("# hello").render();
        assert_eq!(html.trim(), r#"<h1 data-sourcepos="1:1-1:7">hello</h1>"#);
    }
}
