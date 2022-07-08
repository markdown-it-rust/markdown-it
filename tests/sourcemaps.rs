#[cfg(feature="sourcemap")]
mod test_sourcemaps {
    use markdown_it;
    use markdown_it::token::Token;
    use markdown_it::sourcemap::CharMapping;

    fn run(input: &str, f: fn (&[Token], CharMapping)) {
        let md = &mut markdown_it::MarkdownIt::new(Some(markdown_it::Options {
            max_nesting: None,
        }));
        markdown_it::syntax::cmark::add(md);
        markdown_it::syntax::html::add(md);
        let tokens = md.parse(&input);
        f(&tokens, CharMapping::new(input));
    }

    fn getmap(token: &Token, map: &CharMapping) -> ((u32, u32), (u32, u32)) {
        token.map.unwrap().get_positions(map)
    }

    #[test]
    fn paragraph() {
        // same as commonmark.js
        run("foo   \n     \n     \n\n  barbaz\n\tquux   \n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 1), (1, 6)),
            );
            assert_eq!(
                getmap(&tokens[1], &map),
                ((5, 3), (6, 8)),
            );
        });
    }

    #[test]
    fn hr() {
        // same as commonmark.js
        run(" ---  \n\n  * * *\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 2), (1, 6)),
            );
            assert_eq!(
                getmap(&tokens[1], &map),
                ((3, 3), (3, 7)),
            );
        });
    }

    #[test]
    fn heading() {
        // same as commonmark.js
        run("  \n  ### foo ###  \n\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((2, 3), (2, 15)),
            );
        });

        run("  #\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 3), (1, 3)),
            );
        });
    }

    #[test]
    fn lheading() {
        // same as commonmark.js
        run("  foo\n bar\n ----\n\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 3), (3, 5)),
            );
        });
    }

    #[test]
    fn fence() {
        // same as commonmark.js
        run("  ~~~ foo ~~~\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 3), (1, 13)),
            );
        });

        run("  ```\n 12\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 3), (2, 3)),
            );
        });

        run("```\n\n\n\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 1), (4, 0)),
            );
        });

        run("~~~\na\nb\n~~~  \nc\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 1), (4, 5)),
            );
        });
    }

    #[test]
    fn html_block() {
        // same as commonmark.js
        run("  <div>\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 3), (1, 7)),
            );
        });

        run("<div>\n</div>  \n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 1), (2, 8)),
            );
        });
    }

    #[test]
    fn code_block() {
        // same as commonmark.js
        run("      foo\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 5), (1, 9)),
            );
        });

        run("   a\n    b\n     c\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 4), (3, 6)),
            );
        });

        // this I believe to be error in commonmark, code block
        // only have 1 line as per spec, but cmark reports 3 lines
        run("    foobar  \n    \n    \n\nbar\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 5), (1, 12)),
            );
        });
    }

    #[test]
    fn blockquotes() {
        // same as commonmark.js
        run("  > foo  \n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 3), (1, 9)),
            );
        });

        run("> foo\nbar\n\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 1), (2, 3)),
            );
        });
    }

    #[test]
    fn lists() {
        // same as commonmark.js
        run(" 1. foo\n 2. bar\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 2), (2, 7)),
            );

            assert_eq!(
                getmap(&tokens[0].children[0], &map),
                ((1, 2), (1, 7)),
            );
        });

        run(" - foo\n\n - bar\n", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0], &map),
                ((1, 2), (3, 6)),
            );

            assert_eq!(
                getmap(&tokens[0].children[0], &map),
                ((1, 2), (2, 0)),
            );

            assert_eq!(
                getmap(&tokens[0].children[1], &map),
                ((3, 2), (3, 6)),
            );
        });
    }

    #[test]
    fn autolinks() {
        run("foo <http://google.com> bar", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0].children[1], &map),
                ((1, 5), (1, 23)),
            );

            assert_eq!(
                getmap(&tokens[0].children[1].children[0], &map),
                ((1, 6), (1, 22)),
            );
        });
    }

    #[test]
    fn emphasis() {
        run("***foo***", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0].children[0], &map),
                ((1, 1), (1, 9)),
            );

            assert_eq!(
                getmap(&tokens[0].children[0].children[0], &map),
                ((1, 2), (1, 8)),
            );
        });

        run("aaa **bb _cc_ dd** eee", |tokens, map| {
            assert_eq!(
                getmap(&tokens[0].children[1], &map),
                ((1, 5), (1, 18)),
            );

            assert_eq!(
                getmap(&tokens[0].children[1].children[1], &map),
                ((1, 10), (1, 13)),
            );
        });
    }
}
