
#[test]
fn title_example() {
    let parser = &mut markdown_it::parser::new();
    markdown_it::syntax::cmark::add(parser);

    let ast = parser.parse("Hello **world**!");
    let html = markdown_it::renderer::html(&ast);

    assert_eq!(html, "<p>Hello <strong>world</strong>!</p>\n");
}

fn run(input: &str, output: &str) {
    let output = if output == "" { "".to_owned() } else { output.to_owned() + "\n" };
    let md = &mut markdown_it::parser::new();
    markdown_it::syntax::cmark::add(md);
    markdown_it::syntax::html::add(md);
    let node = md.parse(&(input.to_owned() + "\n"));
    node.walk(|node, _| assert!(node.srcmap.is_some()));
    let result = markdown_it::renderer::xhtml(&node);
    assert_eq!(result, output);
}

mod markdown_it_rs_extras {
    use super::run;

    #[test]
    fn regression_test_img() {
        // ! at end of line
        run("Hello!", "<p>Hello!</p>");
    }

    #[test]
    fn regression_list_markers() {
        run("- foo\n- bar", "<ul>\n<li>foo</li>\n<li>bar</li>\n</ul>");
        run("1. foo\n1. bar", "<ol>\n<li>foo</li>\n<li>bar</li>\n</ol>");
    }

    #[test]
    fn tab_offset_in_lists() {
        run("   > -\tfoo\n   >\n   >         foo\n",
r#"<blockquote>
<ul>
<li>
<p>foo</p>
<pre><code> foo
</code></pre>
</li>
</ul>
</blockquote>"#);
    }
}
