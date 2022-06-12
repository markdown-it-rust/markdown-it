mod inline;
mod block;
mod core;

use crate::MarkdownIt;

pub struct CommonMark;

impl CommonMark {
    pub fn add(md: &mut MarkdownIt) {
        md.inline.ruler.push("text",            inline::text::rule);
        //md.inline.ruler.push("linkify",         inline::linkify::rule);
        md.inline.ruler.push("newline",         inline::newline::rule);
        md.inline.ruler.push("escape",          inline::escape::rule);
        md.inline.ruler.push("backticks",       inline::backticks::rule);
        md.inline.ruler.push("strikethrough",   inline::strikethrough::rule);
        md.inline.ruler.push("emphasis",        inline::emphasis::rule);
        md.inline.ruler.push("link",            inline::link::rule);
        md.inline.ruler.push("image",           inline::image::rule);
        md.inline.ruler.push("autolink",        inline::autolink::rule);
        md.inline.ruler.push("html_inline",     inline::html_inline::rule);
        md.inline.ruler.push("entity",          inline::entity::rule);

        // `rule2` ruleset was created specifically for emphasis/strikethrough
        // post-processing and may be changed in the future.
        //
        // Don't use this for anything except pairs (plugins working with `balance_pairs`).
        //
        md.inline.ruler2.push("balance_pairs",  inline::balance_pairs::postprocess);
        md.inline.ruler2.push("strikethrough",  inline::strikethrough::postprocess);
        md.inline.ruler2.push("emphasis",       inline::emphasis::postprocess);
        // rules for pairs separate '**' into its own text tokens, which may be left unused,
        // rule below merges unused segments back with the rest of the text
        md.inline.ruler2.push("fragments_join", inline::fragments_join::postprocess);

        // First 2 params - rule name & source. Secondary array - list of rules,
        // which can be terminated by this one.
        //result.ruler.push("table",              block::table::rule);
        md.block.ruler.push("code",             block::code::rule);
        md.block.ruler.push("fence",            block::fence::rule);
        md.block.ruler.push("blockquote",       block::blockquote::rule);
        md.block.ruler.push("hr",               block::hr::rule);
        md.block.ruler.push("list",             block::list::rule);
        md.block.ruler.push("reference",        block::reference::rule);
        md.block.ruler.push("html_block",       block::html_block::rule);
        md.block.ruler.push("heading",          block::heading::rule);
        md.block.ruler.push("lheading",         block::lheading::rule);
        md.block.ruler.push("paragraph",        block::paragraph::rule);

        md.core.ruler.push("normalize",         core::normalize::rule);
        md.core.ruler.push("block",             core::block::rule);
        md.core.ruler.push("inline",            core::inline::rule);
        //md.core.ruler.push("linkify",           core::linkify::rule);
        //md.core.ruler.push("replacements",      core::replacements::rule);
        //md.core.ruler.push("smartquotes",       core::smartquotes::rule);
        // `text_join` finds `text_special` tokens (for escape sequences)
        // and joins them with the rest of the text
        md.core.ruler.push("text_join",         core::text_join::rule);
    }
}
