use gs11n::decoder::Decoder;
use gs11n::encoder::Encoder;

pub trait TypeId {
    const GS11N_TYPE_ID: usize;
}

impl TypeId for i32 {
    const GS11N_TYPE_ID: usize = 1;
}

impl TypeId for char {
    const GS11N_TYPE_ID: usize = 2;
}

#[gs11n_derive::dynamic]
trait ToString {
    fn to_string(&self) -> String;
}

#[gs11n_derive::dynamic]
impl ToString for i32 {
    fn to_string(&self) -> String {
        let mut v = *self;
        let mut s: String = String::new();
        while v != 0 {
            let i: u8 = (v % 10) as u8;
            v /= 10;
            s.push((i + b'0') as char);
        }
        let str = s.as_str();
        let s2 = str.chars().rev().collect();
        s2
    }
}

#[gs11n_derive::dynamic]
impl ToString for char {
    fn to_string(&self) -> String {
        String::from(*self)
    }
}

#[test]
fn dynamic_test() {
    {
        let b1: Box<dyn ToString> = Box::new(256i32);
        let encoder = Encoder::from(&b1);
        let v = encoder.encode();
        let decoder = Decoder::from_data(v);
        let b2: Box<dyn ToString> = decoder.decode().unwrap();
        assert_eq!(b2.to_string(), "256");
    }

    {
        let b1: Box<dyn ToString> = Box::new('x');
        let encoder = Encoder::from(&b1);
        let v = encoder.encode();
        let decoder = Decoder::from_data(v);
        let b2: Box<dyn ToString> = decoder.decode().unwrap();
        assert_eq!(b2.to_string(), "x");
    }
}
