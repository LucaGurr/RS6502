use crate::define::{Cpu, Mem};
use crate::instructions::*;

pub fn execute(cpu: &mut Cpu, mem: &mut Mem) {
    if cpu.halted {
        return;
    }
    let instruction = fetch(cpu, mem);
    match instruction {
        0x00 => brk(cpu),
        0x18 => clc(cpu),
        0x4C => jmp_abs(cpu, mem),
        0x69 => adc_imm(cpu, mem),
        0x85 => sta_zp(cpu, mem),
        0x8D => sta_abs(cpu, mem),
        0x86 => stx_zp(cpu, mem),
        0xA0 => ldy_imm(cpu, mem),
        0xA2 => ldx_imm(cpu, mem),
        0xA5 => lda_zp(cpu, mem),
        0xA9 => lda_imm(cpu, mem),
        0xAD => lda_abs(cpu, mem),
        0xB5 => lda_zpx(cpu, mem),
        0xC9 => cmp_imm(cpu, mem),
        0xCA => dex(cpu),
        0xD0 => branch(cpu, mem, !cpu.z),
        0xE8 => inx(cpu),
        0xEA => nop(cpu),
        0xF0 => branch(cpu, mem, cpu.z),
        _ => panic!("Unsupported opcode ${instruction:02X} at ${:04X}", cpu.pc - 1),
    }
}

pub fn run(cpu: &mut Cpu, mem: &mut Mem) {
    while !cpu.halted {
        execute(cpu, mem);
    }
}

pub fn fetch(cpu: &mut Cpu, mem: &Mem) -> u8 {
    let data = mem.data[cpu.pc as usize];
    cpu.pc = (cpu.pc + 1) & 0xFFFF;
    data
}
