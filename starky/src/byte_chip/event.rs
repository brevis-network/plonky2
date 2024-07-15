use crate::byte_chip::opcode::ByteOpcode;

/// byte level event as a structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteLookupEvent {
    /// byte level opcode
    pub opcode: ByteOpcode,

    /// output operand
    pub a: u32,

    /// input operand
    pub b: u32,

    /// input operand
    pub c: u32,
}

/// Implementation of ByteLookupEvent
impl ByteLookupEvent {
    /// create a new byte lookup event
    pub fn new(opcode: ByteOpcode, a: u32, b: u32, c: u32) -> Self {
        Self { opcode, a, b, c }
    }

    pub fn get_opcode(&self) -> ByteOpcode {
        self.opcode
    }
}