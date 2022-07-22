use crate::{MarkdownIt, Node};
use crate::common::sourcemap::SourcePos;
use crate::parser::core::CoreRule;
use crate::parser::inline::Text;
use crate::parser::inline::builtin::InlineParserRule;

pub fn add(md: &mut MarkdownIt) {
    md.add_rule::<FragmentsJoinRule>()
        .before_all()
        .after::<InlineParserRule>();
}

#[doc(hidden)]
pub struct FragmentsJoinRule;
impl CoreRule for FragmentsJoinRule {
    fn run(node: &mut Node, _: &MarkdownIt) {
        node.walk_mut(|node, _| fragments_join(node));
    }
}


// Clean up tokens after emphasis and strikethrough postprocessing:
// merge adjacent text nodes into one and re-calculate all token levels
//
// This is necessary because initially emphasis delimiter markers (*, _, ~)
// are treated as their own separate text tokens. Then emphasis rule either
// leaves them as text (needed to merge with adjacent text) or turns them
// into opening/closing tags (which messes up levels inside).
//
fn fragments_join(node: &mut Node) {
    // replace all emph markers with text tokens
    for child in node.children.iter_mut() {
        if let Some(content) = child.node_value.to_text_fragment() {
            child.replace(Text { content });
        }
    }

    // collapse adjacent text tokens
    for idx in 1..node.children.len() {
        let ( tokens1, tokens2 ) = node.children.split_at_mut(idx);

        let token1 = tokens1.last_mut().unwrap();
        if let Some(t1_data) = token1.cast_mut::<Text>() {

            let token2 = tokens2.first_mut().unwrap();
            if let Some(t2_data) = token2.cast_mut::<Text>() {
                // concat contents
                let t2_content = std::mem::take(&mut t2_data.content);
                t1_data.content += &t2_content;

                // adjust source maps
                if let Some(map1) = token1.srcmap {
                    if let Some(map2) = token2.srcmap {
                        token1.srcmap = Some(SourcePos::new(
                            map1.get_byte_offsets().0,
                            map2.get_byte_offsets().1
                        ));
                    }
                }

                node.children.swap(idx - 1, idx);
            }
        }
    }

    // remove all empty tokens
    node.children.retain(|token| {
        if let Some(data) = token.cast::<Text>() {
            !data.content.is_empty()
        } else {
            true
        }
    });
}
