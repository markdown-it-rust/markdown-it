// Inline parser state
//
use crate::env::Env;
use crate::MarkdownIt;
use crate::sourcemap::SourcePos;
use crate::syntax_base::builtin::Text;
use crate::token::Token;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct DelimRun {
    // Starting marker
    pub marker: char,

    // Boolean flags that determine if this delimiter could open or close
    // an emphasis.
    pub can_open: bool,
    pub can_close: bool,

    // Total length of these series of delimiters
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
pub struct State<'a, 'b, 'c> where 'c: 'b, 'b: 'a {
    pub src: String,
    pub srcmap: Vec<(usize, usize)>,
    pub env: &'b mut Env,
    pub md: &'a MarkdownIt,
    pub tokens: &'c mut Vec<Token>,

    pub pos: usize,
    pub pos_max: usize,

    // Stores { start: end } pairs. Useful for backtrack
    // optimization of pairs parse (emphasis, strikes).
    pub cache: HashMap<usize, usize>,

    // Counter used to disable inline linkify-it execution
    // inside <a> and markdown links
    pub link_level: i32,

    // Counter used to prevent recursion by image and link rules
    pub level: u32,
}

impl<'a, 'b, 'c> State<'a, 'b, 'c> {
    pub fn new(
        src: String,
        srcmap: Vec<(usize, usize)>,
        md: &'a MarkdownIt,
        env: &'b mut Env,
        out_tokens: &'c mut Vec<Token>
    ) -> Self {
        let mut result = Self {
            pos:        0,
            pos_max:    src.len(),
            src,
            srcmap,
            env,
            md,
            tokens:     out_tokens,
            cache:      HashMap::new(),
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
        if let Some(text) = self.tokens.last_mut()
                                       .and_then(|t| t.cast_mut::<Text>()) {
            text.content.push_str(&self.src[start..end]);
        } else {
            let mut token = Token::new(Text { content: self.src[start..end].to_owned() });
            token.map = self.get_map(start, end);
            self.tokens.push(token);
        }
    }

    pub fn trailing_text_pop(&mut self, count: usize) {
        if count == 0 { return; }

        let mut token = self.tokens.pop().unwrap();
        let text = token.cast_mut::<Text>().unwrap();
        if text.content.len() == count {
            // do nothing, just remove the token
            drop(token);
        } else {
            // modify the token and reinsert it later
            text.content.truncate(text.content.len() - count);
            #[cfg(feature="sourcemap")]
            if let Some(map) = token.map {
                let (start, end) = map.get_byte_offsets();
                let new_end = self.get_source_pos_for(end - count);
                token.map = Some(SourcePos::new(start, new_end));
            }
            self.tokens.push(token);
        }
    }

    pub fn trailing_text_get(&self) -> &str {
        if let Some(text) = self.tokens.last()
                                       .and_then(|t| t.cast::<Text>()) {
            text.content.as_str()
        } else {
            ""
        }
    }

    pub fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    // Scan a sequence of emphasis-like markers, and determine whether
    // it can start an emphasis sequence or end an emphasis sequence.
    //
    //  - start - position to scan from (it should point at a valid marker);
    //  - can_split_word - determine if these markers can be found inside a word
    //
    pub fn scan_delims(&self, start: usize, can_split_word: bool) -> DelimRun {
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

        if is_next_whitespace {
            left_flanking = false;
        } else if is_next_punct_char {
            if !(is_last_whitespace || is_last_punct_char) {
                left_flanking = false;
            }
        }

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

        DelimRun {
            marker,
            can_open,
            can_close,
            length: count
        }
    }

    #[cfg(feature="sourcemap")]
    fn get_source_pos_for(&self, pos: usize) -> usize {
        let line = match self.srcmap.binary_search_by(|x| x.0.cmp(&pos)) {
            Ok(x) => x,
            Err(x) => x - 1,
        };
        self.srcmap[line].1 + (pos - self.srcmap[line].0)
    }

    pub fn get_map(&self, _start_pos: usize, _end_pos: usize) -> Option<SourcePos> {
        debug_assert!(_start_pos <= _end_pos);
        #[cfg(not(feature="sourcemap"))]
        return None;
        #[cfg(feature="sourcemap")]
        return Some(SourcePos::new(
            self.get_source_pos_for(_start_pos),
            self.get_source_pos_for(_end_pos)
        ));
    }
}
