use crate::field::*;
use crate::qap::Qap;
use ark_ec::CurveGroup;
use ark_ec::PrimeGroup;
use ark_std::test_rng;


#[derive(Debug, Clone)]
pub struct VerifyingKey {
    pub alpha_g1: G1Aff,         
    pub beta_g2: G2Aff,           
    pub gamma_g2: G2Aff,      
    pub delta_g2: G2Aff,      
    pub gamma_abc_g1: Vec<G1Aff>,
}
#[derive(Debug, Clone)]
pub struct ProvingKey {
    pub vk: VerifyingKey,         
    pub beta_g1: G1Aff,         
    pub delta_g1: G1Aff,        
    pub g1_query: Vec<G1Aff>,  
    pub g2_query: Vec<G2Aff>,   
    pub h_query: Vec<G1Aff>,     
    pub l_query_pv: Vec<G1Aff>,    
}
#[derive(Debug, Clone)]
pub struct Randoms{
    pub alpha: ScalarField,
    pub beta: ScalarField,
    pub tau: ScalarField,
    pub gamma: ScalarField,
    pub delta: ScalarField,
}


fn gen_randoms()->Randoms{
    let mut rng = test_rng();
    let alpha = random_fr(&mut rng);
    let beta = random_fr(&mut rng);
    let tau = random_fr(&mut rng);  
    let gamma= random_fr(&mut rng);
    let delta = random_fr(&mut rng);

    Randoms {alpha,beta,tau,gamma,delta}
}

fn cr_verifying_key(num_public_inputs:usize,randoms:&Randoms,l_query_pb:&[ScalarField])->VerifyingKey{
    assert!(l_query_pb.len()==num_public_inputs,"not valid inputs length");
    let g1 = G1::generator(); 
    let g2 = G2::generator();
    let alpha_g1: G1Aff = (g1*randoms.alpha).into_affine();      
    let beta_g2: G2Aff = (g2*randoms.beta).into_affine();           
    let gamma_g2: G2Aff = (g2*randoms.gamma).into_affine();     
    let delta_g2: G2Aff = (g2*randoms.delta).into_affine();      
    let mut gamma_abc_g1: Vec<G1Aff>=vec![G1Aff::identity();num_public_inputs];
    for (j,i) in l_query_pb.iter().enumerate(){
        gamma_abc_g1[j]=(g1*i).into_affine();
    }
    VerifyingKey{alpha_g1,beta_g2,gamma_g2, delta_g2, gamma_abc_g1}
}

pub fn cr_proving_key(polys_deg:usize,num_public_inputs:usize,num_input:usize,qp:&Qap)->ProvingKey{
    let randoms=gen_randoms();
    assert!(polys_deg>0);
    let g1 = G1::generator(); 
    let g2 = G2::generator();
    let  vk: VerifyingKey;        
    let  beta_g1: G1Aff = (g1*randoms.beta).into_affine();         
    let  delta_g1: G1Aff = (g1*randoms.delta).into_affine();         
    let mut g1_query: Vec<G1Aff> = vec![G1Aff::identity();polys_deg+1];  
    let mut g2_query: Vec<G2Aff> = vec![G2Aff::identity();polys_deg+1];   
    let mut h_query: Vec<G1Aff> = vec![G1Aff::identity();polys_deg];     
    let mut l_query_pv: Vec<G1Aff> = vec![G1Aff::identity();num_input-num_public_inputs]; 
    let mut l_query_pb: Vec<ScalarField> = vec![ScalarField::from(0u64);num_public_inputs]; 
    let mut values=vec![ScalarField::from(0u64);polys_deg+1];
    values[0]=ScalarField::from(1u64);
    g1_query[0]= (g1*values[0]).into_affine();
    g2_query[0]= (g2*values[0]).into_affine();
    let val=qp.z_i.eval(randoms.tau)/randoms.delta;
    for i in 1..=polys_deg{
        values[i]=randoms.tau*values[i-1];
        g1_query[i]= (g1*values[i]).into_affine();
        g2_query[i]= (g2*values[i]).into_affine();
        h_query[i-1]=(g1*values[i-1]*val).into_affine();
    }
    for i in 0..(num_input-num_public_inputs){
        l_query_pv[i]=(g1*((randoms.beta * qp.a_i[i+num_public_inputs].eval(randoms.tau)  +randoms.alpha * qp.b_i[i+num_public_inputs].eval(randoms.tau) + qp.c_i[i+num_public_inputs].eval(randoms.tau))/randoms.delta)).into_affine();
    }
    for i in 0..(num_public_inputs){
        l_query_pb[i]=(randoms.beta * qp.a_i[i].eval(randoms.tau)  +randoms.alpha * qp.b_i[i].eval(randoms.tau) + qp.c_i[i].eval(randoms.tau))/randoms.gamma;
    }

    vk = cr_verifying_key(num_public_inputs,&randoms,&l_query_pb);

    ProvingKey{vk,       
     beta_g1,        
     delta_g1,       
     g1_query ,
     g2_query  , 
     h_query   ,
     l_query_pv }
}



// pub struct VerifyingKey {
//     pub alpha_g1: G1Affine,          // [α]₁
//     pub beta_g2: G2Affine,           // [β]₂
//     pub gamma_g2: G2Affine,          // [γ]₂
//     pub delta_g2: G2Affine,          // [δ]₂
//     pub gamma_abc_g1: Vec<G1Affine>, // [Ψ₁..Ψₗ]₁ public inputs
// }

// pub struct ProvingKey {
//     pub vk: VerifyingKey,            // prover also needs vk
//     pub beta_g1: G1Affine,           // [β]₁
//     pub delta_g1: G1Affine,          // [δ]₁
//     pub g1_query: Vec<G1Affine>,     // srsG1
//     pub g2_query: Vec<G2Affine>,     // srsG2 
//     pub h_query: Vec<G1Affine>,      // H terms t(τ)/δ
//     pub l_query: Vec<G1Affine>,      // [Ψₗ₊₁..Ψₘ]₁ private inputs
// }

