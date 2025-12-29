use crate::bus::Bus;
use crate::instructions::{AddrMode, INSTRUCTIONS, Instruction};

#[derive(Default)]
pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    p: u8,

    fetched: u8,   // Data fetched from memory
    addr_abs: u16, // Absolute address calculated by addressing mode
    addr_rel: u16, // Relative address offset for branch instructions
    opcode: u8,    // Current instruction opcode
    cycles: u8,    // Remaining clock cycles
}

const FLAG_CARRY: u8 = 0b000_0001;
const FLAG_ZERO: u8 = 0b000_0010;
const FLAG_INTERRUPT_DISABLE: u8 = 0b000_0100;
const FLAG_DECIMAL: u8 = 0b000_1000;
const FLAG_BREAK: u8 = 0b001_0000;
const FLAG_UNUSED: u8 = 0b010_0000;
const FLAG_OVERFLOW: u8 = 0b0100_0000;
const FLAG_NEGATIVE: u8 = 0b1000_0000;

fn calc_overflow(a: u16, b: u16, result: u16) -> bool {
    !(a ^ b) & (a ^ result) & 0x0080 != 0
}

impl CPU {
    fn current_instruction(&self) -> Instruction {
        let row = (self.opcode / 16) as usize;
        let col = (self.opcode % 16) as usize;
        INSTRUCTIONS[row][col]
    }

    fn has_flag(&self, flag: u8) -> bool {
        self.p & flag != 0
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    fn fetch(&mut self, bus: &mut Bus) -> u8 {
        if self.current_instruction().mode != AddrMode::Imp {
            self.fetched = bus.read(self.addr_abs, false);
        }

        self.fetched
    }

    pub fn reset(&mut self, bus: &mut Bus) {
        self.addr_abs = 0xFFFC;

        let lo = bus.read(self.addr_abs, false) as u16;
        let hi = bus.read(self.addr_abs + 1, false) as u16;
        self.pc = (hi << 8) | lo;

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.p = FLAG_UNUSED;

        self.addr_abs = 0;
        self.addr_rel = 0;
        self.fetched = 0;

        self.cycles = 8;
    }

    pub fn step(&mut self, bus: &mut Bus) {
        if self.cycles != 0 {
            self.cycles -= 1;
            return;
        }

        self.opcode = bus.read(self.pc, false);
        self.pc += 1;

        let instruction = self.current_instruction();

        self.cycles = instruction.cycles;

        // Add +1 cycle only if a page is crossed and the opcode is a read (eligible for the penalty)
        self.cycles += (instruction.addr)(self, bus) & (instruction.operate)(self, bus);
    }

    // Interrupt request
    pub fn irq(&mut self, bus: &mut Bus) {
        if self.has_flag(FLAG_INTERRUPT_DISABLE) {
            return;
        }

        bus.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8);
        self.sp -= 1;
        bus.write(0x0100 + self.sp as u16, self.pc as u8);
        self.sp -= 1;

        self.set_flag(FLAG_BREAK, false);
        self.set_flag(FLAG_UNUSED, true);
        self.set_flag(FLAG_INTERRUPT_DISABLE, true);

        bus.write(0x0100 + self.sp as u16, self.p);
        self.sp -= 1;

        let lo = bus.read(0xFFFE, false) as u16;
        let hi = bus.read(0xFFFF, false) as u16;
        self.pc = (hi << 8) | lo;

        self.cycles = 7;
    }

    // Non-maskable interrupt
    pub fn nmi(&mut self, bus: &mut Bus) {
        bus.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8);
        self.sp -= 1;
        bus.write(0x0100 + self.sp as u16, self.pc as u8);
        self.sp -= 1;

        self.set_flag(FLAG_BREAK, false);
        self.set_flag(FLAG_UNUSED, true);
        self.set_flag(FLAG_INTERRUPT_DISABLE, true);

        bus.write(0x0100 + self.sp as u16, self.p);
        self.sp -= 1;

        let lo = bus.read(0xFFFA, false) as u16;
        let hi = bus.read(0xFFFB, false) as u16;
        self.pc = (hi << 8) | lo;

        self.cycles = 8;
    }

    // Addressing modes

    // Implicit
    pub fn imp(&mut self, _bus: &mut Bus) -> u8 {
        self.fetched = self.a;

        0
    }

    // Immediate
    pub fn imm(&mut self, _bus: &mut Bus) -> u8 {
        self.addr_abs = self.pc;
        self.pc += 1;

        0
    }

    // Zero page
    pub fn zp0(&mut self, bus: &mut Bus) -> u8 {
        self.addr_abs = bus.read(self.pc, false) as u16;
        self.pc += 1;

        0
    }

    // Zero page indexed with x
    pub fn zpx(&mut self, bus: &mut Bus) -> u8 {
        self.addr_abs = (bus.read(self.pc, false) + self.x) as u16;
        self.pc += 1;

        0
    }

    // Zero page indexed with y
    pub fn zpy(&mut self, bus: &mut Bus) -> u8 {
        self.addr_abs = (bus.read(self.pc, false) + self.y) as u16;
        self.pc += 1;

        0
    }

    // Relative
    pub fn rel(&mut self, bus: &mut Bus) -> u8 {
        self.addr_rel = bus.read(self.pc, false) as u16;
        self.pc += 1;

        if self.addr_rel & 0x0080 != 0 {
            // If bit 7 isset, fill upper byte with 1s to preserve negative value
            self.addr_rel |= 0xFF00;
        }

        0
    }

    // Absolute
    pub fn abs(&mut self, bus: &mut Bus) -> u8 {
        let lo = bus.read(self.pc, false) as u16;
        self.pc += 1;
        let hi = bus.read(self.pc, false) as u16;
        self.pc += 1;

        self.addr_abs = (hi << 8) | lo;

        0
    }

    // Absolute indexed with x
    pub fn abx(&mut self, bus: &mut Bus) -> u8 {
        let lo = bus.read(self.pc, false) as u16;
        self.pc += 1;
        let hi = bus.read(self.pc, false) as u16;
        self.pc += 1;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += self.x as u16;

        if self.addr_abs & 0xFF00 != (hi << 8) {
            // Page boundary crossed (+1 cycle)
            1
        } else {
            0
        }
    }

    // Absolute indexed with y
    pub fn aby(&mut self, bus: &mut Bus) -> u8 {
        let lo = bus.read(self.pc, false) as u16;
        self.pc += 1;
        let hi = bus.read(self.pc, false) as u16;
        self.pc += 1;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += self.y as u16;

        if self.addr_abs & 0xFF00 != (hi << 8) {
            // Page boundary crossed (+1 cycle)
            1
        } else {
            0
        }
    }

    // Indirect
    pub fn ind(&mut self, bus: &mut Bus) -> u8 {
        let lo = bus.read(self.pc, false) as u16;
        self.pc += 1;
        let hi = bus.read(self.pc, false) as u16;
        self.pc += 1;

        let addr = (hi << 8) | lo;

        if lo == 0x00FF {
            // Simulates 6502 hardware bug: addr treated as 2 separate bytes, carry not propagated to MSB
            // Example: JMP ($10FF) reads LSB from $10FF and MSB from $1000 (not $1100)
            self.addr_abs =
                ((bus.read(addr & 0xFF00, false) as u16) << 8) | bus.read(addr, false) as u16;
        } else {
            self.addr_abs =
                ((bus.read(addr + 1, false) as u16) << 8) | bus.read(addr, false) as u16;
        }

        0
    }

    // Indirect indexed with x
    pub fn izx(&mut self, bus: &mut Bus) -> u8 {
        let addr = bus.read(self.pc, false);
        self.pc += 1;

        let lo = bus.read((addr + self.x) as u16, false) as u16;
        let hi = bus.read((addr + self.x + 1) as u16, false) as u16;

        self.addr_abs = (hi << 8) | lo;

        0
    }

    // Indirect indexed with y
    pub fn izy(&mut self, bus: &mut Bus) -> u8 {
        let addr = bus.read(self.pc, false);
        self.pc += 1;

        let lo = bus.read(addr as u16, false) as u16;
        let hi = bus.read((addr + 1) as u16, false) as u16;

        // Comparatively to izx, izy adds the index after dereferencing the pointer
        // The instruction is better suited to iterate through data structures that span
        // accross multiple pages
        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += self.y as u16;

        if self.addr_abs & 0xFF00 != (hi << 8) {
            // Page boundary crossed (+1 cycle)
            1
        } else {
            0
        }
    }

    // Opcodes

    // Add with carry
    pub fn adc(&mut self, bus: &mut Bus) -> u8 {
        self.fetch(bus);

        let carry = if self.has_flag(FLAG_CARRY) { 1 } else { 0 };
        let temp = self.a as u16 + self.fetched as u16 + carry;

        self.set_flag(FLAG_CARRY, temp & 0xFF00 != 0);
        self.set_flag(FLAG_ZERO, temp & 0x00FF == 0);
        self.set_flag(FLAG_NEGATIVE, temp & 0x0080 == 0);
        self.set_flag(
            FLAG_OVERFLOW,
            calc_overflow(self.a as u16, self.fetched as u16, temp),
        );

        self.a = temp as u8;

        1
    }

    // Bitwise AND
    pub fn and(&mut self, bus: &mut Bus) -> u8 {
        self.fetch(bus);

        self.a &= self.fetched;

        self.set_flag(FLAG_ZERO, self.a == 0);
        self.set_flag(FLAG_NEGATIVE, self.a & 0x0080 != 0);

        1
    }

    // Arithmetic shift left
    pub fn asl(&mut self, bus: &mut Bus) -> u8 {
        self.fetch(bus);

        let temp = (self.fetched as u16) << 1;

        self.set_flag(FLAG_CARRY, temp & 0xFF00 != 0);
        self.set_flag(FLAG_ZERO, temp & 0x00FF == 0);
        self.set_flag(FLAG_NEGATIVE, temp & 0x0080 == 0);

        if self.current_instruction().mode == AddrMode::Imp {
            self.a = temp as u8;
        } else {
            bus.write(self.addr_abs, temp as u8);
        }

        0
    }

    fn branch_taken(&mut self) {
        self.cycles += 1;
        self.addr_abs = self.pc + self.addr_rel;

        if self.addr_abs & 0xFF00 != self.pc & 0xFF00 {
            // Page boundary crossed (+1 cycle)
            self.cycles += 1;
        }

        self.pc = self.addr_abs;
    }

    // Branch if carry clear
    pub fn bcc(&mut self, _bus: &mut Bus) -> u8 {
        if !self.has_flag(FLAG_CARRY) {
            self.branch_taken();
        }

        0
    }

    // Branch if carry set
    pub fn bcs(&mut self, _bus: &mut Bus) -> u8 {
        if self.has_flag(FLAG_CARRY) {
            self.branch_taken();
        }

        0
    }

    // Branch if equal
    pub fn beq(&mut self, _bus: &mut Bus) -> u8 {
        if !self.has_flag(FLAG_ZERO) {
            self.branch_taken();
        }

        0
    }

    // Bit test
    pub fn bit(&mut self, bus: &mut Bus) -> u8 {
        self.fetch(bus);

        let temp = self.a & self.fetched;

        self.set_flag(FLAG_ZERO, temp == 0);
        self.set_flag(FLAG_OVERFLOW, temp & 0x0040 != 0);
        self.set_flag(FLAG_NEGATIVE, temp & 0x0080 != 0);

        0
    }

    // Branch if minus
    pub fn bmi(&mut self, _bus: &mut Bus) -> u8 {
        if self.has_flag(FLAG_NEGATIVE) {
            self.branch_taken();
        }

        0
    }

    // Branch if not equal
    pub fn bne(&mut self, _bus: &mut Bus) -> u8 {
        if self.has_flag(FLAG_ZERO) {
            self.branch_taken();
        }

        0
    }

    // Branch if plus
    pub fn bpl(&mut self, _bus: &mut Bus) -> u8 {
        if !self.has_flag(FLAG_NEGATIVE) {
            self.branch_taken();
        }

        0
    }

    // Break (software IRQ)
    pub fn brk(&mut self, bus: &mut Bus) -> u8 {
        self.pc += 1;

        self.set_flag(FLAG_INTERRUPT_DISABLE, true);

        bus.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8);
        self.sp -= 1;
        bus.write(0x0100 + self.sp as u16, self.pc as u8);
        self.sp -= 1;

        self.set_flag(FLAG_BREAK, true);

        bus.write(0x0100 + self.sp as u16, self.p);
        self.sp -= 1;

        self.set_flag(FLAG_BREAK, false);

        let lo = bus.read(0xFFFE, false) as u16;
        let hi = bus.read(0xFFFF, false) as u16;
        self.pc = (hi << 8) | lo;

        0
    }

    // Branch if overflow clear
    pub fn bvc(&mut self, _bus: &mut Bus) -> u8 {
        if !self.has_flag(FLAG_OVERFLOW) {
            self.branch_taken();
        }

        0
    }

    // Branch if overflow set
    pub fn bvs(&mut self, _bus: &mut Bus) -> u8 {
        if self.has_flag(FLAG_OVERFLOW) {
            self.branch_taken();
        }

        0
    }

    // Clear carry
    pub fn clc(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG_CARRY, false);

        0
    }

    // Clear decimal
    pub fn cld(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG_DECIMAL, false);

        0
    }

    // Clear interrupt disable
    pub fn cli(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG_INTERRUPT_DISABLE, false);

        0
    }

    // Clear overflow
    pub fn clv(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(FLAG_OVERFLOW, false);

        0
    }

    fn set_compare_flags(&mut self, result: u16) {
        self.set_flag(FLAG_CARRY, result & 0xFF00 == 0);
        self.set_flag(FLAG_OVERFLOW, result == 0);
        self.set_flag(FLAG_NEGATIVE, result & 0x0080 != 0);
    }

    // Compare a
    pub fn cmp(&mut self, bus: &mut Bus) -> u8 {
        self.fetch(bus);

        self.set_compare_flags(self.a as u16 - self.fetched as u16);

        0
    }

    // Compare x
    pub fn cpx(&mut self, bus: &mut Bus) -> u8 {
        self.fetch(bus);

        self.set_compare_flags(self.x as u16 - self.fetched as u16);

        0
    }

    // Compare y
    pub fn cpy(&mut self, bus: &mut Bus) -> u8 {
        self.fetch(bus);

        self.set_compare_flags(self.y as u16 - self.fetched as u16);

        0
    }

    pub fn ora(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn nop(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn php(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn jsr(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn rol(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn plp(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sec(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn rti(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn eor(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn lsr(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn pha(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn jmp(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn rts(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn ror(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn pla(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sei(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sta(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sty(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn stx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn dey(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn txa(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn tya(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn txs(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn ldy(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn lda(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn ldx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn tay(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn tax(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn tsx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn dec(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn iny(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn dex(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sbc(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn inc(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn inx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn sed(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    pub fn xxx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }
}
