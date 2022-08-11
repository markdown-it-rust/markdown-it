// run it like this:
// cargo test --test pathological --jobs 1 -- --nocapture --test-threads=1
use markdown_it::MarkdownIt;
use once_cell::sync::Lazy;
use std::time::SystemTime;

static MD : Lazy<MarkdownIt> = Lazy::new(|| {
    let mut parser = markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(&mut parser);
    markdown_it::plugins::html::add(&mut parser);
    markdown_it::plugins::extra::add(&mut parser);
    parser
});

fn run(src: &str) {
    let now = SystemTime::now();
    MD.parse(src);
    dbg!(now.elapsed().ok().unwrap());
}


mod commonmark {
    // Ported from cmark, https://github.com/commonmark/cmark/blob/master/test/pathological_tests.py
    use super::run;

    #[test]
    fn nested_inlines() {
        run(&format!("{}{}{}", "*".repeat(100000), "a", "*".repeat(100000)));
    }

    #[test]
    fn nested_strong_emph() {
        // suspiciously slow
        run(&format!("{}{}{}", "*a **a".repeat(5000), "b", " a** a*".repeat(5000)));
    }

    #[test]
    fn many_emph_closers_with_no_openers() {
        run(&"a_ ".repeat(100000));
    }

    #[test]
    fn many_emph_openers_with_no_closers() {
        run(&"_a ".repeat(100000));
    }

    #[test]
    fn many_link_closers_with_no_openers() {
        run(&"a]".repeat(100000));
    }

    #[test]
    fn many_link_openers_with_no_closers() {
        // most probably a bug
        run(&"[a".repeat(1500));
    }

    #[test]
    fn mismatched_openers_and_closers() {
        // most probably a bug
        run(&"*a_ ".repeat(50000));
    }

    #[test]
    fn commonmark_cmark_389() {
        run(&format!("{}{}", "*a ".repeat(2000), "_a*_ ".repeat(2000)));
    }

    #[test]
    fn openers_and_closers_multiple_of_3() {
        run(&format!("{}{}", "a**b", "c* ".repeat(50000)));
    }

    #[test]
    fn link_openers_and_emph_closers() {
        run(&"[ a_".repeat(1000));
    }

    #[test]
    fn link_pattern_repeated() {
        run(&"[ (](".repeat(100000));
    }

    #[test]
    fn nested_brackets() {
        // very slow
        run(&format!("{}{}{}", "[".repeat(1500), "a", "]".repeat(1500)));
    }

    #[test]
    fn nested_block_quotes() {
        run(&format!("{}{}", "> ".repeat(50000), "a"));
    }

    #[test]
    fn deeply_nested_lists() {
        let src = (0..5000).map(|x| format!("{}{}", "  ".repeat(x), "* a\n")).collect::<Vec<_>>().join("");
        run(&src);
    }

    #[test]
    fn backticks() {
        let src = (0..1000).map(|x| format!("{}{}", "e", "`".repeat(x))).collect::<Vec<_>>().join("");
        run(&src);
    }

    #[test]
    fn unclosed_links_a() {
        run(&"[a](<b".repeat(30000));
    }

    #[test]
    fn unclosed_links_b() {
        run(&"[a](b".repeat(30000));
    }
}

mod markdownit {
    // Ported from markdown-it.js
    use super::run;

    #[test]
    fn emphasis_pattern() {
        run(&"**_* ".repeat(5000));
    }

    #[test]
    fn backtick_pattern() {
        run(&"``\\".repeat(50000));
    }

    #[test]
    fn autolinks_pattern() {
        run(&format!("{}{}", "<".repeat(100000), ">"));
    }

    #[test]
    fn hardbreak_whitespaces_pattern() {
        run(&format!("{}{}{}", "x", " ".repeat(100000), "x  \nx"));
    }
}
