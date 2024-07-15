use crate::byte_chip::opcode::NUM_BYTE_OP_COLS;

/// Number of columns in the byte chip  
pub const NUM_BYTE_COLS: usize = 3 * NUM_BYTE_OP_COLS + 2;

/// Index of the opcode column
const OPCODE_INDEX: usize = 0;

/// Index of the multiplicities column
const MULTIPLICITIES_INDEX: usize = NUM_BYTE_OP_COLS;

/// Index of the filter column
const FILTER_INDEX: usize = 2 * NUM_BYTE_OP_COLS;

/// Index of the b column
const B_INDEX: usize = NUM_BYTE_COLS - 2;

/// Index of the c column
const C_INDEX: usize = NUM_BYTE_COLS - 1;

/// get the index of the opcode column
pub fn opcode_index(i: usize) -> usize {
    OPCODE_INDEX + i
}

/// get the index of the multiplicities column
pub fn multiplicities_index(i: usize) -> usize {
    MULTIPLICITIES_INDEX + i
}

/// get the index of the filter column
pub fn filter_index(i: usize) -> usize {
    FILTER_INDEX + i
}

/// get the index of the b column
pub fn b_index() -> usize {
    B_INDEX
}

/// get the index of the c column
pub fn c_index() -> usize {
    C_INDEX
}

