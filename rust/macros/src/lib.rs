#![crate_type = "dylib"]
#![feature(quote, plugin_registrar, rustc_private)]
extern crate rustc_plugin;
extern crate syntax;
extern crate regex;

use syntax::ast;
use syntax::codemap;
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult};
use syntax::parse::token;
use std::ops::Deref;
use rustc_plugin::Registry;

mod table;
mod utils;

use self::utils::*;

const EXPECTED_IDENT_TABLE_ETC: &'static str = "Expected one of 'Table', 'Struct', 'Enum' or 'Union'";
const EXPECTED_FAT_ARROW: &'static str = "Expected '=>'";
const EXPECTED_COLON: &'static str = "Expected ':'";
const EXPECTED_TABLE_FIELDS: &'static str = "Expected a list of fields";
const EXPECTED_FIELD_DEF: &'static str = "Expected field";
const INVALID_FIELD_DEF: &'static str = "Invalid field definition";
const EXPECTED_SLOT_INT: &'static str = "Slot must be an integer";
const EXPECTED_STRING: &'static str = "Comments must be a string";
const UNEXPECTED_ATTRIBUTE: &'static str = "Unknown field";
const UNKNOWN_ATTRIBUTE: &'static str = "Unknown attribute";
const EXPECTED_SIZE_INT: &'static str = "Size must be an integer";

#[plugin_registrar]
pub fn plugin_registrar(registry: &mut Registry) {
    registry.register_macro("flatbuffers_object", expand)
}

fn expand(cx: &mut ExtCtxt, sp: codemap::Span, ast: &[ast::TokenTree]) -> Box<MacResult> {
    if let Ok(res) = expand_impl(cx, sp, ast) {
        res
    } else {
        DummyResult::any(sp)        
    }
}

fn expand_impl(cx: &mut ExtCtxt, sp: codemap::Span, ast: &[ast::TokenTree]) -> Result<Box<MacResult>, ()> {
    if ast.len() == 0 {
        cx.span_err(sp, EXPECTED_IDENT_TABLE_ETC);
        return Err(());
    }
    let ident = try!(expect_ident(cx, sp, &ast[0], &["Table", "Struct", "Enum", "Union"], EXPECTED_IDENT_TABLE_ETC));
    if ast.len() < 2 {
        cx.span_err(sp, EXPECTED_FAT_ARROW);
        return Err(());
    }
    try!(consume_fat_arrow(cx, sp, &ast[1],  EXPECTED_FAT_ARROW));
    let ast = &ast[2..];
    match try!(object_type(ident)) {
        ObjectType::Table  =>  expand_table_object(cx, sp, ast),
        ObjectType::Struct =>  expand_struct_object(cx, sp, ast),
        ObjectType::Enum   =>  expand_enum_object(cx, sp, ast),
        ObjectType::Union  =>  expand_union_object(cx, sp, ast),
    }
}

fn get_ident(cx: &mut ExtCtxt, sp: codemap::Span, ast: &ast::TokenTree, msg: &str) ->  Result<ast::Ident, ()> {
    match ast {
        &ast::TokenTree::Token(_, token::Token::Ident(ident)) => {
            return Ok(ident)                
        }
        _ => {
            cx.span_err(sp, msg);
            return Err(())
        }
    }
}

fn get_lit(cx: &mut ExtCtxt, sp: codemap::Span, ast: &ast::TokenTree, msg: &str) ->  Result<token::Lit, ()> {
    match ast {
        &ast::TokenTree::Token(_, token::Token::Literal(lit, _)) => {
            return Ok(lit)                
        }
        _ => {
            cx.span_err(sp, msg);
            return Err(())
        }
    }
}

fn expect_ident(cx: &mut ExtCtxt, sp: codemap::Span, ast: &ast::TokenTree, name: &[&str], msg: &str) -> Result<ast::Ident, ()> {
    match ast {
        &ast::TokenTree::Token(_, token::Token::Ident(ident)) => {
            let res = name.iter().any(|x| { *x == ident.name.as_str() });
            if res {
                return Ok(ident)                
            } 
        }
        _ => {}
    }
    cx.span_err(sp, msg);
    Err(())
}

fn consume_fat_arrow(cx: &mut ExtCtxt, sp: codemap::Span, ast: &ast::TokenTree, msg: &str) -> Result<(), ()> {
    match *ast {
        ast::TokenTree::Token(_, token::Token::FatArrow) => return Ok(()),
        _ => {}
    }
    cx.span_err(sp, msg);
    Err(())   
}

fn consume_colon(cx: &mut ExtCtxt, sp: codemap::Span, ast: &ast::TokenTree, msg: &str) -> Result<(), ()> {
    match *ast {
        ast::TokenTree::Token(_, token::Token::Colon) => return Ok(()),
        _ => {}
    }
    cx.span_err(sp, msg);
    Err(())   
}

fn consume_eq(cx: &mut ExtCtxt, sp: codemap::Span, ast: &ast::TokenTree, msg: &str) -> Result<(), ()> {
    match *ast {
        ast::TokenTree::Token(_, token::Token::Eq) => return Ok(()),
        _ => {}
    }
    cx.span_err(sp, msg);
    Err(())   
}

fn maybe_comma(ast: &ast::TokenTree) -> bool {
    match *ast {
        ast::TokenTree::Token(_, token::Token::Comma) => return true,
        _ => {}
    }
    false
}

fn object_type(ident: ast::Ident) -> Result<ObjectType,()> {
    match ident.name.as_str().deref() {
        "Table"  => Ok(ObjectType::Table),
        "Struct" => Ok(ObjectType::Struct),
        "Enum"   => Ok(ObjectType::Enum),
        "Union"  => Ok(ObjectType::Union),
        _ => Err(())
    } 
}

fn expand_table_object(cx: &mut ExtCtxt, sp: codemap::Span, ast: &[ast::TokenTree]) -> Result<Box<MacResult>, ()> {
    let name = try!(get_name(cx, sp, ast, &format!("Expected a name for the {}", "Table")));
    let attributes = try!(get_attributes(cx, sp, ast));
    let ast = if attributes.len() > 0 {
        &ast[2..]
    } else {
        &ast[1..]
    };
    let fields =  try!(get_obj_fields(cx, sp, ast, EXPECTED_TABLE_FIELDS, ObjectType::Table));
    table::build_table_items(cx, name, fields, attributes)
}

fn expand_struct_object(cx: &mut ExtCtxt, sp: codemap::Span, ast: &[ast::TokenTree]) -> Result<Box<MacResult>, ()> {
    let _name = try!(get_name(cx, sp, ast, &format!("Expected a name for the {}", "Struct")));
    Err(())
}

fn expand_enum_object(cx: &mut ExtCtxt, sp: codemap::Span, ast: &[ast::TokenTree]) -> Result<Box<MacResult>, ()> {
    let _name = try!(get_name(cx, sp, ast, &format!("Expected a name for the {}", "Enum")));
    Err(())
}

fn expand_union_object(cx: &mut ExtCtxt, sp: codemap::Span, ast: &[ast::TokenTree]) -> Result<Box<MacResult>, ()> {
    let _name = try!(get_name(cx, sp, ast, &format!("Expected a name for the {}", "Union")));
    Err(())   
}

fn get_name(cx: &mut ExtCtxt, sp: codemap::Span, ast: &[ast::TokenTree], msg: &str) -> Result<token::InternedString, ()> {
    if ast.len() > 0 {
        match ast[0] {
            ast::TokenTree::Token(_, token::Token::Ident(ident)) => {
                return Ok(ident.name.as_str())
            }
            _ => {}
        }
    }
    cx.span_err(sp, msg);
    Err(())
}

fn get_attributes(cx: &mut ExtCtxt, sp: codemap::Span, ast: &[ast::TokenTree]) -> Result<Vec<ObjAttribute>, ()> {
    if ast.len() == 0 {
        return Ok(vec![])
    }
    match ast[0] {
        ast::TokenTree::Delimited(_, ref delemented) => {
            if delemented.delim == token::DelimToken::Paren {
                return get_obj_attributes(cx, sp, delemented.as_ref())
            }
        }
        _ => {}
    }
    return Ok(vec![])
}

fn get_obj_attributes(cx: &mut ExtCtxt, sp: codemap::Span, ast: &ast::Delimited) -> Result<Vec<ObjAttribute>, ()> {
    let tts = &ast.tts;
    let mut res = Vec::new();
    if tts.len() == 0 {
        return Ok(res);
    }
    let mut i = 0;
    loop {
        if tts.len() < i + 3 {
            cx.span_err(ast.open_span, INVALID_FIELD_DEF);
            return Err(())
        }
        let attribute = try!(expect_ident(cx, tts[i].get_span(), &tts[i], &["size"], EXPECTED_FIELD_DEF));
        let attribute = attribute.name.as_str().deref().to_string();
        try!(consume_colon(cx, sp, &tts[i+1],  EXPECTED_COLON));
        match &*attribute {
            "size" => {
                let lit = try!(get_lit(cx, tts[i+2].get_span(), &tts[i+2], EXPECTED_SIZE_INT));
                if let token::Lit::Integer(size) = lit {
                    res.push( ObjAttribute{
                        name: attribute,
                        value: size.as_str().deref().to_string()
                    } )
                } else {
                    cx.span_err(tts[i+2].get_span(), EXPECTED_SIZE_INT);
                    return Err(())  
                }
            }
            _ => {
                cx.span_warn(ast.open_span, UNKNOWN_ATTRIBUTE);
            }
        }
        if tts.len() >= i + 4 && maybe_comma(&tts[i+3]) {
            i += 4;
        } else {
            i += 3;
        }
        if tts.len() <= i {
            break;
        }
    }
    Ok(res)
}

fn get_obj_fields(cx: &mut ExtCtxt, sp: codemap::Span, ast: &[ast::TokenTree], msg: &str, objty: ObjectType) -> Result<Vec<FieldDef>, ()> {
    if ast.len() == 0 {
        cx.span_warn(sp, msg);
        return Err(())
    }
    match ast[0] {
        ast::TokenTree::Delimited(_, ref delemented) => {
            return get_obj_fields_impl(cx, sp, delemented.as_ref(), objty);
        }
        _ => {}
    }
    cx.span_err(sp, msg);
    Err(())
}

fn get_obj_fields_impl(cx: &mut ExtCtxt, sp: codemap::Span, ast: &ast::Delimited, objty: ObjectType) -> Result<Vec<FieldDef>, ()> {
    let tts = &ast.tts;
    let mut res = Vec::new();
    if tts.len() == 0 {
        return Ok(res);
    }
    let mut i = 0;
    loop {
        if tts.len() < i + 3 {
            cx.span_err(ast.open_span, INVALID_FIELD_DEF);
            return Err(())
        }
        try!(expect_ident(cx, tts[i].get_span(), &tts[i], &["field"], EXPECTED_FIELD_DEF));
        try!(consume_fat_arrow(cx, sp, &tts[i+1],  EXPECTED_FAT_ARROW));
        let def = try!(get_field_def(cx, &tts[i+2], &objty));
        res.push(def);
        if tts.len() >= i + 4 && maybe_comma(&tts[i+3]) {
            i += 4;
        } else {
            i += 3;
        }
        if tts.len() <= i {
            break;
        }
    }
    Ok(res)
}

fn get_field_def(cx: &mut ExtCtxt, ast: &ast::TokenTree, objty: &ObjectType) -> Result<FieldDef, ()> {
    if let &ast::TokenTree::Delimited(_, ref delemented) = ast {
        return parse_field_names(cx, ast.get_span(), &delemented.tts, objty)
    }
    cx.span_err(ast.get_span(), INVALID_FIELD_DEF);
    Err(())
}

fn parse_field_names(cx: &mut ExtCtxt, sp: codemap::Span, ast: &Vec<ast::TokenTree>, objty: &ObjectType) -> Result<FieldDef, ()> {
    if ast.len() == 0 {
        cx.span_err(sp, INVALID_FIELD_DEF);
        return Err(())
    }
    let mut def = FieldDef {
        name: "".to_string(),
        ty: FieldType::Scalar("i8".to_string()),
        slot: "".to_string(),
        default: "".to_string(),
        comments: Vec::new()
    };
    let list = ["name", "typeOf", "slot"];
    let attributes = ["name", "typeOf", "slot", "comment", "default"];
    let mut required = if *objty == ObjectType::Table {
        vec!["default", "name", "slot", "typeOf"] 
    } else {
        vec!["name", "slot", "typeOf"]
    };
    let mut i = 0;
    loop {
        let ident = try!(expect_ident(cx, ast[i].get_span(), &ast[i], &attributes, UNEXPECTED_ATTRIBUTE));
        try!(consume_eq(cx, sp, &ast[i+1],  INVALID_FIELD_DEF));
        match ident.name.as_str().deref() {
            "name" => {
                let ident = try!(get_ident(cx, ast[i+2].get_span(), &ast[i+2], INVALID_FIELD_DEF));
                remove_required(&mut required, "name");
                def.name = ident.name.as_str().deref().to_string();
            }
            "typeOf" => {
                let ident = try!(get_ident(cx, ast[i+2].get_span(), &ast[i+2], INVALID_FIELD_DEF));
                let ty = ident.name.as_str()
                                   .deref()
                                   .to_string();
                let ty = map_ty(ty);
                if ty.is_none() {
                    cx.span_err(ast[i+2].get_span(), INVALID_FIELD_DEF);
                    return Err(())
                }
                remove_required(&mut required, "typeOf");
                def.ty = ty.unwrap();
            }
            "default" => {
                let ident = try!(get_ident(cx, ast[i+2].get_span(), &ast[i+2], INVALID_FIELD_DEF));
                remove_required(&mut required, "default");
                def.default = ident.name.as_str().deref().to_string();
            }
            "slot" => {
                let lit = try!(get_lit(cx, ast[i+2].get_span(), &ast[i+2], EXPECTED_SLOT_INT));
                remove_required(&mut required, "slot");
                if let token::Lit::Integer(slot) = lit {
                    def.slot = slot.to_string();
                } else {
                    cx.span_err(ast[i+2].get_span(), EXPECTED_SLOT_INT);
                    return Err(())  
                }
            }
            "comment" => {
                println!("struct {:?}", ast[i+2]);
                let lit = try!(get_lit(cx, ast[i+2].get_span(), &ast[i+2], EXPECTED_STRING));
                if let token::Lit::Str_(s) = lit {
                    def.comments.push(s.as_str().to_string())
                }
                
            }
            _ => {
                cx.span_err(ast[i].get_span(), INVALID_FIELD_DEF);
                return Err(())
            }
        }
        if ast.len() >= i + 3 || maybe_comma(&ast[i+3]) {
            i += 4;
        } else {
            i += 3;
        }
        if ast.len() <= i {
            break;
        }
    }
    if required.len() > 0 {
        fn tokens_to_string(tokens: &[&str]) -> String {
            let mut i = tokens.iter();
            let b = i.next()
                     .map_or("".to_string(), |t| t.to_string());
            i.enumerate().fold(b, |mut b, (i, ref a)| {
                if tokens.len() > 2 && i == tokens.len() - 2 {
                    b.push_str(", and ");
                } else if tokens.len() == 2 && i == tokens.len() - 2 {
                    b.push_str(" and ");
                } else {
                    b.push_str(", ");
                }
                b.push_str(&a.to_string());
                b
            })
        }
        let msg = format!("Expected {}, missing {}",
                          tokens_to_string(&list),
                          tokens_to_string(&required));
        cx.span_err(sp, &msg);
        return Err(())
    }
    Ok(def)
}


fn remove_required(required: &mut Vec<&str>, found: &str) {
    if let Ok(i) = required.binary_search(&&found) {
        required.remove(i);
    }
}

// #[cfg(test)]
// mod tests {

    

//     flatbuffers_object!{Monster =>
//        { field => { name = pos,
//                     typeOf = Vec3,
//                     slot = 4,
//                     comment = "///Testing all my ways are dead\ntestomg"},
//          field => { name = mana,
//                     typeOf = i16,
//                     slot = 6,
//                     default = 150,
//                     comment = "///Testing all my ways are dead\ntestomg",
//                     comment = "///Testing all my ways are dead\ntestomg"},
//          field => { name = color,
//                     typeOf = Color(i8),
//                     slot = 6,
//                     default = 8,
//                     comment = "///Testing all my ways are dead\ntestomg",
//                     comment = "///Testing all my ways are dead\ntestomg"},
//          field => { name = test_type,
//                     typeOf = AnyType(u8),
//                     slot = 8,
//                     default = 0,
//                     comment = "///Testing all my ways are dead\ntestomg",
//                     comment = "///Testing all my ways are dead\ntestomg"},
//          field => { name = test,
//                     typeOf = Union(Any = test_type),
//                     slot = 10,
//                     default = 0,
//                     comment = "///Testing all my ways are dead\ntestomg",
//                     comment = "///Testing all my ways are dead\ntestomg"},
//        }
// }


//     #[test]
//     fn it_works() {
//     }
// }
