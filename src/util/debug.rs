use colored::Colorize;
use std::collections::BTreeMap;
use std::io::stdin;

use crate::bus::Bus;
use crate::cpu::CPU;
use crate::instructions::{AddrMode, Instruction, get_instruction};

pub fn debug(program: &str) {
    let mut bus = Bus::default();
    let mut cpu = CPU::default();

    bus.load_program(program);
    bus.set_reset_vector();

    cpu.reset(&mut bus);

    loop {
        cpu.step_to_next_instruction(&mut bus);

        let state = cpu.get_state();

        let lines = disassemble(&bus, state.pc - 5, state.pc + 5);
    }
}

fn disassemble(bus: &Bus, start: u16, end: u16) -> BTreeMap<u16, String> {
    let mut addr = start;

    let mut lines: BTreeMap<u16, String> = BTreeMap::new();

    while addr <= end {
        let line_addr = addr;

        let mut instruction_str = format!("${:04X}: ", addr);

        let opcode = bus.read(addr, true);
        addr += 1;

        let Instruction { name, mode, .. } = get_instruction(opcode);

        instruction_str = format!("{}{} ", instruction_str, name.to_uppercase());

        match mode {
            AddrMode::Imp => {
                instruction_str = format!("{}{}", instruction_str, "{IMP}");
            }
            AddrMode::Imm => {
                let value = bus.read(addr, true);
                addr += 1;

                instruction_str = format!("{}#${:02X} {}", instruction_str, value, "{IMM}");
            }
            AddrMode::Zp0 => {
                let lo = bus.read(addr, true);
                addr += 1;

                instruction_str = format!("{}${:02X} {}", instruction_str, lo, "{ZP0}");
            }
            AddrMode::Zpx => {
                let lo = bus.read(addr, true);
                addr += 1;

                instruction_str = format!("{}${:02X}, X {}", instruction_str, lo, "{ZPX}");
            }
            AddrMode::Zpy => {
                let lo = bus.read(addr, true);
                addr += 1;

                instruction_str = format!("{}${:02X}, Y {}", instruction_str, lo, "{ZPY}");
            }
            AddrMode::Rel => {
                let value = bus.read(addr, true);
                addr += 1;

                instruction_str = format!(
                    "{}${:02X} [${:04X}] {}",
                    instruction_str,
                    value,
                    addr.wrapping_add(value as u16),
                    "{REL}"
                );
            }
            AddrMode::Abs => {
                let lo = bus.read(addr, true) as u16;
                addr += 1;
                let hi = bus.read(addr, true) as u16;
                addr += 1;

                instruction_str = format!("{}${:04X} {}", instruction_str, (hi << 8) | lo, "{ABS}");
            }
            AddrMode::Abx => {
                let lo = bus.read(addr, true) as u16;
                addr += 1;
                let hi = bus.read(addr, true) as u16;
                addr += 1;

                instruction_str =
                    format!("{}${:04X}, X {}", instruction_str, (hi << 8) | lo, "{ABX}");
            }
            AddrMode::Aby => {
                let lo = bus.read(addr, true) as u16;
                addr += 1;
                let hi = bus.read(addr, true) as u16;
                addr += 1;

                instruction_str =
                    format!("{}${:04X}, Y {}", instruction_str, (hi << 8) | lo, "{ABY}");
            }
            AddrMode::Ind => {
                let lo = bus.read(addr, true) as u16;
                addr += 1;
                let hi = bus.read(addr, true) as u16;
                addr += 1;

                instruction_str =
                    format!("{} (${:04X}) {}", instruction_str, (hi << 8) | lo, "{IND}");
            }
            AddrMode::Izx => {
                let lo = bus.read(addr, true);
                addr += 1;

                instruction_str = format!("{}(${:02X}, X), {}", instruction_str, lo, "{IZX}");
            }
            AddrMode::Izy => {
                let lo = bus.read(addr, true);
                addr += 1;

                instruction_str = format!("{}(${:02X}), Y {}", instruction_str, lo, "{IZY}");
            }
        }

        lines.insert(line_addr, instruction_str.clone());
    }

    lines
}
