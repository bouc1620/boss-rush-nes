use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;
use ppu::Ppu;
use std::cell::RefCell;
use std::io;
use std::path::Path;
use std::rc::Rc;

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod instructions;
pub mod ppu;

pub struct Nes {
    pub cpu: Cpu,
    pub bus: Bus,
    ppu: Rc<RefCell<Ppu>>,
    cartridge: Rc<RefCell<Cartridge>>,
}

impl Nes {
    pub fn from_rom(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let cartridge = Rc::new(RefCell::new(Cartridge::from_rom(path)?));
        let ppu = Rc::new(RefCell::new(Ppu::default()));

        Ok(Self {
            cpu: Cpu::default(),
            bus: Bus::new(Rc::clone(&ppu), Rc::clone(&cartridge)),
            ppu,
            cartridge,
        })
    }

    pub fn from_program(program: &str) -> Result<Self, String> {
        let cartridge = Rc::new(RefCell::new(Cartridge::from_program(program)?));
        let ppu = Rc::new(RefCell::new(Ppu::default()));

        Ok(Self {
            cpu: Cpu::default(),
            bus: Bus::new(Rc::clone(&ppu), Rc::clone(&cartridge)),
            ppu,
            cartridge,
        })
    }

    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.bus);
    }

    pub fn run(&mut self) {
        println!("Run");
    }
}
