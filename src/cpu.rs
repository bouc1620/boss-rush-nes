use crate::bus::{ADDR_RESET_VECTOR, Bus};
use crate::instructions::{AddrMode, Instruction, get_instruction};

#[derive(Default)]
pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,  // Stack pointer
    pc: u16, // Program counter
    p: u8,   // Processor status

    addr_abs: u16, // Absolute address calculated by addressing mode
    addr_rel: u16, // Relative address offset for branch instructions
    opcode: u8,    // Current instruction opcode
    cycles: u8,    // Remaining clock cycles
}

pub struct StatusFlags;

impl StatusFlags {
    pub const CARRY: u8 = 0b000_0001;
    pub const ZERO: u8 = 0b000_0010;
    pub const INTERRUPT_DISABLE: u8 = 0b000_0100;
    pub const DECIMAL: u8 = 0b000_1000;
    pub const BREAK: u8 = 0b001_0000;
    pub const UNUSED: u8 = 0b010_0000;
    pub const OVERFLOW: u8 = 0b0100_0000;
    pub const NEGATIVE: u8 = 0b1000_0000;
}

// This formula is derived from the truth table for a + b = result | overflow
fn calc_overflow(a: u16, b: u16, result: u16) -> bool {
    !(a ^ b) & (a ^ result) & 0x0080 != 0
}

pub fn has_flag(p: u8, flag: u8) -> bool {
    p & flag != 0
}

impl CPU {
    fn current_instruction(&self) -> Instruction {
        get_instruction(self.opcode)
    }

    fn has_flag(&self, flag: u8) -> bool {
        has_flag(self.p, flag)
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }

        self.p |= StatusFlags::UNUSED;
    }

    fn fetch(&mut self, bus: &mut Bus) -> u8 {
        if matches!(self.current_instruction().mode, AddrMode::Imp) {
            self.a
        } else {
            bus.read(self.addr_abs, false)
        }
    }

    pub fn reset(&mut self, bus: &mut Bus) {
        self.addr_abs = ADDR_RESET_VECTOR as u16;

        let lo = bus.read(self.addr_abs, false) as u16;
        let hi = bus.read(self.addr_abs + 1, false) as u16;
        self.pc = (hi << 8) | lo;

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.p = StatusFlags::UNUSED;

        self.addr_abs = 0;
        self.addr_rel = 0;

        self.cycles = 8;
    }

    pub fn step(&mut self, bus: &mut Bus) {
        if self.cycles > 0 {
            self.cycles = self.cycles.saturating_sub(1);

            return;
        }

        self.opcode = bus.read(self.pc, false);
        self.pc = self.pc.wrapping_add(1);

        let Instruction {
            cycles,
            addr,
            operate,
            ..
        } = self.current_instruction();

        self.cycles = cycles;

        // Add +1 cycle penalty only if a page is crossed and the opcode is a read
        self.cycles += (addr)(self, bus) & (operate)(self, bus);

        self.cycles = self.cycles.saturating_sub(1);
    }

    // Interrupt request
    pub fn irq(&mut self, bus: &mut Bus) {
        if self.has_flag(StatusFlags::INTERRUPT_DISABLE) {
            return;
        }

        bus.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        bus.write(0x0100 + self.sp as u16, self.pc as u8);
        self.sp = self.sp.wrapping_sub(1);

        bus.write(0x0100 + self.sp as u16, self.p);
        self.sp = self.sp.wrapping_sub(1);

        self.set_flag(StatusFlags::BREAK, false);
        self.set_flag(StatusFlags::INTERRUPT_DISABLE, true);

        let lo = bus.read(0xFFFE, false) as u16;
        let hi = bus.read(0xFFFF, false) as u16;
        self.pc = (hi << 8) | lo;

        self.cycles = 7;
    }

    // Non-maskable interrupt
    pub fn nmi(&mut self, bus: &mut Bus) {
        bus.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        bus.write(0x0100 + self.sp as u16, self.pc as u8);
        self.sp = self.sp.wrapping_sub(1);

        bus.write(0x0100 + self.sp as u16, self.p);
        self.sp = self.sp.wrapping_sub(1);

        self.set_flag(StatusFlags::BREAK, false);
        self.set_flag(StatusFlags::INTERRUPT_DISABLE, true);

        let lo = bus.read(0xFFFA, false) as u16;
        let hi = bus.read(0xFFFB, false) as u16;
        self.pc = (hi << 8) | lo;

        self.cycles = 8;
    }
}

impl CPU {
    // Addressing modes

    // Implicit
    pub fn imp(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    // Immediate
    pub fn imm(&mut self, _bus: &mut Bus) -> u8 {
        self.addr_abs = self.pc;
        self.pc = self.pc.wrapping_add(1);

        0
    }

    // Zero page
    pub fn zp0(&mut self, bus: &mut Bus) -> u8 {
        self.addr_abs = bus.read(self.pc, false) as u16;
        self.pc = self.pc.wrapping_add(1);

        0
    }

    // Zero page indexed with x
    pub fn zpx(&mut self, bus: &mut Bus) -> u8 {
        self.addr_abs = bus.read(self.pc, false).wrapping_add(self.x) as u16;
        self.pc = self.pc.wrapping_add(1);

        0
    }

    // Zero page indexed with y
    pub fn zpy(&mut self, bus: &mut Bus) -> u8 {
        self.addr_abs = bus.read(self.pc, false).wrapping_add(self.y) as u16;
        self.pc = self.pc.wrapping_add(1);

        0
    }

    // Relative
    pub fn rel(&mut self, bus: &mut Bus) -> u8 {
        self.addr_rel = bus.read(self.pc, false) as u16;
        self.pc = self.pc.wrapping_add(1);

        if self.addr_rel & 0x0080 != 0 {
            // If bit 7 isset, fill upper byte with 1s to preserve negative value
            self.addr_rel |= 0xFF00;
        }

        0
    }

    // Absolute
    pub fn abs(&mut self, bus: &mut Bus) -> u8 {
        let lo = bus.read(self.pc, false) as u16;
        self.pc = self.pc.wrapping_add(1);
        let hi = bus.read(self.pc, false) as u16;
        self.pc = self.pc.wrapping_add(1);

        self.addr_abs = (hi << 8) | lo;

        0
    }

    // Absolute indexed with x
    pub fn abx(&mut self, bus: &mut Bus) -> u8 {
        let lo = bus.read(self.pc, false) as u16;
        self.pc = self.pc.wrapping_add(1);
        let hi = bus.read(self.pc, false) as u16;
        self.pc = self.pc.wrapping_add(1);

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs = self.addr_abs.wrapping_add(self.x as u16);

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
        self.pc = self.pc.wrapping_add(1);
        let hi = bus.read(self.pc, false) as u16;
        self.pc = self.pc.wrapping_add(1);

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs = self.addr_abs.wrapping_add(self.y as u16);

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
        self.pc = self.pc.wrapping_add(1);
        let hi = bus.read(self.pc, false) as u16;
        self.pc = self.pc.wrapping_add(1);

        let addr = (hi << 8) | lo;

        if lo == 0x00FF {
            // Simulates a 6502 hardware bug: addr treated as 2 separate bytes, carry is not propagated to MSB.
            // Example: JMP ($10FF) reads LSB from $10FF and MSB from $1000 (not $1100).
            self.addr_abs =
                ((bus.read(addr & 0xFF00, false) as u16) << 8) | bus.read(addr, false) as u16;
        } else {
            self.addr_abs = ((bus.read(addr.wrapping_add(1), false) as u16) << 8)
                | bus.read(addr, false) as u16;
        }

        0
    }

    // Indirect indexed with x
    pub fn izx(&mut self, bus: &mut Bus) -> u8 {
        let addr = bus.read(self.pc, false);
        self.pc = self.pc.wrapping_add(1);

        let lo = bus.read(addr.wrapping_add(self.x) as u16, false) as u16;
        let hi = bus.read(addr.wrapping_add(self.x).wrapping_add(1) as u16, false) as u16;

        self.addr_abs = (hi << 8) | lo;

        0
    }

    // Indirect indexed with y
    pub fn izy(&mut self, bus: &mut Bus) -> u8 {
        let addr = bus.read(self.pc, false);
        self.pc = self.pc.wrapping_add(1);

        let lo = bus.read(addr as u16, false) as u16;
        let hi = bus.read(addr.wrapping_add(1) as u16, false) as u16;

        // Comparatively to izx, izy adds the index after dereferencing the pointer.
        // This instruction is better suited to iterate through data structures that span
        // accross multiple pages.
        self.addr_abs = (hi << 8) | lo;
        self.addr_abs = self.addr_abs.wrapping_add(self.y as u16);

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
        let fetched = self.fetch(bus);

        let carry = if self.has_flag(StatusFlags::CARRY) {
            1
        } else {
            0
        };
        let result = (self.a as u16)
            .wrapping_add(fetched as u16)
            .wrapping_add(carry);

        self.set_flag(StatusFlags::CARRY, result & 0xFF00 != 0);
        self.set_flag(StatusFlags::ZERO, result & 0x00FF == 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x0080 != 0);
        self.set_flag(
            StatusFlags::OVERFLOW,
            calc_overflow(self.a as u16, fetched as u16, result),
        );

        self.a = result as u8;

        1
    }

    // Bitwise AND
    pub fn and(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        self.a &= fetched;

        self.set_flag(StatusFlags::ZERO, self.a == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.a & 0x80 != 0);

        1
    }

    // Arithmetic shift left
    pub fn asl(&mut self, bus: &mut Bus) -> u8 {
        let result = (self.fetch(bus) as u16) << 1;

        self.set_flag(StatusFlags::CARRY, result & 0xFF00 != 0);
        self.set_flag(StatusFlags::ZERO, result & 0x00FF == 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x0080 != 0);

        if self.current_instruction().mode == AddrMode::Imp {
            self.a = result as u8;
        } else {
            bus.write(self.addr_abs, result as u8);
        }

        0
    }

    fn branch_taken(&mut self) {
        self.cycles += 1;
        self.addr_abs = self.pc.wrapping_add(self.addr_rel);

        if self.addr_abs & 0xFF00 != self.pc & 0xFF00 {
            // Page boundary crossed (+1 cycle)
            self.cycles += 1;
        }

        self.pc = self.addr_abs;
    }

    // Branch if carry clear
    pub fn bcc(&mut self, _bus: &mut Bus) -> u8 {
        if !self.has_flag(StatusFlags::CARRY) {
            self.branch_taken();
        }

        0
    }

    // Branch if carry set
    pub fn bcs(&mut self, _bus: &mut Bus) -> u8 {
        if self.has_flag(StatusFlags::CARRY) {
            self.branch_taken();
        }

        0
    }

    // Branch if equal
    pub fn beq(&mut self, _bus: &mut Bus) -> u8 {
        if self.has_flag(StatusFlags::ZERO) {
            self.branch_taken();
        }

        0
    }

    // Bit test
    pub fn bit(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        let result = self.a & fetched;

        self.set_flag(StatusFlags::ZERO, result == 0);
        self.set_flag(StatusFlags::OVERFLOW, result & 0x40 != 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x80 != 0);

        0
    }

    // Branch if minus
    pub fn bmi(&mut self, _bus: &mut Bus) -> u8 {
        if self.has_flag(StatusFlags::NEGATIVE) {
            self.branch_taken();
        }

        0
    }

    // Branch if not equal
    pub fn bne(&mut self, _bus: &mut Bus) -> u8 {
        if !self.has_flag(StatusFlags::ZERO) {
            self.branch_taken();
        }

        0
    }

    // Branch if plus
    pub fn bpl(&mut self, _bus: &mut Bus) -> u8 {
        if !self.has_flag(StatusFlags::NEGATIVE) {
            self.branch_taken();
        }

        0
    }

    // Break (software IRQ)
    pub fn brk(&mut self, bus: &mut Bus) -> u8 {
        self.pc = self.pc.wrapping_add(1);

        self.set_flag(StatusFlags::INTERRUPT_DISABLE, true);

        bus.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        bus.write(0x0100 + self.sp as u16, self.pc as u8);
        self.sp = self.sp.wrapping_sub(1);

        self.set_flag(StatusFlags::BREAK, true);

        bus.write(0x0100 + self.sp as u16, self.p);
        self.sp = self.sp.wrapping_sub(1);

        self.set_flag(StatusFlags::BREAK, false);

        let lo = bus.read(0xFFFE, false) as u16;
        let hi = bus.read(0xFFFF, false) as u16;
        self.pc = (hi << 8) | lo;

        0
    }

    // Branch if overflow clear
    pub fn bvc(&mut self, _bus: &mut Bus) -> u8 {
        if !self.has_flag(StatusFlags::OVERFLOW) {
            self.branch_taken();
        }

        0
    }

    // Branch if overflow set
    pub fn bvs(&mut self, _bus: &mut Bus) -> u8 {
        if self.has_flag(StatusFlags::OVERFLOW) {
            self.branch_taken();
        }

        0
    }

    // Clear carry
    pub fn clc(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(StatusFlags::CARRY, false);

        0
    }

    // Clear decimal
    pub fn cld(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(StatusFlags::DECIMAL, false);

        0
    }

    // Clear interrupt disable
    pub fn cli(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(StatusFlags::INTERRUPT_DISABLE, false);

        0
    }

    // Clear overflow
    pub fn clv(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(StatusFlags::OVERFLOW, false);

        0
    }

    fn set_compare_flags(&mut self, result: u16) {
        self.set_flag(StatusFlags::CARRY, result & 0xFF00 == 0);
        self.set_flag(StatusFlags::ZERO, result & 0x00FF == 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x0080 != 0);
    }

    // Compare a
    pub fn cmp(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        self.set_compare_flags((self.a as u16).wrapping_sub(fetched as u16));

        1
    }

    // Compare x
    pub fn cpx(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        self.set_compare_flags((self.x as u16).wrapping_sub(fetched as u16));

        0
    }

    // Compare y
    pub fn cpy(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        self.set_compare_flags((self.y as u16).wrapping_sub(fetched as u16));

        0
    }

    // Decrement memory
    pub fn dec(&mut self, bus: &mut Bus) -> u8 {
        let result = self.fetch(bus).wrapping_sub(1);

        bus.write(self.addr_abs, result);

        self.set_flag(StatusFlags::ZERO, result == 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x80 != 0);

        0
    }

    // Decrement x
    pub fn dex(&mut self, _bus: &mut Bus) -> u8 {
        self.x = self.x.wrapping_sub(1);

        self.set_flag(StatusFlags::ZERO, self.x == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.x & 0x80 != 0);

        0
    }

    // Decrement y
    pub fn dey(&mut self, _bus: &mut Bus) -> u8 {
        self.y = self.y.wrapping_sub(1);

        self.set_flag(StatusFlags::ZERO, self.y == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.y & 0x80 != 0);

        0
    }

    // Bitwise exclusive OR
    pub fn eor(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        self.a ^= fetched;

        self.set_flag(StatusFlags::ZERO, self.a == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.a & 0x80 != 0);

        0
    }

    // Increment memory
    pub fn inc(&mut self, bus: &mut Bus) -> u8 {
        let result = self.fetch(bus).wrapping_add(1);

        bus.write(self.addr_abs, result);

        self.set_flag(StatusFlags::ZERO, result == 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x80 != 0);

        0
    }

    // Increment x
    pub fn inx(&mut self, _bus: &mut Bus) -> u8 {
        self.x = self.x.wrapping_add(1);

        self.set_flag(StatusFlags::ZERO, self.x == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.x & 0x80 != 0);

        0
    }

    // Increment y
    pub fn iny(&mut self, _bus: &mut Bus) -> u8 {
        self.y = self.y.wrapping_add(1);

        self.set_flag(StatusFlags::ZERO, self.y == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.y & 0x80 != 0);

        0
    }

    // Jump
    pub fn jmp(&mut self, _bus: &mut Bus) -> u8 {
        self.pc = self.addr_abs;

        0
    }

    // Jump to subroutine
    pub fn jsr(&mut self, bus: &mut Bus) -> u8 {
        self.pc = self.pc.wrapping_sub(1);

        bus.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1);
        bus.write(0x0100 + self.sp as u16, self.pc as u8);
        self.sp = self.sp.wrapping_sub(1);

        self.pc = self.addr_abs;

        0
    }

    // Load a
    pub fn lda(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        self.a = fetched;

        self.set_flag(StatusFlags::ZERO, self.a == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.a & 0x80 != 0);

        0
    }

    // Load x
    pub fn ldx(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        self.x = fetched;

        self.set_flag(StatusFlags::ZERO, self.x == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.x & 0x80 != 0);

        0
    }

    // Load y
    pub fn ldy(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        self.y = fetched;

        self.set_flag(StatusFlags::ZERO, self.y == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.y & 0x80 != 0);

        0
    }

    // Logical shift right
    pub fn lsr(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        let result = fetched >> 1;

        self.set_flag(StatusFlags::CARRY, fetched & 0x01 != 0);
        self.set_flag(StatusFlags::ZERO, result == 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x80 != 0);

        if self.current_instruction().mode == AddrMode::Imp {
            self.a = result;
        } else {
            bus.write(self.addr_abs, result);
        }

        0
    }

    // No operation
    pub fn nop(&mut self, _bus: &mut Bus) -> u8 {
        0
    }

    // Bitwise or
    pub fn ora(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        self.a |= fetched;

        self.set_flag(StatusFlags::ZERO, self.a == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.a & 0x80 != 0);

        1
    }

    // Push a
    pub fn pha(&mut self, bus: &mut Bus) -> u8 {
        bus.write(0x0100 + self.sp as u16, self.a);
        self.sp = self.sp.wrapping_sub(1);

        0
    }

    // Push processor status
    pub fn php(&mut self, bus: &mut Bus) -> u8 {
        bus.write(
            0x0100 + self.sp as u16,
            self.p | StatusFlags::BREAK | StatusFlags::UNUSED,
        );
        self.sp = self.sp.wrapping_sub(1);

        self.set_flag(StatusFlags::BREAK, false);

        0
    }

    // Pull a
    pub fn pla(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);

        self.a = bus.read(0x0100 + self.sp as u16, false);

        self.set_flag(StatusFlags::ZERO, self.a == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.a & 0x80 != 0);

        0
    }

    // Pull processor status
    pub fn plp(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);

        let temp = bus.read(0x0100 + self.sp as u16, false);

        self.p |= temp & 0b1100_1111;

        0
    }

    // Rotate left
    pub fn rol(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        let carry = if self.has_flag(StatusFlags::CARRY) {
            1
        } else {
            0
        };
        let result = (fetched << 1) | carry;

        self.set_flag(StatusFlags::CARRY, fetched & 0x80 != 0);
        self.set_flag(StatusFlags::ZERO, result == 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x80 != 0);

        if self.current_instruction().mode == AddrMode::Imp {
            self.a = result;
        } else {
            bus.write(self.addr_abs, result);
        }

        0
    }

    // Rotate right
    pub fn ror(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);

        let carry = if self.has_flag(StatusFlags::CARRY) {
            1
        } else {
            0
        };
        let result = (carry << 7) | (fetched >> 1);

        self.set_flag(StatusFlags::CARRY, fetched & 0x01 != 0);
        self.set_flag(StatusFlags::ZERO, result == 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x80 != 0);

        if self.current_instruction().mode == AddrMode::Imp {
            self.a = result;
        } else {
            bus.write(self.addr_abs, result);
        }

        0
    }

    // Return from interrupt
    pub fn rti(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);

        let temp = bus.read(0x0100 + self.sp as u16, false);

        self.p |= temp & 0b1100_1111;

        self.sp = self.sp.wrapping_add(1);
        let lo = bus.read(0x0100 + self.sp as u16, false) as u16;
        self.sp = self.sp.wrapping_add(1);
        let hi = bus.read(0x0100 + self.sp as u16, false) as u16;

        self.pc = (hi << 8) | lo;

        0
    }

    // Return from subroutine
    pub fn rts(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let lo = bus.read(0x0100 + self.sp as u16, false) as u16;
        self.sp = self.sp.wrapping_add(1);
        let hi = bus.read(0x0100 + self.sp as u16, false) as u16;

        self.pc = (hi << 8) | lo;

        self.pc = self.pc.wrapping_add(1);

        0
    }

    // Subtract with carry
    pub fn sbc(&mut self, bus: &mut Bus) -> u8 {
        let fetched = self.fetch(bus);
        let inverted = !fetched;

        let carry = if self.has_flag(StatusFlags::CARRY) {
            1
        } else {
            0
        };
        let result = (self.a as u16)
            .wrapping_add(inverted as u16)
            .wrapping_add(carry);

        self.set_flag(StatusFlags::CARRY, result & 0xFF00 != 0);
        self.set_flag(StatusFlags::ZERO, result & 0x00FF == 0);
        self.set_flag(StatusFlags::NEGATIVE, result & 0x0080 != 0);
        self.set_flag(
            StatusFlags::OVERFLOW,
            calc_overflow(self.a as u16, inverted as u16, result),
        );

        self.a = result as u8;

        1
    }

    // Set carry
    pub fn sec(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(StatusFlags::CARRY, true);

        0
    }

    // Set decimal
    pub fn sed(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(StatusFlags::DECIMAL, true);

        0
    }

    // Set interrupt disable
    pub fn sei(&mut self, _bus: &mut Bus) -> u8 {
        self.set_flag(StatusFlags::INTERRUPT_DISABLE, true);

        0
    }

    // Store a
    pub fn sta(&mut self, bus: &mut Bus) -> u8 {
        bus.write(self.addr_abs, self.a);

        0
    }

    // Store x
    pub fn stx(&mut self, bus: &mut Bus) -> u8 {
        bus.write(self.addr_abs, self.x);

        0
    }

    // Store y
    pub fn sty(&mut self, bus: &mut Bus) -> u8 {
        bus.write(self.addr_abs, self.y);

        0
    }

    // Transfer a to x
    pub fn tax(&mut self, _bus: &mut Bus) -> u8 {
        self.x = self.a;

        self.set_flag(StatusFlags::ZERO, self.x == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.x & 0x80 != 0);

        0
    }

    // Transfer a to y
    pub fn tay(&mut self, _bus: &mut Bus) -> u8 {
        self.y = self.a;

        self.set_flag(StatusFlags::ZERO, self.y == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.y & 0x80 != 0);

        0
    }

    // Transfer stack pointer to x
    pub fn tsx(&mut self, _bus: &mut Bus) -> u8 {
        self.x = self.sp;

        self.set_flag(StatusFlags::ZERO, self.x == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.x & 0x80 != 0);

        0
    }

    // Transfer x to a
    pub fn txa(&mut self, _bus: &mut Bus) -> u8 {
        self.a = self.x;

        self.set_flag(StatusFlags::ZERO, self.a == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.a & 0x80 != 0);

        0
    }

    // Transfer x to stack pointer
    pub fn txs(&mut self, _bus: &mut Bus) -> u8 {
        self.sp = self.x;

        0
    }

    // Transfer y to a
    pub fn tya(&mut self, _bus: &mut Bus) -> u8 {
        self.a = self.y;

        self.set_flag(StatusFlags::ZERO, self.a == 0);
        self.set_flag(StatusFlags::NEGATIVE, self.a & 0x80 != 0);

        0
    }

    // Invalid operations
    pub fn xxx(&mut self, _bus: &mut Bus) -> u8 {
        0
    }
}

#[cfg(feature = "debug")]
pub struct CpuState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub p: u8,
    pub opcode: u8,
    pub cycles: u8,
}

#[cfg(feature = "debug")]
impl CPU {
    #[rustfmt::skip]
    pub fn get_state(&self) -> CpuState {
        let CPU { a, x, y, sp, pc, p, opcode, cycles, .. } = *self;
        CpuState {a, x, y, sp, pc, p, opcode, cycles}
    }

    pub fn step_to_next_instruction(&mut self, bus: &mut Bus) {
        self.step(bus);

        while self.cycles != 0 {
            self.step(bus);
        }
    }
}
