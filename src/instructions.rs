use crate::define::{Cpu, Mem};
use crate::execute::fetch;

fn set_nz(cpu: &mut Cpu, value: u8) {
    cpu.z = value == 0;
    cpu.n = value & 0x80 != 0;
}

pub fn lda_imm(cpu: &mut Cpu, mem: &Mem) {
    cpu.a = fetch(cpu, mem);
    set_nz(cpu, cpu.a);
    cpu.cycles += 2;
}

pub fn lda_zp(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch(cpu, mem) as usize;
    cpu.a = mem.data[addr];
    set_nz(cpu, cpu.a);
    cpu.cycles += 3;
}

pub fn ldx_zp(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch(cpu, mem) as usize;
    cpu.x = mem.data[addr];
    set_nz(cpu, cpu.x);
    cpu.cycles += 3;
}

pub fn ldy_zp(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch(cpu, mem) as usize;
    cpu.y = mem.data[addr];
    set_nz(cpu, cpu.y);
    cpu.cycles += 3;
}

pub fn lda_zpx(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch(cpu, mem).wrapping_add(cpu.x) as usize;
    cpu.a = mem.data[addr];
    set_nz(cpu, cpu.a);
    cpu.cycles += 4;
}

pub fn ldy_zpx(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch(cpu, mem).wrapping_add(cpu.x) as usize;
    cpu.y = mem.data[addr];
    set_nz(cpu, cpu.y);
    cpu.cycles += 4;
}

pub fn ldx_zpy(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch(cpu, mem).wrapping_add(cpu.y) as usize;
    cpu.x = mem.data[addr];
    set_nz(cpu, cpu.x);
    cpu.cycles += 4;
}

pub fn lda_abs(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch_word(cpu, mem) as usize;
    cpu.a = mem.data[addr];
    set_nz(cpu, cpu.a);
    cpu.cycles += 4;
}

pub fn ldy_abs(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch_word(cpu, mem) as usize;
    cpu.y = mem.data[addr];
    set_nz(cpu, cpu.y);
    cpu.cycles += 4;
}

pub fn ldx_abs(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch_word(cpu, mem) as usize;
    cpu.x = mem.data[addr];
    set_nz(cpu, cpu.x);
    cpu.cycles += 4;
}

pub fn ldx_aby(cpu: &mut Cpu, mem: &Mem) {
    let base_addr = fetch_word(cpu, mem);
    let addr = base_addr.wrapping_add(cpu.y as u16) as usize;
    cpu.x = mem.data[addr];
    set_nz(cpu, cpu.x);
    cpu.cycles += 4 + u64::from((base_addr & 0xFF00) != (addr as u16 & 0xFF00));
}

pub fn lda_abx(cpu: &mut Cpu, mem: &Mem) {
    let base_addr = fetch_word(cpu, mem);
    let addr = base_addr.wrapping_add(cpu.x as u16) as usize;
    cpu.a = mem.data[addr];
    set_nz(cpu, cpu.a);
    cpu.cycles += 4 + u64::from((base_addr & 0xFF00) != (addr as u16 & 0xFF00));
}

pub fn lda_aby(cpu: &mut Cpu, mem: &Mem) {
    let base_addr = fetch_word(cpu, mem);
    let addr = base_addr.wrapping_add(cpu.y as u16) as usize;
    cpu.a = mem.data[addr];
    set_nz(cpu, cpu.a);
    cpu.cycles += 4 + u64::from((base_addr & 0xFF00) != (addr as u16 & 0xFF00));
}

pub fn lda_inx(cpu: &mut Cpu, mem: &Mem) {
    let zp_addr = fetch(cpu, mem).wrapping_add(cpu.x) as usize;
    let addr = mem.data[zp_addr] as u16 | (mem.data[zp_addr.wrapping_add(1) % 256] as u16) << 8;
    cpu.a = mem.data[addr as usize];
    set_nz(cpu, cpu.a);
    cpu.cycles += 6;
}

pub fn lda_iny(cpu: &mut Cpu, mem: &Mem) {
    let zp_addr = fetch(cpu, mem).wrapping_add(cpu.y) as usize;
    let addr = mem.data[zp_addr] as u16 | (mem.data[zp_addr.wrapping_add(1) % 256] as u16) << 8;
    cpu.a = mem.data[addr as usize];
    set_nz(cpu, cpu.a);
    cpu.cycles += 6;
}

pub fn ldx_imm(cpu: &mut Cpu, mem: &Mem) {
    cpu.x = fetch(cpu, mem);
    set_nz(cpu, cpu.x);
    cpu.cycles += 2;
}

pub fn ldy_imm(cpu: &mut Cpu, mem: &Mem) {
    cpu.y = fetch(cpu, mem);
    set_nz(cpu, cpu.y);
    cpu.cycles += 2;
}

pub fn sta_zp(cpu: &mut Cpu, mem: &mut Mem) {
    let addr = fetch(cpu, mem) as usize;
    mem.data[addr] = cpu.a;
    cpu.cycles += 3;
}

pub fn sta_abs(cpu: &mut Cpu, mem: &mut Mem) {
    let addr = fetch_word(cpu, mem) as usize;
    let value = cpu.a;
    store(mem, addr, value);
    cpu.cycles += 4;
}

pub fn stx_zp(cpu: &mut Cpu, mem: &mut Mem) {
    let addr = fetch(cpu, mem) as usize;
    mem.data[addr] = cpu.x;
    cpu.cycles += 3;
}

pub fn adc_imm(cpu: &mut Cpu, mem: &Mem) {
    let mem_value = fetch(cpu, mem);
    let sum = cpu.a as u16 + mem_value as u16 + cpu.c as u16;
    let result = sum as u8;
    cpu.v = (!(cpu.a ^ mem_value) & (cpu.a ^ result) & 0x80) != 0;
    cpu.c = sum > 0xFF;
    cpu.a = result;
    set_nz(cpu, result);
    cpu.cycles += 2;
}

pub fn adc_abs(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch_word(cpu, mem) as usize;
    let mem_value = mem.data[addr];
    let sum = cpu.a as u16 + mem_value as u16 + cpu.c as u16;
    let result = sum as u8;
    cpu.v = (!(cpu.a ^ mem_value) & (cpu.a ^ result) & 0x80) != 0;
    cpu.c = sum > 0xFF;
    cpu.a = result;
    set_nz(cpu, result);
    cpu.cycles += 4;
}

pub fn adc_abx(cpu: &mut Cpu, mem: &Mem) {
    let base_addr = fetch_word(cpu, mem);
    let addr = base_addr.wrapping_add(cpu.x as u16) as usize;
    let mem_value = mem.data[addr];
    let sum = cpu.a as u16 + mem_value as u16 + cpu.c as u16;
    let result = sum as u8;
    cpu.v = (!(cpu.a ^ mem_value) & (cpu.a ^ result) & 0x80) != 0;
    cpu.c = sum > 0xFF;
    cpu.a = result;
    set_nz(cpu, result);
    cpu.cycles += 4 + u64::from((base_addr & 0xFF00) != (addr as u16 & 0xFF00));
}

pub fn adc_aby(cpu: &mut Cpu, mem: &Mem) {
    let base_addr = fetch_word(cpu, mem);
    let addr = base_addr.wrapping_add(cpu.y as u16) as usize;
    let mem_value = mem.data[addr];
    let sum = cpu.a as u16 + mem_value as u16 + cpu.c as u16;
    let result = sum as u8;
    cpu.v = (!(cpu.a ^ mem_value) & (cpu.a ^ result) & 0x80) != 0;
    cpu.c = sum > 0xFF;
    cpu.a = result;
    set_nz(cpu, result);
    cpu.cycles += 4 + u64::from((base_addr & 0xFF00) != (addr as u16 & 0xFF00));
}   

pub fn inx(cpu: &mut Cpu) {
    cpu.x = cpu.x.wrapping_add(1);
    set_nz(cpu, cpu.x);
    cpu.cycles += 2;
}

pub fn dex(cpu: &mut Cpu) {
    cpu.x = cpu.x.wrapping_sub(1);
    set_nz(cpu, cpu.x);
    cpu.cycles += 2;
}

pub fn cmp_imm(cpu: &mut Cpu, mem: &Mem) {
    let value = fetch(cpu, mem);
    let result = cpu.a.wrapping_sub(value);
    cpu.c = cpu.a >= value;
    set_nz(cpu, result);
    cpu.cycles += 2;
}

pub fn branch(cpu: &mut Cpu, mem: &Mem, take: bool) {
    let offset = fetch(cpu, mem) as i8;
    cpu.cycles += 2;
    if take {
        let before = cpu.pc as u16;
        let target = before.wrapping_add(offset as i16 as u16);
        cpu.cycles += 1 + u64::from((before & 0xFF00) != (target & 0xFF00));
        cpu.pc = target as u32;
    }
}

pub fn jmp_abs(cpu: &mut Cpu, mem: &Mem) {
    cpu.pc = fetch_word(cpu, mem) as u32;
    cpu.cycles += 3;
}

pub fn nop(cpu: &mut Cpu) {
    cpu.cycles += 2;
}

pub fn clc(cpu: &mut Cpu) {
    cpu.c = false;
    cpu.cycles += 2;
}

pub fn brk(cpu: &mut Cpu) {
    // Emulator convention: BRK stops execution; hardware interrupt stack/vector
    // behavior is not modeled yet.
    cpu.halted = true;
    cpu.cycles += 7;
}

fn fetch_word(cpu: &mut Cpu, mem: &Mem) -> u16 {
    let lo = fetch(cpu, mem) as u16;
    let hi = fetch(cpu, mem) as u16;
    lo | hi << 8
}

fn store(mem: &mut Mem, addr: usize, value: u8) {
    if addr == 0xF001 {
        // arbitrary, system specific
        print!("{}", value as char);
    } else {
        mem.data[addr] = value;
    }
}
