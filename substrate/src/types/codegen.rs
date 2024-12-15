//! Helpers for type codegen.
//!
//! Only necessary until new trait solver is stabilized
//! (https://users.rust-lang.org/t/associated-type-bounds-only-solved-correctly-if-extracted-to-separate-trait/122535/2).

use super::{FlatLen, Flatten, HasBundleKind, HasView, Unflatten};

pub trait HasBundleKindView<V>:
    HasBundleKind<BundleKind: HasView<V, View = <Self as HasBundleKindView<V>>::View>>
{
    type View: HasBundleKind<BundleKind = <Self as HasBundleKind>::BundleKind>;
}
impl<V, T> HasBundleKindView<V> for T
where
    T: HasBundleKind<
        BundleKind: HasView<V, View: HasBundleKind<BundleKind = <T as HasBundleKind>::BundleKind>>,
    >,
{
    type View = <<T as HasBundleKind>::BundleKind as HasView<V>>::View;
}

pub trait FlatLenView<V>:
    HasBundleKind<BundleKind: HasView<V, View = <Self as FlatLenView<V>>::View>>
{
    type View: FlatLen;
}

impl<V, T> FlatLenView<V> for T
where
    T: HasBundleKind<BundleKind: HasView<V, View: FlatLen>>,
{
    type View = <<T as HasBundleKind>::BundleKind as HasView<V>>::View;
}

pub trait FlattenView<V, F>:
    HasBundleKind<BundleKind: HasView<V, View = <Self as FlattenView<V, F>>::View>>
{
    type View: Flatten<F>;
}

impl<V, F, T> FlattenView<V, F> for T
where
    T: HasBundleKind<BundleKind: HasView<V, View: Flatten<F>>>,
{
    type View = <<T as HasBundleKind>::BundleKind as HasView<V>>::View;
}

pub trait UnflattenView<V, D: HasBundleKind, S>:
    HasBundleKind<BundleKind: HasView<V, View = <Self as UnflattenView<V, D, S>>::View>>
{
    type View: Unflatten<<D as HasBundleKind>::BundleKind, S>;
}

impl<V, D: HasBundleKind, S, T> UnflattenView<V, D, S> for T
where
    T: HasBundleKind<BundleKind: HasView<V, View: Unflatten<<D as HasBundleKind>::BundleKind, S>>>,
{
    type View = <<T as HasBundleKind>::BundleKind as HasView<V>>::View;
}
