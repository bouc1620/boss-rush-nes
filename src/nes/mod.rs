use bus::Bus;
use cpu::Cpu;
use ppu::Ppu;
use std::io;
use std::path::Path;

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod instructions;
pub mod ppu;

pub struct Nes {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub bus: Bus,
}

impl Nes {
    pub fn from_rom(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        Ok(Self {
            cpu: Cpu::default(),
            ppu: Ppu::default(),
            bus: Bus::from_rom(path)?,
        })
    }

    pub fn from_program(program: &str) -> Result<Self, String> {
        Ok(Self {
            cpu: Cpu::default(),
            ppu: Ppu::default(),
            bus: Bus::from_program(program)?,
        })
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.bus);
    }

    pub fn run(&mut self) {
        println!("Run");
    }
}
