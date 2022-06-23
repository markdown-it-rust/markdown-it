use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn benchmark(c: &mut Criterion) {

    let source = std::fs::read_to_string("benches/test-file.md").unwrap();
    let md = &mut markdown_it::MarkdownIt::new(Some(markdown_it::Options {
        breaks: false,
        lang_prefix: "language-",
        max_nesting: None,
        xhtml_out: true,
    }));
    markdown_it::syntax::cmark::add(md);
    markdown_it::syntax::html::add(md);

    c.bench_function("markdown-it", |b| b.iter(|| {
        let result = md.render(&source);
        black_box(result);
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
