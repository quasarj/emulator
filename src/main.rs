mod cpu;
mod registers;

fn main() {
    let mut cpu = cpu::CPU::new();

    // memory starts all 0, which is NOP

    // ADD(C)
    cpu.bus.memory[1] = 0x81;

    // add HALT at address 5
    cpu.bus.memory[5] = 0x76;

    // setup registers
    cpu.registers.a = 1;
    cpu.registers.c = 7;

    println!("Starting registers state: {:?}", cpu.registers);
    cpu.run(); // runs forever, or until halt
    println!("Final registers state: {:?}", cpu.registers);
}

#[cfg(test)]
mod tests {
    // Bring the outer module's code into scope
    use super::*;

    // #[test]
    // fn test_instruction() {
    // }
}
