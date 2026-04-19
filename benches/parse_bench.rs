use criterion::{criterion_group, criterion_main, Criterion};
use drift::dialect::Dialect;
use drift::parse::parse;

fn bench_parse(c: &mut Criterion) {
    let mut big = String::new();
    for i in 0..200 {
        big.push_str(&format!(
            "SELECT id, name, created_at FROM users WHERE tenant_id = {i} AND deleted_at IS NULL ORDER BY id LIMIT 50;\n"
        ));
    }
    c.bench_function("parse 200 selects", |b| {
        b.iter(|| parse(&big, Dialect::Postgres))
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
