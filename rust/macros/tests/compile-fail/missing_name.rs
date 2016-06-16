#![feature(plugin)]
#![plugin(flatbuffers_macros)]

extern crate flatbuffers_macros;

flatbuffers_object!{Table => Monster [
    field => { typeOf = pos }
    //~^ Error: Expected name, typeOf, and slot, missing default, name, and slot
]}

fn main() {}
