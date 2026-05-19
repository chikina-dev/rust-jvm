use rust_jvm::{
    structure::{
        class::{Constant, ConstantPool},
        code::CodeByte,
    },
    vm::{
        constant::ConstantPoolExt,
        descriptor::{JavaType, MethodDescriptor, parse_field_descriptor, parse_method_descriptor},
        error::ParseError,
        ids::{ClassIndex, CpIndex},
        instruction::{DecodedInstructionKind, decode_code},
    },
};

fn code_byte(name: &'static str, opcode: u8, data: &[u8]) -> CodeByte {
    CodeByte {
        name,
        opcode,
        length: (1 + data.len()) as u8,
        stack_behavior: "",
        data: data.to_vec(),
    }
}

#[test]
fn parses_field_descriptors() {
    assert_eq!(parse_field_descriptor("I"), Ok(JavaType::Int));
    assert_eq!(
        parse_field_descriptor("Ljava/lang/String;"),
        Ok(JavaType::Object("java/lang/String".to_string()))
    );
    assert_eq!(
        parse_field_descriptor("[[I"),
        Ok(JavaType::Array(Box::new(JavaType::Array(Box::new(
            JavaType::Int
        )))))
    );
}

#[test]
fn parses_method_descriptors() {
    assert_eq!(
        parse_method_descriptor("(ILjava/lang/String;[I)V"),
        Ok(MethodDescriptor {
            parameters: vec![
                JavaType::Int,
                JavaType::Object("java/lang/String".to_string()),
                JavaType::Array(Box::new(JavaType::Int)),
            ],
            return_type: JavaType::Void,
        })
    );
}

#[test]
fn rejects_invalid_descriptors() {
    assert_eq!(
        parse_field_descriptor("V"),
        Err(ParseError::InvalidDescriptor("V".to_string()))
    );
    assert_eq!(
        parse_method_descriptor("(I"),
        Err(ParseError::InvalidDescriptor("(I".to_string()))
    );
    assert_eq!(
        parse_method_descriptor("(V)V"),
        Err(ParseError::InvalidDescriptor("(V)V".to_string()))
    );
}

#[test]
fn typed_constant_pool_accessors_resolve_expected_entries() {
    let pool = ConstantPool {
        count: 6,
        constants: vec![
            Constant::Utf8 {
                length: 11,
                bytes: b"ExampleMain".to_vec(),
            },
            Constant::Class { name_index: 1 },
            Constant::Utf8 {
                length: 4,
                bytes: b"main".to_vec(),
            },
            Constant::Utf8 {
                length: 3,
                bytes: b"()I".to_vec(),
            },
            Constant::NameAndType {
                name_index: 3,
                descriptor_index: 4,
            },
        ],
    };

    assert_eq!(pool.utf8(CpIndex(1)), Ok("ExampleMain".to_string()));
    assert_eq!(
        pool.class_name(ClassIndex(2)),
        Ok("ExampleMain".to_string())
    );
    assert_eq!(
        pool.name_and_type(CpIndex(5)),
        Ok(("main".to_string(), "()I".to_string()))
    );
    assert_eq!(
        pool.utf8(CpIndex(99)),
        Err(ParseError::InvalidConstantPoolIndex(99))
    );
}

#[test]
fn decodes_supported_instructions_with_byte_offsets() {
    let decoded = decode_code(&[
        code_byte("iconst_1", 0x04, &[]),
        code_byte("bipush", 0x10, &[0xfb]),
        code_byte("istore_1", 0x3c, &[]),
        code_byte("iload_1", 0x1b, &[]),
        code_byte("ireturn", 0xac, &[]),
    ])
    .expect("code should decode");

    assert_eq!(decoded[0].pc, 0);
    assert_eq!(decoded[0].next_pc, 1);
    assert_eq!(decoded[0].kind, DecodedInstructionKind::IConst(1));
    assert_eq!(decoded[1].pc, 1);
    assert_eq!(decoded[1].next_pc, 3);
    assert_eq!(decoded[1].kind, DecodedInstructionKind::BiPush(-5));
    assert_eq!(decoded[2].pc, 3);
    assert_eq!(decoded[2].kind, DecodedInstructionKind::IStore(1));
    assert_eq!(decoded[4].pc, 5);
    assert_eq!(decoded[4].kind, DecodedInstructionKind::IReturn);
}

#[test]
fn separates_unknown_and_unsupported_opcodes() {
    let decoded = decode_code(&[
        code_byte("invokedynamic", 0xba, &[0x00, 0x01, 0x00, 0x00]),
        code_byte("Unknown", 0xff, &[]),
    ])
    .expect("code should decode");

    assert_eq!(
        decoded[0].kind,
        DecodedInstructionKind::Unsupported {
            mnemonic: "invokedynamic"
        }
    );
    assert_eq!(
        decoded[1].kind,
        DecodedInstructionKind::Unknown { opcode: 0xff }
    );
}

#[test]
fn rejects_malformed_operands() {
    assert_eq!(
        decode_code(&[code_byte("goto", 0xa7, &[0x00])]),
        Err(ParseError::ClassFormat(
            "goto expects 2 operand bytes, found 1".to_string()
        ))
    );
}
