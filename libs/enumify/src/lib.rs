pub use enumify_macros::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[enumify]
pub enum Opacity<O, C> {
    /// An item whose contents cannot be inspected except in summary form as type `O`.
    Opaque(O),

    /// An item whose contents are visible to users as type `C`.
    Clear(C),
}
