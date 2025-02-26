use itertools::Itertools;
use log::debug;

use crate::field::extension::Extendable;
use crate::fri::proof::{FriProof, FriProofTarget};
use crate::hash::hash_types::RichField;
use crate::iop::witness::WitnessWrite;
use crate::plonk::config::AlgebraicHasher;

/// Set the targets in a `FriProofTarget` to their corresponding values in a `FriProof`.
pub fn set_fri_proof_target<F, W, H, const D: usize>(
    witness: &mut W,
    fri_proof_target: &FriProofTarget<D>,
    fri_proof: &FriProof<F, H, D>,
) where
    F: RichField + Extendable<D>,
    W: WitnessWrite<F> + ?Sized,
    H: AlgebraicHasher<F>,
{
    witness.set_target(fri_proof_target.pow_witness, fri_proof.pow_witness);

    for (&t, &x) in fri_proof_target
        .final_poly
        .0
        .iter()
        .zip_eq(&fri_proof.final_poly.coeffs)
    {
        witness.set_extension_target(t, x);
    }

    debug!("set fri final_poly target done");

    for (t, x) in fri_proof_target
        .commit_phase_merkle_caps
        .iter()
        .zip_eq(&fri_proof.commit_phase_merkle_caps)
    {
        witness.set_cap_target(t, x);
    }

    debug!("set fri commit_phase_merkle_caps target done");

    debug!("fri target query_round_proofs length: {}, fri query_round_proofs: {:?}", fri_proof_target.query_round_proofs.len(), fri_proof.query_round_proofs.len());
    for (qt, q) in fri_proof_target
        .query_round_proofs
        .iter()
        .zip_eq(&fri_proof.query_round_proofs)
    {
        debug!("qt evals_proofs length: {}, q evals_proofs length: {:?}", qt.initial_trees_proof.evals_proofs.len(), q.initial_trees_proof.evals_proofs.len());
        for (at, a) in qt
            .initial_trees_proof
            .evals_proofs
            .iter()
            .zip_eq(&q.initial_trees_proof.evals_proofs)
        {
            debug!("at.0 {:?}, a.0 {:?}", at.0.len(), a.0.len());
            for (&t, &x) in at.0.iter().zip_eq(&a.0) {
                witness.set_target(t, x);
            }
            debug!("at.1.siblings {:?}, a.1.siblings {:?}", at.1.siblings.len(), a.1.siblings.len());
            for (&t, &x) in at.1.siblings.iter().zip_eq(&a.1.siblings) {
                witness.set_hash_target(t, x);
            }
        }

        debug!("set fri query_round_proofs target done");

        for (st, s) in qt.steps.iter().zip_eq(&q.steps) {
            for (&t, &x) in st.evals.iter().zip_eq(&s.evals) {
                witness.set_extension_target(t, x);
            }
            for (&t, &x) in st
                .merkle_proof
                .siblings
                .iter()
                .zip_eq(&s.merkle_proof.siblings)
            {
                witness.set_hash_target(t, x);
            }
            debug!("set fri siblings target done");
        }
    }
}
