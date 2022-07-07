use markdown_it;

fn run(input: &str, output: &str) {
    let output = if output == "" { "".to_owned() } else { output.to_owned() + "\n" };
    let md = &mut markdown_it::MarkdownIt::new(Some(markdown_it::Options {
        max_nesting: None,
    }));
    markdown_it::syntax::cmark::add(md);
    markdown_it::syntax::html::add(md);
    let tokens = md.parse(&(input.to_owned() + "\n"));
    let result = markdown_it::renderer::xhtml(&tokens);
    assert_eq!(result, output);
}

mod markdown_it_rs_extras {
    use super::run;

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
