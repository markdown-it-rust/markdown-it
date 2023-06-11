use markdown_it::parser::block::{BlockRule, BlockState};
use markdown_it::parser::core::CoreRule;
use markdown_it::parser::inline::{InlineRule, InlineState};
use markdown_it::{MarkdownIt, Node, Result};
use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
struct MyError(&'static str);

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl Error for MyError {}

struct FallibleInlineRule;

impl InlineRule for FallibleInlineRule {
    const MARKER: char = '@';

    // This is implementation of a rule that always fails on `@` character.
    fn try_run(state: &mut InlineState) -> Result<Option<(Node, usize)>> {
        // skip other characters
        if !state.src[state.pos..].starts_with(Self::MARKER) { return Ok(None); };

        Err(MyError("AAA").into())
    }

    fn run(state: &mut InlineState) -> Option<(Node, usize)> {
        Self::try_run(state).unwrap_or_default()
    }
}

struct FallibleBlockRule;

impl BlockRule for FallibleBlockRule {
    // This is implementation of a rule that always fails on `@@@` at the start of the line.
    fn try_run(state: &mut BlockState) -> Result<Option<(Node, usize)>> {
        if !state.get_line(state.line).starts_with("@@@") { return Ok(None); };

        Err(MyError("BBB").into())
    }

    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        Self::try_run(state).unwrap_or_default()
    }
}

struct FallibleCoreRule;

impl CoreRule for FallibleCoreRule {
    fn try_run(_root: &mut Node, _md: &MarkdownIt) -> Result<()> {
        Err(MyError("CCC").into())
    }

    fn run(root: &mut Node, md: &MarkdownIt) {
        let _ = Self::try_run(root, md);
    }
}

fn main() {
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);

    md.inline.add_rule::<FallibleInlineRule>();
    md.block.add_rule::<FallibleBlockRule>();
    md.add_rule::<FallibleCoreRule>().after_all();

    // inline rule fails
    let text1 = r#"*hello @world*"#;
    let err = md.try_parse(text1).err().unwrap();
    println!("{err}");
    assert_eq!(err.source().unwrap().to_string(), "AAA");

    // block rule fails
    let text2 = r#"@@@ *hello*"#;
    let err = md.try_parse(text2).err().unwrap();
    println!("{err}");
    assert_eq!(err.source().unwrap().to_string(), "BBB");

    // core rule fails
    let text3 = r#"*hello*"#;
    let err = md.try_parse(text3).err().unwrap();
    println!("{err}");
    assert_eq!(err.source().unwrap().to_string(), "CCC");

    // If you run parse() function instead of try_parse(), failing rules will be skipped.
    // This will result in custom syntax being left as user wrote it (not parsed).
    let html = md.parse(text1).render();
    print!("{html}");
    assert_eq!(html, "<p><em>hello @world</em></p>\n");

    let html = md.parse(text2).render();
    print!("{html}");
    assert_eq!(html, "<p>@@@ <em>hello</em></p>\n");

    let html = md.parse(text3).render();
    print!("{html}");
    assert_eq!(html, "<p><em>hello</em></p>\n");
}
