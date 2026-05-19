use crate::vm::ids::ObjectRef;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Int(i32),
  Long(i64),
  Float(f32),
  Double(f64),
  Ref(Option<ObjectRef>),
  ReturnAddress(u32),
}

impl Value {
  pub fn null() -> Self {
    Self::Ref(None)
  }
}
