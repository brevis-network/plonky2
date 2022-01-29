use anyhow::{ensure, Result};
use plonky2::field::extension_field::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::plonk::circuit_data::CommonCircuitData;
use plonky2::plonk::config::GenericConfig;
use plonky2::plonk::proof::ProofWithPublicInputs;

use crate::config::StarkConfig;
use crate::proof::{StarkProof, StarkProofWithPublicInputs};
use crate::stark::Stark;

pub(crate) fn verify<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    S: Stark<F, D>,
    const D: usize,
>(
    proof_with_pis: StarkProofWithPublicInputs<F, C, D>,
    config: &StarkConfig,
) -> Result<()> {
    let challenges = proof_with_pis.get_challenges(config)?;
    verify_with_challenges(proof_with_pis, challenges, verifier_data, common_data)
}

pub(crate) fn verify_with_challenges<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    S: Stark<F, D>,
    const D: usize,
>(
    proof_with_pis: StarkProofWithPublicInputs<F, C, D>,
    challenges: ProofChallenges<F, D>,
    verifier_data: &VerifierOnlyCircuitData<C, D>,
    common_data: &CommonCircuitData<F, C, D>,
) -> Result<()> {
    assert_eq!(
        proof_with_pis.public_inputs.len(),
        common_data.num_public_inputs
    );
    let public_inputs_hash = &proof_with_pis.get_public_inputs_hash();

    let ProofWithPublicInputs { proof, .. } = proof_with_pis;

    let local_constants = &proof.openings.constants;
    let local_wires = &proof.openings.wires;
    let vars = EvaluationVars {
        local_constants,
        local_wires,
        public_inputs_hash,
    };
    let local_zs = &proof.openings.plonk_zs;
    let next_zs = &proof.openings.plonk_zs_right;
    let s_sigmas = &proof.openings.plonk_sigmas;
    let partial_products = &proof.openings.partial_products;

    // Evaluate the vanishing polynomial at our challenge point, zeta.
    let vanishing_polys_zeta = eval_vanishing_poly(
        common_data,
        challenges.plonk_zeta,
        vars,
        local_zs,
        next_zs,
        partial_products,
        s_sigmas,
        &challenges.plonk_betas,
        &challenges.plonk_gammas,
        &challenges.plonk_alphas,
    );

    // Check each polynomial identity, of the form `vanishing(x) = Z_H(x) quotient(x)`, at zeta.
    let quotient_polys_zeta = &proof.openings.quotient_polys;
    let zeta_pow_deg = challenges
        .plonk_zeta
        .exp_power_of_2(common_data.degree_bits);
    let z_h_zeta = zeta_pow_deg - F::Extension::ONE;
    // `quotient_polys_zeta` holds `num_challenges * quotient_degree_factor` evaluations.
    // Each chunk of `quotient_degree_factor` holds the evaluations of `t_0(zeta),...,t_{quotient_degree_factor-1}(zeta)`
    // where the "real" quotient polynomial is `t(X) = t_0(X) + t_1(X)*X^n + t_2(X)*X^{2n} + ...`.
    // So to reconstruct `t(zeta)` we can compute `reduce_with_powers(chunk, zeta^n)` for each
    // `quotient_degree_factor`-sized chunk of the original evaluations.
    for (i, chunk) in quotient_polys_zeta
        .chunks(common_data.quotient_degree_factor)
        .enumerate()
    {
        ensure!(vanishing_polys_zeta[i] == z_h_zeta * reduce_with_powers(chunk, zeta_pow_deg));
    }

    let merkle_caps = &[
        verifier_data.constants_sigmas_cap.clone(),
        proof.wires_cap,
        proof.plonk_zs_partial_products_cap,
        proof.quotient_polys_cap,
    ];

    verify_fri_proof::<F, C, D>(
        &common_data.get_fri_instance(challenges.plonk_zeta),
        &proof.openings,
        &challenges,
        merkle_caps,
        &proof.opening_proof,
        &common_data.fri_params,
    )?;

    Ok(())
}
