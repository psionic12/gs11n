use gs11n::decoder::Decoder;
use gs11n::plugin::{TraitRegister, REGISTERED_TRAITS};

pub trait TypeId {
    const GS11N_TYPE_ID: usize;
}

impl TypeId for char {
    const GS11N_TYPE_ID: usize = 2;
}

#[gs11n_derive::dynamic]
trait ToString {
    fn to_string(&self) -> String;
}

#[gs11n_derive::dynamic]
impl ToString for char {
    fn to_string(&self) -> String {
        String::from(*self)
    }
}

#[test]
fn plugin_test() {
    // hack, change the full name of trait ToString for testing
    {
        let mut traits = REGISTERED_TRAITS.lock().unwrap();
        let trait_info = traits
            .remove(std::any::type_name::<dyn ToString>())
            .unwrap();
        traits.insert(
            String::from("dyn gs11n_derive_cdylib_dynamic_lib::ToString"),
            trait_info,
        );
    }

    let dylib_path = test_cdylib::build_file("tests/dynamic_lib.rs");
    unsafe {
        let lib = libloading::Library::new(dylib_path).unwrap();
        let sync_traits: libloading::Symbol<fn(caller_register: &TraitRegister)> =
            lib.get(b"sync_traits").unwrap();
        {
            let register_traits = REGISTERED_TRAITS.lock().unwrap();
            sync_traits(&register_traits);
        }

        let get_encode_buffer: libloading::Symbol<fn() -> Vec<u8>> =
            lib.get(b"get_encode_buffer").unwrap();
        let encode_buffer = get_encode_buffer();

        let decoder = Decoder::from_data(encode_buffer);
        let b: Box<dyn ToString> = decoder.decode().unwrap();
        assert_eq!(b.to_string(), "256");
    }
}
