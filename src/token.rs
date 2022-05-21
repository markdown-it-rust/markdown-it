pub type TokenAttrs = Vec<(&'static str, String)>;

// Token class
#[derive(Debug)]
pub struct Token {
    // Type of the token (string, e.g. "paragraph_open")
    pub name: &'static str,

    // html tag name, e.g. "p"
    pub tag: &'static str,

    // Html attributes. Format: `[ [ name1, value1 ], [ name2, value2 ] ]
    pub attrs: TokenAttrs,

    // Source map info. Format: `[ line_begin, line_end ]`
    pub map: Option<[usize; 2]>,

    // Level change (number in {-1, 0, 1} set), where:
    //
    // -  `1` means the tag is opening
    // -  `0` means the tag is self-closing
    // - `-1` means the tag is closing
    pub nesting: i8,

    // nesting level, same as `state.level` for block tags, always 0 for inlines
    pub level: u32,

    // An array of child nodes (inline and img tokens)
    pub children: Vec<Token>,

    // In a case of self-closing tag (code, html, fence, etc.),
    // it has contents of this tag.
    pub content: String,

    // '*' or '_' for emphasis, fence string for fence, etc.
    pub markup: String,

    // Additional information:
    //
    // - Info string for "fence" tokens
    // - The value "auto" for autolink "link_open" and "link_close" tokens
    // - The string value of the item marker for ordered-list "list_item_open" tokens
    pub info: String,

    // True for block-level tokens, false for inline tokens.
    // Used in renderer to calculate line breaks
    pub block: bool,

    // If it's true, ignore this element when rendering. Used for tight lists
    // to hide paragraphs.
    pub hidden: bool,
}

impl Token {
    /**
     * new Token(type, tag, nesting)
     *
     * Create new token and fill passed properties.
     **/
    pub fn new(name: &'static str, tag: &'static str, nesting: i8) -> Self {
        Self {
            name,
            tag,
            attrs:     Vec::new(),
            map:       None,
            nesting,
            level:     0,
            children:  Vec::new(),
            content:   String::new(),
            markup:    String::new(),
            info:      String::new(),
            block:     false,
            hidden:    false,
        }
    }
}
