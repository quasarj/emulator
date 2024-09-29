use crate::registers::{FlagsRegister, Registers};

#[derive(Debug)]
pub enum Instruction {
    HALT,
    NOP,

    ADD(Target),
    ADDHL(Target),
    ADC(Target),
    SUB(Target),
    SBC(Target),
    AND(Target),
    OR(Target),
    XOR(Target),
    CP(Target),

    INC(Target),
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => Some(Instruction::NOP),
            0x03 => Some(Instruction::INC(Target::BC)),
            0x04 => Some(Instruction::INC(Target::B)),
            0x13 => Some(Instruction::INC(Target::DE)),
            0x76 => Some(Instruction::HALT),
            0x81 => Some(Instruction::ADD(Target::C)),
            _ => None,
        }
    }

    // TODO: do we really want ot do this? it will be awful and probably 
    // not really used. Think hard about it!
    pub fn to_byte(instruction: Instruction) -> Option<u8> {
        match instruction {
            Instruction::NOP => Some(0x00),
            Instruction::INC(Target::BC) => Some(0x03),
            Instruction::INC(Target::B) => Some(0x04),
            Instruction::INC(Target::DE) => Some(0x13),
            Instruction::HALT => Some(0x76),
            Instruction::ADD(Target::C) => Some(0x81),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Target {
    A, B, C, D, E, F, H, L, BC, DE, D8, HLI
}

#[derive(Debug)]
pub struct MemoryBus {
    pub memory: [u8; 0xFFFF],
}

impl MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    fn new() -> Self {
        MemoryBus {
            memory: [0; 0xFFFF],
        }
    }
}

#[derive(Debug)]
pub struct CPU {
    pub halt: bool,
    pub registers: Registers,
    pub pc: u16,
    pub bus: MemoryBus,
}

impl CPU {
    pub fn run(&mut self) {
        loop {
            println!("PC: {}", self.pc);
            self.step();

            if self.halt {
                break;
            }
        }
    }
    pub fn step(&mut self) {
        let mut instruction_byte = self.bus.read_byte(self.pc);

        let next_pc = if let Some(instruction) = Instruction::from_byte(instruction_byte) {
            println!("{:?}", instruction);
            self.execute(instruction)
        } else {
            panic!("don't know what to do")
        };

        self.pc = next_pc;
    }
    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::HALT => {
                self.halt = true;
                0
            }
            Instruction::NOP => self.pc.wrapping_add(1),
            Instruction::ADD(target) => {
                match target {
                    Target::C => {
                        let value = self.registers.c;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                        self.pc.wrapping_add(1)
                    }
                    _ => {
                        /* TODO: support more targets */
                        self.pc
                    }
                }
            },
            Instruction::INC(target) => {
                match target {
                    Target::BC => {
                        let mut val = self.registers.get_bc();
                        self.registers.set_bc(val.wrapping_add(1));
                        self.pc.wrapping_add(1)
                    },
                    Target::DE => {
                        let mut val = self.registers.get_de();
                        self.registers.set_de(val.wrapping_add(1));
                        self.pc.wrapping_add(1)
                    },
                    Target::B => {
                        self.registers.b = self.registers.b.wrapping_add(1);
                        self.pc.wrapping_add(1)
                    },
                    Target::D => {
                        self.registers.d = self.registers.d.wrapping_add(1);
                        self.pc.wrapping_add(1)
                    },
                    _ => { panic!("can't inc {:?}", target) }
                }
            },
            _ => {
                /* TODO: support more instructions */
                self.pc
            }
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        // overflowing_add is a rust built-in on int types
        let (new_value, did_overflow) = self.registers.a.overflowing_add(value);
        self.registers.f.zero = new_value == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
        new_value
    }

    pub fn new() -> Self {
        CPU {
            halt: false,
            registers: Registers::new(),
            pc: 0,
            bus: MemoryBus::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    // Bring the outer module's code into scope
    use super::*;

    macro_rules! assert_registers {
        ($val:expr) => {
            assert_eq!($val.f.carry, true, "Carry should be true");
            assert_eq!($val.f.subtract, false, "Subtract should be true");
            assert_eq!($val.f.zero, false, "Zero should be true");
            assert_eq!($val.f.half_carry, true, "Half-carry should be true");
        };
    }

    macro_rules! assert_flags_false {
        ($flags:expr) => {
            assert_eq!($flags.carry, false, "Carry should be false");
            assert_eq!($flags.subtract, false, "Subtract should be false");
            assert_eq!($flags.zero, false, "Zero should be false");
            assert_eq!($flags.half_carry, false, "Half-carry should be false");
        };
    }

    #[test]
    fn test_add() {
        let mut cpu = CPU::new();

        cpu.registers.a = 255;
        cpu.registers.c = 4;
        cpu.execute(Instruction::ADD(Target::C));

        assert_eq!(cpu.registers.a, 3); // because it overflowed
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.f.subtract, false);
        assert_eq!(cpu.registers.f.zero, false);
        assert_eq!(cpu.registers.f.half_carry, true);
    }

    #[test]
    fn test_inc_bc() {
        let mut cpu = CPU::new();

        cpu.registers.b = 0;
        cpu.registers.c = 0xFF; // the max value
        cpu.execute(Instruction::INC(Target::BC));

        assert_eq!(cpu.registers.b, 1, "B should be one higher");
        assert_eq!(cpu.registers.c, 0, "C should wrap to 0");

        assert_flags_false!(cpu.registers.f);
    }

    #[test]
    fn test_inc_de() {
        let mut cpu = CPU::new();

        cpu.registers.d = 0;
        cpu.registers.e = 0xFF; // the max value
        cpu.execute(Instruction::INC(Target::DE));

        assert_eq!(cpu.registers.d, 1, "B should be one higher");
        assert_eq!(cpu.registers.e, 0, "C should wrap to 0");

        assert_flags_false!(cpu.registers.f);
    }

    #[test]
    fn test_inc_singles() {
        let mut cpu = CPU::new();

        cpu.registers.b = 0;
        cpu.registers.d = 0xFF; // the max value
        cpu.execute(Instruction::INC(Target::B));
        cpu.execute(Instruction::INC(Target::D));

        assert_eq!(cpu.registers.b, 1, "B should be one higher");
        assert_eq!(cpu.registers.d, 0, "D should wrap to 0");

        assert_flags_false!(cpu.registers.f);
    }

    #[test]
    #[should_panic]
    fn test_inc_bad() {
        let mut cpu = CPU::new();

        cpu.execute(Instruction::INC(Target::A));
    }

    #[test]
    fn test_instruction_conversion() {
        let mut cpu = CPU::new();

        let test = 0x01;
    }

    // #[test]
    // fn test_execution() {
    //     let mut cpu = CPU::new();

    //     // println!("{:?}", cpu.pc);
    //     cpu.step();
    // }
}
