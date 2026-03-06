use crate::field::*;
use crate::setup::VerifyingKey;
use crate::proof::Proof;
use crate::field::Pairing;
use ark_ec::pairing::Pairing as ArkPairing; 
use ark_ec::VariableBaseMSM;
use ark_ec::CurveGroup;


pub fn verify_proof(vk: &VerifyingKey, proof: &Proof,public_input:&[ScalarField]) -> bool{
    if public_input.len() != vk.gamma_abc_g1.len() { return false;}

    let x=G1::msm(&vk.gamma_abc_g1, public_input).unwrap().into_affine();
    let lhs=Pairing::pairing(proof.a,proof.b);
    let rhs=Pairing::pairing(vk.alpha_g1,vk.beta_g2)+Pairing::pairing(proof.c,vk.delta_g2)+Pairing::pairing(x,vk.gamma_g2);
    println!("{:#?}",lhs);
    println!("{:#?}",rhs);
    lhs == rhs 
    
}
