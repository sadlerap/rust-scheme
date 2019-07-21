use std::vec::Vec;

pub enum Value {
    Bool(bool),
    Number(i64), // i64 until numerical tower implemented
    Character(char),
    Vector(Vec<Value>),
    Str(String),
    Bytevector(Vec<u8>)
}
