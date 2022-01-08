pub mod dynamic;
pub mod serialization;

use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned};
use std::collections::BTreeMap;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Field, Generics, Type, Variant};

pub(crate) static KEYWORD_ATTR_NAME: &str = "serialized";
pub(crate) static KEYWORD_COMPACT: &str = "compact";
pub(crate) static KEYWORD_NO_SERIALIZATION: &str = "no_serialization";
pub(crate) static KEYWORD_NO_DESERIALIZATION: &str = "no_deserialization";

struct Error {
    pub span: Span,
    pub msg: &'static str,
}

pub struct SerializableField<'a> {
    name: &'a Ident,
    ty: &'a Type,
}

pub struct Builder<'a> {
    compact: bool,
    errors: Vec<Error>,
    name: &'a Ident,
    generics: &'a Generics,
    input_data: &'a Data,
    fields: BTreeMap<usize, SerializableField<'a>>,
    no_ser: bool,
    no_de: bool,
}

impl<'a> Builder<'a> {
    pub fn from(input: &DeriveInput) -> Builder {
        let name = &input.ident;
        let mut compact: bool = false;
        let mut no_ser: bool = false;
        let mut no_de: bool = false;
        for attr in &input.attrs {
            if attr.path.is_ident(KEYWORD_COMPACT) {
                compact = true;
                break;
            }
            if attr.path.is_ident(KEYWORD_NO_SERIALIZATION) {
                no_ser = true;
                break;
            }
            if attr.path.is_ident(KEYWORD_NO_DESERIALIZATION) {
                no_de = true;
                break;
            }
        }

        let mut builder = Builder {
            compact,
            errors: Vec::<Error>::new(),
            name,
            generics: &input.generics,
            fields: BTreeMap::<usize, SerializableField>::default(),
            input_data: &input.data,
            no_ser,
            no_de,
        };

        match &input.data {
            Data::Struct(struct_data) => {
                for field in &struct_data.fields {
                    builder.handle_field(field);
                }
            }
            Data::Enum(enum_data) => {
                if builder.compact {
                    builder.add_error(
                        builder.name.span(),
                        "compact attribute on enum makes no sense",
                    );
                } else {
                    for variant in &enum_data.variants {
                        builder.handle_variant(variant);
                    }
                }
            }
            Data::Union(_union_data) => {
                builder.add_error(builder.name.span(), "unions are not supported");
            }
        }

        builder
    }

    fn add_error(&mut self, span: Span, msg: &'static str) {
        self.errors.push(Error { span, msg });
    }

    fn handle_attrs(&mut self, attrs: &[Attribute], name: &'a Ident, ty: &'a Type) {
        let mut got_attr = false;
        for attr in attrs {
            if !attr.path.is_ident(KEYWORD_ATTR_NAME) {
                continue;
            }

            if self.compact {
                // TODO better log
                self.add_error(attr.span(), "compact types cannot have attribute, if you want to add an index, use un-packed type");
                return;
            }

            if got_attr {
                self.add_error(attr.span(), "attr already declared");
                continue;
            } else {
                got_attr = true;
            }

            match attr.parse_args::<syn::LitInt>() {
                Ok(lit_int) => match lit_int.base10_parse() {
                    Ok(k) => {
                        if let std::collections::btree_map::Entry::Vacant(e) = self.fields.entry(k)
                        {
                            let serializable_field = SerializableField { name, ty };
                            e.insert(serializable_field);
                        } else {
                            self.add_error(attr.span(), "field id already used.");
                        }
                    }
                    Err(err) => {
                        self.add_error(err.span(), "cannot parse to decimal");
                    }
                },
                Err(err) => {
                    self.add_error(err.span(), "not a int literal");
                }
            };
        }

        if self.compact && !got_attr {
            let index = self.fields.len();
            self.fields.insert(index, SerializableField { name, ty });
        }
    }

    fn handle_variant(&mut self, variant: &'a Variant) {
        let name = &variant.ident;
        let ty = if variant.fields.len() > 1 {
            self.errors.push(Error {
                span: variant.span(),
                msg: "multiple types are not supported, consider capsule them in a struct type",
            });
            return;
        } else if variant.fields.is_empty() {
            self.errors.push(Error {
                span: variant.span(),
                msg: "WTF: no fields in a variant",
            });
            return;
        } else {
            &variant.fields.iter().last().unwrap().ty
        };

        self.handle_attrs(&variant.attrs, name, ty);
    }

    fn handle_field(&mut self, field: &'a Field) {
        let name: &Ident;
        if let Option::Some(ident) = &field.ident {
            name = ident;
        } else {
            return;
        }
        let ty = &field.ty;
        self.handle_attrs(&field.attrs, name, ty);
    }

    pub fn build(&mut self) -> TokenStream {
        if self.fields.is_empty() {
            self.add_error(self.name.span(), "no serializable field found");
        }

        if self.errors.is_empty() {
            let gen = vec![self.build_serialization()];
            return quote! {
                #(#gen)*
            };
        } else {
            let errors = self.errors.iter().map(|error| {
                let msg = error.msg;
                quote_spanned!(error.span =>
                    compile_error!(#msg);
                )
            });
            quote!(#(#errors)*)
        }
    }
}
