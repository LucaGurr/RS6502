use crate::define::{Cpu, Mem};
use crate::instructions::*;

pub fn execute(cpu: &mut Cpu, mem: &mut Mem) {
    if cpu.halted {
        return;
    }
    let opcode = fetch(cpu, mem);
    match opcode {
        0x00 => brk(cpu),
        0x18 => clc(cpu),
        0x4C => jmp_abs(cpu, mem),
        0x69 => adc_imm(cpu, mem),
        0x85 => sta_zp(cpu, mem),
        0x86 => stx_zp(cpu, mem),
        0x8D => sta_abs(cpu, mem),
        0xA0 => ldy_imm(cpu, mem),
        0xA1 => lda_inx(cpu, mem),
        0xA2 => ldx_imm(cpu, mem),
        0xA4 => ldy_zp(cpu, mem),
        0xA5 => lda_zp(cpu, mem),
        0xA6 => ldx_zp(cpu, mem),
        0xA9 => lda_imm(cpu, mem),
        0xAC => ldy_abs(cpu, mem),
        0xAD => lda_abs(cpu, mem),
        0xAE => ldx_abs(cpu, mem),
        0xB1 => lda_iny(cpu, mem),
        0xB4 => ldy_zpx(cpu, mem),
        0xB5 => lda_zpx(cpu, mem),
        0xB6 => ldx_zpy(cpu, mem),
        0xB9 => lda_aby(cpu, mem),
        0xBD => lda_abx(cpu, mem),
        0xBE => ldx_aby(cpu, mem),
        0xC9 => cmp_imm(cpu, mem),
        0xCA => dex(cpu),
        0xD0 => branch(cpu, mem, !cpu.z),
        0xE8 => inx(cpu),
        0xEA => nop(cpu),
        0xF0 => branch(cpu, mem, cpu.z),
        _ => panic!(
            "Unsupported opcode ${opcode:02X} at ${:04X}",
            cpu.pc - 1
        ),
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
