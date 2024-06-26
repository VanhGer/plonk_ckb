use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use crate::types::G1Point;

/// Commitment contains result
#[derive(Debug, Clone, PartialEq, Eq, CanonicalDeserialize, CanonicalSerialize)]
pub struct KzgCommitment(pub G1Point);

impl KzgCommitment {
    /// A reference to the inner `G1Point` contained within the commitment.
    pub fn inner(&self) -> &G1Point {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;

    use ark_bls12_381::Fr;
    use ark_ec::{AffineRepr, CurveGroup};
    use ark_ff::One;
    use ark_poly::{DenseUVPolynomial, Polynomial};

    use crate::scheme::KzgScheme;
    use crate::srs::Srs;
    use crate::types::{G1Point, Poly};

    #[test]
    /// Tests the commitment functionality in the KZG scheme.
    ///
    /// This test verifies the correctness of committing to a polynomial,
    /// opening the commitment, and verifying the opening.
    fn commit() {
        let secret = Fr::from(2);
        let srs = Srs::new_from_secret(secret, 10);
        let scheme = KzgScheme::new(srs);
        let poly = Poly::from_coefficients_slice(&[1.into(), 2.into(), 3.into()]);
        let commitment = scheme.commit(&poly);
        let d = Fr::one();

        assert_eq!(poly.evaluate(&d), 6.into());

        assert_eq!(
            commitment.0,
            G1Point::generator()
                .mul(poly.evaluate(&secret))
                .into_affine()
        );
        let opening = scheme.open(poly, d);
        assert!(scheme.verify(&commitment, &opening, d));
    }

    #[test]
    /// Tests the scalar multiplication of commitments.
    ///
    /// This test validates the correctness of scalar multiplying a commitment
    /// by a factor in the KZG scheme.
    fn scalar_mul() {
        let srs = Srs::new(5);
        let scheme = KzgScheme::new(srs);
        let coeffs = [1, 2, 3, 4, 5].map(|e| Fr::from(e));
        let poly = Poly::from_coefficients_slice(&coeffs);
        let commit1 = scheme.commit(&poly);
        let factor = Fr::from(9);
        let poly2 = poly.mul(factor);
        let commit2 = scheme.commit(&poly2);
        assert_eq!(commit1 * factor, commit2);
    }
}
