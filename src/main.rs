mod field;
mod r1cs;
mod qap;
mod poly;
mod setup;
mod proof;
mod verify;

use crate::verify::verify_proof;
use crate::field::ScalarField;
use crate::poly::Polynomial;
use crate::qap::r1cs_to_qap;
use crate::qap::Qap;
use crate::r1cs::{Constraint, LinearCombination, R1cs};
use crate::setup::{cr_proving_key,ProvingKey};
use crate::proof::{Proof,generate_proof};

fn main() {
    // witness order: [1, out, x, x², x³, tmp]
    // index:          0   1    2   3    4    5
    // for x=3: out=35, x²=9, x³=27, tmp=30
    let n: usize = 6;
    let y: usize = 2; // public includes constant + out

    // 1) x * x = x²
    let a1 = vec![(2usize, ScalarField::from(1u64))];
    let b1 = vec![(2usize, ScalarField::from(1u64))];
    let c1 = vec![(3usize, ScalarField::from(1u64))];

    // 2) x² * x = x³
    let a2 = vec![(3usize, ScalarField::from(1u64))];
    let b2 = vec![(2usize, ScalarField::from(1u64))];
    let c2 = vec![(4usize, ScalarField::from(1u64))];

    // 3) (x³ + x) * 1 = tmp
    let a3 = vec![
        (4usize, ScalarField::from(1u64)), // x³
        (2usize, ScalarField::from(1u64)), // x
    ];
    let b3 = vec![(0usize, ScalarField::from(1u64))]; // 1
    let c3 = vec![(5usize, ScalarField::from(1u64))]; // tmp

    // 4) (tmp + 5) * 1 = out
    let a4 = vec![
        (5usize, ScalarField::from(1u64)), // tmp
        (0usize, ScalarField::from(5u64)), // 5*1
    ];
    let b4 = vec![(0usize, ScalarField::from(1u64))]; // 1
    let c4 = vec![(1usize, ScalarField::from(1u64))]; // out

    let constraints: Vec<Constraint> = Vec::new();
    let mut r = R1cs {
        num_variables: n,
        num_public_inputs: y,
        constraints,
    };

    r.add_constraint(
        &LinearCombination { terms: a1 },
        &LinearCombination { terms: b1 },
        &LinearCombination { terms: c1 },
    );
    r.add_constraint(
        &LinearCombination { terms: a2 },
        &LinearCombination { terms: b2 },
        &LinearCombination { terms: c2 },
    );
    r.add_constraint(
        &LinearCombination { terms: a3 },
        &LinearCombination { terms: b3 },
        &LinearCombination { terms: c3 },
    );
    r.add_constraint(
        &LinearCombination { terms: a4 },
        &LinearCombination { terms: b4 },
        &LinearCombination { terms: c4 },
    );

    // [1, out, x, x², x³, tmp]
    let wit = vec![
        ScalarField::from(1u64),  // constant
        ScalarField::from(35u64), // out (public)
        ScalarField::from(3u64),  // x
        ScalarField::from(9u64),  // x²
        ScalarField::from(27u64), // x³
        ScalarField::from(30u64), // tmp
    ];

    println!("is_satisfied: {}", r.is_satisfied(&wit));

    let qp: Qap = r1cs_to_qap(&r);
    println!("{:#?}", qp);

    let mut a = Polynomial::zero();
    let mut b = Polynomial::zero();
    let mut c = Polynomial::zero();

    for (j, p) in qp.a_i.iter().enumerate() {
        a = a.add(&p.scale(wit[j]));
    }
    for (j, p) in qp.b_i.iter().enumerate() {
        b = b.add(&p.scale(wit[j]));
    }
    for (j, p) in qp.c_i.iter().enumerate() {
        c = c.add(&p.scale(wit[j]));
    }

    let p = a.mul(&b).sub(&c);
    let (_h, r_rem) = p.div_rem(&qp.z_i);

    println!("Remainder should be zero: {:#?}", r_rem);

    let trusted_setup:ProvingKey = cr_proving_key(qp.z_i.degree()-1,r.num_public_inputs,r.num_variables,&qp);
    println!("{:#?}",trusted_setup);

    let proof:Proof = generate_proof(&trusted_setup, &qp, &wit, &_h, r.num_public_inputs);

    println!("{:#?}",proof);

    println!("is the proof true :{:#?}",verify_proof(&trusted_setup.vk, &proof, &wit[..r.num_public_inputs]));

}
