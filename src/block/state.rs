// Parser state class
//
use crate::common::calc_right_whitespace_with_tabstops;
use crate::env::Env;
use crate::MarkdownIt;
use crate::sourcemap::SourcePos;
use crate::token::Token;

#[derive(Debug)]
pub struct State<'a, 'b, 'c> where 'c: 'b, 'b: 'a {
    pub src: String,

    // link to parser instance
    pub md: &'a MarkdownIt,

    pub env: &'b mut Env,

    //
    // Internal state vartiables
    //
    pub tokens: &'c mut Vec<Token>,

    pub line_offsets: Vec<LineOffset>,

    // block parser variables
    pub blk_indent: usize,        // required block content indent (for example, if we are
                                  // inside a list, it would be positioned after list marker)
    pub line: usize,              // line index in src
    pub line_max: usize,          // lines count
    pub tight: bool,              // loose/tight mode for lists
    pub list_indent: Option<u32>, // indent of the current list block

    pub parent_is_list: bool,

    pub level: u32,
}

#[derive(Debug, Clone)]
pub struct LineOffset {
    // "line_start" is the actual start of the line.
    // "  >  blockquote\r\n"
    //  ^-- it will always point here (must not be modified by rules)
    pub line_start: usize,

    // "line_end" is first newline character after the line,
    // or position after string length if there aren't any newlines left.
    // "  >  blockquote\r\n"
    //                 ^-- it will point here
    pub line_end: usize,

    // "first_nonspace" is the byte offset of the first non-space character in
    // the current line.
    // "   >  blockquote\r\n"
    //        ^-- it will point here when paragraph is parsed
    //     ^----- it is initially pointed here
    // It will be modified by rules (list and blockquote), chars before it
    // must be treated as whitespaces.
    pub first_nonspace: usize,

    // "indent_nonspace" is the indent (amount of virtual spaces from start)
    // of first non-space character in the current line, taking into account
    // tab expansion.
    //
    // For example, in case of " \t foo", indent is 5 (tab ends at multiple of 4,
    // then one space after it). Only tabs and spaces are counted for it,
    // so no funny unicode business (if cmark supported unicode spaces, they'd
    // be counted as 1 each regardless of utf8 width).
    //
    // You should compare "indent_nonspace" with "state.blkindent" when determining
    // real indent after taking into account lists.
    //
    // Most block rules in commonmark are indented 0..=3, and >=4 is code block.
    // Special value of ident_nonspace=-1 is used by this library as a sign
    // that this rule can only be a paragraph continuation (used in blockquotes),
    // so you must take into account that any math can end up negative.
    pub indent_nonspace: i32,
}

impl<'a, 'b, 'c> State<'a, 'b, 'c> {
    pub fn new(src: &str, md: &'a MarkdownIt, env: &'b mut Env, out_tokens: &'c mut Vec<Token>) -> Self {
        let mut result = Self {
            src: src.to_owned(),
            md,
            env,
            tokens: out_tokens,
            line_offsets: Vec::new(),
            blk_indent: 0,
            line: 0,
            line_max: 0,
            tight: false,
            list_indent: None,
            parent_is_list: false,
            level: 0,
        };

        result.generate_caches();
        result
    }

    fn generate_caches(&mut self) {
        // Create caches
        // Generate markers.
        let mut chars = self.src.chars().peekable();
        let mut indent_found = false;
        let mut indent = 0;
        let mut offset = 0;
        let mut start = 0;
        let mut pos = 0;
        let len = self.src.len();

        loop {
            match chars.next() {
                Some(ch @ (' ' | '\t')) if !indent_found => {
                    indent += 1;
                    offset += if ch == '\t' { 4 - offset % 4 } else { 1 };
                    pos += 1;
                }
                ch @ (Some('\n') | None) => {
                    self.line_offsets.push(LineOffset {
                        line_start: start,
                        line_end: pos,
                        first_nonspace: start + indent,
                        indent_nonspace: offset,
                    });

                    indent_found = false;
                    indent = 0;
                    offset = 0;
                    start = pos + 1;
                    pos += 1;

                    if ch.is_none() || chars.peek().is_none() {
                        break;
                    }
                }
                Some(ch) => {
                    indent_found = true;
                    pos += ch.len_utf8();
                }
            }
        }

        self.line_max = self.line_offsets.len();

        // Push fake entry to simplify cache bounds checks
        self.line_offsets.push(LineOffset {
            line_start: len,
            line_end: len,
            first_nonspace: len,
            indent_nonspace: 0,
        });
    }

    // Push new token to "stream".
    //
    pub fn push(&mut self, mut token: Token) {
        token.block = true;
        self.tokens.push(token);
    }

    pub fn is_empty(&self, line: usize) -> bool {
        self.line_offsets[line].first_nonspace >= self.line_offsets[line].line_end
    }

    pub fn skip_empty_lines(&self, from: usize) -> usize {
        let mut line = from;
        while line != self.line_max && self.is_empty(line) {
            line += 1;
        }
        line
    }

    // return line indent of specific line, taking into account blockquotes and lists;
    // it may be negative if a text has less indentation than current list item
    pub fn line_indent(&self, line: usize) -> i32 {
        self.line_offsets[line].indent_nonspace - self.blk_indent as i32
    }

    // return a single line, trimming initial spaces
    pub fn get_line(&self, line: usize) -> &str {
        let pos = self.line_offsets[line].first_nonspace;
        let max = self.line_offsets[line].line_end;
        &self.src[pos..max]
    }

    // Cut a range of lines begin..end (not including end) from the source without preceding indent.
    // Returns a string (lines) plus a mapping (start of each line in result -> start of each line in source).
    pub fn get_lines(&self, begin: usize, end: usize, indent: usize, keep_last_lf: bool) -> (String, Vec<(usize, usize)>) {
        let mut line = begin;
        let mut result = String::new();
        let mut mapping = Vec::new();

        while line < end {
            let offsets = &self.line_offsets[line];

            let mut last = if line + 1 < end || keep_last_lf {
                // No need for bounds check because we have fake entry on tail.
                offsets.line_end + 1
            } else {
                offsets.line_end
            };

            if last > self.src.len() { last = self.src.len(); }

            let (num_spaces, first) = calc_right_whitespace_with_tabstops(
                &self.src[offsets.line_start..offsets.first_nonspace],
                offsets.indent_nonspace - indent as i32
            );

            mapping.push(( result.len(), first ));
            result += &" ".repeat(num_spaces as usize);
            result += &self.src[offsets.line_start+first..last];
            line += 1;
        }

        ( result, mapping )
    }

    pub fn get_map(&self, _start_line: usize, _end_line: usize) -> Option<SourcePos> {
        #[cfg(not(feature="sourcemap"))]
        return None;
        #[cfg(feature="sourcemap")]
        return Some(SourcePos::new(
            self.line_offsets[_start_line].first_nonspace,
            self.line_offsets[_end_line].line_end
        ));
    }

    pub fn get_map_from_offsets(&self, _start_pos: usize, _end_pos: usize) -> Option<SourcePos> {
        #[cfg(not(feature="sourcemap"))]
        return None;
        #[cfg(feature="sourcemap")]
        return Some(SourcePos::new(_start_pos, _end_pos));
    }
}
