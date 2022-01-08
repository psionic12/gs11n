//! # GS11N
//!
//! GS11N is a ***S***erializatio***n*** crate which focus on ***G***ame development. By adding
//! attributes, your rust codes will looks like scripts in Unity or UE
//!
//! The minimum Rust version required to use GS11N is 1.57.0
//!
//! ```ignore
//! #[derive(GS11N, Default)]
//! struct Orc {
//!   #[serialized(0)]
//!   health: usize,
//!   #[serialized(1)]
//!   mana: usize,
//! }
//! ```
//!
//! then serialize and deserialize it:
//! ```ignore
//! let encoder = Encoder::from(&orc);
//! let buffer = encoder.encode();
//!
//! let decoder = Decoder::from_data(buffer);
//! let orc2: Orc = decoder.decode().unwrap();
//! ```
//!
//! Notice that you struct type ***must*** implement trait `Default` for now, this restriction may be relaxed in the future.
//!
//! GS11N use some ideas from Protobuf, which are:
//! 1. Every field of a struct or enum can (not must) have an ID, which makes different versions of your types compatible.
//! 2. Use varint encoding to reduce the size of the serialization.
//!
//! Other features are:
//! 1. You can serialize a `dyn` type, but need to give the type an type ID (check tests for a example)
//! 2. Your `dyn` types can be compiled into a dynamic library, and load it later, this can be useful when debugging
//! or code updating (for example a hot fix or a DLC).
//! 3. Prefab Loader: you can offer a prefab loader when create a decoder, in which you can
//! bind GS11N to your resources system.
//! 4. You can use to serialize a type as a whole, usually some common types, to simplify the code:
//! ```ignore
//! #[derive(PartialEq, Debug, GS11N, Default)]
//! #[compact]
//! struct Color {
//!   r: u8,
//!   g: u8,
//!   b: u8,
//! };
//! ```
//! notice that is you choose to do this, the encoded date will not compatible if fields are added or removed
//!
//! if you do not want to generate serialization or deserialization code:
//! ```no_ignore
//! #[derive(PartialEq, Debug, GS11N, Default)]
//! #[no_deserialization]
//! struct Foo {
//!   str: &'static str,
//! }
//! ```
//!
//! Features in progressing:
//! 1. Optional information about serialized data, can be useful for game editors.
//! 2. Comments instead of attributes.
//! 3. Readable format of (de)serialization data
//! 4. Do not encode filed which values are default (this may need unstable feature)
//!

extern crate proc_macro;

mod macros;

use crate::macros::*;

use crate::dynamic::{expand, Input};
use crate::Builder;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(
    GS11N,
    attributes(serialized, compact, no_serialization, no_deserialization)
)]
pub fn s11n_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    Builder::from(&input).build().into()
}

#[proc_macro_attribute]
pub fn dynamic(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);
    expand(input).into()
}
