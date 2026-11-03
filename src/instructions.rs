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

pub fn lda_zpx(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch(cpu, mem).wrapping_add(cpu.x) as usize;
    cpu.a = mem.data[addr];
    set_nz(cpu, cpu.a);
    cpu.cycles += 4;
}

pub fn lda_abs(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch_word(cpu, mem) as usize;
    cpu.a = mem.data[addr];
    set_nz(cpu, cpu.a);
    cpu.cycles += 4;
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
    let value = fetch(cpu, mem);
    let sum = cpu.a as u16 + value as u16 + u16::from(cpu.c);
    let result = sum as u8;
    cpu.v = (!(cpu.a ^ value) & (cpu.a ^ result) & 0x80) != 0;
    cpu.c = sum > 0xFF;
    cpu.a = result;
    set_nz(cpu, result);
    cpu.cycles += 2;
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
        print!("{}", value as char);
    } else {
        mem.data[addr] = value;
    }
}
