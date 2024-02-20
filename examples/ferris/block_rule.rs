// Replaces `(\/)-------(\/)` with a nice picture.

use markdown_it::parser::block::{BlockRule, BlockState};
use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};

const CRAB_CLAW : &str = r#"(\/)"#;
const CRAB_URL  : &str = "https://upload.wikimedia.org/wikipedia/commons/0/0f/Original_Ferris.svg";

#[derive(Debug)]
// This is a structure that represents your custom Node in AST.
pub struct BlockFerris;

impl NodeValue for BlockFerris {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        // build attributes for `div`
        let mut attrs_div = node.attrs.clone();
        attrs_div.push(("class".into(), "ferris-block".into()));

        // build attributes for `img`
        let attrs_img = vec![("src".into(), CRAB_URL.into())];

        fmt.cr(); // linebreak, multiples get merged
        fmt.open("div", &attrs_div); // opening tag, `<div>`
        fmt.self_close("img", &attrs_img); // `<img>`
        fmt.close("div"); // closing tag, `</div>`
        fmt.cr();
    }
}

// This is an extension for the block subparser.
struct FerrisBlockScanner;

impl BlockRule for FerrisBlockScanner {
    // This is a custom function that will be invoked on every line
    // in a block context.
    //
    // It should get a line number `state.line` and report if your
    // custom structure appears there.
    //
    // If custom structure is found, it:
    //  - creates a new `Node` in AST
    //  - increments `state.line` to a position after this node
    //  - returns true
    //
    // In "silent mode" (when `silent=true`) you aren't allowed to
    // create any nodes, should only increment `state.line`.
    //
    fn run(state: &mut BlockState) -> Option<(Node, usize)> {
        // get contents of a line number `state.line` and check it
        let line = state.get_line(state.line).trim();
        if !line.starts_with(CRAB_CLAW) { return None; }
        if !line.ends_with(CRAB_CLAW)   { return None; }

        // require any number of `-` in between, but no less than 4
        if line.len() < CRAB_CLAW.len() * 2 + 4 { return None; }

        // and make sure no other characters are present there
        let dashes = &line[CRAB_CLAW.len()..line.len()-CRAB_CLAW.len()];
        if dashes.chars().any(|c| c != '-') { return None; }

        // return new node and number of lines it occupies
        Some((
            Node::new(BlockFerris),
            1,
        ))
    }
}

pub fn add(md: &mut MarkdownIt) {
    // insert this rule into block subparser
    md.block.add_rule::<FerrisBlockScanner>();
}
