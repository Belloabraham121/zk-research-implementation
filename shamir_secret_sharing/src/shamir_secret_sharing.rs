use ark_bn254::Fq;
use ark_std::rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use univariate_polynomial::univariate_polynomial_dense::UnivariatePoly;

fn create_polynomia(threshold: usize, secret_value: Fq, secret_point: Fq) -> UnivariatePoly<Fq> {
    let mut points = vec![];
    let base_point = (secret_point, secret_value);

    points.push(base_point);

    let mut rng = StdRng::from_entropy();

    for _ in 1..threshold {
        let random_x_point = rng.gen();
        let random_y_point = rng.gen();

        points.push((random_x_point, random_y_point));
    }

    let poly = UnivariatePoly::interpolate(points);

    poly
}

fn recover_polynomial(points: Vec<(Fq, Fq)>, threshold: usize) -> UnivariatePoly<Fq> {
    if points.len() < threshold {
        panic!("Not enough points to recreate polynomial");
    }

    let selected_points: Vec<(Fq, Fq)> = if points.len() > 3 {
        points[0..4].to_vec()
    } else {
        points.clone()
    };

    UnivariatePoly::interpolate(selected_points)
}

fn get_secret(poly: &UnivariatePoly<Fq>, x_point: Fq) -> Fq {
    let secret = poly.evaluate(Fq::from(x_point));

    secret
}

fn share_points(
    num_of_shares: usize,
    threshold: usize,
    poly: &UnivariatePoly<Fq>,
) -> Vec<(Fq, Fq)> {
    if num_of_shares < threshold {
        panic!("Num of shares too low")
    }

    let mut rng = StdRng::from_entropy();

    let mut shares: Vec<(Fq, Fq)> = vec![];
    // let mut shares: Vec<(Fq, Fq)> = vec![(Fq::from(0), Fq::from(0)); num_of_shares];

    for _ in 0..num_of_shares {
        let random_x_point = rng.gen();

        let y_point = poly.evaluate(random_x_point);

        shares.push((random_x_point, y_point));
    }

    shares
}

#[cfg(test)]
mod test {
    use ark_bn254::Fq;
    use univariate_polynomial::univariate_polynomial_dense::UnivariatePoly;

    use super::{create_polynomia, get_secret, recover_polynomial, share_points};

    #[test]
    fn it_creates_a_correct_polynomial() {
        let threshold = 4;
        let secret_value = Fq::from(40);
        let secret_point = Fq::from(6);

        let mut polynomial = create_polynomia(threshold, secret_value, secret_point);

        let secret_evaluation = polynomial.evaluate(Fq::from(6));

        assert_eq!(polynomial.degree(), 3);
        assert_eq!(secret_evaluation, Fq::from(40));
    }

    #[test]
    fn it_recreates_polynomial_with_valid_points() {
        let points = vec![
            (Fq::from(1), Fq::from(-1)),
            (Fq::from(2), Fq::from(5)),
            (Fq::from(3), Fq::from(13)),
        ];
        let threshold = 3;

        let secret_poly = recover_polynomial(points, threshold);

        assert_eq!(
            secret_poly.coefficient,
            vec![Fq::from(-5), Fq::from(3), Fq::from(1)]
        );
    }

    #[test]
    fn it_returns_right_secret() {
        let secret_poly = UnivariatePoly::new(vec![Fq::from(-5), Fq::from(3), Fq::from(1)]);

        let secret_data = get_secret(&secret_poly, Fq::from(0));

        assert_eq!(secret_data, Fq::from(-5));
    }

    #[test]
    fn it_share_points_correctly() {
        let secret_poly = UnivariatePoly::new(vec![Fq::from(-5), Fq::from(3), Fq::from(1)]);

        let shares = share_points(10, 3, &secret_poly);

        assert_eq!(shares.len(), 10);
    }

    #[test]
    fn it_doesnt_work_with_wrong_points() {
        let wrong_point = (Fq::from(3), Fq::from(1));

        let points = vec![
            (Fq::from(1), Fq::from(-1)),
            (Fq::from(2), Fq::from(5)),
            wrong_point,
        ];

        let polynomial = recover_polynomial(points, 3);

        assert_ne!(
            polynomial.coefficient,
            vec![Fq::from(-5), Fq::from(3), Fq::from(1)]
        );
        assert_ne!(polynomial.evaluate(Fq::from(0)), Fq::from(-5))
    }

    #[test]
    #[should_panic]
    fn it_doesnt_generate_with_few_points() {
        let points = vec![(Fq::from(1), Fq::from(-1)), (Fq::from(2), Fq::from(5))];

        let _ = recover_polynomial(points, 3);
    }

    #[test]
    fn it_all_works_properly() {
        let secret_point = Fq::from(0);
        let secret_data = Fq::from(-5);
        let secret_poly = create_polynomia(3, secret_data, secret_point);

        let shares = share_points(10, 3, &secret_poly);

        let collected_shares = shares[2..6].to_vec();

        let recreated_poly = recover_polynomial(collected_shares, 3);

        let recovered_secret = get_secret(&recreated_poly, secret_point);

        assert_eq!(recreated_poly.coefficient, secret_poly.coefficient);
        assert_eq!(recovered_secret, secret_data);
    }
}
