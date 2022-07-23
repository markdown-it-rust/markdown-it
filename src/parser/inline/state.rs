// Inline parser state
//
use crate::{MarkdownIt, Node};
use crate::common::ErasedSet;
use crate::common::sourcemap::SourcePos;
use crate::parser::inline::Text;

#[derive(Debug, Clone, Copy)]
/// Information about emphasis delimiter run returned from [InlineState::scan_delims].
pub struct DelimiterRun {
    /// Starting marker character.
    pub marker: char,

    /// Boolean flag that determines if this delimiter could open an emphasis.
    pub can_open: bool,

    /// Boolean flag that determines if this delimiter could open an emphasis.
    pub can_close: bool,

    /// Total length of scanned delimiters.
    pub length: usize,
}

fn is_punct_char(ch: char) -> bool {
    use unicode_general_category::get_general_category;
    use unicode_general_category::GeneralCategory::*;

    match get_general_category(ch) {
        // P
        ConnectorPunctuation | DashPunctuation | OpenPunctuation | ClosePunctuation |
        InitialPunctuation | FinalPunctuation | OtherPunctuation => true,

        // L
        UppercaseLetter | LowercaseLetter | TitlecaseLetter | ModifierLetter | OtherLetter |
        // M
        NonspacingMark | SpacingMark | EnclosingMark |
        // N
        DecimalNumber | LetterNumber | OtherNumber |
        // S
        MathSymbol | CurrencySymbol | ModifierSymbol | OtherSymbol |
        // Z
        SpaceSeparator | LineSeparator | ParagraphSeparator |
        // C
        Control | Format | Surrogate | PrivateUse | Unassigned => false
    }
}

#[derive(Debug)]
#[readonly::make]
/// Sandbox object containing data required to parse inline structures.
pub struct InlineState<'a, 'b> where 'b: 'a {
    /// Markdown source.
    #[readonly]
    pub src: String,

    /// Link to parser instance.
    #[readonly]
    pub md: &'a MarkdownIt,

    /// Current node, your rule is supposed to add children to it.
    pub node: Node,

    /// For each line, it holds offset of the start of the line in original
    /// markdown source and offset of the start of the line in `src`.
    pub srcmap: Vec<(usize, usize)>,
    pub root_env: &'b mut ErasedSet,
    pub inline_env: ErasedSet,

    /// Current byte offset in `src`, it must respect char boundaries.
    pub pos: usize,

    /// Maximum allowed byte offset in `src`, it must respect char boundaries.
    pub pos_max: usize,

    /// Counter used to disable inline linkifier execution
    /// inside raw html and markdown links.
    pub link_level: i32,

    /// Counter used to prevent recursion by image and link rules.
    pub level: u32,
}

impl<'a, 'b> InlineState<'a, 'b> {
    pub fn new(
        src: String,
        srcmap: Vec<(usize, usize)>,
        md: &'a MarkdownIt,
        env: &'b mut ErasedSet,
        node: Node,
    ) -> Self {
        let mut result = Self {
            pos:        0,
            pos_max:    src.len(),
            src,
            srcmap,
            root_env:   env,
            inline_env: ErasedSet::new(),
            md,
            node,
            link_level: 0,
            level:      0,
        };

        result.trim_src();
        result
    }

    fn trim_src(&mut self) {
        let mut chars = self.src.as_bytes().iter();
        while let Some(b' ' | b'\t') = chars.next_back() {
            self.pos_max -= 1;
        }
        while let Some(b' ' | b'\t') = chars.next() {
            self.pos += 1;
        }
    }

    pub fn trailing_text_push(&mut self, start: usize, end: usize) {
        if let Some(text) = self.node.children.last_mut()
                                       .and_then(|t| t.cast_mut::<Text>()) {
            text.content.push_str(&self.src[start..end]);

            if let Some(map) = self.node.children.last_mut().unwrap().srcmap {
                let (map_start, _) = map.get_byte_offsets();
                let map_end = self.get_source_pos_for(end);
                self.node.children.last_mut().unwrap().srcmap = Some(SourcePos::new(map_start, map_end));
            }
        } else {
            let mut node = Node::new(Text { content: self.src[start..end].to_owned() });
            node.srcmap = self.get_map(start, end);
            self.node.children.push(node);
        }
    }

    pub fn trailing_text_pop(&mut self, count: usize) {
        if count == 0 { return; }

        let mut node = self.node.children.pop().unwrap();
        let text = node.cast_mut::<Text>().unwrap();
        if text.content.len() == count {
            // do nothing, just remove the node
            drop(node);
        } else {
            // modify the token and reinsert it later
            text.content.truncate(text.content.len() - count);
            if let Some(map) = node.srcmap {
                let (map_start, map_end) = map.get_byte_offsets();
                let map_end = self.get_source_pos_for(map_end - count);
                node.srcmap = Some(SourcePos::new(map_start, map_end));
            }
            self.node.children.push(node);
        }
    }

    pub fn trailing_text_get(&self) -> &str {
        if let Some(text) = self.node.children.last()
                                .and_then(|t| t.cast::<Text>()) {
            text.content.as_str()
        } else {
            ""
        }
    }

    // Scan a sequence of emphasis-like markers, and determine whether
    // it can start an emphasis sequence or end an emphasis sequence.
    //
    //  - start - position to scan from (it should point at a valid marker);
    //  - can_split_word - determine if these markers can be found inside a word
    //
    pub fn scan_delims(&self, start: usize, can_split_word: bool) -> DelimiterRun {
        let mut left_flanking = true;
        let mut right_flanking = true;

        let last_char = if start > 0 {
            self.src[..start].chars().next_back().unwrap()
        } else {
            // treat beginning of the line as a whitespace
            ' '
        };

        let mut chars = self.src[start..self.pos_max].chars();
        let marker = chars.next().unwrap();
        let next_char;
        let mut count = 1;

        loop {
            match chars.next() {
                None => {
                    next_char = ' ';
                    break;
                }
                Some(x) => {
                    if x != marker {
                        // treat end of the line as a whitespace
                        next_char = x;
                        break;
                    }
                }
            }
            count += 1;
        }

        let is_last_punct_char = last_char.is_ascii_punctuation() || is_punct_char(last_char);
        let is_next_punct_char = next_char.is_ascii_punctuation() || is_punct_char(next_char);

        let is_last_whitespace = last_char.is_whitespace();
        let is_next_whitespace = next_char.is_whitespace();

        #[allow(clippy::collapsible_if)]
        if is_next_whitespace {
            left_flanking = false;
        } else if is_next_punct_char {
            if !(is_last_whitespace || is_last_punct_char) {
                left_flanking = false;
            }
        }

        #[allow(clippy::collapsible_if)]
        if is_last_whitespace {
            right_flanking = false;
        } else if is_last_punct_char {
            if !(is_next_whitespace || is_next_punct_char) {
                right_flanking = false;
            }
        }

        let can_open;
        let can_close;

        if !can_split_word {
            can_open  = left_flanking  && (!right_flanking || is_last_punct_char);
            can_close = right_flanking && (!left_flanking  || is_next_punct_char);
        } else {
            can_open  = left_flanking;
            can_close = right_flanking;
        }

        DelimiterRun {
            marker,
            can_open,
            can_close,
            length: count
        }
    }

    #[must_use]
    fn get_source_pos_for(&self, pos: usize) -> usize {
        let line = match self.srcmap.binary_search_by(|x| x.0.cmp(&pos)) {
            Ok(x) => x,
            Err(x) => x - 1,
        };
        self.srcmap[line].1 + (pos - self.srcmap[line].0)
    }

    #[must_use]
    pub fn get_map(&self, start_pos: usize, end_pos: usize) -> Option<SourcePos> {
        debug_assert!(start_pos <= end_pos);

        Some(SourcePos::new(
            self.get_source_pos_for(start_pos),
            self.get_source_pos_for(end_pos)
        ))
    }
}
