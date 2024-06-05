use alloc::vec::Vec;
use core::ops::{Add, Mul, Neg, Sub};

use ark_bls12_381::{Fr, G1Affine, G2Affine};
use ark_ec::{AffineRepr, CurveGroup};
use ark_poly::Polynomial;
use ark_poly::univariate::DensePolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

pub type G1Point = G1Affine;
pub type G2Point = G2Affine;
pub type Poly = DensePolynomial<Fr>;

#[derive(Debug, Clone, CanonicalDeserialize, CanonicalSerialize)]
pub struct Srs {
    g1_points: Vec<G1Point>,
    g2: G2Point,
    g2s_point: G2Point,
}

impl Srs {
    pub fn g1_points(&self) -> Vec<G1Point> {
        self.g1_points.clone()
    }
    pub fn g2(&self) -> G2Point {
        self.g2
    }
    pub fn g2s(&self) -> G2Point {
        self.g2s_point
    }
}

#[derive(CanonicalSerialize, CanonicalDeserialize)]
pub struct CommonPreprocessedInput {
    pub n: usize,
    pub k1: Fr,
    pub k2: Fr,
    pub q_lx: Poly,
    pub q_rx: Poly,
    pub q_mx: Poly,
    pub q_ox: Poly,
    pub q_cx: Poly,
    pub s_sigma_1: Poly,
    pub s_sigma_2: Poly,
    pub s_sigma_3: Poly,
    pub pi_x: Poly,
}

pub struct KzgScheme(Srs);

impl KzgScheme {
    pub fn new(srs: Srs) -> Self {
        Self(srs)
    }

    pub fn commit(&self, polynomial: &Poly) -> KzgCommitment {
        let commitment = self.evaluate_in_s(polynomial);
        KzgCommitment(commitment)
    }

    pub fn commit_para(&self, para: Fr) -> KzgCommitment {
        let g1_0 = *self.0.g1_points.first().unwrap();
        let commitment = g1_0.mul(para).into();
        KzgCommitment(commitment)
    }

    fn evaluate_in_s(&self, polynomial: &Poly) -> G1Point {
        let g1_points = self.0.g1_points();
        assert!(g1_points.len() > polynomial.degree());
        let poly = &polynomial.coeffs;

        let mut res = G1Point::zero().mul(Fr::from(1));
        for i in 0..poly.len() {
            let coef = poly[i];
            if coef == Fr::from(0) {
                continue;
            }
            let point = g1_points[i];

            res = res.add(point.mul(coef));
        }
        res.into_affine()
    }

    pub fn g2(&self) -> G2Point {
        self.0.g2()
    }
    pub fn g2s(&self) -> G2Point {
        self.0.g2s()
    }


}

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

#[derive(Debug, Clone, PartialEq, Eq, CanonicalDeserialize, CanonicalSerialize)]
pub struct KzgCommitment(pub G1Point);

impl KzgCommitment {
    pub fn inner(&self) -> &G1Point {
        &self.0
    }
}

impl Mul<Fr> for KzgCommitment {
    type Output = Self;

    fn mul(self, rhs: Fr) -> Self::Output {
        let element = self.0.mul(rhs);
        Self(element.into())
    }
}

impl Add for KzgCommitment {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let commitment = self.0 + rhs.0;
        Self(commitment.into())
    }
}

impl Sub for KzgCommitment {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::add(self, -rhs)
    }
}

impl Mul<Fr> for &KzgCommitment {
    type Output = KzgCommitment;

    fn mul(self, rhs: Fr) -> Self::Output {
        let element = self.0.mul(rhs);
        KzgCommitment(element.into())
    }
}

impl Neg for KzgCommitment {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let point = self.0;
        Self(-point)
    }
}