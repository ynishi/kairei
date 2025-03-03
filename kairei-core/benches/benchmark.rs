use criterion::{Criterion, criterion_group, criterion_main};

fn bench_example(c: &mut Criterion) {
    c.bench_function("sum 0 to 1000", |b| b.iter(|| (0..1000).sum::<i32>()));
}

// ベンチマークグループの定義
criterion_group!(benches, bench_example);
criterion_main!(benches);
