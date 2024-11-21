use std::time::Instant;
use anyhow::Error;
use log::Level;
use plonky2_field::extension::Extendable;
use crate::hash::hash_types::RichField;
use crate::iop::witness::{PartialWitness, WitnessWrite};
use crate::plonk::circuit_builder::CircuitBuilder;
use crate::plonk::circuit_data::{CircuitConfig, CommonCircuitData, VerifierOnlyCircuitData};
use crate::plonk::config::{AlgebraicHasher, GenericConfig};
use crate::plonk::proof::ProofWithPublicInputs;
use crate::util::timing::TimingTree;
use crate::plonk;

type Proof<F, C, const D: usize> = (
    ProofWithPublicInputs<F, C, D>,
    VerifierOnlyCircuitData<C, D>,
    CommonCircuitData<F, D>,
);

pub fn wrap_plonky2_proof<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    InnerC: GenericConfig<D, F = F>,
    const D: usize,
>(
    inner_proof: ProofWithPublicInputs<F, InnerC, D>,
    inner_vd: VerifierOnlyCircuitData<InnerC, D>,
    inner_cd: CommonCircuitData<F, D>,
) -> Result<Proof<F, C, D>, Error>
where
    InnerC::Hasher: AlgebraicHasher<F>,
{
    let circuit_config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(circuit_config.clone());
    let mut pw = PartialWitness::new();
    let pt = builder.add_virtual_proof_with_pis(&inner_cd);
    pw.set_proof_with_pis_target(&pt, &inner_proof);

    let inner_data = builder.add_virtual_verifier_data(inner_cd.config.fri_config.cap_height);
    pw.set_cap_target(
        &inner_data.constants_sigmas_cap,
        &inner_vd.constants_sigmas_cap,
    );
    pw.set_hash_target(inner_data.circuit_digest, inner_vd.circuit_digest);

    for i in 0..30 {
        builder.register_public_input(pt.public_inputs[0]);
    }

    builder.verify_proof::<InnerC>(&pt, &inner_data, &inner_cd);

    let data = builder.build::<C>();

    let mut timing = TimingTree::new("prove", Level::Debug);

    let start_recursive = Instant::now();
    let proof = plonk::prover::prove(&data.prover_only, &data.common, pw, &mut timing)?;
    println!("plonky2 wrapper use: {}s", start_recursive.elapsed().as_secs_f64());

    println!("plonky2 wrapper degrees: {}", data.common.degree_bits());

    data.verify(proof.clone())?;

    Ok((proof, data.verifier_only, data.common))
}