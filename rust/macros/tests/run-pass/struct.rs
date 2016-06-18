#![feature(plugin)]
#![plugin(flatbuffers_macros)]

extern crate flatbuffers_macros;
extern crate flatbuffers;

enum Color {
    Blue = 1
}

struct Test {
    i: u8
}

flatbuffers_object!{Struct => Vec3 ( size:32 ) [
    field => { name = x,
               typeOf = f32,
               slot = 0,
               default = 0.0 }, 
    field => { name = y,
               typeOf = f32,
               slot = 4,
               default = 0.0 }, 
    field => { name = z,
               typeOf = f32,
               slot = 8,
               default = 0.0 }, 
    field => { name = test1,
               typeOf = f64,
               slot = 16,
               default = 0.0 }, 
    field => { name = test2,
               typeOf = enum Color byte,
               slot = 24,
               default = 0 }, 
    field => { name = test3,
               typeOf = Test,
               slot = 26 }]}

fn main() {}
