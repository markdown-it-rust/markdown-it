//! Pretty-print all urls and fit them into N characters

use crate::parser::linkfmt::{LinkFormatter, MDLinkFormatter};
use crate::MarkdownIt;

#[derive(Debug)]
struct LinkBeautifier {
    max_length: usize,
    parent: Box<dyn LinkFormatter>,
}

impl LinkFormatter for LinkBeautifier {
    fn validate_link(&self, url: &str) -> Option<()> {
        self.parent.as_ref().validate_link(url)
    }

    fn normalize_link(&self, url: &str) -> String {
        mdurl::format_url_for_computers(url)
    }

    fn normalize_link_text(&self, url: &str) -> String {
        mdurl::format_url_for_humans(url, self.max_length)
    }
}


/// Add beautifier plugin, limiting urls to default 50 characters
pub fn add(md: &mut MarkdownIt) {
    add_with_char_limit(md, 50);
}

/// Add beautifier plugin, limiting urls to `max_length` characters
pub fn add_with_char_limit(md: &mut MarkdownIt, max_length: usize) {
    let parent = std::mem::replace(&mut md.link_formatter, Box::new(MDLinkFormatter::new()));
    md.link_formatter = Box::new(LinkBeautifier {
        max_length,
        parent,
    });
}
