use crate::{
    structure::code::CodeByte,
    vm::{error::ParseError, ids::CpIndex},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedInstruction {
    pub pc: u32,
    pub next_pc: u32,
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub kind: DecodedInstructionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedInstructionKind {
    Nop,
    AConstNull,
    IConst(i32),
    BiPush(i8),
    SiPush(i16),
    Ldc(CpIndex),
    ALoad(u16),
    AStore(u16),
    AALoad,
    AAStore,
    IALoad,
    IAStore,
    ILoad(u16),
    IStore(u16),
    Dup,
    IAdd,
    ISub,
    IMul,
    IDiv,
    IRem,
    Goto(i16),
    IfEq(i16),
    IfNe(i16),
    IfLt(i16),
    IfGe(i16),
    IfGt(i16),
    IfLe(i16),
    IfICmpEq(i16),
    IfICmpNe(i16),
    IfICmpLt(i16),
    IfICmpGe(i16),
    IfICmpGt(i16),
    IfICmpLe(i16),
    IInc { index: u16, value: i16 },
    InvokeStatic(CpIndex),
    InvokeSpecial(CpIndex),
    InvokeVirtual(CpIndex),
    New(CpIndex),
    ANewArray(CpIndex),
    NewArray(u8),
    ArrayLength,
    GetStatic(CpIndex),
    GetField(CpIndex),
    PutField(CpIndex),
    Return,
    IReturn,
    AReturn,
    Unsupported { mnemonic: &'static str },
    Unknown { opcode: u8 },
}

pub fn decode_code(code: &[CodeByte]) -> Result<Vec<DecodedInstruction>, ParseError> {
    let mut pc = 0u32;
    let mut decoded = Vec::with_capacity(code.len());

    for byte in code {
        let size = instruction_size(byte)?;
        let next_pc = pc
            .checked_add(size)
            .ok_or_else(|| ParseError::ClassFormat("bytecode pc overflow".to_string()))?;
        let kind = decode_kind(byte)?;
        decoded.push(DecodedInstruction {
            pc,
            next_pc,
            opcode: byte.opcode,
            mnemonic: byte.name,
            kind,
        });
        pc = next_pc;
    }

    Ok(decoded)
}

fn instruction_size(byte: &CodeByte) -> Result<u32, ParseError> {
    let size = 1usize
        .checked_add(byte.data.len())
        .ok_or_else(|| ParseError::ClassFormat("bytecode instruction size overflow".to_string()))?;
    u32::try_from(size)
        .map_err(|_| ParseError::ClassFormat("bytecode instruction size overflow".to_string()))
}

fn decode_kind(byte: &CodeByte) -> Result<DecodedInstructionKind, ParseError> {
    if byte.name == "Unknown" {
        return Ok(DecodedInstructionKind::Unknown {
            opcode: byte.opcode,
        });
    }

    match byte.opcode {
        0x00 => Ok(DecodedInstructionKind::Nop),
        0x01 => Ok(DecodedInstructionKind::AConstNull),
        0x02 => Ok(DecodedInstructionKind::IConst(-1)),
        0x03 => Ok(DecodedInstructionKind::IConst(0)),
        0x04 => Ok(DecodedInstructionKind::IConst(1)),
        0x05 => Ok(DecodedInstructionKind::IConst(2)),
        0x06 => Ok(DecodedInstructionKind::IConst(3)),
        0x07 => Ok(DecodedInstructionKind::IConst(4)),
        0x08 => Ok(DecodedInstructionKind::IConst(5)),
        0x10 => Ok(DecodedInstructionKind::BiPush(read_i8(byte)?)),
        0x11 => Ok(DecodedInstructionKind::SiPush(read_i16(byte)?)),
        0x12 => Ok(DecodedInstructionKind::Ldc(CpIndex(read_u8(byte)? as u16))),
        0x13 | 0x14 => Ok(DecodedInstructionKind::Ldc(CpIndex(read_u16(byte)?))),
        0x15 => Ok(DecodedInstructionKind::ILoad(read_u8(byte)? as u16)),
        0x19 => Ok(DecodedInstructionKind::ALoad(read_u8(byte)? as u16)),
        0x1a => Ok(DecodedInstructionKind::ILoad(0)),
        0x1b => Ok(DecodedInstructionKind::ILoad(1)),
        0x1c => Ok(DecodedInstructionKind::ILoad(2)),
        0x1d => Ok(DecodedInstructionKind::ILoad(3)),
        0x2a => Ok(DecodedInstructionKind::ALoad(0)),
        0x2b => Ok(DecodedInstructionKind::ALoad(1)),
        0x2c => Ok(DecodedInstructionKind::ALoad(2)),
        0x2d => Ok(DecodedInstructionKind::ALoad(3)),
        0x2e => Ok(DecodedInstructionKind::IALoad),
        0x32 => Ok(DecodedInstructionKind::AALoad),
        0x36 => Ok(DecodedInstructionKind::IStore(read_u8(byte)? as u16)),
        0x3a => Ok(DecodedInstructionKind::AStore(read_u8(byte)? as u16)),
        0x3b => Ok(DecodedInstructionKind::IStore(0)),
        0x3c => Ok(DecodedInstructionKind::IStore(1)),
        0x3d => Ok(DecodedInstructionKind::IStore(2)),
        0x3e => Ok(DecodedInstructionKind::IStore(3)),
        0x4b => Ok(DecodedInstructionKind::AStore(0)),
        0x4c => Ok(DecodedInstructionKind::AStore(1)),
        0x4d => Ok(DecodedInstructionKind::AStore(2)),
        0x4e => Ok(DecodedInstructionKind::AStore(3)),
        0x4f => Ok(DecodedInstructionKind::IAStore),
        0x53 => Ok(DecodedInstructionKind::AAStore),
        0x59 => Ok(DecodedInstructionKind::Dup),
        0x60 => Ok(DecodedInstructionKind::IAdd),
        0x64 => Ok(DecodedInstructionKind::ISub),
        0x68 => Ok(DecodedInstructionKind::IMul),
        0x6c => Ok(DecodedInstructionKind::IDiv),
        0x70 => Ok(DecodedInstructionKind::IRem),
        0x84 => decode_iinc(byte),
        0x99 => Ok(DecodedInstructionKind::IfEq(read_i16(byte)?)),
        0x9a => Ok(DecodedInstructionKind::IfNe(read_i16(byte)?)),
        0x9b => Ok(DecodedInstructionKind::IfLt(read_i16(byte)?)),
        0x9c => Ok(DecodedInstructionKind::IfGe(read_i16(byte)?)),
        0x9d => Ok(DecodedInstructionKind::IfGt(read_i16(byte)?)),
        0x9e => Ok(DecodedInstructionKind::IfLe(read_i16(byte)?)),
        0x9f => Ok(DecodedInstructionKind::IfICmpEq(read_i16(byte)?)),
        0xa0 => Ok(DecodedInstructionKind::IfICmpNe(read_i16(byte)?)),
        0xa1 => Ok(DecodedInstructionKind::IfICmpLt(read_i16(byte)?)),
        0xa2 => Ok(DecodedInstructionKind::IfICmpGe(read_i16(byte)?)),
        0xa3 => Ok(DecodedInstructionKind::IfICmpGt(read_i16(byte)?)),
        0xa4 => Ok(DecodedInstructionKind::IfICmpLe(read_i16(byte)?)),
        0xa7 => Ok(DecodedInstructionKind::Goto(read_i16(byte)?)),
        0xac => Ok(DecodedInstructionKind::IReturn),
        0xb0 => Ok(DecodedInstructionKind::AReturn),
        0xb1 => Ok(DecodedInstructionKind::Return),
        0xb2 => Ok(DecodedInstructionKind::GetStatic(CpIndex(read_u16(byte)?))),
        0xb4 => Ok(DecodedInstructionKind::GetField(CpIndex(read_u16(byte)?))),
        0xb5 => Ok(DecodedInstructionKind::PutField(CpIndex(read_u16(byte)?))),
        0xb6 => Ok(DecodedInstructionKind::InvokeVirtual(CpIndex(read_u16(
            byte,
        )?))),
        0xb7 => Ok(DecodedInstructionKind::InvokeSpecial(CpIndex(read_u16(
            byte,
        )?))),
        0xb8 => Ok(DecodedInstructionKind::InvokeStatic(CpIndex(read_u16(
            byte,
        )?))),
        0xbb => Ok(DecodedInstructionKind::New(CpIndex(read_u16(byte)?))),
        0xbc => Ok(DecodedInstructionKind::NewArray(read_u8(byte)?)),
        0xbd => Ok(DecodedInstructionKind::ANewArray(CpIndex(read_u16(byte)?))),
        0xbe => Ok(DecodedInstructionKind::ArrayLength),
        _ => Ok(DecodedInstructionKind::Unsupported {
            mnemonic: byte.name,
        }),
    }
}

fn decode_iinc(byte: &CodeByte) -> Result<DecodedInstructionKind, ParseError> {
    require_operand_len(byte, 2)?;
    Ok(DecodedInstructionKind::IInc {
        index: byte.data[0] as u16,
        value: byte.data[1] as i8 as i16,
    })
}

fn read_u8(byte: &CodeByte) -> Result<u8, ParseError> {
    require_operand_len(byte, 1)?;
    Ok(byte.data[0])
}

fn read_i8(byte: &CodeByte) -> Result<i8, ParseError> {
    Ok(read_u8(byte)? as i8)
}

fn read_u16(byte: &CodeByte) -> Result<u16, ParseError> {
    require_operand_len(byte, 2)?;
    Ok(u16::from_be_bytes([byte.data[0], byte.data[1]]))
}

fn read_i16(byte: &CodeByte) -> Result<i16, ParseError> {
    require_operand_len(byte, 2)?;
    Ok(i16::from_be_bytes([byte.data[0], byte.data[1]]))
}

fn require_operand_len(byte: &CodeByte, expected: usize) -> Result<(), ParseError> {
    if byte.data.len() == expected {
        Ok(())
    } else {
        Err(ParseError::ClassFormat(format!(
            "{} expects {expected} operand bytes, found {}",
            byte.name,
            byte.data.len()
        )))
    }
}
