pub const MAX_DATA: usize = 65536;

#[derive(Default)]
pub struct Cpu {
    pub pc: u32,
    pub cycles: u64,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub c: bool,
    pub z: bool,
    pub i: bool,
    pub d: bool,
    pub b: bool,
    pub v: bool,
    pub n: bool,
    pub halted: bool,
}

pub struct Mem {
    pub data: [u8; MAX_DATA],
}

impl std::fmt::Debug for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cpu {{ pc={:04x} cycles={} sp={:02x} a={:02x} x={:02x} y={:02x} flags=[{}] }}",
            self.pc, self.cycles, self.sp, self.a, self.x, self.y,
            flags_str(self.c, self.z, self.i, self.d, self.b, self.v, self.n))
    }
}

impl std::fmt::Debug for Mem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Mem [{} bytes]", self.data.len())
    }
}

fn flags_str(c: bool, z: bool, i: bool, d: bool, b: bool, v: bool, n: bool) -> String {
    let mut s = String::new();
    if c { s.push('C'); }
    if z { s.push('Z'); }
    if i { s.push('I'); }
    if d { s.push('D'); }
    if b { s.push('B'); }
    if v { s.push('V'); }
    if n { s.push('N'); }
    if s.is_empty() { s.push_str("------"); }
    s
}

pub fn init() -> (Cpu, Mem) {
    let cpu = Cpu { pc: 0, sp: 255, ..Default::default() };
    let mem = Mem { data: [0u8; MAX_DATA] };
    (cpu, mem)
}

