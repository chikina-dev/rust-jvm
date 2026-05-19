use crate::vm::error::ParseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JavaType {
  Byte,
  Char,
  Double,
  Float,
  Int,
  Long,
  Short,
  Boolean,
  Void,
  Object(String),
  Array(Box<JavaType>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDescriptor {
  pub parameters: Vec<JavaType>,
  pub return_type: JavaType,
}

pub fn parse_field_descriptor(input: &str) -> Result<JavaType, ParseError> {
  let mut parser = DescriptorParser::new(input);
  let ty = parser.parse_field_type()?;
  parser.expect_end()?;
  Ok(ty)
}

pub fn parse_method_descriptor(input: &str) -> Result<MethodDescriptor, ParseError> {
  let mut parser = DescriptorParser::new(input);
  parser.expect_byte(b'(')?;

  let mut parameters = Vec::new();
  while parser.peek() != Some(b')') {
    if parser.is_end() {
      return Err(ParseError::InvalidDescriptor(input.to_string()));
    }
    parameters.push(parser.parse_field_type()?);
  }

  parser.expect_byte(b')')?;
  let return_type = parser.parse_return_type()?;
  parser.expect_end()?;

  Ok(MethodDescriptor {
    parameters,
    return_type,
  })
}

struct DescriptorParser<'a> {
  input: &'a str,
  bytes: &'a [u8],
  pos: usize,
}

impl<'a> DescriptorParser<'a> {
  fn new(input: &'a str) -> Self {
    Self {
      input,
      bytes: input.as_bytes(),
      pos: 0,
    }
  }

  fn parse_return_type(&mut self) -> Result<JavaType, ParseError> {
    if self.peek() == Some(b'V') {
      self.pos += 1;
      return Ok(JavaType::Void);
    }
    self.parse_field_type()
  }

  fn parse_field_type(&mut self) -> Result<JavaType, ParseError> {
    match self.next() {
      Some(b'B') => Ok(JavaType::Byte),
      Some(b'C') => Ok(JavaType::Char),
      Some(b'D') => Ok(JavaType::Double),
      Some(b'F') => Ok(JavaType::Float),
      Some(b'I') => Ok(JavaType::Int),
      Some(b'J') => Ok(JavaType::Long),
      Some(b'S') => Ok(JavaType::Short),
      Some(b'Z') => Ok(JavaType::Boolean),
      Some(b'L') => self.parse_object_type(),
      Some(b'[') => {
        let element = self.parse_field_type()?;
        Ok(JavaType::Array(Box::new(element)))
      }
      _ => Err(ParseError::InvalidDescriptor(self.input.to_string())),
    }
  }

  fn parse_object_type(&mut self) -> Result<JavaType, ParseError> {
    let start = self.pos;
    while let Some(byte) = self.peek() {
      if byte == b';' {
        let name = &self.input[start..self.pos];
        self.pos += 1;
        if name.is_empty() {
          return Err(ParseError::InvalidDescriptor(self.input.to_string()));
        }
        return Ok(JavaType::Object(name.to_string()));
      }
      self.pos += 1;
    }

    Err(ParseError::InvalidDescriptor(self.input.to_string()))
  }

  fn expect_byte(&mut self, expected: u8) -> Result<(), ParseError> {
    match self.next() {
      Some(actual) if actual == expected => Ok(()),
      _ => Err(ParseError::InvalidDescriptor(self.input.to_string())),
    }
  }

  fn expect_end(&self) -> Result<(), ParseError> {
    if self.is_end() {
      Ok(())
    } else {
      Err(ParseError::InvalidDescriptor(self.input.to_string()))
    }
  }

  fn next(&mut self) -> Option<u8> {
    let byte = self.peek()?;
    self.pos += 1;
    Some(byte)
  }

  fn peek(&self) -> Option<u8> {
    self.bytes.get(self.pos).copied()
  }

  fn is_end(&self) -> bool {
    self.pos >= self.bytes.len()
  }
}
