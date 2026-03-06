use crate::r1cs::R1cs;
use crate::poly::Polynomial;
use crate::field::ScalarField;

#[derive(Debug, Clone)]
pub struct Qap {
    pub domain: Vec<ScalarField>,
    pub a_i:Vec<Polynomial>,
    pub b_i:Vec<Polynomial>,
    pub c_i:Vec<Polynomial>,
    pub z_i:Polynomial
}

fn interpolation(xs: &[ScalarField], ys: &[ScalarField]) -> Polynomial {
    assert_eq!(xs.len(), ys.len(), "interpolation points mismatch");
    assert!(!xs.is_empty(), "interpolation points must be non-empty");

    let mut acc = Polynomial::zero();

    for i in 0..xs.len() {
        let mut basis = Polynomial::one();
        let mut denom = ScalarField::from(1u64);

        for j in 0..xs.len() {
            if i == j {
                continue;
            }
            let factor = Polynomial::new(vec![-xs[j], ScalarField::from(1u64)]);
            basis = basis.mul(&factor);
            denom *= xs[i] - xs[j];
        }

        assert!(denom != ScalarField::from(0u64), "duplicate x points");
        let li_scaled = basis.scale(ys[i] / denom);
        acc = acc.add(&li_scaled);
    }

    acc
}


pub fn r1cs_to_qap(r1cs: &R1cs) -> Qap{
    let n = r1cs.num_variables;
    let m = r1cs.constraints.len();
    assert!(m > 0);
    let mut domain:Vec<ScalarField>=vec![ScalarField::from(0u64); m];
    let mut a_i:Vec<Polynomial>=vec![Polynomial::zero(); n];
    let mut b_i:Vec<Polynomial>=vec![Polynomial::zero(); n];
    let mut c_i:Vec<Polynomial>=vec![Polynomial::zero(); n];
    let mut z_i:Polynomial=Polynomial::one();
    for i in 0..m {
        domain[i]=ScalarField::from((i+1) as u64);
    }

    let mut va_i: Vec<Vec<ScalarField>> = vec![vec![ScalarField::from(0u64); m]; n];
    let mut vb_i: Vec<Vec<ScalarField>> = vec![vec![ScalarField::from(0u64); m]; n];
    let mut vc_i: Vec<Vec<ScalarField>> = vec![vec![ScalarField::from(0u64); m]; n];
    for (i,mat) in r1cs.constraints.iter().enumerate(){
        for  j in 0..n {
            va_i[j][i]=mat.a.coefficient_of(j);
            vb_i[j][i]=mat.b.coefficient_of(j);
            vc_i[j][i]=mat.c.coefficient_of(j);
        }

    }
     for  j in 0..n {
        a_i[j]=interpolation(&domain,&va_i[j]);
        b_i[j]=interpolation(&domain,&vb_i[j]);
        c_i[j]=interpolation(&domain,&vc_i[j]);
    }
    for v in domain.iter(){
        let  fac=Polynomial::new(vec![-*v,
            ScalarField::from(1u64)]);
        z_i=z_i.mul(&fac)
    }

    Qap{domain,a_i,b_i,c_i,z_i}
}

