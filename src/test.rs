use crate::define::init;
use crate::execute::{execute, run};

#[test]
fn test_lda_imm() {
    let (mut cpu, mut mem) = init();
    mem.data[0] = 0xA9;
    mem.data[1] = 0x42;
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 2);
    assert!(!cpu.z);
    assert!(!cpu.n);
}

#[test]
fn test_lda_imm_zero() {
    let (mut cpu, mut mem) = init();
    mem.data[0] = 0xA9;
    mem.data[1] = 0x00;
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0);
    assert!(cpu.z);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn test_lda_zp() {
    let (mut cpu, mut mem) = init();
    mem.data[0] = 0xA5;
    mem.data[1] = 0x42;
    mem.data[0x42] = 0x99;
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x99);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 3);
    assert!(!cpu.z);
    assert!(cpu.n);
}

#[test]
fn test_lda_zpx_wraps_within_zero_page() {
    let (mut cpu, mut mem) = init();
    cpu.x = 0x20;
    mem.data[0] = 0xB5;
    mem.data[1] = 0xF0;
    mem.data[0x10] = 0x80;
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.n);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn test_lda_absolute() {
    let (mut cpu, mut mem) = init();
    mem.data[0] = 0xAD;
    mem.data[1] = 0x34;
    mem.data[2] = 0x12;
    mem.data[0x1234] = 0x80;
    execute(&mut cpu, &mut mem);
    assert_eq!(cpu.a, 0x80);
    assert!(cpu.n);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn test_assembled_program_runs_with_expected_cycles() {
    let source = "LDA #$2A\nSTA $0200\nLDA $0200\nBRK\n";
    let (start, program) = crate::assembler::assemble(source).unwrap();
    let (mut cpu, mut mem) = init();
    let begin = start as usize;
    mem.data[begin..begin + program.len()].copy_from_slice(&program);
    cpu.pc = start as u32;
    run(&mut cpu, &mut mem);
    assert_eq!(mem.data[0x0200], 0x2A);
    assert_eq!(cpu.a, 0x2A);
    assert_eq!(cpu.cycles, 17); // LDA # 2 + STA abs 4 + LDA abs 4 + BRK 7
    assert!(cpu.halted);
}
