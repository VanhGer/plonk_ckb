use alloc::vec::Vec;
use core::ops::{Add, Mul, Neg, Sub};

use ark_bls12_381::{Fr, G1Affine, G2Affine};
use ark_poly::univariate::DensePolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

/// Type alias for G1 affine points.
pub type G1Point = G1Affine;

/// Type alias for G2 affine points.
pub type G2Point = G2Affine;

/// A structured reference string (SRS) used in the KZG commitment scheme.
#[derive(Debug, Clone, CanonicalDeserialize, CanonicalSerialize)]
pub struct Srs {
    g1_points: Vec<G1Point>,
    g2: G2Point,
    g2s_point: G2Point,
}

impl Srs {
    /// Returns the G2 point from the SRS.
    pub fn g2(&self) -> G2Point {
        self.g2
    }

    /// Returns the second G2 point from the SRS.
    pub fn g2s(&self) -> G2Point {
        self.g2s_point
    }
}

/// Common preprocessed input structure containing various polynomials and scalars.
#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct CommonPreprocessedInput {
    pub n: usize,
    pub k1: Fr,
    pub k2: Fr,
    pub com_q_lx: KzgCommitment,
    pub com_q_rx: KzgCommitment,
    pub com_q_mx: KzgCommitment,
    pub com_q_ox: KzgCommitment,
    pub com_q_cx: KzgCommitment,
    pub com_s_sigma_1: KzgCommitment,
    pub com_s_sigma_2: KzgCommitment,
    pub com_s_sigma_3: KzgCommitment,
    pub pi_x: DensePolynomial<Fr>,
}

/// A struct representing the KZG commitment scheme.
pub struct KzgScheme(Srs);

impl KzgScheme {
    /// Creates a new `KzgScheme` with the given SRS.
    ///
    /// # Arguments
    ///
    /// * `srs` - A structured reference string.
    ///
    /// # Returns
    ///
    /// A new `KzgScheme`.
    pub fn new(srs: Srs) -> Self {
        Self(srs)
    }

    /// Commits to a scalar using the SRS.
    ///
    /// # Arguments
    ///
    /// * `para` - The scalar to commit to.
    ///
    /// # Returns
    ///
    /// A KZG commitment to the scalar.
    pub fn commit_para(&self, para: Fr) -> KzgCommitment {
        let g1_0 = *self.0.g1_points.first().unwrap();
        let commitment = g1_0.mul(para).into();
        KzgCommitment(commitment)
    }

    pub fn g2(&self) -> G2Point {
        self.0.g2()
    }

    pub fn g2s(&self) -> G2Point {
        self.0.g2s()
    }
}

/// A struct representing a proof in the KZG scheme.
#[derive(CanonicalDeserialize, CanonicalSerialize)]
pub struct Proof {
    pub a_commit: KzgCommitment,
    pub b_commit: KzgCommitment,
    pub c_commit: KzgCommitment,
    pub z_commit: KzgCommitment,
    pub t_lo_commit: KzgCommitment,
    pub t_mid_commit: KzgCommitment,
    pub t_hi_commit: KzgCommitment,
    pub w_ev_x_commit: KzgCommitment,
    pub w_ev_wx_commit: KzgCommitment,
    pub bar_a: Fr,
    pub bar_b: Fr,
    pub bar_c: Fr,
    pub bar_s_sigma_1: Fr,
    pub bar_s_sigma_2: Fr,
    pub bar_z_w: Fr,
    pub u: Fr,
    pub degree: usize,
}

/// A struct representing a KZG commitment.
#[derive(Debug, Clone, PartialEq, Eq, CanonicalDeserialize, CanonicalSerialize)]
pub struct KzgCommitment(pub G1Point);

impl KzgCommitment {
    /// Returns a reference to the inner G1 point of the commitment.
    pub fn inner(&self) -> &G1Point {
        &self.0
    }
}

impl Mul<Fr> for KzgCommitment {
    type Output = Self;

    /// Multiplies the commitment by a scalar.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The scalar to multiply by.
    ///
    /// # Returns
    ///
    /// A new commitment representing the result.
    fn mul(self, rhs: Fr) -> Self::Output {
        let element = self.0.mul(rhs);
        Self(element.into())
    }
}

impl Add for KzgCommitment {
    type Output = Self;

    /// Adds two commitments together.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The other commitment to add.
    ///
    /// # Returns
    ///
    /// A new commitment representing the sum.
    fn add(self, rhs: Self) -> Self::Output {
        let commitment = self.0 + rhs.0;
        Self(commitment.into())
    }
}

impl Sub for KzgCommitment {
    type Output = Self;

    /// Subtracts one commitment from another.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The other commitment to subtract.
    ///
    /// # Returns
    ///
    /// A new commitment representing the difference.
    fn sub(self, rhs: Self) -> Self::Output {
        Self::add(self, -rhs)
    }
}

impl Mul<Fr> for &KzgCommitment {
    type Output = KzgCommitment;

    /// Multiplies the commitment by a scalar.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The scalar to multiply by.
    ///
    /// # Returns
    ///
    /// A new commitment representing the result.
    fn mul(self, rhs: Fr) -> Self::Output {
        let element = self.0.mul(rhs);
        KzgCommitment(element.into())
    }
}

impl Neg for KzgCommitment {
    type Output = Self;

    /// Negates the commitment.
    ///
    /// # Returns
    ///
    /// A new commitment representing the negation.
    fn neg(self) -> Self::Output {
        Self(self.0.neg())
    }
}
