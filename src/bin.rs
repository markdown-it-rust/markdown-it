use argparse;
use markdown_it;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut input = "-".to_owned();
    let mut output = "-".to_owned();
    let mut no_html = false;
    let mut sourcepos = false;

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

    let result;
    if sourcepos {
        #[cfg(feature="sourcemap")] {
            result = markdown_it::renderer::html_with_srcmap(&source, &md.parse(&source));
        }
        #[cfg(not(feature="sourcemap"))] {
            panic!(r#"--sourcepos requires markdown-it to be built with --features=sourcemap"#);
        }
    } else {
        result = md.render(&source);
    }

    if output == "-" {
        std::io::stdout().write(result.as_bytes()).unwrap();
    } else {
        std::fs::write(output, &result).unwrap();
    }
}
