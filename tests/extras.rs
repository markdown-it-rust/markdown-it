
#[test]
fn title_example() {
    let parser = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(parser);

    let ast = parser.parse("Hello **world**!");
    let html = ast.render();

    assert_eq!(html, "<p>Hello <strong>world</strong>!</p>\n");
}

#[test]
fn no_plugins() {
    let md = &mut markdown_it::MarkdownIt::new();
    let node = md.parse("hello\nworld");
    let result = node.render();
    assert_eq!(result, "hello\nworld\n");
}

/*#[test]
fn no_block_parser() {
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    md.remove_rule::<markdown_it::parser::block::builtin::BlockParserRule>();
    let node = md.parse("hello *world*");
    let result = node.render();
    assert_eq!(result, "hello <em>world</em>");
}*/

fn run(input: &str, output: &str) {
    let output = if output.is_empty() { "".to_owned() } else { output.to_owned() + "\n" };
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    markdown_it::plugins::html::add(md);
    let node = md.parse(&(input.to_owned() + "\n"));
    node.walk(|node, _| assert!(node.srcmap.is_some()));
    let result = node.render();
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
    fn regression_test_ending_code() {
        run("foo`", "<p>foo`</p>");
        run("foo```", "<p>foo```</p>");
        run("[foo`", "<p>[foo`</p>");
        run("[foo```", "<p>[foo```</p>");
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

    #[test]
    fn null_char_replacement() {
        run("&#0;", "<p>\u{FFFD}</p>");
        run("\0", "<p>\u{FFFD}</p>");
    }

    #[test]
    fn cr_only_newlines() {
        run("foo\rbar", "<p>foo\nbar</p>");
        run("    foo\r    bar", "<pre><code>foo\nbar\n</code></pre>");
    }

    #[test]
    fn cr_lf_newlines() {
        run("foo\r\nbar", "<p>foo\nbar</p>");
        run("    foo\r\n    bar", "<pre><code>foo\nbar\n</code></pre>");
    }
}

mod examples {
    include!("../examples/ferris/main.rs");

    #[test]
    fn test_examples() {
        main();
    }
}
