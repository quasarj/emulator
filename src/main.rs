mod cpu;
mod registers;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut cpu = cpu::CPU::new();

    // memory starts all 0, which is NOP

    cpu.bus.memory[1] = 0x81; // ADD(C)

    cpu.bus.memory[2] = 0x04; // INC(B)

    cpu.bus.memory[10] = 0x76; // HALT

    // setup registers
    cpu.registers.a = 1;
    cpu.registers.c = 7;

    println!("Starting registers state: {:?}", cpu.registers);
    cpu.run(); // runs forever, or until halt
    println!("Final registers state: {:?}", cpu.registers);

    Ok(())
}

#[cfg(test)]
mod tests {
    // Bring the outer module's code into scope
    use super::*;


    // #[test]
    // fn test_things() {
    //     let a = LoadType::Byte(RegisterTarget::A, RegisterTarget::B);

    // }
}
