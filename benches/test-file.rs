use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn benchmark(c: &mut Criterion) {

    let source = std::fs::read_to_string("benches/test-file.md").unwrap();
    let md = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md);
    markdown_it::plugins::html::add(md);

    c.bench_function("markdown-it", |b| b.iter(|| {
        let html = md.parse(&source).render();
        black_box(html);
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
