use crate::nes::bus::{ADDR_PRG_ROM, ADDR_RESET_VECTOR};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper: u8,
    mirroring: Mirroring,
    prg_ram_size: u8,
}

fn get_mirroring(flag6: u8) -> Mirroring {
    match (flag6 & 0x08 != 0, flag6 & 0x01 != 0) {
        (true, _) => Mirroring::FourScreen,
        (false, true) => Mirroring::Vertical,
        (false, false) => Mirroring::Horizontal,
    }
}

impl Cartridge {
    pub fn from_rom(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer)?;

        if &buffer[0..4] != b"NES\x1A" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid NES header",
            ));
        }

        let prg_size = buffer[4] as usize * 16 * 1024;
        let chr_size = buffer[5] as usize * 8 * 1024;

        let flag6 = buffer[6];
        let flag7 = buffer[7];

        let mapper = (flag7 & 0xF0) | (flag6 >> 4);

        let mirroring = get_mirroring(flag6);

        let prg_ram_size = if buffer[8] == 0 { 1 } else { buffer[8] };

        if buffer.len() < 16 + prg_size + chr_size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "ROM file too small for declared PRG/CHR size",
            ));
        }

        let has_trainer = flag6 & 0x04 != 0;
        let prg_start = 16 + if has_trainer { 512 } else { 0 };

        let prg_rom = buffer[prg_start..prg_start + prg_size].to_vec();
        let chr_rom = buffer[prg_start + prg_size..prg_start + prg_size + chr_size].to_vec();

        Ok(Self {
            prg_rom,
            chr_rom,
            mapper,
            mirroring,
            prg_ram_size,
        })
    }

    pub fn from_program(program: &str) -> Result<Self, String> {
        let program_bytes: Result<Vec<u8>, _> = program
            .split_whitespace()
            .map(|byte| u8::from_str_radix(byte, 16))
            .collect();

        let program_bytes = program_bytes.map_err(|e| format!("Invalid hex: {}", e))?;

        let mut prg_rom = vec![0u8; 0x10000];

        prg_rom[0..program_bytes.len()].copy_from_slice(&program_bytes);

        let reset_offset = ADDR_RESET_VECTOR - ADDR_PRG_ROM;
        prg_rom[reset_offset] = 0x00;
        prg_rom[reset_offset + 1] = 0x80;

        Ok(Self {
            prg_rom,
            chr_rom: vec![],
            mapper: 0,
            mirroring: Mirroring::Horizontal,
            prg_ram_size: 1,
        })
    }

    pub fn read(&self, addr: usize) -> u8 {
        if addr < self.prg_rom.len() {
            self.prg_rom[addr]
        } else {
            0
        }
    }
}
