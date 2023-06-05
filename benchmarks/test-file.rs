use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn benchmark(c: &mut Criterion) {
    let source = std::fs::read_to_string("test-file.md").unwrap();
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    markdown_it::plugins::html::add(md);
    c.bench_function("markdown-it", |b| b.iter(|| {
        let html = md.parse(&source).render();
        black_box(html);
    }));

    let md = &mut markdown_it_v5::MarkdownIt::new();
    markdown_it_v5::plugins::cmark::add(md);
    markdown_it_v5::plugins::html::add(md);
    c.bench_function("markdown-it-v5", |b| b.iter(|| {
        let html = md.parse(&source).render();
        black_box(html);
    }));

    use comrak::{format_html, parse_document, Arena, ComrakOptions};
    c.bench_function("comrak", |b| b.iter(|| {
        let arena = Arena::new();
        let root = parse_document(&arena, &source, &ComrakOptions::default());
        let mut output = vec![];
        format_html(root, &ComrakOptions::default(), &mut output).unwrap()
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
