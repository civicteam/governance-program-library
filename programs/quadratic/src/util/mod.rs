use std::ops::Mul;
use crate::state::QuadraticCoefficients;
use rug::Float;

pub fn convert_vote(input_voter_weight: u64, coefficients: &QuadraticCoefficients) -> u64 {
    let big_input_voter_weight = Float::from(input_voter_weight);

    let a = coefficients.a;
    let b = coefficients.b;
    let c = coefficients.c;

    // calculate a * x^0.5
    let first_term = big_input_voter_weight.clone().sqrt().mul(a);

    let full_term = first_term + big_input_voter_weight.mul(b) + c;

    full_term.to_u64().unwrap();
}
