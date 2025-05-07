use anyhow::Context;
use std::fs;

#[cfg(test)]
mod main_tests;

#[derive(Debug, PartialEq)]
enum MovInstructionType {
    RegisterOrMemoryToOrFromRegister,  // opcode -> 0b100010xx
    ImmediateToRegisterOrMemory,       // opcode -> 0b1100011x
    ImmediateToRegister,               // opcode -> 0b1011xxxx
    MemoryToAccumulator,               // opcode -> 0b1010000x
    AccumulatorToMemory,               // opcode -> 0b1010001x
    RegisterOrMemoryToSegmentRegister, // opcode -> 0b10001110
    SegmentRegisterToRegisterOrMemory, // opcode -> 0b10001100
}

impl MovInstructionType {
    // masks
    const REG_MEM_MASK: u8 = 0xFC; // 11111100
    const ACC_MEM_MASK: u8 = 0xFE; // 11111110
    const IMM_REG_MASK: u8 = 0xF0; // 11110000

    /*
        TODO: add extra benchmark to see if putting the opcode directly in the MovInstructionType
        enum has any performance benefits in lieu of using constants here 
    */
    // opcode patterns
    const REG_MEM_PATTERN: u8 = 0x88; // 10001000
    const MEM_ACC_PATTERN: u8 = 0xA0; // 10100000
    const ACC_MEM_PATTERN: u8 = 0xA2; // 10100010
    const IMM_REG_PATTERN: u8 = 0xB0; // 10110000
    const IMM_MEM_PATTERN: u8 = 0xC6; // 11000110
    const MEM_SEG_PATTERN: u8 = 0x8E; // 10001110
    const SEG_MEM_PATTERN: u8 = 0x8C; // 10001100

    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b if (b & Self::REG_MEM_MASK) == Self::REG_MEM_PATTERN => {
                Some(Self::RegisterOrMemoryToOrFromRegister)
            }

            b if (b & Self::ACC_MEM_MASK) == Self::MEM_ACC_PATTERN => {
                Some(Self::MemoryToAccumulator)
            }

            b if (b & Self::ACC_MEM_MASK) == Self::ACC_MEM_PATTERN => {
                Some(Self::AccumulatorToMemory)
            }

            b if (b & Self::IMM_REG_MASK) == Self::IMM_REG_PATTERN => {
                Some(Self::ImmediateToRegister)
            }

            b if (b & Self::ACC_MEM_MASK) == Self::IMM_MEM_PATTERN => {
                Some(Self::ImmediateToRegisterOrMemory)
            }

            b if (b & Self::ACC_MEM_MASK) == Self::MEM_SEG_PATTERN => {
                Some(Self::RegisterOrMemoryToSegmentRegister)
            }

            b if (b & Self::ACC_MEM_MASK) == Self::SEG_MEM_PATTERN => {
                Some(Self::SegmentRegisterToRegisterOrMemory)
            }

            _ => None,
        }
    }

    fn find_instruction(byte: u8) -> Self {
        Self::from_byte(byte)
            .unwrap_or_else(|| panic!("Unable to determine instruction for byte {:08b}", byte))
    }
}

// 6 bits are opcode (mov), 2 bits (d, w)
// second byte 2 bits (mod), 3 (reg) 3 (R/M)
enum ModEncoding {
    /// no displacement
    MemMode = 0b00,
    MemMode8B = 0b01,
    MemMode16B = 0b10,
    RegisterMode = 0b11,
}

impl ModEncoding {
    fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            x if x == ModEncoding::MemMode as u8 => Some(Self::MemMode),
            x if x == ModEncoding::MemMode8B as u8 => Some(Self::MemMode8B),
            x if x == ModEncoding::MemMode16B as u8 => Some(Self::MemMode16B),
            x if x == ModEncoding::RegisterMode as u8 => Some(Self::RegisterMode),
            _ => None,
        }
    }
}

// w0
enum RegisterByteOp {
    AL = 0b000,
    CL = 0b001,
    DL = 0b010,
    BL = 0b011,
    AH = 0b100,
    Ch = 0b101,
    DH = 0b110,
    BH = 0b111,
}

// w1
enum RegisterWordOp {
    // RegisterWordOp
    AX = 0b000,
    CX = 0b001,
    DX = 0b010,
    BX = 0b011,
    SP = 0b100,
    BP = 0b101,
    SI = 0b110,
    DI = 0b111,
}

enum RegisterOp {
    // RegisterByteOp=0b0,
    // RegisterWordOp=0b1,
}

impl RegisterOp {
    fn from_bits(w: u8, bits: u8) -> Option<&'static str> {
        match (w, bits) {
            (0, x) if x == RegisterByteOp::AL as u8 => Some("AL"),
            (0, x) if x == RegisterByteOp::CL as u8 => Some("CL"),
            (0, x) if x == RegisterByteOp::DL as u8 => Some("DL"),
            (0, x) if x == RegisterByteOp::BL as u8 => Some("BL"),
            (0, x) if x == RegisterByteOp::AH as u8 => Some("AH"),
            (0, x) if x == RegisterByteOp::Ch as u8 => Some("Ch"),
            (0, x) if x == RegisterByteOp::DH as u8 => Some("DH"),
            (0, x) if x == RegisterByteOp::BH as u8 => Some("BH"),

            (1, x) if x == RegisterWordOp::AX as u8 => Some("AX"),
            (1, x) if x == RegisterWordOp::CX as u8 => Some("CX"),
            (1, x) if x == RegisterWordOp::DX as u8 => Some("DX"),
            (1, x) if x == RegisterWordOp::BX as u8 => Some("BX"),
            (1, x) if x == RegisterWordOp::SP as u8 => Some("SP"),
            (1, x) if x == RegisterWordOp::BP as u8 => Some("BP"),
            (1, x) if x == RegisterWordOp::SI as u8 => Some("SI"),
            (1, x) if x == RegisterWordOp::DI as u8 => Some("DI"),
            _ => None,
        }
    }
}

fn extract_bits(byte: u8, start: u8, end: u8) -> u8 {
    // Validate inputs
    assert!(start < end, "Start must be less than the end");
    assert!(end <= 8, "The End cannot be greater than 8");

    // Calculate the number of bits to extract
    let num_bits = end - start;

    // Shift left to align desired bits, then shift right
    (byte << start) >> (8 - num_bits)
}
fn disassemble_binary(data: &[u8]) -> String {
    /*
    TODO: if program doesn't panic, the number of ops should be roughly <= size of data slice passed
    into the function, consider checking if creating a vector with with_capacity is simply a better approach
    */
    let mut result: Vec<String> = vec![];
    let mut i = 0;
    // TODO: fix chunk logic, depending on the opcode, you will need to pull in more bytes down the line;
    let chunk_size = 2;
    // println!("first chunk size: {:08b}", data[0]);

    while i < data.len() {
        // TODO: fix chunk logic, depending on the opcode, you will need to pull in more bytes down the line;
        let chunk = &data[i..i + chunk_size];
        if chunk.len() == 2 {
            // Combine 2 bytes into a 16-bit value
            // let value = u16::from_be_bytes([chunk[0], chunk[1]]);
            // println!("16-bit value: {:016b}", value);

            let op_code = extract_bits(chunk[0], 0, 6);
            let d = extract_bits(chunk[0], 6, 7);
            let w = extract_bits(chunk[0], 7, 8);
            let mode = extract_bits(chunk[1], 0, 2);
            let reg = extract_bits(chunk[1], 2, 5);
            let r_m = extract_bits(chunk[1], 5, 8);
            // println!("first byte {:08b}", chunk[0]);
            // println!("second byte {:08b}", chunk[1]);

            // TODO: use the instruction to determine how to handle following bytes
            let _instruction = MovInstructionType::find_instruction(chunk[0]);

            // TODO: improve this as we progress in the course
            let (source, mut destination) = match d {
                // direction is from register (i.e. the data source is from a register)
                0 => (RegisterOp::from_bits(w, reg), None),
                // direction is to register (i.e. the data destination is to a register)
                1 => (None, RegisterOp::from_bits(w, reg)),
                _ => panic!("Direction is not supported"),
            };

            match ModEncoding::from_bits(mode) {
                Some(m) => match m {
                    ModEncoding::RegisterMode => {
                        destination = RegisterOp::from_bits(w, r_m);
                    }
                    _ => panic!("not supported yet"),
                },
                _ => panic!("Invalid mode"),
            }

            match (destination, source) {
                (Some(destination), Some(source)) => {
                    let operation = format!(
                        "mov {}, {}",
                        destination.to_lowercase(),
                        source.to_lowercase()
                    );
                    result.push(operation);
                }
                _ => panic!("oh shit"),
            }
        }
        i += chunk_size;
    }

    result.join("\n")
}

fn main() -> anyhow::Result<()> {
    let bin_file: Vec<u8> =
        fs::read("listing_0039_more_mov").context("Failed to open listing_0039_more_mov.asm")?;

    let _result = disassemble_binary(&bin_file);
    Ok(())
}
