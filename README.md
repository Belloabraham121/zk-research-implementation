# zk-research-implementation

Rust implementations of core algebraic primitives used in zero-knowledge proof systems: univariate and multilinear polynomials, Fiat–Shamir transcripts, Shamir secret sharing, and the sum-check protocol (including a GKR-oriented variant).

Arithmetic is over the BN254 scalar field (`ark_bn254::Fq`) via [arkworks](https://github.com/arkworks-rs).

This repository is a research / learning implementation. It is **not** audited and should not be used in production.

## Motivation

Interactive oracle proofs such as sum-check and GKR rest on a small set of operations:

1. Represent and evaluate polynomials over a finite field.
2. Reduce an *n*-variate claim to a univariate claim by binding one variable at a time.
3. Make the interaction non-interactive with a cryptographic transcript (Fiat–Shamir).

Each crate below implements one of those building blocks. Later crates depend on earlier ones.

## Repository layout

There is no workspace `Cargo.toml`. Each directory is an independent crate; path dependencies wire them together.

```
zk-research-implementation/
├── univariate_polynomial/     dense univariate polynomials + Lagrange interpolation
├── multilinear_polynomial/    hypercube evaluations, ProductPoly, SumPoly
├── fiat_shamir/               Keccak256 transcript
├── shamir_secret_sharing/     (t, n) secret sharing over Fq
└── sum_check/                 sum-check prove/verify + GKR-style sum-check
```

| Crate | Role | Depends on |
|---|---|---|
| `univariate_polynomial` | Dense coefficient polynomials | ark-ff, ark-bn254 |
| `multilinear_polynomial` | Multilinear polynomials in evaluation form | ark-ff, ark-bn254 |
| `fiat_shamir` | Public-coin challenges from a hash transcript | ark-ff, ark-bn254, sha3 |
| `shamir_secret_sharing` | Threshold secret sharing | `univariate_polynomial` |
| `sum_check` | Sum-check and GKR sum-check | `fiat_shamir`, `multilinear_polynomial`, `univariate_polynomial` |

## Prerequisites

- Rust (edition 2021)
- Cargo

## Running tests

From any crate directory:

```bash
cargo test
```

Examples:

```bash
cd univariate_polynomial && cargo test
cd ../multilinear_polynomial && cargo test
cd ../fiat_shamir && cargo test
cd ../shamir_secret_sharing && cargo test
cd ../sum_check && cargo test
```

## Benchmarks

Criterion benches live in:

- `univariate_polynomial/benches/univariate_poly_benchmark.rs` — evaluate (degree 99) and interpolate (10 points)
- `multilinear_polynomial/benches/multilinear_poly_benchmark.rs` — evaluate a 10-variable polynomial
- `sum_check/benches/sum_check_benchmark.rs` — prove and verify on a 3-variable polynomial

```bash
cd univariate_polynomial && cargo bench
cd ../multilinear_polynomial && cargo bench
cd ../sum_check && cargo bench
```

---

## Crate notes

### 1. Univariate polynomials

`univariate_polynomial::univariate_polynomial_dense::UnivariatePoly<F>`

Dense coefficient vector `[c0, c1, …, cd]` for \( f(x) = \sum_i c_i x^i \).

| Operation | What it does |
|---|---|
| `evaluate(x)` | Horner-style power evaluation |
| `interpolate(points)` | Lagrange interpolation through \((x_i, y_i)\) |
| `+` / `*` | Polynomial addition and multiplication |
| `degree()` | Degree after trimming leading zeros |

Interpolation is the reconstruction primitive used by Shamir secret sharing and by the GKR round polynomial in sum-check.

```rust
use ark_bn254::Fq;
use univariate_polynomial::univariate_polynomial_dense::UnivariatePoly;

let points = vec![
    (Fq::from(0), Fq::from(2)),
    (Fq::from(1), Fq::from(4)),
    (Fq::from(2), Fq::from(6)),
];
let poly = UnivariatePoly::interpolate(points);
assert_eq!(poly.evaluate(Fq::from(3)), Fq::from(8));
```

### 2. Multilinear polynomials

`multilinear_polynomial::multilinear_polynomial_evaluation::MultilinearPoly<F>`

An *n*-variate multilinear polynomial is stored as its \(2^n\) evaluations on the boolean hypercube \(\{0,1\}^n\). The constructor panics if the vector length is not a power of two.

| Operation | What it does |
|---|---|
| `partial_evaluate(bit, r)` | Bind one variable to field element `r` (linear interpolation between paired hypercube points) |
| `evaluate(values)` | Bind every variable, returning a single field element |
| `+` / `*` | Pointwise add / multiply of evaluation tables |

**Composed polynomials** (used by GKR sum-check):

- `ProductPoly` — product of several multilinear polynomials of the same size.
- `SumPoly` — sum of `ProductPoly`s of the same degree. This is the shape \(\sum_i \prod_j f_{ij}\) that appears in GKR wiring predicates.

```rust
use ark_bn254::Fq;
use multilinear_polynomial::multilinear_polynomial_evaluation::MultilinearPoly;

// 2-variate: evaluations at (0,0), (0,1), (1,0), (1,1)
let poly = MultilinearPoly::new(vec![
    Fq::from(0), Fq::from(0), Fq::from(3), Fq::from(10),
]);
let result = poly.evaluate(vec![Fq::from(5), Fq::from(1)]);
assert_eq!(result, Fq::from(50));
```

### 3. Fiat–Shamir transcript

`fiat_shamir::fiat_shamir_transcript::Transcript<F>`

Turns an interactive public-coin protocol into a non-interactive one.

- Hash function: Keccak-256 (`sha3::Keccak256`)
- `append(bytes)` absorbs protocol messages
- `get_random_challenge()` squeezes a field element, then re-absorbs the digest so later challenges depend on earlier ones
- `fq_vec_to_bytes` serializes `Fq` vectors in little-endian for absorption

Prover and verifier must append the same messages in the same order, or verification fails.

### 4. Shamir secret sharing

`shamir_secret_sharing`

Classic \((t, n)\) sharing over \( \mathbb{F}_q \):

1. `create_polynomia(threshold, secret, x_secret)` interpolates a degree-\((t-1)\) polynomial that takes value `secret` at `x_secret`. Remaining points are random.
2. `share_points(n, t, poly)` evaluates the polynomial at *n* random x-coordinates.
3. `recover_polynomial(shares, t)` interpolates from at least *t* shares.
4. `get_secret(poly, x_secret)` evaluates at the secret x-coordinate.

Fewer than *t* shares cannot reconstruct the polynomial. Tests cover the full share → recover → secret round-trip and rejection of bad points.

### 5. Sum-check protocol

`sum_check::sum_check_protocol`

Non-interactive sum-check for a multilinear polynomial \( g \) over the boolean hypercube:

\[
\sum_{x \in \{0,1\}^n} g(x) \stackrel{?}{=} H
\]

**`prove` / `verify`** (plain multilinear):

1. Prover commits the evaluation table and claimed sum \( H \) into the transcript.
2. For each variable \( i = 1 \ldots n \):
   - Prover sends the univariate \( g_i(X) = \sum_{x_{i+1},\ldots,x_n} g(r_1,\ldots,r_{i-1}, X, x_{i+1},\ldots,x_n) \), represented by the two evaluations at \( 0 \) and \( 1 \).
   - Verifier checks \( g_i(0) + g_i(1) = \) the current claimed sum.
   - Fiat–Shamir produces challenge \( r_i \); the claim becomes \( g_i(r_i) \).
3. Final check: \( g(r_1,\ldots,r_n) \) equals the last claimed sum.

**`gkr_prove` / `gkr_verify`** (composed `SumPoly`):

Same round structure, but each round polynomial can have degree greater than 1 (product of multiliners). The round polynomial is recovered by evaluating the composed polynomial at \( 0, 1, \ldots, d \) and interpolating a univariate. This is the inner sum-check used inside GKR.

```rust
use ark_bn254::Fq;
use multilinear_polynomial::multilinear_polynomial_evaluation::MultilinearPoly;
use sum_check::sum_check_protocol::{prove, verify};

let poly = MultilinearPoly::new(vec![
    Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2),
    Fq::from(0), Fq::from(10), Fq::from(0), Fq::from(17),
]);

let proof = prove(&poly);
assert!(verify(&poly, proof));
```

A forged claimed sum or round polynomial is rejected (`test_invalid_proof_doesnt_verify`).

## Protocol map

```
univariate interpolation ──► Shamir secret sharing
         │
         ▼
multilinear eval form ──► sum-check (plain)
         │                        ▲
         ├── ProductPoly / SumPoly ┘
         │
         └──► GKR-style sum-check
                    ▲
Fiat–Shamir transcript (Keccak-256)
```

## Dependencies

| Crate | Version | Use |
|---|---|---|
| `ark-ff` | 0.5.0 | Prime-field traits |
| `ark-bn254` | 0.5.0 | Field `Fq` |
| `ark-std` | 0.5.0 | RNG helpers (Shamir) |
| `sha3` | 0.10.8 | Keccak-256 transcript |
| `rand` | 0.8.5 | Share / polynomial sampling |
| `criterion` | 0.5.1 | Benchmarks |

## Status

Implemented and covered by unit tests:

- Univariate evaluate / interpolate / add / mul
- Multilinear partial and full evaluation
- Product and sum of multilinear polynomials
- Fiat–Shamir challenges
- Shamir share and reconstruct
- Sum-check prove and verify
- GKR-oriented sum-check prove and verify

Not in this repo: a full GKR prover for a circuit, polynomial commitment schemes, or SNARK wrapping.

## License

No license file is present in the repository. All rights reserved unless the author adds a license.
