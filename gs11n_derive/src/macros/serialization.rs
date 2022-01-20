use crate::Builder;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Index};

impl<'a> Builder<'a> {
    pub(crate) fn build_serialization(&self) -> TokenStream {
        let (encode_statements, record_statements, decode_statements) = match self.input_data {
            Data::Struct(_) => self.get_struct_statements(),
            Data::Enum(_) => self.get_enum_statements(),
            Data::Union(_) => {
                return quote!("WTF: build for Union");
            }
        };
        let name = self.name;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let mut generated = Vec::with_capacity(3);
        generated.push(quote! {
            impl #impl_generics gs11n::WireTypeTrait for #name #ty_generics #where_clause {
                const WIRE_TYPE: gs11n::serialization::wire_type::WireType = gs11n::serialization::wire_type::WireType::LengthDelimited;
            }
        });
        if !self.no_ser {
            generated.push(quote! {
                impl #impl_generics gs11n::Serialization for #name #ty_generics #where_clause {
                    fn encode(&self, ptr: &mut *mut u8, meta_data: &mut gs11n::meta_data::Metadata) {
                        #encode_statements
                    }
                    fn record(&self, meta_data: &mut gs11n::meta_data::Metadata) {
                        #record_statements
                    }
                }
            });
        }

        if !self.no_de {
            generated.push(quote! {
                impl #impl_generics gs11n::DeSerialization for #name #ty_generics #where_clause {
                    fn decode(ptr: &mut *const u8, ctx: &gs11n::serialization::decoder::DecodeContext) -> Result<Self, gs11n::decoder::DecodeError> {
                        #decode_statements
                    }
                }
            });
        }
        return quote! {
            #(#generated)*
        };
    }

    fn get_enum_statements(&self) -> (TokenStream, TokenStream, TokenStream) {
        let enum_name = self.name;

        let mut encode_enum_items: Vec<TokenStream> = Vec::with_capacity(self.fields.len());
        let mut record_enum_items: Vec<TokenStream> = Vec::with_capacity(self.fields.len());
        let mut decode_enum_items: Vec<TokenStream> = Vec::with_capacity(self.fields.len());

        for (id, field) in &self.fields {
            let element_ty = field.ty;
            let element_name = field.name;

            encode_enum_items.push(quote! {
                #enum_name::#element_name(v) => {
                            #id.encode(ptr, meta_data);
                            v.encode(ptr, meta_data);
                }
            });

            record_enum_items.push(quote! {
                #enum_name::#element_name(v) => { v.record(meta_data.get(0)); }
            });

            decode_enum_items.push(quote! {
                #id => {
                            let v = <#element_ty>::decode(ptr, ctx)?;
                            Ok(Self::#element_name(v))
                        }
            });
        }

        return (
            quote! {
                match self {
                    #(#encode_enum_items)*
                }
            },
            quote! {
                use gs11n::unsigned::EncodeSize;
                match self {
                   #(#record_enum_items)*
                }
                let child_size = meta_data.get(0).size;
                meta_data.size = child_size.varint_size() + child_size;
            },
            quote! {
                let id = usize::decode(ptr, ctx)?;
                match id {
                    #(#decode_enum_items)*
                    _ => {
                        Err(gs11n::decoder::DecodeError::InvalidType)
                    }
                }
            },
        );
    }

    fn get_struct_statements(&self) -> (TokenStream, TokenStream, TokenStream) {
        let mut encode_field_stmts = Vec::with_capacity(self.fields.len());
        let mut record_stmts = Vec::with_capacity(self.fields.len());
        let mut size_calculate_stmts = Vec::with_capacity(self.fields.len());
        let mut decode_stmts = Vec::with_capacity(self.fields.len());

        for (id, field) in &self.fields {
            let id = Index::from(*id);
            let field_name = field.name;
            let field_ty = field.ty;

            record_stmts.push(quote! {
                self.#field_name.record(meta_data.get(#id));
            });

            if self.compact {
                encode_field_stmts.push(quote! {
                    self.#field_name.encode(ptr, meta_data.get(#id));
                });

                size_calculate_stmts.push(quote! {
                    + meta_data.get(#id).size
                });

                decode_stmts.push(quote! {
                    v.#field_name = <#field_ty>::decode(ptr, ctx)?;
                })
            } else {
                encode_field_stmts.push(quote! {
                    gs11n::encoder::encode_field(#id, &self.#field_name, ptr, meta_data.get(#id));
                });

                size_calculate_stmts.push(quote! {
                    + gs11n::encoder::size_of_field::<#field_ty>(#id, meta_data.get(#id))
                });

                decode_stmts.push(quote! {
                    #id => v.#field_name = gs11n::decoder::decode_field(ptr, ctx, is_prefab)?,
                })
            }
        }

        return (
            quote! {
                #(#encode_field_stmts)*
            },
            quote! {
                #(#record_stmts)*

                let size = 0
                    #(#size_calculate_stmts)* ;
                    meta_data.size = size;
            },
            if self.compact {
                quote! {
                    let mut v = Self::default();
                    #(#decode_stmts)*
                    Result::Ok(v)
                }
            } else {
                quote! {
                    use gs11n::wire_type::WireType;
                    use gs11n::decoder::decode_wired_id;
                    let mut v = Self::default();
                    while (*ptr).lt(&ctx.bounds_checker.get_bound()) {
                        let (id, wire_type) = decode_wired_id(ptr, ctx)?;
                        let is_prefab = wire_type == WireType::Prefab;
                        match id {
                            #(#decode_stmts)*
                            _ => {
                                 ctx.skip(ptr, wire_type)?;
                            }
                        }
                    }
                    Result::Ok(v)
                }
            },
        );
    }
}
