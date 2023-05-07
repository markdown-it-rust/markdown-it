// Adds three custom plugins and runs them.
mod block_rule;
mod core_rule;
mod inline_rule;

fn main() {
    // create markdown parser
    let md = &mut markdown_it::MarkdownIt::new();

    // add commonmark syntax, you almost always want to do that
    markdown_it::plugins::cmark::add(md);

    // add custom three rules described above
    inline_rule::add(md);
    block_rule::add(md);
    core_rule::add(md);

    // and now you can use it
    let html = md.parse(r#"
(\/) hello world (\/)
(\/)-------------(\/)
    "#).render();

    print!("{html}");

    assert_eq!(html.trim(), r#"
<p><span class="ferris-inline">🦀</span> hello world <span class="ferris-inline">🦀</span></p>
<div class="ferris-block"><img src="https://upload.wikimedia.org/wikipedia/commons/0/0f/Original_Ferris.svg"></div>
<footer class="ferris-counter">There are 3 crabs lurking in this document.</footer>
    "#.trim());
}
