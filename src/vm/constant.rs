use crate::{
  structure::class::{Constant, ConstantPool},
  util::hex::hex_utf8,
  vm::{
    error::ParseError,
    ids::{ClassIndex, CpIndex},
  },
};

pub trait ConstantPoolExt {
  fn utf8(&self, index: CpIndex) -> Result<String, ParseError>;
  fn class_name(&self, index: ClassIndex) -> Result<String, ParseError>;
  fn name_and_type(&self, index: CpIndex) -> Result<(String, String), ParseError>;
}

impl ConstantPoolExt for ConstantPool {
  fn utf8(&self, index: CpIndex) -> Result<String, ParseError> {
    match self.get_class(index.0) {
      Ok(Constant::Utf8 { bytes, .. }) => Ok(hex_utf8(bytes)),
      Ok(_) => Err(ParseError::ClassFormat(format!(
        "constant pool #{} is not a Utf8 entry",
        index.0
      ))),
      Err(_) => Err(ParseError::InvalidConstantPoolIndex(index.0)),
    }
  }

  fn class_name(&self, index: ClassIndex) -> Result<String, ParseError> {
    match self.get_class(index.0) {
      Ok(Constant::Class { name_index }) => self.utf8(CpIndex(*name_index)),
      Ok(_) => Err(ParseError::ClassFormat(format!(
        "constant pool #{} is not a Class entry",
        index.0
      ))),
      Err(_) => Err(ParseError::InvalidConstantPoolIndex(index.0)),
    }
  }

  fn name_and_type(&self, index: CpIndex) -> Result<(String, String), ParseError> {
    match self.get_class(index.0) {
      Ok(Constant::NameAndType {
        name_index,
        descriptor_index,
      }) => Ok((
        self.utf8(CpIndex(*name_index))?,
        self.utf8(CpIndex(*descriptor_index))?,
      )),
      Ok(_) => Err(ParseError::ClassFormat(format!(
        "constant pool #{} is not a NameAndType entry",
        index.0
      ))),
      Err(_) => Err(ParseError::InvalidConstantPoolIndex(index.0)),
    }
  }
}
