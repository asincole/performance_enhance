use super::*;

mod move_instruction_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::base(0x88)]
    #[case::d_bit_set(0x89)]
    #[case::w_bit_set(0x8A)]
    #[case::both_bits_set(0x8B)]
    fn test_register_or_memory_to_or_from_register(#[case] opcode: u8) {
        assert_eq!(
            MovInstructionType::find_instruction(opcode),
            MovInstructionType::RegisterOrMemoryToOrFromRegister,
            "Failed for opcode: {:#010b}",
            opcode
        );
    }

    #[rstest]
    #[case::byte_operation(0xA0)]
    #[case::word_operation(0xA1)]
    fn test_memory_to_accumulator(#[case] opcode: u8) {
        assert_eq!(
            MovInstructionType::find_instruction(opcode),
            MovInstructionType::MemoryToAccumulator,
            "Failed for opcode: {:#010b}",
            opcode
        );
    }

    #[rstest]
    #[case::byte_operation(0xA2)]
    #[case::word_operation(0xA3)]
    fn test_accumulator_to_memory(#[case] opcode: u8) {
        assert_eq!(
            MovInstructionType::find_instruction(opcode),
            MovInstructionType::AccumulatorToMemory,
            "Failed for opcode: {:#010b}",
            opcode
        );
    }

    #[rstest]
    #[case::al(0xB0)]
    #[case::cl(0xB1)]
    #[case::dl(0xB2)]
    #[case::bl(0xB3)]
    #[case::ah(0xB4)]
    #[case::ch(0xB5)]
    #[case::dh(0xB6)]
    #[case::bh(0xB7)]
    #[case::ax(0xB8)]
    #[case::cx(0xB9)]
    #[case::dx(0xBA)]
    #[case::bx(0xBB)]
    #[case::sp(0xBC)]
    #[case::bp(0xBD)]
    #[case::si(0xBE)]
    #[case::di(0xBF)]
    fn test_immediate_to_register(#[case] opcode: u8) {
        assert_eq!(
            MovInstructionType::find_instruction(opcode),
            MovInstructionType::ImmediateToRegister,
            "Failed for opcode: {:#010b}",
            opcode
        );
    }

    #[rstest]
    #[case::byte_operation(0xC6)]
    #[case::word_operation(0xC7)]
    fn test_immediate_to_register_or_memory(#[case] opcode: u8) {
        assert_eq!(
            MovInstructionType::find_instruction(opcode),
            MovInstructionType::ImmediateToRegisterOrMemory,
            "Failed for opcode: {:#010b}",
            opcode
        );
    }

    #[rstest]
    #[case::byte_operation(0x8E)]
    #[case::word_operation(0x8F)]
    fn test_register_or_memory_to_segment_register(#[case] opcode: u8) {
        assert_eq!(
            MovInstructionType::find_instruction(opcode),
            MovInstructionType::RegisterOrMemoryToSegmentRegister,
            "Failed for opcode: {:#010b}",
            opcode
        );
    }

    #[rstest]
    #[case::byte_operation(0x8C)]
    #[case::word_operation(0x8D)]
    fn test_segment_register_to_register_or_memory(#[case] opcode: u8) {
        assert_eq!(
            MovInstructionType::find_instruction(opcode),
            MovInstructionType::SegmentRegisterToRegisterOrMemory,
            "Failed for opcode: {:#010b}",
            opcode
        );
    }
    
    #[rstest]
    #[case::zero(0x00, "00000000")]
    #[case::one(0x01, "00000001")]
    #[case::arbitrary1(0x10, "00010000")]
    #[case::arbitrary2(0x42, "01000010")]
    #[case::arbitrary3(0xD0, "11010000")]
    #[case::arbitrary4(0xFF, "11111111")]
    #[should_panic(expected = "Unable to determine instruction for byte")]
    fn test_invalid_opcodes(#[case] opcode: u8, #[case] _binary_repr: &str) {
        MovInstructionType::find_instruction(opcode);
    }

    #[test]
    fn test_all_valid_opcodes_return_some() {
        for opcode in 0..=255u8 {
            let expected = match opcode {
                0x88..=0x8B => Some(MovInstructionType::RegisterOrMemoryToOrFromRegister),
                0xA0..=0xA1 => Some(MovInstructionType::MemoryToAccumulator),
                0xA2..=0xA3 => Some(MovInstructionType::AccumulatorToMemory),
                0xB0..=0xBF => Some(MovInstructionType::ImmediateToRegister),
                0xC6..=0xC7 => Some(MovInstructionType::ImmediateToRegisterOrMemory),
                0x8E..=0x8F => Some(MovInstructionType::RegisterOrMemoryToSegmentRegister),
                0x8C..=0x8D => Some(MovInstructionType::SegmentRegisterToRegisterOrMemory),
                _ => None,
            };

            assert_eq!(
                MovInstructionType::from_byte(opcode),
                expected,
                "Incorrect classification for opcode: {:#04X} ({:#010b})",
                opcode,
                opcode
            );
        }
    }
}

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

#[test]
fn test_disassemble_immediate_to_register_binary() {
    let bin_file: Vec<u8> = fs::read("listing_0037_single_register_mov")
        .context("Failed to open listing_0037_single_register_mov")
        .unwrap();

    bin_file.iter().for_each(|byte| {
        print!("{:08b}", byte);
    });

    let expected_result = r"mov cx, bx";
    assert_eq!(disassemble_binary(&bin_file), expected_result);
}
