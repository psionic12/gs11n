use gs11n::decoder::Decoder;
use gs11n::encoder::Encoder;
use gs11n::{DeSerialization, Serialization};
use gs11n_derive::GS11N;

#[derive(PartialEq, Debug, GS11N, Default)]
#[compact]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(PartialEq, Debug, GS11N, Default)]
struct Position<T: Serialization + DeSerialization + Default> {
    #[serialized(0)]
    x: T,
    #[serialized(1)]
    y: T,
}

#[derive(PartialEq, Debug, GS11N)]
enum Baz {
    #[serialized(0)]
    I32(i32),
    #[serialized(1)]
    F32(f32),
}

impl Default for Baz {
    fn default() -> Self {
        Baz::I32(1)
    }
}

#[derive(GS11N, Default)]
struct Foo {
    #[serialized(0)]
    f_0: i16,
    #[serialized(1)]
    f_1: f32,
    #[serialized(2)]
    f_2: u16,
    #[serialized(3)]
    f_3: Vec<i32>,
    #[serialized(4)]
    f_4: Option<i32>,
    #[serialized(5)]
    f_5: Option<i32>,
    #[serialized(6)]
    f_6: Baz,
    #[serialized(7)]
    f_7: Color,
    #[serialized(8)]
    f_8: String,
    #[serialized(9)]
    f_9: [u32; 3],
    #[serialized(30)]
    f_30: u16,
    #[serialized(31)]
    f_31: u32,
}

#[test]
fn serialization_test() {
    let foo = Foo {
        f_0: -1,
        f_1: 0.1,
        f_2: 0x80,
        f_3: vec![1, 10, 100, 1000],
        f_4: None,
        f_5: Some(-1),
        f_6: Baz::F32(std::f32::consts::PI),
        f_7: Color {
            r: 255,
            g: 255,
            b: 255,
        },
        f_8: String::from("test"),
        f_9: [1, 2, 3],
        f_30: 0,
        f_31: 0x80,
    };
    let encoder = Encoder::from(&foo);
    let real = encoder.encode();
    let expected: Vec<u8> = vec![
        // f_0
        0b110_00000,
        0x1,
        // f_1
        0b010_00001,
        0xCD,
        0xCC,
        0xCC,
        0x3D,
        // f_2
        0b110_00010,
        0x80,
        0x1,
        // f_3
        0b111_00011,
        0x7,
        0x4,
        2,
        20,
        200,
        1,
        208,
        15,
        // f_4
        0b111_00100,
        1,
        0,
        // f_5
        0b111_00101,
        0x2,
        0x1,
        0x1,
        // f_6
        0b111_00110,
        0x5,
        0x1,
        0xDB,
        0xF,
        0x49,
        0x40,
        // f_7
        0b111_00111,
        0x6,
        0xFF,
        0x1,
        0xFF,
        0x1,
        0xFF,
        0x1,
        // f_8
        0b111_01000,
        0x5,
        0x4,
        0x74,
        0x65,
        0x73,
        0x74,
        // f_9
        0b111_01001,
        4,
        3,
        1,
        2,
        3,
        // f_30
        0b110_11110,
        0x0,
        // f_31
        0b110_11111,
        0x1,
        0x80,
        0x1,
    ];
    assert_eq!(real, expected);

    let decoder = Decoder::from_data(real.as_slice());
    let foo2 = decoder.decode::<Foo>().unwrap();
    assert_eq!(foo.f_0, foo2.f_0);
    assert_eq!(foo.f_1, foo2.f_1);
    assert_eq!(foo.f_2, foo2.f_2);
    assert_eq!(foo.f_3, foo2.f_3);
    assert_eq!(foo.f_4, foo2.f_4);
    assert_eq!(foo.f_5, foo2.f_5);
    assert_eq!(foo.f_6, foo2.f_6);
    assert_eq!(foo.f_7, foo2.f_7);
    assert_eq!(foo.f_8, foo2.f_8);
    assert_eq!(foo.f_9, foo2.f_9);
    assert_eq!(foo.f_30, foo2.f_30);
    assert_eq!(foo.f_31, foo2.f_31);
}
