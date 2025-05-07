use criterion::{black_box, criterion_group, criterion_main, Criterion};

mod approach1 {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum MovInstructionType {
        RegisterOrMemoryToOrFromRegister,
        ImmediateToRegisterOrMemory,
        ImmediateToRegister,
        MemoryToAccumulator,
        AccumulatorToMemory,
        RegisterOrMemoryToSegmentRegister,
        SegmentRegisterToRegisterOrMemory,
    }

    impl MovInstructionType {
        pub fn from_byte(byte: u8) -> Option<Self> {
            match (byte & 0xFC, byte & 0xFE, byte & 0xF8) {
                (0x88, _, _) => Some(Self::RegisterOrMemoryToOrFromRegister),
                (_, 0xA0, _) => Some(Self::MemoryToAccumulator),
                (_, 0xA2, _) => Some(Self::AccumulatorToMemory),
                (_, _, 0xB0) => Some(Self::ImmediateToRegister),
                (_, 0xC6, _) => Some(Self::ImmediateToRegisterOrMemory),
                (_, 0x8E, _) => Some(Self::RegisterOrMemoryToSegmentRegister),
                (_, 0x8C, _) => Some(Self::SegmentRegisterToRegisterOrMemory),
                _ => None,
            }
        }

        pub fn find_instruction(byte: u8) -> Self {
            Self::from_byte(byte)
                .unwrap_or_else(|| panic!("Unable to determine instruction for byte {:08b}", byte))
        }
    }
}

mod approach2 {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum MovInstructionType {
        RegisterOrMemoryToOrFromRegister,
        ImmediateToRegisterOrMemory,
        ImmediateToRegister,
        MemoryToAccumulator,
        AccumulatorToMemory,
        RegisterOrMemoryToSegmentRegister,
        SegmentRegisterToRegisterOrMemory,
    }

    impl MovInstructionType {
        // Define masks
        const REG_MEM_MASK: u8 = 0xFC;     // 11111100
        const ACC_MEM_MASK: u8 = 0xFE;     // 11111110
        const IMM_REG_MASK: u8 = 0xF8;     // 11111000

        // Define patterns
        const REG_MEM_PATTERN: u8 = 0x88;  // 10001000
        const MEM_ACC_PATTERN: u8 = 0xA0;  // 10100000
        const ACC_MEM_PATTERN: u8 = 0xA2;  // 10100010
        const IMM_REG_PATTERN: u8 = 0xB0;  // 10110000
        const IMM_MEM_PATTERN: u8 = 0xC6;  // 11000110
        const MEM_SEG_PATTERN: u8 = 0x8E;  // 10001110
        const SEG_MEM_PATTERN: u8 = 0x8C;  // 10001100

        pub fn from_byte(byte: u8) -> Option<Self> {
            match byte {
                b if (b & Self::REG_MEM_MASK) == Self::REG_MEM_PATTERN =>
                    Some(Self::RegisterOrMemoryToOrFromRegister),

                b if (b & Self::ACC_MEM_MASK) == Self::MEM_ACC_PATTERN =>
                    Some(Self::MemoryToAccumulator),

                b if (b & Self::ACC_MEM_MASK) == Self::ACC_MEM_PATTERN =>
                    Some(Self::AccumulatorToMemory),

                b if (b & Self::IMM_REG_MASK) == Self::IMM_REG_PATTERN =>
                    Some(Self::ImmediateToRegister),

                b if (b & Self::ACC_MEM_MASK) == Self::IMM_MEM_PATTERN =>
                    Some(Self::ImmediateToRegisterOrMemory),

                b if (b & Self::ACC_MEM_MASK) == Self::MEM_SEG_PATTERN =>
                    Some(Self::RegisterOrMemoryToSegmentRegister),

                b if (b & Self::ACC_MEM_MASK) == Self::SEG_MEM_PATTERN =>
                    Some(Self::SegmentRegisterToRegisterOrMemory),

                _ => None,
            }
        }

        pub fn find_instruction(byte: u8) -> Self {
            Self::from_byte(byte)
                .unwrap_or_else(|| panic!("Unable to determine instruction for byte {:08b}", byte))
        }
    }
}

mod approach3 {
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum MovInstructionType {
        RegisterOrMemoryToOrFromRegister,
        ImmediateToRegisterOrMemory,
        ImmediateToRegister,
        MemoryToAccumulator,
        AccumulatorToMemory,
        RegisterOrMemoryToSegmentRegister,
        SegmentRegisterToRegisterOrMemory,
    }

    impl MovInstructionType {
        pub fn from_byte(byte: u8) -> Option<Self> {
            if (byte & 0xFC) == 0x88 {
                return Some(Self::RegisterOrMemoryToOrFromRegister);
            }

            if (byte & 0xFE) == 0xA0 {
                return Some(Self::MemoryToAccumulator);
            }

            if (byte & 0xFE) == 0xA2 {
                return Some(Self::AccumulatorToMemory);
            }

            if (byte & 0xF8) == 0xB0 {
                return Some(Self::ImmediateToRegister);
            }

            if (byte & 0xFE) == 0xC6 {
                return Some(Self::ImmediateToRegisterOrMemory);
            }

            if (byte & 0xFE) == 0x8E {
                return Some(Self::RegisterOrMemoryToSegmentRegister);
            }

            if (byte & 0xFE) == 0x8C {
                return Some(Self::SegmentRegisterToRegisterOrMemory);
            }

            None
        }

        pub fn find_instruction(byte: u8) -> Self {
            Self::from_byte(byte)
                .unwrap_or_else(|| panic!("Unable to determine instruction for byte {:08b}", byte))
        }
    }
}

fn bench_instruction_decoding(c: &mut Criterion) {
    // test bytes covering all instruction types
    let test_bytes = [
        0x88, 0x89, 0x8A, 0x8B,  // RegisterOrMemoryToOrFromRegister
        0xA0, 0xA1,              // MemoryToAccumulator
        0xA2, 0xA3,              // AccumulatorToMemory
        0xB0, 0xB1, 0xB7, 0xBF,  // ImmediateToRegister
        0xC6, 0xC7,              // ImmediateToRegisterOrMemory
        0x8E, 0x8F,              // RegisterOrMemoryToSegmentRegister
        0x8C, 0x8D,              // SegmentRegisterToRegisterOrMemory
    ];

    let mut group = c.benchmark_group("Instruction Decoding");

    group.bench_function("Approach 1: Match with tuple", |b| {
        b.iter(|| {
            for &byte in &test_bytes {
                black_box(approach1::MovInstructionType::from_byte(black_box(byte)));
            }
        })
    });

    group.bench_function("Approach 2: Match with constants", |b| {
        b.iter(|| {
            for &byte in &test_bytes {
                black_box(approach2::MovInstructionType::from_byte(black_box(byte)));
            }
        })
    });

    group.bench_function("Approach 3: If-else chain", |b| {
        b.iter(|| {
            for &byte in &test_bytes {
                black_box(approach3::MovInstructionType::from_byte(black_box(byte)));
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_instruction_decoding);
criterion_main!(benches);
