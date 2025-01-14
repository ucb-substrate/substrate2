#![allow(dead_code)]
use substrate::types::Io;
use substrate::types::{Input, Output, Signal};

use super::codegen::HasSaveViews;

/// An Io with a generic type parameter.
#[derive(Debug, Clone, Io)]
pub struct GenericIo<T> {
    /// A single input field.
    pub signal: Input<T>,
}

/// A named struct Io.
#[derive(Debug, Clone)]
pub struct NamedStructIo {
    /// An input.
    pub first: Input<Signal>,
    /// An output.
    pub second: Output<Signal>,
}
/// A named struct Io.
pub struct NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
{
    /// An input.
    pub first: <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
    /// An output.
    pub second: <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
}
impl ::substrate::types::FlatLen for NamedStructIo
where
    Input<Signal>: ::substrate::types::FlatLen,
    Output<Signal>: ::substrate::types::FlatLen,
{
    fn len(&self) -> usize {
        <Input<Signal> as ::substrate::types::FlatLen>::len(&self.first)
            + <Output<Signal> as ::substrate::types::FlatLen>::len(&self.second)
    }
}
impl ::substrate::types::Flatten<::substrate::types::Direction> for NamedStructIo
where
    Input<Signal>: ::substrate::types::Flatten<::substrate::types::Direction>,
    Output<Signal>: ::substrate::types::Flatten<::substrate::types::Direction>,
{
    fn flatten<E>(&self, __substrate_output_sink: &mut E)
    where
        E: ::std::iter::Extend<::substrate::types::Direction>,
    {
        <Input<Signal> as ::substrate::types::Flatten<::substrate::types::Direction>>::flatten(
            &self.first,
            __substrate_output_sink,
        );
        <Output<Signal> as ::substrate::types::Flatten<::substrate::types::Direction>>::flatten(
            &self.second,
            __substrate_output_sink,
        );
    }
}
impl ::substrate::types::HasBundleKind for NamedStructIo
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
{
    type BundleKind = NamedStructIoKind;
    fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
        NamedStructIoKind {
            first: <Input<Signal> as ::substrate::types::HasBundleKind>::kind(&self.first),
            second: <Output<Signal> as ::substrate::types::HasBundleKind>::kind(&self.second),
        }
    }
}
impl ::std::clone::Clone for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::clone::Clone,
    <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self {
                    first: <<Input<
                        Signal,
                    > as ::substrate::types::HasBundleKind>::BundleKind as ::std::clone::Clone>::clone(
                        &self.first,
                    ),
                    second: <<Output<
                        Signal,
                    > as ::substrate::types::HasBundleKind>::BundleKind as ::std::clone::Clone>::clone(
                        &self.second,
                    ),
                }
    }
}
impl ::std::fmt::Debug for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
    <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
{
    fn fmt(&self, __substrate_f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        __substrate_f
            .debug_struct("NamedStructIoKind")
            .field("first", &self.first)
            .field("second", &self.second)
            .finish()
    }
}
impl ::std::cmp::PartialEq for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
    <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::fmt::Debug,
{
    fn eq(&self, __substrate_other: &Self) -> bool {
        <<Input<
                    Signal,
                > as ::substrate::types::HasBundleKind>::BundleKind as ::std::cmp::PartialEq>::eq(
                    &self.first,
                    &__substrate_other.first,
                )
                    && <<Output<
                        Signal,
                    > as ::substrate::types::HasBundleKind>::BundleKind as ::std::cmp::PartialEq>::eq(
                        &self.second,
                        &__substrate_other.second,
                    )
    }
}
impl ::std::cmp::Eq for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::cmp::Eq,
    <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::std::cmp::Eq,
{
}
impl ::substrate::types::FlatLen for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::FlatLen,
    <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind: ::substrate::types::FlatLen,
{
    fn len(&self) -> usize {
        <<Input<
                    Signal,
                > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::FlatLen>::len(
                    &self.first,
                )
                    + <<Output<
                        Signal,
                    > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::FlatLen>::len(
                        &self.second,
                    )
    }
}
impl ::substrate::types::HasBundleKind for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
        ::substrate::types::HasBundleKind,
    <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
        ::substrate::types::HasBundleKind,
{
    type BundleKind = NamedStructIoKind;
    fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
        NamedStructIoKind {
                    first: <<Input<
                        Signal,
                    > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasBundleKind>::kind(
                        &self.first,
                    ),
                    second: <<Output<
                        Signal,
                    > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasBundleKind>::kind(
                        &self.second,
                    ),
                }
    }
}
impl ::substrate::types::HasNameTree for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
        ::substrate::types::HasBundleKind,
    <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
        ::substrate::types::HasBundleKind,
{
    fn names(&self) -> ::std::option::Option<::std::vec::Vec<::substrate::types::NameTree>> {
        let v: ::std::vec::Vec<::substrate::types::NameTree> = <[_]>::into_vec(
                        #[rustc_box]
                        ::alloc::boxed::Box::new([
                            (
                                {
                                    const __TEXT: &::arcstr::_private::str = "first";
                                    {
                                        #[allow(clippy::declare_interior_mutable_const)]
                                        const SI: &::arcstr::_private::StaticArcStrInner<
                                            [::arcstr::_private::u8; __TEXT.len()],
                                        > = unsafe {
                                            &::arcstr::_private::StaticArcStrInner {
                                                len_flag: match ::arcstr::_private::StaticArcStrInner::<
                                                    [::arcstr::_private::u8; __TEXT.len()],
                                                >::encode_len(__TEXT.len()) {
                                                    Some(len) => len,
                                                    None => {
                                                        ::core::panicking::panic_fmt(
                                                            format_args!("impossibly long length"),
                                                        );
                                                    }
                                                },
                                                count_flag: ::arcstr::_private::StaticArcStrInner::<
                                                    [::arcstr::_private::u8; __TEXT.len()],
                                                >::STATIC_COUNT_VALUE,
                                                data: *::arcstr::_private::ConstPtrDeref::<
                                                    [::arcstr::_private::u8; __TEXT.len()],
                                                > {
                                                    p: __TEXT.as_ptr(),
                                                }
                                                    .a,
                                            }
                                        };
                                        #[allow(clippy::declare_interior_mutable_const)]
                                        const S: ::arcstr::ArcStr = unsafe {
                                            ::arcstr::ArcStr::_private_new_from_static_data(SI)
                                        };
                                        S
                                    }
                                },
                                <<Input<
                                    Signal,
                                > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasNameTree>::names(
                                    &&self.first,
                                ),
                            ),
                            (
                                {
                                    const __TEXT: &::arcstr::_private::str = "second";
                                    {
                                        #[allow(clippy::declare_interior_mutable_const)]
                                        const SI: &::arcstr::_private::StaticArcStrInner<
                                            [::arcstr::_private::u8; __TEXT.len()],
                                        > = unsafe {
                                            &::arcstr::_private::StaticArcStrInner {
                                                len_flag: match ::arcstr::_private::StaticArcStrInner::<
                                                    [::arcstr::_private::u8; __TEXT.len()],
                                                >::encode_len(__TEXT.len()) {
                                                    Some(len) => len,
                                                    None => {
                                                        ::core::panicking::panic_fmt(
                                                            format_args!("impossibly long length"),
                                                        );
                                                    }
                                                },
                                                count_flag: ::arcstr::_private::StaticArcStrInner::<
                                                    [::arcstr::_private::u8; __TEXT.len()],
                                                >::STATIC_COUNT_VALUE,
                                                data: *::arcstr::_private::ConstPtrDeref::<
                                                    [::arcstr::_private::u8; __TEXT.len()],
                                                > {
                                                    p: __TEXT.as_ptr(),
                                                }
                                                    .a,
                                            }
                                        };
                                        #[allow(clippy::declare_interior_mutable_const)]
                                        const S: ::arcstr::ArcStr = unsafe {
                                            ::arcstr::ArcStr::_private_new_from_static_data(SI)
                                        };
                                        S
                                    }
                                },
                                <<Output<
                                    Signal,
                                > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::HasNameTree>::names(
                                    &&self.second,
                                ),
                            ),
                        ]),
                    )
                    .into_iter()
                    .filter_map(|(frag, children)| {
                        children.map(|c| ::substrate::types::NameTree::new(frag, c))
                    })
                    .collect();
        if v.len() == 0 {
            None
        } else {
            Some(v)
        }
    }
}
/// A named struct Io.
pub struct NamedStructIoView<SubstrateV>
where
    Input<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    Output<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
{
    /// An input.
    pub first: <Input<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View,
    /// An output.
    pub second: <Output<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View,
}
impl<SubstrateV> ::substrate::types::HasBundleKind for NamedStructIoView<SubstrateV>
where
    Input<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    Output<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    <Input<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::HasBundleKind<
            BundleKind = <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
        >,
    <Output<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::HasBundleKind<
            BundleKind = <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
        >,
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::HasBundleKind,
    <Output<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::HasBundleKind,
{
    type BundleKind = NamedStructIoKind;
    fn kind(&self) -> <Self as ::substrate::types::HasBundleKind>::BundleKind {
        NamedStructIoKind {
                    first: <<Input<
                        Signal,
                    > as ::substrate::types::codegen::HasView<
                        SubstrateV,
                    >>::View as ::substrate::types::HasBundleKind>::kind(&self.first),
                    second: <<Output<
                        Signal,
                    > as ::substrate::types::codegen::HasView<
                        SubstrateV,
                    >>::View as ::substrate::types::HasBundleKind>::kind(&self.second),
                }
    }
}
impl<SubstrateV> ::substrate::types::FlatLen for NamedStructIoView<SubstrateV>
where
    Input<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    Output<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    <Input<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::FlatLen,
    <Output<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::FlatLen,
{
    fn len(&self) -> usize {
        <<Input<
                    Signal,
                > as ::substrate::types::codegen::HasView<
                    SubstrateV,
                >>::View as ::substrate::types::FlatLen>::len(&self.first)
                    + <<Output<
                        Signal,
                    > as ::substrate::types::codegen::HasView<
                        SubstrateV,
                    >>::View as ::substrate::types::FlatLen>::len(&self.second)
    }
}
impl<SubstrateV, SubstrateF> ::substrate::types::Flatten<SubstrateF>
    for NamedStructIoView<SubstrateV>
where
    Input<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    Output<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    <Input<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::Flatten<SubstrateF>,
    <Output<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::Flatten<SubstrateF>,
{
    fn flatten<E>(&self, __substrate_output_sink: &mut E)
    where
        E: ::std::iter::Extend<SubstrateF>,
    {
        <<Input<
                    Signal,
                > as ::substrate::types::codegen::HasView<
                    SubstrateV,
                >>::View as ::substrate::types::Flatten<
                    SubstrateF,
                >>::flatten(&self.first, __substrate_output_sink);
        <<Output<
                    Signal,
                > as ::substrate::types::codegen::HasView<
                    SubstrateV,
                >>::View as ::substrate::types::Flatten<
                    SubstrateF,
                >>::flatten(&self.second, __substrate_output_sink);
    }
}
impl<SubstrateV, SubstrateS> ::substrate::types::Unflatten<NamedStructIoKind, SubstrateS>
    for NamedStructIoView<SubstrateV>
where
    Input<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    Output<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::Unflatten<
            <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
            SubstrateS,
        >,
    <Output<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::types::Unflatten<
            <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
            SubstrateS,
        >,
{
    fn unflatten<SubstrateI>(
        __substrate_data: &NamedStructIoKind,
        __substrate_source: &mut SubstrateI,
    ) -> Option<Self>
    where
        SubstrateI: Iterator<Item = SubstrateS>,
    {
        ::std::option::Option::Some(NamedStructIoView {
                    first: <<Input<
                        Signal,
                    > as ::substrate::types::codegen::HasView<
                        SubstrateV,
                    >>::View as ::substrate::types::Unflatten<
                        <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind,
                        SubstrateS,
                    >>::unflatten(&&__substrate_data.first, __substrate_source)?,
                    second: <<Output<
                        Signal,
                    > as ::substrate::types::codegen::HasView<
                        SubstrateV,
                    >>::View as ::substrate::types::Unflatten<
                        <Output<
                            Signal,
                        > as ::substrate::types::HasBundleKind>::BundleKind,
                        SubstrateS,
                    >>::unflatten(&&__substrate_data.second, __substrate_source)?,
                })
    }
}
impl ::substrate::types::schematic::HasNodeBundle for NamedStructIo
where
    Input<Signal>: ::substrate::types::schematic::HasNodeBundle,
    Output<Signal>: ::substrate::types::schematic::HasNodeBundle,
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
{
    type NodeBundle = NamedStructIoView<::substrate::types::codegen::NodeBundle>;
}
impl ::substrate::types::schematic::HasTerminalBundle for NamedStructIo
where
    Input<Signal>: ::substrate::types::schematic::HasTerminalBundle,
    Output<Signal>: ::substrate::types::schematic::HasTerminalBundle,
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
{
    type TerminalBundle = NamedStructIoView<::substrate::types::codegen::TerminalBundle>;
}
impl ::substrate::types::schematic::HasNodeBundle for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
        ::substrate::types::schematic::HasNodeBundle,
    <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
        ::substrate::types::schematic::HasNodeBundle,
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
{
    type NodeBundle = NamedStructIoView<::substrate::types::codegen::NodeBundle>;
}
impl ::substrate::types::schematic::HasTerminalBundle for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    <Input<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
        ::substrate::types::schematic::HasTerminalBundle,
    <Output<Signal> as ::substrate::types::HasBundleKind>::BundleKind:
        ::substrate::types::schematic::HasTerminalBundle,
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
{
    type TerminalBundle = NamedStructIoView<::substrate::types::codegen::TerminalBundle>;
}
impl ::substrate::types::schematic::SchematicBundleKind for NamedStructIoKind
where
    Input<Signal>: ::substrate::types::HasBundleKind,
    Output<Signal>: ::substrate::types::HasBundleKind,
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
{
    fn terminal_view(
        cell: ::substrate::schematic::CellId,
        cell_io: &::substrate::types::schematic::NodeBundle<Self>,
        instance: ::substrate::schematic::InstanceId,
        instance_io: &::substrate::types::schematic::NodeBundle<Self>,
    ) -> ::substrate::types::schematic::TerminalBundle<Self> {
        NamedStructIoView {
                    first: <<Input<
                        Signal,
                    > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::SchematicBundleKind>::terminal_view(
                        cell,
                        &cell_io.first,
                        instance,
                        &instance_io.first,
                    ),
                    second: <<Output<
                        Signal,
                    > as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::SchematicBundleKind>::terminal_view(
                        cell,
                        &cell_io.second,
                        instance,
                        &instance_io.second,
                    ),
                }
    }
}
impl ::substrate::schematic::HasNestedView
    for NamedStructIoView<::substrate::types::codegen::NodeBundle>
where
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    <Input<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NodeBundle,
    >>::View: ::substrate::schematic::HasNestedView,
    <Output<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NodeBundle,
    >>::View: ::substrate::schematic::HasNestedView,
{
    type NestedView = NamedStructIoView<::substrate::types::codegen::NestedNodeBundle>;
    fn nested_view(
        &self,
        __substrate_parent: &::substrate::schematic::InstancePath,
    ) -> ::substrate::schematic::NestedView<Self> {
        NamedStructIoView {
            first: <<Input<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NodeBundle,
            >>::View as ::substrate::schematic::HasNestedView>::nested_view(
                &&self.first,
                __substrate_parent,
            ),
            second: <<Output<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NodeBundle,
            >>::View as ::substrate::schematic::HasNestedView>::nested_view(
                &&self.second,
                __substrate_parent,
            ),
        }
    }
}
impl ::substrate::schematic::HasNestedView
    for NamedStructIoView<::substrate::types::codegen::TerminalBundle>
where
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    <Input<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::TerminalBundle,
    >>::View: ::substrate::schematic::HasNestedView,
    <Output<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::TerminalBundle,
    >>::View: ::substrate::schematic::HasNestedView,
{
    type NestedView = NamedStructIoView<::substrate::types::codegen::NestedTerminalBundle>;
    fn nested_view(
        &self,
        __substrate_parent: &::substrate::schematic::InstancePath,
    ) -> ::substrate::schematic::NestedView<Self> {
        NamedStructIoView {
            first: <<Input<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::TerminalBundle,
            >>::View as ::substrate::schematic::HasNestedView>::nested_view(
                &&self.first,
                __substrate_parent,
            ),
            second: <<Output<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::TerminalBundle,
            >>::View as ::substrate::schematic::HasNestedView>::nested_view(
                &&self.second,
                __substrate_parent,
            ),
        }
    }
}
impl ::substrate::schematic::HasNestedView
    for NamedStructIoView<::substrate::types::codegen::NestedNodeBundle>
where
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    <Input<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NestedNodeBundle,
    >>::View: ::substrate::schematic::HasNestedView<
        NestedView = <Input<Signal> as ::substrate::types::codegen::HasView<
            ::substrate::types::codegen::NestedNodeBundle,
        >>::View,
    >,
    <Output<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NestedNodeBundle,
    >>::View: ::substrate::schematic::HasNestedView<
        NestedView = <Output<Signal> as ::substrate::types::codegen::HasView<
            ::substrate::types::codegen::NestedNodeBundle,
        >>::View,
    >,
    <Input<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NestedNodeBundle,
    >>::View: ::substrate::schematic::HasNestedView,
    <Output<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NestedNodeBundle,
    >>::View: ::substrate::schematic::HasNestedView,
{
    type NestedView = NamedStructIoView<::substrate::types::codegen::NestedNodeBundle>;
    fn nested_view(
        &self,
        __substrate_parent: &::substrate::schematic::InstancePath,
    ) -> ::substrate::schematic::NestedView<Self> {
        NamedStructIoView {
            first: <<Input<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedNodeBundle,
            >>::View as ::substrate::schematic::HasNestedView>::nested_view(
                &&self.first,
                __substrate_parent,
            ),
            second: <<Output<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedNodeBundle,
            >>::View as ::substrate::schematic::HasNestedView>::nested_view(
                &&self.second,
                __substrate_parent,
            ),
        }
    }
}
impl ::substrate::schematic::HasNestedView
    for NamedStructIoView<::substrate::types::codegen::NestedTerminalBundle>
where
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    <Input<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NestedTerminalBundle,
    >>::View: ::substrate::schematic::HasNestedView<
        NestedView = <Input<Signal> as ::substrate::types::codegen::HasView<
            ::substrate::types::codegen::NestedTerminalBundle,
        >>::View,
    >,
    <Output<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NestedTerminalBundle,
    >>::View: ::substrate::schematic::HasNestedView<
        NestedView = <Output<Signal> as ::substrate::types::codegen::HasView<
            ::substrate::types::codegen::NestedTerminalBundle,
        >>::View,
    >,
    <Input<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NestedTerminalBundle,
    >>::View: ::substrate::schematic::HasNestedView,
    <Output<Signal> as ::substrate::types::codegen::HasView<
        ::substrate::types::codegen::NestedTerminalBundle,
    >>::View: ::substrate::schematic::HasNestedView,
{
    type NestedView = NamedStructIoView<::substrate::types::codegen::NestedTerminalBundle>;
    fn nested_view(
        &self,
        __substrate_parent: &::substrate::schematic::InstancePath,
    ) -> ::substrate::schematic::NestedView<Self> {
        NamedStructIoView {
            first: <<Input<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedTerminalBundle,
            >>::View as ::substrate::schematic::HasNestedView>::nested_view(
                &&self.first,
                __substrate_parent,
            ),
            second: <<Output<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedTerminalBundle,
            >>::View as ::substrate::schematic::HasNestedView>::nested_view(
                &&self.second,
                __substrate_parent,
            ),
        }
    }
}
impl NamedStructIoView<::substrate::types::codegen::NodeBundle>
where
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Self: ::substrate::types::HasBundleKind<
        BundleKind: ::substrate::types::schematic::SchematicBundleKind<NodeBundle = Self>,
    >,
{
    /// Views this node bundle as a node bundle of a different kind.
    pub fn view_as<
        SubstrateT: ::substrate::types::HasBundleKind<
            BundleKind: ::substrate::types::schematic::SchematicBundleKind,
        >,
    >(
        &self,
    ) -> ::substrate::types::schematic::NodeBundle<
        <SubstrateT as ::substrate::types::HasBundleKind>::BundleKind,
    >
    where
        <Self as ::substrate::types::HasBundleKind>::BundleKind:
            ::substrate::types::schematic::DataView<
                <SubstrateT as ::substrate::types::HasBundleKind>::BundleKind,
            >,
    {
        <<Self as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::DataView<
                    <SubstrateT as ::substrate::types::HasBundleKind>::BundleKind,
                >>::view_nodes_as(self)
    }
}
impl NamedStructIoView<::substrate::types::codegen::TerminalBundle>
where
    Input<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Output<Signal>: ::substrate::types::codegen::HasSchematicBundleKindViews,
    Self: ::substrate::types::HasBundleKind<
        BundleKind: ::substrate::types::schematic::SchematicBundleKind<TerminalBundle = Self>,
    >,
{
    /// Views this node bundle as a node bundle of a different kind.
    pub fn view_as<
        SubstrateT: ::substrate::types::HasBundleKind<
            BundleKind: ::substrate::types::schematic::SchematicBundleKind,
        >,
    >(
        &self,
    ) -> ::substrate::types::schematic::TerminalBundle<
        <SubstrateT as ::substrate::types::HasBundleKind>::BundleKind,
    >
    where
        <Self as ::substrate::types::HasBundleKind>::BundleKind:
            ::substrate::types::schematic::DataView<
                <SubstrateT as ::substrate::types::HasBundleKind>::BundleKind,
            >,
    {
        <<Self as ::substrate::types::HasBundleKind>::BundleKind as ::substrate::types::schematic::DataView<
                    <SubstrateT as ::substrate::types::HasBundleKind>::BundleKind,
                >>::view_terminals_as(self)
    }
}
impl<SubstrateS, SubstrateA> ::substrate::simulation::data::Save<SubstrateS, SubstrateA>
    for NamedStructIoView<::substrate::types::codegen::NestedNodeBundle>
where
    Input<Signal>: ::substrate::types::codegen::HasSaveViews<SubstrateS, SubstrateA>,
    Output<Signal>: ::substrate::types::codegen::HasSaveViews<SubstrateS, SubstrateA>,
    SubstrateS: ::substrate::simulation::Simulator,
    SubstrateA: ::substrate::simulation::Analysis,
{
    type SaveKey = NamedStructIoView<
        ::substrate::types::codegen::NestedNodeSaveKeyView<SubstrateS, SubstrateA>,
    >;
    type Saved =
        NamedStructIoView<::substrate::types::codegen::NestedNodeSavedView<SubstrateS, SubstrateA>>;
    fn save(
        &self,
        __substrate_ctx: &::substrate::simulation::SimulationContext<SubstrateS>,
        __substrate_opts: &mut <SubstrateS as ::substrate::simulation::Simulator>::Options,
    ) -> <Self as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::SaveKey {
        NamedStructIoView {
            first: <<Input<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedNodeBundle,
            >>::View as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::save(
                &&self.first,
                __substrate_ctx,
                __substrate_opts,
            ),
            second: <<Output<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedNodeBundle,
            >>::View as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::save(
                &&self.second,
                __substrate_ctx,
                __substrate_opts,
            ),
        }
    }
    fn from_saved(
        __substrate_output: &<SubstrateA as ::substrate::simulation::Analysis>::Output,
        __substrate_key: &<Self as ::substrate::simulation::data::Save<
                    SubstrateS,
                    SubstrateA,
                >>::SaveKey,
    ) -> <Self as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::Saved {
        NamedStructIoView {
            first: <<Input<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedNodeBundle,
            >>::View as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::from_saved(
                __substrate_output,
                &__substrate_key.first,
            ),
            second: <<Output<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedNodeBundle,
            >>::View as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::from_saved(
                __substrate_output,
                &__substrate_key.second,
            ),
        }
    }
}
impl<SubstrateS, SubstrateA> ::substrate::simulation::data::Save<SubstrateS, SubstrateA>
    for NamedStructIoView<::substrate::types::codegen::NestedTerminalBundle>
where
    Input<Signal>:
        ::substrate::types::codegen::HasView<::substrate::types::codegen::NestedTerminalBundle>,
    Output<Signal>:
        ::substrate::types::codegen::HasView<::substrate::types::codegen::NestedTerminalBundle>,
    Input<Signal>: ::substrate::types::codegen::HasSaveViews<SubstrateS, SubstrateA>,
    Output<Signal>: ::substrate::types::codegen::HasSaveViews<SubstrateS, SubstrateA>,
    SubstrateS: ::substrate::simulation::Simulator,
    SubstrateA: ::substrate::simulation::Analysis,
{
    type SaveKey = NamedStructIoView<
        ::substrate::types::codegen::NestedTerminalSaveKeyView<SubstrateS, SubstrateA>,
    >;
    type Saved = NamedStructIoView<
        ::substrate::types::codegen::NestedTerminalSavedView<SubstrateS, SubstrateA>,
    >;
    fn save(
        &self,
        __substrate_ctx: &::substrate::simulation::SimulationContext<SubstrateS>,
        __substrate_opts: &mut <SubstrateS as ::substrate::simulation::Simulator>::Options,
    ) -> <Self as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::SaveKey {
        NamedStructIoView {
            first: <<Input<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedTerminalBundle,
            >>::View as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::save(
                &&self.first,
                __substrate_ctx,
                __substrate_opts,
            ),
            second: <<Output<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedTerminalBundle,
            >>::View as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::save(
                &&self.second,
                __substrate_ctx,
                __substrate_opts,
            ),
        }
    }
    fn from_saved(
        __substrate_output: &<SubstrateA as ::substrate::simulation::Analysis>::Output,
        __substrate_key: &<Self as ::substrate::simulation::data::Save<
                    SubstrateS,
                    SubstrateA,
                >>::SaveKey,
    ) -> <Self as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::Saved {
        NamedStructIoView {
            first: <<Input<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedTerminalBundle,
            >>::View as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::from_saved(
                __substrate_output,
                &__substrate_key.first,
            ),
            second: <<Output<Signal> as ::substrate::types::codegen::HasView<
                ::substrate::types::codegen::NestedTerminalBundle,
            >>::View as ::substrate::simulation::data::Save<SubstrateS, SubstrateA>>::from_saved(
                __substrate_output,
                &__substrate_key.second,
            ),
        }
    }
}
impl<SubstrateV> ::substrate::geometry::transform::TranslateRef for NamedStructIoView<SubstrateV>
where
    Input<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    Output<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    <Input<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::geometry::transform::TranslateRef,
    <Output<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::geometry::transform::TranslateRef,
{
    fn translate_ref(&self, __substrate_point: ::substrate::geometry::point::Point) -> Self {
        NamedStructIoView {
                    first: <<Input<
                        Signal,
                    > as ::substrate::types::codegen::HasView<
                        SubstrateV,
                    >>::View as ::substrate::geometry::transform::TranslateRef>::translate_ref(
                        &&self.first,
                        __substrate_point,
                    ),
                    second: <<Output<
                        Signal,
                    > as ::substrate::types::codegen::HasView<
                        SubstrateV,
                    >>::View as ::substrate::geometry::transform::TranslateRef>::translate_ref(
                        &&self.second,
                        __substrate_point,
                    ),
                }
    }
}
impl<SubstrateV> ::substrate::geometry::transform::TransformRef for NamedStructIoView<SubstrateV>
where
    Input<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    Output<Signal>: ::substrate::types::codegen::HasView<SubstrateV>,
    <Input<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::geometry::transform::TransformRef,
    <Output<Signal> as ::substrate::types::codegen::HasView<SubstrateV>>::View:
        ::substrate::geometry::transform::TransformRef,
{
    fn transform_ref(
        &self,
        __substrate_transformation: ::substrate::geometry::transform::Transformation,
    ) -> Self {
        NamedStructIoView {
                    first: <<Input<
                        Signal,
                    > as ::substrate::types::codegen::HasView<
                        SubstrateV,
                    >>::View as ::substrate::geometry::transform::TransformRef>::transform_ref(
                        &&self.first,
                        __substrate_transformation,
                    ),
                    second: <<Output<
                        Signal,
                    > as ::substrate::types::codegen::HasView<
                        SubstrateV,
                    >>::View as ::substrate::geometry::transform::TransformRef>::transform_ref(
                        &&self.second,
                        __substrate_transformation,
                    ),
                }
    }
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
