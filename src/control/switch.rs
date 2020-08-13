/*!
Switch statements on finite types

# Implementation Notes
The difference between a switch statement and a `rec` statement is that the former is implemented with run-length encoding, while
the later is implemented with a `ValArr`. "Compression normalization" is in effect: a `switch` statement which is larger than a
corresponding `rec` statement will normalize to the latter, and vice versa. We assume here that a pointer is 64 bits.
*/
use crate::function::pi::Pi;
use crate::lifetime::Lifetime;
use crate::value::VarId;

/// A switch node
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Switch {
    /// The type of this switch node
    ty: VarId<Pi>,
    /// The lifetime of this switch node
    lt: Lifetime,
    //TODO: RLE branches
}
