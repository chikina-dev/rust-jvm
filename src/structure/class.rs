#![allow(dead_code)]

use std::{ mem::discriminant };

use nom::{ bytes::complete::take, error::ErrorKind, multi::count, number::complete::{ be_u16, be_u32, be_u8 }, Err, IResult, Parser};

use crate::{structure::code::{CodeByte, CODE_BYTES}, util::{class::{method_access_flags, parse_constant_pool}, hex::{hex_utf8, hex_viewer}}};

#[derive(Debug, Default)]
pub struct Header {
  pub magic: u32,
  pub minor: u16,
  pub major: u16,
}

#[derive(Debug, Default, Clone)]
pub struct ConstantPool {
  pub count: u16,
  pub constants: Vec<Constant>,
}

impl ConstantPool {
  pub fn check_class_index(&self, index: u16) -> Result<&Constant, String> {
    let constant = self.get_class(index);
    match constant {
      Ok(c) => {
        let index = discriminant(&Constant::Class { name_index: 0 });
        if index == discriminant(c) {
          Ok(&c)
        } else {
          Err(format!("Expected Class constant, found: {:?}", c))
        }
      },
      Err(e) => Err(e),
    }
  }
  pub fn get_class(&self, index: u16) -> Result<&Constant, String> {
    if index == 0 || index >= self.count {
      return Err("InvalidIndex".to_string());
    }
    self.constants.get(index as usize - 1)
      .ok_or("NotFound".to_string())
  }
}

#[derive(Debug, Clone)]
pub enum Constant {
  Class { name_index: u16 },
  Fieldref { class_index: u16, name_and_type_index: u16 },
  Methodref { class_index: u16, name_and_type_index: u16 },
  InterfaceMethodref { class_index: u16, name_and_type_index: u16 },
  String { string_index: u16 },
  Integer { bytes: u32 },
  Float { bytes: u32 },
  Long { high_bytes: u32, low_bytes: u32 },
  Double { high_bytes: u32, low_bytes: u32 },
  NameAndType { name_index: u16, descriptor_index: u16 },
  Utf8 { length: u16, bytes: Vec<u8> },
  MethodHandle { reference_kind: u8, reference_index: u16 },
  MethodType { descriptor_index: u16 },
  Dynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 },
  InvokeDynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 },
  Module { name_index: u16 },
  Package { name_index: u16 },
  Unknown,
}

#[derive(Debug, Default)]
pub struct Interfaces {
  pub interfaces_count: u16,
  pub interfaces: Vec<u16>,
}

#[derive(Debug, Default)]
pub struct Fields {
  pub fields_count: u16,
  pub fields: Vec<Field>,
}

#[derive(Debug, Default)]
pub struct Field {
  pub access_flags: u16,
  pub name_index: u16,
  pub descriptor_index: u16,
  pub attributes: FieldInfoAttributes,
}

#[derive(Debug, Default)]
pub struct Methods {
  pub methods_count: u16,
  pub methods: Vec<Method>,
}

#[derive(Debug, Default)]
pub struct Method {
  pub access_flags: u16,
  pub name_index: u16,
  pub descriptor_index: u16,
  pub attributes: MethodInfoAttributes,
}

#[derive(Debug, Default)]
pub struct Attribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub info: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct ClassFileAttributes {
  pub attributes_count: u16,
  pub attributes: Vec<ClassFileAttribute>,
}
#[derive(Debug)]
pub enum ClassFileAttribute {
  SourceFile(SourceFileAttribute),
  SourceDebugExtension(SourceDebugExtensionAttribute),
  LineNumberTable(LineNumberTableAttribute),
  InnerClasses(InnerClassesAttribute),
  EnclosingMethod(EnclosingMethodAttribute),
  BootstrapMethods(BootstrapMethodsAttribute),
  Module(ModuleAttribute),
  ModulePackages(ModulePackagesAttribute),
  ModuleMainClass(ModuleMainClassAttribute),
  NestHost(NestHostAttribute),
  NestMembers(NestMembersAttribute),
  Record(RecordAttribute),
  PermittedSubclasses(PermittedSubclassesAttribute),
  RuntimeVisibleAnnotations(RuntimeVisibleAnnotationsAttribute),
  RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotationsAttribute),
  RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotationsAttribute),
  RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotationsAttribute),
  Synthetic(SyntheticAttribute),
  Deprecated(DeprecatedAttribute),
  Signature(SignatureAttribute),
}

impl ClassFileAttribute {
  pub fn parse<'a>(input: &'a [u8], name: &str, index: u16, constant_pool: &ConstantPool) -> IResult<&'a [u8], Self> {
    match name {
      "SourceFile" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, source_file_index) = be_u16(input)?;
        Ok((input, Self::SourceFile(SourceFileAttribute {
          attribute_name_index: index,
          attribute_length,
          source_file_index,
        })))
      },
      "SourceDebugExtension" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, info) = take(attribute_length as usize)(input)?;
        Ok((input, Self::SourceDebugExtension(SourceDebugExtensionAttribute {
          attribute_name_index: index,
          attribute_length,
          debug_extension: info.to_vec(),
        })))
      },
      "LineNumberTable" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, line_number_table_length) = be_u16(input)?;
        let (input, line_number_table) = count(LineNumberTableEntry::parse, line_number_table_length as usize).parse(input)?;
        Ok((input, Self::LineNumberTable(LineNumberTableAttribute {
          attribute_name_index: index,
          attribute_length,
          line_number_table_length,
          line_number_table,
        })))
      },
      "InnerClasses" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, number_of_classes) = be_u16(input)?;
        fn parse_class_info(input: &[u8]) -> IResult<&[u8], ClassesInfo> {
          let (input, inner_class_info_index) = be_u16(input)?;
          let (input, outer_class_info_index) = be_u16(input)?;
          let (input, inner_name_index) = be_u16(input)?;
          let (input, inner_class_access_flags) = be_u16(input)?;
          Ok((input, ClassesInfo {
            inner_class_info_index,
            outer_class_info_index,
            inner_name_index,
            inner_class_access_flags,
          }))
        }
        let (input, classes) = count(parse_class_info, number_of_classes as usize).parse(input)?;
        Ok((input, Self::InnerClasses(InnerClassesAttribute {
          attribute_name_index: index,
          attribute_length,
          number_of_classes,
          classes,
        })))
      },
      "EnclosingMethod" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, class_index) = be_u16(input)?;
        let (input, method_index) = be_u16(input)?;
        Ok((input, Self::EnclosingMethod(EnclosingMethodAttribute {
          attribute_name_index: index,
          attribute_length,
          class_index,
          method_index,
        })))
      },
      "BootstrapMethods" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_bootstrap_methods) = be_u16(input)?;
        fn parse_bootstrap_method(input: &[u8]) -> IResult<&[u8], BootstrapMethod> {
          let (input, bootstrap_method_attr_index) = be_u16(input)?;
          let (input, num_bootstrap_arguments) = be_u16(input)?;
          let (input, bootstrap_arguments) = count(be_u16, num_bootstrap_arguments as usize).parse(input)?;
          Ok((input, BootstrapMethod {
            bootstrap_method_attr_index,
            num_bootstrap_arguments,
            bootstrap_arguments,
          }))
        }
        let (input, bootstrap_methods) = count(parse_bootstrap_method, num_bootstrap_methods as usize).parse(input)?;
        Ok((input, Self::BootstrapMethods(BootstrapMethodsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_bootstrap_methods,
          bootstrap_methods,
        })))
      },
      "Module" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, module_name_index) = be_u16(input)?;
        let (input, module_flags) = be_u16(input)?;
        let (input, module_version_index) = be_u16(input)?;
        let (input, requires_count) = be_u16(input)?;
        fn parse_requires(input: &[u8]) -> IResult<&[u8], ModuleRequires> {
          let (input, requires_index) = be_u16(input)?;
          let (input, requires_flags) = be_u16(input)?;
          let (input, requires_version_index) = be_u16(input)?;
          Ok((input, ModuleRequires {
            requires_index,
            requires_flags,
            requires_version_index,
          }))
        }
        let (input, requires) = count(parse_requires, requires_count as usize).parse(input)?;
        let (input, exports_count) = be_u16(input)?;
        fn parse_exports(input: &[u8]) -> IResult<&[u8], ModuleExports> {
          let (input, exports_index) = be_u16(input)?;
          let (input, exports_flags) = be_u16(input)?;
          let (input, exports_to_count) = be_u16(input)?;
          let (input, exports_to) = count(be_u16, exports_to_count as usize).parse(input)?;
          Ok((input, ModuleExports {
            exports_index,
            exports_flags,
            exports_to_count,
            exports_to,
          }))
        }
        let (input, exports) = count(parse_exports, exports_count as usize).parse(input)?;
        let (input, opens_count) = be_u16(input)?;
        fn parse_opens(input: &[u8]) -> IResult<&[u8], ModuleOpens> {
          let (input, opens_index) = be_u16(input)?;
          let (input, opens_flags) = be_u16(input)?;
          let (input, opens_to_count) = be_u16(input)?;
          let (input, opens_to) = count(be_u16, opens_to_count as usize).parse(input)?;
          Ok((input, ModuleOpens {
            opens_index,
            opens_flags,
            opens_to_count,
            opens_to,
          }))
        }
        let (input, opens) = count(parse_opens, opens_count as usize).parse(input)?;
        let (input, uses_count) = be_u16(input)?;
        let (input, uses) = count(be_u16, uses_count as usize).parse(input)?;
        let (input, provides_count) = be_u16(input)?;
        fn parse_provides(input: &[u8]) -> IResult<&[u8], ModuleProvides> {
          let (input, provides_index) = be_u16(input)?;
          let (input, provides_with_count) = be_u16(input)?;
          let (input, provides_with) = count(be_u16, provides_with_count as usize).parse(input)?;
          Ok((input, ModuleProvides {
            provides_index,
            provides_with_count,
            provides_with,
          }))
        }
        let (input, provides) = count(parse_provides, provides_count as usize).parse(input)?;
        Ok((input, Self::Module(ModuleAttribute {
          attribute_name_index: index,
          attribute_length,
          module_name_index,
          module_flags,
          module_version_index,
          requires_count,
          requires,
          exports_count,
          exports,
          opens_count,
          opens,
          uses_count,
          uses,
          provides_count,
          provides,
        })))
      },
      "ModulePackages" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, packages_count) = be_u16(input)?;
        let (input, packages) = count(be_u16, packages_count as usize).parse(input)?;
        Ok((input, Self::ModulePackages(ModulePackagesAttribute {
          attribute_name_index: index,
          attribute_length,
          packages_count,
          packages,
        })))
      },
      "ModuleMainClass" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, main_class_index) = be_u16(input)?;
        Ok((input, Self::ModuleMainClass(ModuleMainClassAttribute {
          attribute_name_index: index,
          attribute_length,
          main_class_index,
        })))
      },
      "NestHost" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, nest_host_index) = be_u16(input)?;
        Ok((input, Self::NestHost(NestHostAttribute {
          attribute_name_index: index,
          attribute_length,
          nest_host_index,
        })))
      },
      "NestMembers" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, number_of_classes) = be_u16(input)?;
        let (input, classes) = count(be_u16, number_of_classes as usize).parse(input)?;
        Ok((input, Self::NestMembers(NestMembersAttribute {
          attribute_name_index: index,
          attribute_length,
          number_of_classes,
          classes,
        })))
      },
      "Record" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, record_components_count) = be_u16(input)?;

        let parse_record_component = |input_inner: &'a [u8]| -> IResult<&'a [u8], RecordComponentInfo> { // ここで 'a を明示
          let (input_inner, name_index) = be_u16(input_inner)?;
          let (input_inner, descriptor_index) = be_u16(input_inner)?;
          let (input_inner, attributes_count) = be_u16(input_inner)?;

          let parse_record_component_info_attribute = |input_deep: &'a [u8]| -> IResult<&'a [u8], RecordComponentInfoAttribute> { // ここで 'a を明示
            let (input_deep, idx) = be_u16(input_deep)?;
            let name = match constant_pool.get_class(idx) {
              Ok(Constant::Utf8 { bytes, .. }) => hex_utf8(bytes),
              _ => return Err(nom::Err::Error(nom::error::Error::new(input_deep, ErrorKind::Tag))),
            };
            let (input_deep, attribute) = RecordComponentInfoAttribute::parse(input_deep, &name, idx)?;
            Ok((input_deep, attribute))
          };

          let (input_inner, attributes) = count(parse_record_component_info_attribute, attributes_count as usize).parse(input_inner)?;
          let attributes = RecordComponentInfoAttributes {
            attributes_count,
            attributes,
          };
          Ok((input_inner, RecordComponentInfo {
            name_index,
            descriptor_index,
            attributes,
          }))
        };

        let (input, record_components) = count(parse_record_component, record_components_count as usize).parse(input)?;
        Ok((input, Self::Record(RecordAttribute {
          attribute_name_index: index,
          attribute_length,
          record_components_count,
          record_components,
        })))
      }
      "PermittedSubclasses" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, number_of_classes) = be_u16(input)?;
        let (input, classes) = count(be_u16, number_of_classes as usize).parse(input)?;
        Ok((input, Self::PermittedSubclasses(PermittedSubclassesAttribute {
          attribute_name_index: index,
          attribute_length,
          number_of_classes,
          classes,
        })))
      },
      "RuntimeVisibleAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(Annotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeVisibleAnnotations(RuntimeVisibleAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeInvisibleAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(Annotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeVisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(TypeAnnotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeInvisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(TypeAnnotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "Synthetic" => {
        let (input, attribute_length) = be_u32(input)?;
        if attribute_length != 0 {
          return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::LengthValue)));
        }
        Ok((input, Self::Synthetic(SyntheticAttribute {
          attribute_name_index: index,
          attribute_length,
        })))
      },
      "Deprecated" => {
        let (input, attribute_length) = be_u32(input)?;
        if attribute_length != 0 {
          return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::LengthValue)));
        }
        Ok((input, Self::Deprecated(DeprecatedAttribute {
          attribute_name_index: index,
          attribute_length,
        })))
      },
      "Signature" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, signature_index) = be_u16(input)?;
        Ok((input, Self::Signature(SignatureAttribute {
          attribute_name_index: index,
          attribute_length,
          signature_index,
        })))
      },
      _ => Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))),
    }
  }
}

#[derive(Debug, Default)]
pub struct FieldInfoAttributes {
  pub attributes_count: u16,
  pub attributes: Vec<FieldInfoAttribute>,
}

#[derive(Debug)]
pub enum FieldInfoAttribute {
  ConstantValue(ConstantValueAttribute),
  Synthetic(SyntheticAttribute),
  Deprecated(DeprecatedAttribute),
  Signature(SignatureAttribute),
  RuntimeVisibleAnnotations(RuntimeVisibleAnnotationsAttribute),
  RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotationsAttribute),
  RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotationsAttribute),
  RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotationsAttribute),
}

impl FieldInfoAttribute {
  pub fn parse<'a>(input: &'a [u8], name: &str, index: u16) -> IResult<&'a [u8], Self> {
    match name {
      "ConstantValue" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, constant_value_index) = be_u16(input)?;
        Ok((input, Self::ConstantValue(ConstantValueAttribute {
          attribute_name_index: index,
          attribute_length,
          constant_value_index,
        })))
      },
      "Synthetic" => {
        let (input, attribute_length) = be_u32(input)?;
        if attribute_length != 0 {
          return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::LengthValue)));
        }
        Ok((input, Self::Synthetic(SyntheticAttribute {
          attribute_name_index: index,
          attribute_length,
        })))
      },
      "Deprecated" => {
        let (input, attribute_length) = be_u32(input)?;
        if attribute_length != 0 {
          return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::LengthValue)));
        }
        Ok((input, Self::Deprecated(DeprecatedAttribute {
          attribute_name_index: index,
          attribute_length,
        })))
      },
      "Signature" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, signature_index) = be_u16(input)?;
        Ok((input, Self::Signature(SignatureAttribute {
          attribute_name_index: index,
          attribute_length,
          signature_index,
        })))
      },
      "RuntimeVisibleAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(Annotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeVisibleAnnotations(RuntimeVisibleAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeInvisibleAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(Annotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeVisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(TypeAnnotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeInvisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(TypeAnnotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      _ => Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))),
    }
  }
}

#[derive(Debug, Default)]
pub struct MethodInfoAttributes {
  pub attributes_count: u16,
  pub attributes: Vec<MethodInfoAttribute>,
}

#[derive(Debug)]
pub enum MethodInfoAttribute {
  Code(CodeAttribute),
  Exceptions(ExceptionsAttribute),
  AnnotationDefault(AnnotationDefaultAttribute),
  MethodParameters(MethodParametersAttribute),
  Synthetic(SyntheticAttribute),
  Deprecated(DeprecatedAttribute),
  Signature(SignatureAttribute),
  RuntimeVisibleAnnotations(RuntimeVisibleAnnotationsAttribute),
  RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotationsAttribute),
  RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotationsAttribute),
  RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotationsAttribute),
  RuntimeVisibleParameterAnnotations(RuntimeVisibleParameterAnnotationsAttribute),
  RuntimeInvisibleParameterAnnotations(RuntimeInvisibleParameterAnnotationsAttribute),
}

impl MethodInfoAttribute {
  pub fn parse<'a>(input: &'a [u8], name: &str, index: u16, constant_pool: &ConstantPool) -> IResult<&'a [u8], Self> {
    match name {
      "Code" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, max_stack) = be_u16(input)?;
        let (input, max_locals) = be_u16(input)?;
        let (input, code_length) = be_u32(input)?;
        let mut code: Vec<CodeByte> = Vec::new();
        let mut code_length_buffer = code_length as usize;
        let mut input = input;

        while code_length_buffer > 0 {
            let (input_inner, byte) = be_u8(input)?;
            code_length_buffer -= 1;
            input = input_inner;

            let code_byte: CodeByte = CODE_BYTES.get(&byte).cloned().unwrap_or(
                CodeByte {
                    name: "Unknown",
                    opcode: byte,
                    length: 1,
                    stack_behavior: "Unknown bytecode",
                    data: Vec::new(),
                }
            );
            if code_byte.length == 1 {
                code.push(code_byte);
            } else {
                let mut data = Vec::new();
                for _ in 1..code_byte.length {
                    let (input_inner, next_byte) = be_u8(input)?;
                    code_length_buffer -= 1;
                    input = input_inner;
                    data.push(next_byte);
                }
                let mut full_code_byte = code_byte.clone();
                full_code_byte.data = data;
                code.push(full_code_byte);
            }
        }
        let (input, exception_table_length) = be_u16(input)?;
        fn exception_entry(input: &[u8]) -> IResult<&[u8], ExceptionTableEntry> {
          let (input, start_pc) = be_u16(input)?;
          let (input, end_pc) = be_u16(input)?;
          let (input, handler_pc) = be_u16(input)?;
          let (input, catch_type) = be_u16(input)?;
          Ok((input, ExceptionTableEntry { start_pc, end_pc, handler_pc, catch_type }))
        }
        let (input, exception_table) = count(exception_entry, exception_table_length as usize).parse(input)?;
        let (input, attributes_count) = be_u16(input)?;
        let parse_code_attributes = |input_inner: &'a [u8]| -> IResult<&'a [u8], CodeAttributes> {
          let parse_code_nested_attribute = |input_deep: &'a [u8]| -> IResult<&'a [u8], CodeNestedAttribute> {
            let (input_deep, idx) = be_u16(input_deep)?;
            let name = match constant_pool.get_class(idx) {
              Ok(Constant::Utf8 { bytes, .. }) => hex_utf8(bytes),
              _ => return Err(nom::Err::Error(nom::error::Error::new(input_deep, ErrorKind::Tag))),
            };
            let (input_deep, attribute) = CodeNestedAttribute::parse(input_deep, &name, idx)?;
            Ok((input_deep, attribute))
          };
          let (input_inner, attributes) = count(parse_code_nested_attribute, attributes_count as usize).parse(input_inner)?;
          Ok((input_inner, CodeAttributes {
            attributes_count,
            attributes,
          }))
        };
        let (input, attributes) = parse_code_attributes(input)?;
        Ok((input, Self::Code(CodeAttribute {
          attribute_name_index: index,
          attribute_length,
          max_stack,
          max_locals,
          code_length,
          code,
          exception_table_length,
          exception_table,
          attributes_count,
          attributes,
        })))
      },
      "Exceptions" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, number_of_exceptions) = be_u16(input)?;
        let (input, exception_index_table) = count(be_u16, number_of_exceptions as usize).parse(input)?;
        Ok((input, Self::Exceptions(ExceptionsAttribute {
          attribute_name_index: index,
          attribute_length,
          number_of_exceptions,
          exception_index_table,
        })))
      },
      "AnnotationDefault" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, default_value) = ElementValue::parse(input)?;
        Ok((input, Self::AnnotationDefault(AnnotationDefaultAttribute {
          attribute_name_index: index,
          attribute_length,
          default_value,
        })))
      },
      "MethodParameters" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, parameters_count) = be_u8(input)?;
        fn parameter(input: &[u8]) -> IResult<&[u8], MethodParameter> {
          let (input, name_index) = be_u16(input)?;
          let (input, access_flags) = be_u16(input)?;
          Ok((input, MethodParameter { name_index, access_flags }))
        }
        let (input, parameters) = count(parameter, parameters_count as usize).parse(input)?;
        Ok((input, Self::MethodParameters(MethodParametersAttribute {
          attribute_name_index: index,
          attribute_length,
          parameters_count,
          parameters,
        })))
      },
      "Synthetic" => {
        let (input, attribute_length) = be_u32(input)?;
        if attribute_length != 0 {
          return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::LengthValue)));
        }
        Ok((input, Self::Synthetic(SyntheticAttribute {
          attribute_name_index: index,
          attribute_length,
        })))
      },
      "Deprecated" => {
        let (input, attribute_length) = be_u32(input)?;
        if attribute_length != 0 {
          return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::LengthValue)));
        }
        Ok((input, Self::Deprecated(DeprecatedAttribute {
          attribute_name_index: index,
          attribute_length,
        })))
      },
      "Signature" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, signature_index) = be_u16(input)?;
        Ok((input, Self::Signature(SignatureAttribute {
          attribute_name_index: index,
          attribute_length,
          signature_index,
        })))
      },
      "RuntimeVisibleAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(Annotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeVisibleAnnotations(RuntimeVisibleAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeInvisibleAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(Annotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeVisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(TypeAnnotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeInvisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        let (input, annotations) = count(TypeAnnotation::parse, num_annotations as usize).parse(input)?;
        Ok((input, Self::RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        })))
      },
      "RuntimeVisibleParameterAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_parameters) = be_u8(input)?;
        fn parameter_annotations(input: &[u8]) -> IResult<&[u8], ParameterAnnotation> {
          let (input, num_annotations) = be_u16(input)?;
          let (input, annotations) = count(Annotation::parse, num_annotations as usize).parse(input)?;
          Ok((input, ParameterAnnotation { num_annotations, annotations }))
        }
        let (input, parameter_annotations) = count(parameter_annotations, num_parameters as usize).parse(input)?;
        Ok((input, Self::RuntimeVisibleParameterAnnotations(RuntimeVisibleParameterAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_parameters,
          parameter_annotations,
        })))
      },
      "RuntimeInvisibleParameterAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_parameters) = be_u8(input)?;
        fn parameter_annotations(input: &[u8]) -> IResult<&[u8], ParameterAnnotation> {
          let (input, num_annotations) = be_u16(input)?;
          let (input, annotations) = count(Annotation::parse, num_annotations as usize).parse(input)?;
          Ok((input, ParameterAnnotation { num_annotations, annotations }))
        }
        let (input, parameter_annotations) = count(parameter_annotations, num_parameters as usize).parse(input)?;
        Ok((input, Self::RuntimeInvisibleParameterAnnotations(RuntimeInvisibleParameterAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_parameters,
          parameter_annotations,
        })))
      },
      _ => Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))),
    }
  }
}

#[derive(Debug, Default)]
pub struct CodeAttributes {
  attributes_count: u16,
  attributes: Vec<CodeNestedAttribute>,
}

#[derive(Debug)]
pub enum CodeNestedAttribute {
  LineNumberTable(LineNumberTableAttribute),
  LocalVariableTable(LocalVariableTableAttribute),
  LocalVariableTypeTable(LocalVariableTypeTableAttribute),
  StackMapTable(StackMapTableAttribute),
  RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotationsAttribute),
  RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotationsAttribute),
}

impl CodeNestedAttribute {
  pub fn parse<'a>(input: &'a [u8], name: &str, index: u16) -> IResult<&'a [u8], Self> {
    match name {
      "LineNumberTable" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, line_number_table_length) = be_u16(input)?;
        fn parse_line_number_table(input: &[u8], line_number_table_length: u16) -> IResult<&[u8], Vec<LineNumberTableEntry>> {
          let mut entries = Vec::new();
          let mut remaining_input = input;
          for _ in 0..line_number_table_length {
            let (input, start_pc) = be_u16(remaining_input)?;
            let (input, line_number) = be_u16(input)?;
            entries.push(LineNumberTableEntry { start_pc, line_number });
            remaining_input = input;
          }
          Ok((remaining_input, entries))
        }
        let (input, line_number_table) = parse_line_number_table(input, line_number_table_length)?;
        let parsed = LineNumberTableAttribute {
          attribute_name_index: index,
          attribute_length,
          line_number_table_length,
          line_number_table,
        };
        Ok((input, Self::LineNumberTable(parsed)))
      },
      "LocalVariableTable" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, local_variable_table_length) = be_u16(input)?;
        fn parse_local_variable_table(input: &[u8], local_variable_table_length: u16) -> IResult<&[u8], Vec<LocalVariableTableEntry>> {
          let mut entries = Vec::new();
          let mut remaining_input = input;
          for _ in 0..local_variable_table_length {
            let (input, start_pc) = be_u16(remaining_input)?;
            let (input, length) = be_u16(input)?;
            let (input, name_index) = be_u16(input)?;
            let (input, descriptor_index) = be_u16(input)?;
            let (input, index) = be_u16(input)?;
            entries.push(LocalVariableTableEntry { start_pc, length, name_index, descriptor_index, index });
            remaining_input = input;
          }
          Ok((remaining_input, entries))
        }
        let (input, local_variable_table) = parse_local_variable_table(input, local_variable_table_length)?;
        let parsed = LocalVariableTableAttribute {
          attribute_name_index: index,
          attribute_length,
          local_variable_table_length,
          local_variable_table,
        };
        Ok((input, Self::LocalVariableTable(parsed)))
      },
      "LocalVariableTypeTable" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, local_variable_type_table_length) = be_u16(input)?;
        fn parse_local_variable_type_table(input: &[u8], local_variable_type_table_length: u16) -> IResult<&[u8], Vec<LocalVariableTypeTableEntry>> {
          let mut entries = Vec::new();
          let mut remaining_input = input;
          for _ in 0..local_variable_type_table_length {
            let (input, start_pc) = be_u16(remaining_input)?;
            let (input, length) = be_u16(input)?;
            let (input, name_index) = be_u16(input)?;
            let (input, signature_index) = be_u16(input)?;
            let (input, index) = be_u16(input)?;
            entries.push(LocalVariableTypeTableEntry { start_pc, length, name_index, signature_index, index });
            remaining_input = input;
          }
          Ok((remaining_input, entries))
        }
        let (input, local_variable_type_table) = parse_local_variable_type_table(input, local_variable_type_table_length)?;
        let parsed = LocalVariableTypeTableAttribute {
          attribute_name_index: index,
          attribute_length,
          local_variable_type_table_length,
          local_variable_type_table,
        };
        Ok((input, Self::LocalVariableTypeTable(parsed)))
      },
      "StackMapTable" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, number_of_entries) = be_u16(input)?;
        fn parse_stack_map_frame(input: &[u8], number_of_entries: u16) -> IResult<&[u8], Vec<StackMapFrame>> {
          let mut entries = Vec::new();
          let mut remaining_input = input;
          for _ in 0..number_of_entries {
            let (input, frame) = StackMapFrame::parse(remaining_input)?;
            entries.push(frame);
            remaining_input = input;
          }
          Ok((remaining_input, entries))
        }
        let (input, entries) = parse_stack_map_frame(input, number_of_entries)?;
        let parsed = StackMapTableAttribute {
          attribute_name_index: index,
          attribute_length,
          number_of_entries,
          entries,
        };
        Ok((input, Self::StackMapTable(parsed)))
      },
      "RuntimeVisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        fn parse_runtime_visible_type_annotations(input: &[u8], num_annotations: u16) -> IResult<&[u8], Vec<TypeAnnotation>> {
          let mut annotations = Vec::new();
          let mut remaining_input = input;
          for _ in 0..num_annotations {
            let (input, annotation) = TypeAnnotation::parse(remaining_input)?;
            annotations.push(annotation);
            remaining_input = input;
          }
          Ok((remaining_input, annotations))
        }
        let (input, annotations) = parse_runtime_visible_type_annotations(input, num_annotations)?;
        let parsed = RuntimeVisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        };
        Ok((input, Self::RuntimeVisibleTypeAnnotations(parsed)))
      },
      "RuntimeInvisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        fn parse_runtime_invisible_type_annotations(input: &[u8], num_annotations: u16) -> IResult<&[u8], Vec<TypeAnnotation>> {
          let mut annotations = Vec::new();
          let mut remaining_input = input;
          for _ in 0..num_annotations {
            let (input, annotation) = TypeAnnotation::parse(remaining_input)?;
            annotations.push(annotation);
            remaining_input = input;
          }
          Ok((remaining_input, annotations))
        }
        let (input, annotations) = parse_runtime_invisible_type_annotations(input, num_annotations)?;
        let parsed = RuntimeInvisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        };
        Ok((input, Self::RuntimeInvisibleTypeAnnotations(parsed)))
      },
      _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
    }
  }
}


#[derive(Debug, Default)]
pub struct RecordComponentInfoAttributes {
  pub attributes_count: u16,
  pub attributes: Vec<RecordComponentInfoAttribute>,
}

#[derive(Debug)]
pub enum RecordComponentInfoAttribute {
  Signature(SignatureAttribute),
  RuntimeVisibleAnnotations(RuntimeVisibleAnnotationsAttribute),
  RuntimeInvisibleAnnotations(RuntimeInvisibleAnnotationsAttribute),
  RuntimeVisibleTypeAnnotations(RuntimeVisibleTypeAnnotationsAttribute),
  RuntimeInvisibleTypeAnnotations(RuntimeInvisibleTypeAnnotationsAttribute),
}

impl RecordComponentInfoAttribute {
  pub fn parse<'a>(input: &'a [u8], name: &str, index: u16) -> IResult<&'a [u8], Self> {
    match name {
      "Signature" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, signature_index) = be_u16(input)?;
        let parsed = SignatureAttribute {
          attribute_name_index: index,
          attribute_length,
          signature_index,
        };
        Ok((input, Self::Signature(parsed)))
      },
      "RuntimeVisibleAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        fn parse_runtime_visible_annotations(input: &[u8], num_annotations: u16) -> IResult<&[u8], Vec<Annotation>> {
          let mut annotations = Vec::new();
          let mut remaining_input = input;
          for _ in 0..num_annotations {
            let (input, annotation) = Annotation::parse(remaining_input)?;
            annotations.push(annotation);
            remaining_input = input;
          }
          Ok((remaining_input, annotations))
        }
        let (input, annotations) = parse_runtime_visible_annotations(input, num_annotations)?;
        let parsed = RuntimeVisibleAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        };
        Ok((input, Self::RuntimeVisibleAnnotations(parsed)))
      },
      "RuntimeInvisibleAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        fn parse_runtime_invisible_annotations(input: &[u8], num_annotations: u16) -> IResult<&[u8], Vec<Annotation>> {
          let mut annotations = Vec::new();
          let mut remaining_input = input;
          for _ in 0..num_annotations {
            let (input, annotation) = Annotation::parse(remaining_input)?;
            annotations.push(annotation);
            remaining_input = input;
          }
          Ok((remaining_input, annotations))
        }
        let (input, annotations) = parse_runtime_invisible_annotations(input, num_annotations)?;
        let parsed = RuntimeInvisibleAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        };
        Ok((input, Self::RuntimeInvisibleAnnotations(parsed)))
      },
      "RuntimeVisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        fn parse_runtime_visible_type_annotations(input: &[u8], num_annotations: u16) -> IResult<&[u8], Vec<TypeAnnotation>> {
          let mut annotations = Vec::new();
          let mut remaining_input = input;
          for _ in 0..num_annotations {
            let (input, annotation) = TypeAnnotation::parse(remaining_input)?;
            annotations.push(annotation);
            remaining_input = input;
          }
          Ok((remaining_input, annotations))
        }
        let (input, annotations) = parse_runtime_visible_type_annotations(input, num_annotations)?;
        let parsed = RuntimeVisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        };
        Ok((input, Self::RuntimeVisibleTypeAnnotations(parsed)))
      },
      "RuntimeInvisibleTypeAnnotations" => {
        let (input, attribute_length) = be_u32(input)?;
        let (input, num_annotations) = be_u16(input)?;
        fn parse_runtime_invisible_type_annotations(input: &[u8], num_annotations: u16) -> IResult<&[u8], Vec<TypeAnnotation>> {
          let mut annotations = Vec::new();
          let mut remaining_input = input;
          for _ in 0..num_annotations {
            let (input, annotation) = TypeAnnotation::parse(remaining_input)?;
            annotations.push(annotation);
            remaining_input = input;
          }
          Ok((remaining_input, annotations))
        }
        let (input, annotations) = parse_runtime_invisible_type_annotations(input, num_annotations)?;
        let parsed = RuntimeInvisibleTypeAnnotationsAttribute {
          attribute_name_index: index,
          attribute_length,
          num_annotations,
          annotations,
        };
        Ok((input, Self::RuntimeInvisibleTypeAnnotations(parsed)))
      },
      _ => Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))),
    }
  }
}

#[derive(Debug, Default)]
pub struct ConstantValueAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub constant_value_index: u16,
}
#[derive(Debug, Default)]
pub struct CodeAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub max_stack: u16,
  pub max_locals: u16,
  pub code_length: u32,
  pub code: Vec<CodeByte>,
  pub exception_table_length: u16,
  pub exception_table: Vec<ExceptionTableEntry>,
  pub attributes_count: u16,
  pub attributes: CodeAttributes,
}
#[derive(Debug, Default)]
pub struct StackMapTableAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub number_of_entries: u16,
  pub entries: Vec<StackMapFrame>,
}
#[derive(Debug, Default)]
pub struct ExceptionsAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub number_of_exceptions: u16,
  pub exception_index_table: Vec<u16>,
}
#[derive(Debug, Default)]
pub struct InnerClassesAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub number_of_classes: u16,
  pub classes: Vec<ClassesInfo>,
}
#[derive(Debug, Default)]
pub struct EnclosingMethodAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub class_index: u16,
  pub method_index: u16,
}
#[derive(Debug, Default)]
pub struct SyntheticAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
}
#[derive(Debug, Default)]
pub struct SignatureAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub signature_index: u16,
}
#[derive(Debug, Default)]
pub struct SourceFileAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub source_file_index: u16,
}
#[derive(Debug, Default)]
pub struct SourceDebugExtensionAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub debug_extension: Vec<u8>,
}
#[derive(Debug, Default)]
pub struct LineNumberTableAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub line_number_table_length: u16,
  pub line_number_table: Vec<LineNumberTableEntry>,
}
#[derive(Debug, Default)]
pub struct LocalVariableTableAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub local_variable_table_length: u16,
  pub local_variable_table: Vec<LocalVariableTableEntry>,
}
#[derive(Debug, Default)]
pub struct LocalVariableTypeTableAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub local_variable_type_table_length: u16,
  pub local_variable_type_table: Vec<LocalVariableTypeTableEntry>,
}
#[derive(Debug, Default)]
pub struct DeprecatedAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
}
#[derive(Debug, Default)]
pub struct RuntimeVisibleAnnotationsAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub num_annotations: u16,
  pub annotations: Vec<Annotation>,
}
#[derive(Debug, Default)]
pub struct RuntimeInvisibleAnnotationsAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub num_annotations: u16,
  pub annotations: Vec<Annotation>,
}
#[derive(Debug, Default)]
pub struct RuntimeVisibleParameterAnnotationsAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub num_parameters: u8,
  pub parameter_annotations: Vec<ParameterAnnotation>,
}
#[derive(Debug, Default)]
pub struct RuntimeInvisibleParameterAnnotationsAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub num_parameters: u8,
  pub parameter_annotations: Vec<ParameterAnnotation>,
}
#[derive(Debug, Default)]
pub struct RuntimeVisibleTypeAnnotationsAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub num_annotations: u16,
  pub annotations: Vec<TypeAnnotation>,
}
#[derive(Debug, Default)]
pub struct RuntimeInvisibleTypeAnnotationsAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub num_annotations: u16,
  pub annotations: Vec<TypeAnnotation>,
}
#[derive(Debug, Default)]
pub struct AnnotationDefaultAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub default_value: ElementValue,
}
#[derive(Debug, Default)]
pub struct BootstrapMethodsAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub num_bootstrap_methods: u16,
  pub bootstrap_methods: Vec<BootstrapMethod>,
}
#[derive(Debug, Default)]
pub struct MethodParametersAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub parameters_count: u8,
  pub parameters: Vec<MethodParameter>,
}
#[derive(Debug, Default)]
pub struct ModuleAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub module_name_index: u16,
  pub module_flags: u16,
  pub module_version_index: u16,
  pub requires_count: u16,
  pub requires: Vec<ModuleRequires>,
  pub exports_count: u16,
  pub exports: Vec<ModuleExports>,
  pub opens_count: u16,
  pub opens: Vec<ModuleOpens>,
  pub uses_count: u16,
  pub uses: Vec<u16>,
  pub provides_count: u16,
  pub provides: Vec<ModuleProvides>,
}
#[derive(Debug, Default)]
pub struct ModuleRequires {
  pub requires_index: u16,
  pub requires_flags: u16,
  pub requires_version_index: u16,
}
#[derive(Debug, Default)]
pub struct ModuleExports {
  pub exports_index: u16,
  pub exports_flags: u16,
  pub exports_to_count: u16,
  pub exports_to: Vec<u16>,
}
#[derive(Debug, Default)]
pub struct ModuleOpens {
  pub opens_index: u16,
  pub opens_flags: u16,
  pub opens_to_count: u16,
  pub opens_to: Vec<u16>,
}
#[derive(Debug, Default)]
pub struct ModuleProvides {
  pub provides_index: u16,
  pub provides_with_count: u16,
  pub provides_with: Vec<u16>,
}
#[derive(Debug, Default)]
pub struct ModulePackagesAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub packages_count: u16,
  pub packages: Vec<u16>,
}
#[derive(Debug, Default)]
pub struct ModuleMainClassAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub main_class_index: u16,
}
#[derive(Debug, Default)]
pub struct NestHostAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub nest_host_index: u16,
}
#[derive(Debug, Default)]
pub struct NestMembersAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub number_of_classes: u16,
  pub classes: Vec<u16>,
}
#[derive(Debug, Default)]
pub struct RecordAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub record_components_count: u16,
  pub record_components: Vec<RecordComponentInfo>,
}
#[derive(Debug, Default)]
pub struct RecordComponentInfo {
  pub name_index: u16,
  pub descriptor_index: u16,
  pub attributes: RecordComponentInfoAttributes,
}
#[derive(Debug, Default)]
pub struct PermittedSubclassesAttribute {
  pub attribute_name_index: u16,
  pub attribute_length: u32,
  pub number_of_classes: u16,
  pub classes: Vec<u16>,
}

#[derive(Debug, Default)]
pub struct ClassesInfo {
  pub inner_class_info_index: u16,
  pub outer_class_info_index: u16,
  pub inner_name_index: u16,
  pub inner_class_access_flags: u16,
}

#[derive(Debug)]
pub enum StackMapFrame {
  SameFrame {
    frame_type: u8,
  },
  SameLocals1StackItemFrame {
    frame_type: u8, // 0-63
    stack: Vec<VerificationTypeInfo>,
  },
  SameLocals1StackItemFrameExtended {
    frame_type: u8, // 64-127
    offset_delta: u16,
    stack: Vec<VerificationTypeInfo>,
  },
  ChopFrame {
    frame_type: u8, // 248-250
    offset_delta: u16,
  },
  SameFrameExtended {
    frame_type: u8, // 251
    offset_delta: u16,
  },
  AppendFrame {
    frame_type: u8, // 252-254
    offset_delta: u16,
    locals: Vec<VerificationTypeInfo>, // frame_type - 251
  },
  FullFrame {
    frame_type: u8, // 255
    offset_delta: u16,
    number_of_locals: u16,
    locals: Vec<VerificationTypeInfo>,
    number_of_stack_items: u16,
    stack: Vec<VerificationTypeInfo>,
  },
}

impl StackMapFrame {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
    let (input, frame_type) = take(1usize)(input)?;
    let frame_type = frame_type[0];

    match frame_type {
      0..=63 => Ok((input, StackMapFrame::SameFrame { frame_type })),
      64..=127 => {
        let (input, stack) = VerificationTypeInfo::parse_vec(input)?;
        Ok((input, StackMapFrame::SameLocals1StackItemFrame { frame_type, stack }))
      },
      128..=247 => {
        let (input, offset_delta) = be_u16(input)?;
        let (input, stack) = VerificationTypeInfo::parse_vec(input)?;
        Ok((input, StackMapFrame::SameLocals1StackItemFrameExtended { frame_type, offset_delta, stack }))
      },
      248..=250 => {
        let (input, offset_delta) = be_u16(input)?;
        Ok((input, StackMapFrame::ChopFrame { frame_type, offset_delta }))
      },
      251 => {
        let (input, offset_delta) = be_u16(input)?;
        Ok((input, StackMapFrame::SameFrameExtended { frame_type, offset_delta }))
      },
      252..=254 => {
        let (input, offset_delta) = be_u16(input)?;
        let locals_count = (frame_type - 251) as usize;
        let (input, locals) = VerificationTypeInfo::parse_vec_with_count(input, locals_count)?;
        Ok((input, StackMapFrame::AppendFrame { frame_type, offset_delta, locals }))
      },
      255 => {
        let (input, offset_delta) = be_u16(input)?;
        let (input, number_of_locals) = be_u16(input)?;
        let (input, locals) = VerificationTypeInfo::parse_vec_with_count(input, number_of_locals as usize)?;
        let (input, number_of_stack_items) = be_u16(input)?;
        let (input, stack) = VerificationTypeInfo::parse_vec_with_count(input, number_of_stack_items as usize)?;
        Ok((input,
          StackMapFrame::FullFrame {
            frame_type,
            offset_delta,
            number_of_locals,
            locals,
            number_of_stack_items,
            stack,
          },
        ))
      },
    }
  }
}

#[derive(Debug)]
pub enum VerificationTypeInfo {
  TopVariableInfo { tag: u8 }, // 0
  IntegerVariableInfo { tag: u8 }, // 1
  FloatVariableInfo { tag: u8 }, // 2
  LongVariableInfo { tag: u8 }, // 3
  DoubleVariableInfo { tag: u8 }, // 4
  NullVariableInfo { tag: u8 }, // 5
  UninitializedThisVariableInfo { tag: u8 }, // 6
  ObjectVariableInfo { tag: u8, cpool_index: u16 }, // 7
  UninitializedVariableInfo { tag: u8, offset: u16 }, // 8
}

impl VerificationTypeInfo {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Vec<Self>> {
    let mut input = input;
    let mut vec = Vec::new();

    while !input.is_empty() {
      let (rest, tag) = be_u8(input)?;
      input = rest;

      let verification_type_info = match tag {
        0 => VerificationTypeInfo::TopVariableInfo { tag },
        1 => VerificationTypeInfo::IntegerVariableInfo { tag },
        2 => VerificationTypeInfo::FloatVariableInfo { tag },
        3 => VerificationTypeInfo::LongVariableInfo { tag },
        4 => VerificationTypeInfo::DoubleVariableInfo { tag },
        5 => VerificationTypeInfo::NullVariableInfo { tag },
        6 => VerificationTypeInfo::UninitializedThisVariableInfo { tag },
        7 => {
          let (rest, cpool_index) = be_u16(input)?;
          input = rest;
          VerificationTypeInfo::ObjectVariableInfo { tag, cpool_index }
        },
        8 => {
          let (rest, offset) = be_u16(input)?;
          input = rest;
          VerificationTypeInfo::UninitializedVariableInfo { tag, offset }
        },
        _ => return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))),
      };

      vec.push(verification_type_info);
    }

    Ok((input, vec))
  }

  pub fn parse_vec(input: &[u8]) -> IResult<&[u8], Vec<Self>> {
    Self::parse(input)
  }

  pub fn parse_vec_with_count(input: &[u8], count: usize) -> IResult<&[u8], Vec<Self>> {
    let mut input = input;
    let mut vec = Vec::with_capacity(count);

    for _ in 0..count {
      let (rest, verification_type_info) = Self::parse(input)?;
      input = rest;
      vec = verification_type_info;
    }

    Ok((input, vec))
  }
}

#[derive(Debug, Default)]
pub struct ExceptionTableEntry {
  pub start_pc: u16,
  pub end_pc: u16,
  pub handler_pc: u16,
  pub catch_type: u16,
}
#[derive(Debug, Default)]
pub struct LineNumberTableEntry {
  pub start_pc: u16,
  pub line_number: u16,
}

impl LineNumberTableEntry {
  pub fn parse(input: &[u8]) -> IResult<&[u8], LineNumberTableEntry> {
    let (input, start_pc) = be_u16(input)?;
    let (input, line_number) = be_u16(input)?;
    Ok((input, LineNumberTableEntry { start_pc, line_number }))
  }
}

#[derive(Debug, Default)]
pub struct LocalVariableTableEntry {
  pub start_pc: u16,
  pub length: u16,
  pub name_index: u16,
  pub descriptor_index: u16,
  pub index: u16,
}

#[derive(Debug, Default)]
pub struct LocalVariableTypeTableEntry {
  pub start_pc: u16,
  pub length: u16,
  pub name_index: u16,
  pub signature_index: u16,
  pub index: u16,
}

#[derive(Debug, Default)]
pub struct Annotation {
  pub type_index: u16,
  pub num_element_value_pairs: u16,
  pub element_value_pairs: Vec<ElementValuePair>,
}

impl Annotation {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
    let (input, type_index) = be_u16(input)?;
    let (input, num_element_value_pairs) = be_u16(input)?;

    let mut element_value_pairs = Vec::with_capacity(num_element_value_pairs as usize);
    let mut remaining_input = input;
    for _ in 0..num_element_value_pairs {
      let (rest, element_value_pair) = ElementValuePair::parse(remaining_input)?;
      remaining_input = rest;
      element_value_pairs.push(element_value_pair);
    }

    Ok((remaining_input, Annotation {
      type_index,
      num_element_value_pairs,
      element_value_pairs,
    }))
  }
    
}

#[derive(Debug, Default)]
pub struct ParameterAnnotation {
  pub num_annotations: u16,
  pub annotations: Vec<Annotation>,
}

#[derive(Debug, Default)]
pub struct ElementValuePair {
  pub element_name_index: u16,
  pub value: ElementValue,
}

impl ElementValuePair {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
    let (input, element_name_index) = be_u16(input)?;
    let (input, value) = ElementValue::parse(input)?;
    Ok((input, ElementValuePair {
      element_name_index,
      value,
    }))
  }
    
}

#[derive(Debug, Default)]
pub struct TypeAnnotation {
  pub target_type: u8,
  pub target_info: TargetInfo,
  pub target_path: TypePath,
  pub type_index: u16,
  pub element_value_pairs: Vec<ElementValuePair>,
}

impl TypeAnnotation {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
    let (input, target_type) = be_u8(input)?;
    let (input, target_info) = TargetInfo::parse(input)?;
    let (input, target_path) = TypePath::parse(input)?;
    let (input, type_index) = be_u16(input)?;
    let (mut input, num_element_value_pairs) = be_u16(input)?;

    let mut element_value_pairs = Vec::with_capacity(num_element_value_pairs as usize);
    for _ in 0..num_element_value_pairs {
      let (rest, element_value_pair) = ElementValuePair::parse(input)?;
      input = rest;
      element_value_pairs.push(element_value_pair);
    }

    Ok((input, TypeAnnotation {
      target_type,
      target_info,
      target_path,
      type_index,
      element_value_pairs,
    }))
  }
}

#[derive(Debug, Default)]
pub struct TypePath {
  pub path_length: u8,
  pub path: Vec<TypePathEntry>,
}

#[derive(Debug, Default)]
pub struct TypePathEntry {
  pub type_path_kind: u8,
  pub type_argument_index: u8,
}

impl TypePath {
  pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
    let (input, path_length) = be_u8(input)?;
    let mut path = Vec::with_capacity(path_length as usize);
    
    let mut remaining_input = input;
    for _ in 0..path_length {
      let (input, type_path_kind) = be_u8(remaining_input)?;
      let (input, type_argument_index) = be_u8(input)?;
      path.push(TypePathEntry { type_path_kind, type_argument_index });
      remaining_input = input;
    }

    Ok((remaining_input, TypePath { path_length, path }))
  }
}

#[derive(Debug)]
pub enum TargetInfo {
  TypeParameter {
    type_parameter_index: u8,
  },
  Supertype {
    supertype_index: u16,
  },
  TypeParameterBound {
    type_parameter_index: u8,
    bound_index: u8,
  },
  Empty {},
  FormalParameter {
    formal_parameter_index: u8,
  },
  Throws{
    throws_type_index: u16,
  },
  Localvar{
    table_length: u16,
    local_var_table: Vec<LocalVarTableEntry>,
  },
  Catch {
    exception_table_index: u16,
  },
  Offset {
    offset: u16,
  },
  TypeArgument {
    offset: u16,
    type_argument_index: u8,
  },
}

impl Default for TargetInfo {
  fn default() -> Self {
    TargetInfo::Empty {}
  }
}

impl TargetInfo {
  fn parse(input: &[u8]) -> IResult<&[u8], Self> {
    let (input, target_type) = be_u8(input)?;

    match target_type {
      0 => {
        let (input, type_parameter_index) = be_u8(input)?;
        Ok((input, TargetInfo::TypeParameter { type_parameter_index }))
      },
      1 => {
        let (input, supertype_index) = be_u16(input)?;
        Ok((input, TargetInfo::Supertype { supertype_index }))
      },
      2 => {
        let (input, type_parameter_index) = be_u8(input)?;
        let (input, bound_index) = be_u8(input)?;
        Ok((input, TargetInfo::TypeParameterBound { type_parameter_index, bound_index }))
      },
      3 => Ok((input, TargetInfo::Empty {})),
      4 => {
        let (input, formal_parameter_index) = be_u8(input)?;
        Ok((input, TargetInfo::FormalParameter { formal_parameter_index }))
      },
      5 => {
        let (input, throws_type_index) = be_u16(input)?;
        Ok((input, TargetInfo::Throws { throws_type_index }))
      },
      6 => {
        let (mut input, table_length) = be_u16(input)?;
        let mut local_var_table = Vec::with_capacity(table_length as usize);

        for _ in 0..table_length {
            let (rest, start_pc) = be_u16(input)?;
            let (rest, length) = be_u16(rest)?;
            let (rest, index) = be_u16(rest)?;
            
            local_var_table.push(LocalVarTableEntry { start_pc, length, index });
            input = rest;
        }

        Ok((input, TargetInfo::Localvar { table_length, local_var_table }))
      },
      7 => {
        let (input, exception_table_index) = be_u16(input)?;
        Ok((input, TargetInfo::Catch { exception_table_index }))
      },
      8 => {
        let (input, offset) = be_u16(input)?;
        Ok((input, TargetInfo::Offset { offset }))
      },
      9 => {
        let (input, offset) = be_u16(input)?;
        let (input, type_argument_index) = be_u8(input)?;
        Ok((input, TargetInfo::TypeArgument { offset, type_argument_index }))
      },
      _ => Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
    }
  }
}

#[derive(Debug, Default)]
pub struct ElementValue {
  pub tag: u8,
  pub value: ElementValueEnum,
}

impl Default for ElementValueEnum {
  fn default() -> Self {
    ElementValueEnum::ConstValueIndex(0)
  }
}

impl ElementValue {
  fn parse(input: &[u8]) -> IResult<&[u8], ElementValue> {
    let (input, tag) = be_u8(input)?;
    let mut value = ElementValue {
      tag,
      value: ElementValueEnum::default(),
    };

    match tag {
      0x42 => { // 'B' for byte
        let (input, const_value_index) = be_u8(input)?;
        value.value = ElementValueEnum::ConstValueIndex(const_value_index);
        Ok((input, value))
      },
      0x45 => { // 'E' for enum
        let (input, type_name_index) = be_u16(input)?;
        let (input, const_name_index) = be_u16(input)?;
        value.value = ElementValueEnum::EnumConstValue {
          type_name_index,
          const_name_index,
        };
        Ok((input, value))
      },
      0x43 => { // 'C' for class
        let (input, class_info_index) = be_u8(input)?;
        value.value = ElementValueEnum::ClassInfoIndex(class_info_index);
        Ok((input, value))
      },
      0x40 => { // '@' for annotation
        let (input, annotation) = Annotation::parse(input)?;
        value.value = ElementValueEnum::AnnotationValue(annotation);
        Ok((input, value))
      },
      0x5B => { // '[' for array
        let (mut input, num_values) = be_u16(input)?;
        let mut values = Vec::with_capacity(num_values as usize);
        for _ in 0..num_values {
          let (rest, element_value) = ElementValue::parse(input)?;
          input = rest;
          values.push(element_value);
        }
        value.value = ElementValueEnum::ArrayValue {
          num_values,
          values,
        };
        Ok((input, value))
      },
      _ => Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))),
    }
  }
}

#[derive(Debug)]
pub enum ElementValueEnum {
  ConstValueIndex(u8),

  EnumConstValue {
    type_name_index: u16,
    const_name_index: u16,
  },

  ClassInfoIndex(u8),

  AnnotationValue(Annotation),

  ArrayValue {
    num_values: u16,
    values: Vec<ElementValue>,
  },
}
#[derive(Debug, Default)]
pub struct LocalVarTableEntry {
  pub start_pc: u16,
  pub length: u16,
  pub index: u16,
}

#[derive(Debug, Default)]
pub struct BootstrapMethod {
  pub bootstrap_method_attr_index: u16,
  pub num_bootstrap_arguments: u16,
  pub bootstrap_arguments: Vec<u16>,
}

#[derive(Debug, Default)]
pub struct MethodParameter {
  pub name_index: u16,
  pub access_flags: u16,
}

#[derive(Debug, Default)]
pub struct ClassFile {
  pub header: Header,
  pub constant_pool: ConstantPool,
  pub access_flags: u16,
  pub this_class: u16,
  pub super_class: u16,
  pub interfaces: Interfaces,
  pub fields: Fields,
  pub methods: Methods,
  pub attributes: ClassFileAttributes,
}

pub struct ClassFileParser {
  pub constant_pool: ConstantPool,
}

impl<'a> ClassFileParser {
  pub fn new() -> Self {
    ClassFileParser {
      constant_pool: ConstantPool::default(),
    }
  }

  pub fn parse(&mut self, input: &'a [u8]) -> IResult<&'a [u8], ClassFile> {
    let (input, header) = self.parse_header(input)?;

    let (input, constant_pool_count) = be_u16(input)?;

    let (input, constant_pool) = parse_constant_pool(constant_pool_count - 1, input)?;
    let constant_pool_clone = constant_pool.clone();
    self.constant_pool = constant_pool_clone;

    let (input, access_flags) = be_u16(input)?;
    let (input, this_class) = be_u16(input)?;
    let (input, super_class) = be_u16(input)?;
    let _ = self.constant_pool.check_class_index(this_class);
    let _ = self.constant_pool.check_class_index(super_class);

    let (input, interfaces) = self.parse_interfaces(input)?;
    let (input, fields) = self.parse_fields(input)?;
    let (input, methods) = self.parse_methods(input)?;
    let (input, attributes) = self.parse_class_file_attributes(input)?;

    let class_file = ClassFile {
      header,
      constant_pool: constant_pool,
      access_flags,
      this_class,
      super_class,
      interfaces,
      fields,
      methods,
      attributes,
    };
    Ok((input, class_file))
  }

  fn parse_header(&self, input: &'a [u8]) -> IResult<&'a [u8], Header> {
    let (input, magic) = be_u32(input)?;
    if magic != 0xCAFEBABE {
      return Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
      )));
    }

    let (input, minor) = be_u16(input)?;
    let (input, major) = be_u16(input)?;

    Ok((input, Header { magic, minor, major }))
  }

  fn parse_interfaces(&self, input: &'a [u8]) -> IResult<&'a [u8], Interfaces> {
    let (input, interfaces_count) = be_u16(input)?;
    let (input, interfaces) = nom::multi::count(be_u16, interfaces_count as usize).parse(input)?;
    let interfaces = Interfaces {
      interfaces_count,
      interfaces,
    };
    Ok((input, interfaces))
  }

  fn parse_field(&self, input: &'a [u8]) -> IResult<&'a [u8], Field> {
    let (input, access_flags) = be_u16(input)?;
    let (input, name_index) = be_u16(input)?;
    let (input, descriptor_index) = be_u16(input)?;
    let (input, attributes_count) = be_u16(input)?;

    let parse_field_info_attribute = |input: &'a [u8]| -> IResult<&'a [u8], FieldInfoAttribute> {
      let (input, index) = be_u16(input)?;
      let name = match &self.constant_pool.get_class(index) {
        Ok(Constant::Utf8 { bytes, .. }) => hex_utf8(bytes),
        _ => return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))),
      };
      let (input, attribute) = FieldInfoAttribute::parse(input, &name, index)?;
      Ok((input, attribute))
    };

    let (input, attributes) = nom::multi::count(parse_field_info_attribute, attributes_count as usize).parse(input)?;

    Ok((input, Field {
      access_flags,
      name_index,
      descriptor_index,
      attributes: FieldInfoAttributes {
        attributes_count,
        attributes,
      },
    }))
  }

  fn parse_fields(&self, input: &'a [u8]) -> IResult<&'a [u8], Fields> {
    let (input, fields_count) = be_u16(input)?;
    let (input, fields) = count(|i| self.parse_field(i), fields_count as usize).parse(input)?;
    Ok((input, Fields {
      fields_count,
      fields,
    }))
  }

  fn parse_method(&self, input: &'a [u8]) -> IResult<&'a [u8], Method> {
    let (input, access_flags) = be_u16(input)?;
    let (input, name_index) = be_u16(input)?;
    let (input, descriptor_index) = be_u16(input)?;
    let (input, attributes_count) = be_u16(input)?;
    let name = match self.constant_pool.get_class(name_index).unwrap() {
      Constant::Utf8 { bytes, .. } => hex_utf8(bytes),
      _ => return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
    };
    let descriptor = match self.constant_pool.get_class(descriptor_index).unwrap() {
      Constant::Utf8 { bytes, .. } => hex_utf8(bytes),
      _ => return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
    };

    let parse_method_info_attribute = |input: &'a [u8]| -> IResult<&'a [u8], MethodInfoAttribute> {
      let (input, index) = be_u16(input)?;
      let name = match &self.constant_pool.get_class(index).unwrap() {
        Constant::Utf8 { bytes, .. } => hex_utf8(bytes),
        _ => return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))),
      };
      let (input, attribute) = MethodInfoAttribute::parse(input, &name, index, &self.constant_pool)?;
      Ok((input, attribute))
    };

    let (input, attributes) = nom::multi::count(parse_method_info_attribute, attributes_count as usize).parse(input)?;

    Ok((input, Method {
      access_flags,
      name_index,
      descriptor_index,
      attributes: MethodInfoAttributes {
        attributes_count,
        attributes,
      },
    }))
  }

  fn parse_methods(&self, input: &'a [u8]) -> IResult<&'a [u8], Methods> {
    let (input, methods_count) = be_u16(input)?;
    let (input, methods) = count(|i| self.parse_method(i), methods_count as usize).parse(input)?;
    Ok((input, Methods {
      methods_count,
      methods,
    }))
  }

  fn parse_class_file_attribute(&self, input: &'a [u8]) -> IResult<&'a [u8], ClassFileAttribute> {
    let (input, index) = be_u16(input)?;
    let name = match self.constant_pool.get_class(index) {
      Ok(Constant::Utf8 { bytes, .. }) => hex_utf8(bytes),
      _ => return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))),
    };
    let (input, attribute) = ClassFileAttribute::parse(input, &name, index, &self.constant_pool)?;
    Ok((input, attribute))
  }

  fn parse_class_file_attributes(&self, input: &'a [u8]) -> IResult<&'a [u8], ClassFileAttributes> {
    let (input, attributes_count) = be_u16(input)?;
    let (input, attributes) = nom::multi::count(|i| self.parse_class_file_attribute(i), attributes_count as usize).parse(input)?;
    Ok((input, ClassFileAttributes {
      attributes_count,
      attributes,
    }))
  }
}