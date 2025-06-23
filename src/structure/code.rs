use phf::phf_map;

#[derive(Debug, Clone)]
pub struct CodeByte {
  pub name: &'static str,
  pub opcode: u8,
  pub length: u8,
  pub stack_behavior: &'static str,
  pub data: Vec<u8>,
}

pub static CODE_BYTES: phf::Map<u8, CodeByte> = phf_map! {
  0x00u8 => CodeByte {
    name: "nop",
    opcode: 0x00,
    length: 0x01,
    stack_behavior: "No change",
    data: Vec::new()
  },
  0x01u8 => CodeByte {
    name: "aconst_null",
    opcode: 0x01,
    length: 0x01,
    stack_behavior: "... -> ..., null",
    data: Vec::new()
  },
  0x02u8 => CodeByte {
    name: "iconst_m1",
    opcode: 0x02,
    length: 0x01,
    stack_behavior: "... -> ..., -1",
    data: Vec::new()
  },
  0x03u8 => CodeByte {
    name: "iconst_0",
    opcode: 0x03,
    length: 0x01,
    stack_behavior: "... -> ..., 0",
    data: Vec::new()
  },
  0x04u8 => CodeByte {
    name: "iconst_1",
    opcode: 0x04,
    length: 0x01,
    stack_behavior: "... -> ..., 1",
    data: Vec::new()
  },
  0x05u8 => CodeByte {
    name: "iconst_2",
    opcode: 0x05,
    length: 0x01,
    stack_behavior: "... -> ..., 2",
    data: Vec::new()
  },
  0x06u8 => CodeByte {
    name: "iconst_3",
    opcode: 0x06,
    length: 0x01,
    stack_behavior: "... -> ..., 3",
    data: Vec::new()
  },
  0x07u8 => CodeByte {
    name: "iconst_4",
    opcode: 0x07,
    length: 0x01,
    stack_behavior: "... -> ..., 4",
    data: Vec::new()
  },
  0x08u8 => CodeByte {
    name: "iconst_5",
    opcode: 0x08,
    length: 0x01,
    stack_behavior: "... -> ..., 5",
    data: Vec::new()
  },
  0x09u8 => CodeByte {
    name: "lconst_0",
    opcode: 0x09,
    length: 0x01,
    stack_behavior: "... -> ..., 0L",
    data: Vec::new()
  },
  0x0au8 => CodeByte {
    name: "lconst_1",
    opcode: 0x0a,
    length: 0x01,
    stack_behavior: "... -> ..., 1L",
    data: Vec::new()
  },
  0x0bu8 => CodeByte {
    name: "fconst_0",
    opcode: 0x0b,
    length: 0x01,
    stack_behavior: "... -> ..., 0.0f",
    data: Vec::new()
  },
  0x0cu8 => CodeByte {
    name: "fconst_1",
    opcode: 0x0c,
    length: 0x01,
    stack_behavior: "... -> ..., 1.0f",
    data: Vec::new()
  },
  0x0du8 => CodeByte {
    name: "fconst_2",
    opcode: 0x0d,
    length: 0x01,
    stack_behavior: "... -> ..., 2.0f",
    data: Vec::new()
  },
  0x0eu8 => CodeByte {
    name: "dconst_0",
    opcode: 0x0e,
    length: 0x01,
    stack_behavior: "... -> ..., 0.0d",
    data: Vec::new()
  },
  0x0fu8 => CodeByte {
    name: "dconst_1",
    opcode: 0x0f,
    length: 0x01,
    stack_behavior: "... -> ..., 1.0d",
    data: Vec::new()
  },
  0x10u8 => CodeByte {
    name: "bipush",
    opcode: 0x10,
    length: 0x02,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x11u8 => CodeByte {
    name: "sipush",
    opcode: 0x11,
    length: 0x03,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x12u8 => CodeByte {
    name: "ldc",
    opcode: 0x12,
    length: 0x02,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x13u8 => CodeByte {
    name: "ldc_w",
    opcode: 0x13,
    length: 0x03,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x14u8 => CodeByte {
    name: "ldc2_w",
    opcode: 0x14,
    length: 0x03,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x15u8 => CodeByte {
    name: "iload",
    opcode: 0x15,
    length: 0x02,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x16u8 => CodeByte {
    name: "lload",
    opcode: 0x16,
    length: 0x02,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x17u8 => CodeByte {
    name: "fload",
    opcode: 0x17,
    length: 0x02,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x18u8 => CodeByte {
    name: "dload",
    opcode: 0x18,
    length: 0x02,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x19u8 => CodeByte {
    name: "aload",
    opcode: 0x19,
    length: 0x02,
    stack_behavior: "... -> ..., objectref",
    data: Vec::new()
  },
  0x1au8 => CodeByte {
    name: "iload_0",
    opcode: 0x1a,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x1bu8 => CodeByte {
    name: "iload_1",
    opcode: 0x1b,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x1cu8 => CodeByte {
    name: "iload_2",
    opcode: 0x1c,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x1du8 => CodeByte {
    name: "iload_3",
    opcode: 0x1d,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x1eu8 => CodeByte {
    name: "lload_0",
    opcode: 0x1e,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x1fu8 => CodeByte {
    name: "lload_1",
    opcode: 0x1f,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x20u8 => CodeByte {
    name: "lload_2",
    opcode: 0x20,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x21u8 => CodeByte {
    name: "lload_3",
    opcode: 0x21,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x22u8 => CodeByte {
    name: "fload_0",
    opcode: 0x22,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x23u8 => CodeByte {
    name: "fload_1",
    opcode: 0x23,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x24u8 => CodeByte {
    name: "fload_2",
    opcode: 0x24,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x25u8 => CodeByte {
    name: "fload_3",
    opcode: 0x25,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x26u8 => CodeByte {
    name: "dload_0",
    opcode: 0x26,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x27u8 => CodeByte {
    name: "dload_1",
    opcode: 0x27,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x28u8 => CodeByte {
    name: "dload_2",
    opcode: 0x28,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x29u8 => CodeByte {
    name: "dload_3",
    opcode: 0x29,
    length: 0x01,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0x2au8 => CodeByte {
    name: "aload_0",
    opcode: 0x2a,
    length: 0x01,
    stack_behavior: "... -> ..., objectref",
    data: Vec::new()
  },
  0x2bu8 => CodeByte {
    name: "aload_1",
    opcode: 0x2b,
    length: 0x01,
    stack_behavior: "... -> ..., objectref",
    data: Vec::new()
  },
  0x2cu8 => CodeByte {
    name: "aload_2",
    opcode: 0x2c,
    length: 0x01,
    stack_behavior: "... -> ..., objectref",
    data: Vec::new()
  },
  0x2du8 => CodeByte {
    name: "aload_3",
    opcode: 0x2d,
    length: 0x01,
    stack_behavior: "... -> ..., objectref",
    data: Vec::new()
  },
  0x2eu8 => CodeByte {
    name: "iaload",
    opcode: 0x2e,
    length: 0x01,
    stack_behavior: "..., arrayref, index -> ..., value",
    data: Vec::new()
  },
  0x2fu8 => CodeByte {
    name: "laload",
    opcode: 0x2f,
    length: 0x01,
    stack_behavior: "..., arrayref, index -> ..., value",
    data: Vec::new()
  },
  0x30u8 => CodeByte {
    name: "faload",
    opcode: 0x30,
    length: 0x01,
    stack_behavior: "..., arrayref, index -> ..., value",
    data: Vec::new()
  },
  0x31u8 => CodeByte {
    name: "daload",
    opcode: 0x31,
    length: 0x01,
    stack_behavior: "..., arrayref, index -> ..., value",
    data: Vec::new()
  },
  0x32u8 => CodeByte {
    name: "aaload",
    opcode: 0x32,
    length: 0x01,
    stack_behavior: "..., arrayref, index -> ..., value",
    data: Vec::new()
  },
  0x33u8 => CodeByte {
    name: "baload",
    opcode: 0x33,
    length: 0x01,
    stack_behavior: "..., arrayref, index -> ..., value",
    data: Vec::new()
  },
  0x34u8 => CodeByte {
    name: "caload",
    opcode: 0x34,
    length: 0x01,
    stack_behavior: "..., arrayref, index -> ..., value",
    data: Vec::new()
  },
  0x35u8 => CodeByte {
    name: "saload",
    opcode: 0x35,
    length: 0x01,
    stack_behavior: "..., arrayref, index -> ..., value",
    data: Vec::new()
  },
  0x36u8 => CodeByte {
    name: "istore",
    opcode: 0x36,
    length: 0x02,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x37u8 => CodeByte {
    name: "lstore",
    opcode: 0x37,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x38u8 => CodeByte {
    name: "fstore",
    opcode: 0x38,
    length: 0x02,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x39u8 => CodeByte {
    name: "dstore",
    opcode: 0x39,
    length: 0x02,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x3au8 => CodeByte {
    name: "astore",
    opcode: 0x3a,
    length: 0x02,
    stack_behavior: "..., objectref -> ...",
    data: Vec::new()
  },
  0x3bu8 => CodeByte {
    name: "istore_0",
    opcode: 0x3b,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x3cu8 => CodeByte {
    name: "istore_1",
    opcode: 0x3c,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x3du8 => CodeByte {
    name: "istore_2",
    opcode: 0x3d,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x3eu8 => CodeByte {
    name: "istore_3",
    opcode: 0x3e,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x3fu8 => CodeByte {
    name: "lstore_0",
    opcode: 0x3f,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x40u8 => CodeByte {
    name: "lstore_1",
    opcode: 0x40,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x41u8 => CodeByte {
    name: "lstore_2",
    opcode: 0x41,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x42u8 => CodeByte {
    name: "lstore_3",
    opcode: 0x42,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x43u8 => CodeByte {
    name: "fstore_0",
    opcode: 0x43,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x44u8 => CodeByte {
    name: "fstore_1",
    opcode: 0x44,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x45u8 => CodeByte {
    name: "fstore_2",
    opcode: 0x45,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x46u8 => CodeByte {
    name: "fstore_3",
    opcode: 0x46,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x47u8 => CodeByte {
    name: "dstore_0",
    opcode: 0x47,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x48u8 => CodeByte {
    name: "dstore_1",
    opcode: 0x48,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x49u8 => CodeByte {
    name: "dstore_2",
    opcode: 0x49,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x4au8 => CodeByte {
    name: "dstore_3",
    opcode: 0x4a,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x4bu8 => CodeByte {
    name: "astore_0",
    opcode: 0x4b,
    length: 0x01,
    stack_behavior: "..., objectref -> ...",
    data: Vec::new()
  },
  0x4cu8 => CodeByte {
    name: "astore_1",
    opcode: 0x4c,
    length: 0x01,
    stack_behavior: "..., objectref -> ...",
    data: Vec::new()
  },
  0x4du8 => CodeByte {
    name: "astore_2",
    opcode: 0x4d,
    length: 0x01,
    stack_behavior: "..., objectref -> ...",
    data: Vec::new()
  },
  0x4eu8 => CodeByte {
    name: "astore_3",
    opcode: 0x4e,
    length: 0x01,
    stack_behavior: "..., objectref -> ...",
    data: Vec::new()
  },
  0x4fu8 => CodeByte {
    name: "iastore",
    opcode: 0x4f,
    length: 0x01,
    stack_behavior: "..., arrayref, index, value -> ...",
    data: Vec::new()
  },
  0x50u8 => CodeByte {
    name: "lastore",
    opcode: 0x50,
    length: 0x01,
    stack_behavior: "..., arrayref, index, value -> ...",
    data: Vec::new()
  },
  0x51u8 => CodeByte {
    name: "fastore",
    opcode: 0x51,
    length: 0x01,
    stack_behavior: "..., arrayref, index, value -> ...",
    data: Vec::new()
  },
  0x52u8 => CodeByte {
    name: "dastore",
    opcode: 0x52,
    length: 0x01,
    stack_behavior: "..., arrayref, index, value -> ...",
    data: Vec::new()
  },
  0x53u8 => CodeByte {
    name: "aastore",
    opcode: 0x53,
    length: 0x01,
    stack_behavior: "..., arrayref, index, value -> ...",
    data: Vec::new()
  },
  0x54u8 => CodeByte {
    name: "bastore",
    opcode: 0x54,
    length: 0x01,
    stack_behavior: "..., arrayref, index, value -> ...",
    data: Vec::new()
  },
  0x55u8 => CodeByte {
    name: "castore",
    opcode: 0x55,
    length: 0x01,
    stack_behavior: "..., arrayref, index, value -> ...",
    data: Vec::new()
  },
  0x56u8 => CodeByte {
    name: "sastore",
    opcode: 0x56,
    length: 0x01,
    stack_behavior: "..., arrayref, index, value -> ...",
    data: Vec::new()
  },
  0x57u8 => CodeByte {
    name: "pop",
    opcode: 0x57,
    length: 0x01,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x58u8 => CodeByte {
    name: "pop2",
    opcode: 0x58,
    length: 0x01,
    stack_behavior: "..., value2, value1 -> ...",
    data: Vec::new()
  },
  0x59u8 => CodeByte {
    name: "dup",
    opcode: 0x59,
    length: 0x01,
    stack_behavior: "..., value -> ..., value, value",
    data: Vec::new()
  },
  0x5au8 => CodeByte {
    name: "dup_x1",
    opcode: 0x5a,
    length: 0x01,
    stack_behavior: "..., value2, value1 -> ..., value1, value2, value1",
    data: Vec::new()
  },
  0x5bu8 => CodeByte {
    name: "dup_x2",
    opcode: 0x5b,
    length: 0x01,
    stack_behavior: "..., value3, value2, value1 -> ..., value1, value3, value2, value1",
    data: Vec::new()
  },
  0x5cu8 => CodeByte {
    name: "dup2",
    opcode: 0x5c,
    length: 0x01,
    stack_behavior: "..., {value2, value1} -> ..., {value2, value1}, {value2, value1}",
    data: Vec::new()
  },
  0x5du8 => CodeByte {
    name: "dup2_x1",
    opcode: 0x5d,
    length: 0x01,
    stack_behavior: "..., value3, {value2, value1} -> ..., {value2, value1}, value3, {value2, value1}",
    data: Vec::new()
  },
  0x5eu8 => CodeByte {
    name: "dup2_x2",
    opcode: 0x5e,
    length: 0x01,
    stack_behavior: "..., {value4, value3}, {value2, value1} -> ..., {value2, value1}, {value4, value3}, {value2, value1}",
    data: Vec::new()
  },
  0x5fu8 => CodeByte {
    name: "swap",
    opcode: 0x5f,
    length: 0x01,
    stack_behavior: "..., value2, value1 -> ..., value1, value2",
    data: Vec::new()
  },
  0x60u8 => CodeByte {
    name: "iadd",
    opcode: 0x60,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x61u8 => CodeByte {
    name: "ladd",
    opcode: 0x61,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x62u8 => CodeByte {
    name: "fadd",
    opcode: 0x62,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x63u8 => CodeByte {
    name: "dadd",
    opcode: 0x63,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x64u8 => CodeByte {
    name: "isub",
    opcode: 0x64,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x65u8 => CodeByte {
    name: "lsub",
    opcode: 0x65,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x66u8 => CodeByte {
    name: "fsub",
    opcode: 0x66,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x67u8 => CodeByte {
    name: "dsub",
    opcode: 0x67,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x68u8 => CodeByte {
    name: "imul",
    opcode: 0x68,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x69u8 => CodeByte {
    name: "lmul",
    opcode: 0x69,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x6au8 => CodeByte {
    name: "fmul",
    opcode: 0x6a,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x6bu8 => CodeByte {
    name: "dmul",
    opcode: 0x6b,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x6cu8 => CodeByte {
    name: "idiv",
    opcode: 0x6c,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x6du8 => CodeByte {
    name: "ldiv",
    opcode: 0x6d,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x6eu8 => CodeByte {
    name: "fdiv",
    opcode: 0x6e,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x6fu8 => CodeByte {
    name: "ddiv",
    opcode: 0x6f,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x70u8 => CodeByte {
    name: "irem",
    opcode: 0x70,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x71u8 => CodeByte {
    name: "lrem",
    opcode: 0x71,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x72u8 => CodeByte {
    name: "frem",
    opcode: 0x72,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x73u8 => CodeByte {
    name: "drem",
    opcode: 0x73,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x74u8 => CodeByte {
    name: "ineg",
    opcode: 0x74,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x75u8 => CodeByte {
    name: "lneg",
    opcode: 0x75,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x76u8 => CodeByte {
    name: "fneg",
    opcode: 0x76,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x77u8 => CodeByte {
    name: "dneg",
    opcode: 0x77,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x78u8 => CodeByte {
    name: "ishl",
    opcode: 0x78,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x79u8 => CodeByte {
    name: "lshl",
    opcode: 0x79,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x7au8 => CodeByte {
    name: "ishr",
    opcode: 0x7a,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x7bu8 => CodeByte {
    name: "lshr",
    opcode: 0x7b,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x7cu8 => CodeByte {
    name: "iushr",
    opcode: 0x7c,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x7du8 => CodeByte {
    name: "lushr",
    opcode: 0x7d,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x7eu8 => CodeByte {
    name: "iand",
    opcode: 0x7e,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x7fu8 => CodeByte {
    name: "land",
    opcode: 0x7f,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x80u8 => CodeByte {
    name: "ior",
    opcode: 0x80,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x81u8 => CodeByte {
    name: "lor",
    opcode: 0x81,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x82u8 => CodeByte {
    name: "ixor",
    opcode: 0x82,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x83u8 => CodeByte {
    name: "lxor",
    opcode: 0x83,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x84u8 => CodeByte {
    name: "iinc",
    opcode: 0x84,
    length: 0x01,
    stack_behavior: "No change",
    data: Vec::new()
  },
  0x85u8 => CodeByte {
    name: "i2l",
    opcode: 0x85,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x86u8 => CodeByte {
    name: "i2f",
    opcode: 0x86,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x87u8 => CodeByte {
    name: "i2d",
    opcode: 0x87,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x88u8 => CodeByte {
    name: "l2i",
    opcode: 0x88,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x89u8 => CodeByte {
    name: "l2f",
    opcode: 0x89,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x8au8 => CodeByte {
    name: "l2d",
    opcode: 0x8a,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x8bu8 => CodeByte {
    name: "f2i",
    opcode: 0x8b,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x8cu8 => CodeByte {
    name: "f2l",
    opcode: 0x8c,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x8du8 => CodeByte {
    name: "f2d",
    opcode: 0x8d,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x8eu8 => CodeByte {
    name: "d2i",
    opcode: 0x8e,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x8fu8 => CodeByte {
    name: "d2l",
    opcode: 0x8f,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x90u8 => CodeByte {
    name: "d2f",
    opcode: 0x90,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x91u8 => CodeByte {
    name: "i2b",
    opcode: 0x91,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x92u8 => CodeByte {
    name: "i2c",
    opcode: 0x92,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x93u8 => CodeByte {
    name: "i2s",
    opcode: 0x93,
    length: 0x01,
    stack_behavior: "..., value -> ..., result",
    data: Vec::new()
  },
  0x94u8 => CodeByte {
    name: "lcmp",
    opcode: 0x94,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x95u8 => CodeByte {
    name: "fcmpl",
    opcode: 0x95,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x96u8 => CodeByte {
    name: "fcmpg",
    opcode: 0x96,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x97u8 => CodeByte {
    name: "dcmpl",
    opcode: 0x97,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x98u8 => CodeByte {
    name: "dcmpg",
    opcode: 0x98,
    length: 0x01,
    stack_behavior: "..., value1, value2 -> ..., result",
    data: Vec::new()
  },
  0x99u8 => CodeByte {
    name: "ifeq",
    opcode: 0x99,
    length: 0x03,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x9au8 => CodeByte {
    name: "ifne",
    opcode: 0x9a,
    length: 0x03,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x9bu8 => CodeByte {
    name: "iflt",
    opcode: 0x9b,
    length: 0x03,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x9cu8 => CodeByte {
    name: "ifge",
    opcode: 0x9c,
    length: 0x03,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x9du8 => CodeByte {
    name: "ifgt",
    opcode: 0x9d,
    length: 0x03,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x9eu8 => CodeByte {
    name: "ifle",
    opcode: 0x9e,
    length: 0x03,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0x9fu8 => CodeByte {
    name: "if_icmpeq",
    opcode: 0x9f,
    length: 0x03,
    stack_behavior: "..., value1, value2 -> ...",
    data: Vec::new()
  },
  0xa0u8 => CodeByte {
    name: "if_icmpne",
    opcode: 0xa0,
    length: 0x03,
    stack_behavior: "..., value1, value2 -> ...",
    data: Vec::new()
  },
  0xa1u8 => CodeByte {
    name: "if_icmplt",
    opcode: 0xa1,
    length: 0x03,
    stack_behavior: "..., value1, value2 -> ...",
    data: Vec::new()
  },
  0xa2u8 => CodeByte {
    name: "if_icmpge",
    opcode: 0xa2,
    length: 0x03,
    stack_behavior: "..., value1, value2 -> ...",
    data: Vec::new()
  },
  0xa3u8 => CodeByte {
    name: "if_icmpgt",
    opcode: 0xa3,
    length: 0x03,
    stack_behavior: "..., value1, value2 -> ...",
    data: Vec::new()
  },
  0xa4u8 => CodeByte {
    name: "if_icmple",
    opcode: 0xa4,
    length: 0x03,
    stack_behavior: "..., value1, value2 -> ...",
    data: Vec::new()
  },
  0xa5u8 => CodeByte {
    name: "if_acmpeq",
    opcode: 0xa5,
    length: 0x03,
    stack_behavior: "..., value1, value2 -> ...",
    data: Vec::new()
  },
  0xa6u8 => CodeByte {
    name: "if_acmpne",
    opcode: 0xa6,
    length: 0x03,
    stack_behavior: "..., value1, value2 -> ...",
    data: Vec::new()
  },
  0xa7u8 => CodeByte {
    name: "goto",
    opcode: 0xa7,
    length: 0x03,
    stack_behavior: "No change",
    data: Vec::new()
  },
  0xa8u8 => CodeByte {
    name: "jsr",
    opcode: 0xa8,
    length: 0x03,
    stack_behavior: "... -> ..., address",
    data: Vec::new()
  },
  0xa9u8 => CodeByte {
    name: "ret",
    opcode: 0xa9,
    length: 0x02,
    stack_behavior: "No change",
    data: Vec::new()
  },
  0xaau8 => CodeByte { // TODO: テーブルスイッチ命令の長さは可変
    name: "tableswitch",
    opcode: 0xaa,
    length: 0x01,
    stack_behavior: "..., index -> ...",
    data: Vec::new()
  },
  0xabu8 => CodeByte { // TODO: ルックアップスイッチ命令の長さは可変
    name: "lookupswitch",
    opcode: 0xab,
    length: 0x01,
    stack_behavior: "..., key -> ...",
    data: Vec::new()
  },
  0xacu8 => CodeByte {
    name: "ireturn",
    opcode: 0xac,
    length: 0x01,
    stack_behavior: "..., value -> [empty]",
    data: Vec::new()
  },
  0xadu8 => CodeByte {
    name: "lreturn",
    opcode: 0xad,
    length: 0x01,
    stack_behavior: "..., value -> [empty]",
    data: Vec::new()
  },
  0xaeu8 => CodeByte {
    name: "freturn",
    opcode: 0xae,
    length: 0x01,
    stack_behavior: "..., value -> [empty]",
    data: Vec::new()
  },
  0xafu8 => CodeByte {
    name: "dreturn",
    opcode: 0xaf,
    length: 0x01,
    stack_behavior: "..., value -> [empty]",
    data: Vec::new()
  },
  0xb0u8 => CodeByte {
    name: "areturn",
    opcode: 0xb0,
    length: 0x01,
    stack_behavior: "..., objectref -> [empty]",
    data: Vec::new()
  },
  0xb1u8 => CodeByte {
    name: "return",
    opcode: 0xb1,
    length: 0x01,
    stack_behavior: "... -> [empty]",
    data: Vec::new()
  },
  0xb2u8 => CodeByte {
    name: "getstatic",
    opcode: 0xb2,
    length: 0x03,
    stack_behavior: "... -> ..., value",
    data: Vec::new()
  },
  0xb3u8 => CodeByte {
    name: "putstatic",
    opcode: 0xb3,
    length: 0x03,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0xb4u8 => CodeByte {
    name: "getfield",
    opcode: 0xb4,
    length: 0x03,
    stack_behavior: "..., objectref -> ..., value",
    data: Vec::new()
  },
  0xb5u8 => CodeByte {
    name: "putfield",
    opcode: 0xb5,
    length: 0x03,
    stack_behavior: "..., objectref, value -> ...",
    data: Vec::new()
  },
  0xb6u8 => CodeByte {
    name: "invokevirtual",
    opcode: 0xb6,
    length: 0x03,
    stack_behavior: "..., objectref, [arg1, [arg2 ...]] -> ...",
    data: Vec::new()
  },
  0xb7u8 => CodeByte {
    name: "invokespecial",
    opcode: 0xb7,
    length: 0x03,
    stack_behavior: "..., objectref, [arg1, [arg2 ...]] -> ...",
    data: Vec::new()
  },
  0xb8u8 => CodeByte {
    name: "invokestatic",
    opcode: 0xb8,
    length: 0x03,
    stack_behavior: "..., [arg1, [arg2 ...]] -> ...",
    data: Vec::new()
  },
  0xb9u8 => CodeByte {
    name: "invokeinterface",
    opcode: 0xb9,
    length: 0x05,
    stack_behavior: "..., objectref, [arg1, [arg2 ...]] -> ...",
    data: Vec::new()
  },
  0xbau8 => CodeByte {
    name: "invokedynamic",
    opcode: 0xba,
    length: 0x05,
    stack_behavior: "..., [arg1, [arg2 ...]] -> ...",
    data: Vec::new()
  },
  0xbbu8 => CodeByte {
    name: "new",
    opcode: 0xbb,
    length: 0x03,
    stack_behavior: "... -> ..., objectref",
    data: Vec::new()
  },
  0xbcu8 => CodeByte {
    name: "newarray",
    opcode: 0xbc,
    length: 0x02,
    stack_behavior: "..., count -> ..., arrayref",
    data: Vec::new()
  },
  0xbdu8 => CodeByte {
    name: "anewarray",
    opcode: 0xbd,
    length: 0x03,
    stack_behavior: "..., count -> ..., arrayref",
    data: Vec::new()
  },
  0xbeu8 => CodeByte {
    name: "arraylength",
    opcode: 0xbe,
    length: 0x01,
    stack_behavior: "..., arrayref -> ..., length",
    data: Vec::new()
  },
  0xbfu8 => CodeByte {
    name: "athrow",
    opcode: 0xbf,
    length: 0x01,
    stack_behavior: "..., objectref -> objectref",
    data: Vec::new()
  },
  0xc0u8 => CodeByte {
    name: "checkcast",
    opcode: 0xc0,
    length: 0x03,
    stack_behavior: "..., objectref -> ..., objectref",
    data: Vec::new()
  },
  0xc1u8 => CodeByte {
    name: "instanceof",
    opcode: 0xc1,
    length: 0x03,
    stack_behavior: "..., objectref -> ..., result",
    data: Vec::new()
  },
  0xc2u8 => CodeByte {
    name: "monitorenter",
    opcode: 0xc2,
    length: 0x01,
    stack_behavior: "..., objectref -> ...",
    data: Vec::new()
  },
  0xc3u8 => CodeByte {
    name: "monitorexit",
    opcode: 0xc3,
    length: 0x01,
    stack_behavior: "..., objectref -> ...",
    data: Vec::new()
  },
  0xc4u8 => CodeByte { // TODO: wide命令の長さは可変(4バイトまたは6バイト)
    name: "wide",
    opcode: 0xc4,
    length: 0x04,
    stack_behavior: "Same as modified instruction",
    data: Vec::new()
  },
  0xc5u8 => CodeByte {
    name: "multianewarray",
    opcode: 0xc5,
    length: 0x04,
    stack_behavior: "..., count1, [count2, ...] -> ..., arrayref",
    data: Vec::new()
  },
  0xc6u8 => CodeByte {
    name: "ifnull",
    opcode: 0xc6,
    length: 0x03,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0xc7u8 => CodeByte {
    name: "ifnonnull",
    opcode: 0xc7,
    length: 0x03,
    stack_behavior: "..., value -> ...",
    data: Vec::new()
  },
  0xc8u8 => CodeByte {
    name: "goto_w",
    opcode: 0xc8,
    length: 0x05,
    stack_behavior: "No change",
    data: Vec::new()
  },
  0xc9u8 => CodeByte {
    name: "jsr_w",
    opcode: 0xc9,
    length: 0x05,
    stack_behavior: "... -> ..., address",
    data: Vec::new()
  },
  0xcau8 => CodeByte {
    name: "breakpoint",
    opcode: 0xca,
    length: 0x01,
    stack_behavior: "No change",
    data: Vec::new()
  },
  0xfeu8 => CodeByte {
    name: "impdep1",
    opcode: 0xfe,
    length: 0x01,
    stack_behavior: "No change",
    data: Vec::new()
  },
  0xffu8 => CodeByte {
    name: "impdep2",
    opcode: 0xff,
    length: 0x01,
    stack_behavior: "No change",
    data: Vec::new()
  },
};