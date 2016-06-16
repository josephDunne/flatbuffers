#[derive(Debug, PartialEq, Eq)]
pub enum ObjectType {
    Table,
    Struct,
    Enum,
    Union,
}

#[derive(Debug)]
pub struct ObjAttribute {
    pub name: String,
    pub value: String,
}

#[derive(Debug)]
pub struct FieldDef {
    pub name: String,
    pub ty: FieldType,
    pub slot: String,
    pub default: String,
    pub comments: Vec<String>,
}

#[derive(Debug)]
pub enum FieldType {
    Scalar(String),
    Enum(String, Box<FieldType>),
    Union(String),
    Table(String),
    Vector(Box<FieldType>),
}

pub fn map_ty(other: String) -> Option<FieldType> {
    let ft = match &*other {
        "byte" => FieldType::Scalar("i8".to_string()),
        "ubyte" => FieldType::Scalar("u8".to_string()),
        "short" => FieldType::Scalar("i16".to_string()),
        "ushort" => FieldType::Scalar("u16".to_string()),
        "int" => FieldType::Scalar("i32".to_string()),
        "uint" => FieldType::Scalar("u32".to_string()),
        "long" => FieldType::Scalar("i64".to_string()),
        "ulong" => FieldType::Scalar("u64".to_string()),
        "float" => FieldType::Scalar("f32".to_string()),
        "double" => FieldType::Scalar("f64".to_string()),
        "bool" => FieldType::Scalar("bool".to_string()),
        "string" => FieldType::Scalar("&str".to_string()),
        _ => {
            let otherc = other.clone();
            let mut chars = otherc.chars();
            if chars.next() == Some('[') && chars.next_back() == Some(']') {
                // vector
                let ty = chars.as_str()
                    .to_string();
                let ty = map_ty(ty);
                if ty.is_none() {
                    return None;
                }
                FieldType::Vector(Box::new(ty.unwrap()))
            } else if other.starts_with("union ") {
                let mut iter = other.split_whitespace();
                let md = iter.nth(1);
                if md.is_none() {
                    return None;
                }
                FieldType::Union(md.unwrap().to_string())
            } else if other.starts_with("enum ") {
                let mut iter = other.split_whitespace();
                let md = iter.nth(1);
                let ty = iter.nth(2);
                if md.is_none() || ty.is_none() {
                    return None;
                }
                let ty = map_ty(ty.unwrap().to_string());
                if ty.is_none() {
                    return None;
                }
                FieldType::Enum(md.unwrap().to_string(), Box::new(ty.unwrap()))
            } else {
                FieldType::Table(other)
            }
        }
    };
    Some(ft)
}


impl FieldType {
    pub fn get_table_accessor(&self, slot: &str, default: &str) -> String {
        match *self {
            FieldType::Scalar(ref ty) => scalar_accessor(ty, slot, default),
            FieldType::Enum(ref md, ref ty) => enum_accessor(md, &ty, slot, default),
            FieldType::Union(ref md) => union_accessor(md, slot),
            FieldType::Table(_) => table_accessor(slot),
            FieldType::Vector(ref ty) => vector_accessor(&ty, slot),
        }
    }

    pub fn base_type(&self) -> String {
        match *self {
            FieldType::Scalar(ref ty) => ty.to_string(),
            FieldType::Table(ref ty) => ty.to_string(),
            FieldType::Vector(ref ty) => format!("Iter<'a,{}>", ty.base_type()),
            FieldType::Union(ref ty) => ty.to_string(),
            FieldType::Enum(ref md, _) => md.to_string(),
        }
    }
}

fn enum_accessor(md: &str, ty: &FieldType, slot: &str, default: &str) -> String {
    let fun = ty.get_table_accessor(slot, default);
    format!("let v = {}; {}::from(v)", fun, md)
}

fn union_accessor(md: &str, slot: &str) -> String {
    let fun = format!("self.{}_type();", md.to_lowercase());
    let table = format!("let table =  ($i.0).get_slot_table({});", slot);
    format!("{} {} {}::new(table, ty)", fun, table, md.to_lowercase())
}

fn table_accessor(slot: &str) -> String {
    let fun = format!("let t = ($i.0).get_slot_table({})", slot);
    format!("{} if t.is_some() {{ return t.unwrap().into(); }} None",
            fun)
}

fn vector_accessor(ty: &FieldType, slot: &str) -> String {
    let ty = ty.base_type();
    format!("(self.0).get_slot_vector::<{}>({})", ty, slot)
}

fn scalar_accessor(ty: &str, slot: &str, default: &str) -> String {
    match &*ty {
        "i8" => format!("(self.0).get_slot_i8({},{})", slot, default),
        "u8" => format!("(self.0).get_slot_u8({},{})", slot, default),
        "i16" => format!("(self.0).get_slot_i16({},{})", slot, default),
        "u16" => format!("(self.0).get_slot_u16({},{})", slot, default),
        "i32" => format!("(self.0).get_slot_i32({},{})", slot, default),
        "u32" => format!("(self.0).get_slot_u32({},{})", slot, default),
        "i64" => format!("(self.0).get_slot_i64({},{})", slot, default),
        "u64" => format!("(self.0).get_slot_u64({},{})", slot, default),
        "f32" => format!("(self.0).get_slot_f32({},{})", slot, default),
        "f64" => format!("(self.0).get_slot_f64({},{})", slot, default),
        "bool" => format!("(self.0).get_slot_bool({},{})", slot, default),
        "&str" => format!("(self.0).get_slot_str({},{})", slot, default),
        _ => unreachable!(),
    }
}

pub fn find_attribute(name: &str, attributes: &[ObjAttribute]) -> Option<String> {
    for attr in attributes {
        if attr.name == name {
            return Some(attr.value.clone());
        }
    }
    None
}
