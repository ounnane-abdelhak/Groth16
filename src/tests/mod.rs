#[cfg(test)]
mod tests {
    use crate::field::{ScalarField, fr_from_u64};
    use crate::poly::Polynomial;
    use crate::r1cs::{LinearCombination, R1cs};
    use crate::qap::r1cs_to_qap;
    use crate::setup::cr_proving_key;
    use crate::proof::generate_proof;
    use crate::verify::verify_proof;

    // -----------------------------------------------
    // Polynomial tests
    // -----------------------------------------------

    #[test]
    fn test_poly_add() {
        // (1 + 2x) + (3 + 4x) = 4 + 6x
        let a = Polynomial::new(vec![fr_from_u64(1), fr_from_u64(2)]);
        let b = Polynomial::new(vec![fr_from_u64(3), fr_from_u64(4)]);
        let c = a.add(&b);
        assert_eq!(c.coeff[0], fr_from_u64(4));
        assert_eq!(c.coeff[1], fr_from_u64(6));
    }

    #[test]
    fn test_poly_mul() {
        // (1 + x) * (1 + x) = 1 + 2x + x²
        let a = Polynomial::new(vec![fr_from_u64(1), fr_from_u64(1)]);
        let b = Polynomial::new(vec![fr_from_u64(1), fr_from_u64(1)]);
        let c = a.mul(&b);
        assert_eq!(c.coeff[0], fr_from_u64(1));
        assert_eq!(c.coeff[1], fr_from_u64(2));
        assert_eq!(c.coeff[2], fr_from_u64(1));
    }

    #[test]
    fn test_poly_eval() {
        // f(x) = 1 + 2x + 3x²  at x=2 => 1 + 4 + 12 = 17
        let f = Polynomial::new(vec![fr_from_u64(1), fr_from_u64(2), fr_from_u64(3)]);
        assert_eq!(f.eval(fr_from_u64(2)), fr_from_u64(17));
    }

    #[test]
    fn test_poly_div_rem_exact() {
        // (x² - 1) / (x - 1) = x + 1, remainder 0
        // x² - 1 = coeffs [-1, 0, 1]
        // x - 1  = coeffs [-1, 1]
        let num = Polynomial::new(vec![
            ScalarField::from(0u64) - fr_from_u64(1),
            fr_from_u64(0),
            fr_from_u64(1),
        ]);
        let den = Polynomial::new(vec![
            ScalarField::from(0u64) - fr_from_u64(1),
            fr_from_u64(1),
        ]);
        let (q, r) = num.div_rem(&den);
        assert!(r.is_zero(), "remainder should be zero for exact division");
        // quotient should be x + 1
        assert_eq!(q.coeff[0], fr_from_u64(1));
        assert_eq!(q.coeff[1], fr_from_u64(1));
    }

    #[test]
    fn test_poly_is_zero() {
        let z = Polynomial::zero();
        assert!(z.is_zero());
        let nz = Polynomial::new(vec![fr_from_u64(1)]);
        assert!(!nz.is_zero());
    }

    // -----------------------------------------------
    // R1CS tests
    // -----------------------------------------------

    fn build_simple_r1cs() -> (R1cs, Vec<ScalarField>) {
        // Circuit: x * x = x²  for x=3
        // witness: [1, x²=9, x=3]
        // constraint: w[2] * w[2] = w[1]
        let mut r = R1cs {
            num_variables: 3,
            num_public_inputs: 2, // w[0]=1, w[1]=x²
            constraints: Vec::new(),
        };
        let a = vec![(2usize, fr_from_u64(1))];
        let b = vec![(2usize, fr_from_u64(1))];
        let c = vec![(1usize, fr_from_u64(1))];
        r.add_constraint(
            LinearCombination { terms: a },
            LinearCombination { terms: b },
            LinearCombination { terms: c },
        );
        let wit = vec![fr_from_u64(1), fr_from_u64(9), fr_from_u64(3)];
        (r, wit)
    }

    #[test]
    fn test_r1cs_satisfied_valid_witness() {
        let (r, wit) = build_simple_r1cs();
        assert!(r.is_satisfied(&wit), "valid witness should satisfy R1CS");
    }

    #[test]
    fn test_r1cs_not_satisfied_wrong_witness() {
        let (r, mut wit) = build_simple_r1cs();
        wit[1] = fr_from_u64(99); // wrong x²
        assert!(!r.is_satisfied(&wit), "wrong witness should not satisfy R1CS");
    }

    #[test]
    fn test_r1cs_not_satisfied_wrong_length() {
        let (r, _) = build_simple_r1cs();
        let short_wit = vec![fr_from_u64(1), fr_from_u64(9)]; // missing one element
        assert!(!r.is_satisfied(&short_wit), "short witness should not satisfy R1CS");
    }

    #[test]
    fn test_linear_combination_evaluate() {
        // lc = 2*w[0] + 3*w[1], witness=[1, 4] => 2*1 + 3*4 = 14
        let lc = LinearCombination {
            terms: vec![
                (0usize, fr_from_u64(2)),
                (1usize, fr_from_u64(3)),
            ],
        };
        let wit = vec![fr_from_u64(1), fr_from_u64(4)];
        assert_eq!(lc.evaluate(&wit), fr_from_u64(14));
    }

    #[test]
    fn test_linear_combination_coefficient_of() {
        let lc = LinearCombination {
            terms: vec![
                (0usize, fr_from_u64(5)),
                (2usize, fr_from_u64(3)),
            ],
        };
        assert_eq!(lc.coefficient_of(0), fr_from_u64(5));
        assert_eq!(lc.coefficient_of(2), fr_from_u64(3));
        assert_eq!(lc.coefficient_of(1), fr_from_u64(0)); 
    }

    // -----------------------------------------------
    // QAP tests
    // -----------------------------------------------

    #[test]
    fn test_qap_reduction_remainder_zero() {
        // The QAP reduction must produce A(x)*B(x) - C(x) = H(x)*Z(x)
        // i.e. remainder must be zero for a valid witness
        let (r, wit) = build_simple_r1cs();
        let qp = r1cs_to_qap(&r);

        let mut a = Polynomial::zero();
        let mut b = Polynomial::zero();
        let mut c = Polynomial::zero();

        for (j, p) in qp.a_i.iter().enumerate() { a = a.add(&p.scale(wit[j])); }
        for (j, p) in qp.b_i.iter().enumerate() { b = b.add(&p.scale(wit[j])); }
        for (j, p) in qp.c_i.iter().enumerate() { c = c.add(&p.scale(wit[j])); }

        let p = a.mul(&b).sub(&c);
        let (_h, rem) = p.div_rem(&qp.z_i);
        assert!(rem.is_zero(), "QAP remainder must be zero for valid witness");
    }

    #[test]
    fn test_qap_domain_size_matches_constraints() {
        let (r, _) = build_simple_r1cs();
        let qp = r1cs_to_qap(&r);
        assert_eq!(qp.domain.len(), r.constraints.len());
    }

    #[test]
    fn test_qap_polynomial_counts_match_variables() {
        let (r, _) = build_simple_r1cs();
        let qp = r1cs_to_qap(&r);
        assert_eq!(qp.a_i.len(), r.num_variables);
        assert_eq!(qp.b_i.len(), r.num_variables);
        assert_eq!(qp.c_i.len(), r.num_variables);
    }

    // -----------------------------------------------
    // Full proof tests
    // -----------------------------------------------

fn build_full_circuit() -> (R1cs, Vec<ScalarField>) {
    // x³ + x + 5 = 35  for x=3
    // witness: [1, out=35, x=3, x²=9, x³=27, tmp=30]
    let mut r = R1cs {
        num_variables: 6,
        num_public_inputs: 2,
        constraints: Vec::new(),
    };
    // x * x = x²
    r.add_constraint(
        LinearCombination { terms: vec![(2usize, fr_from_u64(1))] },
        LinearCombination { terms: vec![(2usize, fr_from_u64(1))] },
        LinearCombination { terms: vec![(3usize, fr_from_u64(1))] },
    );
    // x² * x = x³
    r.add_constraint(
        LinearCombination { terms: vec![(3usize, fr_from_u64(1))] },
        LinearCombination { terms: vec![(2usize, fr_from_u64(1))] },
        LinearCombination { terms: vec![(4usize, fr_from_u64(1))] },
    );
    // (x³ + x) * 1 = tmp
    r.add_constraint(
        LinearCombination { terms: vec![(4usize, fr_from_u64(1)), (2usize, fr_from_u64(1))] },
        LinearCombination { terms: vec![(0usize, fr_from_u64(1))] },
        LinearCombination { terms: vec![(5usize, fr_from_u64(1))] },
    );
    // (tmp + 5) * 1 = out
    r.add_constraint(
        LinearCombination { terms: vec![(5usize, fr_from_u64(1)), (0usize, fr_from_u64(5))] },
        LinearCombination { terms: vec![(0usize, fr_from_u64(1))] },
        LinearCombination { terms: vec![(1usize, fr_from_u64(1))] },
    );
    let wit = vec![
        fr_from_u64(1),   // w[0] constant
        fr_from_u64(35),  // w[1] out (public)
        fr_from_u64(3),   // w[2] x
        fr_from_u64(9),   // w[3] x²
        fr_from_u64(27),  // w[4] x³
        fr_from_u64(30),  // w[5] tmp = x³ + x
    ];
    (r, wit)
}

    #[test]
    fn test_valid_proof_verifies() {
        let (r, wit) = build_full_circuit();
        let qp = r1cs_to_qap(&r);

        let mut a = Polynomial::zero();
        let mut b = Polynomial::zero();
        let mut c = Polynomial::zero();
        for (j, p) in qp.a_i.iter().enumerate() { a = a.add(&p.scale(wit[j])); }
        for (j, p) in qp.b_i.iter().enumerate() { b = b.add(&p.scale(wit[j])); }
        for (j, p) in qp.c_i.iter().enumerate() { c = c.add(&p.scale(wit[j])); }
        let poly = a.mul(&b).sub(&c);
        let (h, _) = poly.div_rem(&qp.z_i);

        let pk = cr_proving_key(
            qp.z_i.degree() - 1,
            r.num_public_inputs,
            r.num_variables,
            &qp,
        );
        let proof = generate_proof(&pk, &qp, &wit, &h, r.num_public_inputs);
        let result = verify_proof(&pk.vk, &proof, &wit[..r.num_public_inputs]);
        assert!(result, "valid proof should verify");
    }

    #[test]
    fn test_wrong_public_input_fails() {
        let (r, wit) = build_full_circuit();
        let qp = r1cs_to_qap(&r);

        let mut a = Polynomial::zero();
        let mut b = Polynomial::zero();
        let mut c = Polynomial::zero();
        for (j, p) in qp.a_i.iter().enumerate() { a = a.add(&p.scale(wit[j])); }
        for (j, p) in qp.b_i.iter().enumerate() { b = b.add(&p.scale(wit[j])); }
        for (j, p) in qp.c_i.iter().enumerate() { c = c.add(&p.scale(wit[j])); }
        let poly = a.mul(&b).sub(&c);
        let (h, _) = poly.div_rem(&qp.z_i);

        let pk = cr_proving_key(
            qp.z_i.degree() - 1,
            r.num_public_inputs,
            r.num_variables,
            &qp,
        );
        let proof = generate_proof(&pk, &qp, &wit, &h, r.num_public_inputs);

        // tamper with public input — claim x²=99 instead of 9
        let wrong_public = vec![fr_from_u64(1), fr_from_u64(99)];
        let result = verify_proof(&pk.vk, &proof, &wrong_public);
        assert!(!result, "proof with wrong public input should not verify");
    }
}
