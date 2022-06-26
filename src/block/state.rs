// Parser state class
//
use crate::env::Env;
use crate::MarkdownIt;
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

    pub b_marks: Vec<usize>,  // line begin offsets for fast jumps
    pub e_marks: Vec<usize>,  // line end offsets for fast jumps
    pub t_shift: Vec<usize>,  // offsets of the first non-space characters (tabs not expanded)
    pub s_count: Vec<i32>,    // indents for each line (tabs expanded)

    // An amount of virtual spaces (tabs expanded) between beginning
    // of each line (bMarks) and real beginning of that line.
    //
    // It exists only as a hack because blockquotes override bMarks
    // losing information in the process.
    //
    // It's used only when expanding tabs, you can think about it as
    // an initial tab length, e.g. bsCount=21 applied to string `\t123`
    // means first tab should be expanded to 4-21%4 === 3 spaces.
    //
    pub bs_count: Vec<usize>,

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

impl<'a, 'b, 'c> State<'a, 'b, 'c> {
    pub fn new(src: &str, md: &'a MarkdownIt, env: &'b mut Env, out_tokens: &'c mut Vec<Token>) -> Self {
        let mut result = Self {
            src: src.to_owned(),
            md,
            env,
            tokens: out_tokens,
            b_marks: Vec::new(),
            e_marks: Vec::new(),
            t_shift: Vec::new(),
            s_count: Vec::new(),
            bs_count: Vec::new(),
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
                    self.b_marks.push(start);
                    self.e_marks.push(pos);
                    self.t_shift.push(indent);
                    self.s_count.push(offset);
                    self.bs_count.push(0);

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

        self.line_max = self.b_marks.len();

        // Push fake entry to simplify cache bounds checks
        self.b_marks.push(len);
        self.e_marks.push(len);
        self.t_shift.push(0);
        self.s_count.push(0);
        self.bs_count.push(0);
    }

    // Push new token to "stream".
    //
    pub fn push(&mut self, name: &'static str, tag: &'static str, nesting: i8) -> &mut Token {
        let mut token = Token::new(name, tag, nesting);
        token.block = true;

        if nesting < 0 { self.level -= 1; }
        token.level = self.level;
        if nesting > 0 { self.level += 1; }

        self.tokens.push(token);
        self.tokens.last_mut().unwrap()
    }

    pub fn is_empty(&self, line: usize) -> bool {
        self.b_marks[line] + self.t_shift[line] >= self.e_marks[line]
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
        self.s_count[line] - self.blk_indent as i32
    }

    // return a single line, trimming initial spaces
    pub fn get_line(&self, line: usize) -> &str {
        let pos = self.b_marks[line] + self.t_shift[line];
        let max = self.e_marks[line];
        &self.src[pos..max]
    }

    // cut lines range from source.
    pub fn get_lines(&self, begin: usize, end: usize, indent: usize, keep_last_lf: bool) -> String {
        let mut line = begin;
        let mut result = String::new();

        while line < end {
            let mut line_indent = 0;
            let line_start = self.b_marks[line];

            let mut last = if line + 1 < end || keep_last_lf {
                // No need for bounds check because we have fake entry on tail.
                self.e_marks[line] + 1
            } else {
                self.e_marks[line]
            };

            if last > self.src.len() { last = self.src.len(); }

            let mut first = line_start;
            let mut chars = self.src[first..last].chars();

            while line_indent < indent {
                match chars.next() {
                    Some(' ') => {
                        line_indent += 1;
                        first += 1;
                    }
                    Some('\t') => {
                        line_indent += 4 - (line_indent + self.bs_count[line]) % 4;
                        first += 1;
                    }
                    Some(_) if first - line_start < self.t_shift[line] => {
                        // patched tShift masked characters to look like spaces (blockquotes, list markers)
                        line_indent += 1;
                        first += 1;
                    }
                    _ => break,
                }
            }

            if line_indent > indent {
                // partially expanding tabs in code blocks, e.g '\t\tfoobar'
                // with indent=2 becomes '  \tfoobar'
                result += &" ".repeat(line_indent - indent);
            }

            result += &self.src[first..last];
            line += 1;
        }

        result
    }
}
