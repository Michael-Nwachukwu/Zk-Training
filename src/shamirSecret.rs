// Import necessary crates and modules for random number generation, prime field operations, and polynomial operations.
use rand;
use ark_ff::PrimeField;
use polynomial::UnivariatePoly;
use ark_bn254::Fq;

// Define a struct to represent a point in a 2D space, where each coordinate is of type F.
#[derive(Debug)]
struct Point<F> {
    x: F,
    y: F,
}

// Function to generate shares for Shamir's Secret Sharing scheme.
fn generate_shares<F: PrimeField>(
    secret: i32,
    password: i32,
    threshold: usize,
    total_shares: usize,
) -> Vec<Point<F>> {
    // Assert that the threshold is greater than 0.
    assert!(threshold > 0, "Threshold must be greater than 0");
    // Assert that the threshold is not greater than the total number of shares.
    assert!(
        threshold <= total_shares,
        "Threshold greater than total shares"
    );

    // Initialize a random number generator.
    let mut rng = rand::thread_rng();
    // Initialize vectors to hold x and y coordinates of points.
    let mut xs: Vec<F> = Vec::new();
    let mut ys: Vec<F> = Vec::new();

    // Push the password as the first x coordinate and the secret as the first y coordinate.
    xs.push(F::from(password));
    ys.push(F::from(secret));

    // Generate additional points up to the threshold.
    for _ in 1..threshold {
        // Generate a random x coordinate.
        xs.push(F::rand(&mut rng));
        // Generate a random y coordinate.
        ys.push(F::rand(&mut rng));
    }

    // Interpolate a polynomial through the generated points.
    let poly = UnivariatePoly::interpolate(xs, ys);

    // Check if the degree of the interpolated polynomial matches the expected degree.
    if poly.degree() != (threshold - 1).try_into().unwrap() {
        panic!("Failed to interpolate polynomial");
    }

    // Initialize a vector to hold the shares.
    let mut shares = Vec::new();
    // Generate shares by evaluating the polynomial at random x coordinates.
    for _ in 1..=total_shares {
        let x = F::rand(&mut rng);
        let y = poly.evaluate(x);
        shares.push(Point { x, y });
    }

    // Return the generated shares.
    shares
}

// Function to reconstruct the secret from shares.
fn reconstruct_secret<F: PrimeField>(
    shares: &[Point<F>],
    password: i32,
    threshold: usize,
) -> Option<F> {
    // Check if the number of shares is less than the threshold.
    if shares.len() < threshold {
        return None;
    }

    // Prepare x and y coordinates of the shares for interpolation.
    let xs: Vec<F> = shares[0..threshold].iter().map(|p| p.x).collect();
    let ys: Vec<F> = shares[0..threshold].iter().map(|p| p.y).collect();

    // Interpolate a polynomial through the shares.
    let poly = UnivariatePoly::interpolate(xs, ys);

    // Evaluate the polynomial at the password to get the secret.
    Some(poly.evaluate(F::from(password)))
}

fn main() {
    // Example usage of generate_shares function.
    generate_shares::<Fq>(500, 25, 4, 10);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_basic_sharing_and_reconstruction() {
        let secret = 42;
        let threshold = 3;
        let total_shares = 5;
        let password = 25;

        let shares = generate_shares::<Fq>(secret, password, threshold, total_shares);
        assert_eq!(shares.len(), total_shares);

        let reconstructed = reconstruct_secret(&shares[..threshold], password, threshold);
        assert_eq!(reconstructed, Some(Fq::from(secret)));
    }

    #[test]
    fn test_reconstruction_with_different_share_combinations() {
        let secret = 123;
        let threshold = 3;
        let total_shares = 5;
        let password = 25;

        let shares = generate_shares::<Fq>(secret, password, threshold, total_shares);

        let reconstructed1 = reconstruct_secret(&shares[1..4], password, threshold);
        let reconstructed2 = reconstruct_secret(&shares[2..5], password, threshold);

        assert_eq!(reconstructed1, Some(Fq::from(secret)));
        assert_eq!(reconstructed2, Some(Fq::from(secret)));
    }

    #[test]
    fn test_insufficient_shares() {
        let secret = 42;
        let threshold = 3;
        let total_shares = 5;
        let password = 25;

        let shares = generate_shares::<Fq>(secret, password, threshold, total_shares);
        let reconstructed = reconstruct_secret(&shares[..2], password, threshold);
        assert_eq!(reconstructed, None);
    }

    #[test]
    #[should_panic]
    fn test_invalid_threshold() {
        generate_shares::<Fq>(42, 15, 0, 5);
    }
}