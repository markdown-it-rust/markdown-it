use argparse;
use markdown_it;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut input = "-".to_owned();
    let mut output = "-".to_owned();

    {
        let mut cli = argparse::ArgumentParser::new();

        cli.add_option(&["-v", "--version"], argparse::Print(env!("CARGO_PKG_VERSION").to_owned()), "Show version");

        cli
            .refer(&mut output)
            .add_option(&["-o", "--output"], argparse::Store, "File to write");

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
        html: true,
        lang_prefix: "language-",
        max_nesting: None,
        xhtml_out: true,
    }));
    markdown_it::syntax::CommonMark::add(md);
    let result = md.render(&source);

    if output == "-" {
        std::io::stdout().write(result.as_bytes()).unwrap();
    } else {
        std::fs::write(output, &result).unwrap();
    }
}
