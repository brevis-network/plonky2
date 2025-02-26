use log::debug;
use plonky2::field::extension::{Extendable, FieldExtension};
use plonky2::field::packed::PackedField;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::ext_target::ExtensionTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;

use crate::constraint_consumer::{ConstraintConsumer, RecursiveConstraintConsumer};
use crate::cross_table_lookup::{
    eval_cross_table_lookup_checks, eval_cross_table_lookup_checks_circuit, CtlCheckVars,
    CtlCheckVarsTarget,
};
use crate::lookup::{
    eval_ext_lookups_circuit, eval_packed_lookups_generic, Lookup, LookupCheckVars,
    LookupCheckVarsTarget,
};
use crate::stark::Stark;

/// Evaluates all constraint, permutation and cross-table lookup polynomials
/// of the current STARK at the local and next values.
pub(crate) fn eval_vanishing_poly<F, FE, P, S, const D: usize, const D2: usize>(
    stark: &S,
    vars: &S::EvaluationFrame<FE, P, D2>,
    p2_vars: Option<&[S::P2EvaluationFrame<FE, P, D2>]>,
    random_gamma: Option<&[FE]>,
    lookups: &[Lookup<F>],
    lookup_vars: Option<LookupCheckVars<F, FE, P, D2>>,
    ctl_vars: Option<&[CtlCheckVars<F, FE, P, D2>]>,
    consumer: &mut ConstraintConsumer<P>,
) where
    F: RichField + Extendable<D>,
    FE: FieldExtension<D2, BaseField = F>,
    P: PackedField<Scalar = FE>,
    S: Stark<F, D>,
{
    let p2_vars_0 = p2_vars.clone().and_then(|vars|Some(&vars[0]));
    stark.eval_packed_generic(vars,  p2_vars_0.as_deref(), random_gamma.and_then(|gammas| Some(&gammas[0])), consumer);

    // Evaluate all of the STARK's table constraints.
    if stark.name() == "receipt_mpt_stark" || stark.name() == "extension_type_stark" {

        // debug!("receipt_mpt_stark eval_packed_with_challenge");
        p2_vars.unwrap().iter().zip(random_gamma.unwrap()).for_each(|(v, g)| {
            stark.eval_packed_with_challenge(vars, Some(v), Some(g), consumer);
        });
    }
    
    if let Some(lookup_vars) = lookup_vars {
        // Evaluate the STARK constraints related to the permutation arguments.
        eval_packed_lookups_generic::<F, FE, P, S, D, D2>(
            stark,
            lookups,
            vars,
            p2_vars,
            lookup_vars,
            consumer,
        );
    }

    if let Some(ctl_vars) = ctl_vars {
        // Evaluate the STARK constraints related to the CTLs.
        eval_cross_table_lookup_checks::<F, FE, P, S, D, D2>(
            vars,
            p2_vars,
            ctl_vars,
            consumer,
            stark.constraint_degree(),
        );
    }

}

/// Circuit version of `eval_vanishing_poly`.
/// Evaluates all constraint, permutation and cross-table lookup polynomials
/// of the current STARK at the local and next values.
pub(crate) fn eval_vanishing_poly_circuit<F, S, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    stark: &S,
    vars: &S::EvaluationFrameTarget,
    p2_vars: Option<&[S::P2EvaluationFrameTarget]>,
    random_gamma: Option<&[ExtensionTarget<D>]>,
    lookup_vars: Option<LookupCheckVarsTarget<D>>,
    ctl_vars: Option<&[CtlCheckVarsTarget<F, D>]>,
    consumer: &mut RecursiveConstraintConsumer<F, D>,
) where
    F: RichField + Extendable<D>,
    S: Stark<F, D>,
{
    // Evaluate all of the STARK's table constraints.
    let p2_vars_0 = p2_vars.clone().and_then(|vars|Some(&vars[0]));
    stark.eval_ext_circuit(builder, vars, p2_vars_0, random_gamma.and_then(|gammas| Some(gammas[0])), consumer);

    if stark.name() == "receipt_mpt_stark" || stark.name() == "extension_type_stark" {
        // debug!("receipt_mpt_stark eval_packed_with_challenge");
        p2_vars.unwrap().iter().zip(random_gamma.unwrap()).for_each(|(p2_v, g)| {
            stark.eval_ext_with_challenges(builder, vars, Some(p2_v), Some(*g), consumer);
        });
    }

    if let Some(lookup_vars) = lookup_vars {
        // Evaluate all of the STARK's constraints related to the permutation argument.
        eval_ext_lookups_circuit::<F, S, D>(builder, stark, vars, p2_vars, lookup_vars, consumer);
    }
    if let Some(ctl_vars) = ctl_vars {
        // Evaluate all of the STARK's constraints related to the CTLs.
        eval_cross_table_lookup_checks_circuit::<S, F, D>(
            builder,
            vars,
            p2_vars,
            ctl_vars,
            consumer,
            stark.constraint_degree(),
        );
    }
}
