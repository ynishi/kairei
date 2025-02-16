use criterion::{criterion_group, criterion_main, Criterion};

fn bench_example(c: &mut Criterion) {
    c.bench_function("sum 0 to 1000", |b| b.iter(|| (0..1000).sum::<i32>()));
}

// ベンチマークグループの定義
criterion_group!(benches, bench_example);
criterion_main!(benches);
