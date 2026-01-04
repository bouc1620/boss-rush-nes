use super::cartridge::Cartridge;
use super::ppu::PpuRegisters;
use std::path::Path;

pub struct Bus {
    ram: [u8; 64 * 1024],
    cartridge: Cartridge,
    ppu_registers: PpuRegisters,
}

impl Bus {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            ram: [0; 64 * 1024],
            cartridge: Cartridge::new(path)?,
            ppu_registers: PpuRegisters {},
        })
    }
}

pub const ADDR_PRG_ROM: usize = 0x8000;
pub const ADDR_RESET_VECTOR: usize = 0xFFFC;

impl Bus {
    pub fn cpu_read(&self, addr: u16, _readonly: bool) -> u8 {
        if addr < 0x2000 {
            // Internal RAM: 0x0000 - 0x1FFF (mirrored 3 times)
            let addr = addr & 0x07FF;
            self.ram[addr as usize]
        } else if addr < 0x4000 {
            // PPU registers: $2000 - $3FFF (mirrored every 8 bytes)
            let addr = 0x2000 + (addr & 0x0007);
            self.ram[addr as usize]
        } else if addr < 0x4017 {
            // APU / IO: $4000 - $4017
            self.ram[addr as usize]
        } else if addr >= 0x8000 {
            // Cartridge PRG-ROM: 0x8000 - 0xFFFF
            self.ram[addr as usize]
        } else {
            0
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        if addr < 0x2000 {
            // Internal RAM: 0x0000 - 0x1FFF (mirrored 3 times)
            let addr = addr & 0x07FF;
            self.ram[addr as usize] = data;
        }
    }

    pub fn ppu_read(&self, addr: u16, _readonly: bool) -> u8 {
        if addr < 0x2000 {
            // Internal RAM: 0x0000 - 0x1FFF (mirrored 3 times)
            let addr = addr & 0x07FF;
            self.ram[addr as usize]
        } else {
            0
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    pub fn load_cartridge(&self) {}
}

#[cfg(feature = "debug")]
impl Bus {
    pub fn load_program(&mut self, program: &str) -> Result<(), String> {
        let program_bytes: Result<Vec<u8>, _> = program
            .split_whitespace()
            .map(|byte| u8::from_str_radix(byte, 16))
            .collect();

        let program_bytes = program_bytes.map_err(|e| format!("Invalid hex: {}", e))?;

        self.ram[ADDR_PRG_ROM..ADDR_PRG_ROM + program_bytes.len()].copy_from_slice(&program_bytes);

        self.ram[ADDR_RESET_VECTOR] = 0x00;
        self.ram[ADDR_RESET_VECTOR + 1] = 0x80;

        Ok(())
    }

    pub fn print_ram(&self, start: u16, end: u16) {
        for addr in start..=end {
            if (addr - start).is_multiple_of(16) {
                print!("\n{:04X}: ", addr);
            }

            print!("{:02X} ", self.cpu_read(addr, true));
        }

        println!();
    }
}
