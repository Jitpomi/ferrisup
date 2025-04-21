use criterion::{black_box, criterion_group, criterion_main, Criterion};
use {{project_name}}::*;

// This is a default benchmark template. You should modify this to benchmark
// your specific library functions.

fn example_benchmark(c: &mut Criterion) {
    c.bench_function("example", |b| {
        b.iter(|| {
            // Call your library function here
            black_box(2 + 2)
        })
    });
}

// Add benchmark groups for specific features

criterion_group!(benches, example_benchmark);
criterion_main!(benches);
