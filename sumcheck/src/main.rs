use multilinear::multilinear::MultilinearPolynomial;
use sumcheck::fiat_shamir::{
    Transcript,
    FiatShamirTranscriptInterface
};
use ark_ff::{PrimeField, BigInteger};
use std::marker::PhantomData;

// Define a struct to represent a sumcheck prover that is generating the proof
pub struct Prover<F: PrimeField> {
    pub initial_poly: MultilinearPolynomial<F>,
    pub initial_claimed_sum: F,
    pub transcript: Transcript,
    pub uni_poly_for_each_round: Vec<MultilinearPolynomial<F>>
}

fn main() {
    println!("Hello, world!");
}

// A strict that represents a sumcheck proof
pub struct SumcheckProof<F: PrimeField> {
    pub initial_claimed_sum: F, // type of F
    pub initial_poly: MultilinearPolynomial<F>, // type of Multilinear poly
    pub uni_poly_for_each_round: Vec<MultilinearPolynomial<F>>, // vector of univariate polynomials to store reduced poly at each round
}

// Define a struct to represent a sumcheck verifier
pub struct Verifier<F: PrimeField> {
    pub transcript: Transcript,
    _phantom: PhantomData<F>
}

impl<F: PrimeField>Prover<F> {
    pub fn new(initial_poly_evaluation: &Vec<F>) -> Self {
        let polynomial = MultilinearPolynomial::new(&initial_poly_evaluation.clone());
        let transcript = Transcript::new();

        Prover {
            initial_poly: polynomial,
            initial_claimed_sum: initial_poly_evaluation.iter().sum(),
            transcript: transcript,
            uni_poly_for_each_round: Vec::new(),
        }
    }

    pub fn prove(&mut self) -> SumcheckProof<F> {
        // commit the initial polynomial to the transcript as bytes array
        self.transcript.append(&self.initial_poly.convert_to_bytes());
        self.transcript.append(&f_to_bytes(self.initial_claimed_sum));

        let mut current_polynomial = self.initial_poly.clone();

        for _ in 0..self.initial_poly.no_of_vars {
            
            let univariate_poly_values = split_and_reduce(&current_polynomial.evaluated_values);

            // defined a univariate polynomial for this round
            let univariate_polynomial = MultilinearPolynomial::new(&univariate_poly_values);

            // convert the univariate polynomial to bytes to append to our transcript
            let univariate_polynomial_in_bytes = univariate_polynomial.convert_to_bytes();

            // add the univariate polynomial for this round to the vector in sumcheck proof
            self.uni_poly_for_each_round.push(univariate_polynomial);
            
            // commit the univariate polynomial to the transcript as bytes array
            self.transcript.append(&univariate_polynomial_in_bytes);

            
            // Get random challenge <- from Transcript
            let random_challenge: F = self.transcript.random_challenge_as_field_element();

            // Partial evaluate current polynomial using the random_challenge
            current_polynomial = MultilinearPolynomial::partial_evaluate(&current_polynomial.evaluated_values.clone(), 0, random_challenge);
        }

        SumcheckProof {
            initial_claimed_sum: self.initial_claimed_sum,
            initial_poly: self.initial_poly.clone(),
            uni_poly_for_each_round: self.uni_poly_for_each_round.clone(),
        }
    }
}


impl <F: PrimeField>Verifier<F> {
    pub fn new() -> Self {
        Verifier {
            transcript: Transcript::new(),
            _phantom: PhantomData,
        }
    }

    pub fn verify(&mut self, proof: SumcheckProof<F>) -> bool {

        // Check if the number of univariate polynomials in the proof is equal to the number of variables in the initial polynomial
        if proof.uni_poly_for_each_round.len() != proof.initial_poly.no_of_vars {
            return false;
        }

        // let the current_sum be the initial claimed sum from the sent proof
        let mut current_claim_sum = proof.initial_claimed_sum;

        // commit the initial polynomial to the transcript as bytes array
        self.transcript.append(&proof.initial_poly.convert_to_bytes());

        // commit the initial claimed sum to the transcript as bytes using the f_to_bytes function
        self.transcript.append(&f_to_bytes(proof.initial_claimed_sum));

        // creates a new mutable vector called challenges that will store field elements of type F
        // pre-allocates space for a vector that will space equal to the number of univariate polynomials in the proof
        let mut challenges: Vec<F> = Vec::with_capacity(proof.uni_poly_for_each_round.len());

        // Loop through the vector of univariate polynomials
        for i in 0..proof.uni_poly_for_each_round.len() {
            // creates a vector containing just the field element 0
            let evaluation_at_zero = vec![F::zero()];
            // creates a vector containing just the field element 1
            let evaluation_at_one = vec![F::one()];

            // the sum of the univariate polynomial evaluated at 0 and 1 should equal the current claimed sum.
            if proof.uni_poly_for_each_round[i].evaluate(&evaluation_at_zero) + proof.uni_poly_for_each_round[i].evaluate(&evaluation_at_one) != current_claim_sum {
                return false;
            }

            // commit the univariate polynomial to the transcript as bytes array
            self.transcript.append(&proof.uni_poly_for_each_round[i].convert_to_bytes());

            // Get random challenge <- from Transcript
            let challenge: F = self.transcript.random_challenge_as_field_element();
            challenges.push(challenge);

            // update the current claimed sum
            current_claim_sum = proof.uni_poly_for_each_round[i].evaluate(&vec![challenge])
        }

        let final_evaluation = proof.initial_poly.evaluate(&challenges);

        // Oracle Check
        final_evaluation == current_claim_sum
    }
}

pub fn F_to_bytes<F: PrimeField>(field_element: F) -> Vec<u8> {
    field_element.into_bigint().to_bytes_be()
}

pub fn split_and_reduce<F: PrimeField>(polynomial_evaluated_values: &Vec<F>) -> Vec<F> {
    let mut univariate_polynomial: Vec<F> = Vec::with_capacity(2);

    let mid = polynomial_evaluated_values.len() / 2;
    let (left, right) = polynomial_evaluated_values.split_at(mid);

    let left_sum: F = left.iter().sum();
    let right_sum: F = right.iter().sum();

    univariate_polynomial.push(left_sum);
    univariate_polynomial.push(right_sum);

    univariate_polynomial
}


#[cfg(test)]
mod test {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_prover_init() {
        let evaluated_values = vec![Fq::from(0), Fq::from(0), Fq::from(3), Fq::from(8)];
        let prover = Prover::new(&evaluated_values);

        assert_eq!(prover.initial_claimed_sum, Fq::from(11));
        assert_eq!(prover.initial_poly.evaluated_values, evaluated_values);
    }
}
