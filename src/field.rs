use ark_bn254::{Bn254, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ff::{UniformRand};
use ark_std::rand::Rng;


pub type Pairing = Bn254;
pub type ScalarField = Fr;
pub type G1 = G1Projective;
pub type G2 = G2Projective;
pub type G1Aff = G1Affine;
pub type G2Aff = G2Affine;

pub fn fr_from_u64(x: u64) -> ScalarField {
    ScalarField::from(x)
}

pub fn random_fr<R: Rng>(rng: &mut R) -> ScalarField {
    loop {
        let r = ScalarField::rand(rng);
        if r != ScalarField::from(0u64) {
            return r;
        }
    }
}


