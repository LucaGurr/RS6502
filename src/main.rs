mod assembler;
mod define;
mod execute;
mod generic_assembler;
pub mod helpers;
mod instructions;
mod macro_assembler;

#[cfg(test)]
mod test;

use define::init;
use execute::run;
use helpers::*;
use std::io::{self, Write};

fn main() {
    print!("Assembly file to run: ");
    io::stdout().flush().expect("could not flush prompt");
    let mut path = String::new();
    if let Err(error) = io::stdin().read_line(&mut path) {
        eprintln!("Could not read file name: {error}");
        return;
    }
    let path = "assembly/".to_owned() + path.trim();
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Could not read {path}: {error}");
            return;
        }
    };
    let image = match assembler::assemble_image(&source) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("Assembly error: {error}");
            return;
        }
    };
    let (mut cpu, mut mem) = init();
    let begin = image.load_address as usize;
    let end = begin + image.bytes.len();
    if end > mem.data.len() {
        eprintln!("Assembly error: image exceeds the 6502 address space");
        return;
    }
    mem.data[begin..end].copy_from_slice(&image.bytes);
    mem.read_only = image.read_only;
    cpu.pc = image.entrypoint as u32;
    debug_state(&cpu, &mem);
    println!("Running {path}\n");
    if let Err(error) = run(&mut cpu, &mut mem) {
        eprintln!("Runtime I/O error: {error}");
        return;
    }
    println!(
        "\n\nStopped after {} cycles at ${:04X}.",
        cpu.cycles, cpu.pc
    );
    debug_state(&cpu, &mem);
}
