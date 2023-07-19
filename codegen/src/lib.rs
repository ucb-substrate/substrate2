//! Procedural macros for the Substrate analog circuit generator framework.
#![warn(missing_docs)]

mod block;
mod derive;
mod io;
mod pdk;
mod sim;

use darling::FromDeriveInput;
use derive::{derive_trait, DeriveInputReceiver, DeriveTrait};
use io::{io_impl, layout_io, schematic_io, IoInputReceiver};
use pdk::layers::{
    DerivedLayerFamilyInputReceiver, DerivedLayersInputReceiver, LayerFamilyInputReceiver,
    LayerInputReceiver, LayersInputReceiver,
};
use pdk::supported_pdks::supported_pdks_impl;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::proc_macro_error;
use quote::quote;
use sim::simulator_tuples_impl;
use syn::Ident;
use syn::{parse_macro_input, DeriveInput};

macro_rules! handle_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => {
                return err.write_errors().into();
            }
        }
    };
}

/// Enumerates PDKs supported by a certain layout implementation of a block.
///
/// Automatically implements the appropriate trait for all specified PDKs given a process-portable
/// implementation in a single PDK.
///
/// # Examples
///
/// ```
#[doc = include_str!("../build/docs/prelude.rs.hidden")]
#[doc = include_str!("../build/docs/pdk/several_layers.rs.hidden")]
#[doc = include_str!("../build/docs/pdk/several_pdks.rs.hidden")]
#[doc = include_str!("../build/docs/block/inverter.rs.hidden")]
#[doc = include_str!("../build/docs/block/buffer.rs.hidden")]
#[doc = include_str!("../build/docs/layout/inverter_multiprocess.rs")]
#[doc = include_str!("../build/docs/layout/buffer_multiprocess.rs")]
/// ```
#[proc_macro_attribute]
pub fn supported_pdks(args: TokenStream, input: TokenStream) -> TokenStream {
    supported_pdks_impl(args, input)
}

/// Derives a layer implementation on a tuple struct containing only an ID.
///
/// # Examples
///
/// ```
/// # use substrate::Layer;
/// # use substrate::pdk::layers::LayerId;
/// #[derive(Layer, Clone, Copy)]
/// #[layer(name = "poly", gds = "66/20")]
/// pub struct Poly(LayerId);
/// ```
#[proc_macro_derive(Layer, attributes(layer))]
pub fn derive_layer(input: TokenStream) -> TokenStream {
    let receiver = handle_error!(LayerInputReceiver::from_derive_input(&parse_macro_input!(
        input as DeriveInput
    )));
    quote!(
        #receiver
    )
    .into()
}

/// Derives a layer family implementation on a struct.
///
/// See the [`Layers` derive macro](`derive_layers`) for a full example.
#[proc_macro_derive(LayerFamily, attributes(layer))]
pub fn derive_layer_family(input: TokenStream) -> TokenStream {
    let receiver = handle_error!(LayerFamilyInputReceiver::from_derive_input(
        &parse_macro_input!(input as DeriveInput)
    ));
    quote!(
        #receiver
    )
    .into()
}

/// Derives a layer set implementation on a struct.
///
/// # Examples
///
/// ```
#[doc = include_str!("../build/docs/prelude.rs.hidden")]
#[doc = include_str!("../build/docs/pdk/layers.rs")]
/// ```
#[proc_macro_derive(Layers, attributes(layer, layer_family))]
pub fn derive_layers(input: TokenStream) -> TokenStream {
    let receiver = handle_error!(LayersInputReceiver::from_derive_input(&parse_macro_input!(
        input as DeriveInput
    )));
    quote!(
        #receiver
    )
    .into()
}

/// Derives a derived layer family implementation on a struct.
///
/// See the [`DerivedLayers` derive macro](`derive_derived_layers`) for a full example.
#[proc_macro_derive(DerivedLayerFamily, attributes(layer))]
pub fn derive_derived_layer_family(input: TokenStream) -> TokenStream {
    let receiver = DerivedLayerFamilyInputReceiver::from_derive_input(&parse_macro_input!(
        input as DeriveInput
    ));
    let receiver = handle_error!(receiver);
    quote!(
        #receiver
    )
    .into()
}

/// Derives a derived layer set implementation on a struct.
///
/// # Examples
///
/// ```
#[doc = include_str!("../build/docs/prelude.rs.hidden")]
#[doc = include_str!("../build/docs/pdk/several_layers.rs.hidden")]
#[doc = include_str!("../build/docs/pdk/several_pdks.rs.hidden")]
#[doc = include_str!("../build/docs/pdk/derived_layers.rs")]
/// ```
#[proc_macro_derive(DerivedLayers, attributes(layer_family))]
pub fn derive_derived_layers(input: TokenStream) -> TokenStream {
    let receiver = handle_error!(DerivedLayersInputReceiver::from_derive_input(
        &parse_macro_input!(input as DeriveInput)
    ));
    quote!(
        #receiver
    )
    .into()
}

/// Derives `Io` for a struct.
///
/// # Examples
///
/// By default, deriving `Io` for a struct creates general purpose schematic and layout IO structs by suffixing the
/// provided identifier with `Schematic` and `Layout`.
///
/// In the example below, `BufferIoSchematic` and `BufferIoLayout` are automatically created with default
/// settings. These are the structs that users interact with when generating schematics and layout
/// views, respectively.
///
/// ```
#[doc = include_str!("../build/docs/prelude.rs.hidden")]
#[doc = include_str!("../build/docs/block/buffer_io_simple.rs")]
/// ```
///
/// ```
#[doc = include_str!("../build/docs/prelude.rs.hidden")]
///
/// // Autogenerated by `#[derive(Io)]`.
/// pub struct BufferIoSchematic {
///     vdd: InOut<Node>,
///     vss: InOut<Node>,
///     din: Input<Node>,
///     dout: Output<Node>,
/// }
///
/// pub struct BufferIoLayout {
///     vdd: PortGeometry,
///     vss: PortGeometry,
///     din: PortGeometry,
///     dout: PortGeometry,
/// }
/// ```
///
/// However, the general purpose `PortGeometry` structs that represent the geometry of single net ports in
/// `BufferIoLayout` are often unecessary since they contain multiple shapes, whereas most
/// circuits often have a single shape for several of their ports.
///
/// Substrate allows you to customize the type of the ports you interact with when setting up IO in
/// the layout view of a block using the `#[substrate(layout_type = "...")]` attribute.
///
/// ```
#[doc = include_str!("../build/docs/prelude.rs.hidden")]
#[doc = include_str!("../build/docs/block/buffer_io.rs")]
/// ```
///
/// This indicates that the `din` and `dout` of the buffer only have a single shape, making the
/// ports easier to interact with when instantiating the buffer in other blocks.
///
/// If desired, you can even replace the whole IO struct with a layout type of your own (See
/// the [`LayoutType` derive macro](`derive_layout_type`)).
///
#[proc_macro_derive(Io, attributes(substrate))]
pub fn derive_io(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let input = handle_error!(IoInputReceiver::from_derive_input(&parsed));
    let schematic = schematic_io(&input);
    let layout = layout_io(&input);
    let io_impl = io_impl(&input);
    let ident = parsed.ident;
    let (imp, ty, wher) = parsed.generics.split_for_impl();
    let substrate = substrate_ident();
    quote!(
        impl #imp #substrate::io::Io for #ident #ty #wher {}
        #io_impl
        #schematic
        #layout
    )
    .into()
}

/// Derives `LayoutType` for a struct.
///
/// # Examples
///
/// You can create your own layout types and use them as your layout IO to customize the API for
/// accessing shapes within your port. This will work as long as the flattened lengths (i.e. the
/// number of nets) of the original IO and the custom IO are the same.
///
/// ```
#[doc = include_str!("../build/docs/prelude.rs.hidden")]
#[doc = include_str!("../build/docs/block/buffer_io_custom_layout.rs")]
/// ```
#[proc_macro_derive(LayoutType)]
pub fn derive_layout_type(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let input = handle_error!(IoInputReceiver::from_derive_input(&parsed));
    let layout = layout_io(&input);
    let io_impl = io_impl(&input);
    quote!(
        #io_impl
        #layout
    )
    .into()
}

/// Derives `substrate::layout::Data` for a struct.
///
/// The `#[transform]` attribute annotates data that should be transformed with the enclosing instance when
/// instantiated in another block.
///
/// # Examples
///
/// This example stores the individual buffer instances within a buffer chain. The `#[transform]`
/// notes that the buffers in the data should be transformed if the buffer chain is instantiated in another
/// block and transformed.
///
/// ```
#[doc = include_str!("../build/docs/prelude.rs.hidden")]
#[doc = include_str!("../build/docs/pdk/layers.rs.hidden")]
#[doc = include_str!("../build/docs/pdk/pdk.rs.hidden")]
#[doc = include_str!("../build/docs/block/inverter.rs.hidden")]
#[doc = include_str!("../build/docs/block/buffer.rs.hidden")]
#[doc = include_str!("../build/docs/layout/inverter.rs.hidden")]
#[doc = include_str!("../build/docs/layout/buffer.rs.hidden")]
#[doc = include_str!("../build/docs/layout/buffern_data.rs")]
/// ```
#[proc_macro_derive(LayoutData, attributes(substrate))]
pub fn derive_layout_data(input: TokenStream) -> TokenStream {
    let receiver = block::layout::DataInputReceiver::from_derive_input(&parse_macro_input!(
        input as DeriveInput
    ));
    let receiver = handle_error!(receiver);
    quote!(
        #receiver
    )
    .into()
}

/// Derives `substrate::schematic::Data` for a struct.
///
/// The `#[substrate(nested)]` attribute annotates data that represents nested instances or nodes. This allows
/// Substrate to keep track of paths to nested instances and nodes for simulation purposes.
#[proc_macro_derive(SchematicData, attributes(substrate))]
pub fn derive_schematic_data(input: TokenStream) -> TokenStream {
    let receiver = block::schematic::DataInputReceiver::from_derive_input(&parse_macro_input!(
        input as DeriveInput
    ));
    let receiver = handle_error!(receiver);
    quote!(
        #receiver
    )
    .into()
}

/// Derives `substrate::block::Block` for a struct or enum.
///
/// You must specify the block's IO by adding a `#[substrate(io = "IoType")]` attribute:
/// ```
/// use serde::{Serialize, Deserialize};
/// use substrate::Block;
///
/// #[derive(Block, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
/// #[substrate(io = "substrate::io::TestbenchIo")]
/// pub struct MyBlock {
///   // ...
/// }
/// ```
///
/// This derive macro only works if you want to use the default value of the IO.
/// If the IO type does not implement [`Default`], or you want to use a non-default
/// value, you must implement `Block` manually.
///
/// The ID value generated by this macro will have the form
/// `mycrate::mymodule::MyBlock`. The block name function will return
/// the name of the struct/enum converted to snake case. For example, the name
/// of a block called `MyBlock` will be `my_block`.
/// If you wish to customize this behavior, consider implementing `Block` manually.
#[proc_macro_derive(Block, attributes(substrate))]
pub fn derive_block(input: TokenStream) -> TokenStream {
    let receiver =
        block::BlockInputReceiver::from_derive_input(&parse_macro_input!(input as DeriveInput));
    let receiver = handle_error!(receiver);
    quote!(
        #receiver
    )
    .into()
}

/// Implements `substrate::simulation::Supports<Tuple> for Simulator`
/// for all tuples up to a specified max size.
#[proc_macro]
pub fn simulator_tuples(input: TokenStream) -> TokenStream {
    simulator_tuples_impl(input)
}

/// Derives `substrate::geometry::transform::TranslateMut`.
#[proc_macro_derive(TranslateMut)]
pub fn derive_translate_mut(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let receiver = handle_error!(DeriveInputReceiver::from_derive_input(&parsed));
    let substrate = substrate_ident();
    let config = DeriveTrait {
        trait_: quote!(#substrate::geometry::transform::TranslateMut),
        method: quote!(translate_mut),
        extra_arg_idents: vec![quote!(__substrate_derive_point)],
        extra_arg_tys: vec![quote!(#substrate::geometry::point::Point)],
    };

    let expanded = derive_trait(&config, receiver);
    proc_macro::TokenStream::from(expanded)
}

/// Derives `substrate::geometry::transform::TransformMut`.
#[proc_macro_derive(TransformMut)]
pub fn derive_transform_mut(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let receiver = handle_error!(DeriveInputReceiver::from_derive_input(&parsed));
    let substrate = substrate_ident();
    let config = DeriveTrait {
        trait_: quote!(#substrate::geometry::transform::TransformMut),
        method: quote!(transform_mut),
        extra_arg_idents: vec![quote!(__substrate_derive_transformation)],
        extra_arg_tys: vec![quote!(#substrate::geometry::transform::Transformation)],
    };

    let expanded = derive_trait(&config, receiver);
    proc_macro::TokenStream::from(expanded)
}

/// Derives `substrate::schematic::HasSchematicImpl` for any Substrate block.
///
/// This turns the block into a schematic hard macro.
/// You must add a `#[substrate(schematic(...))]` attribute to configure this macro;
/// see the examples below.
/// Using multiple `#[substrate(schematic(...))]` attributes allows you to
/// generate `HasSchematicImpl` implementations for multiple PDKs.
///
/// This macro only works on Substrate blocks,
/// so you must also add a `#[derive(Block)]` attribute
/// or implement `Block` manually.
///
/// # Arguments
///
/// This macro requires the following arguments (see [Supported formats](#supported-formats) for more details):
/// * `source`: The source from which to read the contents of this block's schematic.
/// * `name`: The name of the block's contents in `source`. For example, if
///   source is a SPICE netlist, name should be set to the name of the desired
///   subcircuit in that netlist.
/// * `fmt`: The netlist format.
/// * `pdk`: The PDK to which source corresponds.
///
/// # Supported formats
///
/// The following formats are supported:
///
/// * `spice`: Source should be an expression that evaluates to the file path of a SPICE netlist.
/// * `inline-spice`: Source should be an expression that evaluates to a String-like object
///   (`&str`, `String`, `ArcStr`, etc.) that contains a SPICE netlist.
///
/// Note that expressions can be arbitrary Rust expressions. Here are some examples:
/// * `fmt = "\"/path/to/netlist.spice\""` (note that you need the escaped quotes to make this a
/// string literal).
/// * `fmt = "function_that_returns_path()"`
/// * `fmt = "function_with_arguments_that_returns_path(\"my_argument\")"`
///
/// # Examples
///
/// ```
#[doc = include_str!("../build/docs/prelude.rs.hidden")]
#[doc = include_str!("../build/docs/block/buffer_io_simple.rs.hidden")]
#[doc = include_str!("../build/docs/block/buffer_hard_macro.rs")]
/// ```
#[proc_macro_error]
#[proc_macro_derive(HasSchematicImpl, attributes(substrate))]
pub fn derive_has_schematic_impl(input: TokenStream) -> TokenStream {
    let receiver = block::schematic::HasSchematicImplInputReceiver::from_derive_input(
        &parse_macro_input!(input as DeriveInput),
    );
    let receiver = handle_error!(receiver);
    quote!(
        #receiver
    )
    .into()
}

pub(crate) fn substrate_ident() -> TokenStream2 {
    match crate_name("substrate").expect("substrate is present in `Cargo.toml`") {
        FoundCrate::Itself => quote!(::substrate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}
