use crate::{
    commitment_scheme::{commit, create_aggregate_witness, create_witness},
    range_proof::polynomial,
    transcript::TranscriptProtocol,
};
use ark_bls12_381::{Bls12_381, Fr};
use ark_poly::Polynomial;
use ark_poly::{EvaluationDomain, GeneralEvaluationDomain};
use ark_poly_commit::kzg10::{Commitment, Powers};
use rand::Rng;

pub fn prove(
    pk: &Powers<Bls12_381>,
    domain: &GeneralEvaluationDomain<Fr>,
    z: &Fr,
    transcript: &mut dyn TranscriptProtocol,
) -> (
    Fr,
    Fr,
    Fr,
    Commitment<Bls12_381>,
    Commitment<Bls12_381>,
    Commitment<Bls12_381>,
    Commitment<Bls12_381>,
    Commitment<Bls12_381>,
) {
    // extended domain
    // by adding more elements to `g_poly` (for zero knowledge properties)
    // the domain is extended
    let domain_2n: GeneralEvaluationDomain<Fr> =
        GeneralEvaluationDomain::<Fr>::new(2usize * domain.size()).unwrap();

    // compute all polynomials
    let mut rng = rand::thread_rng();
    let r = Fr::from(rng.gen::<u64>());
    let f_poly = polynomial::compute_f(&domain, &z, &r);

    let alpha = Fr::from(rng.gen::<u64>());
    let beta = Fr::from(rng.gen::<u64>());
    let g_poly = polynomial::compute_g(&domain, &z, &alpha, &beta);
    let (w1_poly, w2_poly) = polynomial::compute_w1_w2(&domain, &g_poly, &f_poly);
    let w3_poly = polynomial::compute_w3(&domain, &domain_2n, &g_poly);

    // aggregate w1, w2 and w3 to compute quotient polynomial
    // `tau` is the random scalar for aggregation
    let tau = transcript.challenge_scalar(b"tau");
    let (q_poly, _) = polynomial::compute_q(&domain, &w1_poly, &w2_poly, &w3_poly, &tau);

    // compute commitments to polynomials
    let f_commitment = commit(&pk, &f_poly);
    let g_commitment = commit(&pk, &g_poly);
    let q_commitment = commit(&pk, &q_poly);

    // `rho` is the random evaluation point
    let rho = transcript.challenge_scalar(b"rho");
    let g_eval = g_poly.evaluate(&rho);

    // evaluate g at `rho * omega`
    let rho_omega = rho * domain.group_gen();
    let g_omega_eval = g_poly.evaluate(&rho_omega);

    // compute evaluation of w_cap at ρ
    let w_cap_poly = polynomial::compute_w_cap(&domain, &f_poly, &q_poly, &rho);
    let w_cap_eval = w_cap_poly.evaluate(&rho);

    // compute witness for g(X) at ρw
    let shifted_witness_poly = create_witness(&g_poly, &rho_omega);
    let shifted_witness_commitment = commit(&pk, &shifted_witness_poly);

    // compute aggregate witness for
    // g(X) at ρ, f(X) at ρ, w_cap(X) at ρ
    let aggregation_challenge = transcript.challenge_scalar(b"aggregation_challenge");
    let aggregate_witness_poly =
        create_aggregate_witness(vec![g_poly, w_cap_poly], &rho, &aggregation_challenge);
    let aggregate_witness_commitment = commit(&pk, &aggregate_witness_poly);

    (
        g_eval,
        g_omega_eval,
        w_cap_eval,
        f_commitment,
        g_commitment,
        q_commitment,
        aggregate_witness_commitment,
        shifted_witness_commitment,
    )
}
