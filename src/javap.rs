use crate::structure::class::{ClassFile, ClassFileAttribute, CodeAttribute, Constant, MethodInfoAttribute};
use crate::util::class::{class_access_flags, constant_pool_viewer, field_access_flags, java_version_name, method_access_flags};
use crate::util::hex::hex_utf8;

pub fn javap_viewer(class_file: ClassFile) {
  println!("Magic: 0x{:X}", class_file.header.magic);
  println!("Minor Version: {}", class_file.header.minor);
  println!("Major Version: {}", class_file.header.major);
  println!("このファイルは{}バージョンのJavaでコンパイルされました", java_version_name(class_file.header.major));

  println!("\nConstant Pool:");
  constant_pool_viewer(&class_file.constant_pool.constants);

  println!("\nAccess Flags: {}", class_access_flags(class_file.access_flags));
  println!("This Class: #{}: {:?}", class_file.this_class, class_file.constant_pool.constants[class_file.this_class as usize - 1]);
  println!("Super Class: #{}: {:?}", class_file.super_class, class_file.constant_pool.constants[class_file.super_class as usize - 1]);

  println!("\nInterfaces count: {}", class_file.interfaces.interfaces_count);
  for (_, interface) in class_file.interfaces.interfaces.iter().enumerate() {
    println!("Interface #{}: {:?}", interface, class_file.constant_pool.constants[*interface as usize - 1]);
  }

  println!("\nFields count: {}", class_file.fields.fields_count);
  for (i, field) in class_file.fields.fields.iter().enumerate() {
    println!("Field #{}", i + 1);
    println!("Access Flags: {}", field_access_flags(field.access_flags));
    println!("Name: #{}: {:?}", field.name_index, class_file.constant_pool.constants[field.name_index as usize - 1]);
    println!("Descriptor: #{}: {:?}", field.descriptor_index, class_file.constant_pool.constants[field.descriptor_index as usize - 1]);
  }

  println!("\nMethods count: {}", class_file.methods.methods_count);
  for (i, method) in class_file.methods.methods.iter().enumerate() {
    println!("Method #{}", i + 1);
    println!("Access Flags: {}", method_access_flags(method.access_flags));
    let method_name = match &class_file.constant_pool.constants[method.name_index as usize - 1] {
      Constant::Utf8 { bytes, .. } => hex_utf8(&bytes),
      _ => format!("#{}", method.name_index),
    };
    println!("Name: #{}: {:?}", method.name_index, method_name);
    let method_descriptor = match &class_file.constant_pool.constants[method.descriptor_index as usize - 1] {
      Constant::Utf8 { bytes, .. } => hex_utf8(&bytes),
      _ => format!("#{}", method.descriptor_index),
    };
    println!("Descriptor: #{}: {:?}", method.descriptor_index, method_descriptor);
    
    if !method.attributes.attributes.is_empty() {
      println!("Attributes count: {}", method.attributes.attributes_count);
      for attr in &method.attributes.attributes {
        match attr {
          MethodInfoAttribute::Code(CodeAttribute { code, max_stack, max_locals, .. }) => {
            println!("Code Attribute:");
            println!("  Max Stack: {}", max_stack);
            println!("  Max Locals: {}", max_locals);
            for (j, byte) in code.iter().enumerate() {
              println!("  {}: {}", j, byte.name);
            }
          },
          _ => {
            println!("Attribute: {:?}", attr);
          }
        };
      }
    }
  }

  println!("\nClass Attributes count: {}", class_file.attributes.attributes_count);
  for attr in &class_file.attributes.attributes {
    match attr {
      ClassFileAttribute::SourceFile(source_file) => {
        let source_file_name = match &class_file.constant_pool.constants[source_file.source_file_index as usize - 1] {
          Constant::Utf8 { bytes, .. } => hex_utf8(&bytes),
          _ => format!("#{}", source_file.source_file_index),
        };
        println!("Source File: {}", source_file_name);
      }
      _ => {
        println!("Attribute: {:?}", attr);
      }
    };
  }
}