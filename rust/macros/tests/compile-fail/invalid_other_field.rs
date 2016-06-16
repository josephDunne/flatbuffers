#![feature(plugin)]
#![plugin(flatbuffers_macros)]

extern crate flatbuffers_macros;

flatbuffers_object!{Table => Monster [
    field => { name = pos,
               typeOf = Vec3,
               slot = 4,
               default = true },
    field => { typeOf = pos }
    //~^ Error: Expected name, typeOf, and slot, missing default, name, and slot
]}

fn main() {}
