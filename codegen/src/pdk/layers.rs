use darling::{ast, FromDeriveInput, FromField, FromMeta};

// A speaking volume. Deriving `FromMeta` will cause this to be usable
/// as a string value for a meta-item key.
#[derive(Debug, Clone, Copy, FromMeta)]
#[darling(default)]
enum Volume {
    Normal,
    Whisper,
    Shout,
}

impl Default for Volume {
    fn default() -> Self {
        Volume::Normal
    }
}

// Support parsing from a full derive input. Unlike FromMeta, this isn't
/// composable; each darling-dependent crate should have its own struct to handle
/// when its trait is derived.
#[derive(Debug, FromDeriveInput)]
// This line says that we want to process all attributes declared with `my_trait`,
// and that darling should panic if this receiver is given an enum.
#[darling(attributes(my_trait), supports(struct_any))]
struct MyInputReceiver {
    /// The struct ident.
    ident: syn::Ident,

    /// The type's generics. You'll need these any time your trait is expected
    /// to work with types that declare generics.
    generics: syn::Generics,

    /// Receives the body of the struct or enum. We don't care about
    /// struct fields because we previously told darling we only accept structs.
    data: ast::Data<(), MyFieldReceiver>,

    /// The Input Receiver demands a volume, so use `Volume::Normal` if the
    /// caller doesn't provide one.
    #[darling(default)]
    volume: Volume,
}

#[derive(Debug, FromField)]
#[darling(attributes(my_trait))]
struct MyFieldReceiver {
    /// Get the ident of the field. For fields in tuple or newtype structs or
    /// enum bodies, this can be `None`.
    ident: Option<syn::Ident>,

    /// This magic field name pulls the type from the input.
    ty: syn::Type,

    /// We declare this as an `Option` so that during tokenization we can write
    /// `field.volume.unwrap_or(derive_input.volume)` to facilitate field-level
    /// overrides of struct-level settings.
    ///
    /// Because this field is an `Option`, we don't need to include `#[darling(default)]`
    volume: Option<Volume>,
}
