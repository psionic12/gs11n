use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{parse_quote, Attribute, Error, ItemImpl, ItemTrait, Token, Type, TypePath, Visibility};

pub fn expand(input: Input) -> TokenStream {
    match input {
        Input::Trait(input) => expand_trait(input),
        Input::Impl(input) => expand_impl(input),
    }
}

pub fn expand_trait(mut trait_input: ItemTrait) -> TokenStream {
    trait_input.items.push(parse_quote! {
        #[doc(hidden)]
        fn type_id(&self) -> usize;
    });
    trait_input.items.push(parse_quote! {
        #[doc(hidden)]
        fn dyn_encode(&self, ptr: &mut *mut u8, meta_data: &mut gs11n::meta_data::Metadata);
    });
    trait_input.items.push(parse_quote! {
        #[doc(hidden)]
        fn dyn_record(&self, meta_data: &mut gs11n::meta_data::Metadata);
    });

    let trait_name = &trait_input.ident;
    let const_name = syn::Ident::new(
        &format!(
            "_DYN_METADATA_FOR_{}",
            trait_name.to_string().to_case(Case::UpperSnake)
        ),
        trait_name.span(),
    );
    // TODO check crate type, avoid dereference if not a dylib
    // https://github.com/rust-lang/rust/issues/20267
    return quote! {
        #trait_input
        const #const_name: () = {
            lazy_static::lazy_static! {
                static ref VTABLE : gs11n::dynamic::VTable<dyn #trait_name> =  std::sync::RwLock::new(rustc_hash::FxHashMap::default());
            }
            impl dyn #trait_name {
                #[doc(hidden)]
                pub fn register_type(id: usize, decoder: gs11n::dynamic::DecodeFn<dyn #trait_name>) {
                    let mut v_table = VTABLE.write().unwrap();
                    v_table.insert(id, decoder);
                }
            }
            impl gs11n::WireTypeTrait for Box<dyn #trait_name> {
                const WIRE_TYPE: gs11n::serialization::wire_type::WireType = gs11n::serialization::wire_type::WireType::LengthDelimited;
            }
            impl gs11n::Serialization for Box<dyn #trait_name> {
                fn encode(&self, ptr: &mut *mut u8, meta_data: &mut gs11n::meta_data::Metadata) {
                    // encode type id
                    self.type_id().encode(ptr, meta_data);
                    self.dyn_encode(ptr, meta_data);
                }

                fn record(&self, meta_data: &mut gs11n::meta_data::Metadata) {
                    self.dyn_record(meta_data);
                }
            }
            impl gs11n::DeSerialization for Box<dyn #trait_name> {
                fn decode(ptr: &mut *const u8, ctx: &gs11n::decoder::DecodeContext) -> Result<Self, gs11n::decoder::DecodeError> {
                    // read type id
                    let id = usize::decode(ptr, ctx)?;
                    let v_table = REF_VTABLE.load(std::sync::atomic::Ordering::Relaxed);
                    let v_table = unsafe { &*v_table }.read().unwrap();

                    match v_table.get(&id) {
                        Some(decode_fn) => {
                            let the_box = decode_fn(ptr, ctx)?;
                            Ok(the_box)
                        }
                        None => {
                            Err(gs11n::decoder::DecodeError::InvalidType)
                        }
                    }
                }
            }

            lazy_static::lazy_static! {
                 static ref REF_VTABLE: std::sync::atomic::AtomicPtr<gs11n::dynamic::VTable<dyn #trait_name>>
                        = std::sync::atomic::AtomicPtr::new(&*VTABLE as *const gs11n::dynamic::VTable<dyn #trait_name> as *mut gs11n::dynamic::VTable<dyn #trait_name>);
            }

            pub unsafe fn sync_trait(v_table: &gs11n::plugin::UnsafeVTable) {
                let v_table: &gs11n::dynamic::VTable<dyn #trait_name> = std::mem::transmute(v_table);
                {
                    let mut caller = v_table.write().unwrap();
                    let callee = VTABLE.read().unwrap();
                    for (id, decode_fn) in &*callee {
                        caller.insert(*id, *decode_fn);
                    }
                }
                REF_VTABLE.store(v_table
                                     as *const std::sync::RwLock<rustc_hash::FxHashMap<usize, gs11n::dynamic::DecodeFn<dyn #trait_name>>>
                                     as *mut std::sync::RwLock<rustc_hash::FxHashMap<usize, gs11n::dynamic::DecodeFn<dyn #trait_name>>>,
                                 std::sync::atomic::Ordering::Relaxed);
            }

            #[ctor::ctor]
            unsafe fn register_trait() {
                let vtable : &gs11n::dynamic::VTable<dyn #trait_name> = &*VTABLE;
                let mut register = gs11n::plugin::REGISTERED_TRAITS.lock().unwrap();
                let trait_info = gs11n::plugin::TraitInfo {
                    vtable: std::mem::transmute(vtable),
                    update_fn: sync_trait,
                };
                register.insert(String::from(std::any::type_name::<dyn #trait_name>()), trait_info);
            }
        };
    };
}

pub fn expand_impl(mut impl_input: ItemImpl) -> TokenStream {
    let self_name = match impl_input.self_ty.as_ref() {
        Type::Path(TypePath { qself: None, path }) => &path.segments.last().unwrap().ident,
        _ => {
            return quote_spanned!(impl_input.span() => compile_error!("s11n dynamic: unrecognized syntax"));
        }
    };

    let trait_name = match &impl_input.trait_ {
        Some(trait_) => &trait_.1.segments.last().unwrap().ident,
        None => {
            return quote_spanned!(impl_input.span() => compile_error!("s11n dynamic: must a trait impl"));
        }
    };

    impl_input.items.push(parse_quote! {
        fn type_id(&self) -> usize {
            Self:: GS11N_TYPE_ID
        }
    });

    impl_input.items.push(parse_quote! {
        fn dyn_encode(&self, ptr: &mut *mut u8, meta_data: &mut gs11n::meta_data::Metadata) {
            use gs11n::Serialization;
            self.encode(ptr, meta_data);
        }
    });

    impl_input.items.push(parse_quote! {
        fn dyn_record(&self, meta_data: &mut gs11n::meta_data::Metadata) {
            use gs11n::Serialization;
            use gs11n::unsigned::EncodeSize;
            self.record(meta_data.get(0));
            meta_data.size = meta_data.get(0).size + Self:: GS11N_TYPE_ID.size();
        }
    });

    let fn_name = syn::Ident::new(
        &format!(
            "register_{}_for_{}",
            self_name.to_string().to_case(Case::Snake),
            trait_name.to_string().to_case(Case::Snake)
        ),
        impl_input.span(),
    );

    return quote! {
        #impl_input
        #[ctor::ctor]
        fn #fn_name () {
            <dyn #trait_name>::register_type(#self_name:: GS11N_TYPE_ID, |ptr: &mut *const u8, ctx: &gs11n::decoder::DecodeContext| {
                use gs11n::DeSerialization;
                let v = #self_name::decode(ptr, ctx)?;
                Ok(Box::new(v))
            })
        }
    };
}

pub enum Input {
    Trait(ItemTrait),
    Impl(ItemImpl),
}

// copied from typetag
impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = Attribute::parse_outer(input)?;

        let ahead = input.fork();
        ahead.parse::<Visibility>()?;
        ahead.parse::<Option<Token![unsafe]>>()?;

        if ahead.peek(Token![trait]) {
            let mut item: ItemTrait = input.parse()?;
            attrs.extend(item.attrs);
            item.attrs = attrs;
            Ok(Input::Trait(item))
        } else if ahead.peek(Token![impl]) {
            let mut item: ItemImpl = input.parse()?;
            if item.trait_.is_none() {
                let impl_token = item.impl_token;
                let ty = item.self_ty;
                let span = quote!(#impl_token #ty);
                let msg = "expected impl Trait for Type";
                return Err(Error::new_spanned(span, msg));
            }
            attrs.extend(item.attrs);
            item.attrs = attrs;
            Ok(Input::Impl(item))
        } else {
            Err(input.error("expected trait or impl block"))
        }
    }
}
