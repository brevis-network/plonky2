use core::marker::PhantomData;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use itertools::Itertools;
use log::{debug, info};

use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::field::polynomial::PolynomialValues;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::timed;
use plonky2::util::timing::TimingTree;

use crate::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use crate::stark::Stark;
use crate::util::trace_rows_to_poly_values;
use crate::evaluation_frame::{StarkFrame};
use crate::lookup::{Lookup, Column, Filter};

use crate::byte_chip::opcode::{NUM_BYTE_OPS, NUM_BYTE_OP_COLS};
use crate::byte_chip::NUM_ROWS;
use crate::byte_chip::columns::{NUM_BYTE_COLS, opcode_index, multiplicities_index, filter_index, b_index, c_index};
use crate::byte_chip::event::ByteLookupEvent;
use crate::byte_chip::opcode::{ByteOpcode, all_byte_opcodes};

/// ByteStark
#[derive(Copy, Clone, Default, Debug)]
pub struct ByteStark<F: RichField + Extendable<D>, const D: usize> {
    f: PhantomData<F>,
}

/// Implementation of ByteStark
impl<F: RichField + Extendable<D>, const D: usize> ByteStark<F, D> {
    /// generate trace from byte lookup events
    pub fn generate_trace(
        &self,
        inputs: Vec<ByteLookupEvent>,
        timing: &mut TimingTree,
    ) -> Vec<PolynomialValues<F>> {
        let (mut trace, map) = Self::init_trace_and_map();

        for input in inputs.iter() {
            let opcode = input.get_opcode();
            match opcode {
                ByteOpcode::U8Range => {
                    let input_b0 = ByteLookupEvent::new(ByteOpcode::LEU, 1, 0, input.b);
                    let input_b1 = ByteLookupEvent::new(ByteOpcode::LEU, 1, input.b, u8::MAX as u32);
                    let input_c0 = ByteLookupEvent::new(ByteOpcode::LEU, 1, 0, input.c);
                    let input_c1 = ByteLookupEvent::new(ByteOpcode::LEU, 1, input.c, u8::MAX as u32);
                    let input_values0 = Self::generate_trace_row(input_b0, &map, &mut trace);
                    let input_values1 = Self::generate_trace_row(input_b1, &map, &mut trace);
                    let input_values2 = Self::generate_trace_row(input_c0, &map, &mut trace);
                    let input_values3 = Self::generate_trace_row(input_c1, &map, &mut trace);
                    trace.push(input_values0);
                    trace.push(input_values1);
                    trace.push(input_values2);
                    trace.push(input_values3);
                }
                _ => {
                    let input_values = Self::generate_trace_row(*input, &map, &mut trace);
                    trace.push(input_values);
                }
            }
        }

        let padded_len = trace.len().next_power_of_two() - trace.len();
        for _ in 0..padded_len {
            trace.push([F::ZERO; NUM_BYTE_COLS]);
        }

        trace_rows_to_poly_values(trace)
    }

    fn generate_trace_row(event: ByteLookupEvent, map: &BTreeMap<ByteLookupEvent, (usize, usize)>, trace: &mut Vec<[F; NUM_BYTE_COLS]>) -> [F; NUM_BYTE_COLS] {
        // deal with counter
        let (row, i) = map[&event];
        trace[row][multiplicities_index(i)] += F::ONE;

        // deal with newly added input
        let mut input_values = [F::ZERO; NUM_BYTE_COLS];
        input_values[opcode_index(i)] = F::from_canonical_u32(event.a);
        input_values[filter_index(i)] = F::ONE;
        input_values[b_index()] = F::from_canonical_u32(event.b);
        input_values[c_index()] = F::from_canonical_u32(event.c);

        input_values
    }

    /// initialize the trace and map
    fn init_trace_and_map() -> (Vec<[F; NUM_BYTE_COLS]>, BTreeMap<ByteLookupEvent, (usize, usize)>) {
        
        let mut trace = vec![[F::ZERO; NUM_BYTE_COLS]; NUM_ROWS];
        let mut map = BTreeMap::new();

        // Get all opcodes to iterate over.
        let opcodes = all_byte_opcodes();

        // Generate the trace row-wise.
        // for debug
        for (row, (b, c)) in (0..=u8::MAX).cartesian_product(0..=u8::MAX).enumerate() {
        //for (row, (b, c)) in (0..4).cartesian_product(0..4).enumerate() {
            let b = b as u8;
            let c = c as u8;
            let row_values = &mut trace[row];

            row_values[b_index()] = F::from_canonical_u8(b);
            row_values[c_index()] = F::from_canonical_u8(c);

            for (i, &opcode) in opcodes.iter().enumerate() {
                match opcode {
                    ByteOpcode::AND => {
                        let and = b & c;
                        row_values[opcode_index(i)] = F::from_canonical_u8(and);
                        map.insert(ByteLookupEvent::new(opcode, and as u32, b as u32, c as u32), (row, i));
                    }
                    ByteOpcode::OR => {
                        let or = b | c;
                        row_values[opcode_index(i)] = F::from_canonical_u8(or);
                        map.insert(ByteLookupEvent::new(opcode, or as u32, b as u32, c as u32), (row, i));
                    }
                    ByteOpcode::XOR => {
                        let xor = b ^ c;
                        row_values[opcode_index(i)] = F::from_canonical_u8(xor);
                        map.insert(ByteLookupEvent::new(opcode, xor as u32, b as u32, c as u32), (row, i));
                    }
                    ByteOpcode::LEU => {
                        let ltu: u32 = if b <= c { 1 } else { 0 };
                        row_values[opcode_index(i)] = F::from_canonical_u32(ltu);
                        map.insert(ByteLookupEvent::new(opcode, ltu, b as u32, c as u32), (row, i));
                    }
                    _ => {}
                };
            }
        }

        (trace, map)
    }
}

/// All implemented with lookups, thus no public inputs
const BYTE_PUBLIC_INPUTS: usize = 0;

impl<F: RichField + Extendable<D>, const D: usize> Stark<F, D> for ByteStark<F, D> {
    type EvaluationFrame<FE, P, const D2: usize> = StarkFrame<
        P, 
        P::Scalar, 
        NUM_BYTE_COLS, 
        BYTE_PUBLIC_INPUTS
    >
    where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>; 

    type EvaluationFrameTarget = StarkFrame<
        ExtensionTarget<D>, 
        ExtensionTarget<D>, 
        NUM_BYTE_COLS, 
        BYTE_PUBLIC_INPUTS
    >;

    fn constraint_degree(&self) -> usize { 0 }

    fn lookups(&self) -> Vec<Lookup<F>> {
        let mut result = vec![];
        for i in 0..NUM_BYTE_OP_COLS {
            // res, b, c, currently hard-coded
            let columns = vec![Column::linear_combination(
                [opcode_index(i), b_index(), c_index()]
                    .iter()
                    .enumerate()
                    .map(|(j, &p)| (p, F::from_canonical_u32(1 << (j * 8)))),
            )];
            // res, b, c
            let table_column = Column::linear_combination(
                [opcode_index(i), b_index(), c_index()]
                    .iter()
                    .enumerate()
                    .map(|(j, &p)| (p, F::from_canonical_u32(1 << (j * 8)))),
            );
            // corresponding frequencies
            let frequencies_column = Column::single(multiplicities_index(i));
            // corresponding filters
            let valid = Column::single(filter_index(i));
            let filter = Filter::new_simple(valid);
            let filter_columns = vec![filter];

            result.push(Lookup {
                columns,
                table_column,
                frequencies_column,
                filter_columns
            });
        }

        result
    }

    // no need to constrain anything
    fn eval_packed_generic<FE, P, const D2: usize>(
        &self,
        _vars: &Self::EvaluationFrame<FE, P, D2>,
        _yield_constr: &mut ConstraintConsumer<P>,
    ) where
        FE: FieldExtension<D2, BaseField = F>,
        P: PackedField<Scalar = FE>,
    {
    }

    // no need to constrain anything
    fn eval_ext_circuit(
        &self,
        _builder: &mut CircuitBuilder<F, D>,
        _vars: &Self::EvaluationFrameTarget,
        _yield_constr: &mut RecursiveConstraintConsumer<F, D>,
    ) {
    }
}