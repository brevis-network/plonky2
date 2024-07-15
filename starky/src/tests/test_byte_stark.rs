#![allow(missing_docs)]
#[cfg(test)]

use anyhow::Result;
use plonky2::field::extension::Extendable;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::witness::PartialWitness;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{AlgebraicHasher, GenericConfig, PoseidonGoldilocksConfig};
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::util::timing::TimingTree;
use plonky2::timed;

use crate::config::StarkConfig;
use crate::proof::StarkProofWithPublicInputs;
use crate::prover::prove;
use crate::recursive_verifier::{
    add_virtual_stark_proof_with_pis, set_stark_proof_with_pis_target,
    verify_stark_proof_circuit,
};
use crate::stark::Stark;
use crate::stark_testing::{test_stark_circuit_constraints, test_stark_low_degree};
use crate::verifier::verify_stark_proof;

use crate::byte_chip::byte_stark::ByteStark;
use crate::byte_chip::event::ByteLookupEvent;
use crate::byte_chip::opcode::ByteOpcode;

use env_logger::Env;
use log::{debug, info};


#[test]
fn test_byte_stark() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;
    type F = GoldilocksField;
    type S = ByteStark<F, D>;

    let config = StarkConfig::standard_fast_config();

    let mut test_input = Vec::new();
    // AND
    let opcode = ByteOpcode::AND;
    for i in 0..3 {
        let b = i;
        let c = i + 1;
        let a = b & c;
        let event = ByteLookupEvent::new(opcode, a, b, c);
        test_input.push(event);
    }
    // OR
    let opcode = ByteOpcode::OR;
    for i in 0..3 {
        let b = i;
        let c = i + 1;
        let a = b | c;
        let event = ByteLookupEvent::new(opcode, a, b, c);
        test_input.push(event);
    }
    // XOR
    let opcode = ByteOpcode::XOR;
    for i in 0..3 {
        let b = i;
        let c = i + 1;
        let a = b ^ c;
        let event = ByteLookupEvent::new(opcode, a, b, c);
        test_input.push(event);
    }
    // LEU
    let opcode = ByteOpcode::LEU;
    for i in 0..3 {
        let b = i;
        let c = i;
        let a = (b <= c) as u32;
        let event = ByteLookupEvent::new(opcode, a, b, c);
        test_input.push(event);
    }
    // U8 Range
    let opcode = ByteOpcode::U8Range;
    for i in 0..3 {
        let b = i;
        let c = i;
        let a = (b < c) as u32;
        let event = ByteLookupEvent::new(opcode, a, b, c);
        test_input.push(event);
    }
    debug!("test_input: {:?}", test_input);

    let mut timing = TimingTree::new("prove byte trace", log::Level::Info);
    let stark = S::default();
    let byte_trace = timed!(
        timing,
        "generate byte trace",
        stark.generate_trace(test_input, &mut timing)
    );

    // #[cfg(debug_assertions)]
    // {
    //     for i in 0..byte_trace[0].len() {
    //         for j in 0..byte_trace.len() {
    //             print!("{:?} ", byte_trace[j].values[i]);
    //         }
    //         println!("");
    //     }
    // }
    // debug!("byte_trace: {:?}", byte_trace);

    let public_inputs: [GoldilocksField; 0] = [];
    let proof = prove::<F, C, S, D>(
        stark,
        &config,
        byte_trace,
        &public_inputs,
        &mut timing,
    )?;

    timing.print();
    debug!("-----------------------------------------------------");

    timing = TimingTree::new("verify single proof", log::Level::Info);
    verify_stark_proof(stark, proof.clone(), &config)?;
    timing.print();

    debug!("-----------------------------------------------------");

    timing = TimingTree::new("verify recursive proof", log::Level::Info);
    recursive_proof::<F, C, S, C, D>(stark, proof, &config, true)?;
    timing.print();

    Ok(())
}

fn recursive_proof<
    F: RichField + Extendable<D>,
    C: GenericConfig<D, F = F>,
    S: Stark<F, D> + Copy,
    InnerC: GenericConfig<D, F = F>,
    const D: usize,
>(
    stark: S,
    inner_proof: StarkProofWithPublicInputs<F, InnerC, D>,
    inner_config: &StarkConfig,
    print_gate_counts: bool,
) -> Result<()> 
where InnerC::Hasher: AlgebraicHasher<F> 
{
    let circuit_config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(circuit_config);
    let mut pw = PartialWitness::new();
    let degree_bits = inner_proof.proof.recover_degree_bits(inner_config);
    let pt =
        add_virtual_stark_proof_with_pis(&mut builder, &stark, inner_config, degree_bits, 0, 0);
    set_stark_proof_with_pis_target(&mut pw, &pt, &inner_proof, builder.zero());

    verify_stark_proof_circuit::<F, InnerC, S, D>(&mut builder, stark, pt, inner_config);

    if print_gate_counts {
        builder.print_gate_counts(0);
    }

    let data = builder.build::<C>();
    let proof = data.prove(pw)?;
    data.verify(proof)
}

