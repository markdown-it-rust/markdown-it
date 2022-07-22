// Replaces `(\/)` with `🦀`.

use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};
use markdown_it::parser::inline::{InlineRule, InlineState};

const CRAB_CLAW : &str = r#"(\/)"#;

#[derive(Debug)]
// This is a structure that represents your custom Node in AST.
pub struct InlineFerris;

// This defines how your custom node should be rendered.
impl NodeValue for InlineFerris {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        // `node.attrs` are custom attributes added by other plugins
        // (for example, source mapping information)
        let mut attrs = node.attrs.clone();

        // add a custom class attribute
        attrs.push(("class", "ferris-inline".into()));

        fmt.open("span", &attrs);
        fmt.text("🦀");
        fmt.close("span");
    }
}

// This is an extension for the inline subparser.
struct FerrisInlineScanner;

impl InlineRule for FerrisInlineScanner {
    // This is a character that starts your custom structure
    // (other characters may get skipped over).
    const MARKER: char = '(';

    // This is a custom function that will be invoked on every character
    // in an inline context.
    //
    // It should look for `state.src` exactly at position `state.pos`
    // and report if your custom structure appears there.
    //
    // If custom structure is found, it:
    //  - creates a new `Node` in AST
    //  - returns length of it
    //
    // In "silent mode" (when `silent=true`) you aren't allowed to
    // create any nodes, should only increment `state.pos`.
    //
    fn run(state: &mut InlineState, silent: bool) -> Option<usize> {
        let input = &state.src[state.pos..state.pos_max]; // look for stuff at state.pos
        if !input.starts_with(CRAB_CLAW) { return None; } // return None if it's not found

        if !silent {
            // create a custom AST node
            let mut node = Node::new(InlineFerris);
            // set source mapping for it
            node.srcmap = state.get_map(state.pos, state.pos + CRAB_CLAW.len());
            // push this node as a last child of `state.node`
            state.node.children.push(node);
        }

        // return length of this structure
        Some(CRAB_CLAW.len())
    }
}

pub fn add(md: &mut MarkdownIt) {
    // insert this rule into inline subparser
    md.inline.add_rule::<FerrisInlineScanner>();
}
