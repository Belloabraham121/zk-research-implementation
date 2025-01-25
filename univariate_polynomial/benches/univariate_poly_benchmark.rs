use ark_bn254::Fq;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use univariate_polynomial::univariate_polynomial_dense::UnivariatePoly;

/// Benchmarks the evaluation of a univariate polynomial.
/// In this example, we create a polynomial with 100 coefficients
/// (i.e. degree 99) and evaluate it at a fixed point.
pub fn evaluate_benchmark(c: &mut Criterion) {
    // Create a polynomial with coefficients [0, 1, 2, ..., 99]
    let coeffs: Vec<Fq> = (0..100).map(|i| Fq::from(i as u64)).collect();
    let poly = UnivariatePoly::new(coeffs);

    // Choose an evaluation point.
    let x = Fq::from(2);

    c.bench_function("UnivariatePoly evaluate", |b| {
        b.iter(|| {
            // black_box prevents compiler optimizations
            let result = poly.evaluate(black_box(x));
            black_box(result);
        })
    });
}

/// Benchmarks the interpolation of a univariate polynomial from points.
/// Here, we generate 10 points that lie on a simple linear function,
/// and then interpolate to recover the polynomial.
pub fn interpolation_benchmark(c: &mut Criterion) {
    // Generate 10 points along the line f(x) = 3 + 2x.
    let points: Vec<(Fq, Fq)> = (0..10)
        .map(|i| {
            let x = Fq::from(i as u64);
            let y = Fq::from(3 + 2 * i as u64);
            (x, y)
        })
        .collect();

    c.bench_function("UnivariatePoly interpolate", |b| {
        b.iter(|| {
            // Clone points to ensure each iteration works on an identical input.
            let poly = UnivariatePoly::interpolate(black_box(points.clone()));
            black_box(poly);
        })
    });
}

criterion_group!(benches, evaluate_benchmark, interpolation_benchmark);
criterion_main!(benches);
