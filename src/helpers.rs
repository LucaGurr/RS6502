use crate::define::{Cpu, Mem};

pub fn debug_state(cpu: &Cpu, mem: &Mem) {
    println!("\n{cpu:?}\n");
    let sp = cpu.sp as usize;
    let pc = cpu.pc as usize;
    let sp_end = (sp + 16).min(mem.data.len());
    let pc_end = (pc + 16).min(mem.data.len());
    println!("  SP[{:04X}]: {}", sp, hex_dump(&mem.data[sp..sp_end]));
    println!("  PC[{:04X}]: {}", pc, hex_dump(&mem.data[pc..pc_end]));
}

pub fn mem_print(mem: &Mem) {
    titlebar("Memory");
    let mut iter = mem.data.iter().peekable();
    while let Some(&byte) = iter.next() {
        print!("{:02X}", byte);
        let mut count = 1;
        while let Some(&next) = iter.peek() {
            if next == &byte {
                count += 1;
                iter.next();
            } else {
                break;
            }
        }
        if count > 1 {
            print!("[{}]...", count);
        }
        print!(" ");
    }
    println!();
}

pub fn hex_dump(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<_>>().join(" ")
}

pub fn titlebar(title: &str) {
    println!("\n{title}:\n");
}
