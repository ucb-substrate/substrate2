use substrate::types::Io;
use substrate::types::{Input, Output, Signal};

use super::codegen::{HasView, NodeBundle, TerminalBundle};
use super::HasBundleKind;

// TODO: uncomment
/// An Io with a generic type parameter.
#[derive(Debug, Clone, Io)]
pub struct GenericIo<T> {
    /// A single input field.
    pub signal: Input<T>,
}

// /// An Io with a generic type parameter.
// pub struct GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
// {
//     /// A single input field.
//     pub signal: <Input<T> as ::substrate::types::HasBundleKind>::BundleKind,
// }
// impl<T> ::substrate::types::FlatLen for GenericIo<T>
// where
//     Input<T>: ::substrate::types::FlatLen,
// {
//     fn len(&self) -> usize {
//         <Input<T> as ::substrate::types::FlatLen>::len(&self.signal)
//     }
// }
// impl<T> ::substrate::types::Flatten<::substrate::types::Direction> for GenericIo<T>
// where
//     Input<T>: ::substrate::types::Flatten<::substrate::types::Direction>,
// {
//     fn flatten<E>(&self, __substrate_output_sink: &mut E)
//     where
//         E: ::std::iter::Extend<::substrate::types::Direction>,
//     {
//         <Input<T> as ::substrate::types::Flatten<::substrate::types::Direction>>::flatten(
//             &self.signal,
//             __substrate_output_sink,
//         );
//     }
// }
// impl<T> ::substrate::types::HasBundleKind for GenericIo<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
// {
//     type BundleKind = GenericIoKind<T>;
//     fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
//         GenericIoKind {
//             signal: <Input<T> as ::substrate::types::HasBundleKind>::kind(&self.signal),
//         }
//     }
// }
// impl<T> ::substrate::types::codegen::ViewSource for GenericIo<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
// {
//     type Kind = ::substrate::types::codegen::FromOther;
//     type Source = GenericIoKind<T>;
// }
// impl<T> ::std::clone::Clone for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::std::clone::Clone,
// {
//     fn clone(&self) -> Self {
//         Self {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::HasBundleKind>::BundleKind as ::std::clone::Clone>::clone(
//                         &self.signal,
//                     ),
//                 }
//     }
// }
// impl<T> ::std::fmt::Debug for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
// {
//     fn fmt(&self, __substrate_f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
//         __substrate_f
//             .debug_struct("GenericIoKind")
//             .field("signal", &self.signal)
//             .finish()
//     }
// }
// impl<T> ::std::cmp::PartialEq for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
// {
//     fn eq(&self, __substrate_other: &Self) -> bool {
//         <<Input<T> as ::substrate::types::HasBundleKind>::BundleKind as ::std::cmp::PartialEq>::eq(
//             &self.signal,
//             &__substrate_other.signal,
//         )
//     }
// }
// impl<T> ::std::cmp::Eq for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::std::cmp::Eq,
// {
// }
// impl<T> ::substrate::types::FlatLen for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::FlatLen,
// {
//     fn len(&self) -> usize {
//         <<Input<
//                     T,
//                 > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::FlatLen>::len(
//                     &self.signal,
//                 )
//     }
// }
// impl<T> ::substrate::types::HasBundleKind for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::HasBundleKind,
// {
//     type BundleKind = GenericIoKind<T>;
//     fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
//         GenericIoKind {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasBundleKind>::kind(
//                         &self.signal,
//                     ),
//                 }
//     }
// }
// impl<T> ::substrate::types::codegen::ViewSource for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::HasBundleKind,
// {
//     type Kind = ::substrate::types::codegen::FromSelf;
//     type Source = Self;
// }
// impl<T> ::substrate::types::HasNameTree for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::HasBundleKind,
// {
//     fn names(&self) -> ::std::option::Option<::std::vec::Vec<::substrate::types::NameTree>> {
//         let v: ::std::vec::Vec<::substrate::types::NameTree> = [
//                     (
//                         arcstr::literal!("signal"),
//                         <<Input<
//                             T,
//                         > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasNameTree>::names(
//                             &&self.signal,
//                         ),
//                     ),
//                 ]
//                     .into_iter()
//                     .filter_map(|(frag, children)| {
//                         children.map(|c| ::substrate::types::NameTree::new(frag, c))
//                     })
//                     .collect();
//         if v.len() == 0 {
//             None
//         } else {
//             Some(v)
//         }
//     }
// }
// /// An Io with a generic type parameter.
// pub struct GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
// {
//     /// A single input field.
//     pub signal: <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View,
// }
// impl<T, __substrate_V> ::substrate::types::codegen::ViewSource for GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind,
// {
//     type Kind = ::substrate::types::codegen::FromSelf;
//     type Source = Self;
// }
// impl<T, __substrate_V> ::substrate::types::HasBundleKind for GenericIoView<T, __substrate_V>
// where
//     Input<T>: HasBundleKind,
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind<
//             BundleKind = <Input<T> as ::substrate::types::HasBundleKind>::BundleKind,
//         >,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind,
// {
//     type BundleKind = GenericIoKind<T>;
//     fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
//         GenericIoKind {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::codegen::HasView<
//                         __substrate_V,
//                     >>::View as ::substrate::types::HasBundleKind>::kind(&self.signal),
//                 }
//     }
// }
// impl<T, __substrate_V> ::substrate::types::FlatLen for GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::FlatLen,
// {
//     fn len(&self) -> usize {
//         <<Input<
//                     T,
//                 > as ::substrate::types::codegen::HasView<
//                     __substrate_V,
//                 >>::View as ::substrate::types::FlatLen>::len(&self.signal)
//     }
// }
// impl<T, __substrate_V, __substrate_F> ::substrate::types::Flatten<__substrate_F>
//     for GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Flatten<__substrate_F>,
// {
//     fn flatten<E>(&self, __substrate_output_sink: &mut E)
//     where
//         E: ::std::iter::Extend<__substrate_F>,
//     {
//         <<Input<
//                     T,
//                 > as ::substrate::types::codegen::HasView<
//                     __substrate_V,
//                 >>::View as ::substrate::types::Flatten<
//                     __substrate_F,
//                 >>::flatten(&self.signal, __substrate_output_sink);
//     }
// }
// impl<T, __substrate_V, __substrate_S> ::substrate::types::Unflatten<GenericIoKind<T>, __substrate_S>
//     for GenericIoView<T, __substrate_V>
// where
//     Input<T>: HasBundleKind,
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Unflatten<
//             <Input<T> as ::substrate::types::HasBundleKind>::BundleKind,
//             __substrate_S,
//         >,
// {
//     fn unflatten<__substrate_I>(
//         __substrate_data: &GenericIoKind<T>,
//         __substrate_source: &mut __substrate_I,
//     ) -> Option<Self>
//     where
//         __substrate_I: Iterator<Item = __substrate_S>,
//     {
//         ::std::option::Option::Some(GenericIoView {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::codegen::HasView<
//                         __substrate_V,
//                     >>::View as ::substrate::types::Unflatten<
//                         <Input<T> as ::substrate::types::HasBundleKind>::BundleKind,
//                         __substrate_S,
//                     >>::unflatten(&&__substrate_data.signal, __substrate_source)?,
//                 })
//     }
// }
// impl<T> ::substrate::types::schematic::SchematicBundleKind for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     Input<T>: ::substrate::types::codegen::HasSchematicBundleKindViews,
// {
//     type NodeBundle = GenericIoView<T, ::substrate::types::codegen::NodeBundle>;
//     type TerminalBundle = GenericIoView<T, ::substrate::types::codegen::TerminalBundle>;
//     fn terminal_view(
//         cell: ::substrate::schematic::CellId,
//         cell_io: &::substrate::types::schematic::NodeBundle<Self>,
//         instance: ::substrate::schematic::InstanceId,
//         instance_io: &::substrate::types::schematic::NodeBundle<Self>,
//     ) -> ::substrate::types::schematic::TerminalBundle<Self> {
//         GenericIoView {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::SchematicBundleKind>::terminal_view(
//                         cell,
//                         &cell_io.signal,
//                         instance,
//                         &instance_io.signal,
//                     ),
//                 }
//     }
// }
// impl<T> ::substrate::schematic::HasNestedView
//         for GenericIoView<T, ::substrate::types::codegen::NodeBundle>
//         where
//             Input<T>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//             <Input<
//                 T,
//             > as ::substrate::types::codegen::HasView<
//                 ::substrate::types::codegen::NodeBundle,
//             >>::View: ::substrate::schematic::HasNestedView,
//         {
//             type NestedView = GenericIoView<
//                 T,
//                 ::substrate::types::codegen::NestedNodeBundle,
//             >;
//             fn nested_view(
//                 &self,
//                 __substrate_parent: &::substrate::schematic::InstancePath,
//             ) -> ::substrate::schematic::NestedView<Self> {
//                 GenericIoView {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::codegen::HasView<
//                         ::substrate::types::codegen::NodeBundle,
//                     >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                         &&self.signal,
//                         __substrate_parent,
//                     ),
//                 }
//             }
//         }
// impl<T> ::substrate::schematic::HasNestedView
//     for GenericIoView<T, ::substrate::types::codegen::TerminalBundle>
// where
//     Input<T>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     <Input<T> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::TerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
// {
//     type NestedView = GenericIoView<T, ::substrate::types::codegen::NestedTerminalBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         GenericIoView {
//             signal: <<Input<T> as ::substrate::types::codegen::HasView<
//                 ::substrate::types::codegen::TerminalBundle,
//             >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                 &&self.signal,
//                 __substrate_parent,
//             ),
//         }
//     }
// }
// impl<T> ::substrate::schematic::HasNestedView
//     for GenericIoView<T, ::substrate::types::codegen::NestedNodeBundle>
// where
//     Input<T>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     <Input<T> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <Input<T> as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedNodeBundle,
//         >>::View,
//     >,
//     <Input<T> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
// {
//     type NestedView = GenericIoView<T, ::substrate::types::codegen::NestedNodeBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         GenericIoView {
//             signal: <<Input<T> as ::substrate::types::codegen::HasView<
//                 ::substrate::types::codegen::NestedNodeBundle,
//             >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                 &&self.signal,
//                 __substrate_parent,
//             ),
//         }
//     }
// }
// impl<T> ::substrate::schematic::HasNestedView
//     for GenericIoView<T, ::substrate::types::codegen::NestedTerminalBundle>
// where
//     Input<T>: ::substrate::types::codegen::HasSchematicBundleKindViews,
//     <Input<T> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <Input<T> as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedTerminalBundle,
//         >>::View,
//     >,
//     <Input<T> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView,
// {
//     type NestedView = GenericIoView<T, ::substrate::types::codegen::NestedTerminalBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         GenericIoView {
//             signal: <<Input<T> as ::substrate::types::codegen::HasView<
//                 ::substrate::types::codegen::NestedTerminalBundle,
//             >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                 &&self.signal,
//                 __substrate_parent,
//             ),
//         }
//     }
// }
// impl<T> GenericIoView<T, ::substrate::types::codegen::NodeBundle>
// where
//     Input<T>: ::substrate::types::codegen::HasSchematicBundleKindViews,
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
// impl<T> GenericIoView<T, ::substrate::types::codegen::TerminalBundle>
// where
//     Input<T>: ::substrate::types::codegen::HasSchematicBundleKindViews,
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

// /// An Io with a generic type parameter.
// pub struct GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
// {
//     /// A single input field.
//     pub signal: <Input<T> as ::substrate::types::HasBundleKind>::BundleKind,
// }
//
// impl<T> Clone for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: Clone,
// {
//     fn clone(&self) -> Self {
//         unimplemented!()
//     }
// }
//
// impl<T> std::fmt::Debug for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: std::fmt::Debug,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         unimplemented!()
//     }
// }
//
// impl<T> std::cmp::PartialEq for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: std::cmp::PartialEq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         unimplemented!()
//     }
// }
//
// impl<T> std::cmp::Eq for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: std::cmp::Eq,
// {
//     fn assert_receiver_is_total_eq(&self) {
//         unimplemented!()
//     }
// }
//
// impl<T> ::substrate::types::FlatLen for GenericIo<T>
// where
//     Input<T>: ::substrate::types::FlatLen,
// {
//     fn len(&self) -> usize {
//         <Input<T> as ::substrate::types::FlatLen>::len(&self.signal)
//     }
// }
// impl<T> ::substrate::types::Flatten<::substrate::types::Direction> for GenericIo<T>
// where
//     Input<T>: ::substrate::types::Flatten<::substrate::types::Direction>,
// {
//     fn flatten<E>(&self, __substrate_output_sink: &mut E)
//     where
//         E: ::std::iter::Extend<::substrate::types::Direction>,
//     {
//         <Input<T> as ::substrate::types::Flatten<::substrate::types::Direction>>::flatten(
//             &self.signal,
//             __substrate_output_sink,
//         );
//     }
// }
// impl<T> ::substrate::types::HasBundleKind for GenericIo<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
// {
//     type BundleKind = GenericIoKind<T>;
//     fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
//         GenericIoKind {
//             signal: <Input<T> as ::substrate::types::HasBundleKind>::kind(&self.signal),
//         }
//     }
// }
// impl<T> ::substrate::types::codegen::ViewSource for GenericIo<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
// {
//     type Kind = ::substrate::types::codegen::FromOther;
//     type Source = GenericIoKind<T>;
// }
// impl<T> ::substrate::types::FlatLen for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::FlatLen,
// {
//     fn len(&self) -> usize {
//         <<Input<
//                     T,
//                 > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::FlatLen>::len(
//                     &self.signal,
//                 )
//     }
// }
// impl<T> ::substrate::types::HasBundleKind for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::HasBundleKind,
// {
//     type BundleKind = GenericIoKind<T>;
//     fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
//         GenericIoKind {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasBundleKind>::kind(
//                         &self.signal,
//                     ),
//                 }
//     }
// }
// impl<T> ::substrate::types::codegen::ViewSource for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
// {
//     type Kind = ::substrate::types::codegen::FromSelf;
//     type Source = Self;
// }
// impl<T> ::substrate::types::HasNameTree for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     <Input<T> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::HasBundleKind,
// {
//     fn names(&self) -> ::std::option::Option<::std::vec::Vec<::substrate::types::NameTree>> {
//         let v: ::std::vec::Vec<::substrate::types::NameTree> = [
//                     (
//                         arcstr::literal!("signal"),
//                         <<Input<
//                             T,
//                         > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasNameTree>::names(
//                             &&self.signal,
//                         ),
//                     ),
//                 ]
//                     .into_iter()
//                     .filter_map(|(frag, children)| {
//                         children.map(|c| ::substrate::types::NameTree::new(frag, c))
//                     })
//                     .collect();
//         if v.len() == 0 {
//             None
//         } else {
//             Some(v)
//         }
//     }
// }
// /// An Io with a generic type parameter.
// pub struct GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
// {
//     /// A single input field.
//     pub signal: <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View,
// }
// impl<T, __substrate_V> ::substrate::types::codegen::ViewSource for GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
// {
//     type Kind = ::substrate::types::codegen::FromSelf;
//     type Source = Self;
// }
// impl<T, __substrate_V> ::substrate::types::HasBundleKind for GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind<
//             BundleKind = <Input<T> as ::substrate::types::HasBundleKind>::BundleKind,
//         >,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::HasBundleKind,
// {
//     type BundleKind = GenericIoKind<T>;
//     fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
//         GenericIoKind {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::codegen::HasView<
//                         __substrate_V,
//                     >>::View as ::substrate::types::HasBundleKind>::kind(&self.signal),
//                 }
//     }
// }
// impl<T, __substrate_V> ::substrate::types::FlatLen for GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::FlatLen,
// {
//     fn len(&self) -> usize {
//         <<Input<
//                     T,
//                 > as ::substrate::types::codegen::HasView<
//                     __substrate_V,
//                 >>::View as ::substrate::types::FlatLen>::len(&self.signal)
//     }
// }
// impl<T, __substrate_V, __substrate_F> ::substrate::types::Flatten<__substrate_F>
//     for GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Flatten<__substrate_F>,
// {
//     fn flatten<E>(&self, __substrate_output_sink: &mut E)
//     where
//         E: ::std::iter::Extend<__substrate_F>,
//     {
//         <<Input<
//                     T,
//                 > as ::substrate::types::codegen::HasView<
//                     __substrate_V,
//                 >>::View as ::substrate::types::Flatten<
//                     __substrate_F,
//                 >>::flatten(&self.signal, __substrate_output_sink);
//     }
// }
// impl<T, __substrate_V, __substrate_S> ::substrate::types::Unflatten<GenericIoKind<T>, __substrate_S>
//     for GenericIoView<T, __substrate_V>
// where
//     Input<T>: ::substrate::types::HasBundleKind,
//     Input<T>: ::substrate::types::codegen::HasView<__substrate_V>,
//     <Input<T> as ::substrate::types::codegen::HasView<__substrate_V>>::View:
//         ::substrate::types::Unflatten<
//             <Input<T> as ::substrate::types::HasBundleKind>::BundleKind,
//             __substrate_S,
//         >,
// {
//     fn unflatten<__substrate_I>(
//         __substrate_data: &GenericIoKind<T>,
//         __substrate_source: &mut __substrate_I,
//     ) -> Option<Self>
//     where
//         __substrate_I: Iterator<Item = __substrate_S>,
//     {
//         ::std::option::Option::Some(GenericIoView {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::codegen::HasView<
//                         __substrate_V,
//                     >>::View as ::substrate::types::Unflatten<
//                         <Input<T> as ::substrate::types::HasBundleKind>::BundleKind,
//                         __substrate_S,
//                     >>::unflatten(&&__substrate_data.signal, __substrate_source)?,
//                 })
//     }
// }
// impl<T> ::substrate::types::schematic::SchematicBundleKind for GenericIoKind<T>
// where
//     Input<T>: ::substrate::types::codegen::HasSchematicBundleKindViews,
// {
//     type NodeBundle = GenericIoView<T, ::substrate::types::codegen::NodeBundle>;
//     type TerminalBundle = GenericIoView<T, ::substrate::types::codegen::TerminalBundle>;
//     fn terminal_view(
//         cell: ::substrate::schematic::CellId,
//         cell_io: &::substrate::types::schematic::NodeBundle<Self>,
//         instance: ::substrate::schematic::InstanceId,
//         instance_io: &::substrate::types::schematic::NodeBundle<Self>,
//     ) -> ::substrate::types::schematic::TerminalBundle<Self> {
//         GenericIoView {
//                     signal: <<Input<
//                         T,
//                     > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::SchematicBundleKind>::terminal_view(
//                         cell,
//                         &&cell_io.signal,
//                         instance,
//                         &&instance_io.signal,
//                     ),
//                 }
//     }
// }
// impl<T> ::substrate::schematic::HasNestedView
//     for GenericIoView<T, ::substrate::types::codegen::NodeBundle>
// where
//     Input<T>: ::substrate::types::codegen::HasSchematicBundleKindViews,
// {
//     type NestedView = GenericIoView<T, ::substrate::types::codegen::NestedNodeBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         GenericIoView {
//             signal: <<Input<T> as ::substrate::types::codegen::HasView<
//                 ::substrate::types::codegen::NodeBundle,
//             >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                 &&self.signal,
//                 __substrate_parent,
//             ),
//         }
//     }
// }
// impl<T> ::substrate::schematic::HasNestedView
//     for GenericIoView<T, ::substrate::types::codegen::TerminalBundle>
// where
//     Input<T>: ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//         View: Send + Sync,
//     >,
//     Input<T>: ::substrate::types::codegen::HasView<::substrate::types::codegen::TerminalBundle>,
//     <Input<T> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::TerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <Input<T> as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedTerminalBundle,
//         >>::View,
//     >,
// {
//     type NestedView = GenericIoView<T, ::substrate::types::codegen::NestedTerminalBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         GenericIoView {
//             signal: <<Input<T> as ::substrate::types::codegen::HasView<
//                 ::substrate::types::codegen::TerminalBundle,
//             >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                 &&self.signal,
//                 __substrate_parent,
//             ),
//         }
//     }
// }
// impl<T> ::substrate::schematic::HasNestedView
//     for GenericIoView<T, ::substrate::types::codegen::NestedNodeBundle>
// where
//     Input<T>: ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//         View: Send + Sync,
//     >,
//     <Input<T> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedNodeBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <Input<T> as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedNodeBundle,
//         >>::View,
//     >,
// {
//     type NestedView = GenericIoView<T, ::substrate::types::codegen::NestedNodeBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         GenericIoView {
//             signal: <<Input<T> as ::substrate::types::codegen::HasView<
//                 ::substrate::types::codegen::NestedNodeBundle,
//             >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                 &&self.signal,
//                 __substrate_parent,
//             ),
//         }
//     }
// }
// impl<T> ::substrate::schematic::HasNestedView
//     for GenericIoView<T, ::substrate::types::codegen::NestedTerminalBundle>
// where
//     Input<T>: ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//         View: Send + Sync,
//     >,
//     <Input<T> as ::substrate::types::codegen::HasView<
//         ::substrate::types::codegen::NestedTerminalBundle,
//     >>::View: ::substrate::schematic::HasNestedView<
//         NestedView = <Input<T> as ::substrate::types::codegen::HasView<
//             ::substrate::types::codegen::NestedTerminalBundle,
//         >>::View,
//     >,
// {
//     type NestedView = GenericIoView<T, ::substrate::types::codegen::NestedTerminalBundle>;
//     fn nested_view(
//         &self,
//         __substrate_parent: &::substrate::schematic::InstancePath,
//     ) -> ::substrate::schematic::NestedView<Self> {
//         GenericIoView {
//             signal: <<Input<T> as ::substrate::types::codegen::HasView<
//                 ::substrate::types::codegen::NestedTerminalBundle,
//             >>::View as ::substrate::schematic::HasNestedView>::nested_view(
//                 &&self.signal,
//                 __substrate_parent,
//             ),
//         }
//     }
// }
// impl<T> GenericIoView<T, ::substrate::types::codegen::NodeBundle>
// where
//     <Self as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::schematic::SchematicBundleKind<NodeBundle = Self>,
//     Input<T>: ::substrate::types::codegen::HasView<::substrate::types::codegen::NodeBundle>,
//     Self: ::substrate::types::HasBundleKind,
// {
//     /// Views this node bundle as a node bundle of a different kind.
//     pub fn view_as<
//         __substrate_T: ::substrate::types::HasBundleKind<
//             BundleKind: ::substrate::types::schematic::SchematicBundleKind,
//         >,
//     >(
//         &self,
//     ) -> ::substrate::types::schematic::NodeBundle<__substrate_T>
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
// impl<T> GenericIoView<T, ::substrate::types::codegen::TerminalBundle>
// where
//     <Self as ::substrate::types::HasBundleKind>::BundleKind:
//         ::substrate::types::schematic::SchematicBundleKind<TerminalBundle = Self>,
//     Input<T>: ::substrate::types::codegen::HasView<::substrate::types::codegen::TerminalBundle>,
//     Self: ::substrate::types::HasBundleKind,
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

// /// Takes an IO type.
// ///
// /// Used to validate that a given type implements `Io`.
// fn takes_io<T: Io>() -> usize {
//     std::mem::size_of::<T>()
// }
//
// #[crate::test]
// fn generic_io_implements_io() {
//     takes_io::<GenericIo<Signal>>();
//     takes_io::<GenericIo<NamedStructIo>>();
//     takes_io::<GenericIo<TupleIo>>();
// }
//
// #[crate::test]
// fn named_struct_io_implements_io() {
//     takes_io::<NamedStructIo>();
// }
//
// #[crate::test]
// fn tuple_io_implements_io() {
//     takes_io::<TupleIo>();
// }
