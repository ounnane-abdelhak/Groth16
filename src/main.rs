mod field;
mod r1cs;
mod qap;
mod poly;
mod setup;
mod proof;
mod verify;
mod tests;

use crate::verify::verify_proof;
use crate::field::ScalarField;
use crate::poly::Polynomial;
use crate::qap::r1cs_to_qap;
use crate::qap::Qap;
use crate::r1cs::{ LinearCombination, R1cs};
use crate::setup::{cr_proving_key, ProvingKey};
use crate::proof::{Proof, generate_proof};

fn main() {
    // Circuit: x⁵ + 3x³ + 2x² + x + 7 = y  for x=2, y=73
    //
    // Intermediate variables:
    // w[0] = 1        (constant)
    // w[1] = y = 73   (public output)
    // w[2] = x = 2    (private input)
    // w[3] = x²= 4    (intermediate)
    // w[4] = x³= 8    (intermediate)
    // w[5] = x⁵= 32   (intermediate)
    // w[6] = tmp1 = x⁵ + 3x³ = 32+24 = 56  (intermediate)
    // w[7] = tmp2 = tmp1 + 2x² = 56+8 = 64  (intermediate)
    // w[8] = tmp3 = tmp2 + x + 7 = 64+2+7 = 73 = y (intermediate)
    //
    // Constraints:
    // 1: x  * x  = x²          w[2]*w[2] = w[3]
    // 2: x² * x  = x³          w[3]*w[2] = w[4]
    // 3: x³ * x² = x⁵          w[4]*w[3] = w[5]
    // 4: (x⁵ + 3x³) * 1 = tmp1 (w[5]+3w[4])*w[0] = w[6]
    // 5: (tmp1 + 2x²) * 1 = tmp2 (w[6]+2w[3])*w[0] = w[7]
    // 6: (tmp2 + x + 7) * 1 = y  (w[7]+w[2]+7w[0])*w[0] = w[1]

    let n: usize = 9;  // total variables
    let y: usize = 2;  // public: w[0]=1, w[1]=y

    // constraint 1: x * x = x²
    let a1 = vec![(2usize, ScalarField::from(1u64))];
    let b1 = vec![(2usize, ScalarField::from(1u64))];
    let c1 = vec![(3usize, ScalarField::from(1u64))];

    // constraint 2: x² * x = x³
    let a2 = vec![(3usize, ScalarField::from(1u64))];
    let b2 = vec![(2usize, ScalarField::from(1u64))];
    let c2 = vec![(4usize, ScalarField::from(1u64))];

    // constraint 3: x³ * x² = x⁵
    let a3 = vec![(4usize, ScalarField::from(1u64))];
    let b3 = vec![(3usize, ScalarField::from(1u64))];
    let c3 = vec![(5usize, ScalarField::from(1u64))];

    // constraint 4: (x⁵ + 3x³) * 1 = tmp1
    let a4 = vec![
        (5usize, ScalarField::from(1u64)),  // x⁵
        (4usize, ScalarField::from(3u64)),  // 3x³
    ];
    let b4 = vec![(0usize, ScalarField::from(1u64))];
    let c4 = vec![(6usize, ScalarField::from(1u64))];

    // constraint 5: (tmp1 + 2x²) * 1 = tmp2
    let a5 = vec![
        (6usize, ScalarField::from(1u64)),  // tmp1
        (3usize, ScalarField::from(2u64)),  // 2x²
    ];
    let b5 = vec![(0usize, ScalarField::from(1u64))];
    let c5 = vec![(7usize, ScalarField::from(1u64))];

    // constraint 6: (tmp2 + x + 7) * 1 = y
    let a6 = vec![
        (7usize, ScalarField::from(1u64)),  // tmp2
        (2usize, ScalarField::from(1u64)),  // x
        (0usize, ScalarField::from(7u64)),  // 7
    ];
    let b6 = vec![(0usize, ScalarField::from(1u64))];
    let c6 = vec![(1usize, ScalarField::from(1u64))];  // y

    let mut r = R1cs {
        num_variables: n,
        num_public_inputs: y,
        constraints: Vec::new(),
    };

    r.add_constraint(LinearCombination { terms: a1 }, LinearCombination { terms: b1 }, LinearCombination { terms: c1 });
    r.add_constraint(LinearCombination { terms: a2 }, LinearCombination { terms: b2 }, LinearCombination { terms: c2 });
    r.add_constraint(LinearCombination { terms: a3 }, LinearCombination { terms: b3 }, LinearCombination { terms: c3 });
    r.add_constraint(LinearCombination { terms: a4 }, LinearCombination { terms: b4 }, LinearCombination { terms: c4 });
    r.add_constraint(LinearCombination { terms: a5 }, LinearCombination { terms: b5 }, LinearCombination { terms: c5 });
    r.add_constraint(LinearCombination { terms: a6 }, LinearCombination { terms: b6 }, LinearCombination { terms: c6 });

    // witness: [1, y, x, x², x³, x⁵, tmp1, tmp2, _]
    let wit = vec![
        ScalarField::from(1u64),   // w[0] constant
        ScalarField::from(73u64),  // w[1] y (public)
        ScalarField::from(2u64),   // w[2] x
        ScalarField::from(4u64),   // w[3] x²
        ScalarField::from(8u64),   // w[4] x³
        ScalarField::from(32u64),  // w[5] x⁵
        ScalarField::from(56u64),  // w[6] tmp1 = x⁵ + 3x³
        ScalarField::from(64u64),  // w[7] tmp2 = tmp1 + 2x²
        ScalarField::from(73u64),  // w[8] tmp3 = tmp2 + x + 7 = y
    ];

    println!("=== R1CS ===");
    println!("is_satisfied: {}", r.is_satisfied(&wit));

    println!("\n=== QAP ===");
    let qp: Qap = r1cs_to_qap(&r);

    let mut a = Polynomial::zero();
    let mut b = Polynomial::zero();
    let mut c = Polynomial::zero();

    for (j, p) in qp.a_i.iter().enumerate() { a = a.add(&p.scale(wit[j])); }
    for (j, p) in qp.b_i.iter().enumerate() { b = b.add(&p.scale(wit[j])); }
    for (j, p) in qp.c_i.iter().enumerate() { c = c.add(&p.scale(wit[j])); }

    let p = a.mul(&b).sub(&c);
    let (_h, r_rem) = p.div_rem(&qp.z_i);
    println!("Remainder is zero: {}", r_rem.is_zero());

    println!("\n=== SETUP ===");
    let trusted_setup: ProvingKey = cr_proving_key(
        qp.z_i.degree() - 1,
        r.num_public_inputs,
        r.num_variables,
        &qp,
    );
    println!("Setup done ");

    println!("\n=== PROOF ===");
    let proof: Proof = generate_proof(
        &trusted_setup, &qp, &wit, &_h, r.num_public_inputs,
    );
    println!("Proof generated ");

    println!("\n=== VERIFY ===");
    let result = verify_proof(
        &trusted_setup.vk,
        &proof,
        &wit[..r.num_public_inputs],
    );
    println!("is the proof valid: {}", result);

    // -----------------------------------------------
    // Test with WRONG witness — should fail
    // -----------------------------------------------
    println!("\n=== VERIFY WITH WRONG WITNESS ===");
    let wrong_wit = vec![
        ScalarField::from(1u64),
        ScalarField::from(99u64),  
        ScalarField::from(2u64),
        ScalarField::from(4u64),
        ScalarField::from(8u64),
        ScalarField::from(32u64),
        ScalarField::from(56u64),
        ScalarField::from(64u64),
        ScalarField::from(73u64),
    ];
    let wrong_proof: Proof = generate_proof(
        &trusted_setup, &qp, &wrong_wit, &_h, r.num_public_inputs,
    );
    let wrong_result = verify_proof(
        &trusted_setup.vk,
        &wrong_proof,
        &wrong_wit[..r.num_public_inputs],
    );
    println!("Wrong proof should be false: {}", wrong_result);
}