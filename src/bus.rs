pub struct Bus {
    bossrush6502: super::cpu::CPU,
    ram: [u8; 65 * 1024],
}

impl Bus {
    pub fn new() -> Self {
        let mut bus = Self {
            bossrush6502: super::cpu::CPU::default(),
            ram: [0; 65 * 1024],
        };
        bus.bossrush6502 = super::cpu::CPU::new(&bus);
        bus
    }
}

impl Bus {
    fn read(&self, addr: u16, _readonly: bool) -> u8 {
        self.ram[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
}

// package emulator

// type bus struct {
// 	bossrush6502 cpu
// 	ram          [64 * 1024]uint8
// }

// func NewBus() bus {
// 	b := bus{}
// 	b.bossrush6502.ConnectBus(&b)
// 	return b
// }

// func (b *bus) Read(addr uint16, readonly bool) uint8 {
// 	if addr >= 0x0000 && addr <= 0xFFFF {
// 		return b.ram[addr]
// 	}

// 	return 0
// }

// func (b *bus) Write(addr uint16, data uint8) {
// 	if addr >= 0x0000 && addr <= 0xFFFF {
// 		b.ram[addr] = data
// 	}
// }
