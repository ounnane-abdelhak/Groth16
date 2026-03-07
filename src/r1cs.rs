use crate::field::ScalarField;

#[derive(Debug, Clone, Default)]
pub struct LinearCombination {
    pub terms: Vec<(usize, ScalarField)>,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub a: LinearCombination,
    pub b: LinearCombination,
    pub c: LinearCombination,
}

#[derive(Debug, Clone)]
pub struct R1cs {
    pub num_variables: usize,
    pub num_public_inputs: usize,
    pub constraints: Vec<Constraint>,
}
impl LinearCombination {
pub fn zero() -> Self {
    Self::default() 
}

pub fn term(mut self,idx :usize, coeff:ScalarField) -> Self{
    self.terms.push((idx,coeff));
    self
}

pub fn evaluate(&self,witness:&[ScalarField]) -> ScalarField {
    self.terms
        .iter()
        .map(|(idx, coeff)| *coeff * witness[*idx])
        .sum()
}

pub fn coefficient_of(&self,variable_index : usize) -> ScalarField {
    self.terms
        .iter()
        .filter(|(idx, _)| *idx == variable_index )
        .map(|(_,coeff)| *coeff)
        .sum()
}
}

impl R1cs {
    pub fn add_constraint(&mut self,a:LinearCombination, b:LinearCombination, c:LinearCombination)  {
        let ac=a.clone();
        let bc=b.clone();
        let cc=c.clone();

        self.constraints.push(Constraint{a:ac,b:bc,c:cc});
    }
    pub fn new(num_public_inputs:usize, num_private_inputs:usize) ->Self{
        Self {
            num_variables: 1 + num_public_inputs + num_private_inputs,
            num_public_inputs,
            constraints: Vec::new(),
        }
    }

    pub fn is_satisfied(&self,witness:&[ScalarField]) -> bool {

        if witness.len()!=self.num_variables {return false;}

        if witness[0] != ScalarField::from(1u64) {return false;}

        self.constraints.iter().all(|constraint| {
            constraint.a.evaluate(witness) * constraint.b.evaluate(witness)
                == constraint.c.evaluate(witness)
        })
    }

}
