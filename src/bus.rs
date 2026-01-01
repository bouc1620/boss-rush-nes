pub struct Bus {
    ram: [u8; 64 * 1024],
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            ram: [0; 64 * 1024],
        }
    }
}

pub const ADDR_RESET_VECTOR: usize = 0xFFFC;

impl Bus {
    pub fn read(&self, addr: u16, _readonly: bool) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
}

#[cfg(feature = "debug")]
impl Bus {
    pub fn load_program(&mut self, program: &str) -> Result<(), String> {
        let program_bytes: Result<Vec<u8>, _> = program
            .split_whitespace()
            .map(|byte| u8::from_str_radix(byte, 16))
            .collect();

        let program_bytes = program_bytes.map_err(|e| format!("Invalid hex: {}", e))?;

        self.ram[0x8000..0x8000 + program_bytes.len()].copy_from_slice(&program_bytes);

        self.ram[ADDR_RESET_VECTOR] = 0x00;
        self.ram[ADDR_RESET_VECTOR + 1] = 0x80;

        Ok(())
    }

    pub fn print_ram(&self, start: u16, end: u16) {
        for addr in start..=end {
            if (addr - start).is_multiple_of(16) {
                print!("\n{:04X}: ", addr);
            }

            print!("{:02X} ", self.read(addr, true));
        }

        println!();
    }
}
