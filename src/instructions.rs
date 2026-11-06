use crate::define::{Cpu, Mem};
use crate::execute::fetch;

fn set_nz(cpu: &mut Cpu, value: u8) {
    cpu.z = value == 0;
    cpu.n = value & 0x80 != 0;
}

fn fetch_word(cpu: &mut Cpu, mem: &Mem) -> u16 {
    let low = fetch(cpu, mem) as u16;
    let high = fetch(cpu, mem) as u16;
    low | high << 8
}

fn store(mem: &mut Mem, addr: usize, value: u8) {
    if mem
        .read_only
        .is_some_and(|(start, end)| (start..end).contains(&addr))
    {
        return;
    }
    if addr == 0xF001 {
        // arbitrary, system specific
        print!("{}", value as char);
    } else {
        mem.data[addr] = value;
    }
}

#[derive(Clone, Copy)]
enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndexedIndirect,
    IndirectIndexed,
    Indirect,
    Relative,
}

#[derive(Clone, Copy)]
enum BranchCondition {
    CarryClear,
    CarrySet,
    Equal,
    NotEqual,
    Minus,
    Plus,
    OverflowClear,
    OverflowSet,
}

#[derive(Clone, Copy)]
enum Shift {
    Asl,
    Lsr,
    Rol,
    Ror,
}

#[derive(Clone, Copy)]
enum Operation {
    Adc,
    And,
    Asl,
    Bit,
    Branch(BranchCondition),
    Brk,
    Clc,
    Cld,
    Cli,
    Clv,
    CompareA,
    CompareX,
    CompareY,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rol,
    Ror,
    Rti,
    Rts,
    Sbc,
    Sec,
    Sed,
    Sei,
    Sta,
    Stx,
    Sty,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
}

#[derive(Clone, Copy)]
struct DecodedInstruction {
    operation: Operation,
    mode: AddressingMode,
}

#[derive(Clone, Copy)]
enum Operand {
    Value(u8),
    Address(u16),
    Accumulator,
}

fn decode_opcode(opcode: u8) -> Option<DecodedInstruction> {
    use AddressingMode::*;
    use Operation::*;

    let group_operation = match opcode & 0xE0 {
        0x00 => Some(Ora),
        0x20 => Some(And),
        0x40 => Some(Eor),
        0x60 => Some(Adc),
        0x80 => Some(Sta),
        0xA0 => Some(Lda),
        0xC0 => Some(CompareA),
        0xE0 => Some(Sbc),
        _ => None,
    };
    if let Some(operation) = group_operation {
        let mode = match opcode & 0x1F {
            0x01 => Some(IndexedIndirect),
            0x05 => Some(ZeroPage),
            0x09 if !matches!(operation, Sta) => Some(Immediate),
            0x0D => Some(Absolute),
            0x11 => Some(IndirectIndexed),
            0x15 => Some(ZeroPageX),
            0x19 => Some(AbsoluteY),
            0x1D => Some(AbsoluteX),
            _ => None,
        };
        if let Some(mode) = mode {
            return Some(DecodedInstruction { operation, mode });
        }
    }

    let (operation, mode) = match opcode {
        0x00 => (Brk, Implied),
        0x06 => (Asl, ZeroPage),
        0x0A => (Asl, Accumulator),
        0x0E => (Asl, Absolute),
        0x10 => (Branch(BranchCondition::Plus), Relative),
        0x16 => (Asl, ZeroPageX),
        0x18 => (Clc, Implied),
        0x1E => (Asl, AbsoluteX),
        0x20 => (Jsr, Absolute),
        0x24 => (Bit, ZeroPage),
        0x26 => (Rol, ZeroPage),
        0x28 => (Plp, Implied),
        0x2A => (Rol, Accumulator),
        0x2C => (Bit, Absolute),
        0x2E => (Rol, Absolute),
        0x30 => (Branch(BranchCondition::Minus), Relative),
        0x36 => (Rol, ZeroPageX),
        0x38 => (Sec, Implied),
        0x3E => (Rol, AbsoluteX),
        0x40 => (Rti, Implied),
        0x46 => (Lsr, ZeroPage),
        0x48 => (Pha, Implied),
        0x4A => (Lsr, Accumulator),
        0x4C => (Jmp, Absolute),
        0x4E => (Lsr, Absolute),
        0x50 => (Branch(BranchCondition::OverflowClear), Relative),
        0x56 => (Lsr, ZeroPageX),
        0x58 => (Cli, Implied),
        0x5E => (Lsr, AbsoluteX),
        0x60 => (Rts, Implied),
        0x66 => (Ror, ZeroPage),
        0x68 => (Pla, Implied),
        0x6A => (Ror, Accumulator),
        0x6C => (Jmp, Indirect),
        0x6E => (Ror, Absolute),
        0x70 => (Branch(BranchCondition::OverflowSet), Relative),
        0x76 => (Ror, ZeroPageX),
        0x78 => (Sei, Implied),
        0x7E => (Ror, AbsoluteX),
        0x08 => (Php, Implied),
        0x84 => (Sty, ZeroPage),
        0x86 => (Stx, ZeroPage),
        0x88 => (Dey, Implied),
        0x8A => (Txa, Implied),
        0x8C => (Sty, Absolute),
        0x8E => (Stx, Absolute),
        0x90 => (Branch(BranchCondition::CarryClear), Relative),
        0x94 => (Sty, ZeroPageX),
        0x96 => (Stx, ZeroPageY),
        0x98 => (Tya, Implied),
        0x9A => (Txs, Implied),
        0xA0 => (Ldy, Immediate),
        0xA2 => (Ldx, Immediate),
        0xA4 => (Ldy, ZeroPage),
        0xA6 => (Ldx, ZeroPage),
        0xA8 => (Tay, Implied),
        0xAA => (Tax, Implied),
        0xAC => (Ldy, Absolute),
        0xAE => (Ldx, Absolute),
        0xB0 => (Branch(BranchCondition::CarrySet), Relative),
        0xB4 => (Ldy, ZeroPageX),
        0xB6 => (Ldx, ZeroPageY),
        0xB8 => (Clv, Implied),
        0xBA => (Tsx, Implied),
        0xBC => (Ldy, AbsoluteX),
        0xBE => (Ldx, AbsoluteY),
        0xC0 => (CompareY, Immediate),
        0xC4 => (CompareY, ZeroPage),
        0xC6 => (Dec, ZeroPage),
        0xC8 => (Iny, Implied),
        0xCA => (Dex, Implied),
        0xCC => (CompareY, Absolute),
        0xCE => (Dec, Absolute),
        0xD0 => (Branch(BranchCondition::NotEqual), Relative),
        0xD6 => (Dec, ZeroPageX),
        0xD8 => (Cld, Implied),
        0xDE => (Dec, AbsoluteX),
        0xE0 => (CompareX, Immediate),
        0xE4 => (CompareX, ZeroPage),
        0xE6 => (Inc, ZeroPage),
        0xE8 => (Inx, Implied),
        0xEA => (Nop, Implied),
        0xEC => (CompareX, Absolute),
        0xEE => (Inc, Absolute),
        0xF0 => (Branch(BranchCondition::Equal), Relative),
        0xF6 => (Inc, ZeroPageX),
        0xF8 => (Sed, Implied),
        0xFE => (Inc, AbsoluteX),
        _ => return None,
    };
    Some(DecodedInstruction { operation, mode })
}

pub fn execute_opcode(cpu: &mut Cpu, mem: &mut Mem, opcode: u8) {
    let instruction = decode_opcode(opcode).unwrap_or_else(|| {
        panic!(
            "Unsupported opcode ${opcode:02X} at ${:04X}",
            cpu.pc.wrapping_sub(1) & 0xFFFF
        )
    });
    let operation = instruction.operation;

    match operation {
        Operation::Brk => {
            fetch(cpu, mem);
            let return_address = cpu.pc as u16;
            push(cpu, mem, (return_address >> 8) as u8);
            push(cpu, mem, return_address as u8);
            push(cpu, mem, status(cpu, true));
            cpu.i = true;
            cpu.pc = fetch_word_at(mem, 0xFFFE) as u32;
            cpu.cycles += 7;
            return;
        }
        Operation::Branch(condition) => {
            let offset = fetch(cpu, mem) as i8;
            cpu.cycles += 2;
            if branch_condition(cpu, condition) {
                let old_pc = cpu.pc as u16;
                let target = old_pc.wrapping_add(offset as i16 as u16);
                cpu.pc = target as u32;
                cpu.cycles += 1 + u64::from((old_pc & 0xFF00) != (target & 0xFF00));
            }
            return;
        }
        Operation::Jmp if matches!(instruction.mode, AddressingMode::Absolute) => {
            cpu.pc = fetch_word(cpu, mem) as u32;
            cpu.cycles += 3;
            return;
        }
        Operation::Jmp => {
            let pointer = fetch_word(cpu, mem);
            let high_address = (pointer & 0xFF00) | (pointer.wrapping_add(1) & 0x00FF);
            cpu.pc = (mem.data[pointer as usize] as u16
                | (mem.data[high_address as usize] as u16) << 8) as u32;
            cpu.cycles += 5;
            return;
        }
        Operation::Jsr => {
            let target = fetch_word(cpu, mem);
            let return_address = (cpu.pc as u16).wrapping_sub(1);
            push(cpu, mem, (return_address >> 8) as u8);
            push(cpu, mem, return_address as u8);
            cpu.pc = target as u32;
            cpu.cycles += 6;
            return;
        }
        Operation::Rts => {
            let low = pop(cpu, mem) as u16;
            let high = pop(cpu, mem) as u16;
            cpu.pc = (low | high << 8).wrapping_add(1) as u32;
            cpu.cycles += 6;
            return;
        }
        Operation::Rti => {
            let value = pop(cpu, mem);
            restore_status(cpu, value);
            let low = pop(cpu, mem) as u16;
            let high = pop(cpu, mem) as u16;
            cpu.pc = (low | high << 8) as u32;
            cpu.cycles += 6;
            return;
        }
        Operation::Pha => {
            push(cpu, mem, cpu.a);
            cpu.cycles += 3;
            return;
        }
        Operation::Php => {
            push(cpu, mem, status(cpu, true));
            cpu.cycles += 3;
            return;
        }
        Operation::Pla => {
            cpu.a = pop(cpu, mem);
            set_nz(cpu, cpu.a);
            cpu.cycles += 4;
            return;
        }
        Operation::Plp => {
            let value = pop(cpu, mem);
            restore_status(cpu, value);
            cpu.cycles += 4;
            return;
        }
        _ => {}
    }

    let (operand, page_crossed) = resolve_operand(cpu, mem, instruction.mode);
    execute_regular(cpu, mem, operation, operand);
    cpu.cycles += base_cycles(operation, instruction.mode)
        + u64::from(page_crossed && adds_page_cross_cycle(operation, instruction.mode));
}

fn resolve_operand(cpu: &mut Cpu, mem: &Mem, mode: AddressingMode) -> (Operand, bool) {
    use AddressingMode::*;
    match mode {
        Implied => (Operand::Value(0), false),
        Accumulator => (Operand::Accumulator, false),
        Immediate => (Operand::Value(fetch(cpu, mem)), false),
        Relative => (Operand::Value(fetch(cpu, mem)), false),
        ZeroPage => (Operand::Address(fetch(cpu, mem) as u16), false),
        ZeroPageX => {
            let base = fetch(cpu, mem);
            (Operand::Address(base.wrapping_add(cpu.x) as u16), false)
        }
        ZeroPageY => {
            let base = fetch(cpu, mem);
            (Operand::Address(base.wrapping_add(cpu.y) as u16), false)
        }
        Absolute => (Operand::Address(fetch_word(cpu, mem)), false),
        AbsoluteX | AbsoluteY => {
            let base = fetch_word(cpu, mem);
            let index = if matches!(mode, AbsoluteX) {
                cpu.x
            } else {
                cpu.y
            };
            let address = base.wrapping_add(index as u16);
            (
                Operand::Address(address),
                (base & 0xFF00) != (address & 0xFF00),
            )
        }
        IndexedIndirect => {
            let pointer = fetch(cpu, mem).wrapping_add(cpu.x);
            let low = mem.data[pointer as usize] as u16;
            let high = mem.data[pointer.wrapping_add(1) as usize] as u16;
            (Operand::Address(low | high << 8), false)
        }
        IndirectIndexed => {
            let pointer = fetch(cpu, mem);
            let low = mem.data[pointer as usize] as u16;
            let high = mem.data[pointer.wrapping_add(1) as usize] as u16;
            let base = low | high << 8;
            let address = base.wrapping_add(cpu.y as u16);
            (
                Operand::Address(address),
                (base & 0xFF00) != (address & 0xFF00),
            )
        }
        Indirect => (Operand::Value(fetch_word(cpu, mem) as u8), false),
    }
}

fn execute_regular(cpu: &mut Cpu, mem: &mut Mem, operation: Operation, operand: Operand) {
    let value = match operand {
        Operand::Value(value) => value,
        Operand::Address(address) => mem.data[address as usize],
        Operand::Accumulator => cpu.a,
    };

    match operation {
        Operation::Adc => adc(cpu, value),
        Operation::And => {
            cpu.a &= value;
            set_nz(cpu, cpu.a);
        }
        Operation::Asl | Operation::Lsr | Operation::Rol | Operation::Ror => {
            let shift = match operation {
                Operation::Asl => Shift::Asl,
                Operation::Lsr => Shift::Lsr,
                Operation::Rol => Shift::Rol,
                Operation::Ror => Shift::Ror,
                _ => unreachable!(),
            };
            let result = shift_value(cpu, value, shift);
            write_result(cpu, mem, operand, result);
        }
        Operation::Bit => {
            cpu.z = cpu.a & value == 0;
            cpu.n = value & 0x80 != 0;
            cpu.v = value & 0x40 != 0;
        }
        Operation::Clc => cpu.c = false,
        Operation::Cld => cpu.d = false,
        Operation::Cli => cpu.i = false,
        Operation::Clv => cpu.v = false,
        Operation::CompareA => compare(cpu, cpu.a, value),
        Operation::CompareX => compare(cpu, cpu.x, value),
        Operation::CompareY => compare(cpu, cpu.y, value),
        Operation::Dec | Operation::Inc => {
            let result = if matches!(operation, Operation::Dec) {
                value.wrapping_sub(1)
            } else {
                value.wrapping_add(1)
            };
            set_nz(cpu, result);
            write_result(cpu, mem, operand, result);
        }
        Operation::Dex => {
            cpu.x = cpu.x.wrapping_sub(1);
            set_nz(cpu, cpu.x);
        }
        Operation::Dey => {
            cpu.y = cpu.y.wrapping_sub(1);
            set_nz(cpu, cpu.y);
        }
        Operation::Eor => {
            cpu.a ^= value;
            set_nz(cpu, cpu.a);
        }
        Operation::Inx => {
            cpu.x = cpu.x.wrapping_add(1);
            set_nz(cpu, cpu.x);
        }
        Operation::Iny => {
            cpu.y = cpu.y.wrapping_add(1);
            set_nz(cpu, cpu.y);
        }
        Operation::Lda => {
            cpu.a = value;
            set_nz(cpu, value);
        }
        Operation::Ldx => {
            cpu.x = value;
            set_nz(cpu, value);
        }
        Operation::Ldy => {
            cpu.y = value;
            set_nz(cpu, value);
        }
        Operation::Nop => {}
        Operation::Ora => {
            cpu.a |= value;
            set_nz(cpu, cpu.a);
        }
        Operation::Sbc => sbc(cpu, value),
        Operation::Sec => cpu.c = true,
        Operation::Sed => cpu.d = true,
        Operation::Sei => cpu.i = true,
        Operation::Sta => write_result(cpu, mem, operand, cpu.a),
        Operation::Stx => write_result(cpu, mem, operand, cpu.x),
        Operation::Sty => write_result(cpu, mem, operand, cpu.y),
        Operation::Tax => {
            cpu.x = cpu.a;
            set_nz(cpu, cpu.x);
        }
        Operation::Tay => {
            cpu.y = cpu.a;
            set_nz(cpu, cpu.y);
        }
        Operation::Tsx => {
            cpu.x = cpu.sp;
            set_nz(cpu, cpu.x);
        }
        Operation::Txa => {
            cpu.a = cpu.x;
            set_nz(cpu, cpu.a);
        }
        Operation::Txs => cpu.sp = cpu.x,
        Operation::Tya => {
            cpu.a = cpu.y;
            set_nz(cpu, cpu.a);
        }
        Operation::Branch(_)
        | Operation::Brk
        | Operation::Jmp
        | Operation::Jsr
        | Operation::Pha
        | Operation::Php
        | Operation::Pla
        | Operation::Plp
        | Operation::Rti
        | Operation::Rts => unreachable!(),
    }
}

fn write_result(cpu: &mut Cpu, mem: &mut Mem, operand: Operand, value: u8) {
    match operand {
        Operand::Accumulator => cpu.a = value,
        Operand::Address(address) => store(mem, address as usize, value),
        _ => panic!("invalid destination for opcode"),
    }
}

fn shift_value(cpu: &mut Cpu, value: u8, shift: Shift) -> u8 {
    let result = match shift {
        Shift::Asl => {
            cpu.c = value & 0x80 != 0;
            value.wrapping_shl(1)
        }
        Shift::Lsr => {
            cpu.c = value & 0x01 != 0;
            value >> 1
        }
        Shift::Rol => {
            let carry = u8::from(cpu.c);
            cpu.c = value & 0x80 != 0;
            value.wrapping_shl(1) | carry
        }
        Shift::Ror => {
            let carry = u8::from(cpu.c) << 7;
            cpu.c = value & 0x01 != 0;
            (value >> 1) | carry
        }
    };
    set_nz(cpu, result);
    result
}

fn compare(cpu: &mut Cpu, register: u8, value: u8) {
    let result = register.wrapping_sub(value);
    cpu.c = register >= value;
    set_nz(cpu, result);
}

fn adc(cpu: &mut Cpu, value: u8) {
    let accumulator = cpu.a;
    let carry = u16::from(cpu.c);
    let binary_sum = accumulator as u16 + value as u16 + carry;
    let binary_result = binary_sum as u8;
    cpu.v = (!(accumulator ^ value) & (accumulator ^ binary_result) & 0x80) != 0;

    if cpu.d {
        let mut decimal_sum = binary_sum;
        if (accumulator & 0x0F) as u16 + (value & 0x0F) as u16 + carry > 9 {
            decimal_sum += 0x06;
        }
        if decimal_sum > 0x99 {
            decimal_sum += 0x60;
        }
        cpu.c = decimal_sum > 0xFF;
        cpu.a = decimal_sum as u8;
        set_nz(cpu, binary_result);
    } else {
        cpu.c = binary_sum > 0xFF;
        cpu.a = binary_result;
        set_nz(cpu, binary_result);
    }
}

fn sbc(cpu: &mut Cpu, value: u8) {
    let accumulator = cpu.a;
    let borrow = i16::from(!cpu.c);
    let binary_difference = accumulator as i16 - value as i16 - borrow;
    let binary_result = binary_difference as u8;
    cpu.v = ((accumulator ^ binary_result) & (accumulator ^ value) & 0x80) != 0;
    cpu.c = binary_difference >= 0;

    if cpu.d {
        let mut low = (accumulator & 0x0F) as i16 - (value & 0x0F) as i16 - borrow;
        let mut high = (accumulator >> 4) as i16 - (value >> 4) as i16;
        if low < 0 {
            low -= 6;
            high -= 1;
        }
        if high < 0 {
            high -= 6;
        }
        cpu.a = ((high as u8) << 4) | (low as u8 & 0x0F);
        set_nz(cpu, binary_result);
    } else {
        cpu.a = binary_result;
        set_nz(cpu, binary_result);
    }
}

fn branch_condition(cpu: &Cpu, condition: BranchCondition) -> bool {
    match condition {
        BranchCondition::CarryClear => !cpu.c,
        BranchCondition::CarrySet => cpu.c,
        BranchCondition::Equal => cpu.z,
        BranchCondition::NotEqual => !cpu.z,
        BranchCondition::Minus => cpu.n,
        BranchCondition::Plus => !cpu.n,
        BranchCondition::OverflowClear => !cpu.v,
        BranchCondition::OverflowSet => cpu.v,
    }
}

fn base_cycles(operation: Operation, mode: AddressingMode) -> u64 {
    use AddressingMode::*;
    match mode {
        Implied | Accumulator => 2,
        Immediate => 2,
        ZeroPage => match operation {
            Operation::Asl
            | Operation::Dec
            | Operation::Inc
            | Operation::Lsr
            | Operation::Rol
            | Operation::Ror => 5,
            _ => 3,
        },
        ZeroPageX | ZeroPageY => match operation {
            Operation::Asl
            | Operation::Dec
            | Operation::Inc
            | Operation::Lsr
            | Operation::Rol
            | Operation::Ror => 6,
            _ => 4,
        },
        Absolute => match operation {
            Operation::Asl
            | Operation::Dec
            | Operation::Inc
            | Operation::Lsr
            | Operation::Rol
            | Operation::Ror => 6,
            _ => 4,
        },
        AbsoluteX | AbsoluteY => match operation {
            Operation::Sta | Operation::Stx | Operation::Sty => 5,
            Operation::Asl
            | Operation::Dec
            | Operation::Inc
            | Operation::Lsr
            | Operation::Rol
            | Operation::Ror => 7,
            _ => 4,
        },
        IndexedIndirect => 6,
        IndirectIndexed => {
            if matches!(operation, Operation::Sta | Operation::Stx | Operation::Sty) {
                6
            } else {
                5
            }
        }
        Indirect => 5,
        Relative => 0,
    }
}

fn adds_page_cross_cycle(operation: Operation, mode: AddressingMode) -> bool {
    matches!(
        mode,
        AddressingMode::AbsoluteX | AddressingMode::AbsoluteY | AddressingMode::IndirectIndexed
    ) && !matches!(
        operation,
        Operation::Sta
            | Operation::Stx
            | Operation::Sty
            | Operation::Asl
            | Operation::Dec
            | Operation::Inc
            | Operation::Lsr
            | Operation::Rol
            | Operation::Ror
    )
}

fn push(cpu: &mut Cpu, mem: &mut Mem, value: u8) {
    mem.data[0x0100 | cpu.sp as usize] = value;
    cpu.sp = cpu.sp.wrapping_sub(1);
}

fn pop(cpu: &mut Cpu, mem: &Mem) -> u8 {
    cpu.sp = cpu.sp.wrapping_add(1);
    mem.data[0x0100 | cpu.sp as usize]
}

fn status(cpu: &Cpu, break_flag: bool) -> u8 {
    u8::from(cpu.c)
        | (u8::from(cpu.z) << 1)
        | (u8::from(cpu.i) << 2)
        | (u8::from(cpu.d) << 3)
        | (u8::from(break_flag) << 4)
        | 0x20
        | (u8::from(cpu.v) << 6)
        | (u8::from(cpu.n) << 7)
}

fn restore_status(cpu: &mut Cpu, value: u8) {
    cpu.c = value & 0x01 != 0;
    cpu.z = value & 0x02 != 0;
    cpu.i = value & 0x04 != 0;
    cpu.d = value & 0x08 != 0;
    cpu.b = false;
    cpu.v = value & 0x40 != 0;
    cpu.n = value & 0x80 != 0;
}

fn fetch_word_at(mem: &Mem, address: u16) -> u16 {
    mem.data[address as usize] as u16 | (mem.data[address.wrapping_add(1) as usize] as u16) << 8
}
