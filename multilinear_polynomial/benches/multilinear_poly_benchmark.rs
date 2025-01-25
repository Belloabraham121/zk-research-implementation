use ark_bn254::Fq;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use multilinear_polynomial::multilinear_polynomial_evaluation::MultilinearPoly;

pub fn criterion_benchmark(c: &mut Criterion) {
    // Set the number of variables; our polynomial will have 2^(num_vars) evaluations.
    let num_vars = 10;
    let num_evals = 1 << num_vars;

    // Build a vector of evaluations for the polynomial.
    // Here we simply use Fq::from(i) for each index.
    let evaluations: Vec<Fq> = (0..num_evals).map(|i| Fq::from(i as u64)).collect();

    // Construct the multilinear polynomial.
    let poly = MultilinearPoly::new(evaluations);

    // Define a fixed assignment for each variable.
    // In this case, we evaluate the polynomial at (2, 2, ..., 2).
    let values: Vec<Fq> = vec![Fq::from(2); num_vars];

    // Benchmark the full evaluation of the polynomial.
    c.bench_function("MultilinearPoly evaluate", |b| {
        b.iter(|| {
            // Use black_box to prevent compiler optimizations from removing our computation.
            let result = poly.evaluate(black_box(values.clone()));
            black_box(result);
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
