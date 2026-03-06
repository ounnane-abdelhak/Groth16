use crate::field::ScalarField;
use crate::field::fr_from_u64;


#[derive(Debug, Clone, Default)]
pub struct Polynomial{
 pub coeff:Vec<ScalarField>  
}

impl Polynomial {


    pub fn new(coeffs: Vec<ScalarField>) -> Self {
        let mut coeff = coeffs;
        if coeff.len()==0 {return Self { coeff:vec![ScalarField::from(0u64)] }}; 
        while coeff.last()==Some(&ScalarField::from(0u64)) && coeff.len()>1 {
            coeff.pop() ;
        }
        Self { coeff }
    }


    pub fn degree(&self)-> usize{
        self.coeff.len()-1
    }

    pub fn zero() -> Self {
        Self{coeff:vec![fr_from_u64(0u64)]}
    }

    pub fn one() -> Self {
        Self{coeff:vec![fr_from_u64(1u64)]}
    }

    pub fn is_zero(&self)-> bool{
        self.coeff.iter()
                  .all(|val| *val == fr_from_u64(0u64))

    }

    pub fn eval(&self, x: ScalarField) -> ScalarField {
        self.coeff.iter().rev()
            .fold(ScalarField::from(0u64), |acc, coeff| acc * x + coeff)
    }

    pub fn add(&self,x:&Self) -> Self{
        let len = self.coeff.len().max(x.coeff.len());
        let res = (0..len)
                                .map(|i| {
            let a = self.coeff.get(i).cloned().unwrap_or(ScalarField::from(0u64));
            let b = x.coeff.get(i).cloned().unwrap_or(ScalarField::from(0u64));
            a + b
        }).collect::<Vec<ScalarField>>();
        Self::new(res)

    }
    pub fn sub(&self,x:&Self) -> Self{
        let len = self.coeff.len().max(x.coeff.len());
        let res = (0..len)
                                .map(|i| {
            let a = self.coeff.get(i).cloned().unwrap_or(ScalarField::from(0u64));
            let b = x.coeff.get(i).cloned().unwrap_or(ScalarField::from(0u64));
            a - b
        }).collect::<Vec<ScalarField>>();
        Self::new(res)
    }
    pub fn scale(&self,val:ScalarField)->Self {
        let res=self.coeff.iter()
                  .map(|num | *num * val )
                  .collect();
        Self::new(res)
    }

    pub fn mul(&self,x:&Self) -> Self{
        let len = self.coeff.len()+x.coeff.len()-1;
        let mut res= vec![ScalarField::from(0u64);len];
        for (i, a) in self.coeff.iter().enumerate() {
        for (j, b) in x.coeff.iter().enumerate() {
            res[i+j]+=a*b;
        }
    }
        Self::new(res)
    }

 pub fn div_rem(&self, divisor: &Self) -> (Self, Self) {
        assert!(!divisor.is_zero(), "division by zero polynomial");

        if self.is_zero() || self.degree() < divisor.degree() {
            return (Self::zero(), self.clone());
        }

        let mut rem = self.coeff.clone();
        let mut quo = vec![ScalarField::from(0u64); self.degree() - divisor.degree() + 1];
        let ddeg = divisor.degree();
        let dlead = divisor.coeff[ddeg];

        while rem.len() >= divisor.coeff.len() && !(rem.len() == 1 && rem[0] == ScalarField::from(0u64))
        {
            let rdeg = rem.len() - 1;
            let shift = rdeg - ddeg;
            let cf = rem[rdeg] / dlead;

            quo[shift] += cf;
            for (i, dc) in divisor.coeff.iter().enumerate() {
                rem[shift + i] -= cf * *dc;
            }

            while rem.len() > 1 && rem.last() == Some(&ScalarField::from(0u64)) {
                rem.pop();
            }
        }

        (Self::new(quo), Self::new(rem))
    }

}