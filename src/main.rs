use anyhow::Context;
use std::fs;

const DATA_TRANSFER_MOV: [(&'static str, u8); 1] = [("mov", 0b100010)];

#[derive(Debug, PartialEq)]
enum MovInstructionTypes {
    RegisterOrMemoryToOrFromRegister = 0b10001000,
    RegisterOrMemoryToOrFromSegmentRegister = 0b10001100,
    ImmediateToRegisterOrMemory = 0b11000110,
    ImmediateToRegister = 0b10110000,
    MemoryToAccumulator = 0b10100000,
    AccumulatorToMemory = 0b10100010,
}

impl MovInstructionTypes {

    fn find_instruction(byte: u8) -> Self {
        match byte {
            byte if (byte & Self::RegisterOrMemoryToOrFromRegister as u8)
                == Self::RegisterOrMemoryToOrFromRegister as u8 =>
            {
                Self::RegisterOrMemoryToOrFromRegister
            }
            byte if (byte & Self::ImmediateToRegister as u8) == Self::ImmediateToRegister as u8 => {
                Self::ImmediateToRegister
            }
            byte if (byte & Self::ImmediateToRegisterOrMemory as u8)
                == Self::ImmediateToRegisterOrMemory as u8 =>
            {
                Self::ImmediateToRegisterOrMemory
            }
            byte if (byte & Self::ImmediateToRegister as u8) == Self::ImmediateToRegister as u8 => {
                Self::ImmediateToRegister
            }
            byte if (byte & Self::MemoryToAccumulator as u8) == Self::MemoryToAccumulator as u8 => {
                Self::MemoryToAccumulator
            }
            byte if (byte & Self::AccumulatorToMemory as u8) == Self::AccumulatorToMemory as u8 => {
                Self::AccumulatorToMemory
            }
            _ => panic!("Unable to determine instruction for byte {:08b}", byte),
        }
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
    let mut result: Vec<String> = vec![];
    let mut i = 0;
    let chunk_size = 2;
    // println!("first chunk size: {:08b}", data[0]);
    data.iter().for_each(|byte| {
        println!("first chunk size: {:08b}", byte);
    });
    
    while i < data.len() {
        // TODO: fix chunk logic;
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

            let instruction = match MovInstructionTypes::find_instruction(chunk[0]) {
                MovInstructionTypes::RegisterOrMemoryToOrFromRegister => "mov",
                MovInstructionTypes::RegisterOrMemoryToOrFromSegmentRegister => "mov",
                MovInstructionTypes::ImmediateToRegisterOrMemory => "mov",
                MovInstructionTypes::ImmediateToRegister => "mov",
                MovInstructionTypes::MemoryToAccumulator => "mov",
                MovInstructionTypes::AccumulatorToMemory => "mov",
            };

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
                        "{instruction} {}, {}",
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
    let bin_file: Vec<u8> = fs::read("listing_0039_more_mov")
        .context("Failed to open listing_0039_more_mov.asm")?;

    // println!("length {:#?}", bin_file.len());
    // bin_file.iter().for_each(|byte| {
    //     println!("{:08b}", byte);
    // });
    let result = disassemble_binary(&bin_file);
    // println!("{}", result);
    // println!("{:08b}", 0b100010);
    // println!("{:08b}", 0b100010 << 2);
    // println!("{:08b}", (0b100010 << 2) & 0b10001000);
    // println!("{:08b}", (0b10110000) & 0b10001000);
    // println!("{}", (0b10110000 & 0b10110000) == 0b10110000);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_single_register_binary() {
        let bin_file: Vec<u8> = fs::read("listing_0037_single_register_mov")
            .context("Failed to open listing_0037_single_register_mov")
            .unwrap();

        let expected_result = r"mov cx, bx";
        assert_eq!(disassemble_binary(&bin_file), expected_result);
    }

    #[test]
    fn test_disassemble_many_register_binary() {
        let bin_file: Vec<u8> = fs::read("listing_0038_many_register_mov")
            .context("Failed to open listing_0038_many_register_mov")
            .unwrap();

        let expected_result = r"mov cx, bx
mov ch, ah
mov dx, bx
mov si, bx
mov bx, di
mov al, cl
mov ch, ch
mov bx, ax
mov bx, si
mov sp, di
mov bp, ax";
        assert_eq!(disassemble_binary(&bin_file), expected_result);
    }
    //     mov cl, 12
    #[test]
    fn test_disassemble_immediate_to_register_binary() {
        let bin_file: Vec<u8> = "mov cx, bx".bytes().collect();
        let bin_file: Vec<u8> = fs::read("listing_0037_single_register_mov")
            .context("Failed to open listing_0037_single_register_mov")
            .unwrap();

        bin_file.iter().for_each(|byte| {
            print!("{:08b}", byte);
        });

        let expected_result = r"mov cx, bx";
        assert_eq!(disassemble_binary(&bin_file), expected_result);
    }
}
