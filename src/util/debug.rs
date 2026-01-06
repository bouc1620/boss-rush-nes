use crate::nes::{
    Nes, bus::ADDR_PRG_ROM, bus::Bus, cpu::Cpu, cpu::CpuState, cpu::StatusFlags, cpu::has_flag,
    instructions::AddrMode, instructions::Instruction, instructions::get_instruction,
};
use colored::Colorize;
use std::collections::BTreeMap;
use std::io::{Write, stdin, stdout};

// Small program that multiplies 10 by 3 and stores the result at address 0x0002:
// "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA"

pub fn debug_cpu(program: &str) -> Result<(), String> {
    let mut nes = Nes::from_program(program)?;

    nes.reset();
    nes.cpu.step_to_next_instruction(&mut nes.bus);

    let lines = disassemble(
        &nes.bus,
        ADDR_PRG_ROM as u16,
        (ADDR_PRG_ROM + program.len()) as u16,
    );

    loop {
        let state = nes.cpu.get_state();

        print!("\x1B[2J\x1B[1;1H");
        stdout().flush().unwrap();

        print_cpu_status(state.p);

        println!();
        print_cpu_registers(&state);

        nes.bus.print_ram(0x0000, 0x001F);
        nes.bus.print_ram(ADDR_PRG_ROM as u16, 0x801F);

        println!();
        print_instructions(&lines, state.pc);

        println!("\nPress Enter to step, q to quit...");

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() == "q" {
            break;
        }

        nes.cpu.step_to_next_instruction(&mut nes.bus);
    }

    Ok(())
}

fn print_cpu_status(processor_status: u8) {
    fn get_formatted_flag(status: u8, flag: u8, flag_char: char) -> String {
        if has_flag(status, flag) {
            flag_char.to_string().green().to_string()
        } else {
            flag_char.to_string().red().to_string()
        }
    }

    println!(
        "{} {} {} {} {} {} {} {}",
        get_formatted_flag(processor_status, StatusFlags::NEGATIVE, 'N'),
        get_formatted_flag(processor_status, StatusFlags::OVERFLOW, 'V'),
        get_formatted_flag(processor_status, StatusFlags::UNUSED, '-'),
        get_formatted_flag(processor_status, StatusFlags::BREAK, 'B'),
        get_formatted_flag(processor_status, StatusFlags::DECIMAL, 'D'),
        get_formatted_flag(processor_status, StatusFlags::INTERRUPT_DISABLE, 'I'),
        get_formatted_flag(processor_status, StatusFlags::ZERO, 'Z'),
        get_formatted_flag(processor_status, StatusFlags::CARRY, 'C'),
    );
}

fn print_cpu_registers(state: &CpuState) {
    println!("PC: #${:04X}", state.pc);
    println!("A:  #${:02X}", state.a);
    println!("X:  #${:02X}", state.x);
    println!("Y:  #${:02X}", state.y);
    println!("SP: #${:02X}", state.sp);
}

fn print_instructions(instructions: &BTreeMap<u16, String>, current_pc: u16) {
    let window_size = 15;
    let half_window = window_size / 2;

    let addresses: Vec<u16> = instructions.keys().copied().collect();

    if let Some(idx) = addresses.iter().position(|&addr| addr == current_pc) {
        let mut start = idx.saturating_sub(half_window);
        let mut end = (idx + half_window + 1).min(addresses.len());

        let actual_window = end - start;
        if actual_window < window_size && end < addresses.len() {
            end = (end + (window_size - actual_window)).min(addresses.len());
        }

        let actual_window = end - start;
        if actual_window < window_size && start > 0 {
            start = start.saturating_sub(window_size - actual_window)
        };

        for addr in &addresses[start..end] {
            if let Some(instr) = instructions.get(addr) {
                if *addr == current_pc {
                    println!("{}", instr.blue());
                } else {
                    println!("{}", instr);
                }
            }
        }
    }
}

fn disassemble(bus: &Bus, start: u16, end: u16) -> BTreeMap<u16, String> {
    let mut addr = start;

    let mut lines: BTreeMap<u16, String> = BTreeMap::new();

    while addr <= end {
        let line_addr = addr;

        let mut instruction_str = format!("${:04X}: ", addr);

        let opcode = bus.cpu_read(addr, true);
        addr += 1;

        let Instruction { name, mode, .. } = get_instruction(opcode);

        instruction_str = format!("{}{} ", instruction_str, name.to_uppercase());

        match mode {
            AddrMode::Imp => {
                instruction_str = format!("{}{}", instruction_str, "{IMP}");
            }
            AddrMode::Imm => {
                let value = bus.cpu_read(addr, true);
                addr += 1;

                instruction_str = format!("{}#${:02X} {}", instruction_str, value, "{IMM}");
            }
            AddrMode::Zp0 => {
                let lo = bus.cpu_read(addr, true);
                addr += 1;

                instruction_str = format!("{}${:02X} {}", instruction_str, lo, "{ZP0}");
            }
            AddrMode::Zpx => {
                let lo = bus.cpu_read(addr, true);
                addr += 1;

                instruction_str = format!("{}${:02X}, X {}", instruction_str, lo, "{ZPX}");
            }
            AddrMode::Zpy => {
                let lo = bus.cpu_read(addr, true);
                addr += 1;

                instruction_str = format!("{}${:02X}, Y {}", instruction_str, lo, "{ZPY}");
            }
            AddrMode::Rel => {
                let value = bus.cpu_read(addr, true) as u16;
                addr += 1;

                let value = if value & 0x0080 != 0 {
                    // If bit 7 isset, fill upper byte with 1s to preserve negative value
                    value | 0xFF00
                } else {
                    value
                };

                instruction_str = format!(
                    "{}${:02X} [${:04X}] {}",
                    instruction_str,
                    value,
                    addr.wrapping_add(value),
                    "{REL}"
                );
            }
            AddrMode::Abs => {
                let lo = bus.cpu_read(addr, true) as u16;
                addr += 1;
                let hi = bus.cpu_read(addr, true) as u16;
                addr += 1;

                instruction_str = format!("{}${:04X} {}", instruction_str, (hi << 8) | lo, "{ABS}");
            }
            AddrMode::Abx => {
                let lo = bus.cpu_read(addr, true) as u16;
                addr += 1;
                let hi = bus.cpu_read(addr, true) as u16;
                addr += 1;

                instruction_str =
                    format!("{}${:04X}, X {}", instruction_str, (hi << 8) | lo, "{ABX}");
            }
            AddrMode::Aby => {
                let lo = bus.cpu_read(addr, true) as u16;
                addr += 1;
                let hi = bus.cpu_read(addr, true) as u16;
                addr += 1;

                instruction_str =
                    format!("{}${:04X}, Y {}", instruction_str, (hi << 8) | lo, "{ABY}");
            }
            AddrMode::Ind => {
                let lo = bus.cpu_read(addr, true) as u16;
                addr += 1;
                let hi = bus.cpu_read(addr, true) as u16;
                addr += 1;

                instruction_str =
                    format!("{} (${:04X}) {}", instruction_str, (hi << 8) | lo, "{IND}");
            }
            AddrMode::Izx => {
                let lo = bus.cpu_read(addr, true);
                addr += 1;

                instruction_str = format!("{}(${:02X}, X), {}", instruction_str, lo, "{IZX}");
            }
            AddrMode::Izy => {
                let lo = bus.cpu_read(addr, true);
                addr += 1;

                instruction_str = format!("{}(${:02X}), Y {}", instruction_str, lo, "{IZY}");
            }
        }

        lines.insert(line_addr, instruction_str.clone());
    }

    lines
}
