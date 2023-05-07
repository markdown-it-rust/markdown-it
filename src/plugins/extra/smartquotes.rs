//! Replaces `"` and `'` quotes with "nicer" ones like `‘`, `’`, `“`, `”`, or
//! with `’` for words like "isn't".
//!
//! This currently only supports single character quotes, which is a limitation
//! of the Rust implementation due to the use of `const` generics.
//!
//! ## Implementation notes
//!
//! The main obstacle to implementing this was the fact that the document is
//! necessarily represented as a tree of nodes.
//! Each node is thus necessarily referenced by its parents, which means that an
//! any given moment we cannot hold a mutable reference to a node if any other
//! part of the code holds a reference to the document. At least that's my
//! understanding of the problem.
//! The smartquotes algorithm from the JS library makes heavy use of iteration
//! backwards and forwards through a flat list of tokens. This isn't really
//! possible in the Rust implementation. Building a flat representation of all
//! `Node` objects is easy, but holding that list precludes us from executing a
//! `root.walk_mut` call at the same time.
//! On top of that, while the smartquotes algorithm iterates linearly over all
//! nodes/tokens, looking at a specific token with index `j` can trigger
//! replacements in any of the tokens with `0` to `j - 1`.
//!
//! The solution proposed here is to first compute all the replacement
//! operations on a read-only flat view of the document, and _then_ to perform
//! all replacements in a single call to `root.walk_mut`.
use std::collections::HashMap;

use crate::common::utils::is_punct_char;
use crate::parser::core::CoreRule;
use crate::parser::inline::Text;
use crate::plugins::cmark::block::paragraph::Paragraph;
use crate::plugins::cmark::inline::newline::{Hardbreak, Softbreak};
use crate::plugins::html::html_inline::HtmlInline;
use crate::{MarkdownIt, Node};

const APOSTROPHE: char = '\u{2019}';
const SINGLE_QUOTE: char = '\'';
const DOUBLE_QUOTE: char = '"';
const SPACE: char = ' ';

/// Add smartquotes with the "classic" quote set of `‘`, `’`, `“`, and `”`.
pub fn add(md: &mut MarkdownIt) {
    add_with::<'‘', '’', '“', '”'>(md);
}

pub fn add_with<
    const OPEN_SINGLE_QUOTE: char,
    const CLOSE_SINGLE_QUOTE: char,
    const OPEN_DOUBLE_QUOTE: char,
    const CLOSE_DOUBLE_QUOTE: char,
>(
    md: &mut MarkdownIt,
) {
    md.add_rule::<SmartQuotesRule<
        OPEN_SINGLE_QUOTE,
        CLOSE_SINGLE_QUOTE,
        OPEN_DOUBLE_QUOTE,
        CLOSE_DOUBLE_QUOTE>>();
}

/// Simplified Node type that only holds the info we need
///
/// To replace quotes, we'll be iterating forward and backward over the nodes in
/// our document tree. The `Node` class doesn't provide a mechanism to do this
/// efficiently, and in any case we only care about certain parts of the
/// information. This struct will be used to build a flat view of the document;
/// the `Irrelevant` variant serves as a "filler" so that the indexes of the
/// entries line up correctly with the order we see during tree traversal.
enum FlatToken<'a> {
    LineBreak,
    Text {
        content: &'a str,
        nesting_level: u32,
    },
    Irrelevant,
}

/// A simple enum to distinguish single and double quotes
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum QuoteType {
    Single,
    Double,
}

/// Holds information about quotes we have encountered thus far.
///
/// These quotes may or may not be used to close a pair further down the line.
/// The different fields thus hold all the information we need to a) decide
/// whether or not to match them up with anoter quote we encounter, and b) to
/// perform the correct replacement, should be indeed use this quote to close a
/// pair.
struct QuoteMarker {
    /// The iteration index of the node in which this quote was found.
    ///
    /// This is the index at which this quote's `Node` appears in a pre-order
    /// depth-first walk of the document tree. Since we can only _modify_ nodes
    /// during a walk, we rely on this index to tell us which nodes to modify.
    walk_index: usize,
    /// The position of the quote within node's `content`
    quote_position: usize,
    /// Whether this is a single or a double quote
    quote_type: QuoteType,
    /// Nesting level of the containing token
    ///
    /// This is the nesting of the containing `Node` within the document tree.
    /// It is used to decide which quotes can be matched up.
    level: u32,
}

/// Description of a single quote replacement to be executed
///
/// As described above, we have to compute the replacements in a first step that
/// treats the entire document tree read-only. Only then can we perform the
/// actual replacements. This `struct` holds the information we need to perform
/// the replacement of a single quote character during a `walk_mut`.
struct ReplacementOp {
    walk_index: usize,
    quote_position: usize,
    quote: char,
}

pub struct SmartQuotesRule<
    const OPEN_SINGLE_QUOTE: char,
    const CLOSE_SINGLE_QUOTE: char,
    const OPEN_DOUBLE_QUOTE: char,
    const CLOSE_DOUBLE_QUOTE: char,
>;

impl<
        const OPEN_SINGLE_QUOTE: char,
        const CLOSE_SINGLE_QUOTE: char,
        const OPEN_DOUBLE_QUOTE: char,
        const CLOSE_DOUBLE_QUOTE: char,
    > CoreRule
    for SmartQuotesRule<
        OPEN_SINGLE_QUOTE,
        CLOSE_SINGLE_QUOTE,
        OPEN_DOUBLE_QUOTE,
        CLOSE_DOUBLE_QUOTE,
    >
{
    fn run(root: &mut Node, _: &MarkdownIt) {
        let text_tokens = all_text_tokens(root);

        let replacement_ops = Self::compute_replacements(text_tokens);

        // now that we know what we want to replace where, we go over the nodes a _third_ time to do all the actual replacements.
        let mut current_index: usize = 0;
        root.walk_mut(|node, _| {
            if let Some(current_replacements) = replacement_ops.get(&current_index) {
                let mut text_node = node.cast_mut::<Text>().expect("Expected to find a text node at this index because we constructed our replacements HashMap accordingly.");
                text_node.content = execute_replacements(current_replacements, &text_node.content);
            };
            current_index += 1;
        });
    }
}

impl<
        const OPEN_SINGLE_QUOTE: char,
        const CLOSE_SINGLE_QUOTE: char,
        const OPEN_DOUBLE_QUOTE: char,
        const CLOSE_DOUBLE_QUOTE: char,
    >
    SmartQuotesRule<OPEN_SINGLE_QUOTE, CLOSE_SINGLE_QUOTE, OPEN_DOUBLE_QUOTE, CLOSE_DOUBLE_QUOTE>
{
    /// Walk the list of tokens to figure out what needs replacing where. to do
    /// this, we need to search back and forth over the nodes to find matching
    /// quotes across nodes. The borrow checker won't let us handle the entire
    /// set of nodes as mutable at the same time however, so all we do here is
    /// figure out what we _want_ to replace in which node.
    fn compute_replacements(text_tokens: Vec<FlatToken>) -> HashMap<usize, HashMap<usize, char>> {
        let mut quote_stack: Vec<QuoteMarker> = Vec::new();
        let mut replacement_ops: HashMap<usize, HashMap<usize, char>> = HashMap::new();
        for (walk_index, token) in text_tokens.iter().enumerate() {
            if let FlatToken::Text {
                content,
                nesting_level,
            } = token
            {
                for op in Self::replace_smartquotes(
                    content,
                    walk_index,
                    *nesting_level,
                    &text_tokens,
                    &mut quote_stack,
                ) {
                    replacement_ops
                        .entry(op.walk_index)
                        .or_default()
                        .insert(op.quote_position, op.quote);
                }
            }
        }
        replacement_ops
    }

    /// Compute quote replacements found by looking at a single text block
    fn replace_smartquotes(
        content: &str,
        walk_index: usize,
        level: u32,
        text_tokens: &[FlatToken],
        quote_stack: &mut Vec<QuoteMarker>,
    ) -> Vec<ReplacementOp> {
        truncate_stack(quote_stack, level);

        let mut result: Vec<_> = Vec::new();
        for (quote_position, quote_type) in find_quotes(content) {
            let last_char = find_last_char_before(text_tokens, walk_index, quote_position);
            let next_char = find_first_char_after(text_tokens, walk_index, quote_position);

            let (can_open, can_close): (bool, bool) =
                can_open_or_close(&quote_type, last_char, next_char);

            if !can_open && !can_close {
                // if this is a single quote then we're in the middle of a word and
                // assume it to be an apostrophe
                if quote_type == QuoteType::Single {
                    result.push(ReplacementOp {
                        walk_index,
                        quote_position,
                        quote: APOSTROPHE,
                    });
                }
                // in any case, we're done with this quote and continue searching
                // for more quotes in this text block
                continue;
            }

            if can_close {
                if let Some((opening_op, closing_op, new_stack_len)) =
                    Self::try_close(quote_stack, walk_index, level, quote_type, quote_position)
                {
                    quote_stack.truncate(new_stack_len);
                    result.push(opening_op);
                    result.push(closing_op);
                    continue;
                }
            }

            if can_open {
                quote_stack.push(QuoteMarker {
                    walk_index,
                    quote_position,
                    quote_type,
                    level,
                });
            } else if can_close && quote_type == QuoteType::Single {
                result.push(ReplacementOp {
                    walk_index,
                    quote_position,
                    quote: APOSTROPHE,
                });
            }
        }
        result
    }

    /// Try to find a matching opening quote to the given one.
    ///
    /// If a match is found, returns `Some` with two `ReplacementOp`s to be
    /// added to the result, and with the resulting length of the `quote_stack`.
    fn try_close(
        quote_stack: &[QuoteMarker],
        walk_index: usize,
        level: u32,
        quote_type: QuoteType,
        quote_position: usize,
    ) -> Option<(ReplacementOp, ReplacementOp, usize)> {
        for (j, other_item) in quote_stack.iter().enumerate().rev() {
            if other_item.level < level {
                return None;
            }
            if other_item.quote_type == quote_type && other_item.level == level {
                return Some((
                    ReplacementOp {
                        walk_index: other_item.walk_index,
                        quote_position: other_item.quote_position,
                        quote: if quote_type == QuoteType::Single {
                            OPEN_SINGLE_QUOTE
                        } else {
                            OPEN_DOUBLE_QUOTE
                        },
                    },
                    ReplacementOp {
                        walk_index,
                        quote_position,
                        quote: if quote_type == QuoteType::Single {
                            CLOSE_SINGLE_QUOTE
                        } else {
                            CLOSE_DOUBLE_QUOTE
                        },
                    },
                    j,
                ));
            }
        }
        None
    }
}

/// Produces a simplified flat list of all tokens, with the necessary
/// information to do smart quote replacement.
///
/// This handles inline html and inline code like JS version seems to do.
/// This list is a work-around for the fact that we can't build a flat list of
/// all nodes for iteration back and forth, and at the same time do a mutable
/// walk on the document tree.
///
/// Returns a `Vec<FlatToken<'a>>` where `<'a>` is the same lifetime as `root`.
/// This simply reflects the fact that the `content: &str` entries of the
/// `FlatToken` structs reference the same memory as `root`'s children.
/// Every entry in the `Vec` will produce an entry in the result, meaning that
/// the index of a token in the resulting `Vec` will be the same as the index it
/// would get during a `root.walk` call.
fn all_text_tokens(root: &Node) -> Vec<FlatToken> {
    let mut result = Vec::new();
    let mut walk_index = 0;
    root.walk(|node, nesting_level| {
        if let Some(text_node) = node.cast::<Text>() {
            result.push(FlatToken::Text {
                content: &text_node.content,
                nesting_level,
            });
        } else if let Some(html_node) = node.cast::<HtmlInline>() {
            result.push(FlatToken::Text {
                content: &html_node.content,
                nesting_level,
            });
        } else if node.is::<Paragraph>() || node.is::<Hardbreak>() || node.is::<Softbreak>() {
            result.push(FlatToken::LineBreak);
        } else {
            result.push(FlatToken::Irrelevant);
        }
        walk_index += 1;
    });
    result
}

/// Checks whether we can open or close a pair of quotes, given the quote type
/// and the type of characters before and after the quote
fn can_open_or_close(quote_type: &QuoteType, last_char: char, next_char: char) -> (bool, bool) {
    // special case: 1"" -> count first quote as an inch
    // We handle this before doing anything else to simplify the conditions
    // below.
    let is_double = *quote_type == QuoteType::Double;
    let next_is_double = next_char == DOUBLE_QUOTE;
    let last_is_digit = last_char.is_ascii_digit();
    if next_is_double && is_double && last_is_digit {
        return (false, false);
    }

    // using `is_ascii_punctuation` here matches the JS version exactly, but
    // that also means we might inherit that implementation's shortcomings
    // by ignoring unicode punctuation. `is_punct_char` however should
    // compensate for this.
    let is_last_punctuation = last_char.is_ascii_punctuation() || is_punct_char(last_char);
    let is_next_punctuation = next_char.is_ascii_punctuation() || is_punct_char(next_char);

    // Yet again we rely on rust's built-in character handling. The definition
    // of `is_whitespace` according to the unicode proplist.txt shows that the
    // difference to the JS version.
    // https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt
    //
    // Recognized as whitespace by Rust, but not by JS:
    // 0x85, 0x28, 0x29
    let is_last_whitespace = last_char.is_whitespace();
    let is_next_whitespace = next_char.is_whitespace();

    let can_open =
        !is_next_whitespace && (!is_next_punctuation || is_last_whitespace || is_last_punctuation);
    let can_close =
        !is_last_whitespace && (!is_last_punctuation || is_next_whitespace || is_next_punctuation);

    if can_open && can_close {
        // Replace quotes in the middle of punctuation sequence, but not
        // in the middle of the words, i.e.:
        //
        // 1. foo " bar " baz - not replaced
        // 2. foo-"-bar-"-baz - replaced
        // 3. foo"bar"baz     - not replaced
        return (is_last_punctuation, is_next_punctuation);
    }

    (can_open, can_close)
}

/// Executes a set of character replacements on a string
fn execute_replacements(replacement_ops: &HashMap<usize, char>, content: &str) -> String {
    content
        .chars()
        .enumerate()
        .map(|(i, c)| *replacement_ops.get(&i).unwrap_or(&c))
        .collect()
}

/// Truncates the stack of quotes following the JS implementation.
///
/// This _might_ be simplified by removing the `rev` call and using
/// `Vec::take_while` instead, but I'm not 100% sure yet that the levels on the
/// stack are really monotonously increasing, so I'm leaving it as is for now.
fn truncate_stack(quote_stack: &mut Vec<QuoteMarker>, level: u32) {
    let stack_len = quote_stack
        .iter()
        .rev()
        .skip_while(|qm| qm.level > level)
        .count();
    quote_stack.truncate(stack_len);
}

/// Finds all single or double quotes in a string, together with their positions
///
/// This might be replaced with a regex search, but not sure that's really worth
/// it, given that we only check for two fixed characters.
fn find_quotes(content: &str) -> impl Iterator<Item = (usize, QuoteType)> + '_ {
    content.chars().enumerate().filter_map(|(p, c)| {
        if c == SINGLE_QUOTE || c == DOUBLE_QUOTE {
            Some((
                p,
                if c == SINGLE_QUOTE {
                    QuoteType::Single
                } else {
                    QuoteType::Double
                },
            ))
        } else {
            None
        }
    })
}

/// Finds the next relevant character after a given position
///
/// This is the mirror image of `find_last_char_before`.
///
/// The position given is that of a quote we found. It is identified by its
/// token/node index and the position of the quote inside that token. The full
/// sequence of the text tokens is searched forwards from that point and the
/// first character is returned.
///
/// If a line break or the end of the document is encountered during search,
/// space (0x20) is returned.
///
/// This function is a bit simpler than `find_last_char_before` because Vec
/// conveniently returns None for out-of-range indexes at the top end, while not
/// allowing to index with negative index.
fn find_first_char_after(
    text_tokens: &[FlatToken],
    token_index: usize,
    quote_position: usize,
) -> char {
    for (idx_t, text_token) in text_tokens.iter().enumerate().skip(token_index) {
        let token = match text_token {
            FlatToken::LineBreak => return SPACE,
            FlatToken::Text {
                content,
                nesting_level: _,
            } => content,
            FlatToken::Irrelevant => continue,
        };
        let start_index = if idx_t == token_index {
            quote_position + 1
        } else {
            0
        };
        if let Some(c) = token.chars().nth(start_index) {
            return c;
        }
    }
    // this will be hit if we start searching at the last position of the last
    // text token
    SPACE
}

/// Finds the last relevant character before a given position
///
/// The position given is that of a quote we found. It is identified by its
/// token/node index and the position of the quote inside that token. The full
/// sequence of the text tokens is searched backwards from that point and the
/// first character is returned.
///
/// If a line break or the beginning of the document is encountered during
/// search, space (0x20) is returned.
fn find_last_char_before(
    text_tokens: &[FlatToken],
    token_index: usize,
    quote_position: usize,
) -> char {
    for idx_t in (0..=token_index).rev() {
        let token = match &text_tokens[idx_t] {
            FlatToken::LineBreak => return SPACE,
            FlatToken::Text {
                content,
                nesting_level: _,
            } => content,
            FlatToken::Irrelevant => continue,
        };

        // this is _not_ the first index we want to look at, but rather the
        // index just _after_ that.  The reason is simply that this is `usize`
        // and we want to first check if it's possible to still subtract 1 from
        // it without panicking.
        let start_index: usize = if idx_t == token_index {
            quote_position
        } else {
            token.len()
        };
        // means we can't go any further left -> try the next token (i.e. the
        // one preceding this one)
        if start_index == 0 {
            continue;
        }
        // unwrapping is safe here, we built our index to match the length of
        // the string, or (in the case of the token containing the quote itself)
        // it should be indexing a _prefix_ of the string.
        return token.chars().nth(start_index - 1).unwrap();
    }
    // this will be hit if we find a quote in the first position of the first token
    SPACE
}
