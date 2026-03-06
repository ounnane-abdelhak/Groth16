use crate::field::*;
use crate::poly::Polynomial;
use crate::setup::ProvingKey;
use crate::qap::Qap;
use ark_std::test_rng;
use ark_ec::VariableBaseMSM;
use ark_ec::CurveGroup;


#[derive(Debug, Clone)]
pub struct Proof{
    pub a:G1Aff,
    pub b:G2Aff,
    pub c:G1Aff
}

fn gen_random()->(ScalarField,ScalarField){
    let mut rng = test_rng();
    let r = random_fr(&mut rng);
    let s = random_fr(&mut rng);

    (r,s)

}

pub fn generate_proof(setup:&ProvingKey,qp:&Qap,wit:&[ScalarField],hpoly:&Polynomial,num_public_inputs:usize)-> Proof {
    assert_eq!(wit.len(), qp.a_i.len());
    assert_eq!(qp.a_i.len(), qp.b_i.len());
    assert_eq!(qp.b_i.len(), qp.c_i.len());
    assert_eq!(setup.l_query_pv.len(), wit.len() - num_public_inputs);
    let  a:G1Aff;
    let  b2:G2Aff;
    let  b1:G1Aff;
    let  c:G1Aff;
    let mut res1 :Vec<G1Aff> = vec![G1Aff::identity();qp.a_i.len()];
    let mut res2 :Vec<G2Aff> = vec![G2Aff::identity();qp.a_i.len()];

    let (_r,_s) = gen_random();

    for (i,j) in qp.a_i.iter().enumerate(){
        let coeffs = &j.coeff;
        res1[i] = G1::msm(&setup.g1_query[..coeffs.len()], coeffs).unwrap().into_affine();

    }
    a = (setup.vk.alpha_g1 + setup.delta_g1 *_r + G1::msm(&res1, wit).unwrap()).into_affine();

    for (i,j) in qp.b_i.iter().enumerate(){
        let coeffs = &j.coeff;
        res2[i]=(G2::msm(&setup.g2_query[..coeffs.len()], coeffs).unwrap()).into_affine();
    }

    b2 = (setup.vk.beta_g2 + setup.vk.delta_g2 *_s + G2::msm(&res2, wit).unwrap()).into_affine();

    for (i,j) in qp.b_i.iter().enumerate(){
        let coeffs = &j.coeff;
        res1[i]=(G1::msm(&setup.g1_query[..coeffs.len()], coeffs).unwrap()).into_affine();
    }
    b1 = (setup.beta_g1 + setup.delta_g1 *_s + G1::msm(&res1, wit).unwrap()).into_affine();

let pvwit: Vec<ScalarField> = wit[num_public_inputs..].to_vec();

    c = (a*_s + b1*_r-setup.delta_g1 * (_r*_s ) + G1::msm(&setup.h_query, &hpoly.coeff).unwrap() + G1::msm(&setup.l_query_pv, &pvwit).unwrap()).into_affine();

    Proof { a, b:b2, c }
}