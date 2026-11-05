use crate::define::{init, Cpu, Mem};
use crate::execute::execute;

fn load_instruction(mem: &mut Mem, bytes: &[u8]) {
    mem.data[..bytes.len()].copy_from_slice(bytes);
}

fn load_instruction_at(cpu: &mut crate::define::Cpu, mem: &mut Mem, address: u16, bytes: &[u8]) {
    cpu.pc = address as u32;
    mem.data[address as usize..address as usize + bytes.len()].copy_from_slice(bytes);
}

fn assert_nz(cpu: &Cpu, value: u8) {
    assert_eq!(cpu.z, value == 0);
    assert_eq!(cpu.n, value & 0x80 != 0);
}

fn assert_unmodified_flags(opcode: u8, before: [bool; 7], cpu: &Cpu) {
    let after = [cpu.c, cpu.z, cpu.i, cpu.d, cpu.b, cpu.v, cpu.n];
    let preserved: &[usize] = match opcode {
        0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 | 0xA2 | 0xA6 | 0xB6 | 0xAE
        | 0xBE | 0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC | 0xAA | 0xA8 | 0x8A | 0x98 | 0xBA | 0xE8
        | 0xC8 | 0xCA | 0x88 | 0xE6 | 0xF6 | 0xEE | 0xFE | 0xC6 | 0xD6 | 0xCE | 0xDE | 0x29
        | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 | 0x09 | 0x05 | 0x15 | 0x0D | 0x1D
        | 0x19 | 0x01 | 0x11 | 0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
            &[0, 2, 3, 4, 5]
        }
        0x69 | 0x61 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x71 | 0xE9 | 0xE1 | 0xE5 | 0xF5
        | 0xED | 0xFD | 0xF9 | 0xF1 => &[2, 3, 4],
        0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 | 0xE0 | 0xE4 | 0xEC | 0xC0
        | 0xC4 | 0xCC => &[2, 3, 4, 5],
        0x24 | 0x2C => &[0, 2, 3, 4],
        0x0A | 0x06 | 0x16 | 0x0E | 0x1E | 0x4A | 0x46 | 0x56 | 0x4E | 0x5E | 0x2A | 0x26
        | 0x36 | 0x2E | 0x3E | 0x6A | 0x66 | 0x76 | 0x6E | 0x7E => &[2, 3, 4, 5],
        0x18 | 0x38 => &[1, 2, 3, 4, 5, 6],
        0x58 | 0x78 => &[0, 1, 3, 4, 5, 6],
        0xB8 => &[0, 1, 2, 3, 4, 6],
        0xD8 | 0xF8 => &[0, 1, 2, 4, 5, 6],
        0x00 => &[0, 1, 3, 4, 5, 6],
        0x68 => &[0, 2, 3, 4, 5],
        0xEA | 0x4C | 0x6C | 0x20 | 0x60 | 0x48 | 0x08 | 0x9A | 0x10 | 0x30 | 0x50 | 0x70
        | 0x90 | 0xB0 | 0xD0 | 0xF0 | 0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 | 0x86
        | 0x96 | 0x8E | 0x84 | 0x94 | 0x8C => &[0, 1, 2, 3, 4, 5, 6],
        _ => &[],
    };
    const FLAG_NAMES: [&str; 7] = ["C", "Z", "I", "D", "B", "V", "N"];
    for &index in preserved {
        assert_eq!(
            after[index], before[index],
            "opcode ${opcode:02X} unexpectedly changed {}",
            FLAG_NAMES[index]
        );
    }
}

fn assert_official_opcode_behavior(opcode: u8) {
    let (mut cpu, mut mem) = init();
    cpu.pc = 0x0200;
    cpu.a = 0x45;
    cpu.x = 1;
    cpu.y = 1;
    cpu.sp = 0xFD;
    cpu.i = true;
    cpu.d = true;
    cpu.b = true;
    cpu.v = true;

    let operand = match opcode {
        0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 => [0x02, 0],
        0x20 | 0x4C => [0x00, 0x30],
        0x6C => [0xFF, 0x30],
        0x01 | 0x11 | 0x21 | 0x31 | 0x41 | 0x51 | 0x61 | 0x71 | 0x81 | 0x91 | 0xA1 | 0xB1
        | 0xC1 | 0xD1 | 0xE1 | 0xF1 => [0x50, 0],
        0x19 | 0x1D | 0x39 | 0x3D | 0x59 | 0x5D | 0x79 | 0x7D | 0x99 | 0x9D | 0xB9 | 0xBD
        | 0xBE | 0xBC | 0xD9 | 0xDD | 0xF9 | 0xFD | 0xFE | 0xDE | 0x1E | 0x3E | 0x5E | 0x7E => {
            [0x3F, 0x30]
        }
        0x0D | 0x0E | 0x2C | 0x2D | 0x2E | 0x4D | 0x4E | 0x6D | 0x6E | 0x8C | 0x8D | 0x8E
        | 0xAC | 0xAD | 0xAE | 0xCD | 0xCE | 0xEC | 0xED | 0xEE | 0xCC => [0x00, 0x30],
        0x09 | 0x29 | 0x49 | 0x69 | 0x89 | 0xA0 | 0xA2 | 0xA9 | 0xC0 | 0xC9 | 0xE0 | 0xE9 => {
            [0x03, 0]
        }
        _ => [0x40, 0],
    };
    load_instruction_at(
        &mut cpu,
        &mut mem,
        0x0200,
        &[opcode, operand[0], operand[1]],
    );
    mem.data[0x40] = 0x03;
    mem.data[0x41] = 0x03;
    mem.data[0x50] = 0x3F;
    mem.data[0x51] = 0x00;
    mem.data[0x52] = 0x30;
    mem.data[0x3000] = 0x03;
    mem.data[0x303F] = 0x03;
    mem.data[0x3040] = 0x03;
    mem.data[0x30FF] = 0x34;

    match opcode {
        0x01 | 0x21 | 0x41 | 0x61 | 0x81 | 0xA1 | 0xC1 | 0xE1 => {
            mem.data[0x51] = 0x40;
        }
        0x11 | 0x31 | 0x51 | 0x71 | 0x91 | 0xB1 | 0xD1 | 0xF1 => {
            mem.data[0x50] = 0x3F;
            mem.data[0x51] = 0x30;
        }
        0x24 | 0x2C => {
            mem.data[0x40] = 0xC3;
            mem.data[0x3000] = 0xC3;
        }
        0x6C => {
            mem.data[0x3000] = 0x12;
        }
        0x06 | 0x16 | 0x0E | 0x1E | 0x26 | 0x36 | 0x2E | 0x3E | 0x46 | 0x56 | 0x4E | 0x5E
        | 0x66 | 0x76 | 0x6E | 0x7E | 0xC6 | 0xD6 | 0xCE | 0xDE | 0xE6 | 0xF6 | 0xEE | 0xFE => {
            let value = match opcode {
                0xC6 | 0xD6 | 0xCE | 0xDE => 0,
                _ => 0x81,
            };
            mem.data[0x40] = value;
            mem.data[0x3000] = value;
            mem.data[0x303F] = value;
            mem.data[0x3040] = value;
        }
        _ => {}
    }

    if matches!(
        opcode,
        0x2A | 0x2E | 0x26 | 0x36 | 0x3E | 0x6A | 0x66 | 0x76 | 0x6E | 0x7E
    ) {
        cpu.c = true;
    }
    if matches!(
        opcode,
        0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1
    ) {
        cpu.c = true;
    }
    if matches!(opcode, 0x10 | 0x50 | 0x90 | 0xD0) {
        cpu.c = false;
        cpu.n = false;
        cpu.v = false;
        cpu.z = false;
    }
    if matches!(opcode, 0x30 | 0x70 | 0xB0 | 0xF0) {
        cpu.c = true;
        cpu.n = true;
        cpu.v = true;
        cpu.z = true;
    }
    if opcode == 0x40 {
        cpu.sp = 0xFA;
        mem.data[0x01FB] = 0xCF;
        mem.data[0x01FC] = 0x34;
        mem.data[0x01FD] = 0x12;
    }
    if opcode == 0x60 {
        cpu.sp = 0xFB;
        mem.data[0x01FC] = 0x02;
        mem.data[0x01FD] = 0x02;
    }
    if opcode == 0x68 {
        cpu.sp = 0xFC;
        mem.data[0x01FD] = 0x80;
    }
    if opcode == 0x28 {
        cpu.sp = 0xFC;
        mem.data[0x01FD] = 0xCF;
    }
    if opcode == 0x78 {
        cpu.i = false;
    }
    if opcode == 0xD8 {
        cpu.d = true;
    }
    if opcode == 0xB8 {
        cpu.v = true;
    }
    if opcode == 0xF8 {
        cpu.d = false;
    }
    if opcode == 0x18 {
        cpu.c = true;
    }
    if opcode == 0x00 {
        mem.data[0xFFFE] = 0x34;
        mem.data[0xFFFF] = 0x12;
    }

    let flags_before = [cpu.c, cpu.z, cpu.i, cpu.d, cpu.b, cpu.v, cpu.n];
    execute(&mut cpu, &mut mem);

    let cycles = match opcode {
        0x00 => 7,
        0x01 | 0x21 | 0x41 | 0x61 | 0x81 | 0xA1 | 0xC1 | 0xE1 => 6,
        0x11 | 0x31 | 0x51 | 0x71 | 0x91 | 0xB1 | 0xD1 | 0xF1 => 5,
        0x20 => 6,
        0x40 | 0x60 => 6,
        0x48 | 0x08 => 3,
        0x68 | 0x28 => 4,
        0x4C => 3,
        0x6C => 5,
        0x0E | 0x2E | 0x4E | 0x6E | 0xCE | 0xEE => 6,
        0x1E | 0x3E | 0x5E | 0x7E | 0xDE | 0xFE => 7,
        0x06 | 0x26 | 0x46 | 0x66 | 0xC6 | 0xE6 => 5,
        0x16 | 0x36 | 0x56 | 0x76 | 0xD6 | 0xF6 => 6,
        0x0A | 0x2A | 0x4A | 0x6A | 0x18 | 0x38 | 0x58 | 0x78 | 0x98 | 0x8A | 0xA8 | 0xAA
        | 0xB8 | 0x9A | 0xBA | 0xC8 | 0xCA | 0xD8 | 0xE8 | 0x88 | 0xEA | 0xF8 => 2,
        0x85 | 0x86 | 0x84 | 0xA5 | 0xA6 | 0xA4 | 0xC5 | 0xC4 | 0xE4 | 0x65 | 0xE5 | 0x25
        | 0x05 | 0x45 => 3,
        0x95 | 0x96 | 0x94 | 0xB5 | 0xB6 | 0xB4 | 0xD5 | 0x75 | 0xF5 | 0x35 | 0x15 | 0x55 => 4,
        0x8D | 0x8E | 0x8C | 0xAD | 0xAE | 0xAC | 0xCD | 0xCC | 0xEC | 0x6D | 0xED | 0x2D
        | 0x0D | 0x4D => 4,
        0x99 | 0x9D => 5,
        0xBD | 0xB9 | 0xBE | 0xBC | 0xDD | 0xD9 | 0x7D | 0x79 | 0xFD | 0xF9 | 0x3D | 0x39
        | 0x1D | 0x19 | 0x5D | 0x59 => 4,
        0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 => 3,
        0x09 | 0x29 | 0x49 | 0x69 | 0xA0 | 0xA2 | 0xA9 | 0xC0 | 0xC9 | 0xE0 | 0xE9 => 2,
        _ => 2,
    };
    assert_eq!(cpu.cycles, cycles, "opcode ${opcode:02X} cycle count");

    let instruction_bytes = match opcode {
        0x00 => 2,
        0x0D | 0x0E | 0x1D | 0x1E | 0x19 | 0x2C | 0x2D | 0x2E | 0x3D | 0x3E | 0x39 | 0x4C
        | 0x4D | 0x4E | 0x5D | 0x5E | 0x59 | 0x6C | 0x6D | 0x6E | 0x7D | 0x7E | 0x79 | 0x8C
        | 0x8D | 0x8E | 0x99 | 0x9D | 0xAC | 0xAD | 0xAE | 0xB9 | 0xBC | 0xBD | 0xBE | 0xCC
        | 0xCD | 0xCE | 0xD9 | 0xDD | 0xDE | 0xEC | 0xED | 0xEE | 0xF9 | 0xFD | 0xFE | 0x20 => 3,
        0x08 | 0x0A | 0x18 | 0x28 | 0x2A | 0x38 | 0x40 | 0x48 | 0x4A | 0x58 | 0x60 | 0x68
        | 0x6A | 0x78 | 0x88 | 0x8A | 0x98 | 0x9A | 0xA8 | 0xAA | 0xB8 | 0xBA | 0xC8 | 0xCA
        | 0xD8 | 0xE8 | 0xEA | 0xF8 => 1,
        _ => 2,
    };
    let expected_pc = match opcode {
        0x00 => 0x1234,
        0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 => 0x0204,
        0x20 | 0x4C => 0x3000,
        0x40 | 0x6C => 0x1234,
        0x60 => 0x0203,
        _ => 0x0200 + instruction_bytes,
    };
    assert_eq!(cpu.pc, expected_pc, "opcode ${opcode:02X} program counter");

    match opcode {
        0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
            assert_eq!(cpu.a, 0x03, "opcode ${opcode:02X} accumulator result");
            assert_nz(&cpu, 0x03);
        }
        0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
            assert_eq!(cpu.x, 0x03, "opcode ${opcode:02X} X result");
            assert_nz(&cpu, 0x03);
        }
        0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
            assert_eq!(cpu.y, 0x03, "opcode ${opcode:02X} Y result");
            assert_nz(&cpu, 0x03);
        }
        0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
            let address = match opcode {
                0x85 => 0x40,
                0x95 => 0x41,
                0x8D => 0x3000,
                0x9D | 0x99 => 0x3040,
                0x81 | 0x91 => 0x3040,
                _ => unreachable!(),
            };
            assert_eq!(
                mem.data[address], cpu.a,
                "opcode ${opcode:02X} stored value"
            );
            assert_eq!(cpu.a, 0x45);
        }
        0x86 | 0x96 | 0x8E => {
            let address = match opcode {
                0x86 => 0x40,
                0x96 => 0x41,
                _ => 0x3000,
            };
            assert_eq!(mem.data[address], cpu.x);
            assert_eq!(cpu.x, 1);
        }
        0x84 | 0x94 | 0x8C => {
            let address = match opcode {
                0x84 => 0x40,
                0x94 => 0x41,
                _ => 0x3000,
            };
            assert_eq!(mem.data[address], cpu.y);
            assert_eq!(cpu.y, 1);
        }
        0xAA => {
            assert_eq!(cpu.x, 0x45);
            assert_nz(&cpu, 0x45);
        }
        0xA8 => {
            assert_eq!(cpu.y, 0x45);
            assert_nz(&cpu, 0x45);
        }
        0x8A => {
            assert_eq!(cpu.a, 1);
            assert_nz(&cpu, 1);
        }
        0x98 => {
            assert_eq!(cpu.a, 1);
            assert_nz(&cpu, 1);
        }
        0xBA => {
            assert_eq!(cpu.x, 0xFD);
            assert_nz(&cpu, 0xFD);
        }
        0x9A => {
            assert_eq!(cpu.sp, 1);
        }
        0x69 => {
            assert_eq!(cpu.a, 0x48);
            assert!(!cpu.c && !cpu.v);
            assert_nz(&cpu, 0x48);
        }
        0x61 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x71 => {
            assert_eq!(cpu.a, 0x48);
            assert!(!cpu.c && !cpu.v);
            assert_nz(&cpu, 0x48);
        }
        0xE9 | 0xE1 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xF1 => {
            assert_eq!(cpu.a, 0x42);
            assert!(cpu.c);
            assert!(!cpu.v);
            assert_nz(&cpu, 0x42);
        }
        0xE8 => {
            assert_eq!(cpu.x, 2);
            assert_nz(&cpu, 2);
        }
        0xC8 => {
            assert_eq!(cpu.y, 2);
            assert_nz(&cpu, 2);
        }
        0xCA => {
            assert_eq!(cpu.x, 0);
            assert_nz(&cpu, 0);
        }
        0x88 => {
            assert_eq!(cpu.y, 0);
            assert_nz(&cpu, 0);
        }
        0xE6 | 0xF6 | 0xEE | 0xFE => {
            let address = match opcode {
                0xE6 => 0x40,
                0xF6 => 0x40,
                0xEE => 0x3000,
                _ => 0x3040,
            };
            assert_eq!(mem.data[address], 0x82);
            assert_nz(&cpu, 0x82);
        }
        0xC6 | 0xD6 | 0xCE | 0xDE => {
            let address = match opcode {
                0xC6 => 0x40,
                0xD6 => 0x40,
                0xCE => 0x3000,
                _ => 0x3040,
            };
            assert_eq!(mem.data[address], 0xFF);
            assert_nz(&cpu, 0xFF);
        }
        0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
            assert_eq!(cpu.a, 0x01);
            assert_nz(&cpu, 0x01);
        }
        0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
            assert_eq!(cpu.a, 0x47);
            assert_nz(&cpu, 0x47);
        }
        0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
            assert_eq!(cpu.a, 0x46);
            assert_nz(&cpu, 0x46);
        }
        0x24 | 0x2C => {
            assert!(!cpu.z && cpu.n && cpu.v);
        }
        0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
            assert!(cpu.c);
            assert!(!cpu.z && !cpu.n);
            assert_eq!(cpu.a, 0x45);
        }
        0xE0 | 0xE4 | 0xEC => {
            assert!(!cpu.c && !cpu.z && cpu.n);
        }
        0xC0 | 0xC4 | 0xCC => {
            assert!(!cpu.c && !cpu.z && cpu.n);
        }
        0x0A | 0x06 | 0x16 | 0x0E | 0x1E => {
            if opcode == 0x0A {
                assert_eq!(cpu.a, 0x8A);
            } else {
                let address = match opcode {
                    0x06 => 0x40,
                    0x16 => 0x40,
                    0x0E => 0x3000,
                    _ => 0x3040,
                };
                assert_eq!(mem.data[address], 0x02);
            }
            assert_eq!(cpu.c, opcode != 0x0A);
            assert!(!cpu.z && !cpu.n);
        }
        0x4A | 0x46 | 0x56 | 0x4E | 0x5E => {
            if opcode == 0x4A {
                assert_eq!(cpu.a, 0x22);
            } else {
                let address = match opcode {
                    0x46 => 0x40,
                    0x56 => 0x40,
                    0x4E => 0x3000,
                    _ => 0x3040,
                };
                assert_eq!(mem.data[address], 0x40);
            }
            assert!(cpu.c);
            assert!(!cpu.z && !cpu.n);
        }
        0x2A | 0x26 | 0x36 | 0x2E | 0x3E => {
            if opcode == 0x2A {
                assert_eq!(cpu.a, 0x8B);
            } else {
                let address = match opcode {
                    0x26 => 0x40,
                    0x36 => 0x40,
                    0x2E => 0x3000,
                    _ => 0x3040,
                };
                assert_eq!(mem.data[address], 0x03);
            }
            assert_eq!(cpu.c, opcode != 0x2A);
            assert!(!cpu.z && !cpu.n);
        }
        0x6A | 0x66 | 0x76 | 0x6E | 0x7E => {
            if opcode == 0x6A {
                assert_eq!(cpu.a, 0xA2);
            } else {
                let address = match opcode {
                    0x66 => 0x40,
                    0x76 => 0x40,
                    0x6E => 0x3000,
                    _ => 0x3040,
                };
                assert_eq!(mem.data[address], 0xC0);
            }
            assert!(cpu.c);
            assert!(!cpu.z && cpu.n);
        }
        0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 => {
            assert_eq!(cpu.pc, 0x0204);
        }
        0x4C => assert_eq!(cpu.pc, 0x3000),
        0x6C => assert_eq!(cpu.pc, 0x1234),
        0x20 => {
            assert_eq!(cpu.pc, 0x3000);
            assert_eq!(cpu.sp, 0xFB);
            assert_eq!(mem.data[0x01FD], 0x02);
            assert_eq!(mem.data[0x01FC], 0x02);
        }
        0x60 => {
            assert_eq!(cpu.pc, 0x0203);
            assert_eq!(cpu.sp, 0xFD);
        }
        0x40 => {
            assert_eq!(cpu.pc, 0x1234);
            assert_eq!(cpu.sp, 0xFD);
            assert!(cpu.c && cpu.z && cpu.i && cpu.d && cpu.v && cpu.n);
            assert!(!cpu.b);
        }
        0x48 => {
            assert_eq!(mem.data[0x01FD], 0x45);
            assert_eq!(cpu.sp, 0xFC);
        }
        0x08 => {
            assert_eq!(mem.data[0x01FD], 0x7C);
            assert_eq!(cpu.sp, 0xFC);
        }
        0x68 => {
            assert_eq!(cpu.a, 0x80);
            assert_nz(&cpu, 0x80);
            assert_eq!(cpu.sp, 0xFD);
        }
        0x28 => {
            assert!(cpu.c && cpu.z && cpu.i && cpu.d && cpu.v && cpu.n);
            assert!(!cpu.b);
            assert_eq!(cpu.sp, 0xFD);
        }
        0x18 => assert!(!cpu.c),
        0x38 => assert!(cpu.c),
        0x58 => assert!(!cpu.i),
        0x78 => assert!(cpu.i),
        0xB8 => assert!(!cpu.v),
        0xD8 => assert!(!cpu.d),
        0xF8 => assert!(cpu.d),
        0x00 => {
            assert_eq!(cpu.sp, 0xFA);
            assert_eq!(mem.data[0x01FD], 0x02);
            assert_eq!(mem.data[0x01FC], 0x02);
            assert_eq!(mem.data[0x01FB] & 0x30, 0x30);
            assert!(cpu.i);
        }
        0xEA => {
            assert_eq!(cpu.a, 0x45);
            assert_eq!(cpu.x, 1);
            assert_eq!(cpu.y, 1);
        }
        _ => panic!("missing semantic expectation for opcode ${opcode:02X}"),
    }
    assert_unmodified_flags(opcode, flags_before, &cpu);
}

macro_rules! official_opcode_test {
    ($name:ident, $opcode:literal) => {
        #[test]
        fn $name() {
            assert_official_opcode_behavior($opcode);
        }
    };
}

official_opcode_test!(brk_has_expected_effects, 0x00);
official_opcode_test!(ora_indexed_indirect_has_expected_effects, 0x01);
official_opcode_test!(ora_zero_page_has_expected_effects, 0x05);
official_opcode_test!(asl_zero_page_has_expected_effects, 0x06);
official_opcode_test!(php_has_expected_effects, 0x08);
official_opcode_test!(ora_immediate_has_expected_effects, 0x09);
official_opcode_test!(asl_accumulator_has_expected_effects, 0x0A);
official_opcode_test!(ora_absolute_has_expected_effects, 0x0D);
official_opcode_test!(asl_absolute_has_expected_effects, 0x0E);
official_opcode_test!(bpl_has_expected_effects, 0x10);
official_opcode_test!(ora_indirect_indexed_has_expected_effects, 0x11);
official_opcode_test!(ora_zero_page_x_has_expected_effects, 0x15);
official_opcode_test!(asl_zero_page_x_has_expected_effects, 0x16);
official_opcode_test!(clc_has_expected_effects, 0x18);
official_opcode_test!(ora_absolute_y_has_expected_effects, 0x19);
official_opcode_test!(ora_absolute_x_has_expected_effects, 0x1D);
official_opcode_test!(asl_absolute_x_has_expected_effects, 0x1E);
official_opcode_test!(jsr_has_expected_effects, 0x20);
official_opcode_test!(and_indexed_indirect_has_expected_effects, 0x21);
official_opcode_test!(bit_zero_page_has_expected_effects, 0x24);
official_opcode_test!(and_zero_page_has_expected_effects, 0x25);
official_opcode_test!(rol_zero_page_has_expected_effects, 0x26);
official_opcode_test!(plp_has_expected_effects, 0x28);
official_opcode_test!(and_immediate_has_expected_effects, 0x29);
official_opcode_test!(rol_accumulator_has_expected_effects, 0x2A);
official_opcode_test!(bit_absolute_has_expected_effects, 0x2C);
official_opcode_test!(and_absolute_has_expected_effects, 0x2D);
official_opcode_test!(rol_absolute_has_expected_effects, 0x2E);
official_opcode_test!(bmi_has_expected_effects, 0x30);
official_opcode_test!(and_indirect_indexed_has_expected_effects, 0x31);
official_opcode_test!(and_zero_page_x_has_expected_effects, 0x35);
official_opcode_test!(rol_zero_page_x_has_expected_effects, 0x36);
official_opcode_test!(sec_has_expected_effects, 0x38);
official_opcode_test!(and_absolute_y_has_expected_effects, 0x39);
official_opcode_test!(and_absolute_x_has_expected_effects, 0x3D);
official_opcode_test!(rol_absolute_x_has_expected_effects, 0x3E);
official_opcode_test!(rti_has_expected_effects, 0x40);
official_opcode_test!(eor_indexed_indirect_has_expected_effects, 0x41);
official_opcode_test!(eor_zero_page_has_expected_effects, 0x45);
official_opcode_test!(lsr_zero_page_has_expected_effects, 0x46);
official_opcode_test!(pha_has_expected_effects, 0x48);
official_opcode_test!(eor_immediate_has_expected_effects, 0x49);
official_opcode_test!(lsr_accumulator_has_expected_effects, 0x4A);
official_opcode_test!(jmp_absolute_has_expected_effects, 0x4C);
official_opcode_test!(eor_absolute_has_expected_effects, 0x4D);
official_opcode_test!(lsr_absolute_has_expected_effects, 0x4E);
official_opcode_test!(bvc_has_expected_effects, 0x50);
official_opcode_test!(eor_indirect_indexed_has_expected_effects, 0x51);
official_opcode_test!(eor_zero_page_x_has_expected_effects, 0x55);
official_opcode_test!(lsr_zero_page_x_has_expected_effects, 0x56);
official_opcode_test!(cli_has_expected_effects, 0x58);
official_opcode_test!(eor_absolute_y_has_expected_effects, 0x59);
official_opcode_test!(eor_absolute_x_has_expected_effects, 0x5D);
official_opcode_test!(lsr_absolute_x_has_expected_effects, 0x5E);
official_opcode_test!(rts_has_expected_effects, 0x60);
official_opcode_test!(adc_indexed_indirect_has_expected_effects, 0x61);
official_opcode_test!(adc_zero_page_has_expected_effects, 0x65);
official_opcode_test!(ror_zero_page_has_expected_effects, 0x66);
official_opcode_test!(pla_has_expected_effects, 0x68);
official_opcode_test!(adc_immediate_has_expected_effects, 0x69);
official_opcode_test!(ror_accumulator_has_expected_effects, 0x6A);
official_opcode_test!(jmp_indirect_has_expected_effects, 0x6C);
official_opcode_test!(adc_absolute_has_expected_effects, 0x6D);
official_opcode_test!(ror_absolute_has_expected_effects, 0x6E);
official_opcode_test!(bvs_has_expected_effects, 0x70);
official_opcode_test!(adc_indirect_indexed_has_expected_effects, 0x71);
official_opcode_test!(adc_zero_page_x_has_expected_effects, 0x75);
official_opcode_test!(ror_zero_page_x_has_expected_effects, 0x76);
official_opcode_test!(sei_has_expected_effects, 0x78);
official_opcode_test!(adc_absolute_y_has_expected_effects, 0x79);
official_opcode_test!(adc_absolute_x_has_expected_effects, 0x7D);
official_opcode_test!(ror_absolute_x_has_expected_effects, 0x7E);
official_opcode_test!(sta_indexed_indirect_has_expected_effects, 0x81);
official_opcode_test!(sty_zero_page_has_expected_effects, 0x84);
official_opcode_test!(sta_zero_page_has_expected_effects, 0x85);
official_opcode_test!(stx_zero_page_has_expected_effects, 0x86);
official_opcode_test!(dey_has_expected_effects, 0x88);
official_opcode_test!(txa_has_expected_effects, 0x8A);
official_opcode_test!(sty_absolute_has_expected_effects, 0x8C);
official_opcode_test!(sta_absolute_has_expected_effects, 0x8D);
official_opcode_test!(stx_absolute_has_expected_effects, 0x8E);
official_opcode_test!(bcc_has_expected_effects, 0x90);
official_opcode_test!(sta_indirect_indexed_has_expected_effects, 0x91);
official_opcode_test!(sty_zero_page_x_has_expected_effects, 0x94);
official_opcode_test!(sta_zero_page_x_has_expected_effects, 0x95);
official_opcode_test!(stx_zero_page_y_has_expected_effects, 0x96);
official_opcode_test!(tya_has_expected_effects, 0x98);
official_opcode_test!(sta_absolute_y_has_expected_effects, 0x99);
official_opcode_test!(txs_has_expected_effects, 0x9A);
official_opcode_test!(sta_absolute_x_has_expected_effects, 0x9D);
official_opcode_test!(ldy_immediate_has_expected_effects, 0xA0);
official_opcode_test!(lda_indexed_indirect_has_expected_effects, 0xA1);
official_opcode_test!(ldx_immediate_has_expected_effects, 0xA2);
official_opcode_test!(ldy_zero_page_has_expected_effects, 0xA4);
official_opcode_test!(lda_zero_page_has_expected_effects, 0xA5);
official_opcode_test!(ldx_zero_page_has_expected_effects, 0xA6);
official_opcode_test!(tay_has_expected_effects, 0xA8);
official_opcode_test!(lda_immediate_has_expected_effects, 0xA9);
official_opcode_test!(tax_has_expected_effects, 0xAA);
official_opcode_test!(ldy_absolute_has_expected_effects, 0xAC);
official_opcode_test!(lda_absolute_has_expected_effects, 0xAD);
official_opcode_test!(ldx_absolute_has_expected_effects, 0xAE);
official_opcode_test!(bcs_has_expected_effects, 0xB0);
official_opcode_test!(lda_indirect_indexed_has_expected_effects, 0xB1);
official_opcode_test!(ldy_zero_page_x_has_expected_effects, 0xB4);
official_opcode_test!(lda_zero_page_x_has_expected_effects, 0xB5);
official_opcode_test!(ldx_zero_page_y_has_expected_effects, 0xB6);
official_opcode_test!(clv_has_expected_effects, 0xB8);
official_opcode_test!(lda_absolute_y_has_expected_effects, 0xB9);
official_opcode_test!(tsx_has_expected_effects, 0xBA);
official_opcode_test!(ldy_absolute_x_has_expected_effects, 0xBC);
official_opcode_test!(lda_absolute_x_has_expected_effects, 0xBD);
official_opcode_test!(ldx_absolute_y_has_expected_effects, 0xBE);
official_opcode_test!(cpy_immediate_has_expected_effects, 0xC0);
official_opcode_test!(cmp_indexed_indirect_has_expected_effects, 0xC1);
official_opcode_test!(cpy_zero_page_has_expected_effects, 0xC4);
official_opcode_test!(cmp_zero_page_has_expected_effects, 0xC5);
official_opcode_test!(dec_zero_page_has_expected_effects, 0xC6);
official_opcode_test!(iny_has_expected_effects, 0xC8);
official_opcode_test!(cmp_immediate_has_expected_effects, 0xC9);
official_opcode_test!(dex_has_expected_effects, 0xCA);
official_opcode_test!(cpy_absolute_has_expected_effects, 0xCC);
official_opcode_test!(cmp_absolute_has_expected_effects, 0xCD);
official_opcode_test!(dec_absolute_has_expected_effects, 0xCE);
official_opcode_test!(bne_has_expected_effects, 0xD0);
official_opcode_test!(cmp_indirect_indexed_has_expected_effects, 0xD1);
official_opcode_test!(cmp_zero_page_x_has_expected_effects, 0xD5);
official_opcode_test!(dec_zero_page_x_has_expected_effects, 0xD6);
official_opcode_test!(cld_has_expected_effects, 0xD8);
official_opcode_test!(cmp_absolute_y_has_expected_effects, 0xD9);
official_opcode_test!(cmp_absolute_x_has_expected_effects, 0xDD);
official_opcode_test!(dec_absolute_x_has_expected_effects, 0xDE);
official_opcode_test!(cpx_immediate_has_expected_effects, 0xE0);
official_opcode_test!(sbc_indexed_indirect_has_expected_effects, 0xE1);
official_opcode_test!(cpx_zero_page_has_expected_effects, 0xE4);
official_opcode_test!(sbc_zero_page_has_expected_effects, 0xE5);
official_opcode_test!(inc_zero_page_has_expected_effects, 0xE6);
official_opcode_test!(inx_has_expected_effects, 0xE8);
official_opcode_test!(sbc_immediate_has_expected_effects, 0xE9);
official_opcode_test!(nop_has_expected_effects, 0xEA);
official_opcode_test!(cpx_absolute_has_expected_effects, 0xEC);
official_opcode_test!(sbc_absolute_has_expected_effects, 0xED);
official_opcode_test!(inc_absolute_has_expected_effects, 0xEE);
official_opcode_test!(beq_has_expected_effects, 0xF0);
official_opcode_test!(sbc_indirect_indexed_has_expected_effects, 0xF1);
official_opcode_test!(sbc_zero_page_x_has_expected_effects, 0xF5);
official_opcode_test!(inc_zero_page_x_has_expected_effects, 0xF6);
official_opcode_test!(sed_has_expected_effects, 0xF8);
official_opcode_test!(sbc_absolute_y_has_expected_effects, 0xF9);
official_opcode_test!(sbc_absolute_x_has_expected_effects, 0xFD);
official_opcode_test!(inc_absolute_x_has_expected_effects, 0xFE);

fn assert_indexed_read_page_cross(opcode: u8, mode: u8) {
    let (mut cpu, mut mem) = init();
    cpu.pc = 0x0200;
    cpu.a = 1;
    cpu.x = 1;
    cpu.y = 1;
    mem.data[0x2100] = 1;
    match mode {
        0 => mem.data[0x0200..0x0203].copy_from_slice(&[opcode, 0xFF, 0x20]),
        1 => mem.data[0x0200..0x0203].copy_from_slice(&[opcode, 0xFF, 0x20]),
        2 => {
            mem.data[0x0200..0x0202].copy_from_slice(&[opcode, 0x50]);
            mem.data[0x50] = 0xFF;
            mem.data[0x51] = 0x20;
        }
        _ => unreachable!(),
    }

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.cycles, if mode == 2 { 6 } else { 5 });
    assert_eq!(cpu.pc, if mode == 2 { 0x0202 } else { 0x0203 });
}

macro_rules! indexed_read_page_cross_test {
    ($name:ident, $opcode:literal, $mode:literal) => {
        #[test]
        fn $name() {
            assert_indexed_read_page_cross($opcode, $mode);
        }
    };
}

indexed_read_page_cross_test!(lda_absolute_x_page_cross_costs_one_cycle, 0xBD, 0);
indexed_read_page_cross_test!(adc_absolute_x_page_cross_costs_one_cycle, 0x7D, 0);
indexed_read_page_cross_test!(sbc_absolute_x_page_cross_costs_one_cycle, 0xFD, 0);
indexed_read_page_cross_test!(cmp_absolute_x_page_cross_costs_one_cycle, 0xDD, 0);
indexed_read_page_cross_test!(and_absolute_x_page_cross_costs_one_cycle, 0x3D, 0);
indexed_read_page_cross_test!(ora_absolute_x_page_cross_costs_one_cycle, 0x1D, 0);
indexed_read_page_cross_test!(eor_absolute_x_page_cross_costs_one_cycle, 0x5D, 0);
indexed_read_page_cross_test!(ldy_absolute_x_page_cross_costs_one_cycle, 0xBC, 0);
indexed_read_page_cross_test!(lda_absolute_y_page_cross_costs_one_cycle, 0xB9, 1);
indexed_read_page_cross_test!(adc_absolute_y_page_cross_costs_one_cycle, 0x79, 1);
indexed_read_page_cross_test!(sbc_absolute_y_page_cross_costs_one_cycle, 0xF9, 1);
indexed_read_page_cross_test!(cmp_absolute_y_page_cross_costs_one_cycle, 0xD9, 1);
indexed_read_page_cross_test!(and_absolute_y_page_cross_costs_one_cycle, 0x39, 1);
indexed_read_page_cross_test!(ora_absolute_y_page_cross_costs_one_cycle, 0x19, 1);
indexed_read_page_cross_test!(eor_absolute_y_page_cross_costs_one_cycle, 0x59, 1);
indexed_read_page_cross_test!(ldx_absolute_y_page_cross_costs_one_cycle, 0xBE, 1);
indexed_read_page_cross_test!(lda_indirect_indexed_page_cross_costs_one_cycle, 0xB1, 2);
indexed_read_page_cross_test!(adc_indirect_indexed_page_cross_costs_one_cycle, 0x71, 2);
indexed_read_page_cross_test!(sbc_indirect_indexed_page_cross_costs_one_cycle, 0xF1, 2);
indexed_read_page_cross_test!(cmp_indirect_indexed_page_cross_costs_one_cycle, 0xD1, 2);
indexed_read_page_cross_test!(and_indirect_indexed_page_cross_costs_one_cycle, 0x31, 2);
indexed_read_page_cross_test!(ora_indirect_indexed_page_cross_costs_one_cycle, 0x11, 2);
indexed_read_page_cross_test!(eor_indirect_indexed_page_cross_costs_one_cycle, 0x51, 2);

macro_rules! branch_not_taken_test {
    ($name:ident, $opcode:literal, $set_flag:expr) => {
        #[test]
        fn $name() {
            let (mut cpu, mut mem) = init();
            mem.data[0..2].copy_from_slice(&[$opcode, 0xFE]);
            ($set_flag)(&mut cpu);

            execute(&mut cpu, &mut mem);

            assert_eq!(cpu.pc, 2);
            assert_eq!(cpu.cycles, 2);
        }
    };
}

branch_not_taken_test!(
    bcc_does_not_branch_when_carry_is_set,
    0x90,
    |cpu: &mut Cpu| cpu.c = true
);
branch_not_taken_test!(
    bcs_does_not_branch_when_carry_is_clear,
    0xB0,
    |cpu: &mut Cpu| cpu.c = false
);
branch_not_taken_test!(
    bmi_does_not_branch_when_negative_is_clear,
    0x30,
    |cpu: &mut Cpu| cpu.n = false
);
branch_not_taken_test!(
    bpl_does_not_branch_when_negative_is_set,
    0x10,
    |cpu: &mut Cpu| cpu.n = true
);
branch_not_taken_test!(
    bvc_does_not_branch_when_overflow_is_set,
    0x50,
    |cpu: &mut Cpu| cpu.v = true
);
branch_not_taken_test!(
    bvs_does_not_branch_when_overflow_is_clear,
    0x70,
    |cpu: &mut Cpu| cpu.v = false
);

#[test]
fn lda_immediate_loads_value_and_updates_negative_flag() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xA9, 0x80]);
    cpu.c = true;
    cpu.v = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x80);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 2);
    assert!(!cpu.z);
    assert!(cpu.n);
    assert!(cpu.c);
    assert!(cpu.v);
}

#[test]
fn lda_immediate_zero_sets_zero_and_clears_negative() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xA9, 0x00]);
    cpu.n = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0);
    assert!(cpu.z);
    assert!(!cpu.n);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn lda_zero_page_loads_value_and_updates_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xA5, 0x42]);
    mem.data[0x42] = 0x99;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x99);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 3);
    assert!(!cpu.z);
    assert!(cpu.n);
}

#[test]
fn lda_zero_page_x_wraps_at_zero_page_boundary() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xB5, 0xF0]);
    cpu.x = 0x20;
    mem.data[0x10] = 0x80;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x80);
    assert_eq!(cpu.cycles, 4);
    assert!(cpu.n);
}

#[test]
fn lda_zero_page_x_wraps_and_sets_zero_flag() {
    let (mut cpu, mut mem) = init();
    load_instruction_at(&mut cpu, &mut mem, 0x0200, &[0xB5, 0xFF]);
    cpu.x = 1;
    mem.data[0] = 0x00;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0);
    assert!(cpu.z);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn lda_absolute_loads_value_from_little_endian_address() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xAD, 0x34, 0x12]);
    mem.data[0x1234] = 0x55;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x55);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.cycles, 4);
    assert!(!cpu.z);
    assert!(!cpu.n);
}

#[test]
fn lda_absolute_x_without_page_cross_uses_base_cycles() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xBD, 0x00, 0x20]);
    cpu.x = 0x10;
    mem.data[0x2010] = 0x55;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x55);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn lda_absolute_x_page_cross_adds_one_cycle() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xBD, 0xFF, 0x20]);
    cpu.x = 1;
    mem.data[0x2100] = 0x80;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x80);
    assert_eq!(cpu.cycles, 5);
    assert!(cpu.n);
}

#[test]
fn lda_absolute_y_without_page_cross_uses_base_cycles() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xB9, 0x00, 0x20]);
    cpu.y = 0x10;
    mem.data[0x2010] = 0x55;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x55);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn lda_absolute_y_page_cross_adds_one_cycle() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xB9, 0xFF, 0x20]);
    cpu.y = 1;
    mem.data[0x2100] = 0x00;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0);
    assert_eq!(cpu.cycles, 5);
    assert!(cpu.z);
}

#[test]
fn lda_absolute_x_wraps_the_sixteen_bit_address() {
    let (mut cpu, mut mem) = init();
    cpu.pc = 0x0100;
    mem.data[0x0100..0x0103].copy_from_slice(&[0xBD, 0xFF, 0xFF]);
    cpu.x = 1;
    mem.data[0] = 0x42;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.pc, 0x0103);
    assert_eq!(cpu.cycles, 5);
}

#[test]
fn lda_indexed_indirect_adds_x_and_wraps_the_zero_page_pointer() {
    let (mut cpu, mut mem) = init();
    load_instruction_at(&mut cpu, &mut mem, 0x0200, &[0xA1, 0xF8]);
    cpu.x = 7;
    mem.data[0xFF] = 0x34;
    mem.data[0] = 0x12;
    mem.data[0x1234] = 0xA5;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0xA5);
    assert_eq!(cpu.pc, 0x0202);
    assert_eq!(cpu.cycles, 6);
    assert!(cpu.n);
}

#[test]
fn lda_indexed_indirect_wraps_the_indexed_zero_page_operand() {
    let (mut cpu, mut mem) = init();
    load_instruction_at(&mut cpu, &mut mem, 0x0200, &[0xA1, 0xFF]);
    cpu.x = 1;
    mem.data[0] = 0x78;
    mem.data[1] = 0x56;
    mem.data[0x5678] = 0x00;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0);
    assert!(cpu.z);
    assert_eq!(cpu.cycles, 6);
}

#[test]
fn lda_indirect_indexed_adds_y_to_the_pointed_address() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xB1, 0x10]);
    cpu.y = 1;
    mem.data[0x10] = 0x34;
    mem.data[0x11] = 0x12;
    mem.data[0x1235] = 0x55;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x55);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 5);
}

#[test]
fn lda_indirect_indexed_adds_a_cycle_when_y_crosses_a_page() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xB1, 0x10]);
    cpu.y = 1;
    mem.data[0x10] = 0xFF;
    mem.data[0x11] = 0x12;
    mem.data[0x1300] = 0x80;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x80);
    assert_eq!(cpu.cycles, 6);
    assert!(cpu.n);
}

#[test]
fn lda_indirect_indexed_wraps_the_zero_page_pointer_high_byte() {
    let (mut cpu, mut mem) = init();
    load_instruction_at(&mut cpu, &mut mem, 0x0200, &[0xB1, 0xFF]);
    cpu.y = 1;
    mem.data[0xFF] = 0xFF;
    mem.data[0] = 0x12;
    mem.data[0x1300] = 0x42;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.cycles, 6);
}

#[test]
fn ldx_immediate_loads_value_and_updates_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xA2, 0x80]);

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.x, 0x80);
    assert!(cpu.n);
    assert!(!cpu.z);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn ldx_zero_page_loads_value_and_updates_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xA6, 0x42]);
    mem.data[0x42] = 0;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.x, 0);
    assert!(cpu.z);
    assert!(!cpu.n);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn ldx_zero_page_y_wraps_at_zero_page_boundary() {
    let (mut cpu, mut mem) = init();
    load_instruction_at(&mut cpu, &mut mem, 0x0200, &[0xB6, 0xFF]);
    cpu.y = 1;
    mem.data[0] = 0x81;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.x, 0x81);
    assert!(cpu.n);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn ldx_absolute_loads_value_from_little_endian_address() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xAE, 0x34, 0x12]);
    mem.data[0x1234] = 0x42;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.x, 0x42);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn ldx_absolute_y_adds_a_cycle_when_index_crosses_a_page() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xBE, 0xFF, 0x20]);
    cpu.y = 1;
    mem.data[0x2100] = 0x00;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.x, 0);
    assert!(cpu.z);
    assert_eq!(cpu.cycles, 5);
}

#[test]
fn ldy_immediate_loads_value_and_updates_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xA0, 0x00]);

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.y, 0);
    assert!(cpu.z);
    assert!(!cpu.n);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn ldy_zero_page_loads_value_and_updates_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xA4, 0x42]);
    mem.data[0x42] = 0x80;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.y, 0x80);
    assert!(cpu.n);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn ldy_zero_page_x_wraps_at_zero_page_boundary() {
    let (mut cpu, mut mem) = init();
    load_instruction_at(&mut cpu, &mut mem, 0x0200, &[0xB4, 0xFF]);
    cpu.x = 1;
    mem.data[0] = 0x7F;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.y, 0x7F);
    assert!(!cpu.n);
    assert!(!cpu.z);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn ldy_absolute_loads_value_from_little_endian_address() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xAC, 0x34, 0x12]);
    mem.data[0x1234] = 0xFF;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.y, 0xFF);
    assert!(cpu.n);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn sta_zero_page_stores_accumulator_without_changing_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0x85, 0x42]);
    cpu.a = 0x80;
    cpu.c = true;
    cpu.z = true;
    cpu.v = true;
    cpu.n = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(mem.data[0x42], 0x80);
    assert!(cpu.c);
    assert!(cpu.z);
    assert!(cpu.v);
    assert!(cpu.n);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn sta_absolute_stores_accumulator_at_little_endian_address() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0x8D, 0x34, 0x12]);
    cpu.a = 0x5A;

    execute(&mut cpu, &mut mem);

    assert_eq!(mem.data[0x1234], 0x5A);
    assert_eq!(cpu.pc, 3);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn stx_zero_page_stores_x_without_changing_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0x86, 0x42]);
    cpu.x = 0xA5;
    cpu.z = true;
    cpu.n = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(mem.data[0x42], 0xA5);
    assert!(cpu.z);
    assert!(cpu.n);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn adc_immediate_adds_without_carry_and_clears_stale_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0x69, 0x01]);
    cpu.a = 0x10;
    cpu.c = false;
    cpu.z = true;
    cpu.n = true;
    cpu.v = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x11);
    assert!(!cpu.c);
    assert!(!cpu.z);
    assert!(!cpu.n);
    assert!(!cpu.v);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn adc_immediate_carry_in_wraps_result_to_zero() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0x69, 0x00]);
    cpu.a = 0xFF;
    cpu.c = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0);
    assert!(cpu.c);
    assert!(cpu.z);
    assert!(!cpu.n);
    assert!(!cpu.v);
}

#[test]
fn adc_immediate_detects_signed_overflow() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0x69, 0x01]);
    cpu.a = 0x7F;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x80);
    assert!(!cpu.c);
    assert!(cpu.v);
    assert!(cpu.n);
    assert!(!cpu.z);
}

#[test]
fn inx_wraps_from_ff_to_zero_and_updates_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xE8]);
    cpu.x = 0xFF;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.x, 0);
    assert!(cpu.z);
    assert!(!cpu.n);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn dex_wraps_from_zero_to_ff_and_updates_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xCA]);

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.x, 0xFF);
    assert!(!cpu.z);
    assert!(cpu.n);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn cmp_immediate_equal_sets_zero_and_carry_without_changing_accumulator() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xC9, 0x42]);
    cpu.a = 0x42;
    cpu.n = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x42);
    assert!(cpu.c);
    assert!(cpu.z);
    assert!(!cpu.n);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn cmp_immediate_greater_sets_carry_and_negative_result_correctly() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xC9, 0x40]);
    cpu.a = 0x80;

    execute(&mut cpu, &mut mem);

    assert!(cpu.c);
    assert!(!cpu.z);
    assert!(!cpu.n);
}

#[test]
fn cmp_immediate_less_clears_carry_and_sets_negative_result() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xC9, 0x01]);
    cpu.a = 0;

    execute(&mut cpu, &mut mem);

    assert!(!cpu.c);
    assert!(!cpu.z);
    assert!(cpu.n);
}

#[test]
fn bne_does_not_branch_when_zero_flag_is_set() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xD0, 0x20]);
    cpu.z = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn bne_branches_forward_when_zero_flag_is_clear() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xD0, 0x05]);
    cpu.z = false;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 7);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn beq_branches_backward_with_a_signed_relative_offset() {
    let (mut cpu, mut mem) = init();
    cpu.pc = 0x0200;
    mem.data[0x0200..0x0202].copy_from_slice(&[0xF0, 0xFC]);
    cpu.z = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 0x01FE);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn bne_adds_a_cycle_when_taken_branch_crosses_page() {
    let (mut cpu, mut mem) = init();
    cpu.pc = 0x00FD;
    mem.data[0x00FD..0x00FF].copy_from_slice(&[0xD0, 0x02]);
    cpu.z = false;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 0x0101);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn beq_does_not_branch_when_zero_flag_is_clear() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xF0, 0xFE]);
    cpu.z = false;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn jmp_absolute_loads_little_endian_target_address() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0x4C, 0x34, 0x12]);

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn jmp_absolute_fetches_operands_across_program_counter_wrap() {
    let (mut cpu, mut mem) = init();
    cpu.pc = 0xFFFE;
    mem.data[0xFFFE] = 0x4C;
    mem.data[0xFFFF] = 0x34;
    mem.data[0] = 0x12;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn nop_advances_pc_and_cycles_without_changing_registers_or_flags() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0xEA]);
    cpu.a = 0x11;
    cpu.x = 0x22;
    cpu.y = 0x33;
    cpu.c = true;
    cpu.z = true;
    cpu.v = true;
    cpu.n = true;

    execute(&mut cpu, &mut mem);

    assert_eq!((cpu.a, cpu.x, cpu.y), (0x11, 0x22, 0x33));
    assert!(cpu.c && cpu.z && cpu.v && cpu.n);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn clc_clears_only_carry_and_adds_two_cycles() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0x18]);
    cpu.c = true;
    cpu.z = true;
    cpu.v = true;
    cpu.n = true;

    execute(&mut cpu, &mut mem);

    assert!(!cpu.c);
    assert!(cpu.z && cpu.v && cpu.n);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn brk_pushes_return_state_and_jumps_through_irq_vector() {
    let (mut cpu, mut mem) = init();
    load_instruction_at(&mut cpu, &mut mem, 0x0200, &[0x00, 0xEA]);
    mem.data[0xFFFE] = 0x34;
    mem.data[0xFFFF] = 0x12;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.sp, 0xFC);
    assert_eq!(mem.data[0x01FF], 0x02);
    assert_eq!(mem.data[0x01FE], 0x02);
    assert_eq!(mem.data[0x01FD] & 0x30, 0x30);
    assert!(cpu.i);
    assert_eq!(cpu.cycles, 7);
}

#[test]
fn execute_does_not_change_halted_cpu_state() {
    let (mut cpu, mut mem) = init();
    cpu.halted = true;
    cpu.pc = 0x1234;
    cpu.cycles = 99;

    execute(&mut cpu, &mut mem);

    assert!(cpu.halted);
    assert_eq!(cpu.pc, 0x1234);
    assert_eq!(cpu.cycles, 99);
}

#[test]
fn execute_wraps_program_counter_when_fetching_opcode_at_ffff() {
    let (mut cpu, mut mem) = init();
    cpu.pc = 0xFFFF;
    mem.data[0xFFFF] = 0xA9;
    mem.data[0] = 0x42;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn unsupported_opcode_reports_its_address() {
    let (mut cpu, mut mem) = init();
    load_instruction(&mut mem, &[0x02]);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        execute(&mut cpu, &mut mem);
    }));

    assert!(result.is_err());
}

#[test]
fn adc_decimal_mode_adds_bcd_digits_and_produces_decimal_carry() {
    let (mut cpu, mut mem) = init();
    mem.data[0..2].copy_from_slice(&[0x69, 0x55]);
    cpu.a = 0x45;
    cpu.d = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x00);
    assert!(cpu.c);
    assert!(cpu.d);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn sbc_decimal_mode_subtracts_bcd_digits_without_borrow() {
    let (mut cpu, mut mem) = init();
    mem.data[0..2].copy_from_slice(&[0xE9, 0x01]);
    cpu.a = 0x50;
    cpu.c = true;
    cpu.d = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x49);
    assert!(cpu.c);
    assert!(cpu.d);
}

#[test]
fn sbc_binary_mode_borrow_wraps_result_to_ff() {
    let (mut cpu, mut mem) = init();
    mem.data[0..2].copy_from_slice(&[0xE9, 0x01]);
    cpu.a = 0;
    cpu.c = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0xFF);
    assert!(!cpu.c);
    assert!(!cpu.v);
    assert_nz(&cpu, 0xFF);
}

#[test]
fn sbc_binary_mode_detects_signed_overflow() {
    let (mut cpu, mut mem) = init();
    mem.data[0..2].copy_from_slice(&[0xE9, 0x01]);
    cpu.a = 0x80;
    cpu.c = true;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.a, 0x7F);
    assert!(cpu.c);
    assert!(cpu.v);
    assert_nz(&cpu, 0x7F);
}

#[test]
fn pha_wraps_stack_pointer_from_zero_to_ff() {
    let (mut cpu, mut mem) = init();
    mem.data[0] = 0x48;
    cpu.a = 0xA5;
    cpu.sp = 0;

    execute(&mut cpu, &mut mem);

    assert_eq!(mem.data[0x0100], 0xA5);
    assert_eq!(cpu.sp, 0xFF);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn bne_relative_offset_wraps_program_counter_at_ffff() {
    let (mut cpu, mut mem) = init();
    cpu.pc = 0xFFFD;
    mem.data[0xFFFD..0xFFFF].copy_from_slice(&[0xD0, 0x01]);
    cpu.z = false;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 0);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn bne_takes_the_most_negative_relative_offset() {
    let (mut cpu, mut mem) = init();
    mem.data[0..2].copy_from_slice(&[0xD0, 0x80]);
    cpu.z = false;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 0xFF82);
    assert_eq!(cpu.cycles, 4);
}

#[test]
fn bne_takes_the_most_positive_relative_offset() {
    let (mut cpu, mut mem) = init();
    mem.data[0..2].copy_from_slice(&[0xD0, 0x7F]);
    cpu.z = false;

    execute(&mut cpu, &mut mem);

    assert_eq!(cpu.pc, 0x0081);
    assert_eq!(cpu.cycles, 3);
}

#[test]
fn assembled_program_executes_load_store_and_reload() {
    let source = "LDA #$2A\nSTA $0300\nLDA $0300\n";
    let (start, program) = crate::assembler::assemble(source).unwrap();
    let (mut cpu, mut mem) = init();
    let begin = start as usize;
    mem.data[begin..begin + program.len()].copy_from_slice(&program);
    cpu.pc = start as u32;

    execute(&mut cpu, &mut mem);
    execute(&mut cpu, &mut mem);
    execute(&mut cpu, &mut mem);

    assert_eq!(mem.data[0x0300], 0x2A);
    assert_eq!(cpu.a, 0x2A);
    assert_eq!(cpu.cycles, 10);
    assert!(!cpu.halted);
}
