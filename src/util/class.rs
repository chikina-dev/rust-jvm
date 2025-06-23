use nom::{
  bytes::complete::take,
  number::complete::{be_u8,be_u16,be_u32},
  IResult,
};

use crate::{structure::class::{ Constant, ConstantPool }, util::hex::hex_utf8};

use std::mem::discriminant;

pub fn java_version_name(major: u16) -> &'static str {
  match major {
    69 => "Java SE 25",
    68 => "Java SE 24",
    67 => "Java SE 23",
    66 => "Java SE 22",
    65 => "Java SE 21",
    64 => "Java SE 20",
    63 => "Java SE 19",
    62 => "Java SE 18",
    61 => "Java SE 17",
    60 => "Java SE 16",
    59 => "Java SE 15",
    58 => "Java SE 14",
    57 => "Java SE 13",
    56 => "Java SE 12",
    55 => "Java SE 11",
    54 => "Java SE 10",
    53 => "Java SE 9",
    52 => "Java SE 8",
    51 => "Java SE 7",
    50 => "Java SE 6.0",
    49 => "Java SE 5.0",
    48 => "JDK 1.4",
    47 => "JDK 1.3",
    46 => "JDK 1.2",
    45 => "JDK 1.1",
    _ => "Unknown Java Version",
  }
}

pub fn constant_pool_viewer(constant_pool: &[Constant]) {
  let mut e = constant_pool.iter().enumerate();
  while let Some((i, constant)) = e.next() {
    match constant {
      Constant::Utf8 { length: _, bytes } => {
        let string = hex_utf8(&bytes);
        println!("{:>4} = UTF8 \"{}\"", format!("#{}", i + 1), string);
      },
      _ => {
        println!("{:>4} = {:?}", format!("#{}", i + 1), constant);
      }
    }
  }
}

pub fn parse_constant_pool(count: u16, input: &[u8]) -> IResult<&[u8], ConstantPool> {
  let mut constants = Vec::new();
  let mut remaining_input = input;

  for _ in 0..count {
    let (input, tag) = be_u8(remaining_input)?;

    let constant: Constant = match tag {
      7 => {
        let (input, name_index) = be_u16(input)?;
        remaining_input = input;
        Constant::Class { name_index }
      },
      9 => {
        let (input, class_index) = be_u16(input)?;
        let (input, name_and_type_index) = be_u16(input)?;
        remaining_input = input;
        Constant::Fieldref { class_index, name_and_type_index }
      },
      10 => {
        let (input, class_index) = be_u16(input)?;
        let (input, name_and_type_index) = be_u16(input)?;
        remaining_input = input;
        Constant::Methodref { class_index, name_and_type_index }
      },
      11 => {
        let (input, class_index) = be_u16(input)?;
        let (input, name_and_type_index) = be_u16(input)?;
        remaining_input = input;
        Constant::InterfaceMethodref { class_index, name_and_type_index }
      },
      8 => {
        let (input, string_index) = be_u16(input)?;
        remaining_input = input;
        Constant::String { string_index }
      },
      3 => {
        let (input, bytes) = be_u32(input)?;
        remaining_input = input;
        Constant::Integer { bytes }
      },
      4 => {
        let (input, bytes) = be_u32(input)?;
        remaining_input = input;
        Constant::Float { bytes }
      },
      5 => {
        let (input, high_bytes) = be_u32(input)?;
        let (input, low_bytes) = be_u32(input)?;
        remaining_input = input;
        Constant::Long { high_bytes, low_bytes }
      },
      6 => {
        let (input, high_bytes) = be_u32(input)?;
        let (input, low_bytes) = be_u32(input)?;
        remaining_input = input;
        Constant::Double { high_bytes, low_bytes }
      },
      12 => {
        let (input, name_index) = be_u16(input)?;
        let (input, descriptor_index) = be_u16(input)?;
        remaining_input = input;
        Constant::NameAndType { name_index, descriptor_index }
      },
      1 => {
        let (input, length) = be_u16(input)?;
        let (input, bytes) = take(length as usize)(input)?;
        remaining_input = input;
        let bytes = bytes.to_vec();
        Constant::Utf8 { length, bytes }
      },
      15 => {
        let (input, reference_kind) = be_u8(input)?;
        let (input, reference_index) = be_u16(input)?;
        remaining_input = input;
        Constant::MethodHandle { reference_kind, reference_index }
      },
      16 => {
        let (input, descriptor_index) = be_u16(input)?;
        remaining_input = input;
        Constant::MethodType { descriptor_index }
      },
      17 => {
        let (input, bootstrap_method_attr_index) = be_u16(input)?;
        let (input, name_and_type_index) = be_u16(input)?;
        remaining_input = input;
        Constant::Dynamic { bootstrap_method_attr_index, name_and_type_index }
      },
      18 => {
        let (input, bootstrap_method_attr_index) = be_u16(input)?;
        let (input, name_and_type_index) = be_u16(input)?;
        remaining_input = input;
        Constant::InvokeDynamic { bootstrap_method_attr_index, name_and_type_index }
      },
      19 => {
        let (input, name_index) = be_u16(input)?;
        remaining_input = input;
        Constant::Module { name_index }
      },
      20 => {
        let (input, name_index) = be_u16(input)?;
        remaining_input = input;
        Constant::Package { name_index }
      },
      _ => {
        println!("Unknown constant type: {}", tag);
        Constant::Unknown
      },
    };
    constants.push(constant);
  }

  Ok((remaining_input, ConstantPool {
    constants,
    count,
  }))
}

pub fn check_constant_pool_class(constant: &Constant) -> Result<&Constant, String> {
  let index = discriminant(&Constant::Class { name_index: 0 });
  if index == discriminant(constant) {
    Ok(&constant)
  } else {
    Err(format!("Expected Class constant, found: {:?}", constant))
  }
}

pub fn class_access_flags(flags: u16) -> String {
  let mut result = String::new();
  if flags & 0x0001 != 0 {
    result.push_str("ACC_PUBLIC ");
  }
  if flags & 0x0010 != 0 {
    result.push_str("ACC_FINAL ");
  }
  if flags & 0x0020 != 0 {
    result.push_str("ACC_SUPER ");
  }
  if flags & 0x0200 != 0 {
    result.push_str("ACC_INTERFACE ");
  }
  if flags & 0x0400 != 0 {
    result.push_str("ACC_ABSTRACT ");
  }
  if flags & 0x1000 != 0 {
    result.push_str("ACC_SYNTHETIC ");
  }
  if flags & 0x2000 != 0 {
    result.push_str("ACC_ANNOTATION ");
  }
  if flags & 0x4000 != 0 {
    result.push_str("ACC_ENUM ");
  }
  if flags & 0x8000 != 0 {
    result.push_str("ACC_MODULE ");
  }
  if result.is_empty() {
    result.push_str("No access flags");
  }
  result
}

pub fn field_access_flags(flags: u16) -> String {
  let mut result = String::new();
  if flags & 0x0001 != 0 {
    result.push_str("ACC_PUBLIC ");
  }
  if flags & 0x0002 != 0 {
    result.push_str("ACC_PRIVATE ");
  }
  if flags & 0x0004 != 0 {
    result.push_str("ACC_PROTECTED ");
  }
  if flags & 0x0008 != 0 {
    result.push_str("ACC_STATIC ");
  }
  if flags & 0x0010 != 0 {
    result.push_str("ACC_FINAL ");
  }
  if flags & 0x0040 != 0 {
    result.push_str("ACC_VOLATILE ");
  }
  if flags & 0x0080 != 0 {
    result.push_str("ACC_TRANSIENT ");
  }
  if flags & 0x1000 != 0 {
    result.push_str("ACC_SYNTHETIC ");
  }
  if flags & 0x4000 != 0 {
    result.push_str("ACC_ENUM ");
  }
  
  if result.is_empty() {
    result.push_str("No access flags");
  }
  
  result
}

pub fn method_access_flags(flags: u16) -> String {
  let mut result = String::new();
  if flags & 0x0001 != 0 {
    result.push_str("ACC_PUBLIC ");
  }
  if flags & 0x0002 != 0 {
    result.push_str("ACC_PRIVATE ");
  }
  if flags & 0x0004 != 0 {
    result.push_str("ACC_PROTECTED ");
  }
  if flags & 0x0008 != 0 {
    result.push_str("ACC_STATIC ");
  }
  if flags & 0x0010 != 0 {
    result.push_str("ACC_FINAL ");
  }
  if flags & 0x0020 != 0 {
    result.push_str("ACC_SYNCHRONIZED ");
  }
  if flags & 0x0040 != 0 {
    result.push_str("ACC_BRIDGE ");
  }
  if flags & 0x0080 != 0 {
    result.push_str("ACC_VARARGS ");
  }
  if flags & 0x0100 != 0 {
    result.push_str("ACC_NATIVE ");
  }
  if flags & 0x0400 != 0 {
    result.push_str("ACC_ABSTRACT ");
  }
  if flags & 0x0800 != 0 {
    result.push_str("ACC_STRICT ");
  }
  if flags & 0x1000 != 0 {
    result.push_str("ACC_SYNTHETIC ");
  }
  
  if result.is_empty() {
    result.push_str("No access flags");
  }
  
  result
}

pub fn inner_class_access_flags(flags: u16) -> String {
  let mut result = String::new();
  if flags & 0x0001 != 0 {
    result.push_str("ACC_PUBLIC ");
  }
  if flags & 0x0002 != 0 {
    result.push_str("ACC_PRIVATE ");
  }
  if flags & 0x0004 != 0 {
    result.push_str("ACC_PROTECTED ");
  }
  if flags & 0x0008 != 0 {
    result.push_str("ACC_STATIC ");
  }
  if flags & 0x0010 != 0 {
    result.push_str("ACC_FINAL ");
  }
  if flags & 0x0020 != 0 {
    result.push_str("ACC_INTERFACE ");
  }
  if flags & 0x0200 != 0 {
    result.push_str("ACC_ABSTRACT ");
  }
  if flags & 0x1000 != 0 {
    result.push_str("ACC_SYNTHETIC ");
  }
  if flags & 0x2000 != 0 {
    result.push_str("ACC_ANNOTATION ");
  }
  if flags & 0x4000 != 0 {
    result.push_str("ACC_ENUM ");
  }
  
  if result.is_empty() {
    result.push_str("No access flags");
  }
  
  result
}