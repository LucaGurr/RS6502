mod assembler;
mod define;
mod execute;
mod instructions;

#[cfg(test)]
mod test;

use define::init;
use execute::run;
use std::io::{self, Write};

fn main() {
    print!("Assembly file to run: ");
    io::stdout().flush().expect("could not flush prompt");
    let mut path = String::new();
    if let Err(error) = io::stdin().read_line(&mut path) {
        eprintln!("Could not read file name: {error}");
        return;
    }
    let path = path.trim();
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Could not read {path}: {error}");
            return;
        }
    };
    let (start, program) = match assembler::assemble(&source) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("Assembly error: {error}");
            return;
        }
    };
    let (mut cpu, mut mem) = init();
    let begin = start as usize;
    mem.data[begin..begin + program.len()].copy_from_slice(&program);
    cpu.pc = start as u32;
    println!("Running {path}\n");
    run(&mut cpu, &mut mem);
    println!("\n\nStopped after {} cycles at ${:04X}.", cpu.cycles, cpu.pc);

    mem_print(&mem);
    
    execute(&mut cpu, &mem);
}
