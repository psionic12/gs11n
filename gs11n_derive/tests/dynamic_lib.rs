use gs11n::encoder::Encoder;

pub trait TypeId {
    const GS11N_TYPE_ID: usize;
}

impl TypeId for i32 {
    const GS11N_TYPE_ID: usize = 1;
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

#[no_mangle]
fn get_encode_buffer() -> Vec<u8> {
    let b1: Box<dyn ToString> = Box::new(256i32);
    let encoder = Encoder::from(&b1);
    let v = encoder.encode();
    v
}
