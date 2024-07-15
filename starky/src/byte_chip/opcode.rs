/// Number of byte opcodes supported
pub const NUM_BYTE_OPS: usize = 5;
/// Number of columns in the trace table that each byte opcode accounts for.
/// Currently includes AND, OR, XOR, LTU.
pub const NUM_BYTE_OP_COLS: usize = 4;

/// byte level opcode
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ByteOpcode {
    /// This part of byte opcodes each accounts for one column in the trace table.
    /// Bitwise AND
    AND = 0,

    /// Bitwise OR. This accounts for one column in the trace table.
    OR = 1,

    /// Bitwise XOR. This accounts for one column in the trace table.
    XOR = 2,

    /// Byte less than or equal to unsigned. This accounts for one column in the trace table.
    LEU = 3,

    /// This part of byte opcodes does not account for any columns in the trace table.
    /// U8 Range check.
    U8Range = 4,
}

/// All byte opcodes listed
pub fn all_byte_opcodes() -> Vec<ByteOpcode> {
    vec![
        ByteOpcode::AND,
        ByteOpcode::OR,
        ByteOpcode::XOR,
        ByteOpcode::LEU,
        ByteOpcode::U8Range,
    ]
}