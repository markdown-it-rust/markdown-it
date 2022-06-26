use argparse;
use markdown_it;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut input = "-".to_owned();
    let mut output = "-".to_owned();
    let mut no_html = false;
    let mut show_tree = false;

    {
        let mut cli = argparse::ArgumentParser::new();

        cli.add_option(&["-v", "--version"], argparse::Print(env!("CARGO_PKG_VERSION").to_owned()), "Show version");

        cli
            .refer(&mut output)
            .add_option(&["-o", "--output"], argparse::Store, "File to write");

        cli
            .refer(&mut no_html)
            .add_option(&["--no-html"], argparse::StoreTrue, "Disable embedded HTML");

        cli
            .refer(&mut show_tree)
            .add_option(&["--tree"], argparse::StoreTrue, "Print syntax tree for debugging");

        cli
            .refer(&mut input)
            .add_argument("file", argparse::Store, "File to read");

        cli.parse_args_or_exit();
    }

    let vec = if input == "-" {
        let mut vec = Vec::new();
        std::io::stdin().read_to_end(&mut vec).unwrap();
        vec
    } else {
        std::fs::read(input).unwrap()
    };

    let source = String::from_utf8_lossy(&vec);
    let md = &mut markdown_it::MarkdownIt::new(Some(markdown_it::Options {
        breaks: false,
        lang_prefix: "language-",
        max_nesting: None,
        xhtml_out: true,
    }));
    markdown_it::syntax::cmark::add(md);
    if !no_html {
        markdown_it::syntax::html::add(md);
    }

    if show_tree {
        use markdown_it::token::Token;

        pub trait TokenList {
            fn walk(&mut self, f: fn (&mut Token, lvl: u32));
        }

        impl TokenList for Vec<Token> {
            fn walk(&mut self, f: fn (&mut Token, lvl: u32)) {
                walk(self, f, 0);
            }
        }

        fn walk(tokens: &mut Vec<Token>, f: fn (&mut Token, lvl: u32), lvl: u32) {
            for token in tokens.iter_mut() {
                f(token, lvl);
                walk(&mut token.children, f, lvl + 1);
            }
        }

        let mut tree = md.parse(&source);

        tree.walk(|node, lvl| {
            print!("{}", "    ".repeat(lvl as usize));
            if node.content.is_empty() {
                println!("{}", node.name);
            } else {
                println!("{}: {:?}", node.name, node.content);
            }
        });

        return;
    }

    let result = md.render(&source);

    if output == "-" {
        std::io::stdout().write(result.as_bytes()).unwrap();
    } else {
        std::fs::write(output, &result).unwrap();
    }
}
