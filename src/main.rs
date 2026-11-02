const MAX_DATA: usize = 65536;

#[derive(Default)]
struct Cpu {
    pc: u32,
    cycles: u64,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    c: bool,
    z: bool,
    i: bool,
    d: bool,
    b: bool,
    v: bool,
    n: bool,
}

struct Mem {
    data: [u8; MAX_DATA],
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Cpu {{ pc={:04x} cycles={} sp={:02x} a={:02x} x={:02x} y={:02x} flags=[{}] }}",
            self.pc,
            self.cycles,
            self.sp,
            self.a,
            self.x,
            self.y,
            flags_str(self.c, self.z, self.i, self.d, self.b, self.v, self.n),
        )
    }
}

impl std::fmt::Debug for Mem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Mem [{} bytes]", self.data.len())
    }
}

fn titlebar(title: &'static str){
    println!("");
    println!("{}:", title);
    println!("");
}

fn flags_str(c: bool, z: bool, i: bool, d: bool, b: bool, v: bool, n: bool) -> String {
    let mut s = String::new();
    if c {
        s.push('C');
    }
    if z {
        s.push('Z');
    }
    if i {
        s.push('I');
    }
    if d {
        s.push('D');
    }
    if b {
        s.push('B');
    }
    if v {
        s.push('V');
    }
    if n {
        s.push('N');
    }
    if s.is_empty() {
        s.push_str("------");
    }
    s
}

fn debug_state(cpu: &Cpu, mem: &Mem) {
    println!("");
    println!("{cpu:?}");    
    println!("");
    let sp = cpu.sp as usize;
    let pc = cpu.pc as usize;
    println!("  SP[{:04x}]: {}", sp, hex_dump(&mem.data[sp..sp + 16]));
    println!("  PC[{:04x}]: {}", pc, hex_dump(&mem.data[pc..pc + 16]));
}

fn hex_dump(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn init() -> (Cpu, Mem) {
    let cpu = Cpu {
        pc: 0,
        sp: 255,
        ..Default::default()
    };
    let mem = Mem {
        data: [0u8; MAX_DATA],
    };
    (cpu, mem)
}

fn mem_print(mem: &Mem) {
    let mut iter = mem.data.iter().peekable();
    
    titlebar("Memory");
    
    while let Some(&byte) = iter.next() {
        print!("{:02x}", byte);
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

fn lda_imm(cpu: &mut Cpu, mem: &Mem) {
    let val = fetch(cpu, mem);
    cpu.a = val;
    cpu.z = val == 0;
    cpu.n = val & 0x80 != 0;
    cpu.cycles += 2;
}

fn lda_zp(cpu: &mut Cpu, mem: &Mem) {
    let addr = fetch(cpu, mem) as usize;
    let val = mem.data[addr];

    cpu.a = val;
    cpu.z = val == 0;
    cpu.n = val & 0x80 != 0;
    cpu.cycles += 3;
}

fn lda_zpx(cpu: &mut Cpu, mem: &Mem) {
    let base = fetch(cpu, mem);
    let addr = base.wrapping_add(cpu.x) as usize;
    let val = mem.data[addr];

    cpu.a = val;
    cpu.z = val == 0;
    cpu.n = val & 0x80 != 0;
    cpu.cycles += 4;
}

fn execute(cpu: &mut Cpu, mem: &Mem) {

    let instruction = fetch(cpu, mem);
    match instruction {
        0xA9 => lda_imm(cpu, mem),      // 2 cycles
        0xA5 => lda_zp(cpu, mem),       // 3 cycles
        0xB5 => lda_zpx(cpu, mem),      // 4 cycles
        _ => panic!("The given instruction is currently not supported"),
    }
}

fn fetch(cpu: &mut Cpu, mem: &Mem) -> u8 {
    let data = mem.data[cpu.pc as usize];
    cpu.pc += 1;
    data
}
fn main() {
    let (mut cpu, mut mem) = init();

    mem.data[0] = 0xA9; // LDA #imm (example)
    mem.data[1] = 0x42;
    mem.data[255] = 0xFF;

    debug_state(&cpu, &mem);

    mem_print(&mem);
    
    execute(&mut cpu, &mem);
}

#[test]
fn test_lda_imm() {
    let (mut cpu, mut mem) = init();
    mem.data[0] = 0xA9;
    mem.data[1] = 0x42;

    execute(&mut cpu, &mem);

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

    execute(&mut cpu, &mem);

    assert_eq!(cpu.a, 0);
    assert!(cpu.z);
    assert_eq!(cpu.cycles, 2);
}

#[test]
fn test_lda_zp() {
    let (mut cpu, mut mem) = init();

    mem.data[0] = 0xA5; // LDA ZP
    mem.data[1] = 0x42; // Address
    mem.data[0x42] = 0x99; // Value at address

    execute(&mut cpu, &mem);

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
    mem.data[0] = 0xB5; // LDA $F0,X
    mem.data[1] = 0xF0;
    mem.data[0x10] = 0x80; // ($F0 + $20) wraps to $10

    execute(&mut cpu, &mem);

    assert_eq!(cpu.a, 0x80);
    assert!(cpu.n);
    assert_eq!(cpu.cycles, 4);
}