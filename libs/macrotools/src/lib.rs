//! Utilities for writing proc macros quickly.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use darling::ast::Style;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{
    parse_quote, Attribute, Data, DeriveInput, Expr, Field, Fields, GenericParam, Generics, Ident,
    Index, Token, Type, Variant, Visibility, WherePredicate,
};

#[macro_export]
macro_rules! handle_syn_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => {
                return err.to_compile_error().into();
            }
        }
    };
}

#[macro_export]
macro_rules! handle_darling_error {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(err) => {
                return err.write_errors().into();
            }
        }
    };
}

/// Add a bound `T: trait_` to every type parameter T.
pub fn add_trait_bounds(generics: &mut Generics, trait_: TokenStream) {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(syn::parse_quote!(#trait_));
        }
    }
}

/// Generates a [`struct@syn::Ident`] for a destructuring an element of a tuple.
pub fn tuple_ident(idx: usize) -> syn::Ident {
    format_ident!("__type_dispatch_derive_field{idx}")
}

pub fn pretty_ident(idx: usize, ident: &Option<Ident>) -> Ident {
    if let Some(ident) = ident {
        ident.clone()
    } else {
        format_ident!("elem{idx}")
    }
}

/// Tokens used for generating struct fields in derived implementations.
pub struct FieldTokens {
    /// For named structs: "pub field:"
    /// For tuple structs: "pub"
    pub declare: TokenStream,
    /// For named structs: "self.field"
    /// For tuple structs: "self.2"
    pub refer: TokenStream,
    /// For named structs: "field:"
    /// For tuple structs: ""
    pub assign: TokenStream,
    /// For named structs: "field"
    /// For tuple structs: "__substrate_derive_field2"
    pub temp: TokenStream,
    /// For named structs: "field"
    /// For tuple structs: "elem2"
    pub pretty_ident: TokenStream,
}

/// Returns a [`FieldTokens`] object for a struct that can be referenced using
/// the tokens in `referent`.
pub fn field_tokens_with_referent(
    style: Style,
    vis: &Visibility,
    attrs: &Vec<syn::Attribute>,
    idx: usize,
    ident: &Option<syn::Ident>,
    referent: TokenStream,
) -> FieldTokens {
    let tuple_ident = tuple_ident(idx);
    let pretty_tuple_ident = format_ident!("elem{idx}");
    let idx = syn::Index::from(idx);

    let (declare, refer, assign, temp, pretty_ident) = match style {
        Style::Unit => (quote!(), quote!(), quote!(), quote!(), quote!()),
        Style::Struct => (
            quote!(#(#attrs)* #vis #ident:),
            quote!(#referent.#ident),
            quote!(#ident:),
            quote!(#ident),
            quote!(#ident),
        ),
        Style::Tuple => (
            quote!(#(#attrs)* #vis),
            quote!(#referent.#idx),
            quote!(),
            quote!(#tuple_ident),
            quote!(#pretty_tuple_ident),
        ),
    };

    FieldTokens {
        declare,
        refer,
        assign,
        temp,
        pretty_ident,
    }
}

/// Returns a [`FieldTokens`] object for a struct that can be referenced with `self`.
pub fn field_tokens(
    style: Style,
    vis: &Visibility,
    attrs: &Vec<syn::Attribute>,
    idx: usize,
    ident: &Option<syn::Ident>,
) -> FieldTokens {
    field_tokens_with_referent(style, vis, attrs, idx, ident, syn::parse_quote!(self))
}

pub fn field_decl(field: &Field) -> TokenStream {
    let Field {
        ref ident,
        ref vis,
        ref ty,
        ref attrs,
        ..
    } = field;

    match ident {
        Some(ident) => {
            quote! {
                #(#attrs)*
                #vis #ident: #ty,
            }
        }
        None => {
            quote! {
                #ty,
            }
        }
    }
}

pub fn field_referent(prefix: Option<&TokenStream>, idx: usize, field: &Field) -> TokenStream {
    let ident = &field.ident;
    let tuple_ident = tuple_ident(idx);
    let idx = syn::Index::from(idx);
    match (prefix, ident) {
        (Some(prefix), Some(ident)) => quote!(&#prefix.#ident),
        (Some(prefix), None) => quote!(&#prefix.#idx),
        (None, Some(ident)) => quote!(#ident),
        (None, None) => quote!(#tuple_ident),
    }
}

pub fn field_assign(
    prefix: Option<&TokenStream>,
    idx: usize,
    field: &Field,
    prev_tys: Vec<Type>,
    val: impl FnOnce(&MapField) -> TokenStream,
) -> TokenStream {
    let Field {
        ref ident, ref ty, ..
    } = field;

    let pretty = pretty_ident(idx, ident);
    let refer = field_referent(prefix, idx, field);

    let value = val(&MapField {
        ty: ty.clone(),
        refer,
        pretty_ident: pretty,
        prev_tys,
    });

    match ident {
        Some(ident) => quote! { #ident: #value, },
        None => quote! { #value, },
    }
}

pub fn variant_decl(variant: &Variant) -> TokenStream {
    let Variant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let decls = fields.iter().map(|f| field_decl(f));
    match fields {
        Fields::Unit => quote!(#ident,),
        Fields::Unnamed(_) => quote!(#ident( #(#decls)* ),),
        Fields::Named(_) => quote!(#ident { #(#decls)* },),
    }
}

pub fn variant_map_arm(input_type: &Type, variant: &Variant, body: &TokenStream) -> TokenStream {
    let Variant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let destructure = fields
        .iter()
        .enumerate()
        .map(|(i, f)| f.ident.clone().unwrap_or_else(|| tuple_ident(i)));
    match fields {
        Fields::Unit => quote!(#input_type::#ident => #body,),
        Fields::Unnamed(_) => {
            quote!(#input_type::#ident( #(#destructure),* ) => #body,)
        }
        Fields::Named(_) => {
            quote!(#input_type::#ident { #(#destructure),* } => #body,)
        }
    }
}

pub fn double_variant_map_arm(
    input_type: &Type,
    variant: &Variant,
    body: &TokenStream,
) -> TokenStream {
    let Variant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let destructure0 = fields.iter().enumerate().map(|(i, f)| {
        f.ident
            .as_ref()
            .map(|ident| {
                let new_ident = format_ident!("{}_0", ident);
                quote! { #ident: #new_ident}
            })
            .unwrap_or_else(|| {
                let ident = tuple_ident(2 * i);
                quote! { #ident }
            })
    });
    let destructure1 = fields.iter().enumerate().map(|(i, f)| {
        f.ident
            .as_ref()
            .map(|ident| {
                let new_ident = format_ident!("{}_1", ident);
                quote! { #ident: #new_ident}
            })
            .unwrap_or_else(|| {
                let ident = tuple_ident(2 * i + 1);
                quote! { #ident }
            })
    });
    match fields {
        Fields::Unit => quote!((#input_type::#ident, #input_type::#ident) => #body,),
        Fields::Unnamed(_) => {
            quote!((#input_type::#ident( #(#destructure0),* ), #input_type::#ident( #(#destructure1),* )) => #body,)
        }
        Fields::Named(_) => {
            quote!((#input_type::#ident{ #(#destructure0),* }, #input_type::#ident{ #(#destructure1),* }) => #body,)
        }
    }
}

pub fn variant_assign_arm(
    input_type: &Type,
    output_type: &Type,
    variant: &Variant,
    prev_tys: &[Vec<Type>],
    val: impl Fn(&MapField) -> TokenStream,
) -> TokenStream {
    let Variant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let assign = fields
        .iter()
        .enumerate()
        .map(|(i, f)| field_assign(None, i, f, prev_tys[i].clone(), &val));
    variant_map_arm(
        input_type,
        variant,
        &match fields {
            Fields::Unit => quote!(#output_type::#ident),
            Fields::Unnamed(_) => {
                quote!(#output_type::#ident( #(#assign)* ))
            }
            Fields::Named(_) => {
                quote!(#output_type::#ident { #(#assign)* })
            }
        },
    )
}

pub fn double_variant_assign_arm(
    input_type: &Type,
    output_type: &Type,
    variant: &Variant,
    prev_tys: &[Vec<Type>],
    val: impl Fn(&MapField, &MapField) -> TokenStream,
) -> TokenStream {
    let Variant {
        ref ident,
        ref fields,
        ..
    } = variant;
    let destructure0 = fields.iter().enumerate().map(|(i, f)| {
        f.ident
            .as_ref()
            .map(|ident| {
                let new_ident = format_ident!("{}_0", ident);
                quote! { #ident: #new_ident}
            })
            .unwrap_or_else(|| {
                let ident = tuple_ident(2 * i);
                quote! { #ident }
            })
    });
    let destructure1 = fields.iter().enumerate().map(|(i, f)| {
        f.ident
            .as_ref()
            .map(|ident| {
                let new_ident = format_ident!("{}_1", ident);
                quote! { #ident: #new_ident}
            })
            .unwrap_or_else(|| {
                let ident = tuple_ident(2 * i + 1);
                quote! { #ident }
            })
    });
    let assign = fields.iter().enumerate().map(|(i, f)| {
        let Field {
            ref ident, ref ty, ..
        } = f;

        let pretty = pretty_ident(i, ident);
        let refer0 = field_referent(None, 2 * i, f);
        let refer1 = field_referent(None, 2 * i + 1, f);

        let value = val(
            &MapField {
                ty: ty.clone(),
                refer: refer0,
                pretty_ident: pretty.clone(),
                prev_tys: prev_tys[i].clone(),
            },
            &MapField {
                ty: ty.clone(),
                refer: refer1,
                pretty_ident: pretty,
                prev_tys: prev_tys[i].clone(),
            },
        );

        match ident {
            Some(ident) => quote! { #ident: #value, },
            None => quote! { #value, },
        }
    });
    match fields {
        Fields::Unit => quote!((#input_type::#ident, #input_type::#ident) => #output_type::#ident,),
        Fields::Unnamed(_) => {
            quote!((#input_type::#ident( #(#destructure0),* ), #input_type::#ident( #(#destructure1),* )) => #output_type::#ident( #(#assign)* ),)
        }
        Fields::Named(_) => {
            quote!((#input_type::#ident{ #(#destructure0),* }, #input_type::#ident{ #(#destructure1),* }) => #output_type::#ident{ #(#assign)* },)
        }
    }
}

/// Formats the contents of a struct body in the appropriate style.
pub fn struct_body(style: Style, decl: bool, contents: TokenStream) -> TokenStream {
    if decl {
        match style {
            Style::Unit => quote!(;),
            Style::Tuple => quote!( ( #contents ); ),
            Style::Struct => quote!( { #contents } ),
        }
    } else {
        match style {
            Style::Unit => quote!(),
            Style::Tuple => quote!( ( #contents ) ),
            Style::Struct => quote!( { #contents } ),
        }
    }
}

#[derive(Clone)]
pub struct AnnotatedField {
    pub field: Field,
    pub style: Style,
    pub idx: usize,
}

#[derive(Clone)]
pub struct DeriveInputHelper {
    input: DeriveInput,
    referent: TokenStream,
    prev_types: Vec<Vec<syn::Type>>,
    generic_type_bindings: HashMap<Ident, Type>,
    assignments: Vec<TokenStream>,
}

/// Configuration for implementing a trait.
pub struct ImplTrait {
    /// The trait to be implemented.
    pub trait_name: TokenStream,
    /// The trait's body.
    pub trait_body: TokenStream,
    pub extra_generics: Vec<GenericParam>,
    pub extra_where_predicates: Vec<WherePredicate>,
}

pub struct MapField {
    pub ty: Type,
    pub pretty_ident: Ident,
    pub refer: TokenStream,
    pub prev_tys: Vec<Type>,
}

pub fn get_fields(data: &Data) -> Vec<&Field> {
    match &data {
        Data::Struct(s) => s.fields.iter().collect(),
        Data::Enum(e) => e.variants.iter().flat_map(|v| v.fields.iter()).collect(),
        Data::Union(_) => {
            unreachable!()
        }
    }
}

impl DeriveInputHelper {
    pub fn new(input: DeriveInput) -> syn::Result<Self> {
        let num_fields = get_fields(&input.data).len();
        Ok(match &input.data {
            Data::Struct(_) | Data::Enum(_) => DeriveInputHelper {
                input,
                referent: quote! { self },
                prev_types: vec![vec![]; num_fields],
                generic_type_bindings: HashMap::default(),
                assignments: vec![],
            },
            Data::Union(_) => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "this trait cannot be implemented on unions",
                ));
            }
        })
    }

    pub fn get_input(&self) -> &DeriveInput {
        &self.input
    }

    pub fn get_data_decl_token(&self) -> TokenStream {
        match &self.input.data {
            Data::Struct(s) => s.struct_token.to_token_stream(),
            Data::Enum(e) => e.enum_token.to_token_stream(),
            Data::Union(_) => unreachable!(),
        }
    }

    pub fn get_ident(&self) -> &syn::Ident {
        &self.input.ident
    }

    pub fn get_type(&self) -> syn::Type {
        let ident = self.get_ident();
        parse_quote! { #ident }
    }

    pub fn with_ident(mut self, ident: Ident) -> Self {
        self.input.ident = ident;
        self
    }

    pub fn set_ident(&mut self, ident: Ident) -> &mut Self {
        self.input.ident = ident;
        self
    }

    pub fn with_referent(mut self, referent: TokenStream) -> Self {
        self.referent = referent;
        self
    }

    pub fn set_referent(&mut self, referent: TokenStream) -> &mut Self {
        self.referent = referent;
        self
    }

    pub fn add_generic_type_binding(&mut self, ident: Ident, ty: Type) {
        self.generic_type_bindings.insert(ident, ty);
    }

    pub fn custom_split_for_impl(&self) -> (TokenStream, TokenStream, TokenStream) {
        let mut generics = self.input.generics.clone();

        generics.params = self
            .input
            .generics
            .params
            .iter()
            .filter(|p| match p {
                GenericParam::Type(t) => !self.generic_type_bindings.contains_key(&t.ident),
                GenericParam::Lifetime(_) | GenericParam::Const(_) => true,
            })
            .cloned()
            .collect();
        let (imp, _, wher) = generics.split_for_impl();
        let mut custom_ty = TokenStream::new();
        if !self.input.generics.params.is_empty() {
            self.input
                .generics
                .lt_token
                .unwrap_or_default()
                .to_tokens(&mut custom_ty);

            // Print lifetimes before types and consts, regardless of their
            // order in self.params.
            let mut trailing_or_empty = true;
            for param in self.input.generics.params.pairs() {
                if let GenericParam::Lifetime(def) = *param.value() {
                    // Leave off the lifetime bounds and attributes
                    def.lifetime.to_tokens(&mut custom_ty);
                    param.punct().to_tokens(&mut custom_ty);
                    trailing_or_empty = param.punct().is_some();
                }
            }
            for param in self.input.generics.params.pairs() {
                if let GenericParam::Lifetime(_) = **param.value() {
                    continue;
                }
                if !trailing_or_empty {
                    <Token![,]>::default().to_tokens(&mut custom_ty);
                    trailing_or_empty = true;
                }
                match param.value() {
                    GenericParam::Lifetime(_) => unreachable!(),
                    GenericParam::Type(param) => {
                        // Leave off the type parameter defaults
                        if let Some(binding) = self.generic_type_bindings.get(&param.ident) {
                            binding.to_tokens(&mut custom_ty);
                        } else {
                            param.ident.to_tokens(&mut custom_ty);
                        }
                    }
                    GenericParam::Const(param) => {
                        // Leave off the const parameter defaults
                        param.ident.to_tokens(&mut custom_ty);
                    }
                }
                param.punct().to_tokens(&mut custom_ty);
            }

            self.input
                .generics
                .gt_token
                .unwrap_or_default()
                .to_tokens(&mut custom_ty);
        }

        (quote! { #imp }, custom_ty, quote! { #wher })
    }

    pub fn get_full_type(&self) -> syn::Type {
        let (_, ty, _) = self.custom_split_for_impl();
        let ident = self.get_ident();

        parse_quote! { #ident #ty }
    }

    pub fn fields(&self) -> Vec<&Field> {
        match &self.input.data {
            Data::Struct(s) => s.fields.iter().collect(),
            Data::Enum(e) => e.variants.iter().flat_map(|v| v.fields.iter()).collect(),
            Data::Union(_) => {
                unreachable!()
            }
        }
    }

    pub fn fields_mut(&mut self) -> Vec<&mut Field> {
        match &mut self.input.data {
            Data::Struct(s) => s.fields.iter_mut().collect(),
            Data::Enum(e) => e
                .variants
                .iter_mut()
                .flat_map(|v| v.fields.iter_mut())
                .collect(),
            Data::Union(_) => {
                unreachable!()
            }
        }
    }

    pub fn map_types(&mut self, ty_map: impl Fn(&Type) -> Type) {
        let mut prev_types = Vec::new();
        for field in self.fields_mut() {
            let ty = ty_map(&field.ty);
            prev_types.push(std::mem::replace(&mut field.ty, ty));
        }
        for (prev_types, prev_ty) in self.prev_types.iter_mut().zip(prev_types) {
            prev_types.push(prev_ty);
        }
    }

    pub fn push_generic_param(&mut self, param: GenericParam) {
        self.input.generics.params.push(param);
    }

    pub fn push_where_predicate(&mut self, predicate: WherePredicate) {
        self.input
            .generics
            .make_where_clause()
            .predicates
            .push(predicate);
    }

    pub fn push_where_predicate_per_field(
        &mut self,
        ty_map: impl Fn(&Type, &[Type]) -> WherePredicate,
    ) {
        let predicates = self
            .fields()
            .iter()
            .zip(self.prev_types.iter())
            .map(|(f, prev_tys)| ty_map(&f.ty, prev_tys))
            .collect::<Vec<_>>();
        for predicate in predicates {
            self.push_where_predicate(predicate);
        }
    }

    pub fn push_attr(&mut self, attr: Attribute) {
        self.input.attrs.push(attr);
    }

    pub fn decl_data(&self) -> TokenStream {
        let DeriveInput {
            attrs,
            vis,
            generics,
            ident,
            ..
        } = &self.input;

        let where_clause = &generics.where_clause;

        let data_decl_token = self.get_data_decl_token();

        let body = match &self.input.data {
            Data::Struct(s) => {
                let decls = s.fields.iter().map(field_decl).collect::<Vec<_>>();
                let body = struct_body(Style::from(&s.fields), true, quote! { #( #decls )* });
                if let Fields::Unnamed(_) = s.fields {
                    println!(
                        "{:?} {:?}",
                        body.to_string(),
                        decls
                            .iter()
                            .map(|decl| decl.to_string())
                            .collect::<Vec<_>>()
                    );
                    println!(
                        "{:?}",
                        quote! {
                            #(#attrs)*
                            #vis #data_decl_token #ident #generics #where_clause #body
                        }
                        .to_string()
                    );
                }
                body
            }
            Data::Enum(e) => {
                let decls = e.variants.iter().map(variant_decl);
                quote! {
                    {
                        #( #decls )*
                    }
                }
            }
            Data::Union(_) => {
                unreachable!()
            }
        };

        quote! {
            #(#attrs)*
            #vis #data_decl_token #ident #generics #where_clause #body
        }
    }

    /// Maps data of this derive input's type stored at `self.referent` to another type with the same structure.
    ///
    /// `map_fn` takes in the field type and a reference to a field of an instantiation of this
    /// derive input's type and returns a stream of tokens that produce a field of the other type.
    pub fn map_data(
        &self,
        other_type: &Type,
        map_fn: impl Fn(&MapField) -> TokenStream,
    ) -> TokenStream {
        match &self.input.data {
            Data::Struct(s) => {
                let assignments = s.fields.iter().enumerate().map(|(i, f)| {
                    field_assign(
                        Some(&self.referent),
                        i,
                        f,
                        self.prev_types[i].clone(),
                        &map_fn,
                    )
                });
                let body =
                    struct_body(Style::from(&s.fields), false, quote! { #( #assignments )* });

                quote! {
                    #other_type #body
                }
            }
            Data::Enum(e) => {
                let ident = self.get_ident();
                let mut field_idx = 0;
                let arms = e.variants.iter().map(|v| {
                    let num_fields = v.fields.len();
                    let arm = variant_assign_arm(
                        &parse_quote!(#ident),
                        &other_type,
                        v,
                        &self.prev_types[field_idx..field_idx + num_fields],
                        &map_fn,
                    );
                    field_idx += num_fields;
                    arm
                });
                let referent = &self.referent;
                quote! {
                    {
                        match #referent {
                            #(#arms)*
                        }
                    }
                }
            }
            Data::Union(_) => {
                unreachable!()
            }
        }
    }

    // Maps two of the input data simultaneously.
    pub fn double_map_data(
        &self,
        other_type: &Type,
        referents: (&TokenStream, &TokenStream),
        map_fn: impl Fn(&MapField, &MapField) -> TokenStream,
        fallback: TokenStream,
    ) -> TokenStream {
        match &self.input.data {
            Data::Struct(s) => {
                let assignments = s.fields.iter().enumerate().map(|(i, f)| {
                    let Field {
                        ref ident, ref ty, ..
                    } = f;

                    let pretty = pretty_ident(i, ident);
                    let refer0 = field_referent(Some(referents.0), i, f);
                    let refer1 = field_referent(Some(referents.1), i, f);

                    let value = map_fn(
                        &MapField {
                            ty: ty.clone(),
                            refer: refer0,
                            pretty_ident: pretty.clone(),
                            prev_tys: self.prev_types[i].clone(),
                        },
                        &MapField {
                            ty: ty.clone(),
                            refer: refer1,
                            pretty_ident: pretty,
                            prev_tys: self.prev_types[i].clone(),
                        },
                    );

                    match ident {
                        Some(ident) => quote! { #ident: #value, },
                        None => quote! { #value, },
                    }
                });
                let body =
                    struct_body(Style::from(&s.fields), false, quote! { #( #assignments )* });

                quote! {
                    #other_type #body
                }
            }
            Data::Enum(e) => {
                let ident = self.get_ident();
                let mut field_idx = 0;
                let arms = e.variants.iter().map(|v| {
                    let num_fields = v.fields.len();
                    let arm = double_variant_assign_arm(
                        &parse_quote!(#ident),
                        &other_type,
                        v,
                        &self.prev_types[field_idx..field_idx + num_fields],
                        &map_fn,
                    );
                    field_idx += num_fields;
                    arm
                });
                let refer0 = &referents.0;
                let refer1 = &referents.1;
                quote! {
                    {
                        match (#refer0, #refer1) {
                            #(#arms)*
                            _ => #fallback,
                        }
                    }
                }
            }
            Data::Union(_) => {
                unreachable!()
            }
        }
    }

    /// Maps data of this derive input's type stored at `self.referent` to a single return value.
    ///
    /// `map_fn` takes in a list of field types, pretty identifiers, and references to the corresponding field
    /// of an instantiation of this derive input's type and
    /// returns a stream of tokens that produce the desired output.
    pub fn map(&self, map_fn: impl Fn(&[&MapField]) -> TokenStream) -> TokenStream {
        match &self.input.data {
            Data::Struct(s) => {
                let mapped_fields: Vec<_> = s
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| MapField {
                        ty: f.ty.clone(),
                        pretty_ident: pretty_ident(i, &f.ident),
                        refer: field_referent(Some(&self.referent), i, f),
                        prev_tys: self.prev_types[i].clone(),
                    })
                    .collect();
                map_fn(&mapped_fields.iter().collect::<Vec<_>>())
            }
            Data::Enum(e) => {
                let ident = self.get_ident();
                let mut field_idx = 0;
                let arms = e.variants.iter().map(|v| {
                    let mapped_fields: Vec<_> = v
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let field = MapField {
                                ty: f.ty.clone(),
                                pretty_ident: pretty_ident(i, &f.ident),
                                refer: field_referent(None, i, f),
                                prev_tys: self.prev_types[field_idx].clone(),
                            };
                            field_idx += 1;
                            field
                        })
                        .collect();
                    variant_map_arm(
                        &parse_quote!(#ident),
                        v,
                        &map_fn(&mapped_fields.iter().collect::<Vec<_>>()),
                    )
                });
                let referent = &self.referent;
                quote! {
                    {
                        match #referent {
                            #(#arms)*
                        }
                    }
                }
            }
            Data::Union(_) => {
                unreachable!()
            }
        }
    }

    // Maps two of the input data simultaneously.
    pub fn double_map(
        &self,
        referents: (&TokenStream, &TokenStream),
        map_fn: impl Fn(&[(&MapField, &MapField)]) -> TokenStream,
        fallback: TokenStream,
    ) -> TokenStream {
        match &self.input.data {
            Data::Struct(s) => {
                let mapped_fields: Vec<_> = s
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let refer0 = field_referent(Some(referents.0), i, f);
                        let refer1 = field_referent(Some(referents.1), i, f);
                        (
                            MapField {
                                ty: f.ty.clone(),
                                pretty_ident: pretty_ident(i, &f.ident),
                                refer: refer0,
                                prev_tys: self.prev_types[i].clone(),
                            },
                            MapField {
                                ty: f.ty.clone(),
                                pretty_ident: pretty_ident(i, &f.ident),
                                refer: refer1,
                                prev_tys: self.prev_types[i].clone(),
                            },
                        )
                    })
                    .collect();
                map_fn(
                    &mapped_fields
                        .iter()
                        .map(|(a, b)| (a, b))
                        .collect::<Vec<_>>(),
                )
            }
            Data::Enum(e) => {
                let ident = self.get_ident();
                let mut field_idx = 0;
                let arms = e.variants.iter().map(|v| {
                    let mapped_fields: Vec<_> = v
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let field = (
                                MapField {
                                    ty: f.ty.clone(),
                                    pretty_ident: pretty_ident(i, &f.ident),
                                    refer: field_referent(None, i + 1, f),
                                    prev_tys: self.prev_types[field_idx].clone(),
                                },
                                MapField {
                                    ty: f.ty.clone(),
                                    pretty_ident: pretty_ident(i, &f.ident),
                                    refer: field_referent(None, 2 * i + 1, f),
                                    prev_tys: self.prev_types[field_idx].clone(),
                                },
                            );
                            field_idx += 1;
                            field
                        })
                        .collect();
                    double_variant_map_arm(
                        &parse_quote!(#ident),
                        v,
                        &map_fn(
                            &mapped_fields
                                .iter()
                                .map(|(a, b)| (a, b))
                                .collect::<Vec<_>>(),
                        ),
                    )
                });
                let refer0 = referents.0;
                let refer1 = referents.1;
                quote! {
                    {
                        match (#refer0, #refer1) {
                            #(#arms)*
                            _ => #fallback
                        }
                    }
                }
            }
            Data::Union(_) => {
                unreachable!()
            }
        }
    }

    /// Implements the provided trait with the provided trait body, filling in generics based on
    /// the configuration of `self`. Additional generics and where predicates can be added here.
    pub fn impl_trait(&self, config: &ImplTrait) -> TokenStream {
        let ImplTrait {
            trait_name,
            trait_body,
            extra_generics,
            extra_where_predicates,
        } = config;
        let mut other = (*self).clone();
        for param in extra_generics {
            other.push_generic_param(param.clone());
        }
        for where_predicate in extra_where_predicates {
            other.push_where_predicate(where_predicate.clone());
        }

        let ident = &other.input.ident;
        let (_, ty, _) = self.custom_split_for_impl();
        let (imp, _, wher) = other.custom_split_for_impl();

        quote! {
            impl #imp #trait_name for #ident #ty #wher {
                #trait_body
            }
        }
    }
}

/// Configuration for deriving a trait.
pub struct DeriveTrait {
    /// The trait to be implemented.
    pub trait_: TokenStream,
    /// The trait's associated method.
    pub method: TokenStream,
    /// Identifiers for extra arguments to the trait's associated methods.
    pub extra_arg_idents: Vec<TokenStream>,
    /// Types for extra arguments to the trait's associated methods.
    pub extra_arg_tys: Vec<TokenStream>,
}

/// Derives a trait using the given configuration and input.
pub fn derive_trait(config: &DeriveTrait, input: &DeriveInput) -> proc_macro2::TokenStream {
    let DeriveTrait {
        ref trait_,
        ref method,
        ref extra_arg_idents,
        ref extra_arg_tys,
    } = *config;

    let mut generics = input.generics.clone();
    add_trait_bounds(&mut generics, quote!(#trait_));
    let (imp, ty, wher) = generics.split_for_impl();

    let match_clause: TokenStream = match &input.data {
        Data::Struct(ref s) => match &s.fields {
            Fields::Unnamed(fields) => {
                let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let idx = Index::from(i);
                    quote_spanned! { f.span() =>
                        #trait_::#method(&mut self.#idx, #(#extra_arg_idents),*);
                    }
                });
                quote! { #(#recurse)* }
            }
            Fields::Named(fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    quote_spanned! { f.span() =>
                        #trait_::#method(&mut self.#name, #(#extra_arg_idents),*);
                    }
                });
                quote! { #(#recurse)* }
            }
            Fields::Unit => quote!(),
        },
        Data::Enum(ref data) => {
            let clauses = data.variants.iter().map(|v| {
                let inner = match v.fields {
                    syn::Fields::Named(ref fields) => {
                        let recurse = fields.named.iter().map(|f| {
                            let name = f.ident.as_ref().unwrap();
                            quote_spanned! { f.span() =>
                                #trait_::#method(#name, #(#extra_arg_idents),*);
                            }
                        });
                        let declare = fields.named.iter().map(|f| {
                            let name = f.ident.as_ref().unwrap();
                            quote_spanned! { f.span() =>
                                ref mut #name,
                            }
                        });
                        quote! {
                            { #(#declare)* } => { #(#recurse)* },
                        }
                    }
                    syn::Fields::Unnamed(ref fields) => {
                        let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let ident = format_ident!("field{i}");
                            quote_spanned! { f.span() =>
                                #trait_::#method(#ident, #(#extra_arg_idents),*);
                            }
                        });
                        let declare = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let ident = format_ident!("field{i}");
                            quote_spanned! { f.span() =>
                                ref mut #ident,
                            }
                        });
                        quote! {
                            ( #(#declare)* ) => { #(#recurse)* },
                        }
                    }
                    syn::Fields::Unit => quote! { => (), },
                };

                let ident = &v.ident;
                quote! {
                    Self::#ident #inner
                }
            });
            quote! {
                match self {
                    #(#clauses)*
                }
            }
        }
        Data::Union(_) => {
            return syn::Error::new(
                Span::call_site(),
                "this trait cannot be implemented on unions",
            )
            .to_compile_error();
        }
    };

    let ident = &input.ident;

    let extra_args_sig = extra_arg_idents
        .iter()
        .zip(extra_arg_tys)
        .map(|(ident, ty)| {
            quote! {
                #ident: #ty
            }
        });

    quote! {
        impl #imp #trait_ for #ident #ty #wher {
            fn #method(&mut self, #(#extra_args_sig),*) {
                #match_clause
            }
        }
    }
}
