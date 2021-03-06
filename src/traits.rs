use std::fmt::Debug;

use crate::vclock::{Actor, VClock};

/// State based CRDT's replicate by transmitting the entire CRDT state.
pub trait CvRDT {
    /// Merge the given CRDT into the current CRDT.
    fn merge(&mut self, other: Self);
}

/// Operation based CRDT's replicate by transmitting each operation.
pub trait CmRDT {
    /// Op defines a mutation to the CRDT.
    /// As long as Op's from one actor are replayed in exactly the same order
    /// they were generated by that actor, the CRDT will converge. In other
    /// words, we must have a total ordering on each actors operations,
    /// while requiring only a partial order over all ops.
    /// E.g.
    ///
    /// Actor A produces ops A1, A2
    /// Actor B produces ops B1, B2
    ///
    /// the only valid orderings are:
    /// A1 < A2 < B1 < B2
    /// A1 < B1 < A2 < B2
    /// B1 < A1 < A2 < B2
    /// A1 < B1 < B2 < A2
    /// B1 < A1 < B2 < A2
    /// B1 < B2 < A1 < A2
    ///
    /// Applying ops in any of the valid orders will converge to the same CRDT
    /// state
    ///
    /// Op's must be idempotent, meaning any Op may be applied more than once.
    type Op: Debug;

    /// Apply an Op to the CRDT
    fn apply(&mut self, op: Self::Op);
}

/// CRDT's are causal if they are built on top of vector clocks.
pub trait Causal<A: Actor> {
    /// Forget data that is strictly smaller than this clock
    fn forget(&mut self, clock: &VClock<A>);
}

/// Funky variant of the `CvRDT` trait.
///
/// This trait is for CvRDT's whose state space can't be easily encoded in rusts
/// typesystem so we rely on runtime error checking.
/// E.g. the unicity of timestamp assumption in LWWReg
pub trait FunkyCvRDT {
    /// User chosen error type
    type Error: Debug;

    /// Merge the given CRDT into the current CRDT.
    fn merge(&mut self, other: Self) -> Result<(), Self::Error>;
}

/// Funky variant of the `CmRDT` trait.
///
/// This trait is for CvRDT's whose state space can't be easily encoded in rusts
/// typesystem so we rely on runtime error checking.
/// E.g. the unicity property of timestamp assumption in LWWReg
pub trait FunkyCmRDT {
    /// User chosen error type
    type Error: Debug;

    /// Same Op laws from non-funky CmRDT above
    type Op: Debug + Clone;

    /// Apply an Op to the CRDT
    fn apply(&mut self, op: Self::Op) -> Result<(), Self::Error>;
}
