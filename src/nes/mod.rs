use std::path::Path;

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod instructions;
pub mod ppu;

use bus::Bus;
use cpu::Cpu;
use ppu::Ppu;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    bus: Bus,
}

impl Nes {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut cpu = Cpu::default();
        let mut ppu = Ppu::default();
        let mut bus = Bus::new(path)?;

        Ok(Self { cpu, ppu, bus })
    }

    pub fn load_cartridge(&self) {
        self.bus.load_cartridge();
    }
}

// let cart = nes::Cartridge::new(path)?;
