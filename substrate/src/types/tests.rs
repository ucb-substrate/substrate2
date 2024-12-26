use substrate::types::Io;
use substrate::types::{Input, Output, Signal};

/// An Io with a generic type parameter.
#[derive(Debug, Clone, Io)]
pub struct GenericIo<T> {
    /// A single input field.
    pub signal: Input<T>,
}

/// A named struct Io.
#[derive(Debug, Clone, Io)]
pub struct NamedStructIo {
    /// An input.
    pub first: Input<Signal>,
    /// An output.
    pub second: Output<Signal>,
}

/// A tuple struct Io.
#[derive(Debug, Clone, Io)]
pub struct TupleIo(pub Input<Signal>, pub Output<Signal>);

/// An enum Io.
#[derive(Debug, Clone, Io)]
pub enum EnumIo {
    A,
    B { a: Input<Signal>, b: Output<Signal> },
    C(NamedStructIo),
}

// /// An enum Io.
// pub enum EnumIoKind {
//     A,
//     B {
//         a: <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
//         b: <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
//     },
//     C(<NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind),
// }
// impl ::substrate::types::FlatLen for EnumIo
// where
//     Input<Signal>: ::substrate::types::FlatLen,
//     Output<Signal>: ::substrate::types::FlatLen,
//     NamedStructIo: ::substrate::types::FlatLen,
// {
//     fn len(&self) -> usize {
//         {
//             match self {
//                 EnumIo::A => 0,
//                 EnumIo::B { a, b } => {
//                     <Input<Signal> as ::substrate::types::FlatLen>::len(a)
//                         + <Output<Signal> as ::substrate::types::FlatLen>::len(b)
//                 }
//                 EnumIo::C(__macrotools_derive_field0) => {
//                     <NamedStructIo as ::substrate::types::FlatLen>::len(__macrotools_derive_field0)
//                 }
//             }
//         }
//     }
// }
// impl ::substrate::types::Flatten<::substrate::types::Direction> for EnumIo
// where
//     Input<Signal>: ::substrate::types::Flatten<::substrate::types::Direction>,
//     Output<Signal>: ::substrate::types::Flatten<::substrate::types::Direction>,
//     NamedStructIo: ::substrate::types::Flatten<::substrate::types::Direction>,
// {
//     fn flatten<E>(&self, __substrate_output_sink: &mut E)
//     where
//         E: ::std::iter::Extend<::substrate::types::Direction>,
//     {
//         {
//             match self {
//                 EnumIo::A => {}
//                 EnumIo::B { a, b } => {
//                     <Input<
//                                 Signal,
//                             > as ::substrate::types::Flatten<
//                                 ::substrate::types::Direction,
//                             >>::flatten(a, __substrate_output_sink);
//                     <Output<Signal> as ::substrate::types::Flatten<
//                         ::substrate::types::Direction,
//                     >>::flatten(b, __substrate_output_sink);
//                 }
//                 EnumIo::C(__macrotools_derive_field0) => {
//                     <NamedStructIo as ::substrate::types::Flatten<
//                                 ::substrate::types::Direction,
//                             >>::flatten(
//                                 __macrotools_derive_field0,
//                                 __substrate_output_sink,
//                             );
//                 }
//             }
//         }
//     }
// }
// impl ::substrate::types::HasBundleKind for EnumIo
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
// {
//     type BundleKind = EnumIoKind;
//     fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
//         {
//             match self {
//                 EnumIo::A => EnumIoKind::A,
//                 EnumIo::B { a, b } => EnumIoKind::B {
//                     a: <Input<Signal> as ::substrate::types::HasBundleKind>::kind(a),
//                     b: <Output<Signal> as ::substrate::types::HasBundleKind>::kind(b),
//                 },
//                 EnumIo::C(__macrotools_derive_field0) => {
//                     EnumIoKind::C(<NamedStructIo as ::substrate::types::HasBundleKind>::kind(
//                         __macrotools_derive_field0,
//                     ))
//                 }
//             }
//         }
//     }
// }
// impl ::substrate::types::codegen::ViewSource for EnumIo
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
// {
//     type Kind = ::substrate::types::codegen::FromOther;
//     type Source = EnumIoKind;
// }
// impl ::std::clone::Clone for EnumIoKind
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::clone::Clone,
//     <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::clone::Clone,
//     <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind: ::std::clone::Clone,
// {
//     fn clone(&self) -> Self {
//         {
//             match self {
//                         EnumIoKind::A => Self::A,
//                         EnumIoKind::B { a, b } => {
//                             Self::B {
//                                 a: <<Input<
//                                     Signal,
//                                 > as ::substrate::types::HasBundleKind>::BundleKind as ::std::clone::Clone>::clone(
//                                     a,
//                                 ),
//                                 b: <<Output<
//                                     Signal,
//                                 > as ::substrate::types::HasBundleKind>::BundleKind as ::std::clone::Clone>::clone(
//                                     b,
//                                 ),
//                             }
//                         }
//                         EnumIoKind::C(__macrotools_derive_field0) => {
//                             Self::C(
//                                 <<NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind as ::std::clone::Clone>::clone(
//                                     __macrotools_derive_field0,
//                                 ),
//                             )
//                         }
//                     }
//         }
//     }
// }
// impl ::std::fmt::Debug for EnumIoKind
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
//     <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
//     <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
// {
//     fn fmt(&self, __substrate_f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
//         {
//             match self {
//                 EnumIoKind::A => __substrate_f.debug_struct("EnumIoKind").finish(),
//                 EnumIoKind::B { a, b } => __substrate_f
//                     .debug_struct("EnumIoKind")
//                     .field("a", a)
//                     .field("b", b)
//                     .finish(),
//                 EnumIoKind::C(__macrotools_derive_field0) => __substrate_f
//                     .debug_struct("EnumIoKind")
//                     .field("elem0", __macrotools_derive_field0)
//                     .finish(),
//             }
//         }
//     }
// }
// impl ::std::cmp::PartialEq for EnumIoKind
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
//     <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
//     <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
// {
//     fn eq(&self, __substrate_other: &Self) -> bool {
//         {
//             match (self, __substrate_other) {
//                         (EnumIoKind::A, EnumIoKind::A) => true,
//                         (
//                             EnumIoKind::B { a: a_0, b: b_0 },
//                             EnumIoKind::B { a: a_1, b: b_1 },
//                         ) => {
//                             <<Input<
//                                 Signal,
//                             > as ::substrate::types::HasBundleKind>::BundleKind as ::std::cmp::PartialEq>::eq(
//                                 a_0,
//                                 a_1,
//                             )
//                                 && <<Output<
//                                     Signal,
//                                 > as ::substrate::types::HasBundleKind>::BundleKind as ::std::cmp::PartialEq>::eq(
//                                     b_0,
//                                     b_1,
//                                 )
//                         }
//                         (
//                             EnumIoKind::C(__macrotools_derive_field0),
//                             EnumIoKind::C(__macrotools_derive_field1),
//                         ) => {
//                             <<NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind as ::std::cmp::PartialEq>::eq(
//                                 __macrotools_derive_field0,
//                                 __macrotools_derive_field1,
//                             )
//                         }
//                         _ => false,
//                     }
//         }
//     }
// }
// impl ::std::cmp::Eq for EnumIoKind
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::cmp::Eq,
//     <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::cmp::Eq,
//     <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind: ::std::cmp::Eq,
// {
// }
// impl ::substrate::types::FlatLen for EnumIoKind
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::FlatLen,
//     <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::FlatLen,
//     <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::FlatLen,
// {
//     fn len(&self) -> usize {
//         {
//             match self {
//                         EnumIoKind::A => 0,
//                         EnumIoKind::B { a, b } => {
//                             <<Input<
//                                 Signal,
//                             > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::FlatLen>::len(
//                                 a,
//                             )
//                                 + <<Output<
//                                     Signal,
//                                 > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::FlatLen>::len(
//                                     b,
//                                 )
//                         }
//                         EnumIoKind::C(__macrotools_derive_field0) => {
//                             <<NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::FlatLen>::len(
//                                 __macrotools_derive_field0,
//                             )
//                         }
//                     }
//         }
//     }
// }
// impl ::substrate::types::HasBundleKind for EnumIoKind
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::HasBundleKind,
//     <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::HasBundleKind,
//     <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::HasBundleKind,
// {
//     type BundleKind = EnumIoKind;
//     fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
//         {
//             match self {
//                         EnumIoKind::A => EnumIoKind::A,
//                         EnumIoKind::B { a, b } => {
//                             EnumIoKind::B {
//                                 a: <<Input<
//                                     Signal,
//                                 > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasBundleKind>::kind(
//                                     a,
//                                 ),
//                                 b: <<Output<
//                                     Signal,
//                                 > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasBundleKind>::kind(
//                                     b,
//                                 ),
//                             }
//                         }
//                         EnumIoKind::C(__macrotools_derive_field0) => {
//                             EnumIoKind::C(
//                                 <<NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasBundleKind>::kind(
//                                     __macrotools_derive_field0,
//                                 ),
//                             )
//                         }
//                     }
//         }
//     }
// }
// impl ::substrate::types::codegen::ViewSource for EnumIoKind
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::HasBundleKind,
//     <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::HasBundleKind,
//     <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::HasBundleKind,
// {
//     type Kind = ::substrate::types::codegen::FromSelf;
//     type Source = Self;
// }
// impl ::substrate::types::HasNameTree for EnumIoKind
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::HasBundleKind,
//     <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::HasBundleKind,
//     <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::HasBundleKind,
// {
//     fn names(&self) -> ::std::option::Option<::std::vec::Vec<::substrate::types::NameTree>> {
//         let v: ::std::vec::Vec<::substrate::types::NameTree> = {
//             match self {
//                 EnumIoKind::A => vec![],
//                 EnumIoKind::B { a, b } => {
//                     vec![(arcstr::literal!("hi"), None)]
//                 }
//                 EnumIoKind::C(__macrotools_derive_field0) => {
//                     vec![]
//                 }
//             }
//         }
//         .into_iter()
//         .filter_map(|(frag, children)| children.map(|c| ::substrate::types::NameTree::new(frag, c)))
//         .collect();
//         if v.len() == 0 {
//             None
//         } else {
//             Some(v)
//         }
//     }
// }
// /// An enum Io.
// pub enum EnumIoView<__substrate_V>
// where
//     Input<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     Output<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     NamedStructIo: ::substrate::types::codegen::HasView<__substrate_V>,
// {
//     A,
//     B {
//         a: <Input<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View,
//         b: <Output<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View,
//     },
//     C(<NamedStructIo as ::substrate::types::codegen::HasView<__substrate_V>>::View),
// }
// impl<__substrate_V> ::substrate::types::codegen::ViewSource for EnumIoView<__substrate_V>
// where
//     Input<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     Output<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     NamedStructIo: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind,
//     <Output<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind,
//     <NamedStructIo as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind,
// {
//     type Kind = ::substrate::types::codegen::FromSelf;
//     type Source = Self;
// }
// impl<__substrate_V> ::substrate::types::HasBundleKind for EnumIoView<__substrate_V>
// where
//     Input<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     Output<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     NamedStructIo: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind<
//             BundleKind = <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
//         >,
//     <Output<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind<
//             BundleKind = <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
//         >,
//     <NamedStructIo as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind<
//             BundleKind = <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind,
//         >,
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind,
//     <Output<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind,
//     <NamedStructIo as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind,
// {
//     type BundleKind = EnumIoKind;
//     fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
//         {
//             match self {
//                         EnumIoView::A => EnumIoKind::A,
//                         EnumIoView::B { a, b } => {
//                             EnumIoKind::B {
//                                 a: <<Input<
//                                     Signal,
//                                 > as ::substrate::types::codegen::HasView<
//                                     __substrate_V,
//                                 >>::View as ::substrate::types::HasBundleKind>::kind(a),
//                                 b: <<Output<
//                                     Signal,
//                                 > as ::substrate::types::codegen::HasView<
//                                     __substrate_V,
//                                 >>::View as ::substrate::types::HasBundleKind>::kind(b),
//                             }
//                         }
//                         EnumIoView::C(__macrotools_derive_field0) => {
//                             EnumIoKind::C(
//                                 <<NamedStructIo as ::substrate::types::codegen::HasView<
//                                     __substrate_V,
//                                 >>::View as ::substrate::types::HasBundleKind>::kind(
//                                     __macrotools_derive_field0,
//                                 ),
//                             )
//                         }
//                     }
//         }
//     }
// }
// impl<__substrate_V> ::substrate::types::FlatLen for EnumIoView<__substrate_V>
// where
//     Input<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     Output<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     NamedStructIo: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::FlatLen,
//     <Output<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::FlatLen,
//     <NamedStructIo as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::FlatLen,
// {
//     fn len(&self) -> usize {
//         {
//             match self {
//                         EnumIoView::A => 0,
//                         EnumIoView::B { a, b } => {
//                             <<Input<
//                                 Signal,
//                             > as ::substrate::types::codegen::HasView<
//                                 __substrate_V,
//                             >>::View as ::substrate::types::FlatLen>::len(a)
//                                 + <<Output<
//                                     Signal,
//                                 > as ::substrate::types::codegen::HasView<
//                                     __substrate_V,
//                                 >>::View as ::substrate::types::FlatLen>::len(b)
//                         }
//                         EnumIoView::C(__macrotools_derive_field0) => {
//                             <<NamedStructIo as ::substrate::types::codegen::HasView<
//                                 __substrate_V,
//                             >>::View as ::substrate::types::FlatLen>::len(
//                                 __macrotools_derive_field0,
//                             )
//                         }
//                     }
//         }
//     }
// }
// impl<__substrate_V, __substrate_F> ::substrate::types::Flatten<__substrate_F>
//     for EnumIoView<__substrate_V>
// where
//     Input<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     Output<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     NamedStructIo: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Flatten<__substrate_F>,
//     <Output<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Flatten<__substrate_F>,
//     <NamedStructIo as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Flatten<__substrate_F>,
// {
//     fn flatten<E>(&self, __substrate_output_sink: &mut E)
//     where
//         E: ::std::iter::Extend<__substrate_F>,
//     {
//         {
//             match self {
//                 EnumIoView::A => {}
//                 EnumIoView::B { a, b } => {
//                     <<Input<
//                                 Signal,
//                             > as ::substrate::types::codegen::HasView<
//                                 __substrate_V,
//                             >>::View as ::substrate::types::Flatten<
//                                 __substrate_F,
//                             >>::flatten(a, __substrate_output_sink);
//                     <<Output<
//                                 Signal,
//                             > as ::substrate::types::codegen::HasView<
//                                 __substrate_V,
//                             >>::View as ::substrate::types::Flatten<
//                                 __substrate_F,
//                             >>::flatten(b, __substrate_output_sink);
//                 }
//                 EnumIoView::C(__macrotools_derive_field0) => {
//                     <<NamedStructIo as ::substrate::types::codegen::HasView<
//                                 __substrate_V,
//                             >>::View as ::substrate::types::Flatten<
//                                 __substrate_F,
//                             >>::flatten(
//                                 __macrotools_derive_field0,
//                                 __substrate_output_sink,
//                             );
//                 }
//             }
//         }
//     }
// }
// impl<__substrate_V, __substrate_S> ::substrate::types::Unflatten<EnumIoKind, __substrate_S>
//     for EnumIoView<__substrate_V>
// where
//     Input<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     Output<Signal>: ::substrate::types::codegen::HasView<__substrate_V>,
//     NamedStructIo: ::substrate::types::codegen::HasView<__substrate_V>,
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     <Input<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Unflatten<
//             <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
//             __substrate_S,
//         >,
//     <Output<Signal> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Unflatten<
//             <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
//             __substrate_S,
//         >,
//     <NamedStructIo as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Unflatten<
//             <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind,
//             __substrate_S,
//         >,
// {
//     fn unflatten<__substrate_I>(
//         __substrate_data: &EnumIoKind,
//         __substrate_source: &mut __substrate_I,
//     ) -> Option<Self>
//     where
//         __substrate_I: Iterator<Item = __substrate_S>,
//     {
//         ::std::option::Option::Some({
//             match __substrate_data {
//                 EnumIoKind::A => EnumIoView::A,
//                 EnumIoKind::B { a, b } => {
//                     EnumIoView::B {
//                         a: <<Input<Signal> as ::substrate::types::codegen::HasView<
//                             __substrate_V,
//                         >>::View as ::substrate::types::Unflatten<
//                             <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
//                             __substrate_S,
//                         >>::unflatten(&a, __substrate_source)?,
//                         b: <<Output<Signal> as ::substrate::types::codegen::HasView<
//                             __substrate_V,
//                         >>::View as ::substrate::types::Unflatten<
//                             <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
//                             __substrate_S,
//                         >>::unflatten(&b, __substrate_source)?,
//                     }
//                 }
//                 EnumIoKind::C(__macrotools_derive_field0) => {
//                     EnumIoView::C(<<NamedStructIo as ::substrate::types::codegen::HasView<
//                         __substrate_V,
//                     >>::View as ::substrate::types::Unflatten<
//                         <NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind,
//                         __substrate_S,
//                     >>::unflatten(
//                         &__macrotools_derive_field0, __substrate_source
//                     )?)
//                 }
//             }
//         })
//     }
// }
// impl ::substrate::types::schematic::SchematicBundleKind for EnumIoKind
// where
//     Input<Signal>: ::substrate::types::HasBundleKind,
//     Output<Signal>: ::substrate::types::HasBundleKind,
//     NamedStructIo: ::substrate::types::HasBundleKind,
//     Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     NamedStructIo: ::substrate::types::codegen::HasSchematicBundleKindViews,
// {
//     type NodeBundle = EnumIoView<::substrate::types::codegen::NodeBundle>;
//     type TerminalBundle = EnumIoView<::substrate::types::codegen::TerminalBundle>;
//     fn terminal_view(
//         cell: ::substrate::schematic::CellId,
//         cell_io: &::substrate::types::schematic::NodeBundle<Self>,
//         instance: ::substrate::schematic::InstanceId,
//         instance_io: &::substrate::types::schematic::NodeBundle<Self>,
//     ) -> ::substrate::types::schematic::TerminalBundle<Self> {
//         {
//             match (cell_io, instance_io) {
//                         (EnumIoView::A, EnumIoView::A) => EnumIoView::A,
//                         (
//                             EnumIoView::B { a: a_0, b: b_0 },
//                             EnumIoView::B { a: a_1, b: b_1 },
//                         ) => {
//                             EnumIoView::B {
//                                 a: <<Input<
//                                     Signal,
//                                 > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::SchematicBundleKind>::terminal_view(
//                                     cell,
//                                     a_0,
//                                     instance,
//                                     a_1,
//                                 ),
//                                 b: <<Output<
//                                     Signal,
//                                 > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::SchematicBundleKind>::terminal_view(
//                                     cell,
//                                     b_0,
//                                     instance,
//                                     b_1,
//                                 ),
//                             }
//                         }
//                         (
//                             EnumIoView::C(__macrotools_derive_field0),
//                             EnumIoView::C(__macrotools_derive_field1),
//                         ) => {
//                             EnumIoView::C(
//                                 <<NamedStructIo as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::SchematicBundleKind>::terminal_view(
//                                     cell,
//                                     __macrotools_derive_field0,
//                                     instance,
//                                     __macrotools_derive_field1,
//                                 ),
//                             )
//                         }
//                         _ => {
//                             panic!("cell and instance IOs are not the same kind");
//                         }
//                     }
//         }
//     }
// }
// impl ::substrate::schematic::HasNestedView for EnumIoView<::substrate::types::codegen::NodeBundle>
// where
//     Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     NamedStructIo: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     <Input<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
//     <Output<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
//     <NamedStructIo as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
// {
//     type NestedView = EnumIoView<::substrate::types::codegen::NestedNodeBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         {
//             match self {
//                 EnumIoView::A => EnumIoView::A,
//                 EnumIoView::B { a, b } => EnumIoView::B {
//                     a: <<Input<Signal> as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NodeBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &a,
//                         __substrate_parent,
//                     ),
//                     b: <<Output<Signal> as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NodeBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &b,
//                         __substrate_parent,
//                     ),
//                 },
//                 EnumIoView::C(__macrotools_derive_field0) => {
//                     EnumIoView::C(<<NamedStructIo as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NodeBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &__macrotools_derive_field0,
//                         __substrate_parent,
//                     ))
//                 }
//             }
//         }
//     }
// }
// impl ::substrate::schematic::HasNestedView
//     for EnumIoView<::substrate::types::codegen::TerminalBundle>
// where
//     Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     NamedStructIo: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     <Input<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::TerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
//     <Output<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::TerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
//     <NamedStructIo as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::TerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
// {
//     type NestedView = EnumIoView<::substrate::types::codegen::NestedTerminalBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         {
//             match self {
//                 EnumIoView::A => EnumIoView::A,
//                 EnumIoView::B { a, b } => EnumIoView::B {
//                     a: <<Input<Signal> as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::TerminalBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &a,
//                         __substrate_parent,
//                     ),
//                     b: <<Output<Signal> as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::TerminalBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &b,
//                         __substrate_parent,
//                     ),
//                 },
//                 EnumIoView::C(__macrotools_derive_field0) => {
//                     EnumIoView::C(<<NamedStructIo as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::TerminalBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &__macrotools_derive_field0,
//                         __substrate_parent,
//                     ))
//                 }
//             }
//         }
//     }
// }
// impl ::substrate::schematic::HasNestedView
//     for EnumIoView<::substrate::types::codegen::NestedNodeBundle>
// where
//     Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     NamedStructIo: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     <Input<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <Input<Signal> as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedNodeBundle,
//         >>::View,
//     >,
//     <Output<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <Output<Signal> as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedNodeBundle,
//         >>::View,
//     >,
//     <NamedStructIo as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <NamedStructIo as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedNodeBundle,
//         >>::View,
//     >,
//     <Input<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
//     <Output<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
//     <NamedStructIo as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
// {
//     type NestedView = EnumIoView<::substrate::types::codegen::NestedNodeBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         {
//             match self {
//                 EnumIoView::A => EnumIoView::A,
//                 EnumIoView::B { a, b } => EnumIoView::B {
//                     a: <<Input<Signal> as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NestedNodeBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &a,
//                         __substrate_parent,
//                     ),
//                     b: <<Output<Signal> as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NestedNodeBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &b,
//                         __substrate_parent,
//                     ),
//                 },
//                 EnumIoView::C(__macrotools_derive_field0) => {
//                     EnumIoView::C(<<NamedStructIo as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NestedNodeBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &__macrotools_derive_field0,
//                         __substrate_parent,
//                     ))
//                 }
//             }
//         }
//     }
// }
// impl ::substrate::schematic::HasNestedView
//     for EnumIoView<::substrate::types::codegen::NestedTerminalBundle>
// where
//     Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     NamedStructIo: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     <Input<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <Input<Signal> as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedTerminalBundle,
//         >>::View,
//     >,
//     <Output<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <Output<Signal> as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedTerminalBundle,
//         >>::View,
//     >,
//     <NamedStructIo as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <NamedStructIo as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedTerminalBundle,
//         >>::View,
//     >,
//     <Input<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
//     <Output<Signal> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
//     <NamedStructIo as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
// {
//     type NestedView = EnumIoView<::substrate::types::codegen::NestedTerminalBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         {
//             match self {
//                 EnumIoView::A => EnumIoView::A,
//                 EnumIoView::B { a, b } => EnumIoView::B {
//                     a: <<Input<Signal> as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NestedTerminalBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &a,
//                         __substrate_parent,
//                     ),
//                     b: <<Output<Signal> as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NestedTerminalBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &b,
//                         __substrate_parent,
//                     ),
//                 },
//                 EnumIoView::C(__macrotools_derive_field0) => {
//                     EnumIoView::C(<<NamedStructIo as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NestedTerminalBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &__macrotools_derive_field0,
//                         __substrate_parent,
//                     ))
//                 }
//             }
//         }
//     }
// }
// impl EnumIoView<::substrate::types::codegen::NodeBundle>
// where
//     Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     NamedStructIo: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     Self: ::substrate::types::HasBundleKind<
//         BundleKind: ::substrate::types::schematic::SchematicBundleKind<NodeBundle = Self>,
//     >,
// {
//     /// Views this node bundle as a node bundle of a different kind.
//     pub fn view_as<
//         __substrate_T: ::substrate::types::HasBundleKind<
//             BundleKind: ::substrate::types::schematic::SchematicBundleKind,
//         >,
//     >(
//         &self,
//     ) -> ::substrate::types::schematic::NodeBundle<
//         <__substrate_T as ::substrate::types::HasBundleKind>::BundleKind,
//     >
//     where
//         <Self as ::substrate::types::HasBundleKind>::BundleKind:
//             ::substrate::types::schematic::DataView<
//                 <__substrate_T as ::substrate::types::HasBundleKind>::BundleKind,
//             >,
//     {
//         <<Self as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::DataView<
//                     <__substrate_T as ::substrate::types::HasBundleKind>::BundleKind,
//                 >>::view_nodes_as(self)
//     }
// }
// impl EnumIoView<::substrate::types::codegen::TerminalBundle>
// where
//     Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     NamedStructIo: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     Self: ::substrate::types::HasBundleKind<
//         BundleKind: ::substrate::types::schematic::SchematicBundleKind<TerminalBundle = Self>,
//     >,
// {
//     /// Views this node bundle as a node bundle of a different kind.
//     pub fn view_as<
//         __substrate_T: ::substrate::types::HasBundleKind<
//             BundleKind: ::substrate::types::schematic::SchematicBundleKind,
//         >,
//     >(
//         &self,
//     ) -> ::substrate::types::schematic::TerminalBundle<
//         <__substrate_T as ::substrate::types::HasBundleKind>::BundleKind,
//     >
//     where
//         <Self as ::substrate::types::HasBundleKind>::BundleKind:
//             ::substrate::types::schematic::DataView<
//                 <__substrate_T as ::substrate::types::HasBundleKind>::BundleKind,
//             >,
//     {
//         <<Self as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::DataView<
//                     <__substrate_T as ::substrate::types::HasBundleKind>::BundleKind,
//                 >>::view_terminals_as(self)
//     }
// }

/// Takes an IO type.
///
/// Used to validate that a given type implements `Io`.
fn takes_io<T: Io>() -> usize {
    std::mem::size_of::<T>()
}

#[crate::test]
fn generic_io_implements_io() {
    takes_io::<GenericIo<Signal>>();
    takes_io::<GenericIo<NamedStructIo>>();
    takes_io::<GenericIo<TupleIo>>();
    takes_io::<GenericIo<EnumIo>>();
}

#[crate::test]
fn named_struct_io_implements_io() {
    takes_io::<NamedStructIo>();
}

#[crate::test]
fn tuple_io_implements_io() {
    takes_io::<TupleIo>();
}
