const MAX_DATA: usize = 65536;

#[derive(Default)]
struct Cpu {
    pc: u32,
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
            "Cpu {{ pc={:04x} sp={:02x} a={:02x} x={:02x} y={:02x} flags=[{}] }}",
            self.pc,
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
    println!("{cpu:?}");
    let sp = cpu.sp as usize;
    let pc = cpu.pc as usize;
    println!("  SP[{:04x}]: {}", sp, hex_dump(&mem.data[sp..sp + 16]));
    println!("  PC[{:04x}]: {}", pc, hex_dump(&mem.data[pc..pc + 16]));
}

fn hex_dump(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ")
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

fn main() {
    let (cpu,mut mem) = init();

    mem.data[0] = 0xA9; // LDA #imm (example)
    mem.data[1] = 0x42;

    debug_state(&cpu, &mem);
}
