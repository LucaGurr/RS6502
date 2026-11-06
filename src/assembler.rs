pub const LOAD_ADDRESS: u16 = 0x0600;

pub struct AssemblyImage {
    pub load_address: u16,
    pub entrypoint: u16,
    pub bytes: Vec<u8>,
    pub read_only: Option<(usize, usize)>,
}

pub fn assemble_image(source: &str) -> Result<AssemblyImage, String> {
    if source.lines().any(|line| {
        line.trim_start()
            .split_whitespace()
            .next()
            .is_some_and(|word| word.eq_ignore_ascii_case("TITLE"))
    }) {
        return crate::macro_assembler::assemble(source);
    }
    crate::generic_assembler::assemble(source)
}
