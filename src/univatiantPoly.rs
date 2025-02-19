struct Polynomial {
    terms: Vec<(usize, usize)>,
}

impl Polynomial {
    fn new(terms: Vec<(usize, usize)>) -> Self {
        Polynomial { terms }
    }

    fn evaluate(&self, x: usize) -> usize {
        self.terms.iter().map(|(coeff, exp)| coeff * x.pow(*exp as u32)).sum()
    }

    fn degree(&self) -> usize {
        self.terms.iter().map(|(_, exp)| exp).max().cloned().unwrap_or(0)
    }
}

// Example usage
fn main() {
    let poly = Polynomial::new(vec![(2, 1), (5, 0)]);
    println!("Degree: {}", poly.degree());
    println!("Evaluation at x=3: {}", poly.evaluate(3));
}

// 2x + 5
// 4x pow3 + 8 pow2