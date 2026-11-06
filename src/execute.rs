use crate::define::{Cpu, Mem};
use crate::instructions::execute_opcode;
use std::collections::VecDeque;
use std::io::{self, BufRead, Write};

pub const SIM_INPUT_HOOK: u16 = 0xFFF0;
pub const SIM_OUTPUT_HOOK: u16 = 0xFFF3;
pub const SIM_CHAR_INPUT_HOOK: u16 = 0xFFF6;

pub fn execute(cpu: &mut Cpu, mem: &mut Mem) {
    if cpu.halted {
        return;
    }
    let opcode = fetch(cpu, mem);
    execute_opcode(cpu, mem, opcode);
}

pub fn run(cpu: &mut Cpu, mem: &mut Mem) -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_with_io(cpu, mem, stdin.lock(), stdout.lock())
}

pub fn run_with_io<R: BufRead, W: Write>(
    cpu: &mut Cpu,
    mem: &mut Mem,
    mut input: R,
    mut output: W,
) -> io::Result<()> {
    let mut pending_input = VecDeque::new();
    while !cpu.halted {
        match cpu.pc as u16 {
            SIM_INPUT_HOOK => {
                let Some(character) = next_input_character(&mut input, &mut pending_input)? else {
                    cpu.halted = true;
                    continue;
                };
                cpu.a = character;
                cpu.z = character == 0;
                cpu.n = character & 0x80 != 0;
                return_from_subroutine(cpu, mem);
                return_from_subroutine(cpu, mem);
                cpu.cycles += 12;
            }
            SIM_OUTPUT_HOOK => {
                output.write_all(&[cpu.a])?;
                output.flush()?;
                return_from_subroutine(cpu, mem);
                cpu.cycles += 6;
            }
            SIM_CHAR_INPUT_HOOK => {
                let Some(character) = next_input_character(&mut input, &mut pending_input)? else {
                    cpu.halted = true;
                    continue;
                };
                cpu.a = character;
                cpu.z = character == 0;
                cpu.n = character & 0x80 != 0;
                return_from_subroutine(cpu, mem);
                cpu.cycles += 6;
            }
            _ => execute(cpu, mem),
        }
    }
    Ok(())
}

fn next_input_character<R: BufRead>(
    input: &mut R,
    pending: &mut VecDeque<u8>,
) -> io::Result<Option<u8>> {
    if pending.is_empty() {
        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        while line.ends_with(['\n', '\r']) {
            line.pop();
        }
        pending.extend(line.bytes());
        pending.push_back(b'\r');
    }
    Ok(pending.pop_front())
}

fn return_from_subroutine(cpu: &mut Cpu, mem: &Mem) {
    cpu.sp = cpu.sp.wrapping_add(1);
    let low = mem.data[0x0100 | cpu.sp as usize] as u16;
    cpu.sp = cpu.sp.wrapping_add(1);
    let high = mem.data[0x0100 | cpu.sp as usize] as u16;
    cpu.pc = (high << 8 | low).wrapping_add(1) as u32;
}

pub fn fetch(cpu: &mut Cpu, mem: &Mem) -> u8 {
    let data = mem.data[cpu.pc as usize];
    cpu.pc = (cpu.pc + 1) & 0xFFFF;
    data
}
