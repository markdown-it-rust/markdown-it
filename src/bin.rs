use argparse;
use markdown_it;
use markdown_it::syntax_base::builtin::Text;
use markdown_it::syntax_base::builtin::TextSpecial;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut input = "-".to_owned();
    let mut output = "-".to_owned();
    let mut no_html = false;
    let mut sourcepos = false;
    let mut show_tree = false;

    {
        let mut cli = argparse::ArgumentParser::new();

        cli.add_option(&["-v", "--version"], argparse::Print(env!("CARGO_PKG_VERSION").to_owned()), "Show version");

        cli
            .refer(&mut output)
            .add_option(&["-o", "--output"], argparse::Store, "File to write");

        cli
            .refer(&mut sourcepos)
            .add_option(&["--sourcepos"], argparse::StoreTrue, "Include source mappings in HTML attributes");

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
        max_nesting: None,
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

        // TODO: generic walk
        fn walk(tokens: &mut Vec<Token>, f: fn (&mut Token, lvl: u32), lvl: u32) {
            for token in tokens.iter_mut() {
                f(token, lvl);
                walk(&mut token.children, f, lvl + 1);
            }
        }

        let mut tree = md.parse(&source);

        tree.walk(|node, lvl| {
            print!("{}", "    ".repeat(lvl as usize));
            let name = &node.name()[node.name().rfind("::").map(|x| x+2).unwrap_or_default()..];
            if let Some(data) = node.cast::<Text>() {
                println!("{}: {:?}", name, data.content);
            } else if let Some(data) = node.cast::<TextSpecial>() {
                println!("{}: {:?}", name, data.content);
            } else {
                println!("{}", name);
            }
        });

        return;
    }

    let result;
    if sourcepos {
        result = markdown_it::renderer::html_with_srcmap(&source, &md.parse(&source));
    } else {
        result = md.render(&source);
    }

    if output == "-" {
        std::io::stdout().write(result.as_bytes()).unwrap();
    } else {
        std::fs::write(output, &result).unwrap();
    }
}
