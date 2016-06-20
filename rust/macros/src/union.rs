use syntax::ext::base::{ExtCtxt, MacResult, MacEager};
use syntax::parse::token;
use syntax::util::small_vector::SmallVector;
use syntax::ext::quote::rt::ExtParseUtils;

use utils::*;

pub fn build_simple_enum(cx: &mut ExtCtxt, name: token::InternedString, ty: token::InternedString, items: Vec<EnumItem>) -> Result<Box<MacResult>, ()> {
    let enum_def = cx.parse_item(build_enum_def(&name, &ty, &items));
    let from_def = cx.parse_item(build_from_def(&name, &ty, &items));
    Ok(MacEager::items(SmallVector::many(vec![enum_def, from_def])))
}

fn build_enum_def(name: &str, ty: &str, items: &[EnumItem]) -> String {
    let mut str1 = format!("#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]\n \
        #[repr({})]\n \
        pub enum {} {{\n", ty, name);
    for item in items {
        str1 = format!("{} {} = {},\n", str1, item.name, item.value);
    }
    format!("{} }}", str1)
}

fn build_from_def(name: &str, ty: &str, items: &[EnumItem]) -> String {
    let mut str1 = format!("impl {} {{ \
            pub fn from(value: {}) -> Option<{}> {{ \
                match value {{ ", name, ty, name);
    for item in items {
        str1 = format!("{} {} => Some({}::{}),\n", str1,
                       item.value,
                       name, item.name);
    }
    format!("{} _ => None }}}}}}", str1)
}
