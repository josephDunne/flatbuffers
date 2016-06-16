#![feature(plugin)]
#![plugin(flatbuffers_macros)]

extern crate flatbuffers_macros;

flatbuffers_object!{Table => Monster [
    field => { name = pos,
               typeOf = Vec3,
               wierd = true,
               //~^ Error: Unknown field
               slot = 4 }
]}

fn main() {}
