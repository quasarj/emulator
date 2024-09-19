use crate::registers::{FlagsRegister, Registers};

#[derive(Debug)]
enum Instruction {
    HALT,
    NOP,

    ADD(ArithmeticTarget),
    ADDHL(ArithmeticTarget),
    ADC(ArithmeticTarget),
    SUB(ArithmeticTarget),
    SBC(ArithmeticTarget),
    AND(ArithmeticTarget),
    OR(ArithmeticTarget),
    XOR(ArithmeticTarget),
    CP(ArithmeticTarget),

    INC(IncDecTarget),
}

impl Instruction {
    fn from_byte(byte: u8) -> Option<Instruction> {
        match byte {
            0x00 => Some(Instruction::NOP),
            0x02 => Some(Instruction::INC(IncDecTarget::BC)),
            0x13 => Some(Instruction::INC(IncDecTarget::DE)),
            0x76 => Some(Instruction::HALT),
            0x81 => Some(Instruction::ADD(ArithmeticTarget::C)),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}
#[derive(Debug)]
enum IncDecTarget {
    BC,
    DE,
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
                    ArithmeticTarget::C => {
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
            }
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

    #[test]
    fn test_add() {
        let mut cpu = CPU::new();

        cpu.registers.a = 255;
        cpu.registers.c = 4;
        cpu.execute(Instruction::ADD(ArithmeticTarget::C));

        assert_eq!(cpu.registers.a, 3); // because it overflowed
        assert_eq!(cpu.registers.f.carry, true);
        assert_eq!(cpu.registers.f.subtract, false);
        assert_eq!(cpu.registers.f.zero, false);
        assert_eq!(cpu.registers.f.half_carry, true);
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
