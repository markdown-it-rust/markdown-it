use markdown_it::parser::inline::{Text, TextSpecial};
use std::io::{Read, Write};

#[cfg(not(tarpaulin_include))]
fn main() {
    let mut input = "-".to_owned();
    let mut output = "-".to_owned();
    let mut no_html = false;
    #[cfg(feature = "linkify")]
    let mut linkify = false;
    let mut typographer = false;
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

        #[cfg(feature = "linkify")]
        cli
            .refer(&mut linkify)
            .add_option(&["-l", "--linkify"], argparse::StoreTrue, "Autolink text");

        cli
            .refer(&mut typographer)
            .add_option(&["-t", "--typographer"], argparse::StoreTrue, "Enable smartquotes and other typographic replacements");

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
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    #[cfg(feature = "syntect")]
    markdown_it::plugins::extra::syntect::add(md);
    markdown_it::plugins::extra::tables::add(md);
    markdown_it::plugins::extra::strikethrough::add(md);
    markdown_it::plugins::extra::beautify_links::add(md);
    if !no_html {
        markdown_it::plugins::html::add(md);
    }
    if sourcepos {
        markdown_it::plugins::sourcepos::add(md);
    }
    #[cfg(feature = "linkify")]
    if linkify {
        markdown_it::plugins::extra::linkify::add(md);
    }
    if typographer {
        markdown_it::plugins::extra::smartquotes::add(md);
        markdown_it::plugins::extra::typographer::add(md);
    }

    let ast = md.parse(&source);

    if show_tree {
        ast.walk(|node, depth| {
            print!("{}", "    ".repeat(depth as usize));
            let name = &node.name()[node.name().rfind("::").map(|x| x+2).unwrap_or_default()..];
            if let Some(data) = node.cast::<Text>() {
                println!("{name}: {:?}", data.content);
            } else if let Some(data) = node.cast::<TextSpecial>() {
                println!("{name}: {:?}", data.content);
            } else {
                println!("{name}");
            }
        });
        return;
    }

    let result = ast.render();

    if output == "-" {
        std::io::stdout().write_all(result.as_bytes()).unwrap();
    } else {
        std::fs::write(output, &result).unwrap();
    }
}
