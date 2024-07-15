#![allow(missing_docs)]

/// Byte opcodes
pub mod opcode;
/// Byte columns    
pub mod columns;
/// Byte stark
pub mod byte_stark;
/// Byte event
pub mod event;

/// Number of rows in the byte trace
// for debug
pub const NUM_ROWS: usize = 1 << 16;
// pub const NUM_ROWS: usize = 16;