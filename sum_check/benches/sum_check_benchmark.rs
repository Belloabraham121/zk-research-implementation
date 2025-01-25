use ark_bn254::Fq;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use multilinear_polynomial::multilinear_polynomial_evaluation::MultilinearPoly;
use sum_check::sum_check_protocol::{prove, verify};

/// Benchmarks the proving procedure of the sum-check protocol.
/// This benchmark creates a fixed multilinear polynomial and repeatedly
/// computes its proof.
pub fn prove_benchmark(c: &mut Criterion) {
    // For simplicity, we use an 8-evaluation polynomial (e.g. a 3-variable multilinear polynomial)
    let evaluations: Vec<Fq> = vec![
        Fq::from(0),
        Fq::from(0),
        Fq::from(0),
        Fq::from(2),
        Fq::from(0),
        Fq::from(10),
        Fq::from(0),
        Fq::from(17),
    ];
    let poly = MultilinearPoly::new(evaluations);

    c.bench_function("SumCheck Prove", |b| {
        b.iter(|| {
            // Call prove on the polynomial.
            let proof = prove(black_box(&poly));
            // Use black_box to avoid optimizations removing the computation.
            black_box(proof);
        })
    });
}

/// Benchmarks the verification procedure of the sum-check protocol.
/// First, we precompute a valid proof from a fixed polynomial, and then in the benchmark loop
/// we verify it. (This assumes that `Proof` is cloneable; if not, consider modifying the benchmark.)
pub fn verify_benchmark(c: &mut Criterion) {
    // Same fixed polynomial as above.
    let evaluations: Vec<Fq> = vec![
        Fq::from(0),
        Fq::from(0),
        Fq::from(0),
        Fq::from(2),
        Fq::from(0),
        Fq::from(10),
        Fq::from(0),
        Fq::from(17),
    ];
    let poly = MultilinearPoly::new(evaluations);
    // Precompute a valid proof.
    let proof = prove(&poly);

    c.bench_function("SumCheck Verify", |b| {
        b.iter(|| {
            // Assuming Proof implements Clone so we can use the same proof repeatedly.
            let valid = verify(black_box(&poly), black_box(proof.clone()));
            black_box(valid);
        })
    });
}

criterion_group!(benches, prove_benchmark, verify_benchmark);
criterion_main!(benches);
