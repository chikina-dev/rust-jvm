use crate::vm::ids::CpIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidMagic,
    UnexpectedEof,
    InvalidConstantPoolIndex(u16),
    InvalidConstantPoolTag(u8),
    InvalidDescriptor(String),
    InvalidOpcode(u8),
    InvalidAttributeLength { name: String },
    ClassFormat(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnsupportedFeature {
    Opcode {
        opcode: u8,
        mnemonic: &'static str,
    },
    Attribute {
        name: String,
    },
    Constant {
        tag: &'static str,
    },
    ClassVersion {
        major: u16,
        minor: u16,
    },
    NativeMethod {
        class: String,
        name: String,
        descriptor: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    ClassNotFound(String),
    NoSuchMethod {
        class: String,
        name: String,
        descriptor: String,
    },
    NoSuchField {
        class: String,
        name: String,
        descriptor: String,
    },
    IncompatibleClassChange(String),
    UnsupportedFeature(UnsupportedFeature),
    VerificationError(String),
    LinkageError(String),
    RuntimeException(String),
    InternalError(String),
    Parse(ParseError),
    ConstantResolution {
        index: CpIndex,
        reason: String,
    },
}

impl From<ParseError> for VmError {
    fn from(error: ParseError) -> Self {
        Self::Parse(error)
    }
}
