use indoc::indoc;
use markdown_it::{
    parser::{core::CoreRule, inline::Text},
    plugins::{
        cmark::{
            block::{code::CodeBlock, fence::CodeFence, heading::ATXHeading, paragraph::Paragraph},
            inline::{
                backticks::CodeInline,
                emphasis::{Em, Strong},
                link::Link,
            },
        },
        extra::strikethrough::Strikethrough,
    },
    MarkdownIt, Node, NodeValue, Renderer,
};
use slug::slugify;

#[derive(Debug)]
struct HeadingWithSlug {
    level: u8,
    slug: String,
}

impl NodeValue for HeadingWithSlug {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        let tag = &format!("h{}", self.level);

        let attrs = &[("id", self.slug.clone())];

        fmt.cr();
        fmt.open(tag, attrs);
        fmt.contents(&node.children);
        fmt.close(tag);
        fmt.cr();
    }
}

struct HeadingsWithIds;

impl CoreRule for HeadingsWithIds {
    fn run(node: &mut Node, _: &MarkdownIt) {
        node.walk_post_mut(|node, _| {
            if let Some(heading) = node.cast::<ATXHeading>() {
                let slug = slugify(node_to_string(node));

                node.replace(HeadingWithSlug {
                    level: heading.level,
                    slug,
                });
            }
        });
    }
}

fn node_to_string(node: &Node) -> String {
    let mut pieces: Vec<String> = Vec::new();

    for node in node.children.iter() {
        if let Some(txt) = node.cast::<Text>() {
            pieces.push(txt.content.clone());
        } else if node.is::<Paragraph>() {
            pieces.push(format!(" {} ", node_to_string(node)));
        } else if let Some(code) = node.cast::<CodeFence>() {
            pieces.push(code.content.clone());
        } else if let Some(code) = node.cast::<CodeBlock>() {
            pieces.push(code.content.clone());
        } else if node.is::<CodeInline>()
            || node.is::<Link>()
            || node.is::<Strong>()
            || node.is::<Em>()
            || node.is::<Strikethrough>()
        {
            pieces.push(node_to_string(node));
        }
    }

    pieces.join("").trim().to_owned()
}

fn main() {
    let md = &mut MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    md.add_rule::<HeadingsWithIds>();

    let text = indoc! { r#"
        Welcome to this page!

        # This heading is very important

        But the next one is also pretty important.

        ## This one has `code` in it

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

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use markdown_it::MarkdownIt;

    use crate::HeadingsWithIds;

    #[test]
    fn headings_with_ids() {
        let md = &mut MarkdownIt::new();
        markdown_it::plugins::cmark::add(md);
        md.add_rule::<HeadingsWithIds>();

        let cases: Vec<(&str, &str)> = vec![(
            indoc! { "
                Welcome to this page!

                # This heading is very important

                But the next one is also pretty important.

                ## This one has `code` in it

                ## Yep, second level ain't bad

                Now they start to get a bit less important.

                ### And now for the third tier

                Starting to get a little sad.

                #### Le sigh

                Welp.
            " },
            indoc! {r#"
                <p>Welcome to this page!</p>
                <h1 id="this-heading-is-very-important">This heading is very important</h1>
                <p>But the next one is also pretty important.</p>
                <h2 id="this-one-has-code-in-it">This one has <code>code</code> in it</h2>
                <h2 id="yep-second-level-ain-t-bad">Yep, second level ain't bad</h2>
                <p>Now they start to get a bit less important.</p>
                <h3 id="and-now-for-the-third-tier">And now for the third tier</h3>
                <p>Starting to get a little sad.</p>
                <h4 id="le-sigh">Le sigh</h4>
                <p>Welp.</p>
            "#},
        )];

        for (input, expected_html) in cases {
            let html = md.parse(input).render();
            assert_eq!(expected_html, &html);
        }
    }
}
