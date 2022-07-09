use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn benchmark(c: &mut Criterion) {

    let source = std::fs::read_to_string("benches/test-file.md").unwrap();
    let md = &mut markdown_it::parser::new();
    markdown_it::syntax::cmark::add(md);
    markdown_it::syntax::html::add(md);

    c.bench_function("markdown-it", |b| b.iter(|| {
        let ast = md.parse(&source);
        let html = markdown_it::renderer::html(&ast);
        black_box(html);
    }));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
