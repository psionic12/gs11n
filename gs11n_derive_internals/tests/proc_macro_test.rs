use gs11n_derive_internals::dynamic::{expand, Input};
use gs11n_derive_internals::Builder;
use proc_macro2::TokenStream;
use std::str::FromStr;
use syn::DeriveInput;

#[test]
fn duplicated_attr_test() {
    let ts = TokenStream::from_str(
        "
            #[derive(GS11N)]
            struct Foo {
                #[serialized(1)]
                #[serialized(1)]
                i: i32,

            }",
    )
    .unwrap();
    let input = syn::parse2::<DeriveInput>(ts).unwrap();
    assert_eq!(
        r#"compile_error ! ("attr already declared") ;"#,
        Builder::from(&input).build().to_string()
    );
}

#[test]
fn invalid_attr_test() {
    let ts = TokenStream::from_str(
        "
            #[derive(GS11N)]
            struct Foo {
                #[serialized(a)]
                i: i32,
            }",
    )
    .unwrap();
    let input = syn::parse2::<DeriveInput>(ts).unwrap();
    assert_eq!(
        r#"compile_error ! ("not a int literal") ; compile_error ! ("no serializable field found") ;"#,
        Builder::from(&input).build().to_string()
    );

    let ts = TokenStream::from_str(
        "#[derive(GS11N)]
            struct Foo {
                #[serialized(2)]
                i: i32,
                #[serialized(2)]
                j: f32,
            }",
    )
    .unwrap();
    let input = syn::parse2::<DeriveInput>(ts).unwrap();
    assert_eq!(
        r#"compile_error ! ("field id already used.") ;"#,
        Builder::from(&input).build().to_string()
    );
}

#[test]
fn no_serializable_field_test() {
    let ts = TokenStream::from_str(
        "
            #[derive(GS11N)]
            struct Foo {
                i: i32,
                j: f32,
            }",
    )
    .unwrap();
    let input = syn::parse2::<DeriveInput>(ts).unwrap();
    assert_eq!(
        r#"compile_error ! ("no serializable field found") ;"#,
        Builder::from(&input).build().to_string()
    );
}

#[test]
// Used to generate code manually
fn derive_test() {
    let ts = TokenStream::from_str(
        r"
        #[derive(GS11N)]
        struct Position<T: Serialization + Default> {
            #[serialized(0)]
            x: T,
            #[serialized(1)]
            y: T,
        }
    ",
    )
    .unwrap();
    let input = syn::parse2::<DeriveInput>(ts).unwrap();
    println!("{}", Builder::from(&input).build().to_string());
}

#[test]
fn attribute_test() {
    let ts = TokenStream::from_str(
        r"
       trait ToString { fn to_string(&self) -> String; }
        ",
    )
    .unwrap();
    let input = syn::parse2::<Input>(ts).unwrap();
    println!("{}", expand(input))
}

#[test]
fn impl_test() {
    let ts = TokenStream::from_str(
        r"
       impl ToString for i32 {fn to_string(&self) -> String {
        let mut v = self.clone();
        let mut s: String = String::new();
        while v != 0 {
            let i: u8 = (v % 10) as u8;
            v = v / 10;
            s.push((i + '0' as u8) as char);
        }
        let str = s.as_str();
        let s2 = str.chars().rev().collect();
        s2
    }}
        ",
    )
    .unwrap();
    let input = syn::parse2::<Input>(ts).unwrap();
    println!("{}", expand(input))
}
