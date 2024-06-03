use core::ops::Mul;

use ark_bls12_381::{Bls12_381, Fr};
use ark_ec::pairing::Pairing;
use ark_ff::{BigInt, Field};
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain, Polynomial};
use ckb_std::debug;
use sha2::Digest;

use crate::challenge::ChallengeGenerator;
use crate::data_structures::{CommonPreprocessedInput, KzgScheme, Proof, Srs};
use crate::error::Error;

pub fn plonk_verify<T: Digest + Default>(
    proof: Proof,
    cip: CommonPreprocessedInput,
    srs: Srs,
) -> Result<(), Error> {
    // Initialize the KZG scheme with the structured reference string (SRS)
    let scheme = KzgScheme::new(srs.clone());

    // Commit to various polynomials
    let com_q_m = scheme.commit(&cip.q_mx);
    debug!("commit");
    let com_q_l = scheme.commit(&cip.q_lx);
    debug!("commit");
    let com_q_r = scheme.commit(&cip.q_rx);
    debug!("commit");
    let com_q_o = scheme.commit(&cip.q_ox);
    debug!("commit");
    let com_q_c = scheme.commit(&cip.q_cx);
    debug!("commit");
    let com_s_sigma_1 = scheme.commit(&cip.s_sigma_1);
    debug!("commit");
    let com_s_sigma_2 = scheme.commit(&cip.s_sigma_2);
    debug!("commit");
    let com_s_sigma_3 = scheme.commit(&cip.s_sigma_3);
    debug!("commit");

    debug!("verify challenge");

    // Generate and verify challenges
    let (alpha, beta, gamma, evaluation_challenge, v, u) = verify_challenges::<T>(&proof, &scheme);

    // Check if 'u' challenge matches the proof
    if u != proof.u {
        return Err(Error::Verify);
    }

    // Initialize evaluation domain
    let domain = <GeneralEvaluationDomain<Fr>>::new(cip.n).unwrap();
    let w = domain.element(1);

    // Compute zero polynomial evaluation at the evaluation challenge point
    let z_h_e = evaluation_challenge.pow(BigInt::new([domain.size() as u64])) - Fr::from(1);
    let l_1_e = z_h_e / (Fr::from(cip.n as u128) * (evaluation_challenge - Fr::from(1)));
    let p_i_e = cip.pi_x.evaluate(&evaluation_challenge);

    debug!("Compute r0");

    // Compute r0 using the linearization polynomial
    let r_0 = p_i_e
        - l_1_e * alpha * alpha
        - alpha
        * (proof.bar_a + proof.bar_s_sigma_1 * beta + gamma)
        * (proof.bar_b + proof.bar_s_sigma_2 * beta + gamma)
        * (proof.bar_c + gamma)
        * proof.bar_z_w;

    debug!("Compute [D]");

    let d_line1 = com_q_m.mul(proof.bar_a * proof.bar_b)
        + com_q_l.mul(proof.bar_a)
        + com_q_r.mul(proof.bar_b)
        + com_q_o.mul(proof.bar_c)
        + com_q_c;

    let d_line2 = proof.z_commit.mul(
        (proof.bar_a + beta * evaluation_challenge + gamma)
            * (proof.bar_b + beta * cip.k1 * evaluation_challenge + gamma)
            * (proof.bar_c + beta * cip.k2 * evaluation_challenge + gamma)
            * alpha
            + l_1_e * alpha * alpha
            + u,
    );

    let d_line3 = com_s_sigma_3.mul(
        (proof.bar_a + beta * proof.bar_s_sigma_1 + gamma)
            * (proof.bar_b + beta * proof.bar_s_sigma_2 + gamma)
            * alpha
            * beta
            * proof.bar_z_w,
    );

    let d_line4 = (proof.t_lo_commit
        + proof
        .t_mid_commit
        .mul(evaluation_challenge.pow(BigInt::new([proof.degree as u64 + 1])))
        + proof
        .t_hi_commit
        .mul(evaluation_challenge.pow(BigInt::new([proof.degree as u64 * 2 + 2]))))
        .mul(z_h_e);

    let d = d_line1 + d_line2 - d_line3 - d_line4;

    debug!("Compute [F]");

    let f = d
        + proof.a_commit.mul(v)
        + proof.b_commit.mul(v * v)
        + proof.c_commit.mul(v * v * v)
        + com_s_sigma_1.mul(v * v * v * v)
        + com_s_sigma_2.mul(v * v * v * v * v);

    debug!("Compute [E]");

    // Compute polynomial E using r0 and other linear combinations
    let e = -r_0
        + v * proof.bar_a
        + v * v * proof.bar_b
        + v * v * v * proof.bar_c
        + v * v * v * v * proof.bar_s_sigma_1
        + v * v * v * v * v * proof.bar_s_sigma_2
        + u * proof.bar_z_w;
    let e = scheme.commit_para(e);

    debug!("Compute left side of pairing");

    // Compute the left side of the pairing equation
    let pairing_left_side = Bls12_381::pairing(
        (proof.w_ev_x_commit.clone() + proof.w_ev_wx_commit.clone().mul(u)).0,
        scheme.g2s(),
    );

    debug!("Compute right side of pairing");

    // Compute the right side of the pairing equation
    let pairing_right_side = Bls12_381::pairing(
        (proof.w_ev_x_commit.clone().mul(evaluation_challenge)
            + proof
            .w_ev_wx_commit
            .clone()
            .mul(u * evaluation_challenge * w)
            + f
            - e)
            .0,
        scheme.g2(),
    );

    debug!("Check pairing");

    // Verify if both sides of the pairing equation are equal
    if pairing_left_side != pairing_right_side {
        return Err(Error::Verify);
    }

    debug!("Accepted!!!");

    Ok(())
}

// Function to generate and verify challenges from the proof
fn verify_challenges<T: Digest + Default>(
    proof: &Proof,
    scheme: &KzgScheme,
) -> (Fr, Fr, Fr, Fr, Fr, Fr) {
    let commitments = [
        proof.a_commit.clone(),
        proof.b_commit.clone(),
        proof.c_commit.clone(),
    ];
    let mut challenge = ChallengeGenerator::<T>::from_commitments(&commitments);
    let [beta, gamma] = challenge.generate_challenges();
    challenge.feed(&proof.z_commit);
    let [alpha] = challenge.generate_challenges();
    challenge.feed(&proof.t_lo_commit);
    challenge.feed(&proof.t_mid_commit);
    challenge.feed(&proof.t_hi_commit);
    let [evaluation_challenge] = challenge.generate_challenges();

    challenge.feed(&scheme.commit_para(proof.bar_a));
    challenge.feed(&scheme.commit_para(proof.bar_b));
    challenge.feed(&scheme.commit_para(proof.bar_c));
    challenge.feed(&scheme.commit_para(proof.bar_s_sigma_1));
    challenge.feed(&scheme.commit_para(proof.bar_s_sigma_2));
    challenge.feed(&scheme.commit_para(proof.bar_z_w));
    let [v] = challenge.generate_challenges();

    challenge.feed(&proof.w_ev_x_commit);
    challenge.feed(&proof.w_ev_wx_commit);

    let [u] = challenge.generate_challenges();

    (alpha, beta, gamma, evaluation_challenge, v, u)
}
