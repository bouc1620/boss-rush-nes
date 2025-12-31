#[cfg(feature = "debug")]
mod util;

#[cfg(feature = "debug")]
use crate::util::debug::disassemble;
#[cfg(feature = "debug")]
use std::io::stdin;

pub mod bus;
pub mod cpu;
pub mod instructions;

use crate::bus::Bus;
use crate::cpu::CPU;

fn main() {
    let mut bus = Bus::default();
    let mut cpu = CPU::default();

    /*
        *=$8000
        LDX #10
        STX $0000
        LDX #3
        STX $0001
        LDY $0000
        LDA #0
        CLC
        loop
        ADC $0001
        DEY
        BNE loop
        STA $0002
        NOP
        NOP
        NOP
    */
    bus.load_program(
        "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA",
    );
    bus.set_reset_vector();

    cpu.reset(&mut bus);

    loop {
        let lines = disassemble(&bus, 0x8000, 0x801F);
        let mut sorted_lines: Vec<_> = lines.iter().collect();
        sorted_lines.sort_by_key(|(addr, _)| *addr);
        for (_addr, line) in sorted_lines {
            println!("{}", line);
        }

        cpu.print_state();
        bus.print_ram(0x0000, 0x001F);
        bus.print_ram(0x8000, 0x801F);

        println!("Press Enter to step, q to quit...");
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        if input.trim() == "q" {
            break;
        }

        cpu.step_to_next_instruction(&mut bus);
    }
}
