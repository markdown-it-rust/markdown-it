use indoc::indoc;
use markdown_it::{
    parser::core::CoreRule, plugins::cmark::block::heading::ATXHeading, MarkdownIt, Node,
};

fn main() {
    let md = &mut MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    md.add_rule::<CustomHeadings>();

    let text = indoc! { r#"
        Welcome to this page!

        # This heading is very important

        But the next one is also pretty important.

        ## Yep, second level ain't bad

        Now they start to get a bit less important.

        ### And now for the third tier

        Starting to get a little sad.

        #### Le sigh

        Welp.
    "# };
    let html = md.parse(text).render();
    println!("{}", html);
}

struct CustomHeading;

struct CustomHeadings;

impl CoreRule for CustomHeadings {
    fn run(node: &mut Node, _: &MarkdownIt) {
        node.walk_post_mut(|node, _| {
            if let Some(heading) = node.cast::<ATXHeading>() {
                println!("{:?}", heading);
            }
        });
    }
}
