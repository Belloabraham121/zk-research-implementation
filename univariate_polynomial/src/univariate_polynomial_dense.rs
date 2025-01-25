use ark_ff::PrimeField;
use std::ops::{Add, Mul};

#[derive(Debug, Clone)]
pub struct UnivariatePoly<F: PrimeField> {
    pub coefficient: Vec<F>,
}

impl<F: PrimeField> UnivariatePoly<F> {
    pub fn new(coeff: Vec<F>) -> Self {
        UnivariatePoly { coefficient: coeff }
    }

    fn trim(&mut self) {
        while self.coefficient.last() == Some(&F::zero()) {
            self.coefficient.pop();
        }
    }

    pub fn evaluate(&self, x: F) -> F {
        self.coefficient
            .iter()
            .enumerate()
            .map(|(index, coeff)| *coeff * x.pow(&[index as u64]))
            .sum()
    }

    pub fn degree(&mut self) -> usize {
        self.trim();

        self.coefficient.len() - 1
    }

    fn scalar_mul(&self, scalar: F) -> Self {
        let coefficients = self
            .coefficient
            .iter()
            .map(|point| *point * scalar)
            .collect();

        let mut poly = UnivariatePoly::new(coefficients);

        poly.trim();

        poly
    }

    pub fn interpolate(points: Vec<(F, F)>) -> UnivariatePoly<F> {
        let n = points.len();
        let mut result = UnivariatePoly::new(vec![F::zero()]);

        for i in 0..n {
            let (x_i, y_i) = points[i];
            let mut l_i = UnivariatePoly::new(vec![F::one()]);

            for j in 0..n {
                if i != j {
                    let (x_j, _) = points[j];

                    let numerator = UnivariatePoly::new(vec![-x_j, F::one()]);

                    let denominator = x_i - x_j;

                    l_i = l_i * numerator.scalar_mul(F::one() / denominator);
                }
            }

            result = result + l_i.scalar_mul(y_i);
        }

        result.trim();

        result
    }
}

impl<F: PrimeField> Add for UnivariatePoly<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut result = vec![F::zero(); self.coefficient.len().max(other.coefficient.len())];

        for (i, &coeff) in self.coefficient.iter().enumerate() {
            result[i] += coeff;
        }

        for (i, &coeff) in other.coefficient.iter().enumerate() {
            result[i] += coeff;
        }

        UnivariatePoly::new(result)
    }
}

impl<F: PrimeField> Mul for UnivariatePoly<F> {
    type Output = Self;

    fn mul(mut self, mut other: Self) -> Self {
        let mut coeffs = vec![F::zero(); self.degree() + other.degree() + 1];

        for (i, a) in self.coefficient.iter().enumerate() {
            for (j, b) in other.coefficient.iter().enumerate() {
                coeffs[i + j] += *a * b;
            }
        }

        UnivariatePoly::new(coeffs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn it_returns_degree() {
        let mut poly_1: UnivariatePoly<Fq> = UnivariatePoly {
            coefficient: vec![Fq::from(3), Fq::from(4), Fq::from(3)],
        };

        assert!(poly_1.degree() == 2);
    }

    #[test]
    fn it_evaluates_poly() {
        let poly_1 = UnivariatePoly {
            coefficient: vec![Fq::from(3), Fq::from(4), Fq::from(3)],
        };

        assert!(poly_1.evaluate(Fq::from(3)) == Fq::from(42));
    }

    #[test]
    fn it_scales() {
        let poly_1 = UnivariatePoly {
            coefficient: vec![Fq::from(3), Fq::from(4), Fq::from(3)],
        };

        assert!(
            poly_1.scalar_mul(Fq::from(2)).coefficient
                == vec![Fq::from(6), Fq::from(8), Fq::from(6)]
        );
    }

    #[test]
    fn it_adds_correctly() {
        let poly_1 = UnivariatePoly {
            coefficient: vec![Fq::from(3), Fq::from(4), Fq::from(3)],
        };

        let poly_2 = UnivariatePoly {
            coefficient: vec![Fq::from(-3), Fq::from(0), Fq::from(0), Fq::from(4)],
        };

        assert!(
            (poly_1 + poly_2).coefficient
                == vec![Fq::from(0), Fq::from(4), Fq::from(3), Fq::from(4)]
        );
    }

    #[test]
    fn it_multiplies_correctly() {
        let poly_1 = UnivariatePoly {
            coefficient: vec![Fq::from(3), Fq::from(4), Fq::from(3)],
        };

        let poly_2 = UnivariatePoly {
            coefficient: vec![Fq::from(-3), Fq::from(0), Fq::from(0), Fq::from(4)],
        };

        assert!(
            (poly_1 * poly_2).coefficient
                == vec![
                    Fq::from(-9),
                    Fq::from(-12),
                    Fq::from(-9),
                    Fq::from(12),
                    Fq::from(16),
                    Fq::from(12)
                ]
        );
    }

    #[test]
    fn it_interpolates_points() {
        let points = vec![
            (Fq::from(0), Fq::from(2)),
            (Fq::from(1), Fq::from(4)),
            (Fq::from(2), Fq::from(6)),
        ];

        let new_poly = UnivariatePoly::interpolate(points);

        assert!(new_poly.coefficient == vec![Fq::from(2), Fq::from(2)]);
    }
}
